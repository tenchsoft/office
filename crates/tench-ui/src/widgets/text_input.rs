//! TextInput widget — single-line text input field.

use kurbo::{Axis, Rect, Size};
use vello::Scene;

use crate::core::events::{ImeEvent, LogicalKey, NamedKey, PointerEvent, TextEvent};
use crate::core::types::Color;
use crate::core::widget::{
    AccessRole, AccessibilityNode, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetState,
};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A single-line text input field.
pub struct TextInput {
    value: String,
    placeholder: String,
    focused: bool,
    cursor_pos: usize,
    // clippy: callback field types are idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str) + Send>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Box<dyn FnMut(&str) + Send>>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            focused: false,
            cursor_pos: 0,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn with_value(mut self, text: impl Into<String>) -> Self {
        self.value = text.into();
        self.cursor_pos = self.value.len();
        self
    }

    pub fn on_change(mut self, f: impl FnMut(&str) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_submit(mut self, f: impl FnMut(&str) + Send + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, text: String) {
        self.value = text;
        self.cursor_pos = self.value.len();
    }
}

impl Widget for TextInput {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        let theme = Theme::default();
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => theme.input_height,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);

        let bg_color = if self.focused {
            theme.surface
        } else {
            Color::lerp(theme.surface, theme.background, 0.5)
        };

        let border_color = if self.focused {
            theme.primary
        } else {
            theme.border
        };

        let rect = Rect::from_origin_size((0.0, 0.0), size);
        painter.fill_rounded_rect(rect, bg_color, theme.border_radius);
        painter.stroke_rounded_rect(rect, border_color, 1.5, theme.border_radius);

        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        let text_color = if self.value.is_empty() {
            theme.disabled
        } else {
            theme.on_surface
        };

        let text_y = size.height / 2.0;
        let text_x = 8.0;

        painter.draw_text(
            display_text,
            text_x,
            text_y,
            text_color,
            theme.font_size,
            parley::FontWeight::NORMAL,
            false,
        );

        if self.focused {
            let char_width = theme.font_size as f64 * 0.6;
            let cursor_x = text_x + self.cursor_pos as f64 * char_width;
            let cursor_top = (size.height - theme.font_size as f64) / 2.0;
            let cursor_bottom = cursor_top + theme.font_size as f64;
            painter.draw_line(
                kurbo::Point::new(cursor_x, cursor_top),
                kurbo::Point::new(cursor_x, cursor_bottom),
                theme.primary,
                1.5,
            );
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Down(btn_event) = event {
            self.focused = true;
            let char_width = Theme::default().font_size as f64 * 0.6;
            let click_col = ((btn_event.pos.x - 8.0) / char_width).round() as usize;
            self.cursor_pos = click_col.min(self.value.len());
        }
    }

    fn on_text_event(&mut self, _ctx: &mut EventCtx, event: &TextEvent) {
        match event {
            TextEvent::Keyboard(key_event) if key_event.is_pressed => {
                match &key_event.logical_key {
                    LogicalKey::Character(ch) if ch.len() == 1 => {
                        let c = ch.chars().next().unwrap();
                        if !c.is_control() {
                            self.value.insert(self.cursor_pos, c);
                            self.cursor_pos += 1;
                            if let Some(on_change) = &mut self.on_change {
                                on_change(&self.value);
                            }
                        }
                    }
                    LogicalKey::Named(NamedKey::Backspace) if self.cursor_pos > 0 => {
                        self.cursor_pos -= 1;
                        self.value.remove(self.cursor_pos);
                        if let Some(on_change) = &mut self.on_change {
                            on_change(&self.value);
                        }
                    }
                    LogicalKey::Named(NamedKey::Delete) if self.cursor_pos < self.value.len() => {
                        self.value.remove(self.cursor_pos);
                        if let Some(on_change) = &mut self.on_change {
                            on_change(&self.value);
                        }
                    }
                    LogicalKey::Named(NamedKey::Enter) => {
                        if let Some(on_submit) = &mut self.on_submit {
                            on_submit(&self.value);
                        }
                    }
                    LogicalKey::Named(NamedKey::ArrowLeft) if self.cursor_pos > 0 => {
                        self.cursor_pos -= 1;
                    }
                    LogicalKey::Named(NamedKey::ArrowRight)
                        if self.cursor_pos < self.value.len() =>
                    {
                        self.cursor_pos += 1;
                    }
                    LogicalKey::Named(NamedKey::Home) => {
                        self.cursor_pos = 0;
                    }
                    LogicalKey::Named(NamedKey::End) => {
                        self.cursor_pos = self.value.len();
                    }
                    _ => {}
                }
            }
            TextEvent::Ime(ImeEvent::Commit(text)) => {
                for c in text.chars() {
                    if !c.is_control() {
                        self.value.insert(self.cursor_pos, c);
                        self.cursor_pos += 1;
                    }
                }
                if let Some(on_change) = &mut self.on_change {
                    on_change(&self.value);
                }
            }
            _ => {}
        }
    }

    fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
        AccessibilityNode {
            role: AccessRole::TextInput,
            label: (!self.placeholder.is_empty()).then(|| self.placeholder.clone()),
            value: Some(self.value.clone()),
            focused: state.is_focused || self.focused,
            disabled: state.is_disabled,
            children: Vec::new(),
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        true
    }
}
