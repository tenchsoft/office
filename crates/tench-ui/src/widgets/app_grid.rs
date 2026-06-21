//! AppGrid — responsive grid of app cards for the launcher.

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Rect, Size};

/// An app entry for the grid.
#[derive(Clone, Debug)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: Color,
}

/// A responsive grid of app cards.
pub struct AppGrid {
    apps: Vec<AppEntry>,
    columns: usize,
    card_size: Size,
    gap: f64,
    hovered: Option<usize>,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_launch: Option<Box<dyn Fn(&str) + Send>>,
}

impl AppGrid {
    pub fn new(apps: Vec<AppEntry>) -> Self {
        Self {
            apps,
            columns: 4,
            card_size: Size::new(120.0, 100.0),
            gap: 16.0,
            hovered: None,
            on_launch: None,
        }
    }

    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }

    pub fn on_launch(mut self, f: impl Fn(&str) + Send + 'static) -> Self {
        self.on_launch = Some(Box::new(f));
        self
    }

    fn card_rect(&self, index: usize, available_width: f64) -> Rect {
        let col = index % self.columns;
        let row = index / self.columns;
        let total_gap = self.gap * (self.columns - 1) as f64;
        let card_w = (available_width - total_gap) / self.columns as f64;
        let x = col as f64 * (card_w + self.gap);
        let y = row as f64 * (self.card_size.height + self.gap);
        Rect::new(x, y, x + card_w, y + self.card_size.height)
    }
}

impl Widget for AppGrid {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => {
                let rows = self.apps.len().div_ceil(self.columns);
                rows as f64 * (self.card_size.height + self.gap) - self.gap
            }
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        for (i, app) in self.apps.iter().enumerate() {
            let rect = self.card_rect(i, size.width);

            // Card background
            let bg = if self.hovered == Some(i) {
                Color::rgb8(55, 55, 55)
            } else {
                Color::rgb8(42, 42, 42)
            };
            painter.fill_rounded_rect(rect, bg, 8.0);

            // App icon (colored circle)
            let icon_center = Point::new(rect.x0 + (rect.width() / 2.0), rect.y0 + 30.0);
            painter.fill_rounded_rect(
                Rect::new(
                    icon_center.x - 16.0,
                    icon_center.y - 16.0,
                    icon_center.x + 16.0,
                    icon_center.y + 16.0,
                ),
                app.color,
                8.0,
            );

            // App name
            painter.draw_text(
                &app.name,
                rect.x0 + 8.0,
                rect.y0 + 65.0,
                theme.on_background,
                12.0,
                parley::FontWeight::NORMAL,
                false,
            );

            // Description (truncated)
            if app.description.len() > 15 {
                painter.draw_text(
                    &format!("{}...", &app.description[..15]),
                    rect.x0 + 8.0,
                    rect.y0 + 80.0,
                    Color::rgb8(120, 120, 120),
                    10.0,
                    parley::FontWeight::NORMAL,
                    false,
                );
            }
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Move(e) => {
                self.hovered = None;
                for i in 0..self.apps.len() {
                    let rect = self.card_rect(i, 9999.0); // approximate
                    if rect.contains(e.pos) {
                        self.hovered = Some(i);
                        break;
                    }
                }
            }
            PointerEvent::Down(e) => {
                for i in 0..self.apps.len() {
                    let rect = self.card_rect(i, 9999.0);
                    if rect.contains(e.pos) {
                        if let Some(ref on_launch) = self.on_launch {
                            on_launch(&self.apps[i].id);
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
