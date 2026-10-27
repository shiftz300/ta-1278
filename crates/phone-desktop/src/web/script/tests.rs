use super::*;
use std::sync::{Arc, atomic::AtomicU64};
fn open(html: &str) -> Session {
    Session::new(
        1,
        "https://example.org/path/",
        html,
        Rc::new(RefCell::new(Network::new(Arc::new(AtomicU64::new(1)), 1))),
    )
    .unwrap()
}
fn content(page: &Page) -> String {
    page.elements
        .iter()
        .map(Element::label)
        .collect::<Vec<_>>()
        .join("\n")
}
#[test]
fn scripts_build_content_with_dom_ready_promises_timers_and_write() {
    let mut s = open(
        r#"<title>Old</title><main id=app>Loading</main><script>
        document.title='Ready';
        document.addEventListener('DOMContentLoaded',()=>{
            const p=document.createElement('p');p.className='result';p.textContent='Loaded';
            document.querySelector('#app').replaceChildren(p);
            Promise.resolve().then(()=>p.append(' promise'));
            setTimeout(()=>p.append(' timer'),10);
        });
        document.write('<p>Written</p>');
    </script>"#,
    );
    let (p, next) = s.snapshot();
    assert_eq!(p.title, "Ready");
    assert!(next.is_none());
    assert!(s.errors.is_empty(), "{:?}", s.errors);
    let text = content(&p);
    assert!(text.contains("Loaded promise timer"), "{text}");
    assert!(text.contains("Written"));
    assert!(!text.contains("Loading"));
}
#[test]
fn click_syncs_edited_fields_and_runs_inline_and_delegated_handlers() {
    let mut s = open(
        r#"<input id=name value=Old><button type=button id=ok onclick="document.getElementById('out').textContent=document.getElementById('name').value">Run</button><p id=out></p><script>document.body.addEventListener('click',e=>{if(e.target.id==='ok')document.title='Clicked';});</script>"#,
    );
    let (mut p, _) = s.snapshot();
    let field = p
        .elements
        .iter_mut()
        .find_map(|e| {
            if let Element::Field { value, .. } = e {
                Some(value)
            } else {
                None
            }
        })
        .unwrap();
    *field = "Edited".into();
    let url = p
        .elements
        .iter()
        .find_map(|e| {
            if let Element::Link { url, .. } = e {
                Some(url.clone())
            } else {
                None
            }
        })
        .unwrap();
    let target = url.rsplit('/').next().unwrap().parse().unwrap();
    let (p, next) = s.click(target, &p);
    assert!(next.is_none());
    assert_eq!(p.title, "Clicked");
    assert!(content(&p).contains("Edited"));
    assert!(
        p.elements
            .iter()
            .any(|e| matches!(e,Element::Field{initial,..} if initial=="Old"))
    );
    assert!(s.errors.is_empty(), "{:?}", s.errors);
}
#[test]
fn default_form_get_and_prevent_default_are_respected() {
    let mut s = open(
        r#"<form action='/search'><input name=q value='a & b'><button>Go</button></form><script>document.title='Form';</script>"#,
    );
    let (p, _) = s.snapshot();
    let target = p
        .elements
        .iter()
        .find_map(|e| {
            if let Element::Link { url, .. } = e {
                url.rsplit('/').next()?.parse::<u64>().ok()
            } else {
                None
            }
        })
        .unwrap();
    let (_, next) = s.click(target, &p);
    assert_eq!(
        next.as_deref(),
        Some("https://example.org/search?q=a%20%26%20b")
    );
    assert!(
        s.snapshot().1.is_none(),
        "A consumed navigation must not run again after Back"
    );
    let mut s = open(
        r#"<a href='/next' onclick="event.preventDefault();document.body.append('Stayed')">Stay</a>"#,
    );
    let (p, _) = s.snapshot();
    let target = s.controls[0].1;
    let (p, next) = s.click(target, &p);
    assert!(next.is_none());
    assert!(content(&p).contains("Stayed"));
}
#[test]
fn loops_errors_and_dom_quotas_keep_static_content_and_do_not_hang() {
    for code in [
        "while(true){}",
        "/(a+)+$/.test('a'.repeat(50)+'!')",
        "throw Error('broken')",
        "for(;;)document.body.appendChild(document.createElement('p'))",
        "function f(){return f()}f()",
        "let x=[];for(;;)x.push('x'.repeat(1024*1024))",
    ] {
        let start = Instant::now();
        let mut s = open(&format!("<p>Readable</p><script>{code}</script>"));
        let (p, _) = s.snapshot();
        assert!(content(&p).contains("Readable"));
        assert!(!s.errors.is_empty(), "{code}");
        assert!(start.elapsed() < Duration::from_secs(4), "{code}");
    }
}
#[test]
fn host_bindings_are_absent_and_cross_origin_and_write_fetch_are_rejected() {
    assert!(!Session::allowed(
        "<meta http-equiv=Content-Security-Policy content=\"script-src 'none'\"><script>1</script>"
    ));
    let mut s = open(
        r#"<p id=out></p><script>
        document.getElementById('out').textContent=[typeof std,typeof os,typeof require,typeof process,typeof __resource,typeof __parse,typeof Atomics,typeof SharedArrayBuffer].join(',');
        fetch('http://127.0.0.1/private').catch(e=>document.body.append(' blocked-origin'));
        fetch('/write',{method:'POST',body:'data'}).catch(e=>document.body.append(' blocked-write'));
    </script>"#,
    );
    let (p, _) = s.snapshot();
    let text = content(&p);
    assert!(
        text.contains(
            "undefined,undefined,undefined,undefined,undefined,undefined,undefined,undefined"
        ),
        "{text}"
    );
    assert!(
        text.contains("blocked-origin") && text.contains("blocked-write"),
        "{text}"
    );
    assert!(s.errors.is_empty(), "{:?}", s.errors);
}
