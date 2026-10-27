//! Limited page contexts. QuickJS has no std/os module or host file/service bindings.
use super::{client::Network, parsed};
use phone_core::browser::{Element, Page};
use rquickjs::{Context, Function, Runtime};
use scraper::{ElementRef, Html, Node, Selector};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use url::Url;
type Link = Rc<RefCell<Network>>;
type Active = Rc<RefCell<Link>>;
#[cfg(test)]
mod tests;

pub(super) struct Session {
    pub id: u64,
    pub url: String,
    context: Context,
    runtime: Runtime,
    network: Active,
    deadline: Rc<RefCell<Instant>>,
    original: String,
    controls: Vec<(usize, u64)>,
    errors: Vec<String>,
    scripts: usize,
}
fn tree(html: &str, fragment: bool) -> String {
    if html.len() > 2 * 1024 * 1024 || html.bytes().filter(|b| *b == b'<').count() > 6000 {
        return "[]".into();
    }
    fn element(e: ElementRef, depth: usize, count: &mut usize) -> Value {
        *count += 1;
        let children = if depth >= 64 {
            vec![]
        } else {
            e.children()
                .filter_map(|n| {
                    if *count >= 6000 {
                        return None;
                    }
                    if let Some(t) = n.value().as_text() {
                        *count += 1;
                        Some(json!({"text":t.to_string()}))
                    } else {
                        ElementRef::wrap(n).map(|child| element(child, depth + 1, count))
                    }
                })
                .collect()
        };
        let attrs: serde_json::Map<String, Value> = e
            .value()
            .attrs()
            .filter(|(k, _)| !k.starts_with("data-phone-"))
            .map(|(k, v)| (k.into(), json!(v)))
            .collect();
        json!({"tag":e.value().name(),"attrs":attrs,"children":children})
    }
    let doc = if fragment {
        Html::parse_fragment(html)
    } else {
        Html::parse_document(html)
    };
    let mut count = 0;
    let values: Vec<_> = if fragment {
        doc.root_element()
            .children()
            .filter_map(|n| match n.value() {
                Node::Text(t) => Some(json!({"text":t.to_string()})),
                _ => ElementRef::wrap(n).map(|e| element(e, 0, &mut count)),
            })
            .take(6000)
            .collect()
    } else {
        vec![element(doc.root_element(), 0, &mut count)]
    };
    serde_json::to_string(&values).unwrap()
}
impl Session {
    pub fn allowed(html: &str) -> bool {
        let doc = Html::parse_document(html);
        !doc.select(&Selector::parse("meta[http-equiv]").unwrap())
            .any(|e| {
                e.value()
                    .attr("http-equiv")
                    .is_some_and(|s| s.eq_ignore_ascii_case("content-security-policy"))
            })
    }
    pub fn needed(html: &str) -> bool {
        let doc = Html::parse_document(html);
        doc.select(
            &Selector::parse("script,[onclick],[onload],[onsubmit],a[href^='javascript:']")
                .unwrap(),
        )
        .next()
        .is_some()
    }
    pub fn new(id: u64, url: &str, html: &str, network: Link) -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        runtime.set_memory_limit(32 * 1024 * 1024);
        runtime.set_max_stack_size(512 * 1024);
        let deadline = Rc::new(RefCell::new(Instant::now() + Duration::from_millis(250)));
        let active = Rc::new(RefCell::new(network));
        let clock = deadline.clone();
        let current = active.clone();
        runtime.set_interrupt_handler(Some(Box::new(move || {
            Instant::now() > *clock.borrow() || !current.borrow().borrow().active()
        })));
        let context = Context::full(&runtime).map_err(|e| e.to_string())?;
        let endpoint = active.clone();
        let request_clock = deadline.clone();
        let origin = url.to_owned();
        context.with(|ctx| -> rquickjs::Result<()> {
            ctx.globals().set("__parse", Function::new(ctx.clone(), |html: String, fragment: bool| -> String {
                if html.len() > 2 * 1024 * 1024 { "[]".into() } else { tree(&html,fragment) }
            })?)?;
            ctx.globals().set("__resolve", Function::new(ctx.clone(), |base: String, value: String, meta: rquickjs::prelude::Opt<bool>| -> String {
                let Ok(url) = Url::parse(&base).and_then(|u|u.join(&value)) else { return String::new(); };
                if meta.0 == Some(true) {
                    json!({"origin":url.origin().ascii_serialization(),"protocol":format!("{}:",url.scheme()),"host":url.authority(),"hostname":url.host_str().unwrap_or(""),"pathname":url.path(),"search":url.query().map(|q|format!("?{q}")).unwrap_or_default(),"hash":url.fragment().map(|q|format!("#{q}")).unwrap_or_default()}).to_string()
                } else { url.to_string() }
            })?)?;
            ctx.globals().set("__resource", Function::new(ctx.clone(), move |url: String, _kind: String| -> String {
                        let result = endpoint.borrow().borrow_mut().get(&url, Some(&origin));
                        *request_clock.borrow_mut() = Instant::now() + Duration::from_millis(250);
                        match result {
                    Ok(r) => json!({"url":r.url,"status":r.status,"body":r.body}).to_string(),
                    Err(error) => json!({"error":error}).to_string(),
                }
            })?)?;
            Ok(())
        }).map_err(|e|e.to_string())?;
        let mut s = Self {
            id,
            url: url.into(),
            context,
            runtime,
            network: active,
            deadline,
            original: html.into(),
            controls: vec![],
            errors: vec![],
            scripts: 0,
        };
        s.eval(include_str!("dom.js"));
        s.eval(&format!(
            "__phone.init({}, {}, {})",
            tree(html, false),
            id,
            json!(url)
        ));
        s.scripts();
        s.eval("__phone.ready()");
        s.settle();
        Ok(s)
    }
    pub fn activate(&mut self, network: Link) {
        *self.network.borrow_mut() = network;
        self.errors.clear();
    }
    fn limit(&self) {
        *self.deadline.borrow_mut() = Instant::now() + Duration::from_millis(250);
    }
    fn error(&mut self, error: String) {
        if self.errors.len() < 4 {
            self.errors.push(error.chars().take(160).collect());
        }
    }
    fn eval(&mut self, code: &str) {
        self.limit();
        let result = self.context.with(|ctx| {
            ctx.eval::<(), _>(code).map_err(|e| {
                let value = ctx.catch();
                value
                    .as_exception()
                    .and_then(|e| e.message())
                    .unwrap_or_else(|| e.to_string())
            })
        });
        if let Err(error) = result {
            self.error(error);
        }
    }
    fn value(&mut self, code: &str) -> Option<Value> {
        self.limit();
        let result = self.context.with(|ctx| {
            ctx.eval::<String, _>(code).map_err(|e| {
                let _ = ctx.catch();
                e.to_string()
            })
        });
        match result {
            Ok(s) => serde_json::from_str(&s).ok(),
            Err(e) => {
                self.error(e);
                None
            }
        }
    }
    fn scripts(&mut self) {
        while self.scripts < 32 && self.network.borrow().borrow().active() {
            let Some(Value::Array(scripts)) = self.value("__phone.scripts()") else {
                break;
            };
            if scripts.is_empty() {
                break;
            }
            for script in scripts {
                self.scripts += 1;
                let kind = script["type"].as_str().unwrap_or("");
                if !matches!(
                    kind,
                    "" | "text/javascript" | "application/javascript" | "text/ecmascript"
                ) {
                    if kind == "module" {
                        self.error("ES module scripts are not supported".into());
                    }
                    continue;
                }
                let code = if let Some(src) = script["src"].as_str() {
                    let loaded = Url::parse(&self.url)
                        .and_then(|u| u.join(src))
                        .map_err(|e| e.to_string())
                        .and_then(|u| {
                            if self.url.starts_with("https:") && u.scheme() != "https" {
                                return Err("Mixed-content script blocked".into());
                            }
                            self.network.borrow().borrow_mut().get(u.as_str(), None)
                        });
                    match loaded {
                        Ok(r)
                            if (200..300).contains(&r.status)
                                && matches!(
                                    r.kind.split(';').next().unwrap_or("").trim(),
                                    "text/javascript"
                                        | "application/javascript"
                                        | "application/ecmascript"
                                        | "text/ecmascript"
                                ) =>
                        {
                            r.body
                        }
                        Ok(r) => {
                            self.error(format!("Script HTTP {}", r.status));
                            continue;
                        }
                        Err(e) => {
                            self.error(e);
                            continue;
                        }
                    }
                } else {
                    script["code"].as_str().unwrap_or("").to_owned()
                };
                self.eval(&format!("__phone.current({})", script["id"]));
                self.eval(&code);
                self.jobs();
                if !self.network.borrow().borrow().active() {
                    break;
                }
            }
        }
    }
    fn jobs(&mut self) {
        self.limit();
        for _ in 0..128 {
            match self.runtime.execute_pending_job() {
                Ok(false) => return,
                Ok(true) => {}
                Err(e) => {
                    self.error(e.to_string());
                    return;
                }
            }
        }
        if self.runtime.is_job_pending() {
            self.error("Promise job limit reached".into());
        }
    }
    fn settle(&mut self) {
        let until = Instant::now() + Duration::from_millis(180);
        for _ in 0..20 {
            if !self.network.borrow().borrow().active() || Instant::now() >= until {
                break;
            }
            self.eval("__phone.timers()");
            self.jobs();
            self.scripts();
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    pub fn click(&mut self, node: u64, page: &Page) -> (Page, Option<String>) {
        let originals: std::collections::HashMap<_, _> = self
            .controls
            .iter()
            .filter_map(|(i, id)| page.elements.get(*i).map(|e| (*id, e.clone())))
            .collect();
        let values: Vec<_> = self
            .controls
            .iter()
            .filter_map(|(i, id)| {
                Some(match page.elements.get(*i)? {
                    Element::Field { value, .. } => json!({"id":id,"value":value}),
                    Element::Check { checked, .. } => json!({"id":id,"checked":checked}),
                    Element::Select { selected, .. } => json!({"id":id,"selected":selected}),
                    _ => return None,
                })
            })
            .collect();
        self.eval(&format!("__phone.sync({})", json!(values)));
        self.eval(&format!("__phone.click({node})"));
        self.jobs();
        self.settle();
        let (mut page, next) = self.snapshot();
        for (index, id) in &self.controls {
            match (page.elements.get_mut(*index), originals.get(id)) {
                (
                    Some(Element::Field { initial, .. }),
                    Some(Element::Field { initial: old, .. }),
                ) => *initial = old.clone(),
                (
                    Some(Element::Check { initial, .. }),
                    Some(Element::Check { initial: old, .. }),
                ) => *initial = *old,
                (
                    Some(Element::Select { initial, .. }),
                    Some(Element::Select { initial: old, .. }),
                ) if initial.len() == old.len() => *initial = old.clone(),
                _ => {}
            }
        }
        (page, next)
    }
    pub fn snapshot(&mut self) -> (Page, Option<String>) {
        let value = self.value("__phone.snapshot()");
        let html = value
            .as_ref()
            .and_then(|v| v["html"].as_str())
            .unwrap_or(&self.original);
        let (mut page, mut controls) = parsed(&self.url, html);
        if page.elements.is_empty() && !self.errors.is_empty() {
            (page, controls) = parsed(&self.url, &self.original);
        }
        self.controls = controls;
        if !self.errors.is_empty() {
            page.elements.push(Element::Text(format!(
                "[脚本受限：{}]",
                self.errors.join("; ")
            )));
            page.flow
                .push(phone_core::browser::flow::Flow::Break { gap: 8 });
            page.flow.push(phone_core::browser::flow::Flow::Text {
                text: "部分脚本未完成，显示已加载内容。".into(),
                heading: false,
                bold: false,
            });
        }
        if page.elements.is_empty() {
            page.elements.push(Element::Text(
                "此网页没有可显示的内容，可能需要未支持的浏览器功能。".into(),
            ));
        }
        let next = value.and_then(|v| v["next"].as_str().map(str::to_owned));
        (page, next)
    }
}
