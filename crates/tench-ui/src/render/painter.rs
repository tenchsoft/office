//! High-level drawing API wrapping Vello Scene.

use kurbo::{Affine, BezPath, Circle, Line, Point, Rect, RoundedRect, Size, Stroke};
use parley::layout::PositionedLayoutItem;
use parley::FontWeight;
use peniko::{Brush, Color as PenikoColor, Fill, Gradient};
use vello::Glyph;
use vello::Scene;

use crate::core::types::Color;

/// Direction for a linear gradient.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientDirection {
    /// Gradient flows from left to right.
    Horizontal,
    /// Gradient flows from top to bottom.
    Vertical,
}

/// Cached text resources for reuse across draw_text calls.
///
/// Creating a `FontContext` and `LayoutContext` on every frame is expensive.
/// Store a `TextCache` and pass it to `Painter::draw_text_cached` instead.
///
/// The cache also pre-configures CJK font fallback chains so that Chinese,
/// Japanese, and Korean text render correctly even when the primary font
/// does not contain CJK glyphs.
pub struct TextCache {
    pub(crate) font_ctx: parley::FontContext,
    pub(crate) layout_ctx: parley::LayoutContext<()>,
}

impl TextCache {
    /// Creates a new text cache with default system font discovery and
    /// CJK fallback chains.
    pub fn new() -> Self {
        let mut font_ctx = parley::FontContext::new();
        configure_cjk_fallbacks(&mut font_ctx);
        Self {
            font_ctx,
            layout_ctx: parley::LayoutContext::new(),
        }
    }

    /// Measures the advance width of `text` at the given font size and weight.
    ///
    /// Uses Parley layout internally, so the returned width reflects actual
    /// glyph advances including kerning and ligatures.
    pub fn measure_text_width(&mut self, text: &str, font_size: f32, weight: FontWeight) -> f64 {
        let layout = build_text_layout(
            &mut self.font_ctx,
            &mut self.layout_ctx,
            text,
            font_size,
            weight,
            false,
        );
        f64::from(layout.width())
    }
}

thread_local! {
    static DEFAULT_TEXT_CACHE: std::cell::RefCell<TextCache> =
        std::cell::RefCell::new(TextCache::new());
}

impl Default for TextCache {
    fn default() -> Self {
        Self::new()
    }
}

fn build_text_layout(
    font_ctx: &mut parley::FontContext,
    layout_ctx: &mut parley::LayoutContext<()>,
    text: &str,
    font_size: f32,
    weight: FontWeight,
    italic: bool,
) -> parley::Layout<()> {
    let mut builder = layout_ctx.ranged_builder(font_ctx, text, 1.0, false);

    builder.push_default(parley::style::StyleProperty::FontSize(font_size));
    builder.push_default(parley::style::StyleProperty::FontWeight(
        parley::style::FontWeight::new(weight.value()),
    ));
    if italic {
        builder.push_default(parley::style::StyleProperty::FontStyle(
            parley::style::FontStyle::Italic,
        ));
    }

    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    layout
}

fn text_layout_origin(layout: &parley::Layout<()>, x: f64, y: f64, center: bool) -> (f64, f64) {
    if center {
        return (
            x - f64::from(layout.width()) / 2.0,
            y - f64::from(layout.height()) / 2.0,
        );
    }

    let first_baseline = layout
        .lines()
        .next()
        .map(|line| f64::from(line.metrics().baseline))
        .unwrap_or(0.0);

    (x, y - first_baseline)
}

/// A drawing wrapper around a Vello Scene reference.
pub struct Painter<'a> {
    scene: &'a mut Scene,
    transform: Affine,
}

impl<'a> Painter<'a> {
    /// Creates a new painter wrapping a Vello Scene.
    pub fn new(scene: &'a mut Scene) -> Self {
        Self {
            scene,
            transform: Affine::IDENTITY,
        }
    }

    /// Fills a rectangle with a color.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let brush: Brush = color.into();
        self.scene
            .fill(Fill::NonZero, self.transform, &brush, None, &rect);
    }

    /// Fills a rectangle with a linear gradient.
    ///
    /// `start_color` and `end_color` are interpolated along the `direction`.
    pub fn fill_rect_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: Color,
        end_color: Color,
        direction: GradientDirection,
    ) {
        let start: PenikoColor = start_color.into();
        let end: PenikoColor = end_color.into();

        let (start_point, end_point) = match direction {
            GradientDirection::Horizontal => {
                (Point::new(rect.x0, rect.y0), Point::new(rect.x1, rect.y0))
            }
            GradientDirection::Vertical => {
                (Point::new(rect.x0, rect.y0), Point::new(rect.x0, rect.y1))
            }
        };

        let gradient = Gradient::new_linear(start_point, end_point).with_stops([
            peniko::color::DynamicColor::from_alpha_color(start),
            peniko::color::DynamicColor::from_alpha_color(end),
        ]);

        self.scene
            .fill(Fill::NonZero, self.transform, &gradient, None, &rect);
    }

    /// Fills a rounded rectangle with a color.
    pub fn fill_rounded_rect(&mut self, rect: Rect, color: Color, radius: f64) {
        let brush: Brush = color.into();
        let rounded = RoundedRect::new(rect.x0, rect.y0, rect.x1, rect.y1, radius);
        self.scene
            .fill(Fill::NonZero, self.transform, &brush, None, &rounded);
    }

    /// Strokes a rounded rectangle outline.
    pub fn stroke_rounded_rect(&mut self, rect: Rect, color: Color, width: f64, radius: f64) {
        let brush: Brush = color.into();
        let rounded = RoundedRect::new(rect.x0, rect.y0, rect.x1, rect.y1, radius);
        self.scene
            .stroke(&Stroke::new(width), self.transform, &brush, None, &rounded);
    }

    /// Fills a circle with a color.
    pub fn fill_circle(&mut self, center: Point, radius: f64, color: Color) {
        let brush: Brush = color.into();
        let circle = Circle::new(center, radius);
        self.scene
            .fill(Fill::NonZero, self.transform, &brush, None, &circle);
    }

    /// Strokes a circle outline.
    pub fn stroke_circle(&mut self, center: Point, radius: f64, color: Color, width: f64) {
        let brush: Brush = color.into();
        let circle = Circle::new(center, radius);
        self.scene
            .stroke(&Stroke::new(width), self.transform, &brush, None, &circle);
    }

    /// Fills a triangle defined by three points.
    pub fn fill_triangle(&mut self, p0: Point, p1: Point, p2: Point, color: Color) {
        let brush: Brush = color.into();
        let mut path = BezPath::new();
        path.move_to(p0);
        path.line_to(p1);
        path.line_to(p2);
        path.close_path();
        self.scene
            .fill(Fill::NonZero, self.transform, &brush, None, &path);
    }

    /// Fills a rounded rectangle with a drop shadow behind it.
    ///
    /// The shadow is rendered as a blurred rounded rect using Vello's
    /// `draw_blurred_rounded_rect`, positioned at `(rect + shadow_offset)`.
    /// Then the main rect is drawn on top.
    pub fn fill_rounded_rect_with_shadow(
        &mut self,
        rect: Rect,
        radius: f64,
        color: Color,
        shadow_offset: (f64, f64),
        shadow_blur: f64,
        shadow_color: Color,
    ) {
        let shadow_rect = Rect::new(
            rect.x0 + shadow_offset.0,
            rect.y0 + shadow_offset.1,
            rect.x1 + shadow_offset.0,
            rect.y1 + shadow_offset.1,
        );

        // Draw the blurred shadow
        let shadow_peniko: PenikoColor = shadow_color.into();
        self.scene.draw_blurred_rounded_rect(
            self.transform,
            shadow_rect,
            shadow_peniko,
            radius,
            shadow_blur,
        );

        // Draw the main rounded rect on top
        self.fill_rounded_rect(rect, color, radius);
    }

    /// Draws an image at the given rect using a pre-loaded wgpu texture.
    ///
    /// The `texture` must be a wgpu texture with `Rgba8Unorm` format and
    /// `TEXTURE_BINDING` usage. The image is scaled to fill `rect`.
    pub fn draw_image(&mut self, image: &peniko::ImageData, rect: Rect) {
        let scale_x = rect.width() / image.width as f64;
        let scale_y = rect.height() / image.height as f64;
        let image_transform = self.transform
            * Affine::translate((rect.x0, rect.y0))
            * Affine::scale_non_uniform(scale_x, scale_y);

        let brush = peniko::ImageBrush::new(image.clone());
        self.scene.draw_image(&brush, image_transform);
    }

    /// Draws a line from `p0` to `p1`.
    pub fn draw_line(&mut self, p0: Point, p1: Point, color: Color, width: f64) {
        let brush: Brush = color.into();
        self.scene.stroke(
            &Stroke::new(width),
            self.transform,
            &brush,
            None,
            &Line::new(p0, p1),
        );
    }

    /// Fills the entire background with a color.
    pub fn fill_background(&mut self, size: Size, color: Color) {
        let rect = Rect::from_origin_size(Point::ZERO, size);
        self.fill_rect(rect, color);
    }

    /// Returns a mutable reference to the underlying Vello Scene.
    ///
    /// Crate-internal: used by `TextLayout::paint` to render glyphs directly.
    pub(crate) fn scene_mut(&mut self) -> &mut Scene {
        self.scene
    }

    /// Returns the current transform applied to all drawing operations.
    ///
    /// Crate-internal: used by `TextLayout::paint` to compose with the painter's transform.
    pub(crate) fn current_transform(&self) -> Affine {
        self.transform
    }

    /// Draws a pre-built `TextLayout` at the given position.
    pub fn draw_text_layout(
        &mut self,
        layout: &crate::text::TextLayout,
        x: f64,
        y: f64,
        color: Color,
    ) {
        layout.paint(kurbo::Point::new(x, y), self, color);
    }

    /// Pushes a clip rectangle.
    pub fn push_clip(&mut self, rect: Rect) {
        self.scene.push_layer(
            peniko::Fill::NonZero,
            peniko::BlendMode::default(),
            1.0,
            self.transform,
            &rect,
        );
    }

    /// Pops a clip layer.
    pub fn pop_clip(&mut self) {
        self.scene.pop_layer();
    }

    /// Draws text at the given position using a reusable `TextCache`.
    ///
    /// Prefer this over `draw_text` to avoid recreating font/layout contexts
    /// on every call.
    // clippy: splitting into a struct would over-engineer a draw call
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_cached(
        &mut self,
        cache: &mut TextCache,
        text: &str,
        x: f64,
        y: f64,
        color: Color,
        font_size: f32,
        weight: FontWeight,
        center: bool,
        italic: bool,
    ) {
        let layout = build_text_layout(
            &mut cache.font_ctx,
            &mut cache.layout_ctx,
            text,
            font_size,
            weight,
            italic,
        );
        self.draw_parley_layout(&layout, x, y, color, center);
    }

    /// Draws text at the given position using Parley for layout and Vello for rendering.
    ///
    /// This creates a new `FontContext` and `LayoutContext` on every call.
    /// For better performance, use `draw_text_cached` with a `TextCache`.
    // clippy: splitting into a struct would over-engineer a draw call
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        color: Color,
        font_size: f32,
        weight: FontWeight,
        center: bool,
    ) {
        DEFAULT_TEXT_CACHE.with(|cache| {
            self.draw_text_cached(
                &mut cache.borrow_mut(),
                text,
                x,
                y,
                color,
                font_size,
                weight,
                center,
                false,
            );
        });
    }

    fn draw_parley_layout(
        &mut self,
        layout: &parley::Layout<()>,
        x: f64,
        y: f64,
        color: Color,
        center: bool,
    ) {
        let origin = text_layout_origin(layout, x, y, center);
        let text_transform = self.transform * Affine::translate(origin);
        let brush: Brush = color.into();

        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let font = run.font().clone();
                let run_size = run.font_size();

                let glyphs: Vec<Glyph> = glyph_run
                    .positioned_glyphs()
                    .map(|g| Glyph {
                        id: g.id,
                        x: g.x,
                        y: g.y,
                    })
                    .collect();

                if glyphs.is_empty() {
                    continue;
                }

                self.scene
                    .draw_glyphs(&font)
                    .transform(text_transform)
                    .font_size(run_size)
                    .brush(&brush)
                    .draw(Fill::NonZero, glyphs.into_iter());
            }
        }
    }
}

/// Configures CJK font fallback chains on the given `FontContext`.
///
/// For each CJK script (Hanzi/Han, Hiragana/Katakana, Hangul) we register
/// well-known font families as fallbacks. The search priority is:
///
/// 1. User-specified font family (set by the application)
/// 2. System default for the script
/// 3. CJK fallback chain configured here
///
/// The function is idempotent — calling it multiple times appends
/// duplicates but does not break anything because fontique deduplicates
/// internally.
fn configure_cjk_fallbacks(font_ctx: &mut parley::FontContext) {
    use parley::fontique::Script;

    // Ensure system fonts are loaded so family lookups succeed.
    font_ctx.collection.load_system_fonts();

    // Collect family IDs for well-known CJK font families.
    // We try multiple names per script because availability varies by OS.
    let cjk_family_names: &[&[&str]] = &[
        // CJK Unified Ideographs / Simplified Chinese
        &[
            "SimSun",
            "Noto Sans CJK SC",
            "Source Han Sans SC",
            "WenQuanYi Micro Hei",
        ],
        // CJK Unified Ideographs / Traditional Chinese
        &[
            "MingLiU",
            "Noto Sans CJK TC",
            "Source Han Sans TC",
            "PMingLiU",
        ],
        // Japanese Hiragana/Katakana
        &[
            "MS Gothic",
            "Noto Sans CJK JP",
            "Source Han Sans JP",
            "IPAGothic",
        ],
        // Korean Hangul
        &[
            "Malgun Gothic",
            "Noto Sans CJK KR",
            "Source Han Sans KR",
            "NanumGothic",
        ],
    ];

    // Register fallback families for CJK scripts via fontique.
    // ISO 15924 "Hani" covers all CJK ideographs.
    let han_script = Script::from_bytes(*b"Hani");
    for names in cjk_family_names {
        let mut family_ids: Vec<parley::fontique::FamilyId> = Vec::new();
        for name in *names {
            if let Some(id) = font_ctx.collection.family_id(name) {
                family_ids.push(id);
            }
        }
        if !family_ids.is_empty() {
            let _ = font_ctx
                .collection
                .append_fallbacks(han_script, family_ids.into_iter());
        }
    }

    // Hangul-specific fallback (ISO 15924 "Hang").
    {
        let korean_names: &[&str] = &[
            "Malgun Gothic",
            "Noto Sans CJK KR",
            "Source Han Sans KR",
            "NanumGothic",
        ];
        let mut family_ids: Vec<parley::fontique::FamilyId> = Vec::new();
        for name in korean_names {
            if let Some(id) = font_ctx.collection.family_id(name) {
                family_ids.push(id);
            }
        }
        if !family_ids.is_empty() {
            let _ = font_ctx
                .collection
                .append_fallbacks(Script::from_bytes(*b"Hang"), family_ids.into_iter());
        }
    }

    // Hiragana (ISO 15924 "Hira") and Katakana (ISO 15924 "Kana") fallback.
    {
        let jp_names: &[&str] = &[
            "MS Gothic",
            "Noto Sans CJK JP",
            "Source Han Sans JP",
            "IPAGothic",
        ];
        let mut family_ids: Vec<parley::fontique::FamilyId> = Vec::new();
        for name in jp_names {
            if let Some(id) = font_ctx.collection.family_id(name) {
                family_ids.push(id);
            }
        }
        if !family_ids.is_empty() {
            let _ = font_ctx
                .collection
                .append_fallbacks(Script::from_bytes(*b"Hira"), family_ids.into_iter());
        }
    }

    // Katakana (ISO 15924 "Kana") fallback — reuse the same Japanese font list.
    {
        let jp_names: &[&str] = &[
            "MS Gothic",
            "Noto Sans CJK JP",
            "Source Han Sans JP",
            "IPAGothic",
        ];
        let mut family_ids: Vec<parley::fontique::FamilyId> = Vec::new();
        for name in jp_names {
            if let Some(id) = font_ctx.collection.family_id(name) {
                family_ids.push(id);
            }
        }
        if !family_ids.is_empty() {
            let _ = font_ctx
                .collection
                .append_fallbacks(Script::from_bytes(*b"Kana"), family_ids.into_iter());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Painter;
    use crate::core::types::Color;
    use parley::FontWeight;
    use vello::Scene;

    #[test]
    fn draw_text_emits_visible_glyph_resources() {
        let mut scene = Scene::new();
        let mut painter = Painter::new(&mut scene);

        painter.draw_text(
            "Tench",
            24.0,
            32.0,
            Color::WHITE,
            14.0,
            FontWeight::NORMAL,
            false,
        );

        let resources = &scene.encoding().resources;
        assert!(
            !resources.glyph_runs.is_empty(),
            "text drawing must encode at least one glyph run"
        );
        assert!(
            !resources.glyphs.is_empty(),
            "text drawing must encode positioned glyphs"
        );
    }

    #[test]
    fn draw_text_uses_positioned_glyph_advances() {
        let mut scene = Scene::new();
        let mut painter = Painter::new(&mut scene);

        painter.draw_text(
            "Tench",
            24.0,
            32.0,
            Color::WHITE,
            14.0,
            FontWeight::NORMAL,
            false,
        );

        let glyphs = &scene.encoding().resources.glyphs;
        let min_x = glyphs
            .iter()
            .map(|glyph| glyph.x)
            .fold(f32::INFINITY, f32::min);
        let max_x = glyphs
            .iter()
            .map(|glyph| glyph.x)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(
            max_x - min_x > 1.0,
            "glyphs must be advanced across the text run instead of overlapping"
        );
    }

    #[test]
    fn draw_text_uses_requested_font_size() {
        let mut scene = Scene::new();
        let mut painter = Painter::new(&mut scene);

        painter.draw_text(
            "Tench",
            24.0,
            32.0,
            Color::WHITE,
            14.0,
            FontWeight::NORMAL,
            false,
        );

        for run in &scene.encoding().resources.glyph_runs {
            assert_eq!(
                run.font_size, 14.0,
                "layout scale must not multiply the requested font size"
            );
        }
    }
}
