//! Split widget — a horizontal or vertical split pane with a draggable divider.

use kurbo::{Axis, Point, Rect, Size};
use vello::Scene;

use crate::core::events::{PointerButton, PointerEvent};
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// Direction of the split.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// A split pane with a draggable divider.
pub struct Split {
    direction: SplitDirection,
    ratio: f64,
    dragging: bool,
    min_ratio: f64,
    max_ratio: f64,
    child_a: WidgetPod,
    child_b: WidgetPod,
}

impl Split {
    pub fn new(
        direction: SplitDirection,
        child_a: impl Widget + 'static,
        child_b: impl Widget + 'static,
    ) -> Self {
        Self {
            direction,
            ratio: 0.5,
            dragging: false,
            min_ratio: 0.1,
            max_ratio: 0.9,
            child_a: WidgetPod::new(child_a),
            child_b: WidgetPod::new(child_b),
        }
    }

    pub fn with_ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio.clamp(self.min_ratio, self.max_ratio);
        self
    }

    fn divider_thickness() -> f64 {
        6.0
    }

    fn axis(&self) -> Axis {
        match self.direction {
            SplitDirection::Horizontal => Axis::Horizontal,
            SplitDirection::Vertical => Axis::Vertical,
        }
    }
}

impl Widget for Split {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        // Split takes whatever space is offered
        let _ = axis;
        available
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        let dt = Self::divider_thickness();
        let main_axis = self.axis();

        let total = match main_axis {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        };

        let a_size_main = (total - dt) * self.ratio;
        let b_size_main = (total - dt) - a_size_main;

        let (a_size, b_size, a_pos, b_pos) = match main_axis {
            Axis::Horizontal => (
                Size::new(a_size_main, size.height),
                Size::new(b_size_main, size.height),
                Point::new(0.0, 0.0),
                Point::new(a_size_main + dt, 0.0),
            ),
            Axis::Vertical => (
                Size::new(size.width, a_size_main),
                Size::new(size.width, b_size_main),
                Point::new(0.0, 0.0),
                Point::new(0.0, a_size_main + dt),
            ),
        };

        self.child_a.state.position = a_pos;
        self.child_a.state.size = a_size;
        self.child_b.state.position = b_pos;
        self.child_b.state.size = b_size;

        let mut ctx_a = LayoutCtx {
            state: &mut self.child_a.state,
            global: ctx.global,
        };
        self.child_a.widget.layout(&mut ctx_a, a_size);

        let mut ctx_b = LayoutCtx {
            state: &mut self.child_b.state,
            global: ctx.global,
        };
        self.child_b.widget.layout(&mut ctx_b, b_size);
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let _theme = Theme::default();
        let size = ctx.state.size;
        let dt = Self::divider_thickness();
        let main_axis = self.axis();

        // Paint children
        let theme = ctx.theme().clone();
        {
            let mut child_ctx = PaintCtx {
                state: &mut self.child_a.state,
                global: ctx.global,
                theme: theme.clone(),
            };
            self.child_a.widget.paint(&mut child_ctx, scene);
        }
        {
            let mut child_ctx = PaintCtx {
                state: &mut self.child_b.state,
                global: ctx.global,
                theme: theme.clone(),
            };
            self.child_b.widget.paint(&mut child_ctx, scene);
        }

        // Paint divider
        let mut painter = Painter::new(scene);
        let divider_rect = match main_axis {
            Axis::Horizontal => {
                let x = size.width * self.ratio - dt / 2.0;
                Rect::new(x, 0.0, x + dt, size.height)
            }
            Axis::Vertical => {
                let y = size.height * self.ratio - dt / 2.0;
                Rect::new(0.0, y, size.width, y + dt)
            }
        };
        let divider_color = if self.dragging {
            theme.primary
        } else {
            theme.border
        };
        painter.fill_rect(divider_rect, divider_color);
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        let size = ctx.state.size;
        let dt = Self::divider_thickness();
        let main_axis = self.axis();

        match event {
            PointerEvent::Down(btn) if btn.button == PointerButton::Primary => {
                let pos_main = match main_axis {
                    Axis::Horizontal => btn.pos.x,
                    Axis::Vertical => btn.pos.y,
                };
                let divider_center = match main_axis {
                    Axis::Horizontal => size.width * self.ratio,
                    Axis::Vertical => size.height * self.ratio,
                };
                if (pos_main - divider_center).abs() < dt * 2.0 {
                    self.dragging = true;
                }
            }
            PointerEvent::Up(_) => {
                self.dragging = false;
            }
            PointerEvent::Move(move_evt) if self.dragging => {
                let total = match main_axis {
                    Axis::Horizontal => size.width,
                    Axis::Vertical => size.height,
                };
                let pos_main = match main_axis {
                    Axis::Horizontal => move_evt.pos.x,
                    Axis::Vertical => move_evt.pos.y,
                };
                self.ratio = (pos_main / total).clamp(self.min_ratio, self.max_ratio);
            }
            _ => {}
        }
    }

    fn children(&self) -> Vec<crate::core::types::WidgetId> {
        vec![self.child_a.state.id, self.child_b.state.id]
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        if self.child_a.state.id == id {
            Some(&mut self.child_a)
        } else if self.child_b.state.id == id {
            Some(&mut self.child_b)
        } else {
            None
        }
    }
}
