//! SizedBox widget — constrains child to a fixed size.

use kurbo::{Axis, Size};
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};

/// A widget that forces its child to a specific size.
pub struct SizedBox {
    child: Option<WidgetPod>,
    width: Option<f64>,
    height: Option<f64>,
}

impl SizedBox {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Some(WidgetPod::new(child)),
            width: None,
            height: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            child: None,
            width: None,
            height: None,
        }
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Creates a SizedBox that expands to fill available space.
    pub fn expand(child: impl Widget + 'static) -> Self {
        Self::new(child).with_width(f64::MAX).with_height(f64::MAX)
    }
}

impl Widget for SizedBox {
    fn measure(&mut self, ctx: &mut MeasureCtx, axis: Axis, _available: f64) -> f64 {
        match axis {
            Axis::Horizontal => self.width.unwrap_or_else(|| {
                self.child
                    .as_mut()
                    .map(|c| {
                        let mut child_ctx = MeasureCtx {
                            state: &mut c.state,
                            global: ctx.global,
                        };
                        c.widget.measure(&mut child_ctx, axis, _available)
                    })
                    .unwrap_or(0.0)
            }),
            Axis::Vertical => self.height.unwrap_or_else(|| {
                self.child
                    .as_mut()
                    .map(|c| {
                        let mut child_ctx = MeasureCtx {
                            state: &mut c.state,
                            global: ctx.global,
                        };
                        c.widget.measure(&mut child_ctx, axis, _available)
                    })
                    .unwrap_or(0.0)
            }),
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        if let Some(child) = &mut self.child {
            let child_size = Size::new(
                self.width.unwrap_or(size.width),
                self.height.unwrap_or(size.height),
            );
            child.state.size = child_size;
            let mut child_ctx = LayoutCtx {
                state: &mut child.state,
                global: ctx.global,
            };
            child.widget.layout(&mut child_ctx, child_size);
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        if let Some(child) = &mut self.child {
            let theme = ctx.theme().clone();
            let mut child_ctx = PaintCtx {
                state: &mut child.state,
                global: ctx.global,
                theme,
            };
            child.widget.paint(&mut child_ctx, scene);
        }
    }

    fn children(&self) -> Vec<crate::core::types::WidgetId> {
        self.child
            .as_ref()
            .map(|c| vec![c.state.id])
            .unwrap_or_default()
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        self.child
            .as_mut()
            .and_then(|c| if c.state.id == id { Some(c) } else { None })
    }
}
