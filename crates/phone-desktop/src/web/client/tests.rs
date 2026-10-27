use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{Mutex, atomic::AtomicBool},
};
struct Server {
    base: String,
    hits: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new() -> Self {
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        let base = format!("http://{}", socket.local_addr().unwrap());
        let hits = Arc::new(Mutex::new(vec![]));
        let stop = Arc::new(AtomicBool::new(false));
        let log = hits.clone();
        let done = stop.clone();
        let worker = thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                let Ok((mut stream, _)) = socket.accept() else {
                    thread::sleep(Duration::from_millis(4));
                    continue;
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut bytes = [0; 8192];
                let n = stream.read(&mut bytes).unwrap_or(0);
                let path = String::from_utf8_lossy(&bytes[..n])
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/")
                    .to_owned();
                log.lock().unwrap().push(path.clone());
                if path == "/redirect" {
                    let _=stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/blocked\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    continue;
                }
                let (kind, body) = match path.as_str() {
                    "/" | "/csp" => (
                        "text/html",
                        r#"<title>Before</title><main id=app>Loading</main><noscript>Static fallback</noscript><script src='/app.js'></script>"#,
                    ),
                    "/app.js" => (
                        "text/javascript",
                        r#"fetch('/data').then(r=>r.json()).then(d=>{document.title='JS ready';document.getElementById('app').innerHTML='<p>'+d.text+'</p><button type=button id=run>Run</button><a href=/next>Next</a>';document.getElementById('run').onclick=()=>document.getElementById('app').append(' clicked');});"#,
                    ),
                    "/data" => ("application/json", r#"{"text":"Fetched body"}"#),
                    "/loop" => ("text/html", "<p>Fallback</p><script>while(true){}</script>"),
                    _ => ("text/html", "<p>Next page</p>"),
                };
                let csp = if path == "/csp" {
                    "Content-Security-Policy: script-src 'none'\r\n"
                } else {
                    ""
                };
                let message = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\n{csp}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(message.as_bytes());
            }
        });
        Self {
            base,
            hits,
            stop,
            worker: Some(worker),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}
fn result(web: &Web, id: u64) -> Result<Page, String> {
    loop {
        let (i, p) = web.results.recv_timeout(Duration::from_secs(12)).unwrap();
        if i == id {
            return p;
        }
    }
}
fn text(p: &Page) -> String {
    p.elements
        .iter()
        .map(Element::label)
        .collect::<Vec<_>>()
        .join(" ")
}
#[test]
fn resource_schemes_credentials_and_cross_origin_are_rejected_before_io() {
    let mut net = Network::new(Arc::new(AtomicU64::new(1)), 1);
    for address in [
        "file:///etc/passwd",
        "data:text/javascript,code",
        "http://user:password@localhost/",
        "ftp://localhost/",
    ] {
        assert!(net.get(address, None).is_err());
    }
    assert!(
        net.get("https://other.invalid/", Some("https://example.org/"))
            .is_err()
    );
    assert_eq!(net.count, 0);
}
#[test]
#[ignore = "Requires localhost HTTP; no public network or audio"]
fn script_http_click_navigation_disable_and_cancel() {
    let server = Server::new();
    let web = Web::configured(true, true);
    web.load(1, server.base.clone()).unwrap();
    let page = result(&web, 1).unwrap();
    assert_eq!(
        page.title,
        "JS ready",
        "{page:?}; hits={:?}",
        server.hits.lock().unwrap()
    );
    assert!(text(&page).contains("Fetched body"));
    assert!(!text(&page).contains("Loading"));
    assert_eq!(&*server.hits.lock().unwrap(), &["/", "/app.js", "/data"]);
    let action = page
        .elements
        .iter()
        .find_map(|e| {
            if let Element::Link { label, url } = e {
                (label == "Run").then_some(url.clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut alien = page.clone();
    alien.url = "https://other.invalid/".into();
    web.load_page(2, action.clone(), Some(alien)).unwrap();
    assert!(result(&web, 2).is_err());
    web.load_page(3, action, Some(page)).unwrap();
    let page = result(&web, 3).unwrap();
    assert!(text(&page).contains("clicked"));
    let next = page
        .elements
        .iter()
        .find_map(|e| {
            if let Element::Link { label, url } = e {
                (label == "Next").then_some(url.clone())
            } else {
                None
            }
        })
        .unwrap();
    web.load_page(4, next, Some(page)).unwrap();
    assert!(text(&result(&web, 4).unwrap()).contains("Next page"));
    let disabled = Web::configured(true, false);
    let before = server.hits.lock().unwrap().len();
    disabled.load(1, server.base.clone()).unwrap();
    let page = result(&disabled, 1).unwrap();
    assert_eq!(page.title, "Before");
    assert!(text(&page).contains("Static fallback"));
    assert_eq!(server.hits.lock().unwrap().len(), before + 1);
    let mut net = Network::new(Arc::new(AtomicU64::new(1)), 1);
    let error = net
        .get(&format!("{}/redirect", server.base), Some(&server.base))
        .err()
        .unwrap();
    assert!(error.contains("Cross-origin"), "{error}");
    assert_eq!(net.count, 1);
    web.load(5, format!("{}/loop", server.base)).unwrap();
    thread::sleep(Duration::from_millis(40));
    let start = Instant::now();
    web.cancel();
    web.load(6, format!("{}/next", server.base)).unwrap();
    assert!(text(&result(&web, 6).unwrap()).contains("Next page"));
    assert!(start.elapsed() < Duration::from_secs(3));
    let before = server.hits.lock().unwrap().len();
    web.load(7, format!("{}/csp", server.base)).unwrap();
    assert_eq!(result(&web, 7).unwrap().title, "Before");
    assert_eq!(server.hits.lock().unwrap().len(), before + 1);
}
#[test]
#[ignore = "Explicit public HTTP(S) readability smoke; not a website compatibility survey"]
fn public_pages() {
    let web = Web::configured(true, true);
    for (i, url) in [
        "https://example.com/",
        "https://www.rust-lang.org/",
        "https://news.ycombinator.com/",
    ]
    .into_iter()
    .enumerate()
    {
        let start = Instant::now();
        let id = i as u64 + 1;
        web.load(id, url.into()).unwrap();
        let page = result(&web, id).unwrap_or_else(|e| panic!("{url}: {e}"));
        eprintln!(
            "PUBLIC {url} -> {} | {} elements | {:?}",
            page.title,
            page.elements.len(),
            start.elapsed()
        );
        assert!(!page.elements.is_empty());
    }
}
