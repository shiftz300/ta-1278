//! Bounded HTTP and script work. Nothing runs on the window thread.
use super::{demo, parse, script::Session};
use phone_core::browser::{Element, Page};
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use url::Url;
#[cfg(test)]
mod tests;

pub(super) struct Resource {
    pub url: String,
    pub body: String,
    pub kind: String,
    pub status: u16,
    pub csp: bool,
}
pub(super) struct Network {
    agent: ureq::Agent,
    latest: Arc<AtomicU64>,
    id: u64,
    until: Instant,
    count: usize,
    bytes: usize,
}
impl Network {
    pub(super) fn new(latest: Arc<AtomicU64>, id: u64) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(2)))
            .max_redirects(0)
            .http_status_as_error(false)
            .build();
        Self {
            agent: config.into(),
            latest,
            id,
            until: Instant::now() + Duration::from_secs(10),
            count: 0,
            bytes: 0,
        }
    }
    pub fn active(&self) -> bool {
        Instant::now() < self.until && self.latest.load(Ordering::Relaxed) == self.id
    }
    pub fn get(&mut self, address: &str, origin: Option<&str>) -> Result<Resource, String> {
        if address.len() > 8192 {
            return Err("Resource URL limit reached".into());
        }
        let mut url = Url::parse(address).map_err(|e| e.to_string())?;
        for _ in 0..6 {
            if url.as_str().len() > 8192 {
                return Err("Resource URL limit reached".into());
            }
            if !self.active() || self.count >= 16 || self.bytes >= 4 * 1024 * 1024 {
                return Err("Page resource/time limit reached".into());
            }
            if !matches!(url.scheme(), "http" | "https")
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err("Only HTTP(S) resources without URL credentials are supported".into());
            }
            if let Some(base) = origin
                && Url::parse(base).map_err(|e| e.to_string())?.origin() != url.origin()
            {
                return Err("Cross-origin fetch is not supported".into());
            }
            let timeout = self
                .until
                .saturating_duration_since(Instant::now())
                .min(Duration::from_secs(if self.count == 0 { 8 } else { 3 }));
            self.count += 1;
            let mut response = self
                .agent
                .get(url.as_str())
                .config()
                .timeout_global(Some(timeout))
                .build()
                .call()
                .map_err(|e| e.to_string())?;
            let status = response.status().as_u16();
            if (300..400).contains(&status)
                && let Some(next) = response.headers().get("location")
            {
                url = url
                    .join(next.to_str().map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
                continue;
            }
            let kind = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("text/html")
                .to_lowercase();
            let body = response
                .body_mut()
                .with_config()
                .limit((2 * 1024 * 1024).min(4 * 1024 * 1024 - self.bytes) as u64)
                .read_to_string()
                .map_err(|e| e.to_string())?;
            let csp = response.headers().contains_key("content-security-policy");
            self.bytes += body.len();
            return Ok(Resource {
                url: url.to_string(),
                body,
                kind,
                status,
                csp,
            });
        }
        Err("Too many redirects".into())
    }
}
struct Job {
    id: u64,
    url: String,
    page: Option<Page>,
}
pub struct Web {
    jobs: mpsc::SyncSender<Job>,
    network: bool,
    latest: Arc<AtomicU64>,
    pub results: mpsc::Receiver<(u64, Result<Page, String>)>,
}
impl Web {
    #[cfg(test)]
    pub fn new(network: bool) -> Self {
        Self::configured(network, true)
    }
    pub fn configured(network: bool, javascript: bool) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Job>(2);
        let (rt, rr) = mpsc::channel();
        let latest = Arc::new(AtomicU64::new(0));
        let current = latest.clone();
        thread::spawn(move || {
            let mut sessions: VecDeque<Session> = VecDeque::new();
            let mut serial = 0;
            while let Ok(job) = rx.recv() {
                if current.load(Ordering::Relaxed) != job.id {
                    continue;
                }
                let net = Rc::new(RefCell::new(Network::new(current.clone(), job.id)));
                let result = load(&job, net, &mut sessions, &mut serial, javascript);
                if rt.send((job.id, result)).is_err() {
                    break;
                }
            }
        });
        Self {
            network,
            jobs: tx,
            latest,
            results: rr,
        }
    }
    #[cfg(test)]
    pub fn load(&self, id: u64, url: String) -> Result<(), String> {
        self.load_page(id, url, None)
    }
    pub fn load_page(&self, id: u64, url: String, page: Option<Page>) -> Result<(), String> {
        if !self.network
            && !matches!(url.as_str(), "about:demo")
            && !url.starts_with("about:result?")
        {
            return Err("网络功能已停用。".into());
        }
        self.latest.store(id, Ordering::Relaxed);
        self.jobs
            .try_send(Job { id, url, page })
            .map_err(|_| "Browser is busy; stop and try again shortly".to_owned())?;
        Ok(())
    }
    pub fn cancel(&self) {
        self.latest.store(u64::MAX, Ordering::Relaxed);
    }
}
impl Drop for Web {
    fn drop(&mut self) {
        self.cancel();
    }
}
fn load(
    job: &Job,
    net: Rc<RefCell<Network>>,
    sessions: &mut VecDeque<Session>,
    serial: &mut u64,
    javascript: bool,
) -> Result<Page, String> {
    let mut address = job.url.clone();
    if address.starts_with("about:script/") {
        if !javascript {
            return Err("JavaScript is disabled".into());
        }
        let parts: Vec<_> = address
            .trim_start_matches("about:script/")
            .split('?')
            .next()
            .unwrap_or("")
            .split('/')
            .collect();
        let session = parts
            .first()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or("Invalid script action")?;
        let node = parts
            .get(1)
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or("Invalid script target")?;
        let s = sessions
            .iter_mut()
            .find(|s| s.id == session)
            .ok_or("此页面脚本已过期，请刷新后重试。")?;
        let page = job
            .page
            .as_ref()
            .filter(|p| p.url == s.url)
            .ok_or("Script action belongs to another page")?;
        s.activate(net.clone());
        let (page, next) = s.click(node, page);
        if let Some(next) = next {
            address = next;
        } else {
            return Ok(page);
        }
    }
    for _ in 0..5 {
        if address == "about:demo" {
            return Ok(demo());
        }
        let url = Url::parse(&address).map_err(|e| e.to_string())?;
        if url.scheme() == "about" && url.path() == "result" {
            return Ok(Page {
                flow: vec![],
                url: address,
                title: "Local form result".into(),
                elements: url
                    .query_pairs()
                    .map(|(k, v)| Element::Text(format!("{k} = {v}")))
                    .collect(),
                forms: vec![],
            });
        }
        let doc = net.borrow_mut().get(&address, None)?;
        if doc.body.bytes().filter(|b| *b == b'<').count() > 12000 {
            return Err("HTML node limit reached".into());
        }
        if doc.kind.starts_with("text/plain") || doc.kind.starts_with("application/json") {
            return Ok(Page {
                flow: vec![],
                url: doc.url,
                title: "Text document".into(),
                elements: doc
                    .body
                    .lines()
                    .take(1200)
                    .map(|s| Element::Text(s.into()))
                    .collect(),
                forms: vec![],
            });
        }
        if !(doc.kind.starts_with("text/") || doc.kind.starts_with("application/xhtml+xml")) {
            return Err(format!("Unsupported content: {}", doc.kind));
        }
        if !javascript || doc.csp || !Session::allowed(&doc.body) || !Session::needed(&doc.body) {
            return Ok(parse(&doc.url, &doc.body));
        }
        *serial += 1;
        let mut session = Session::new(*serial, &doc.url, &doc.body, net.clone())?;
        let (page, next) = session.snapshot();
        sessions.push_back(session);
        while sessions.len() > 2 {
            sessions.pop_front();
        }
        if let Some(next) = next {
            address = next;
        } else {
            return Ok(page);
        }
    }
    Err("Too many script navigations".into())
}
