//! Dropdown — dropdown selector widget.

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Rect, Size};

/// A dropdown selector.
pub struct Dropdown {
    options: Vec<String>,
    selected_index: usize,
    open: bool,
    hovered_option: Option<usize>,
    on_change: Option<Box<dyn Fn(usize) + Send>>,
}

impl Dropdown {
    pub fn new(options: Vec<String>, selected: usize) -> Self {
        Self {
            options,
            selected_index: selected,
            open: false,
            hovered_option: None,
            on_change: None,
        }
    }

    pub fn on_change(mut self, f: impl Fn(usize) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn selected(&self) -> &str {
        self.options
            .get(self.selected_index)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
}

impl Widget for Dropdown {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => 160.0_f64.min(available),
            Axis::Vertical => {
                if self.open {
                    32.0 + self.options.len() as f64 * 28.0
                } else {
                    32.0
                }
            }
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        // Button area
        let btn_rect = Rect::from_origin_size(Point::ZERO, Size::new(size.width, 32.0));
        painter.fill_rounded_rect(btn_rect, Color::rgb8(50, 50, 50), 4.0);
        painter.stroke_rounded_rect(btn_rect, theme.border, 1.0, 4.0);

        // Selected text
        if let Some(label) = self.options.get(self.selected_index) {
            painter.draw_text(
                label,
                10.0,
                20.0,
                theme.on_background,
                13.0,
                parley::FontWeight::NORMAL,
                false,
            );
        }

        // Arrow
        let arrow = if self.open { "v" } else { ">" };
        painter.draw_text(
            arrow,
            size.width - 20.0,
            20.0,
            Color::rgb8(150, 150, 150),
            12.0,
            parley::FontWeight::NORMAL,
            false,
        );

        // Dropdown list
        if self.open {
            for (i, opt) in self.options.iter().enumerate() {
                let y = 32.0 + i as f64 * 28.0;
                let opt_rect = Rect::new(0.0, y, size.width, y + 28.0);

                if self.hovered_option == Some(i) {
                    painter.fill_rect(opt_rect, Color::rgb8(60, 60, 60));
                } else {
                    painter.fill_rect(opt_rect, Color::rgb8(40, 40, 40));
                }

                painter.draw_text(
                    opt,
                    10.0,
                    y + 18.0,
                    if i == self.selected_index {
                        theme.primary
                    } else {
                        theme.on_background
                    },
                    13.0,
                    parley::FontWeight::NORMAL,
                    false,
                );
            }
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Down(e) => {
                if e.pos.y < 32.0 {
                    self.open = !self.open;
                } else if self.open {
                    let idx = ((e.pos.y - 32.0) / 28.0) as usize;
                    if idx < self.options.len() {
                        self.selected_index = idx;
                        self.open = false;
                    }
                }
            }
            PointerEvent::Move(e) => {
                if self.open && e.pos.y >= 32.0 {
                    let idx = ((e.pos.y - 32.0) / 28.0) as usize;
                    self.hovered_option = Some(idx.min(self.options.len() - 1));
                } else {
                    self.hovered_option = None;
                }
            }
            _ => {}
        }
    }
}
