//! Text layout helpers wrapping Parley.
//!
//! `TextLayout` provides a high-level API for text measurement, line breaking,
//! hit testing, and painting. It wraps a Parley `Layout<()>` and caches the
//! result so that widgets can reuse layout across frames without recomputation.

use kurbo::{Affine, Point};
use parley::layout::{Cluster, Layout, Line, PositionedLayoutItem};
use parley::{FontContext, FontWeight, LayoutContext};
use peniko::Brush;
use vello::Glyph;
use vello::Scene;

use crate::core::types::Color as TenchColor;
use crate::render::painter::TextCache;

// ---------------------------------------------------------------------------
// LayoutLine — lightweight snapshot of a single computed line
// ---------------------------------------------------------------------------

/// A snapshot of a single line within a `TextLayout`.
#[derive(Debug, Clone)]
pub struct LayoutLine {
    /// Byte range within the source text.
    pub text_range: std::ops::Range<usize>,
    /// Typographic ascent.
    pub ascent: f32,
    /// Typographic descent.
    pub descent: f32,
    /// Leading.
    pub leading: f32,
    /// Baseline offset from the top of the line.
    pub baseline: f32,
    /// Total line height.
    pub line_height: f32,
    /// Y offset of this line's top edge from the layout origin.
    pub y_offset: f64,
}

impl LayoutLine {
    fn from_parley(line: Line<'_, ()>, y_offset: f64) -> Self {
        let m = line.metrics();
        Self {
            text_range: line.text_range(),
            ascent: m.ascent,
            descent: m.descent,
            leading: m.leading,
            baseline: m.baseline,
            line_height: m.line_height,
            y_offset,
        }
    }
}

// ---------------------------------------------------------------------------
// TextPosition — result of a hit test
// ---------------------------------------------------------------------------

/// The position within a `TextLayout` returned by [`TextLayout::hit_test`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPosition {
    /// Byte offset into the source text.
    pub byte_index: usize,
    /// Index of the line containing the position.
    pub line: usize,
}

// ---------------------------------------------------------------------------
// TextLayout
// ---------------------------------------------------------------------------

/// A computed text layout backed by Parley.
///
/// Create via [`TextLayout::new`] or [`TextLayout::new_with_cache`] (to reuse
/// font/layout contexts across calls), then query metrics, iterate lines, or
/// hit-test.
pub struct TextLayout {
    text: String,
    font_size: f32,
    max_width: Option<f64>,
    layout: Layout<()>,
    /// Pre-computed line snapshots (built lazily).
    line_snapshots: Vec<LayoutLine>,
}

impl TextLayout {
    /// Creates a new `TextLayout` using fresh font/layout contexts.
    ///
    /// For repeated creation (e.g. every frame), prefer [`Self::new_with_cache`]
    /// to avoid the cost of re-discovering system fonts each time.
    pub fn new(text: &str, font_size: f64, max_width: Option<f64>) -> Self {
        let mut cache = TextCache::new();
        Self::new_with_cache(text, font_size, max_width, &mut cache)
    }

    /// Creates a new `TextLayout` reusing an existing `TextCache`.
    pub fn new_with_cache(
        text: &str,
        font_size: f64,
        max_width: Option<f64>,
        cache: &mut TextCache,
    ) -> Self {
        let layout = build_layout(
            &mut cache.font_ctx,
            &mut cache.layout_ctx,
            text,
            font_size as f32,
            max_width.map(|w| w as f32),
        );
        Self {
            text: text.to_string(),
            font_size: font_size as f32,
            max_width,
            layout,
            line_snapshots: Vec::new(),
        }
    }

    // -- Accessors ----------------------------------------------------------

    /// Returns the source text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the font size used for this layout.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the maximum width constraint, if any.
    pub fn max_width(&self) -> Option<f64> {
        self.max_width
    }

    /// Returns the total computed width of the layout.
    pub fn width(&self) -> f64 {
        f64::from(self.layout.width())
    }

    /// Returns the total computed height of the layout.
    pub fn height(&self) -> f64 {
        f64::from(self.layout.height())
    }

    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        self.layout.len()
    }

    // -- Line snapshots -----------------------------------------------------

    /// Returns pre-computed line snapshots.
    ///
    /// The first call materialises the snapshots; subsequent calls return the
    /// cached slice.
    pub fn lines(&mut self) -> &[LayoutLine] {
        if self.line_snapshots.is_empty() && !self.layout.is_empty() {
            self.rebuild_line_snapshots();
        }
        &self.line_snapshots
    }

    /// Rebuilds the layout (e.g. after a max-width change).
    pub fn rebuild(&mut self, cache: &mut TextCache) {
        self.layout = build_layout(
            &mut cache.font_ctx,
            &mut cache.layout_ctx,
            &self.text,
            self.font_size,
            self.max_width.map(|w| w as f32),
        );
        self.line_snapshots.clear();
    }

    /// Updates the max-width constraint and rebuilds the layout.
    pub fn set_max_width(&mut self, max_width: Option<f64>, cache: &mut TextCache) {
        self.max_width = max_width;
        self.rebuild(cache);
    }

    /// Updates the text and rebuilds the layout.
    pub fn set_text(&mut self, text: &str, cache: &mut TextCache) {
        self.text = text.to_string();
        self.rebuild(cache);
    }

    // -- Hit testing --------------------------------------------------------

    /// Performs a hit test at the given local coordinates.
    ///
    /// Returns `None` when the point falls outside the layout area.
    /// Coordinates are relative to the layout origin (top-left of the first
    /// line's baseline area).
    pub fn hit_test(&self, x: f64, y: f64) -> Option<TextPosition> {
        let (cluster, _side) = Cluster::from_point(&self.layout, x as f32, y as f32)?;
        let text_range = cluster.text_range();
        let byte_index = text_range.start;
        let line = cluster.path().line_index();
        Some(TextPosition { byte_index, line })
    }

    // -- Painting -----------------------------------------------------------

    /// Paints the layout at the given origin using the supplied painter and
    /// color.
    pub fn paint(&self, origin: Point, painter: &mut crate::render::Painter, color: TenchColor) {
        let transform = painter.current_transform() * Affine::translate((origin.x, origin.y));
        let brush: Brush = color.into();
        paint_parley_layout(&self.layout, transform, &brush, painter.scene_mut());
    }

    // -- Internal -----------------------------------------------------------

    fn rebuild_line_snapshots(&mut self) {
        let mut y_offset = 0.0;
        let mut snapshots = Vec::with_capacity(self.layout.len());
        for i in 0..self.layout.len() {
            if let Some(line) = self.layout.get(i) {
                snapshots.push(LayoutLine::from_parley(line, y_offset));
                y_offset += f64::from(line.metrics().line_height);
            }
        }
        self.line_snapshots = snapshots;
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_layout(
    font_ctx: &mut FontContext,
    layout_ctx: &mut LayoutContext<()>,
    text: &str,
    font_size: f32,
    max_width: Option<f32>,
) -> Layout<()> {
    let mut builder = layout_ctx.ranged_builder(font_ctx, text, 1.0, false);
    builder.push_default(parley::style::StyleProperty::FontSize(font_size));
    builder.push_default(parley::style::StyleProperty::FontWeight(
        parley::style::FontWeight::new(FontWeight::NORMAL.value()),
    ));
    let mut layout = builder.build(text);
    layout.break_all_lines(max_width);
    layout
}

/// Renders a Parley layout into a Vello Scene.
///
/// This is the shared rendering path used by both `Painter::draw_text_layout`
/// and `TextLayout::paint`.
pub(crate) fn paint_parley_layout(
    layout: &Layout<()>,
    transform: Affine,
    brush: &Brush,
    scene: &mut Scene,
) {
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

            scene
                .draw_glyphs(&font)
                .transform(transform)
                .font_size(run_size)
                .brush(brush)
                .draw(peniko::Fill::NonZero, glyphs.into_iter());
        }
    }
}
