use std::io::{Read, Write};
use std::path::Path;

use tench_document_core::{
    OfficeContent, OfficeRect, PresentationContent, SlideContent, SlideObject,
};

#[cfg(test)]
use super::format;
use super::write_zip_bytes;

/// Import a PPTX file into OfficeContent.
pub fn import_pptx(path: &Path) -> Result<OfficeContent, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open PPTX: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read PPTX: {e}"))?;
    crate::zip_util::check_archive_limits(&mut archive, &crate::zip_util::ArchiveLimits::desktop())
        .map_err(|e| format!("PPTX archive rejected by safety limits: {e}"))?;

    let slide_count = count_slides(&mut archive);
    let mut slides = Vec::new();

    for idx in 0..slide_count {
        let slide_path = format!("ppt/slides/slide{}.xml", idx + 1);
        let objects = if let Ok(mut slide_file) = archive.by_name(&slide_path) {
            let mut xml = String::new();
            slide_file.read_to_string(&mut xml).unwrap_or_default();
            parse_slide_xml(&xml)
        } else {
            Vec::new()
        };

        slides.push(SlideContent {
            id: format!("slide_{}", idx),
            index: idx as u32,
            title: extract_slide_title(&objects),
            notes: None,
            objects,
            background: serde_json::json!({}),
            transition: serde_json::json!({}),
        });
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

/// Export OfficeContent to PPTX bytes.
pub fn export_pptx_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let presentation = match content {
        OfficeContent::Slides(p) => p,
        _ => return Err("Expected Slides content for PPTX export.".to_string()),
    };

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    // [Content_Types].xml
    files.push((
        "[Content_Types].xml".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
</Types>"#
            .to_vec(),
    ));

    // _rels/.rels
    files.push((
        "_rels/.rels".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#
            .to_vec(),
    ));

    // ppt/_rels/presentation.xml.rels
    let mut pres_rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#
        .to_vec();
    for idx in 0..presentation.slides.len() {
        writeln!(
            pres_rels,
            r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#,
            idx + 1,
            idx + 1
        )
        .unwrap();
    }
    pres_rels.extend_from_slice(b"</Relationships>");
    files.push(("ppt/_rels/presentation.xml.rels".to_string(), pres_rels));

    // ppt/presentation.xml
    let mut pres_xml =
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:sldIdLst>"#
            .to_vec();
    for (idx, _slide) in presentation.slides.iter().enumerate() {
        writeln!(
            pres_xml,
            r#"<p:sldId id="{}" r:id="rId{}"/>"#,
            256 + idx,
            idx + 1
        )
        .unwrap();
    }
    pres_xml.extend_from_slice(b"</p:sldIdLst></p:presentation>");
    files.push(("ppt/presentation.xml".to_string(), pres_xml));

    // ppt/slides/slideN.xml
    for (idx, slide) in presentation.slides.iter().enumerate() {
        let mut slide_xml =
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr>"#
                .to_vec();

        write!(
            slide_xml,
            r#"<a:xfrm><a:off x="0" y="0"/><a:ext cx="{}" cy="{}"/><a:chOff x="0" y="0"/><a:chExt cx="{}" cy="{}"/></a:xfrm>"#,
            presentation.width, presentation.height, presentation.width, presentation.height
        )
        .unwrap();
        slide_xml.extend_from_slice(b"</p:grpSpPr>");

        for (obj_idx, obj) in slide.objects.iter().enumerate() {
            write!(
                slide_xml,
                r#"<p:sp><p:nvSpPr><p:cNvPr id="{}" name="Object {}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>"#,
                obj_idx + 2,
                obj_idx + 1
            )
            .unwrap();
            write!(
                slide_xml,
                r#"<a:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm>"#,
                obj.bounds.x as i64,
                obj.bounds.y as i64,
                obj.bounds.width as u64,
                obj.bounds.height as u64
            )
            .unwrap();
            slide_xml.extend_from_slice(b"</p:spPr>");

            // Text content
            if let Some(text) = obj.data.get("text").and_then(|t| t.as_str()) {
                write!(
                    slide_xml,
                    r#"<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>{}</a:t></a:r></a:p></p:txBody>"#,
                    xml_escape(text)
                )
                .unwrap();
            }

            slide_xml.extend_from_slice(b"</p:sp>");
        }

        slide_xml.extend_from_slice(b"</p:spTree></p:cSld></p:sld>");
        let slide_path = format!("ppt/slides/slide{}.xml", idx + 1);
        files.push((slide_path, slide_xml));
    }

    write_zip_bytes(&files)
}

fn count_slides(archive: &mut zip::ZipArchive<std::fs::File>) -> usize {
    let mut count = 0;
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                count += 1;
            }
        }
    }
    count
}

fn parse_slide_xml(xml: &str) -> Vec<SlideObject> {
    let mut objects = Vec::new();
    let mut in_text = false;
    let mut current_text = String::new();
    let mut obj_idx = 0;

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("p:sp>") || token.starts_with("p:sp ") {
            // New shape
            obj_idx += 1;
            current_text.clear();
            in_text = false;
        } else if token.starts_with("a:t>") {
            in_text = true;
            if let Some(text) = token.strip_prefix("a:t>") {
                current_text.push_str(text);
            }
        } else if in_text && token.starts_with('/') {
            in_text = false;
        } else if in_text {
            current_text.push('<');
            current_text.push_str(token);
        }

        // End of shape
        if token.starts_with("/p:sp") && !current_text.is_empty() {
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

fn extract_slide_title(objects: &[SlideObject]) -> Option<String> {
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
    use tench_document_core::OfficeContent;

    #[test]
    fn pptx_export_produces_valid_zip() {
        let content = format::empty_presentation_content("Test");
        let bytes = export_pptx_bytes(&content).expect("export");
        assert!(!bytes.is_empty());

        let reader = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(reader).expect("valid zip");
        assert!(archive.file_names().any(|n| n == "ppt/presentation.xml"));
    }

    #[test]
    fn pptx_round_trip_preserves_slides() {
        let dir = std::env::temp_dir().join(format!("tench_pptx_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("RoundTrip.pptx");

        let content = format::empty_presentation_content("RoundTrip");
        let bytes = export_pptx_bytes(&content).expect("export");
        std::fs::write(&path, &bytes).unwrap();

        let imported = import_pptx(&path).expect("import");
        if let OfficeContent::Slides(pres) = imported {
            assert_eq!(pres.slides.len(), 1);
            // Title is derived from filename for PPTX import
            assert_eq!(pres.title, "RoundTrip");
        } else {
            panic!("Expected Slides content");
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn pptx_import_rejects_unsafe_zip_entries_security_regression() {
        let dir = std::env::temp_dir().join(format!("tench_pptx_unsafe_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Unsafe.pptx");
        let file = std::fs::File::create(&path).expect("create pptx");
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer
            .start_file("ppt/slides/slide1.xml", options)
            .expect("slide");
        writer.write_all(b"<p:sld/>").expect("slide xml");
        writer.start_file("../evil.txt", options).expect("evil");
        writer.write_all(b"evil").expect("evil");
        writer.finish().expect("finish");

        let result = import_pptx(&path);

        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("safety limits"),
            "PPTX import should reject unsafe archive entries"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
