use super::*;

impl SlidesState {
    pub fn set_slide_background_color(&mut self, color: Color) {
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.background = SlideBackground {
                color: Some(color),
                ..Default::default()
            };
        }
        self.sync_content_from_slides();
    }

    pub fn set_slide_background_gradient(&mut self, start: Color, end: Color) {
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.background = SlideBackground {
                gradient_start: Some(start),
                gradient_end: Some(end),
                ..Default::default()
            };
        }
        self.sync_content_from_slides();
    }

    // ── Phase 5.6: Slide transitions ────────────────────────────────

    pub fn set_slide_transition(&mut self, name: &str, duration_ms: u32) {
        if let Some(slide) = self.current_slide_mut() {
            slide.transition = SlideTransition {
                name: name.into(),
                duration_ms,
            };
        }
        self.transition_name = name.into();
        self.sync_content_from_slides();
    }

    // ── Phase 5.3: Slide layout presets ─────────────────────────────

    pub fn apply_slide_layout(&mut self, layout: SlideLayoutType) {
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.layout_type = layout;
            match layout {
                SlideLayoutType::Title => {
                    slide.elements.clear();
                    slide.elements.push(SlideElement::new_title(
                        &slide.title,
                        60.0,
                        100.0,
                        520.0,
                        80.0,
                    ));
                }
                SlideLayoutType::TitleContent => {
                    slide.elements.clear();
                    slide.elements.push(SlideElement::new_title(
                        &slide.title,
                        40.0,
                        30.0,
                        560.0,
                        50.0,
                    ));
                    slide.elements.push(SlideElement::new_text(
                        "Content placeholder",
                        40.0,
                        100.0,
                        560.0,
                        220.0,
                    ));
                }
                SlideLayoutType::TwoColumn => {
                    slide.elements.clear();
                    slide.elements.push(SlideElement::new_title(
                        &slide.title,
                        40.0,
                        30.0,
                        560.0,
                        50.0,
                    ));
                    slide.elements.push(SlideElement::new_text(
                        "Left column",
                        40.0,
                        100.0,
                        270.0,
                        220.0,
                    ));
                    slide.elements.push(SlideElement::new_text(
                        "Right column",
                        330.0,
                        100.0,
                        270.0,
                        220.0,
                    ));
                }
                SlideLayoutType::SectionHeader => {
                    slide.elements.clear();
                    slide.elements.push(SlideElement::new_title(
                        &slide.title,
                        60.0,
                        120.0,
                        520.0,
                        80.0,
                    ));
                    slide.elements.push(SlideElement::new_subtitle(
                        "Section subtitle",
                        60.0,
                        210.0,
                        520.0,
                        40.0,
                    ));
                }
                SlideLayoutType::Blank => {
                    slide.elements.clear();
                }
            }
        }
        self.selected_element = None;
        self.sync_content_from_slides();
    }

    // ── Phase 6: Presentation mode ──────────────────────────────────

    pub fn start_presentation(&mut self) {
        self.presentation = PresentationState {
            active: true,
            current_slide: self.current_slide,
            start_time: Some(std::time::Instant::now()),
            slide_start_time: Some(std::time::Instant::now()),
            laser_pointer: false,
            laser_pos: Point::ZERO,
            auto_advance_ms: None,
        };
        self.presenting = true;
    }

    pub fn stop_presentation(&mut self) {
        self.presentation.active = false;
        self.presenting = false;
    }

    pub fn presentation_next_slide(&mut self) -> bool {
        if self.presentation.current_slide + 1 >= self.slides.len() {
            return false;
        }
        self.presentation.current_slide += 1;
        self.presentation.slide_start_time = Some(std::time::Instant::now());
        true
    }

    pub fn presentation_prev_slide(&mut self) -> bool {
        if self.presentation.current_slide == 0 {
            return false;
        }
        self.presentation.current_slide -= 1;
        self.presentation.slide_start_time = Some(std::time::Instant::now());
        true
    }

    pub fn toggle_laser_pointer(&mut self) {
        self.presentation.laser_pointer = !self.presentation.laser_pointer;
    }

    pub fn set_laser_position(&mut self, pos: Point) {
        self.presentation.laser_pos = pos;
    }

    pub fn presentation_elapsed_secs(&self) -> f64 {
        self.presentation
            .start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    pub fn presentation_slide_elapsed_secs(&self) -> f64 {
        self.presentation
            .slide_start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }
}
