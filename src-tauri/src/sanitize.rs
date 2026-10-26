use ammonia::Builder;

/// Sanitizes rich-text topic content produced by the editor before it is stored.
pub fn clean(html: &str) -> String {
    Builder::default()
        .add_tag_attributes("img", ["title"])
        .add_tag_attributes("pre", ["class"])
        .add_tag_attributes("code", ["class"])
        .url_schemes(["http", "https", "mailto", "tel"].into())
        .link_rel(Some("noopener noreferrer nofollow"))
        .clean(html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn strips_scripts_and_event_handlers() {
        let out = clean(r#"<p onclick="x()">hi</p><script>alert(1)</script>"#);
        assert_eq!(out, "<p>hi</p>");
    }

    #[test]
    fn keeps_editor_markup_and_relative_media() {
        let html = r#"<pre class="x"><code>a</code></pre><img src="/media/abc" alt="a">"#;
        let out = clean(html);
        assert!(out.contains(r#"<pre class="x">"#));
        assert!(out.contains(r#"src="/media/abc""#));
    }

    #[test]
    fn rejects_dangerous_link_schemes_and_forces_rel() {
        let out = clean(r#"<a href="javascript:alert(1)">x</a><a href="https://e.com">y</a>"#);
        assert!(!out.contains("javascript"));
        assert!(out.contains(r#"rel="noopener noreferrer nofollow""#));
    }
}
