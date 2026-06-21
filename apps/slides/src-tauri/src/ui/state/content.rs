use serde_json::json;
use tench_document_core::{
    OfficeContent, OfficeRect, PresentationContent, SlideContent, SlideObject,
};
use tench_ui::prelude::Color;

use super::{BorderStyle, Slide, SlideBackground, SlideElement, SlideLayoutType, SlideTransition};

// ── Phase 0.4: content_to_slides reverse conversion ────────────────

pub fn content_to_slides(content: &OfficeContent) -> Vec<Slide> {
    let OfficeContent::Slides(pres) = content else {
        return Vec::new();
    };
    pres.slides
        .iter()
        .map(|sc| {
            let elements: Vec<SlideElement> = sc
                .objects
                .iter()
                .map(|obj| {
                    let fill =
                        if let Some(fill_val) = obj.style.get("fill").and_then(|v| v.as_str()) {
                            match fill_val {
                                "accent" => Some(Color::rgb8(0x60, 0xA5, 0xFA)),
                                _ => None,
                            }
                        } else {
                            None
                        };
                    SlideElement {
                        kind: obj.object_type.clone(),
                        x: obj.bounds.x as f64,
                        y: obj.bounds.y as f64,
                        w: obj.bounds.width as f64,
                        h: obj.bounds.height as f64,
                        text: obj.data.get("text").and_then(|v| {
                            let s = v.as_str().unwrap_or("");
                            if s.is_empty() {
                                None
                            } else {
                                Some(s.into())
                            }
                        }),
                        fill,
                        ..Default::default()
                    }
                })
                .collect();
            let background = SlideBackground {
                color: None,
                gradient_start: None,
                gradient_end: None,
                image_path: None,
            };
            let transition = SlideTransition {
                name: sc
                    .transition
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none")
                    .into(),
                duration_ms: sc
                    .transition
                    .get("duration_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(500) as u32,
            };
            Slide {
                title: sc.title.clone().unwrap_or_default(),
                elements,
                notes: sc.notes.clone().unwrap_or_default(),
                background,
                layout_type: SlideLayoutType::default(),
                transition,
            }
        })
        .collect()
}

// ── Forward conversion ──────────────────────────────────────────────

pub(super) fn slides_to_content(title: &str, slides: &[Slide]) -> OfficeContent {
    OfficeContent::Slides(PresentationContent {
        id: "native_presentation".into(),
        title: title.into(),
        width: 9144000,
        height: 6858000,
        slides: slides
            .iter()
            .enumerate()
            .map(|(slide_idx, slide)| SlideContent {
                id: format!("slide_{slide_idx}"),
                index: slide_idx as u32,
                title: Some(slide.title.clone()),
                notes: (!slide.notes.is_empty()).then(|| slide.notes.clone()),
                objects: slide
                    .elements
                    .iter()
                    .enumerate()
                    .map(|(object_idx, element)| SlideObject {
                        id: format!("slide_{slide_idx}_object_{object_idx}"),
                        object_type: element.kind.clone(),
                        bounds: OfficeRect {
                            x: element.x as f32,
                            y: element.y as f32,
                            width: element.w as f32,
                            height: element.h as f32,
                        },
                        data: json!({
                            "text": element.text.clone().unwrap_or_default(),
                            "rotation": element.rotation,
                            "opacity": element.opacity,
                            "group_id": element.group_id,
                        }),
                        style: json!({
                            "fill": element.fill.map(|_| "accent"),
                            "border_width": element.border.as_ref().map(|b| b.width),
                            "border_style": element.border.as_ref().map(|b| match b.style {
                                BorderStyle::Solid => "solid",
                                BorderStyle::Dashed => "dashed",
                                BorderStyle::Dotted => "dotted",
                                BorderStyle::None => "none",
                            }),
                            "shadow": element.shadow.is_some(),
                            "z_index": element.z_index,
                        }),
                    })
                    .collect(),
                background: json!({
                    "color": slide.background.color.map(|c| format!("{:x}", c.to_u32())),
                    "gradient_start": slide.background.gradient_start.map(|c| format!("{:x}", c.to_u32())),
                    "gradient_end": slide.background.gradient_end.map(|c| format!("{:x}", c.to_u32())),
                    "image_path": slide.background.image_path,
                }),
                transition: json!({
                    "name": slide.transition.name,
                    "duration_ms": slide.transition.duration_ms,
                }),
            })
            .collect(),
        assets: Vec::new(),
    })
}

pub(super) fn slides_signature(slides: &[Slide]) -> String {
    slides
        .iter()
        .map(|slide| {
            let elements = slide
                .elements
                .iter()
                .map(|element| {
                    format!(
                        "{}:{}:{}:{}:{}:{}:{:.2}:{:.2}",
                        element.kind,
                        element.x,
                        element.y,
                        element.w,
                        element.h,
                        element.text.as_deref().unwrap_or_default(),
                        element.rotation,
                        element.opacity
                    )
                })
                .collect::<Vec<_>>()
                .join("|");
            format!(
                "{}:{}:{elements}:{}",
                slide.title, slide.notes, slide.transition.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
