mod flow;
use phone_core::browser::{Element, Form, Page};
use scraper::{ElementRef, Html};
use std::{sync::mpsc, thread, time::Duration};
use ureq::ResponseExt;
use url::Url;
pub const DEMO: &str = r#"<html><head><title>Browser demo</title></head><body><h1>TA-1278 browser</h1><p>A small text browser. Up/down selects a line. Center opens a link or field.</p><a href="https://example.com">Open example.com</a><form action="about:result" method="get"><p>Local form: no network request</p><input name="message" value="Hello" maxlength="80"><input type="checkbox" name="updates" value="yes"> Updates<select name="color"><option value="blue">Blue</option><option value="red">Red</option></select><select name="music" multiple><option value="one">Track one</option><option value="two">Track two</option></select><button type="submit">Test form</button><button type="reset">Reset</button></form><p>No JavaScript, CSS layout or WML. HTTP(S) only.</p></body></html>"#;
pub fn demo() -> Page {
    parse("about:demo", DEMO)
}
fn text(e: ElementRef) -> String {
    e.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn resolve(base: &str, value: &str) -> String {
    Url::parse(base)
        .and_then(|u| u.join(value))
        .map(|u| u.to_string())
        .unwrap_or_else(|_| value.into())
}
pub fn parse(base: &str, html: &str) -> Page {
    let doc = Html::parse_document(html);
    let title = doc
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
        .map(text)
        .unwrap_or_else(|| base.into());
    let mut page = Page {
        flow: vec![],
        url: base.into(),
        title,
        elements: vec![],
        forms: vec![],
    };
    let body = doc
        .select(&scraper::Selector::parse("body").unwrap())
        .next()
        .unwrap_or_else(|| doc.root_element());
    let mut controls = vec![];
    walk(body, &mut page, None, &mut String::new(), 0, &mut controls);
    page.flow = flow::build(body, &controls);
    if page.elements.is_empty() {
        page.elements.push(Element::Text("Empty page".into()));
    }
    page
}
fn flush(p: &mut Page, s: &mut String) {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if !t.is_empty() {
        let chars: Vec<_> = t.chars().collect();
        for chunk in chars.chunks(90) {
            if p.elements.len() >= 1200 {
                break;
            }
            p.elements.push(Element::Text(chunk.iter().collect()));
        }
    }
    s.clear();
}
fn label_text(e: ElementRef) -> String {
    fn visit(e: ElementRef, out: &mut String) {
        for node in e.children() {
            if let Some(t) = node.value().as_text() {
                out.push_str(t);
                out.push(' ');
            } else if let Some(child) = ElementRef::wrap(node)
                && !matches!(
                    child.value().name(),
                    "input" | "select" | "textarea" | "button"
                )
            {
                visit(child, out);
            }
        }
    }
    let mut s = String::new();
    visit(e, &mut s);
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn associated_label(e: ElementRef) -> Option<String> {
    if let Some(label) = e
        .ancestors()
        .filter_map(ElementRef::wrap)
        .find(|a| a.value().name() == "label")
    {
        return Some(label_text(label));
    }
    let id = e.value().attr("id")?;
    let root = e.ancestors().filter_map(ElementRef::wrap).last()?;
    root.select(&scraper::Selector::parse("label").unwrap())
        .find(|l| l.value().attr("for") == Some(id))
        .map(label_text)
}
fn walk<'a>(
    e: ElementRef<'a>,
    p: &mut Page,
    form: Option<usize>,
    buffer: &mut String,
    depth: usize,
    controls: &mut Vec<(ElementRef<'a>, usize)>,
) {
    if p.elements.len() >= 1200 || depth >= 128 {
        return;
    }
    let v = e.value();
    let tag = v.name();
    if matches!(tag, "script" | "style" | "head" | "template" | "noscript")
        || v.attr("hidden").is_some()
    {
        return;
    }
    if matches!(tag, "input" | "textarea" | "select" | "button") && v.attr("disabled").is_some() {
        return;
    }
    let block = matches!(
        tag,
        "p" | "div" | "h1" | "h2" | "h3" | "li" | "br" | "form" | "section" | "table" | "tr"
    );
    if block {
        flush(p, buffer);
    }
    let mut form = form;
    if tag == "form" {
        form = Some(p.forms.len());
        p.forms.push(Form {
            action: resolve(&p.url, v.attr("action").unwrap_or(&p.url)),
            method: v.attr("method").unwrap_or("get").to_lowercase(),
            hidden: vec![],
        });
    }
    let name = v.attr("name").unwrap_or("").to_owned();
    let label = v
        .attr("aria-label")
        .map(str::to_owned)
        .or_else(|| associated_label(e))
        .or_else(|| v.attr("placeholder").map(str::to_owned))
        .or_else(|| v.attr("name").map(str::to_owned))
        .unwrap_or_default();
    let f = form.unwrap_or(usize::MAX);
    let element = match tag {
        "a" if v.attr("href").is_some() => Some(Element::Link {
            label: text(e),
            url: resolve(&p.url, v.attr("href").unwrap()),
        }),
        "input" if v.attr("disabled").is_none() => {
            let value = v.attr("value").unwrap_or("").to_owned();
            match v.attr("type").unwrap_or("text") {
                "checkbox" | "radio" => Some(Element::Check {
                    radio: v.attr("type") == Some("radio"),
                    label,
                    name,
                    checked: v.attr("checked").is_some(),
                    initial: v.attr("checked").is_some(),
                    value: if value.is_empty() { "on".into() } else { value },
                    form: f,
                }),
                "submit" => Some(Element::Submit {
                    label: if value.is_empty() {
                        "Submit".into()
                    } else {
                        value.clone()
                    },
                    name,
                    value,
                    form: f,
                }),
                "reset" => Some(Element::Reset { form: f }),
                "hidden" => {
                    if let Some(form) = p.forms.get_mut(f) {
                        form.hidden.push((name, value));
                    }
                    None
                }
                "text" | "search" | "email" | "url" | "tel" | "number" | "password" | "time" => {
                    Some(Element::Field {
                        label,
                        name,
                        value: value.clone(),
                        initial: value,
                        limit: v
                            .attr("maxlength")
                            .and_then(|s| s.parse::<usize>().ok())
                            .filter(|n| *n > 0)
                            .unwrap_or(1024)
                            .min(1024),
                        password: v.attr("type") == Some("password"),
                        read_only: v.attr("readonly").is_some(),
                        form: f,
                    })
                }
                other => Some(Element::Text(format!("[Unsupported input: {other}]"))),
            }
        }
        "textarea" => {
            let value = e.text().collect::<String>();
            Some(Element::Field {
                label,
                name,
                value: value.clone(),
                initial: value,
                limit: v
                    .attr("maxlength")
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|n| *n > 0)
                    .unwrap_or(1024)
                    .min(1024),
                password: false,
                read_only: v.attr("readonly").is_some(),
                form: f,
            })
        }
        "select" => {
            let options: Vec<_> = e
                .select(&scraper::Selector::parse("option").unwrap())
                .collect();
            let multiple = v.attr("multiple").is_some();
            let mut selected: Vec<_> = options
                .iter()
                .map(|o| o.value().attr("selected").is_some())
                .collect();
            if !multiple && !selected.is_empty() && !selected.iter().any(|x| *x) {
                selected[0] = true;
            }
            Some(Element::Select {
                label,
                name,
                options: options
                    .iter()
                    .map(|o| {
                        let t = text(*o);
                        let value = o.value().attr("value").unwrap_or(&t).to_owned();
                        (t, value)
                    })
                    .collect(),
                initial: selected.clone(),
                selected,
                multiple,
                form: f,
            })
        }
        "button" => Some(if v.attr("type") == Some("reset") {
            Element::Reset { form: f }
        } else {
            Element::Submit {
                label: text(e),
                name,
                value: v.attr("value").unwrap_or("").into(),
                form: f,
            }
        }),
        _ => None,
    };
    if let Some(el) = element {
        flush(p, buffer);
        if !matches!(el, Element::Text(_)) {
            controls.push((e, p.elements.len()));
        }
        p.elements.push(el);
        return;
    }
    if matches!(tag, "input" | "select" | "textarea" | "button") {
        return;
    }
    for child in e.children() {
        if let Some(t) = child.value().as_text() {
            if tag == "label" {
                continue;
            }
            buffer.push_str(t);
            buffer.push(' ');
        } else if let Some(c) = ElementRef::wrap(child) {
            walk(c, p, form, buffer, depth + 1, controls);
        }
    }
    if block || tag == "body" {
        flush(p, buffer);
    }
}
pub fn submit(page: &Page, form: usize, values: Vec<(String, String)>) -> Result<String, String> {
    let f = page.forms.get(form).ok_or("Field is outside a form")?;
    if f.method != "get" {
        return Err("Only GET forms are supported".into());
    }
    let mut url = Url::parse(&f.action).map_err(|e| e.to_string())?;
    url.set_query(None);
    url.query_pairs_mut().extend_pairs(values);
    Ok(url.into())
}
pub struct Web {
    jobs: mpsc::SyncSender<(u64, String)>,
    network: bool,
    pub results: mpsc::Receiver<(u64, Result<Page, String>)>,
}
impl Web {
    pub fn new(network: bool) -> Self {
        let (tx, rx) = mpsc::sync_channel::<(u64, String)>(2);
        let (rt, rr) = mpsc::channel();
        thread::spawn(move || {
            let config = ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(12)))
                .build();
            let agent: ureq::Agent = config.into();
            while let Ok((id, url)) = rx.recv() {
                let result = fetch(&agent, &url);
                if rt.send((id, result)).is_err() {
                    break;
                }
            }
        });
        Self {
            network,
            jobs: tx,
            results: rr,
        }
    }
    pub fn load(&self, id: u64, url: String) -> Result<(), String> {
        if !self.network && !url.starts_with("about:") {
            return Err("网络功能已停用。".into());
        }
        self.jobs
            .try_send((id, url))
            .map_err(|_| "Browser is busy; stop and try again shortly".into())
    }
}
fn fetch(agent: &ureq::Agent, address: &str) -> Result<Page, String> {
    let url = Url::parse(address).map_err(|e| e.to_string())?;
    if address == "about:demo" {
        return Ok(demo());
    }
    if url.scheme() == "about" && url.path() == "result" {
        return Ok(Page {
            flow: vec![],
            url: address.into(),
            title: "Local form result".into(),
            elements: url
                .query_pairs()
                .map(|(k, v)| Element::Text(format!("{k} = {v}")))
                .collect(),
            forms: vec![],
        });
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Only http://, https:// and about:demo are supported".into());
    }
    let mut response = agent.get(url.as_str()).call().map_err(|e| e.to_string())?;
    let final_url = response.get_uri().to_string();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_owned();
    if !(content_type.starts_with("text/") || content_type.starts_with("application/xhtml+xml")) {
        return Err(format!("Unsupported content: {content_type}"));
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(2 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    if content_type.starts_with("text/plain") {
        Ok(Page {
            flow: vec![],
            url: final_url.clone(),
            title: "Text document".into(),
            elements: body
                .lines()
                .take(1200)
                .map(|s| Element::Text(s.into()))
                .collect(),
            forms: vec![],
        })
    } else {
        Ok(parse(&final_url, &body))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_has_real_fields_and_get_submission() {
        let p = demo();
        assert_eq!(p.forms.len(), 1);
        assert!(
            p.elements
                .iter()
                .any(|e| matches!(e, Element::Select { multiple: true, .. }))
        );
        assert_eq!(
            submit(&p, 0, vec![("message".into(), "中 &".into())]).unwrap(),
            "about:result?message=%E4%B8%AD+%26"
        );
    }
    #[test]
    fn excludes_script_and_limits() {
        let p = parse(
            "https://example.com/path/",
            "<script>secret()</script><p>hello <a href='next'>world</a> end</p><input maxlength='99999'>",
        );
        assert!(!format!("{:?}", p).contains("secret()"));
        assert!(
            p.elements
                .iter()
                .any(|e| matches!(e,Element::Link{url,..} if url=="https://example.com/path/next"))
        );
        assert!(
            p.elements
                .iter()
                .any(|e| matches!(e, Element::Field { limit: 1024, .. }))
        );
    }
    #[test]
    fn radios_are_exclusive_only_inside_their_group_and_reset() {
        let page = parse(
            "https://example.com/",
            "<form><input type=radio name=size value=s checked><input type=radio name=size value=m><input type=radio name=other value=x checked><input type=checkbox name=topping value=a><input type=checkbox name=topping value=b></form>",
        );
        let mut b = phone_core::browser::Browser::new(page);
        b.toggle_check(1);
        b.toggle_check(3);
        b.toggle_check(4);
        assert_eq!(
            b.form_values(0, None),
            vec![
                ("size".into(), "m".into()),
                ("other".into(), "x".into()),
                ("topping".into(), "a".into()),
                ("topping".into(), "b".into())
            ]
        );
        b.reset(0);
        assert_eq!(
            b.form_values(0, None),
            vec![("size".into(), "s".into()), ("other".into(), "x".into())]
        );
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;
    #[test]
    fn html_labels_are_not_duplicated_and_disabled_controls_are_skipped() {
        let p = parse(
            "https://example.com/",
            "<label>Customer name: <input name=custname></label><label><input type=radio name=size>Medium</label><label for=note>Note</label><textarea id=note></textarea><input disabled><textarea disabled>skip</textarea>",
        );
        assert_eq!(p.elements.len(), 3);
        assert!(matches!(&p.elements[0],Element::Field{label,..} if label=="Customer name:"));
        assert!(matches!(&p.elements[1],Element::Check{label,..} if label=="Medium"));
        assert!(matches!(&p.elements[2],Element::Field{label,..} if label=="Note"));
    }
}

#[cfg(test)]
mod capability_tests {
    use super::*;
    #[test]
    fn disabled_network_rejects_requests_but_keeps_local_browser() {
        let web = Web::new(false);
        assert!(web.load(1, "https://example.com".into()).is_err());
        web.load(2, "about:demo".into()).unwrap();
        let (id, result) = web.results.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(id, 2);
        assert!(result.is_ok());
    }
}
