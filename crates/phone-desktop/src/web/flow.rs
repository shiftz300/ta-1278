//! Preserve inline order, block boundaries and fieldsets separately from focusable values.
use phone_core::browser::flow::Flow;
use scraper::ElementRef;
pub fn build(body: ElementRef, controls: &[(ElementRef, usize)]) -> Vec<Flow> {
    let mut out = vec![];
    walk(body, controls, &mut out, false, false, 0);
    out
}
fn walk(
    e: ElementRef,
    controls: &[(ElementRef, usize)],
    out: &mut Vec<Flow>,
    heading: bool,
    bold: bool,
    depth: usize,
) {
    if depth >= 128 || out.len() >= 4800 {
        return;
    }
    let v = e.value();
    let tag = v.name();
    if matches!(
        tag,
        "head" | "title" | "script" | "style" | "template" | "legend"
    ) || v.attr("hidden").is_some()
    {
        return;
    }
    if tag == "input" && v.attr("type") == Some("hidden") {
        return;
    }
    let heading = heading || matches!(tag, "h1" | "h2" | "h3" | "h4");
    let bold = bold || heading || matches!(tag, "b" | "strong");
    let block = matches!(
        tag,
        "p" | "div" | "h1" | "h2" | "h3" | "h4" | "li" | "br" | "tr" | "section"
    );
    if block {
        out.push(Flow::Break {
            gap: if tag == "p" { 12 } else { 0 },
        });
    }
    if matches!(tag, "ul" | "ol") {
        out.push(Flow::Break { gap: 0 });
        out.push(Flow::Indent(20));
    }
    if tag == "li" {
        out.push(Flow::Bullet);
    }
    if tag == "fieldset" {
        let legend = e
            .select(&scraper::Selector::parse("legend").unwrap())
            .next()
            .map(super::text)
            .unwrap_or_default();
        out.push(Flow::BoxStart(legend));
    }
    let control = v.attr("data-phone-action").is_some()
        || matches!(tag, "input" | "textarea" | "select" | "button")
        || (tag == "a" && v.attr("href").is_some());
    if tag == "input"
        && v.attr("data-phone-action").is_none()
        && !matches!(
            v.attr("type").unwrap_or("text"),
            "text"
                | "search"
                | "email"
                | "url"
                | "tel"
                | "number"
                | "password"
                | "time"
                | "radio"
                | "checkbox"
                | "submit"
                | "reset"
        )
    {
        return;
    }
    if control {
        let width = match tag {
            "textarea" => 102,
            "select" => 0,
            "button" | "a" => 0,
            _ => {
                if matches!(v.attr("type"), Some("radio" | "checkbox")) {
                    10
                } else {
                    v.attr("size")
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(20)
                        .saturating_mul(5)
                        .saturating_add(2)
                        .clamp(10, 230)
                }
            }
        };
        let height = if tag == "textarea" {
            v.attr("rows")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(2)
                .saturating_mul(19)
                .saturating_add(4)
                .min(160)
        } else if matches!(v.attr("type"), Some("radio" | "checkbox")) {
            10
        } else {
            15
        };
        if tag != "a" && v.attr("disabled").is_some() {
            out.push(Flow::Disabled {
                width: width.max(24),
                height,
                value: v.attr("value").unwrap_or("").into(),
            });
        } else if let Some(&(_, element)) = controls.iter().find(|(node, _)| *node == e) {
            out.push(Flow::Control {
                element,
                width,
                height,
            });
        }
        return;
    }
    for node in e.children() {
        if let Some(t) = node.value().as_text() {
            let text = t.split_whitespace().collect::<Vec<_>>().join(" ");
            if !text.is_empty() {
                out.push(Flow::Text {
                    text: format!(
                        "{}{}{}",
                        if t.starts_with(char::is_whitespace) {
                            " "
                        } else {
                            ""
                        },
                        text,
                        if t.ends_with(char::is_whitespace) {
                            " "
                        } else {
                            ""
                        }
                    ),
                    heading,
                    bold,
                });
            } else if !t.is_empty() {
                out.push(Flow::Text {
                    text: " ".into(),
                    heading,
                    bold,
                });
            }
        } else if let Some(child) = ElementRef::wrap(node) {
            walk(child, controls, out, heading, bold, depth + 1);
        }
    }
    if tag == "fieldset" {
        out.push(Flow::BoxEnd);
    }
    if matches!(tag, "ul" | "ol") {
        out.push(Flow::Indent(-20));
        out.push(Flow::Break { gap: 0 });
    }
    if block && tag != "br" {
        out.push(Flow::Break {
            gap: if matches!(tag, "p" | "h1" | "h2" | "h3" | "h4") {
                12
            } else {
                0
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phone_core::browser::Element;
    #[test]
    fn controls_keep_dom_identity_across_skipped_and_nested_nodes() {
        let page = super::super::parse(
            "https://example.com/",
            r#"
            <input hidden name=hidden><input type=file><input disabled value=disabled>
            <label for=a>A: <input id=a name=a readonly value=first></label>
            <fieldset><legend><a href=legend>Legend</a></legend>
            <input name=b value=second></fieldset><a disabled href=next>Next</a>
        "#,
        );
        let indices: Vec<_> = page
            .flow
            .iter()
            .filter_map(|f| match f {
                Flow::Control { element, .. } => Some(*element),
                _ => None,
            })
            .collect();
        assert_eq!(indices.len(), 3);
        assert!(
            matches!(&page.elements[indices[0]], Element::Field {name, read_only:true,..} if name=="a")
        );
        assert!(matches!(&page.elements[indices[1]], Element::Field {name,..} if name=="b"));
        assert!(matches!(&page.elements[indices[2]], Element::Link {label,..} if label=="Next"));
        assert!(
            page.flow
                .iter()
                .any(|f| matches!(f, Flow::Disabled {value,..} if value=="disabled"))
        );
    }
}
