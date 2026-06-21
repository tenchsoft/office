use super::*;

impl SlidesState {
    pub fn add_slide(&mut self) {
        self.push_undo();
        let slide_idx = self.slides.len();
        self.slides.push(Slide {
            title: format!("Slide {}", slide_idx + 1),
            elements: vec![SlideElement::new_title(
                "New Slide",
                60.0,
                50.0,
                520.0,
                60.0,
            )],
            ..Default::default()
        });
        self.current_slide = self.slides.len() - 1;
        self.selected_element = Some(0);
        self.sync_content_from_slides();
    }

    pub fn add_blank_slide(&mut self) {
        self.push_undo();
        let slide_idx = self.slides.len();
        self.slides.push(Slide {
            title: format!("Slide {}", slide_idx + 1),
            elements: Vec::new(),
            ..Default::default()
        });
        self.current_slide = self.slides.len() - 1;
        self.selected_element = None;
        self.sync_content_from_slides();
    }

    /// Phase 7.1: Add a slide with a specific layout type.
    pub fn add_slide_with_layout(&mut self, layout: SlideLayoutType) {
        self.push_undo();
        let slide_idx = self.slides.len();
        let elements = match layout {
            SlideLayoutType::Blank => Vec::new(),
            SlideLayoutType::Title => vec![SlideElement::new_title(
                "Slide Title",
                60.0,
                50.0,
                520.0,
                60.0,
            )],
            SlideLayoutType::TitleContent => vec![
                SlideElement::new_title("Slide Title", 60.0, 30.0, 520.0, 50.0),
                SlideElement {
                    kind: "text".into(),
                    x: 60.0,
                    y: 100.0,
                    w: 520.0,
                    h: 200.0,
                    text: Some("Content placeholder".into()),
                    ..Default::default()
                },
            ],
            SlideLayoutType::TwoColumn => vec![
                SlideElement::new_title("Slide Title", 60.0, 30.0, 520.0, 50.0),
                SlideElement {
                    kind: "text".into(),
                    x: 60.0,
                    y: 100.0,
                    w: 250.0,
                    h: 200.0,
                    text: Some("Left column".into()),
                    ..Default::default()
                },
                SlideElement {
                    kind: "text".into(),
                    x: 330.0,
                    y: 100.0,
                    w: 250.0,
                    h: 200.0,
                    text: Some("Right column".into()),
                    ..Default::default()
                },
            ],
            SlideLayoutType::SectionHeader => vec![
                SlideElement::new_title("Section Title", 60.0, 120.0, 520.0, 80.0),
                SlideElement {
                    kind: "text".into(),
                    x: 60.0,
                    y: 220.0,
                    w: 520.0,
                    h: 40.0,
                    text: Some("Section subtitle".into()),
                    ..Default::default()
                },
            ],
        };
        self.slides.push(Slide {
            title: format!("Slide {}", slide_idx + 1),
            elements,
            background: SlideBackground {
                color: Some(self.slide_theme.background_color),
                ..Default::default()
            },
            layout_type: layout,
            ..Default::default()
        });
        self.current_slide = self.slides.len() - 1;
        self.selected_element = None;
        self.sync_content_from_slides();
    }

    /// Phase 7.2: Apply a theme to all slides.
    pub fn set_slide_theme(&mut self, theme: SlideTheme) {
        self.push_undo();
        let bg_color = theme.background_color;
        for slide in &mut self.slides {
            slide.background.color = Some(bg_color);
            slide.background.gradient_start = None;
            slide.background.gradient_end = None;
        }
        self.slide_theme = theme;
        self.sync_content_from_slides();
    }

    /// Phase 7.4: Set current slide background.
    pub fn set_current_slide_background(&mut self, bg: SlideBackground) {
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.background = bg;
        }
        self.sync_content_from_slides();
    }

    pub fn duplicate_slide(&mut self, index: usize) -> bool {
        if index >= self.slides.len() {
            return false;
        }
        self.push_undo();
        let mut dup = self.slides[index].clone();
        dup.title = format!("{} (copy)", dup.title);
        self.slides.insert(index + 1, dup);
        self.current_slide = index + 1;
        self.selected_element = None;
        self.sync_content_from_slides();
        true
    }

    pub fn delete_slide(&mut self, index: usize) -> bool {
        if self.slides.len() <= 1 || index >= self.slides.len() {
            return false;
        }
        self.push_undo();
        self.slides.remove(index);
        if self.current_slide >= self.slides.len() {
            self.current_slide = self.slides.len() - 1;
        }
        self.selected_element = None;
        self.sync_content_from_slides();
        true
    }

    pub fn move_slide(&mut self, from: usize, to: usize) -> bool {
        if from >= self.slides.len() || to >= self.slides.len() || from == to {
            return false;
        }
        self.push_undo();
        let slide = self.slides.remove(from);
        self.slides.insert(to, slide);
        self.current_slide = to;
        self.sync_content_from_slides();
        true
    }

    pub fn select_slide(&mut self, index: usize) -> bool {
        if index >= self.slides.len() {
            return false;
        }
        self.current_slide = index;
        self.selected_element = if self.slides[index].elements.is_empty() {
            None
        } else {
            Some(0)
        };
        self.selected_elements.clear();
        true
    }

    pub fn previous_slide(&mut self) -> bool {
        if self.current_slide == 0 {
            return false;
        }
        self.current_slide -= 1;
        self.selected_element = None;
        self.selected_elements.clear();
        true
    }

    pub fn next_slide(&mut self) -> bool {
        if self.current_slide + 1 >= self.slides.len() {
            return false;
        }
        self.current_slide += 1;
        self.selected_element = None;
        self.selected_elements.clear();
        true
    }

    // ── Toast ──────────────────────────────────────────────────────

    /// Phase 11: Show a toast message that auto-dismisses after ~3 seconds.
    pub fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast = Some(msg.into());
        self.toast_frames = 180; // ~3 seconds at 60fps
    }

    /// Phase 11: Decrement toast counter; clear when expired.
    pub fn tick_toast(&mut self) {
        if self.toast_frames > 0 {
            self.toast_frames -= 1;
            if self.toast_frames == 0 {
                self.toast = None;
            }
        }
    }
}
