//! Portal widget — a scrollable viewport for content larger than its bounds.

use kurbo::{Axis, Point, Rect, Size};
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A scrollable viewport.
pub struct Portal {
    child: WidgetPod,
    scroll_offset: Point,
    content_size: Size,
    viewport_size: Size,
    scrollbar_hovered: bool,
}

impl Portal {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: WidgetPod::new(child),
            scroll_offset: Point::ZERO,
            content_size: Size::ZERO,
            viewport_size: Size::ZERO,
            scrollbar_hovered: false,
        }
    }

    fn scrollbar_width() -> f64 {
        8.0
    }

    fn needs_vertical_scroll(&self) -> bool {
        self.content_size.height > self.viewport_size.height
    }

    fn max_scroll_y(&self) -> f64 {
        (self.content_size.height - self.viewport_size.height).max(0.0)
    }
}

impl Widget for Portal {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        // Portal takes whatever space is offered
        let _ = axis;
        available
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        self.viewport_size = size;

        // Measure child with unconstrained vertical space for scrolling
        let child_available_width = size.width - Self::scrollbar_width();

        let child_width = {
            let mut child_ctx = MeasureCtx {
                state: &mut self.child.state,
                global: ctx.global,
            };
            self.child
                .widget
                .measure(&mut child_ctx, Axis::Horizontal, child_available_width)
        };

        let child_height = {
            let mut child_ctx = MeasureCtx {
                state: &mut self.child.state,
                global: ctx.global,
            };
            self.child
                .widget
                .measure(&mut child_ctx, Axis::Vertical, f64::MAX)
        };

        self.content_size = Size::new(child_width, child_height);
        self.child.state.size = self.content_size;

        let mut child_ctx = LayoutCtx {
            state: &mut self.child.state,
            global: ctx.global,
        };
        self.child.widget.layout(&mut child_ctx, self.content_size);
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;

        // Clip to viewport
        let clip_rect = Rect::from_origin_size(Point::ZERO, size);

        // Translate for scroll offset
        self.child.state.position = Point::new(0.0, -self.scroll_offset.y);

        // Paint child directly into scene
        {
            let theme = ctx.theme().clone();
            let mut child_ctx = PaintCtx {
                state: &mut self.child.state,
                global: ctx.global,
                theme,
            };
            self.child.widget.paint(&mut child_ctx, scene);
        }

        // Now draw clip border and scrollbar using a fresh painter
        {
            let mut painter = Painter::new(scene);
            // Re-apply clip (draw over for scrollbar area)
            painter.push_clip(clip_rect);

            // Draw scrollbar if needed
            if self.needs_vertical_scroll() {
                let sb_width = Self::scrollbar_width();
                let sb_track_height = size.height;
                let sb_thumb_height = (sb_track_height * size.height / self.content_size.height)
                    .max(20.0)
                    .min(sb_track_height);
                let sb_thumb_y = if self.max_scroll_y() > 0.0 {
                    (self.scroll_offset.y / self.max_scroll_y())
                        * (sb_track_height - sb_thumb_height)
                } else {
                    0.0
                };

                let sb_x = size.width - sb_width;
                let thumb_rect = Rect::new(
                    sb_x,
                    sb_thumb_y,
                    sb_x + sb_width,
                    sb_thumb_y + sb_thumb_height,
                );

                let thumb_color = if self.scrollbar_hovered {
                    theme.secondary
                } else {
                    theme.disabled
                };

                painter.fill_rounded_rect(thumb_rect, thumb_color, sb_width / 2.0);
            }

            painter.pop_clip();
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Scroll(scroll_evt) => {
                self.scroll_offset.y = (self.scroll_offset.y + scroll_evt.delta.y * 40.0)
                    .clamp(0.0, self.max_scroll_y());
            }
            PointerEvent::Enter => {
                self.scrollbar_hovered = true;
            }
            PointerEvent::Leave => {
                self.scrollbar_hovered = false;
            }
            _ => {}
        }
    }

    fn children(&self) -> Vec<crate::core::types::WidgetId> {
        vec![self.child.state.id]
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        if self.child.state.id == id {
            Some(&mut self.child)
        } else {
            None
        }
    }
}
