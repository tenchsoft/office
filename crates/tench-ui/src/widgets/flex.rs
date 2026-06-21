//! Flex container widget — arranges children in a row or column.

use kurbo::{Axis, Point, Size};
use smallvec::SmallVec;
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};

/// Direction of flex layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
}

/// A flex container that arranges children horizontally or vertically.
pub struct Flex {
    direction: FlexDirection,
    children: Vec<WidgetPod>,
    gap: f64,
    cross_align: CrossAxisAlignment,
}

/// Cross-axis alignment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Extract the component of a Size along the given axis.
fn axis_size(axis: kurbo::Axis, size: Size) -> f64 {
    match axis {
        kurbo::Axis::Horizontal => size.width,
        kurbo::Axis::Vertical => size.height,
    }
}

impl Flex {
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            children: Vec::new(),
            gap: 0.0,
            cross_align: CrossAxisAlignment::Start,
        }
    }

    pub fn row() -> Self {
        Self::new(FlexDirection::Row)
    }

    pub fn column() -> Self {
        Self::new(FlexDirection::Column)
    }

    pub fn with_gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_cross_align(mut self, align: CrossAxisAlignment) -> Self {
        self.cross_align = align;
        self
    }

    pub fn add_child(&mut self, child: impl Widget + 'static) {
        self.children.push(WidgetPod::new(child));
    }

    pub fn with_child(mut self, child: impl Widget + 'static) -> Self {
        self.add_child(child);
        self
    }

    fn main_axis(&self) -> Axis {
        match self.direction {
            FlexDirection::Row => Axis::Horizontal,
            FlexDirection::Column => Axis::Vertical,
        }
    }

    fn cross_axis(&self) -> Axis {
        match self.direction {
            FlexDirection::Row => Axis::Vertical,
            FlexDirection::Column => Axis::Horizontal,
        }
    }
}

impl Widget for Flex {
    fn measure(&mut self, ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        let gap = if self.children.len() > 1 {
            self.gap
        } else {
            0.0
        };

        if axis == self.main_axis() {
            // Sum of children sizes along main axis
            let total: f64 = self
                .children
                .iter_mut()
                .map(|child| {
                    let mut child_ctx = MeasureCtx {
                        state: &mut child.state,
                        global: ctx.global,
                    };
                    child.widget.measure(&mut child_ctx, axis, available)
                })
                .sum();
            total + gap * (self.children.len().saturating_sub(1)) as f64
        } else {
            // Max of children sizes along cross axis
            self.children
                .iter_mut()
                .map(|child| {
                    let mut child_ctx = MeasureCtx {
                        state: &mut child.state,
                        global: ctx.global,
                    };
                    child.widget.measure(&mut child_ctx, axis, available)
                })
                .fold(0.0_f64, f64::max)
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        let gap = if self.children.len() > 1 {
            self.gap
        } else {
            0.0
        };
        let main_axis = self.main_axis();
        let cross_axis = self.cross_axis();

        let total_gap = gap * (self.children.len().saturating_sub(1)) as f64;
        let available_main = axis_size(main_axis, size) - total_gap;
        let available_cross = axis_size(cross_axis, size);

        // First pass: measure all children
        let child_sizes: SmallVec<[f64; 8]> = self
            .children
            .iter_mut()
            .map(|child| {
                let mut child_ctx = MeasureCtx {
                    state: &mut child.state,
                    global: ctx.global,
                };
                child
                    .widget
                    .measure(&mut child_ctx, main_axis, available_main)
            })
            .collect();

        // Second pass: position children
        let mut offset = 0.0;
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_main = child_sizes[i];
            let child_cross = match self.cross_align {
                CrossAxisAlignment::Stretch => available_cross,
                _ => {
                    let mut child_ctx = MeasureCtx {
                        state: &mut child.state,
                        global: ctx.global,
                    };
                    child
                        .widget
                        .measure(&mut child_ctx, cross_axis, available_cross)
                }
            };

            let cross_offset = match self.cross_align {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::Center => (available_cross - child_cross) / 2.0,
                CrossAxisAlignment::End => available_cross - child_cross,
                CrossAxisAlignment::Stretch => 0.0,
            };

            let child_size = match self.direction {
                FlexDirection::Row => Size::new(child_main, child_cross),
                FlexDirection::Column => Size::new(child_cross, child_main),
            };

            let child_pos = match self.direction {
                FlexDirection::Row => Point::new(offset, cross_offset),
                FlexDirection::Column => Point::new(cross_offset, offset),
            };

            child.state.position = child_pos;
            child.state.size = child_size;

            let mut child_ctx = LayoutCtx {
                state: &mut child.state,
                global: ctx.global,
            };
            child.widget.layout(&mut child_ctx, child_size);

            offset += child_main + gap;
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        for child in &mut self.children {
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
        self.children.iter().map(|c| c.state.id).collect()
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        self.children.iter_mut().find(|c| c.state.id == id)
    }
}
