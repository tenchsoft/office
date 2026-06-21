//! Layout engine — orchestrates measure and layout passes.

pub mod flexbox;

use kurbo::Size;

use crate::core::{GlobalState, LayoutCtx, MeasureCtx, WidgetPod};

/// Constraints passed from parent to child during layout.
#[derive(Clone, Copy, Debug)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    pub fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    pub fn unbounded() -> Self {
        Self {
            min: Size::ZERO,
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min.width, self.max.width),
            size.height.clamp(self.min.height, self.max.height),
        )
    }
}

/// The layout pass walks the widget tree: bottom-up (measure) then top-down (layout).
///
/// Container widgets (Flex, Grid, Tabs, etc.) handle their own children's layout
/// internally via their `measure()` / `layout()` methods. This pass orchestrates
/// the root widget and passes the shared `GlobalState` through.
pub struct LayoutPass;

impl LayoutPass {
    /// Runs the layout pass on the root widget using a shared `GlobalState`.
    pub fn run(root: &mut WidgetPod, available: Size, global: &mut GlobalState) {
        // Measure phase: determine preferred sizes
        let preferred_width = {
            let mut ctx = MeasureCtx {
                state: &mut root.state,
                global,
            };
            root.widget
                .measure(&mut ctx, kurbo::Axis::Horizontal, available.width)
        };

        let preferred_height = {
            let mut ctx = MeasureCtx {
                state: &mut root.state,
                global,
            };
            root.widget
                .measure(&mut ctx, kurbo::Axis::Vertical, available.height)
        };

        let final_size = Size::new(
            preferred_width.min(available.width),
            preferred_height.min(available.height),
        );

        // Layout phase: assign sizes and positions
        root.state.size = final_size;
        root.state.needs_layout = false;

        let mut ctx = LayoutCtx {
            state: &mut root.state,
            global,
        };
        root.widget.layout(&mut ctx, final_size);
    }
}
