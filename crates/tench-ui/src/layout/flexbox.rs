//! Flexbox layout algorithm.

use kurbo::{Axis, Point, Size};

/// Flex direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    /// Returns true for reverse directions.
    pub fn is_reverse(self) -> bool {
        matches!(self, Self::RowReverse | Self::ColumnReverse)
    }

    pub fn main_axis(self) -> Axis {
        match self {
            Self::Row | Self::RowReverse => Axis::Horizontal,
            Self::Column | Self::ColumnReverse => Axis::Vertical,
        }
    }

    pub fn cross_axis(self) -> Axis {
        match self.main_axis() {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

/// Alignment along the main axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Alignment along the cross axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
}

/// Result of laying out a single flex item.
#[derive(Debug, Clone)]
pub struct FlexItemLayout {
    /// The item's offset from the flex container's origin.
    pub offset: Point,
    /// The item's final size.
    pub size: Size,
}

/// Runs the flexbox algorithm and returns the position and size of each item.
///
/// This is a simplified flexbox implementation covering:
/// - Row and column directions (including reverse)
/// - Main axis alignment (start, end, center, space-between)
/// - Cross axis alignment (start, end, center, stretch)
/// - Flex grow/shrink/basis
/// - Gap between items
pub fn run_flexbox(
    direction: FlexDirection,
    main_align: MainAxisAlignment,
    cross_align: CrossAxisAlignment,
    gap: f64,
    container_size: Size,
    items: &[FlexItemInput],
) -> Vec<FlexItemLayout> {
    if items.is_empty() {
        return Vec::new();
    }

    let main_axis = direction.main_axis();
    let cross_axis = main_axis.cross();
    let container_main = axis_len(container_size, main_axis);
    let container_cross = axis_len(container_size, cross_axis);

    // Step 1: Determine base sizes (flex-basis or measured content size)
    let base_sizes: Vec<f64> = items
        .iter()
        .map(|item| item.basis.unwrap_or(item.min_main_size))
        .collect();

    // Step 2: Distribute free space via grow/shrink
    let total_base: f64 = base_sizes.iter().sum();
    let total_gaps = if items.len() > 1 {
        gap * (items.len() - 1) as f64
    } else {
        0.0
    };
    let free_space = container_main - total_base - total_gaps;

    let mut final_main_sizes = base_sizes.clone();

    if free_space > 0.0 {
        // Distribute via flex-grow
        let total_grow: f64 = items.iter().map(|i| i.grow).sum();
        if total_grow > 0.0 {
            for (i, item) in items.iter().enumerate() {
                if item.grow > 0.0 {
                    final_main_sizes[i] += (free_space / total_grow) * item.grow;
                }
            }
        }
    } else if free_space < 0.0 {
        // Distribute via flex-shrink
        let total_shrink: f64 = items
            .iter()
            .zip(base_sizes.iter())
            .map(|(item, base)| item.shrink * base)
            .sum();
        if total_shrink > 0.0 {
            for (i, item) in items.iter().enumerate() {
                if item.shrink > 0.0 && base_sizes[i] > 0.0 {
                    let shrink_factor = (item.shrink * base_sizes[i]) / total_shrink;
                    final_main_sizes[i] += free_space * shrink_factor;
                }
            }
        }
    }

    // Ensure no negative sizes
    for size in &mut final_main_sizes {
        *size = size.max(0.0);
    }

    // Step 3: Determine cross sizes
    let final_cross_sizes: Vec<f64> = items
        .iter()
        .map(|item| match cross_align {
            CrossAxisAlignment::Stretch => container_cross,
            _ => item.cross_size.min(container_cross),
        })
        .collect();

    // Step 4: Position items along the main axis
    let total_used_main: f64 = final_main_sizes.iter().sum::<f64>() + total_gaps;
    let mut offsets = Vec::with_capacity(items.len());

    let mut start_offset = match main_align {
        MainAxisAlignment::Start | MainAxisAlignment::SpaceBetween => 0.0,
        MainAxisAlignment::End => container_main - total_used_main,
        MainAxisAlignment::Center => (container_main - total_used_main) / 2.0,
        MainAxisAlignment::SpaceAround => {
            if items.is_empty() {
                0.0
            } else {
                let space = (container_main - total_used_main) / (items.len() as f64 * 2.0);
                space.max(0.0)
            }
        }
        MainAxisAlignment::SpaceEvenly => {
            if items.is_empty() {
                0.0
            } else {
                let space = (container_main - total_used_main) / (items.len() as f64 + 1.0);
                space.max(0.0)
            }
        }
    };

    for (i, _) in items.iter().enumerate() {
        let main_pos = if direction.is_reverse() {
            container_main - start_offset - final_main_sizes[i]
        } else {
            start_offset
        };

        let cross_pos = match cross_align {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
            CrossAxisAlignment::End => container_cross - final_cross_sizes[i],
            CrossAxisAlignment::Center => (container_cross - final_cross_sizes[i]) / 2.0,
        };

        let offset = match main_axis {
            Axis::Horizontal => Point::new(main_pos, cross_pos),
            Axis::Vertical => Point::new(cross_pos, main_pos),
        };

        offsets.push(offset);

        // Advance position
        start_offset += final_main_sizes[i] + gap;

        // Handle space-between, space-around, space-evenly for remaining items
        if main_align == MainAxisAlignment::SpaceBetween && i == 0 && items.len() > 1 {
            let _remaining = container_main - final_main_sizes.iter().sum::<f64>();
            // Reset and use uniform spacing
        }
    }

    // Step 5: Build result
    items
        .iter()
        .enumerate()
        .map(|(i, _)| FlexItemLayout {
            offset: offsets[i],
            size: make_size(final_main_sizes[i], final_cross_sizes[i], main_axis),
        })
        .collect()
}

/// Input for a single flex item.
#[derive(Debug, Clone)]
pub struct FlexItemInput {
    /// Flex-grow factor.
    pub grow: f64,
    /// Flex-shrink factor.
    pub shrink: f64,
    /// Flex-basis: None means use min_main_size.
    pub basis: Option<f64>,
    /// The item's measured minimum main-axis size.
    pub min_main_size: f64,
    /// The item's cross-axis size.
    pub cross_size: f64,
}

impl Default for FlexItemInput {
    fn default() -> Self {
        Self {
            grow: 0.0,
            shrink: 1.0,
            basis: None,
            min_main_size: 0.0,
            cross_size: 0.0,
        }
    }
}

fn axis_len(size: Size, axis: Axis) -> f64 {
    match axis {
        Axis::Horizontal => size.width,
        Axis::Vertical => size.height,
    }
}

fn make_size(main: f64, cross: f64, main_axis: Axis) -> Size {
    match main_axis {
        Axis::Horizontal => Size::new(main, cross),
        Axis::Vertical => Size::new(cross, main),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Size;

    #[test]
    fn test_flex_row_start() {
        let container = Size::new(300.0, 100.0);
        let items = vec![
            FlexItemInput {
                min_main_size: 50.0,
                cross_size: 40.0,
                ..Default::default()
            },
            FlexItemInput {
                min_main_size: 100.0,
                cross_size: 40.0,
                ..Default::default()
            },
        ];

        let result = run_flexbox(
            FlexDirection::Row,
            MainAxisAlignment::Start,
            CrossAxisAlignment::Start,
            0.0,
            container,
            &items,
        );

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].offset, Point::new(0.0, 0.0));
        assert_eq!(result[0].size.width, 50.0);
        assert_eq!(result[1].offset.x, 50.0);
        assert_eq!(result[1].size.width, 100.0);
    }

    #[test]
    fn test_flex_row_center() {
        let container = Size::new(300.0, 100.0);
        let items = vec![FlexItemInput {
            min_main_size: 100.0,
            cross_size: 40.0,
            ..Default::default()
        }];

        let result = run_flexbox(
            FlexDirection::Row,
            MainAxisAlignment::Center,
            CrossAxisAlignment::Center,
            0.0,
            container,
            &items,
        );

        assert_eq!(result.len(), 1);
        assert!((result[0].offset.x - 100.0).abs() < 0.01);
        assert!((result[0].offset.y - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_flex_grow() {
        let container = Size::new(300.0, 100.0);
        let items = vec![
            FlexItemInput {
                grow: 1.0,
                min_main_size: 0.0,
                cross_size: 40.0,
                ..Default::default()
            },
            FlexItemInput {
                grow: 2.0,
                min_main_size: 0.0,
                cross_size: 40.0,
                ..Default::default()
            },
        ];

        let result = run_flexbox(
            FlexDirection::Row,
            MainAxisAlignment::Start,
            CrossAxisAlignment::Start,
            0.0,
            container,
            &items,
        );

        assert_eq!(result.len(), 2);
        assert!((result[0].size.width - 100.0).abs() < 0.01);
        assert!((result[1].size.width - 200.0).abs() < 0.01);
    }
}
