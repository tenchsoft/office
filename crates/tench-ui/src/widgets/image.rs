//! Image widget — displays a bitmap image.

use kurbo::{Axis, Rect, Size};
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// How the image should fit within its bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFit {
    /// Scale the image to fill the bounds, maintaining aspect ratio.
    Contain,
    /// Scale the image to cover the bounds, cropping if necessary.
    Cover,
    /// Stretch the image to exactly fill the bounds.
    Fill,
    /// Use the image's natural size.
    None,
}

/// A widget that displays an image.
///
/// When an `ImageData` is provided via [`Image::with_image_data`], the widget
/// renders the actual pixels. Otherwise a placeholder rectangle is drawn.
pub struct Image {
    /// Natural width of the image in pixels.
    natural_width: f64,
    /// Natural height of the image in pixels.
    natural_height: f64,
    /// How to fit the image within the widget bounds.
    fit: ImageFit,
    /// Placeholder background color when no texture is loaded.
    placeholder_color: Option<crate::core::types::Color>,
    /// Optional image data for actual rendering.
    image_data: Option<peniko::ImageData>,
}

impl Image {
    /// Creates a new image widget with the given natural dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            natural_width: width as f64,
            natural_height: height as f64,
            fit: ImageFit::Contain,
            placeholder_color: None,
            image_data: None,
        }
    }

    /// Sets the fit mode.
    pub fn with_fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Sets a placeholder color shown when no image data is loaded.
    pub fn placeholder_color(mut self, color: crate::core::types::Color) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    /// Sets the image data for actual rendering.
    ///
    /// The `ImageData` carries the pixel buffer, format, and dimensions.
    /// When set, the widget will render the actual image instead of a
    /// placeholder.
    pub fn with_image_data(mut self, data: peniko::ImageData) -> Self {
        self.natural_width = data.width as f64;
        self.natural_height = data.height as f64;
        self.image_data = Some(data);
        self
    }

    fn compute_draw_rect(&self, bounds: Size) -> Rect {
        let aspect = self.natural_width / self.natural_height;
        match self.fit {
            ImageFit::Contain => {
                let bounds_aspect = bounds.width / bounds.height;
                if aspect > bounds_aspect {
                    let w = bounds.width;
                    let h = w / aspect;
                    let y = (bounds.height - h) / 2.0;
                    Rect::new(0.0, y, w, y + h)
                } else {
                    let h = bounds.height;
                    let w = h * aspect;
                    let x = (bounds.width - w) / 2.0;
                    Rect::new(x, 0.0, x + w, h)
                }
            }
            ImageFit::Cover => {
                let bounds_aspect = bounds.width / bounds.height;
                if aspect > bounds_aspect {
                    let h = bounds.height;
                    let w = h * aspect;
                    let x = (bounds.width - w) / 2.0;
                    Rect::new(x, 0.0, x + w, h)
                } else {
                    let w = bounds.width;
                    let h = w / aspect;
                    let y = (bounds.height - h) / 2.0;
                    Rect::new(0.0, y, w, y + h)
                }
            }
            ImageFit::Fill => Rect::from_origin_size((0.0, 0.0), bounds),
            ImageFit::None => {
                let x = (bounds.width - self.natural_width) / 2.0;
                let y = (bounds.height - self.natural_height) / 2.0;
                Rect::new(x, y, x + self.natural_width, y + self.natural_height)
            }
        }
    }
}

impl Widget for Image {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => self.natural_width.min(available),
            Axis::Vertical => self.natural_height.min(available),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);
        let draw_rect = self.compute_draw_rect(size);

        if let Some(ref image_data) = self.image_data {
            // Render the actual image
            painter.draw_image(image_data, draw_rect);
        } else {
            // Draw placeholder rectangle
            let bg = self.placeholder_color.unwrap_or(theme.disabled);
            painter.fill_rounded_rect(draw_rect, bg, 2.0);
        }
    }
}
