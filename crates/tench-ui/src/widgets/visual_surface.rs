//! Generic visual learning/analysis surface.
//!
//! This widget provides shared viewport math, hit testing, and lightweight
//! rendering primitives for product-owned visual draw plans. Study can map
//! learning visuals into this surface, while Research can use the same surface
//! for graph, timeline, and analysis-preview foundations.

use kurbo::{Axis, Point, Rect, Size, Vec2};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::{LogicalKey, NamedKey, PointerEvent, TextEvent};
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualSurfaceViewport {
    pub zoom: f64,
    pub pan: Vec2,
    pub timeline_position: f32,
    pub reduced_motion: bool,
}

impl Default for VisualSurfaceViewport {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            timeline_position: 0.0,
            reduced_motion: false,
        }
    }
}

impl VisualSurfaceViewport {
    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.2, 8.0);
    }

    pub fn set_timeline_position(&mut self, position: f32) {
        self.timeline_position = position.clamp(0.0, 1.0);
    }
}

pub fn visual_surface_keyboard_viewport(
    mut viewport: VisualSurfaceViewport,
    key: NamedKey,
    shift: bool,
) -> Option<VisualSurfaceViewport> {
    let fine_step = if shift { 0.01 } else { 0.05 };
    let pan_step = if shift { 8.0 } else { 24.0 };
    match key {
        NamedKey::ArrowLeft => {
            viewport.set_timeline_position(viewport.timeline_position - fine_step);
        }
        NamedKey::ArrowRight => {
            viewport.set_timeline_position(viewport.timeline_position + fine_step);
        }
        NamedKey::ArrowUp => {
            viewport.pan_by(Vec2::new(0.0, pan_step));
        }
        NamedKey::ArrowDown => {
            viewport.pan_by(Vec2::new(0.0, -pan_step));
        }
        NamedKey::Home => {
            viewport.set_timeline_position(0.0);
        }
        NamedKey::End => {
            viewport.set_timeline_position(1.0);
        }
        NamedKey::PageUp => {
            viewport.set_zoom(viewport.zoom + 0.2);
        }
        NamedKey::PageDown => {
            viewport.set_zoom(viewport.zoom - 0.2);
        }
        _ => return None,
    }
    Some(viewport)
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualSurfaceCommand {
    pub id: String,
    pub kind: VisualSurfaceCommandKind,
    pub label: Option<String>,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VisualSurfaceCommandKind {
    Axis2d {
        x_label: String,
        y_label: String,
    },
    Shape2d {
        unit_rect: Rect,
        progress: f32,
        selected: bool,
    },
    Shape3dProxy {
        unit_rect: Rect,
        rotation: f32,
        selected: bool,
    },
    TimelineCursor {
        position: f32,
    },
    TextLabel {
        unit_position: Point,
        text: String,
    },
    ParameterMarker {
        unit_track: Rect,
        value: f64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualSurfaceHit {
    pub command_id: String,
}

pub fn visual_surface_frame(bounds: Rect, viewport: VisualSurfaceViewport) -> Rect {
    let inset = 16.0;
    let mut frame = bounds.inset(-inset);
    let center = frame.center();
    let width = frame.width() * viewport.zoom;
    let height = frame.height() * viewport.zoom;
    frame = Rect::from_center_size(center, Size::new(width, height));
    frame + viewport.pan
}

pub fn visual_surface_unit_rect(
    bounds: Rect,
    viewport: VisualSurfaceViewport,
    unit_rect: Rect,
) -> Rect {
    let frame = visual_surface_frame(bounds, viewport);
    Rect::new(
        frame.x0 + frame.width() * unit_rect.x0,
        frame.y0 + frame.height() * unit_rect.y0,
        frame.x0 + frame.width() * unit_rect.x1,
        frame.y0 + frame.height() * unit_rect.y1,
    )
}

pub fn visual_surface_timeline_position(bounds: Rect, point: Point) -> Option<f32> {
    let track = timeline_track(bounds);
    if !track.inflate(0.0, 8.0).contains(point) {
        return None;
    }
    Some(((point.x - track.x0) / track.width().max(1.0)).clamp(0.0, 1.0) as f32)
}

pub fn visual_surface_hit_test(
    commands: &[VisualSurfaceCommand],
    bounds: Rect,
    viewport: VisualSurfaceViewport,
    point: Point,
) -> Option<VisualSurfaceHit> {
    commands.iter().rev().find_map(|command| {
        let rect = match command.kind {
            VisualSurfaceCommandKind::Shape2d { unit_rect, .. }
            | VisualSurfaceCommandKind::Shape3dProxy { unit_rect, .. } => {
                visual_surface_unit_rect(bounds, viewport, unit_rect)
            }
            VisualSurfaceCommandKind::ParameterMarker { unit_track, .. } => {
                visual_surface_unit_rect(bounds, viewport, unit_track).inflate(0.0, 10.0)
            }
            _ => return None,
        };
        rect.contains(point).then(|| VisualSurfaceHit {
            command_id: command.id.clone(),
        })
    })
}

pub struct VisualSurface {
    commands: Vec<VisualSurfaceCommand>,
    viewport: VisualSurfaceViewport,
    accessibility_summary: String,
    drag_origin: Option<Point>,
    size: Size,
}

impl VisualSurface {
    pub fn new(commands: Vec<VisualSurfaceCommand>) -> Self {
        Self {
            commands,
            viewport: VisualSurfaceViewport::default(),
            accessibility_summary: "visual surface".to_string(),
            drag_origin: None,
            size: Size::ZERO,
        }
    }

    pub fn with_viewport(mut self, viewport: VisualSurfaceViewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn with_accessibility_summary(mut self, summary: impl Into<String>) -> Self {
        self.accessibility_summary = summary.into();
        self
    }

    pub fn viewport(&self) -> VisualSurfaceViewport {
        self.viewport
    }

    pub fn hit_test(&self, bounds: Rect, point: Point) -> Option<VisualSurfaceHit> {
        visual_surface_hit_test(&self.commands, bounds, self.viewport, point)
    }

    pub fn paint_in_rect(&self, painter: &mut Painter<'_>, bounds: Rect, theme: &Theme) {
        painter.fill_rect(bounds, theme.background);
        painter.stroke_rounded_rect(bounds, theme.border, 1.0, 4.0);
        painter.push_clip(bounds);

        if self.commands.is_empty() {
            painter.draw_text(
                &self.accessibility_summary,
                bounds.x0 + 16.0,
                bounds.y0 + 28.0,
                theme.disabled,
                theme.font_size,
                FontWeight::NORMAL,
                false,
            );
            painter.pop_clip();
            return;
        }

        for command in &self.commands {
            paint_visual_command(painter, bounds, self.viewport, command, theme);
        }
        paint_timeline(painter, bounds, self.viewport, theme);
        painter.draw_text(
            &self.accessibility_summary,
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

impl Widget for VisualSurface {
    fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, size: Size) {
        self.size = size;
    }

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
                let bounds = Rect::from_origin_size(Point::ZERO, self.size);
                if let Some(position) = visual_surface_timeline_position(bounds, event.pos) {
                    self.viewport.set_timeline_position(position);
                    ctx.request_paint();
                }
            }
            PointerEvent::Move(event) if self.drag_origin.replace(event.pos).is_some() => {
                self.viewport.pan_by(event.delta);
                ctx.request_paint();
            }
            PointerEvent::Up(_) => {
                self.drag_origin = None;
            }
            _ => {}
        }
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx<'_>, event: &TextEvent) {
        let TextEvent::Keyboard(event) = event else {
            return;
        };
        if !event.is_pressed {
            return;
        }
        let LogicalKey::Named(key) = &event.logical_key else {
            return;
        };
        if let Some(viewport) =
            visual_surface_keyboard_viewport(self.viewport, *key, event.modifiers.shift)
        {
            self.viewport = viewport;
            ctx.request_paint();
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }
}

fn paint_visual_command(
    painter: &mut Painter<'_>,
    bounds: Rect,
    viewport: VisualSurfaceViewport,
    command: &VisualSurfaceCommand,
    theme: &Theme,
) {
    match &command.kind {
        VisualSurfaceCommandKind::Axis2d { x_label, y_label } => {
            let frame = visual_surface_frame(bounds, viewport).inset(24.0);
            painter.draw_line(
                Point::new(frame.x0, frame.y1),
                Point::new(frame.x1, frame.y1),
                theme.border,
                1.0,
            );
            painter.draw_line(
                Point::new(frame.x0, frame.y0),
                Point::new(frame.x0, frame.y1),
                theme.border,
                1.0,
            );
            painter.draw_text(
                x_label,
                frame.x1 - 12.0,
                frame.y1 + 14.0,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            painter.draw_text(
                y_label,
                frame.x0 - 10.0,
                frame.y0 + 12.0,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        VisualSurfaceCommandKind::Shape2d {
            unit_rect,
            progress,
            selected,
        } => {
            let rect = visual_surface_unit_rect(bounds, viewport, *unit_rect);
            painter.fill_rounded_rect(rect, command.color, 6.0);
            if *selected {
                painter.stroke_rounded_rect(rect.inflate(3.0, 3.0), theme.primary, 2.0, 8.0);
            }
            let progress_width = rect.width() * f64::from(progress.clamp(0.0, 1.0));
            painter.fill_rect(
                Rect::new(rect.x0, rect.y1 - 4.0, rect.x0 + progress_width, rect.y1),
                theme.primary,
            );
        }
        VisualSurfaceCommandKind::Shape3dProxy {
            unit_rect,
            rotation,
            selected,
        } => {
            let rect = visual_surface_unit_rect(bounds, viewport, *unit_rect);
            painter.stroke_rounded_rect(rect, command.color, 2.0, 8.0);
            let center = rect.center();
            let arm = rect.width().min(rect.height()) * 0.35;
            let angle = f64::from(rotation.clamp(0.0, 1.0)) * std::f64::consts::TAU;
            painter.draw_line(
                center,
                Point::new(center.x + angle.cos() * arm, center.y + angle.sin() * arm),
                command.color,
                2.0,
            );
            if *selected {
                painter.stroke_rounded_rect(rect.inflate(3.0, 3.0), theme.primary, 2.0, 8.0);
            }
        }
        VisualSurfaceCommandKind::TimelineCursor { position } => {
            let track = timeline_track(bounds);
            let x = track.x0 + track.width() * f64::from(position.clamp(0.0, 1.0));
            painter.draw_line(
                Point::new(x, track.y0 - 8.0),
                Point::new(x, track.y1 + 8.0),
                theme.primary,
                2.0,
            );
        }
        VisualSurfaceCommandKind::TextLabel {
            unit_position,
            text,
        } => {
            let frame = visual_surface_frame(bounds, viewport);
            painter.draw_text(
                text,
                frame.x0 + frame.width() * unit_position.x,
                frame.y0 + frame.height() * unit_position.y,
                command.color,
                theme.font_size,
                FontWeight::MEDIUM,
                false,
            );
        }
        VisualSurfaceCommandKind::ParameterMarker { unit_track, value } => {
            let track = visual_surface_unit_rect(bounds, viewport, *unit_track);
            painter.stroke_rounded_rect(track, command.color, 1.0, 999.0);
            let x = track.x0 + track.width() * value.clamp(0.0, 1.0);
            painter.fill_rounded_rect(
                Rect::new(x - 4.0, track.y0 - 4.0, x + 4.0, track.y1 + 4.0),
                command.color,
                999.0,
            );
        }
    }

    if let Some(label) = &command.label {
        let frame = visual_surface_frame(bounds, viewport);
        painter.draw_text(
            label,
            frame.x0 + 12.0,
            frame.y0 + 18.0,
            theme.on_background,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }
}

fn paint_timeline(
    painter: &mut Painter<'_>,
    bounds: Rect,
    viewport: VisualSurfaceViewport,
    theme: &Theme,
) {
    let track = timeline_track(bounds);
    let position = if viewport.reduced_motion {
        1.0
    } else {
        viewport.timeline_position
    };
    let x = track.x0 + track.width() * f64::from(position.clamp(0.0, 1.0));
    painter.fill_rounded_rect(track, theme.border, 999.0);
    painter.fill_rounded_rect(
        Rect::new(track.x0, track.y0, x, track.y1),
        theme.primary,
        999.0,
    );
    painter.fill_rounded_rect(
        Rect::new(x - 5.0, track.y0 - 5.0, x + 5.0, track.y1 + 5.0),
        theme.primary,
        999.0,
    );
}

fn timeline_track(bounds: Rect) -> Rect {
    Rect::new(
        bounds.x0 + 18.0,
        bounds.y1 - 34.0,
        bounds.x1 - 18.0,
        bounds.y1 - 30.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_position_clamps_to_track() {
        let bounds = Rect::new(0.0, 0.0, 220.0, 140.0);
        let track = timeline_track(bounds);

        assert_eq!(
            visual_surface_timeline_position(bounds, Point::new(track.x0, track.y0)),
            Some(0.0)
        );
        assert!(
            visual_surface_timeline_position(bounds, Point::new(track.x1 - 0.1, track.y0))
                .expect("timeline position")
                > 0.99
        );
    }

    #[test]
    fn unit_rect_uses_viewport_zoom_and_pan() {
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let viewport = VisualSurfaceViewport {
            zoom: 2.0,
            pan: Vec2::new(10.0, -10.0),
            ..VisualSurfaceViewport::default()
        };

        let unit_rect = Rect::new(0.25, 0.25, 0.75, 0.75);
        let without_pan = visual_surface_unit_rect(
            bounds,
            VisualSurfaceViewport {
                zoom: 2.0,
                ..VisualSurfaceViewport::default()
            },
            unit_rect,
        );
        let rect = visual_surface_unit_rect(bounds, viewport, unit_rect);

        assert!(rect.width() > 60.0);
        assert!(rect.x0 > without_pan.x0);
    }

    #[test]
    fn hit_test_returns_topmost_shape() {
        let commands = vec![
            VisualSurfaceCommand {
                id: "back".to_string(),
                kind: VisualSurfaceCommandKind::Shape2d {
                    unit_rect: Rect::new(0.1, 0.1, 0.9, 0.9),
                    progress: 0.0,
                    selected: false,
                },
                label: None,
                color: Color::rgb8(1, 1, 1),
            },
            VisualSurfaceCommand {
                id: "front".to_string(),
                kind: VisualSurfaceCommandKind::Shape2d {
                    unit_rect: Rect::new(0.2, 0.2, 0.8, 0.8),
                    progress: 1.0,
                    selected: true,
                },
                label: None,
                color: Color::rgb8(2, 2, 2),
            },
        ];

        let hit = visual_surface_hit_test(
            &commands,
            Rect::new(0.0, 0.0, 200.0, 200.0),
            VisualSurfaceViewport::default(),
            Point::new(100.0, 100.0),
        )
        .expect("hit");

        assert_eq!(hit.command_id, "front");
    }

    #[test]
    fn keyboard_viewport_supports_timeline_pan_and_zoom() {
        let viewport = VisualSurfaceViewport {
            timeline_position: 0.5,
            ..VisualSurfaceViewport::default()
        };

        let viewport =
            visual_surface_keyboard_viewport(viewport, NamedKey::ArrowRight, false).unwrap();
        assert!(viewport.timeline_position > 0.54);

        let viewport = visual_surface_keyboard_viewport(viewport, NamedKey::ArrowUp, true).unwrap();
        assert!(viewport.pan.y > 0.0);

        let viewport = visual_surface_keyboard_viewport(viewport, NamedKey::PageUp, false).unwrap();
        assert!(viewport.zoom > 1.0);

        let viewport = visual_surface_keyboard_viewport(viewport, NamedKey::End, false).unwrap();
        assert_eq!(viewport.timeline_position, 1.0);
    }
}
