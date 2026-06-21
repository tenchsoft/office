use serde_json::json;
use tench_document_core::{OfficeContent, PresentationContent, SlideContent};

pub fn empty_presentation_content(title: &str) -> OfficeContent {
    OfficeContent::Slides(PresentationContent {
        id: format!(
            "pres_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        title: title.to_string(),
        width: 9144000,
        height: 6858000,
        slides: vec![SlideContent {
            id: "slide_0".to_string(),
            index: 0,
            title: Some(format!("{} - Slide 1", title)),
            notes: None,
            objects: Vec::new(),
            background: json!({}),
            transition: json!({}),
        }],
        assets: Vec::new(),
    })
}

pub fn presentation_to_plain_text(content: &OfficeContent) -> String {
    match content {
        OfficeContent::Slides(pres) => {
            let mut text = String::new();
            for slide in &pres.slides {
                if let Some(title) = &slide.title {
                    text.push_str(title);
                    text.push('\n');
                }
                for obj in &slide.objects {
                    if let Some(text_content) = obj.data.get("text").and_then(|v| v.as_str()) {
                        text.push_str(text_content);
                        text.push('\n');
                    }
                }
            }
            text
        }
        _ => String::new(),
    }
}
