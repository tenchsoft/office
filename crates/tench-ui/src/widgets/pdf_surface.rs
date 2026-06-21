//! PDF surface widget for page rendering, overlays, and interaction math.

use kurbo::{Axis, Point, Rect, Size, Vec2};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfSurfaceTheme {
    Paper,
    Dark,
    Sepia,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfSurfaceViewport {
    pub current_page: u32,
    pub zoom: f64,
    pub pan: Vec2,
    pub page_gap: f64,
    pub theme: PdfSurfaceTheme,
}

impl Default for PdfSurfaceViewport {
    fn default() -> Self {
        Self {
            current_page: 1,
            zoom: 1.0,
            pan: Vec2::ZERO,
            page_gap: 24.0,
            theme: PdfSurfaceTheme::Paper,
        }
    }
}

impl PdfSurfaceViewport {
    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.1, 8.0);
    }

    pub fn zoom_at(&mut self, anchor: Point, delta: f64) {
        let before = self.zoom;
        self.set_zoom(self.zoom + delta);
        let factor = self.zoom / before.max(0.1);
        self.pan = Vec2::new(
            anchor.x - (anchor.x - self.pan.x) * factor,
            anchor.y - (anchor.y - self.pan.y) * factor,
        );
    }
}

#[derive(Clone, Debug)]
pub struct PdfSurfacePage {
    pub page: u32,
    pub label: String,
    pub size: Size,
    pub image_data: Option<peniko::ImageData>,
}

impl PdfSurfacePage {
    pub fn new(page: u32, size: Size) -> Self {
        Self {
            page,
            label: page.to_string(),
            size,
            image_data: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PdfSurfaceOverlay {
    pub id: String,
    pub page: u32,
    pub rect: Rect,
    pub kind: PdfSurfaceOverlayKind,
    pub color: Color,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfSurfaceOverlayKind {
    Highlight,
    Underline,
    Strikeout,
    SearchResult { active: bool },
    TextSelection,
    StickyNote,
    Bookmark,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PdfSurfaceHit {
    pub overlay_id: String,
    pub page: u32,
    pub kind: PdfSurfaceOverlayKind,
}

pub fn pdf_cache_window(current_page: u32, page_count: u32, radius: u32) -> Vec<u32> {
    if page_count == 0 {
        return Vec::new();
    }
    let current_page = current_page.clamp(1, page_count);
    let start = current_page.saturating_sub(radius).max(1);
    let end = current_page.saturating_add(radius).min(page_count);
    (start..=end).collect()
}

pub struct PdfSurface {
    pages: Vec<PdfSurfacePage>,
    overlays: Vec<PdfSurfaceOverlay>,
    viewport: PdfSurfaceViewport,
    accessibility_summary: String,
    drag_origin: Option<Point>,
}

impl PdfSurface {
    pub fn new(pages: Vec<PdfSurfacePage>) -> Self {
        Self {
            pages,
            overlays: Vec::new(),
            viewport: PdfSurfaceViewport::default(),
            accessibility_summary: "PDF document surface".to_string(),
            drag_origin: None,
        }
    }

    pub fn with_overlays(mut self, overlays: Vec<PdfSurfaceOverlay>) -> Self {
        self.overlays = overlays;
        self
    }

    pub fn with_viewport(mut self, viewport: PdfSurfaceViewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn with_accessibility_summary(mut self, summary: impl Into<String>) -> Self {
        self.accessibility_summary = summary.into();
        self
    }

    pub fn viewport(&self) -> PdfSurfaceViewport {
        self.viewport
    }

    pub fn current_page(&self) -> Option<&PdfSurfacePage> {
        self.pages
            .iter()
            .find(|page| page.page == self.viewport.current_page)
            .or_else(|| self.pages.first())
    }

    pub fn page_frame(&self, bounds: Rect, page_size: Size) -> Rect {
        page_frame(bounds, page_size, self.viewport)
    }

    pub fn page_to_screen_rect(&self, bounds: Rect, page_size: Size, rect: Rect) -> Rect {
        page_to_screen_rect(bounds, page_size, self.viewport, rect)
    }

    pub fn screen_to_page_point(&self, bounds: Rect, page_size: Size, point: Point) -> Point {
        screen_to_page_point(bounds, page_size, self.viewport, point)
    }

    pub fn hit_test_overlay(&self, bounds: Rect, point: Point) -> Option<PdfSurfaceHit> {
        let page = self.current_page()?;
        self.overlays
            .iter()
            .rev()
            .filter(|overlay| overlay.page == page.page)
            .find(|overlay| {
                self.page_to_screen_rect(bounds, page.size, overlay.rect)
                    .inflate(3.0, 3.0)
                    .contains(point)
            })
            .map(|overlay| PdfSurfaceHit {
                overlay_id: overlay.id.clone(),
                page: overlay.page,
                kind: overlay.kind,
            })
    }

    pub fn paint_in_rect(&self, painter: &mut Painter<'_>, bounds: Rect, theme: &Theme) {
        painter.fill_rect(bounds, surface_background(self.viewport.theme, theme));
        let Some(page) = self.current_page() else {
            painter.draw_text(
                "No PDF page loaded",
                bounds.x0 + 16.0,
                bounds.y0 + 28.0,
                theme.disabled,
                theme.font_size,
                FontWeight::NORMAL,
                false,
            );
            return;
        };

        let page_frame = self.page_frame(bounds, page.size);
        painter.push_clip(bounds);
        painter.fill_rect(page_frame, page_background(self.viewport.theme));
        if let Some(image) = &page.image_data {
            painter.draw_image(image, page_frame);
        } else {
            paint_page_placeholder(painter, page_frame, theme, self.viewport.theme);
        }
        painter.stroke_rounded_rect(page_frame, theme.border, 1.0, 2.0);

        for overlay in self
            .overlays
            .iter()
            .filter(|overlay| overlay.page == page.page)
        {
            let rect = self.page_to_screen_rect(bounds, page.size, overlay.rect);
            paint_overlay(painter, rect, overlay, theme);
        }

        painter.draw_text(
            &format!(
                "Page {} | {:.0}% | {}",
                page.label,
                self.viewport.zoom * 100.0,
                self.accessibility_summary
            ),
            bounds.x0 + 12.0,
            bounds.y1 - 12.0,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        painter.pop_clip();
    }
}

impl Widget for PdfSurface {
    fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene) {
        let mut painter = Painter::new(scene);
        self.paint_in_rect(
            &mut painter,
            Rect::from_origin_size(Point::ZERO, ctx.size()),
            ctx.theme(),
        );
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx<'_>, event: &PointerEvent) {
        match event {
            PointerEvent::Down(event) => {
                self.drag_origin = Some(event.pos);
            }
            PointerEvent::Move(event) if self.drag_origin.is_some() => {
                self.viewport.pan_by(event.delta);
                ctx.request_paint();
            }
            PointerEvent::Up(_) => {
                self.drag_origin = None;
            }
            PointerEvent::Scroll(event) => {
                self.viewport.zoom_at(event.pos, -event.delta.y * 0.002);
                ctx.request_paint();
            }
            _ => {}
        }
    }
}

pub fn page_frame(bounds: Rect, page_size: Size, viewport: PdfSurfaceViewport) -> Rect {
    let width = page_size.width * viewport.zoom;
    let height = page_size.height * viewport.zoom;
    let x = bounds.x0 + (bounds.width() - width) / 2.0 + viewport.pan.x;
    let y = bounds.y0 + 16.0 + viewport.pan.y;
    Rect::new(x, y, x + width, y + height)
}

pub fn page_to_screen_rect(
    bounds: Rect,
    page_size: Size,
    viewport: PdfSurfaceViewport,
    rect: Rect,
) -> Rect {
    let frame = page_frame(bounds, page_size, viewport);
    Rect::new(
        frame.x0 + rect.x0 * viewport.zoom,
        frame.y0 + rect.y0 * viewport.zoom,
        frame.x0 + rect.x1 * viewport.zoom,
        frame.y0 + rect.y1 * viewport.zoom,
    )
}

pub fn screen_to_page_point(
    bounds: Rect,
    page_size: Size,
    viewport: PdfSurfaceViewport,
    point: Point,
) -> Point {
    let frame = page_frame(bounds, page_size, viewport);
    Point::new(
        (point.x - frame.x0) / viewport.zoom,
        (point.y - frame.y0) / viewport.zoom,
    )
}

fn paint_page_placeholder(
    painter: &mut Painter<'_>,
    rect: Rect,
    theme: &Theme,
    surface_theme: PdfSurfaceTheme,
) {
    let line_color = match surface_theme {
        PdfSurfaceTheme::Dark => Color::rgba8(210, 218, 232, 60),
        PdfSurfaceTheme::Paper | PdfSurfaceTheme::Sepia => Color::rgba8(48, 56, 72, 45),
    };
    let mut y = rect.y0 + 36.0;
    while y < rect.y1 - 32.0 {
        let width = if (((y - rect.y0) / 18.0) as u32).is_multiple_of(5) {
            rect.width() * 0.56
        } else {
            rect.width() * 0.78
        };
        painter.fill_rounded_rect(
            Rect::new(rect.x0 + 36.0, y, rect.x0 + 36.0 + width, y + 5.0),
            line_color,
            2.0,
        );
        y += 18.0;
    }
    painter.draw_text(
        "PDF",
        rect.x0 + 36.0,
        rect.y0 + 24.0,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

fn paint_overlay(
    painter: &mut Painter<'_>,
    rect: Rect,
    overlay: &PdfSurfaceOverlay,
    theme: &Theme,
) {
    match overlay.kind {
        PdfSurfaceOverlayKind::Highlight => {
            painter.fill_rounded_rect(rect, overlay.color, 2.0);
        }
        PdfSurfaceOverlayKind::Underline => {
            painter.draw_line(
                Point::new(rect.x0, rect.y1),
                Point::new(rect.x1, rect.y1),
                overlay.color,
                2.0,
            );
        }
        PdfSurfaceOverlayKind::Strikeout => {
            painter.draw_line(
                Point::new(rect.x0, rect.center().y),
                Point::new(rect.x1, rect.center().y),
                overlay.color,
                2.0,
            );
        }
        PdfSurfaceOverlayKind::SearchResult { active } => {
            painter.stroke_rounded_rect(
                rect.inflate(1.5, 1.5),
                if active { theme.primary } else { overlay.color },
                if active { 2.0 } else { 1.0 },
                2.0,
            );
        }
        PdfSurfaceOverlayKind::TextSelection => {
            painter.fill_rounded_rect(rect, Color::rgba8(76, 132, 255, 80), 2.0);
        }
        PdfSurfaceOverlayKind::StickyNote => {
            painter.fill_circle(rect.origin() + Vec2::new(5.0, 5.0), 5.0, overlay.color);
            if let Some(label) = &overlay.label {
                painter.draw_text(
                    label,
                    rect.x0 + 14.0,
                    rect.y0 + 12.0,
                    theme.on_background,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
            }
        }
        PdfSurfaceOverlayKind::Bookmark => {
            painter.fill_rounded_rect(
                Rect::new(rect.x0, rect.y0, rect.x0 + 8.0, rect.y1.max(rect.y0 + 24.0)),
                overlay.color,
                1.0,
            );
        }
    }
}

fn surface_background(surface_theme: PdfSurfaceTheme, theme: &Theme) -> Color {
    match surface_theme {
        PdfSurfaceTheme::Paper => theme.background,
        PdfSurfaceTheme::Dark => Color::rgb8(24, 28, 36),
        PdfSurfaceTheme::Sepia => Color::rgb8(236, 229, 211),
    }
}

fn page_background(surface_theme: PdfSurfaceTheme) -> Color {
    match surface_theme {
        PdfSurfaceTheme::Paper => Color::WHITE,
        PdfSurfaceTheme::Dark => Color::rgb8(34, 39, 50),
        PdfSurfaceTheme::Sepia => Color::rgb8(252, 246, 227),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_window_clamps_to_document_edges() {
        assert_eq!(pdf_cache_window(1, 5, 2), vec![1, 2, 3]);
        assert_eq!(pdf_cache_window(5, 5, 2), vec![3, 4, 5]);
        assert_eq!(pdf_cache_window(10, 5, 1), vec![4, 5]);
    }

    #[test]
    fn screen_and_page_transform_round_trip_with_pan_and_zoom() {
        let viewport = PdfSurfaceViewport {
            current_page: 1,
            zoom: 2.0,
            pan: Vec2::new(12.0, -8.0),
            page_gap: 24.0,
            theme: PdfSurfaceTheme::Paper,
        };
        let bounds = Rect::new(0.0, 0.0, 500.0, 500.0);
        let page = Size::new(100.0, 200.0);
        let point = Point::new(25.0, 40.0);
        let screen = page_to_screen_rect(
            bounds,
            page,
            viewport,
            Rect::new(point.x, point.y, point.x, point.y),
        )
        .origin();

        let round_trip = screen_to_page_point(bounds, page, viewport, screen);

        assert!((round_trip.x - point.x).abs() < 0.001);
        assert!((round_trip.y - point.y).abs() < 0.001);
    }

    #[test]
    fn hit_testing_uses_transformed_overlay_rects() {
        let viewport = PdfSurfaceViewport {
            current_page: 2,
            zoom: 1.5,
            pan: Vec2::new(10.0, 0.0),
            page_gap: 24.0,
            theme: PdfSurfaceTheme::Paper,
        };
        let surface = PdfSurface::new(vec![PdfSurfacePage::new(2, Size::new(200.0, 300.0))])
            .with_viewport(viewport)
            .with_overlays(vec![PdfSurfaceOverlay {
                id: "ann_1".to_string(),
                page: 2,
                rect: Rect::new(20.0, 30.0, 80.0, 45.0),
                kind: PdfSurfaceOverlayKind::Highlight,
                color: Color::rgba8(255, 220, 0, 90),
                label: None,
            }]);
        let bounds = Rect::new(0.0, 0.0, 500.0, 500.0);
        let screen_rect =
            surface.page_to_screen_rect(bounds, Size::new(200.0, 300.0), surface.overlays[0].rect);

        let hit = surface
            .hit_test_overlay(bounds, screen_rect.center())
            .expect("hit overlay");

        assert_eq!(hit.overlay_id, "ann_1");
        assert_eq!(hit.kind, PdfSurfaceOverlayKind::Highlight);
    }
}
