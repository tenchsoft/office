use super::state::SlidesState;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

/// Actions that toolbar buttons can trigger.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolbarAction {
    NewSlide,
    OpenFile,
    Save,
    Undo,
    Redo,
    InsertText,
    InsertShape,
    InsertImage,
    TogglePresentation,
    License,
}

/// A single item in the toolbar layout.
pub enum ToolbarItemKind {
    Button {
        label: &'static str,
        action: ToolbarAction,
    },
    Separator,
}

/// Returns the ordered list of toolbar items.
fn toolbar_items() -> Vec<ToolbarItemKind> {
    vec![
        ToolbarItemKind::Button {
            label: "New",
            action: ToolbarAction::NewSlide,
        },
        ToolbarItemKind::Button {
            label: "Open",
            action: ToolbarAction::OpenFile,
        },
        ToolbarItemKind::Button {
            label: "Save",
            action: ToolbarAction::Save,
        },
        ToolbarItemKind::Button {
            label: "Undo",
            action: ToolbarAction::Undo,
        },
        ToolbarItemKind::Button {
            label: "Redo",
            action: ToolbarAction::Redo,
        },
        ToolbarItemKind::Separator,
        ToolbarItemKind::Button {
            label: "Text",
            action: ToolbarAction::InsertText,
        },
        ToolbarItemKind::Button {
            label: "Shape",
            action: ToolbarAction::InsertShape,
        },
        ToolbarItemKind::Button {
            label: "Image",
            action: ToolbarAction::InsertImage,
        },
        ToolbarItemKind::Separator,
        ToolbarItemKind::Button {
            label: "Play",
            action: ToolbarAction::TogglePresentation,
        },
        ToolbarItemKind::Separator,
        ToolbarItemKind::Button {
            label: "License",
            action: ToolbarAction::License,
        },
    ]
}

const BUTTON_W: f64 = 50.0;
const BUTTON_H_PAD: f64 = 58.0;
const SEP_W: f64 = 8.0;

/// Computes the layout: returns (label, x_start, x_end, action) for each button.
/// Separators are skipped.
pub fn toolbar_layout(rect: Rect) -> Vec<(&'static str, f64, f64, ToolbarAction)> {
    let mut result = Vec::new();
    let mut x = rect.x0 + 12.0;
    for item in &toolbar_items() {
        match item {
            ToolbarItemKind::Separator => {
                x += SEP_W;
            }
            ToolbarItemKind::Button { label, action } => {
                let x_start = x;
                let x_end = x + BUTTON_H_PAD;
                result.push((*label, x_start, x_end, *action));
                x = x_end;
            }
        }
    }
    result
}

pub fn paint_toolbar(state: &SlidesState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, theme.surface);
    let layout = toolbar_layout(rect);
    for (label, x_start, _x_end, _action) in &layout {
        let button = Rect::new(*x_start, rect.y0 + 6.0, *x_start + BUTTON_W, rect.y1 - 6.0);
        p.fill_rounded_rect(button, theme.background, 4.0);
        p.draw_text(
            label,
            *x_start + 8.0,
            rect.y0 + 22.0,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }
    // Notification label — only painted when the license is not active.
    // Click handling lives in mod.rs (looks up the same rect via
    // notification_label_rect).
    if let Some(msg) = notification_label_message(state) {
        let label_rect = notification_label_rect(rect);
        p.draw_text(
            msg,
            label_rect.x0,
            rect.y0 + 22.0,
            theme.primary,
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
    }
    p.draw_text(
        &format!(
            "{} slides | {}{} | {:.0}%",
            state.slides.len(),
            state.transition_name,
            if state.presenting {
                " | presenting"
            } else {
                ""
            },
            state.zoom.level * 100.0
        ),
        rect.x1 - 420.0,
        rect.y0 + 22.0,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
    p.draw_text(
        state.status_line(),
        rect.x1 - 140.0,
        rect.y0 + 22.0,
        if state.is_dirty() {
            theme.primary
        } else {
            theme.secondary
        },
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
}

/// Returns the message to display in the notification label, or None if the
/// label should be hidden (license is active).
///
/// Behavior matches the spec:
/// - license unauthenticated + no update available: fixed "$1/month" message
/// - license unauthenticated + update available: 2-message cycle, 5s each
/// - license authenticated: hidden
pub(super) fn notification_label_message(state: &SlidesState) -> Option<&'static str> {
    if state.license_active {
        return None;
    }
    if state.update_available {
        let cycle = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            / 5)
            % 2;
        if cycle == 0 {
            Some("신규 업데이트 있음")
        } else {
            Some("월 $1 로 라이선스 활성화 가능")
        }
    } else {
        Some("월 $1 로 라이선스 활성화 가능")
    }
}

/// Geometry of the notification label inside the toolbar rect. Used by both
/// the painter and the click handler so they stay in sync.
pub(super) fn notification_label_rect(toolbar_rect: Rect) -> Rect {
    // Park the label just to the left of the slide-count info text.
    // paint_toolbar draws slide count at rect.x1 - 420.
    let label_w = 220.0;
    let label_x = toolbar_rect.x1 - 660.0;
    Rect::new(label_x, toolbar_rect.y0, label_x + label_w, toolbar_rect.y1)
}
