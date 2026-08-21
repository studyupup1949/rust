const MARKDOWN_TAG: &str = "<markdown>";
const MARKDOWN_TAG_SELF_CLOSING: &str = "<markdown/>";
const MARKDOWN_TAG_SELF_CLOSING_SPACED: &str = "<markdown />";

pub fn render(template: &str, html_content: &str) -> String {
    if let Some(pos) = template.find(MARKDOWN_TAG) {
        let mut out = String::with_capacity(template.len() + html_content.len());
        out.push_str(&template[..pos]);
        out.push_str(html_content);
        out.push_str(&template[pos + MARKDOWN_TAG.len()..]);
        return out;
    }
    for tag in [MARKDOWN_TAG_SELF_CLOSING, MARKDOWN_TAG_SELF_CLOSING_SPACED] {
        if let Some(pos) = template.find(tag) {
            let mut out = String::with_capacity(template.len() + html_content.len());
            out.push_str(&template[..pos]);
            out.push_str(html_content);
            out.push_str(&template[pos + tag.len()..]);
            return out;
        }
    }

    #[cfg(debug_assertions)]
    eprintln!("actix_mark: template contains no <markdown> tag; appending content after template");

    format!("{template}{html_content}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_open_tag() {
        let tmpl = "<html><body><markdown></body></html>";
        assert_eq!(render(tmpl, "<p>hi</p>"), "<html><body><p>hi</p></body></html>");
    }

    #[test]
    fn replaces_self_closing_tag() {
        let tmpl = "<html><body><markdown/></body></html>";
        assert_eq!(render(tmpl, "<p>hi</p>"), "<html><body><p>hi</p></body></html>");
    }

    #[test]
    fn replaces_self_closing_tag_spaced() {
        let tmpl = "<html><body><markdown /></body></html>";
        assert_eq!(render(tmpl, "<p>hi</p>"), "<html><body><p>hi</p></body></html>");
    }
}
