//! TextArea widget — multi-line text editing.

use kurbo::{Axis, Point, Rect, Size};
use vello::Scene;

use crate::core::events::{ImeEvent, LogicalKey, NamedKey, PointerEvent, TextEvent};
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::{Painter, TextCache};
use crate::theme::Theme;

/// A multi-line text area.
pub struct TextArea {
    value: String,
    placeholder: String,
    focused: bool,
    cursor_pos: usize,
    scroll_y: f64,
    line_height: f64,
    text_cache: TextCache,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str) + Send>>,
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            focused: false,
            cursor_pos: 0,
            scroll_y: 0.0,
            line_height: 20.0,
            text_cache: TextCache::new(),
            on_change: None,
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

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, text: String) {
        self.value = text;
        self.cursor_pos = self.value.len();
    }

    fn line_count(&self) -> usize {
        self.value.lines().count().max(1)
    }

    fn content_height(&self) -> f64 {
        self.line_count() as f64 * self.line_height
    }
}

impl Widget for TextArea {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => self.content_height().max(100.0),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);

        let rect = Rect::from_origin_size((0.0, 0.0), size);
        painter.fill_rounded_rect(rect, theme.surface, theme.border_radius);
        painter.stroke_rounded_rect(rect, theme.border, 1.0, theme.border_radius);

        painter.push_clip(rect);

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

        let mut y = 4.0 - self.scroll_y;
        for line in display_text.lines() {
            if y + self.line_height > 0.0 && y < size.height {
                painter.draw_text_cached(
                    &mut self.text_cache,
                    line,
                    8.0,
                    y + self.line_height / 2.0,
                    text_color,
                    theme.font_size,
                    parley::FontWeight::NORMAL,
                    false,
                    false,
                );
            }
            y += self.line_height;
        }

        if self.focused {
            let text_before_cursor: String = self.value[..self.cursor_pos]
                .lines()
                .last()
                .unwrap_or("")
                .to_string();
            let line_index = self.value[..self.cursor_pos].matches('\n').count();
            let cursor_x = 8.0
                + self.text_cache.measure_text_width(
                    &text_before_cursor,
                    theme.font_size,
                    parley::FontWeight::NORMAL,
                );
            let cursor_y = 4.0 + line_index as f64 * self.line_height - self.scroll_y;
            let cursor_bottom = cursor_y + self.line_height;

            if cursor_y >= 0.0 && cursor_bottom <= size.height {
                painter.draw_line(
                    Point::new(cursor_x, cursor_y),
                    Point::new(cursor_x, cursor_bottom),
                    theme.primary,
                    1.5,
                );
            }
        }

        painter.pop_clip();
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Down(btn_event) => {
                self.focused = true;
                let line_index =
                    ((btn_event.pos.y + self.scroll_y - 4.0) / self.line_height) as usize;
                let click_x = btn_event.pos.x - 8.0;
                let font_size = Theme::default().font_size;

                let mut current_pos = 0;
                for (i, line) in self.value.lines().enumerate() {
                    if i == line_index {
                        // Find the column whose measured width is closest to click_x.
                        let mut best_col = 0;
                        let mut best_dist = click_x.abs();
                        for col in 1..=line.len() {
                            let w = self.text_cache.measure_text_width(
                                &line[..col],
                                font_size,
                                parley::FontWeight::NORMAL,
                            );
                            let dist = (click_x - w).abs();
                            if dist < best_dist {
                                best_dist = dist;
                                best_col = col;
                            }
                        }
                        current_pos += best_col;
                        break;
                    }
                    current_pos += line.len() + 1;
                }
                self.cursor_pos = current_pos.min(self.value.len());
            }
            PointerEvent::Scroll(scroll_event) => {
                self.scroll_y = (self.scroll_y + scroll_event.delta.y * 30.0).clamp(
                    0.0,
                    (self.content_height() - ctx.state.size.height).max(0.0),
                );
            }
            _ => {}
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
                    LogicalKey::Named(NamedKey::Enter) => {
                        self.value.insert(self.cursor_pos, '\n');
                        self.cursor_pos += 1;
                        if let Some(on_change) = &mut self.on_change {
                            on_change(&self.value);
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
                    LogicalKey::Named(NamedKey::ArrowLeft) if self.cursor_pos > 0 => {
                        self.cursor_pos -= 1;
                    }
                    LogicalKey::Named(NamedKey::ArrowRight)
                        if self.cursor_pos < self.value.len() =>
                    {
                        self.cursor_pos += 1;
                    }
                    LogicalKey::Named(NamedKey::ArrowUp) => {
                        let line_start = self.value[..self.cursor_pos]
                            .rfind('\n')
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        if line_start > 0 {
                            let prev_line_start = self.value[..line_start - 1]
                                .rfind('\n')
                                .map(|i| i + 1)
                                .unwrap_or(0);
                            let col = self.cursor_pos - line_start;
                            self.cursor_pos = (prev_line_start + col).min(line_start - 1);
                        }
                    }
                    LogicalKey::Named(NamedKey::ArrowDown) => {
                        if let Some(next_nl) = self.value[self.cursor_pos..].find('\n') {
                            let abs_next = self.cursor_pos + next_nl + 1;
                            let line_start = self.value[..self.cursor_pos]
                                .rfind('\n')
                                .map(|i| i + 1)
                                .unwrap_or(0);
                            let col = self.cursor_pos - line_start;
                            let next_line_end = self.value[abs_next..]
                                .find('\n')
                                .map(|i| abs_next + i)
                                .unwrap_or(self.value.len());
                            self.cursor_pos = (abs_next + col).min(next_line_end);
                        }
                    }
                    LogicalKey::Named(NamedKey::Home) => {
                        let line_start = self.value[..self.cursor_pos]
                            .rfind('\n')
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        self.cursor_pos = line_start;
                    }
                    LogicalKey::Named(NamedKey::End) => {
                        let line_end = self.value[self.cursor_pos..]
                            .find('\n')
                            .map(|i| self.cursor_pos + i)
                            .unwrap_or(self.value.len());
                        self.cursor_pos = line_end;
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
}
