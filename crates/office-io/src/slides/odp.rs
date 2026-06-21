use std::io::{Read, Write};
use std::path::Path;

use tench_document_core::{
    OfficeContent, OfficeRect, PresentationContent, SlideContent, SlideObject,
};

#[cfg(test)]
use super::format;
use super::write_zip_bytes;

/// Import an ODP file into OfficeContent.
pub fn import_odp(path: &Path) -> Result<OfficeContent, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open ODP: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ODP: {e}"))?;

    let mut slides = Vec::new();
    let mut idx = 0;

    loop {
        let slide_path = format!("content{}.xml", idx);
        let objects = if let Ok(mut slide_file) = archive.by_name(&slide_path) {
            let mut xml = String::new();
            slide_file.read_to_string(&mut xml).unwrap_or_default();
            parse_odp_slide(&xml)
        } else {
            break;
        };

        slides.push(SlideContent {
            id: format!("slide_{}", idx),
            index: idx as u32,
            title: extract_odp_title(&objects),
            notes: None,
            objects,
            background: serde_json::json!({}),
            transition: serde_json::json!({}),
        });
        idx += 1;
    }

    // Try content.xml as fallback
    if slides.is_empty() {
        if let Ok(mut content_file) = archive.by_name("content.xml") {
            let mut xml = String::new();
            content_file.read_to_string(&mut xml).unwrap_or_default();
            let objects = parse_odp_slide(&xml);
            if !objects.is_empty() {
                slides.push(SlideContent {
                    id: "slide_0".to_string(),
                    index: 0,
                    title: extract_odp_title(&objects),
                    notes: None,
                    objects,
                    background: serde_json::json!({}),
                    transition: serde_json::json!({}),
                });
            }
        }
    }

    if slides.is_empty() {
        slides.push(SlideContent {
            id: "slide_0".to_string(),
            index: 0,
            title: Some("Slide 1".to_string()),
            notes: None,
            objects: Vec::new(),
            background: serde_json::json!({}),
            transition: serde_json::json!({}),
        });
    }

    Ok(OfficeContent::Slides(PresentationContent {
        id: format!(
            "pres_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string(),
        width: 9144000,
        height: 6858000,
        slides,
        assets: Vec::new(),
    }))
}

/// Export OfficeContent to ODP bytes.
pub fn export_odp_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let presentation = match content {
        OfficeContent::Slides(p) => p,
        _ => return Err("Expected Slides content for ODP export.".to_string()),
    };

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    // mimetype
    files.push((
        "mimetype".to_string(),
        b"application/vnd.oasis.opendocument.presentation".to_vec(),
    ));

    // META-INF/manifest.xml
    files.push(("META-INF/manifest.xml".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <manifest:file-entry manifest:media-type="application/vnd.oasis.opendocument.presentation" manifest:full-path="/"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="content.xml"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="styles.xml"/>
</manifest:manifest>"#.to_vec(),
    ));

    // styles.xml
    files.push((
        "styles.xml".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                        xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
                        xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
  <office:styles/>
</office:document-styles>"#
            .to_vec(),
    ));

    // content.xml
    let mut content_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
                         xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
                         xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
                         xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
                         xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
                         xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
                         xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0"
                         xmlns:dc="http://purl.org/dc/elements/1.1/">
<office:body><office:presentation>"#
        .to_vec();

    for slide in &presentation.slides {
        content_xml.extend_from_slice(b"<draw:page>");
        for obj in &slide.objects {
            write!(
                content_xml,
                r#"<draw:frame svg:x="{:.1}cm" svg:y="{:.1}cm" svg:width="{:.1}cm" svg:height="{:.1}cm"><draw:text-box><text:p>{}</text:p></draw:text-box></draw:frame>"#,
                obj.bounds.x / 914400.0,
                obj.bounds.y / 914400.0,
                obj.bounds.width / 914400.0,
                obj.bounds.height / 914400.0,
                obj.data.get("text").and_then(|t| t.as_str()).map(xml_escape).unwrap_or_default()
            )
            .unwrap();
        }
        content_xml.extend_from_slice(b"</draw:page>");
    }

    content_xml
        .extend_from_slice(b"</office:presentation></office:body></office:document-content>");
    files.push(("content.xml".to_string(), content_xml));

    write_zip_bytes(&files)
}

fn parse_odp_slide(xml: &str) -> Vec<SlideObject> {
    let mut objects = Vec::new();
    let mut in_text = false;
    let mut current_text = String::new();
    let mut obj_idx = 0;

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("draw:frame") {
            obj_idx += 1;
            current_text.clear();
            in_text = false;
        } else if token.starts_with("text:p>") {
            in_text = true;
            if let Some(text) = token.strip_prefix("text:p>") {
                current_text.push_str(text);
            }
        } else if in_text && token.starts_with('/') {
            in_text = false;
        } else if in_text {
            current_text.push('<');
            current_text.push_str(token);
        }

        if token.starts_with("/draw:frame") && !current_text.is_empty() {
            objects.push(SlideObject {
                id: format!("obj_{}", obj_idx),
                object_type: "text".to_string(),
                bounds: OfficeRect {
                    x: 0.0,
                    y: (obj_idx as f32) * 100.0,
                    width: 600.0,
                    height: 60.0,
                },
                data: serde_json::json!({ "text": xml_unescape(&current_text) }),
                style: serde_json::json!({}),
            });
            current_text.clear();
        }
    }

    objects
}

fn extract_odp_title(objects: &[SlideObject]) -> Option<String> {
    objects
        .first()
        .and_then(|obj| obj.data.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| {
            if s.len() > 80 {
                format!("{}...", &s[..77])
            } else {
                s.to_string()
            }
        })
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odp_export_produces_valid_zip() {
        let content = format::empty_presentation_content("Test");
        let bytes = export_odp_bytes(&content).expect("export");
        assert!(!bytes.is_empty());

        let reader = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(reader).expect("valid zip");
        assert!(archive.file_names().any(|n| n == "content.xml"));
    }
}
