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
