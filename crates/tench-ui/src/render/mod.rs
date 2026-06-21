//! Rendering pipeline — walks the widget tree and produces a Vello Scene.

pub mod image_cache;
pub mod painter;

pub use image_cache::ImageCache;
pub use painter::{GradientDirection, Painter, TextCache};

use kurbo::Size;

use crate::core::{GlobalState, PaintCtx, WidgetPod};
use crate::theme::Theme;

/// The render pass walks the widget tree and calls `paint()` on each widget.
///
/// Container widgets (Flex, Grid, Tabs, etc.) handle their own children's painting
/// internally. This pass orchestrates the root widget and passes the shared
/// `GlobalState` and `Theme` through.
pub struct RenderPass;

impl RenderPass {
    /// Runs the render pass on the root widget, producing a Vello Scene.
    pub fn run(
        root: &mut WidgetPod,
        size: Size,
        global: &mut GlobalState,
        theme: &Theme,
    ) -> vello::Scene {
        let mut scene = vello::Scene::new();
        {
            let mut painter = Painter::new(&mut scene);
            painter.fill_background(size, crate::core::types::Color::rgb8(0x1E, 0x1E, 0x2E));
        }

        // Paint the root widget
        root.state.needs_paint = false;
        {
            let mut paint_ctx = PaintCtx {
                state: &mut root.state,
                global,
                theme: theme.clone(),
            };
            root.widget.paint(&mut paint_ctx, &mut scene);
        }

        scene
    }
}
