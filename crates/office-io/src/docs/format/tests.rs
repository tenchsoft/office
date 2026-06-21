use super::*;
use tench_document_core::{BlockNode, ImageSource, InlineNode};

#[test]
fn plain_text_round_trips_as_document_text() {
    let doc = plain_text_to_tdm("Alpha\nBeta");
    assert_eq!(tdm_to_plain_text(&doc), "Alpha\nBeta");
}

#[test]
fn plain_text_round_trips_via_office_content() {
    let content = plain_text_to_docs_content("Alpha\nBeta");
    assert_eq!(docs_content_to_plain_text(&content), "Alpha\nBeta");
}

#[test]
fn markdown_import_preserves_headings_and_lists() {
    let doc = markdown_to_tdm("# Title\n\n- One\n- Two\n\nBody");
    let md = tdm_to_markdown(&doc);
    let text = tdm_to_plain_text(&doc);

    assert!(md.contains("# Title"));
    assert!(md.contains("- One"));
    assert!(md.contains("- Two"));
    assert!(text.contains("Body"));
}

#[test]
fn markdown_import_via_office_content() {
    let content = markdown_to_docs_content("# Title\n\n- One\n- Two\n\nBody");
    let markdown = docs_content_to_markdown(&content);
    let text = docs_content_to_plain_text(&content);

    assert!(markdown.contains("# Title"));
    assert!(markdown.contains("- One"));
    assert!(text.contains("Body"));
}

#[test]
fn html_import_removes_script_content() {
    let doc = html_to_tdm("<h1>Title</h1><script>alert(1)</script><p>Body</p>");
    let text = tdm_to_plain_text(&doc);

    assert!(text.contains("Title"));
    assert!(text.contains("Body"));
    assert!(!text.contains("alert"));
}

#[test]
fn html_export_escapes_text_content() {
    let doc = plain_text_to_tdm("A < B & C");
    let html = tdm_to_html(&doc);

    assert!(html.contains("A &lt; B &amp; C"));
    assert!(html.contains("<!doctype html>"));
}

#[test]
fn empty_tdm_has_one_paragraph() {
    let doc = empty_tdm();
    assert_eq!(doc.content.len(), 1);
    assert!(matches!(&doc.content[0], BlockNode::Paragraph { .. }));
}

#[test]
fn html_import_parses_bold_italic() {
    let html = "<p><strong>Bold</strong> <em>Italic</em></p>";
    let doc = html_to_tdm(html);
    let text = tdm_to_plain_text(&doc);
    assert!(text.contains("Bold"));
    assert!(text.contains("Italic"));

    // Check that marks are preserved.
    if let BlockNode::Paragraph { content, .. } = &doc.content[0] {
        let bold_node = content
            .iter()
            .find(|n| matches!(n, InlineNode::Text { marks, .. } if marks.bold));
        assert!(bold_node.is_some(), "Should have a bold text node");

        let italic_node = content
            .iter()
            .find(|n| matches!(n, InlineNode::Text { marks, .. } if marks.italic));
        assert!(italic_node.is_some(), "Should have an italic text node");
    } else {
        panic!("Expected paragraph");
    }
}

#[test]
fn html_import_parses_table() {
    let html = "<table><tr><td>A</td><td>B</td></tr></table>";
    let doc = html_to_tdm(html);
    assert!(!doc.content.is_empty());
    if let BlockNode::Table { rows } = &doc.content[0] {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells.len(), 2);
    } else {
        panic!("Expected table, got {:?}", doc.content[0]);
    }
}

#[test]
fn html_import_parses_image() {
    let html = "<img src=\"photo.png\" alt=\"A photo\">";
    let doc = html_to_tdm(html);
    if let BlockNode::Image { source, alt, .. } = &doc.content[0] {
        assert_eq!(alt.as_deref(), Some("A photo"));
        if let ImageSource::Referenced { path } = source {
            assert_eq!(path, "photo.png");
        } else {
            panic!("Expected referenced image");
        }
    } else {
        panic!("Expected image block");
    }
}

#[test]
fn html_import_parses_css_inline_styles() {
    let html =
        r#"<p><span style="font-weight: bold; color: #ff0000; font-size: 14pt">Styled</span></p>"#;
    let doc = html_to_tdm(html);
    if let BlockNode::Paragraph { content, .. } = &doc.content[0] {
        if let InlineNode::Text { text, marks } = &content[0] {
            assert_eq!(text, "Styled");
            assert!(marks.bold);
            assert_eq!(marks.text_color.as_deref(), Some("#ff0000"));
            assert_eq!(marks.font_size, Some(14.0));
        } else {
            panic!("Expected text node");
        }
    } else {
        panic!("Expected paragraph");
    }
}

#[test]
fn html_import_parses_heading_levels() {
    let html = "<h2>Subtitle</h2><p>Body</p>";
    let doc = html_to_tdm(html);
    if let BlockNode::Heading { level, .. } = &doc.content[0] {
        assert_eq!(*level, 2);
    } else {
        panic!("Expected heading");
    }
}

#[test]
fn html_import_parses_lists() {
    let html = "<ul><li>One</li><li>Two</li></ul><ol><li>First</li></ol>";
    let doc = html_to_tdm(html);
    assert!(doc.content.len() >= 2);
    assert!(matches!(&doc.content[0], BlockNode::BulletList { .. }));
    assert!(matches!(&doc.content[1], BlockNode::OrderedList { .. }));
}

#[test]
fn html_import_parses_link() {
    let html = r#"<p><a href="https://example.com">Example</a></p>"#;
    let doc = html_to_tdm(html);
    if let BlockNode::Paragraph { content, .. } = &doc.content[0] {
        if let InlineNode::Link { href, text, .. } = &content[0] {
            assert_eq!(href, "https://example.com");
            assert_eq!(text, "Example");
        } else {
            panic!("Expected link node");
        }
    } else {
        panic!("Expected paragraph");
    }
}
