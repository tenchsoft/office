//! Drawer widget.

use kurbo::{Axis, Rect, Size};
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawerSide {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct Drawer {
    side: DrawerSide,
    open: bool,
    extent: f64,
}

impl Drawer {
    pub fn new(side: DrawerSide) -> Self {
        Self {
            side,
            open: true,
            extent: 280.0,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn extent(mut self, extent: f64) -> Self {
        self.extent = extent.max(0.0);
        self
    }

    pub fn panel_rect(&self, size: Size) -> Rect {
        if !self.open {
            return Rect::ZERO;
        }
        match self.side {
            DrawerSide::Left => Rect::new(0.0, 0.0, self.extent.min(size.width), size.height),
            DrawerSide::Right => Rect::new(
                (size.width - self.extent).max(0.0),
                0.0,
                size.width,
                size.height,
            ),
            DrawerSide::Top => Rect::new(0.0, 0.0, size.width, self.extent.min(size.height)),
            DrawerSide::Bottom => Rect::new(
                0.0,
                (size.height - self.extent).max(0.0),
                size.width,
                size.height,
            ),
        }
    }
}

impl Widget for Drawer {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        available
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        if !self.open {
            return;
        }
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        p.fill_rect(
            Rect::new(0.0, 0.0, size.width, size.height),
            theme.background,
        );
        p.fill_rect(self.panel_rect(size), theme.surface);
    }
}
