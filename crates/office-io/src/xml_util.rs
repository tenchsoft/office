/// Escape special XML characters in a string.
pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Decode XML entities in a string.
pub fn decode_xml(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Escape special HTML characters (subset of XML entities).
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Decode basic HTML entities.
pub fn decode_basic_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Strip all HTML/XML tags from a string, keeping only text content.
pub fn strip_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

/// Remove an entire block delimited by `<tag ...>...</tag>` (case-insensitive).
pub fn remove_tag_block(input: &str, tag: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    let open = format!("<{tag}");
    let close = format!("</{tag}>");

    loop {
        let lower = rest.to_ascii_lowercase();
        let Some(start) = lower.find(&open) else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let after_open = &rest[start..];
        let lower_after_open = after_open.to_ascii_lowercase();
        let Some(end) = lower_after_open.find(&close) else {
            break;
        };
        rest = &after_open[end + close.len()..];
    }

    output
}

/// Get the child nodes array from a serde_json Value.
pub fn child_nodes(value: &serde_json::Value) -> &[serde_json::Value] {
    value["content"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Extract all text content recursively from a Tiptap JSON node.
pub fn child_text(value: &serde_json::Value) -> String {
    match node_type(value) {
        Some("text") => value["text"].as_str().unwrap_or_default().to_string(),
        _ => child_nodes(value)
            .iter()
            .map(child_text)
            .collect::<Vec<_>>()
            .join(""),
    }
}

/// Get the `type` field of a Tiptap JSON node.
pub fn node_type(value: &serde_json::Value) -> Option<&str> {
    value["type"].as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_xml_handles_special_chars() {
        assert_eq!(
            escape_xml("a<b>c&d\"e'f"),
            "a&lt;b&gt;c&amp;d&quot;e&apos;f"
        );
    }

    #[test]
    fn decode_xml_handles_entities() {
        assert_eq!(decode_xml("a&lt;b&gt;c&amp;d"), "a<b>c&d");
    }

    #[test]
    fn escape_html_handles_special_chars() {
        assert_eq!(escape_html("a<b&c"), "a&lt;b&amp;c");
    }

    #[test]
    fn decode_basic_entities_handles_common_entities() {
        assert_eq!(decode_basic_entities("a&nbsp;b&amp;c"), "a b&c");
    }

    #[test]
    fn strip_tags_removes_html() {
        assert_eq!(strip_tags("<p>Hello <b>World</b></p>"), "Hello World");
    }

    #[test]
    fn strip_tags_preserves_text_without_tags() {
        assert_eq!(strip_tags("plain text"), "plain text");
    }

    #[test]
    fn remove_tag_block_removes_script() {
        let html = "<p>Before</p><script>alert(1)</script><p>After</p>";
        assert_eq!(
            remove_tag_block(html, "script"),
            "<p>Before</p><p>After</p>"
        );
    }

    #[test]
    fn remove_tag_block_is_case_insensitive() {
        let html = "<p>Before</p><SCRIPT>alert(1)</SCRIPT><p>After</p>";
        assert_eq!(
            remove_tag_block(html, "script"),
            "<p>Before</p><p>After</p>"
        );
    }

    #[test]
    fn child_nodes_returns_array() {
        let value = serde_json::json!({"content": [{"type": "text"}]});
        assert_eq!(child_nodes(&value).len(), 1);
    }

    #[test]
    fn child_nodes_returns_empty_for_missing() {
        let value = serde_json::json!({"type": "paragraph"});
        assert!(child_nodes(&value).is_empty());
    }

    #[test]
    fn child_text_extracts_text() {
        let value = serde_json::json!({
            "type": "paragraph",
            "content": [{"type": "text", "text": "Hello"}]
        });
        assert_eq!(child_text(&value), "Hello");
    }

    #[test]
    fn node_type_returns_type() {
        let value = serde_json::json!({"type": "heading"});
        assert_eq!(node_type(&value), Some("heading"));
    }

    #[test]
    fn node_type_returns_none_for_missing() {
        let value = serde_json::json!({});
        assert_eq!(node_type(&value), None);
    }
}
