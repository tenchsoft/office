//! Separator widget for drawing horizontal or vertical dividers.

use kurbo::{Axis, Rect, Size};
use vello::Scene;

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeparatorDirection {
    Horizontal,
    Vertical,
}

pub struct Separator {
    direction: SeparatorDirection,
    thickness: f64,
    color: Option<Color>,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self::new(SeparatorDirection::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(SeparatorDirection::Vertical)
    }

    pub fn new(direction: SeparatorDirection) -> Self {
        Self {
            direction,
            thickness: 1.0,
            color: None,
        }
    }

    pub fn thickness(mut self, thickness: f64) -> Self {
        self.thickness = thickness.max(1.0);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for Separator {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match (self.direction, axis) {
            (SeparatorDirection::Horizontal, Axis::Vertical)
            | (SeparatorDirection::Vertical, Axis::Horizontal) => self.thickness,
            _ => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let size = ctx.state.size;
        let color = self.color.unwrap_or_else(|| Theme::default().border);
        let rect = match self.direction {
            SeparatorDirection::Horizontal => {
                let y = (size.height - self.thickness) * 0.5;
                Rect::new(0.0, y, size.width, y + self.thickness)
            }
            SeparatorDirection::Vertical => {
                let x = (size.width - self.thickness) * 0.5;
                Rect::new(x, 0.0, x + self.thickness, size.height)
            }
        };
        Painter::new(scene).fill_rect(rect, color);
    }
}
