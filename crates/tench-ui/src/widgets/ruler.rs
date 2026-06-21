//! Ruler widget.

use kurbo::{Axis, Point, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct Ruler {
    horizontal: bool,
    pixels_per_unit: f64,
}

impl Ruler {
    pub fn horizontal() -> Self {
        Self {
            horizontal: true,
            pixels_per_unit: 72.0,
        }
    }

    pub fn vertical() -> Self {
        Self {
            horizontal: false,
            pixels_per_unit: 72.0,
        }
    }

    pub fn pixels_per_unit(mut self, pixels: f64) -> Self {
        self.pixels_per_unit = pixels.max(1.0);
        self
    }
}

impl Widget for Ruler {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match (self.horizontal, axis) {
            (true, Axis::Vertical) | (false, Axis::Horizontal) => 22.0,
            _ => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        p.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), theme.surface);
        let max = if self.horizontal {
            size.width
        } else {
            size.height
        };
        let mut offset = 0.0;
        let mut unit = 0;
        while offset < max {
            let major = unit % 2 == 0;
            if self.horizontal {
                p.draw_line(
                    Point::new(offset, size.height - if major { 12.0 } else { 6.0 }),
                    Point::new(offset, size.height),
                    theme.secondary,
                    1.0,
                );
                if major {
                    p.draw_text(
                        &format!("{}", unit / 2),
                        offset + 2.0,
                        10.0,
                        theme.secondary,
                        9.0,
                        FontWeight::NORMAL,
                        false,
                    );
                }
            } else {
                p.draw_line(
                    Point::new(size.width - if major { 12.0 } else { 6.0 }, offset),
                    Point::new(size.width, offset),
                    theme.secondary,
                    1.0,
                );
            }
            offset += self.pixels_per_unit / 2.0;
            unit += 1;
        }
    }
}
