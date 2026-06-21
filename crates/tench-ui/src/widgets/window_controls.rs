//! Platform window caption controls (minimize / maximize-restore / close).
//!
//! These are free painting + hit-testing helpers used by each product app's
//! menu bar. Apps paint the controls at the top-right of the menu bar and
//! submit [`crate::core::events::WindowAction`]s on click. The native backend
//! drains and executes the actions (see `platform::native`).
//!
//! The controls are kept framework-neutral (caption buttons look the same
//! across all products) so the helpers live in the shared crate.

use crate::core::types::Color;
use crate::render::Painter;
use kurbo::{Point, Rect};

/// Width of the resize hit zone along each window edge, in logical pixels.
pub const WINDOW_RESIZE_EDGE: f64 = 6.0;

/// Half-size of the resize hit zone at each window corner, in logical pixels.
/// Larger than `WINDOW_RESIZE_EDGE` so corners are easy to grab — without this
/// the corner is only `EDGE × EDGE` and the pointer almost always lands on an
/// adjacent edge instead of the diagonal.
pub const WINDOW_RESIZE_CORNER: f64 = 12.0;

/// Conservative height of the caption-button zone at the top-right corner.
/// The native backend excludes this rectangle from edge-resize hit-testing so
/// the caption buttons (which span each app's header) keep priority. Covers
/// the tallest header in the suite (sheets ≈ 56px) plus margin.
const CAPTION_ZONE_H: f64 = 80.0;

/// Which window edge / corner a pointer is over, for native resize.
///
/// Mirrors winit's `ResizeDirection` without coupling the widget layer to
/// winit. The native backend maps this to `ResizeDirection` + `CursorIcon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowResizeEdge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Hit-test the window border for resize. Returns the edge / corner under
/// `(x, y)`, or `None` when the point is in the interior or inside the
/// caption-button zone (top-right), where caption clicks take priority.
///
/// `width` / `height` are the current window size in logical pixels.
pub fn window_resize_edge_at(x: f64, y: f64, width: f64, height: f64) -> Option<WindowResizeEdge> {
    // Exclude the caption-button rectangle (top-right) entirely.
    if x > width - WINDOW_CONTROLS_W && y < CAPTION_ZONE_H {
        return None;
    }

    // Corner zones (larger) are checked first so a corner grab wins over the
    // adjacent edges — this is what makes diagonal resize reachable. Without
    // this, the corner would only be EDGE × EDGE and the pointer almost always
    // lands on an adjacent edge instead.
    let corner_left = x < WINDOW_RESIZE_CORNER;
    let corner_right = x > width - WINDOW_RESIZE_CORNER;
    let corner_top = y < WINDOW_RESIZE_CORNER;
    let corner_bottom = y > height - WINDOW_RESIZE_CORNER;
    if corner_left && corner_top {
        return Some(WindowResizeEdge::NorthWest);
    }
    if corner_right && corner_top {
        return Some(WindowResizeEdge::NorthEast);
    }
    if corner_left && corner_bottom {
        return Some(WindowResizeEdge::SouthWest);
    }
    if corner_right && corner_bottom {
        return Some(WindowResizeEdge::SouthEast);
    }

    // Edge zones (narrower).
    let near_left = x < WINDOW_RESIZE_EDGE;
    let near_right = x > width - WINDOW_RESIZE_EDGE;
    let near_top = y < WINDOW_RESIZE_EDGE;
    let near_bottom = y > height - WINDOW_RESIZE_EDGE;
    match (near_left, near_right, near_top, near_bottom) {
        (true, false, false, false) => Some(WindowResizeEdge::West),
        (false, true, false, false) => Some(WindowResizeEdge::East),
        (false, false, true, false) => Some(WindowResizeEdge::North),
        (false, false, false, true) => Some(WindowResizeEdge::South),
        _ => None,
    }
}

/// Width of a single caption button (minimize / maximize / close).
pub const WINDOW_CONTROL_BTN_W: f64 = 46.0;

/// Total width occupied by the three caption buttons on the right of the
/// menu bar. Apps reserve this much horizontal space (shift other right-side
/// chrome content left of `window_width - WINDOW_CONTROLS_W`).
pub const WINDOW_CONTROLS_W: f64 = 3.0 * WINDOW_CONTROL_BTN_W;

/// Which caption button is under the pointer / targeted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowControl {
    /// Minimize to taskbar / dock.
    Minimize,
    /// Maximize / restore toggle.
    MaximizeRestore,
    /// Close the window.
    Close,
}

// Caption button colors. Neutral enough to sit on any product's dark menu bar.
const ICON: Color = Color::rgba8(0xE4, 0xE4, 0xE4, 0xFF);
const HOVER_BG: Color = Color::rgba8(0x2A, 0x2A, 0x2A, 0xFF);
const CLOSE_HOVER_BG: Color = Color::rgba8(0xE8, 0x11, 0x23, 0xFF);
const CLOSE_HOVER_ICON: Color = Color::rgba8(0xFF, 0xFF, 0xFF, 0xFF);

/// Return the rectangle for a specific caption button.
///
/// `window_width` is the current window width in logical pixels. The close
/// button is flush against the right edge; the buttons lay out right-to-left.
pub fn control_rect(window_width: f64, menu_bar_h: f64, control: WindowControl) -> Rect {
    let idx = match control {
        WindowControl::Close => 0,
        WindowControl::MaximizeRestore => 1,
        WindowControl::Minimize => 2,
    };
    let x1 = window_width - idx as f64 * WINDOW_CONTROL_BTN_W;
    let x0 = x1 - WINDOW_CONTROL_BTN_W;
    Rect::new(x0, 0.0, x1, menu_bar_h)
}

/// Hit-test the caption button zone. Returns the button under `(x, y)`, or
/// `None` if the point is outside the rightmost `WINDOW_CONTROLS_W` band or
/// above/below the menu bar.
pub fn window_control_at(
    x: f64,
    y: f64,
    window_width: f64,
    menu_bar_h: f64,
) -> Option<WindowControl> {
    if y < 0.0 || y > menu_bar_h {
        return None;
    }
    [
        WindowControl::Close,
        WindowControl::MaximizeRestore,
        WindowControl::Minimize,
    ]
    .into_iter()
    .find(|&control| control_rect(window_width, menu_bar_h, control).contains(Point::new(x, y)))
}

/// Paint the three caption buttons at the top-right of the menu bar.
///
/// `is_maximized` selects the maximize-vs-restore glyph. `hovered` is the
/// button currently under the pointer (if any) for hover feedback.
pub fn paint_window_controls(
    p: &mut Painter<'_>,
    window_width: f64,
    menu_bar_h: f64,
    is_maximized: bool,
    hovered: Option<WindowControl>,
) {
    for control in [
        WindowControl::Minimize,
        WindowControl::MaximizeRestore,
        WindowControl::Close,
    ] {
        let rect = control_rect(window_width, menu_bar_h, control);
        let is_hovered = hovered == Some(control);
        let (bg, icon) = match (control, is_hovered) {
            (WindowControl::Close, true) => (Some(CLOSE_HOVER_BG), CLOSE_HOVER_ICON),
            (_, true) => (Some(HOVER_BG), ICON),
            (_, false) => (None, ICON),
        };
        if let Some(bg) = bg {
            p.fill_rect(rect, bg);
        }
        paint_glyph(p, rect, control, icon, is_maximized);
    }
}

fn paint_glyph(
    p: &mut Painter<'_>,
    rect: Rect,
    control: WindowControl,
    icon: Color,
    is_maximized: bool,
) {
    let cx = rect.x0 + rect.width() / 2.0;
    let cy = rect.y0 + rect.height() / 2.0;
    match control {
        WindowControl::Minimize => {
            // Horizontal bar near the bottom-centre.
            let y = cy + 5.0;
            p.draw_line(Point::new(cx - 8.0, y), Point::new(cx + 8.0, y), icon, 1.0);
        }
        WindowControl::MaximizeRestore => {
            if is_maximized {
                // Restore: two overlapping squares.
                let front = Rect::new(cx - 7.0, cy - 7.0, cx + 6.0, cy + 6.0);
                let back = Rect::new(cx - 4.0, cy - 4.0, cx + 9.0, cy + 9.0);
                p.stroke_rounded_rect(front, icon, 1.0, 1.0);
                p.stroke_rounded_rect(back, icon, 1.0, 1.0);
            } else {
                // Maximize: a single square outline.
                let square = Rect::new(cx - 8.0, cy - 8.0, cx + 8.0, cy + 8.0);
                p.stroke_rounded_rect(square, icon, 1.0, 1.0);
            }
        }
        WindowControl::Close => {
            // An X.
            let r = 7.0;
            p.draw_line(
                Point::new(cx - r, cy - r),
                Point::new(cx + r, cy + r),
                icon,
                1.0,
            );
            p.draw_line(
                Point::new(cx - r, cy + r),
                Point::new(cx + r, cy - r),
                icon,
                1.0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test viewport. WINDOW_CONTROLS_W=138, CAPTION_ZONE_H=80, edge zone=6.
    const W: f64 = 1280.0;
    const H: f64 = 820.0;

    #[test]
    fn interior_point_is_not_resizable() {
        assert_eq!(window_resize_edge_at(640.0, 410.0, W, H), None);
    }

    #[test]
    fn just_outside_edge_is_not_resizable() {
        // 7px from the left is one pixel beyond the 6px edge zone.
        assert_eq!(window_resize_edge_at(7.0, 410.0, W, H), None);
    }

    #[test]
    fn four_edges_match_their_directions() {
        assert_eq!(
            window_resize_edge_at(3.0, 410.0, W, H),
            Some(WindowResizeEdge::West)
        );
        assert_eq!(
            window_resize_edge_at(W - 3.0, 410.0, W, H),
            Some(WindowResizeEdge::East)
        );
        assert_eq!(
            window_resize_edge_at(640.0, 3.0, W, H),
            Some(WindowResizeEdge::North)
        );
        assert_eq!(
            window_resize_edge_at(640.0, H - 3.0, W, H),
            Some(WindowResizeEdge::South)
        );
    }

    #[test]
    fn corners_match_their_diagonals() {
        assert_eq!(
            window_resize_edge_at(3.0, 3.0, W, H),
            Some(WindowResizeEdge::NorthWest)
        );
        assert_eq!(
            window_resize_edge_at(W - 3.0, H - 3.0, W, H),
            Some(WindowResizeEdge::SouthEast)
        );
        assert_eq!(
            window_resize_edge_at(3.0, H - 3.0, W, H),
            Some(WindowResizeEdge::SouthWest)
        );
    }

    #[test]
    fn caption_zone_top_right_is_excluded() {
        // The top-right corner belongs to the caption buttons, not resize.
        assert_eq!(window_resize_edge_at(W - 3.0, 3.0, W, H), None);
        // Right edge within the caption zone height is also excluded.
        assert_eq!(window_resize_edge_at(W - 3.0, 40.0, W, H), None);
        // Top edge within the caption zone width is also excluded.
        assert_eq!(window_resize_edge_at(W - 50.0, 3.0, W, H), None);
    }

    #[test]
    fn right_edge_below_caption_zone_resizes() {
        // y=100 is below the 80px caption exclusion → right edge resize works.
        assert_eq!(
            window_resize_edge_at(W - 3.0, 100.0, W, H),
            Some(WindowResizeEdge::East)
        );
    }

    #[test]
    fn enlarged_corner_zone_wins_over_adjacent_edge() {
        // (10, 4) is within the 12px corner but outside the 6px edge zone on x.
        // With a uniform 6px zone this would resolve to the top edge (North);
        // the enlarged corner must grab it as a diagonal instead.
        assert_eq!(
            window_resize_edge_at(10.0, 4.0, W, H),
            Some(WindowResizeEdge::NorthWest)
        );
        assert_eq!(
            window_resize_edge_at(4.0, 10.0, W, H),
            Some(WindowResizeEdge::NorthWest)
        );
        // Bottom-right corner enlarged zone.
        assert_eq!(
            window_resize_edge_at(W - 10.0, H - 4.0, W, H),
            Some(WindowResizeEdge::SouthEast)
        );
    }

    #[test]
    fn beyond_corner_zone_falls_back_to_edge() {
        // 13px from the corner on one axis is outside the 12px corner zone, so
        // it must fall back to the edge (not the diagonal).
        assert_eq!(
            window_resize_edge_at(13.0, 3.0, W, H),
            Some(WindowResizeEdge::North)
        );
    }
}
