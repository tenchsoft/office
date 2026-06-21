//! Tabs widget — a tab bar with switchable content panels.

use kurbo::{Axis, Point, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A single tab entry.
struct TabEntry {
    label: String,
    panel: WidgetPod,
}

/// A tab bar with switchable content panels.
pub struct Tabs {
    tabs: Vec<TabEntry>,
    active_index: usize,
    tab_height: f64,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
            tab_height: 36.0,
        }
    }

    pub fn with_tab(mut self, label: impl Into<String>, panel: impl Widget + 'static) -> Self {
        self.tabs.push(TabEntry {
            label: label.into(),
            panel: WidgetPod::new(panel),
        });
        self
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn set_active_index(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }
}

impl Widget for Tabs {
    fn measure(&mut self, ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => {
                // Tab bar height + active panel height
                let panel_height = if let Some(tab) = self.tabs.get_mut(self.active_index) {
                    let mut child_ctx = MeasureCtx {
                        state: &mut tab.panel.state,
                        global: ctx.global,
                    };
                    tab.panel.widget.measure(
                        &mut child_ctx,
                        Axis::Vertical,
                        available - self.tab_height,
                    )
                } else {
                    0.0
                };
                self.tab_height + panel_height
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        // Layout active panel below the tab bar
        let panel_size = Size::new(size.width, (size.height - self.tab_height).max(0.0));
        if let Some(tab) = self.tabs.get_mut(self.active_index) {
            tab.panel.state.position = Point::new(0.0, self.tab_height);
            tab.panel.state.size = panel_size;
            let mut child_ctx = LayoutCtx {
                state: &mut tab.panel.state,
                global: ctx.global,
            };
            tab.panel.widget.layout(&mut child_ctx, panel_size);
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);

        // Draw tab bar background
        let bar_rect = Rect::new(0.0, 0.0, size.width, self.tab_height);
        painter.fill_rect(bar_rect, theme.surface);

        // Draw tabs
        let tab_width = if !self.tabs.is_empty() {
            (size.width / self.tabs.len() as f64).min(200.0)
        } else {
            0.0
        };

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = i as f64 * tab_width;
            let tab_rect = Rect::new(x, 0.0, x + tab_width, self.tab_height);

            if i == self.active_index {
                painter.fill_rect(tab_rect, theme.background);
                // Active indicator line at bottom
                painter.fill_rect(
                    Rect::new(x, self.tab_height - 2.0, x + tab_width, self.tab_height),
                    theme.primary,
                );
            }

            // Tab label
            let text_color = if i == self.active_index {
                theme.on_surface
            } else {
                theme.disabled
            };
            painter.draw_text(
                &tab.label,
                x + tab_width / 2.0,
                self.tab_height / 2.0,
                text_color,
                theme.font_size_small,
                FontWeight::NORMAL,
                true,
            );
        }

        // Separator line below tab bar
        painter.fill_rect(
            Rect::new(0.0, self.tab_height - 1.0, size.width, self.tab_height),
            theme.border,
        );

        // Paint active panel
        if let Some(tab) = self.tabs.get_mut(self.active_index) {
            let theme = ctx.theme().clone();
            let mut child_ctx = PaintCtx {
                state: &mut tab.panel.state,
                global: ctx.global,
                theme,
            };
            tab.panel.widget.paint(&mut child_ctx, scene);
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Up(btn) = event {
            if btn.pos.y < self.tab_height {
                let tab_width = if !self.tabs.is_empty() {
                    (_ctx.state.size.width / self.tabs.len() as f64).min(200.0)
                } else {
                    0.0
                };
                let clicked_index = (btn.pos.x / tab_width) as usize;
                if clicked_index < self.tabs.len() {
                    self.active_index = clicked_index;
                }
            }
        }
    }

    fn children(&self) -> Vec<crate::core::types::WidgetId> {
        self.tabs.iter().map(|t| t.panel.state.id).collect()
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        self.tabs
            .iter_mut()
            .find(|t| t.panel.state.id == id)
            .map(|t| &mut t.panel)
    }
}
