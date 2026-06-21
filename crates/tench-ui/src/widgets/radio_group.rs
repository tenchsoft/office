//! RadioGroup — single-select radio buttons.

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Size};

/// A group of radio buttons for single selection.
pub struct RadioGroup {
    options: Vec<String>,
    selected: usize,
    on_change: Option<Box<dyn Fn(usize) + Send>>,
}

impl RadioGroup {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: 0,
            on_change: None,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn on_change(mut self, f: impl Fn(usize) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_text(&self) -> &str {
        self.options
            .get(self.selected)
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

impl Widget for RadioGroup {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => self.options.len() as f64 * 28.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let _size = ctx.size();
        let mut painter = Painter::new(scene);

        for (i, label) in self.options.iter().enumerate() {
            let y = i as f64 * 28.0;
            let _radio_center = Point::new(14.0, y + 14.0);

            // Outer circle
            painter.draw_text(
                if i == self.selected { "●" } else { "○" },
                4.0,
                y + 18.0,
                if i == self.selected {
                    theme.primary
                } else {
                    Color::rgb8(120, 120, 120)
                },
                16.0,
                parley::FontWeight::NORMAL,
                false,
            );

            // Label
            painter.draw_text(
                label,
                28.0,
                y + 18.0,
                theme.on_background,
                13.0,
                parley::FontWeight::NORMAL,
                false,
            );
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Down(e) = event {
            let idx = (e.pos.y / 28.0) as usize;
            if idx < self.options.len() && idx != self.selected {
                self.selected = idx;
                if let Some(ref on_change) = self.on_change {
                    on_change(self.selected);
                }
            }
        }
    }
}
