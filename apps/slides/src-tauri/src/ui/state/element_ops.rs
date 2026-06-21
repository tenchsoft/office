use super::*;

impl SlidesState {
    pub fn nudge_selected_element(&mut self, dx: f64, dy: f64) {
        let Some(idx) = self.selected_element else {
            return;
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.x += dx;
                elem.y += dy;
            }
        }
        self.sync_content_from_slides();
    }

    pub fn delete_selected_element(&mut self) -> bool {
        let Some(idx) = self.selected_element.take() else {
            return false;
        };
        {
            let Some(slide) = self.current_slide() else {
                return false;
            };
            if idx >= slide.elements.len() {
                return false;
            }
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.remove(idx);
        }
        self.sync_content_from_slides();
        true
    }

    pub fn duplicate_selected_element(&mut self) -> bool {
        let Some(idx) = self.selected_element else {
            return false;
        };
        let Some(mut elem) = self
            .current_slide()
            .and_then(|s| s.elements.get(idx).cloned())
        else {
            return false;
        };
        self.push_undo();
        elem.x += 16.0;
        elem.y += 16.0;
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.push(elem);
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
        true
    }

    pub fn insert_text_element(&mut self, text: impl Into<String>) -> bool {
        if self.current_slide().is_none() {
            return false;
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide
                .elements
                .push(SlideElement::new_text(text, 64.0, 168.0, 500.0, 80.0));
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
        true
    }

    // ── Phase 3.1: Basic shape insertion ────────────────────────────

    pub fn insert_shape(&mut self, shape_kind: &str, x: f64, y: f64, w: f64, h: f64) -> bool {
        if self.current_slide().is_none() {
            return false;
        }
        self.push_undo();
        let mut elem = SlideElement {
            kind: shape_kind.into(),
            x,
            y,
            w,
            h,
            fill: Some(Color::rgb8(0x60, 0xA5, 0xFA)),
            ..Default::default()
        };
        match shape_kind {
            "line" | "arrow" => {
                elem.fill = None;
                elem.border = Some(ElementBorder {
                    color: Color::BLACK,
                    width: 2.0,
                    style: BorderStyle::Solid,
                });
            }
            _ => {}
        }
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.push(elem);
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
        true
    }

    // ── Phase 3.4: Image insertion ──────────────────────────────────

    pub fn insert_image_element(
        &mut self,
        image_path: &str,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    ) -> bool {
        if self.current_slide().is_none() {
            return false;
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.push(SlideElement {
                kind: "image".into(),
                x,
                y,
                w,
                h,
                text: Some(image_path.into()),
                ..Default::default()
            });
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
        true
    }

    // ── Phase 4.1: Table insertion ──────────────────────────────────

    pub fn insert_table(&mut self, rows: usize, cols: usize, x: f64, y: f64) -> bool {
        if self.current_slide().is_none() {
            return false;
        }
        self.push_undo();
        let cell_w = 80.0;
        let cell_h = 28.0;
        let table_w = cols as f64 * cell_w;
        let table_h = rows as f64 * cell_h;
        let mut cells = Vec::new();
        for r in 0..rows {
            for _c in 0..cols {
                cells.push(json!({
                    "row": r,
                    "col": _c,
                    "text": "",
                    "merged": false
                }));
            }
        }
        let table_elem = SlideElement {
            kind: "table".into(),
            x,
            y,
            w: table_w,
            h: table_h,
            text: Some(
                serde_json::to_string(&json!({
                    "rows": rows,
                    "cols": cols,
                    "cells": cells
                }))
                .unwrap_or_default(),
            ),
            border: Some(ElementBorder {
                color: Color::rgb8(0xCC, 0xCC, 0xCC),
                width: 1.0,
                style: BorderStyle::Solid,
            }),
            ..Default::default()
        };
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.push(table_elem);
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
        true
    }

    // ── Phase 4.4: Chart insertion ──────────────────────────────────

    pub fn insert_chart(&mut self, chart_type: &str, x: f64, y: f64) -> bool {
        if self.current_slide().is_none() {
            return false;
        }
        self.push_undo();
        let chart_data = json!({
            "chart_type": chart_type,
            "series": [
                {"name": "Series 1", "values": [10.0, 20.0, 30.0, 40.0]},
                {"name": "Series 2", "values": [15.0, 25.0, 35.0, 45.0]},
            ],
            "categories": ["Q1", "Q2", "Q3", "Q4"],
            "title": "Chart Title",
            "show_legend": true,
            "show_values": false
        });
        let chart_elem = SlideElement {
            kind: "chart".into(),
            x,
            y,
            w: 400.0,
            h: 280.0,
            text: Some(serde_json::to_string(&chart_data).unwrap_or_default()),
            ..Default::default()
        };
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.push(chart_elem);
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
        true
    }

    // ── Element property setters (Phase 2) ─────────────────────────

    pub fn set_element_x(&mut self, x: f64) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.x = x;
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_y(&mut self, y: f64) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.y = y;
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_w(&mut self, w: f64) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.w = w.max(10.0);
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_h(&mut self, h: f64) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.h = h.max(10.0);
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_rotation(&mut self, rotation: f64) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.rotation = rotation;
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_opacity(&mut self, opacity: f64) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.opacity = opacity.clamp(0.0, 1.0);
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_fill(&mut self, color: Color) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.fill = Some(color);
            }
        }
        self.sync_content_from_slides();
    }

    pub fn set_element_border_color(&mut self, color: Color) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                if let Some(border) = &mut elem.border {
                    border.color = color;
                }
            }
        }
        self.sync_content_from_slides();
    }

    pub fn cycle_element_fill_color(&mut self) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        let palette = [
            Color::rgb8(0x60, 0xA5, 0xFA), // blue
            Color::rgb8(0x22, 0xC5, 0x5E), // green
            Color::rgb8(0xEF, 0x44, 0x44), // red
            Color::rgb8(0xF5, 0x9E, 0x0B), // amber
            Color::rgb8(0x8B, 0x5C, 0xF6), // purple
            Color::rgb8(0xEC, 0x48, 0x99), // pink
            Color::rgb8(0x06, 0xB6, 0xD4), // cyan
            Color::rgb8(0xFF, 0xFF, 0xFF), // white
            Color::rgb8(0x1F, 0x29, 0x37), // dark
        ];
        let current = self
            .current_slide()
            .and_then(|s| s.elements.get(idx))
            .and_then(|e| e.fill);
        let next = match current {
            Some(c) => {
                let pos = palette.iter().position(|&p| p == c).unwrap_or(0);
                palette[(pos + 1) % palette.len()]
            }
            None => palette[0],
        };
        self.set_element_fill(next);
    }

    pub fn cycle_element_border_color(&mut self) {
        let idx = match self.selected_element {
            Some(i) => i,
            None => return,
        };
        let palette = [
            Color::BLACK,
            Color::rgb8(0x60, 0xA5, 0xFA),
            Color::rgb8(0x22, 0xC5, 0x5E),
            Color::rgb8(0xEF, 0x44, 0x44),
            Color::rgb8(0xF5, 0x9E, 0x0B),
            Color::rgb8(0x8B, 0x5C, 0xF6),
        ];
        let current = self
            .current_slide()
            .and_then(|s| s.elements.get(idx))
            .and_then(|e| e.border.as_ref().map(|b| b.color));
        let next = match current {
            Some(c) => {
                let pos = palette.iter().position(|&p| p == c).unwrap_or(0);
                palette[(pos + 1) % palette.len()]
            }
            None => palette[0],
        };
        self.set_element_border_color(next);
    }
}
