//! Screenshot/bitmap comparison utilities for visual regression testing.
//!
//! Provides tools to:
//! - Render a widget tree to an in-memory bitmap.
//! - Compare rendered output against a baseline image.
//! - Compute pixel-level diffs with configurable tolerance.
//! - Generate diff images highlighting mismatches.

use std::io::Cursor;
use std::sync::mpsc;

use kurbo::Size;
use tench_ui::vello::wgpu;
use tench_ui::vello::{AaSupport, RenderParams, Renderer, RendererOptions, Scene};
use tench_ui_automation_core::UiAutomationError;

/// Result of comparing two images.
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Total number of pixels compared.
    pub total_pixels: u64,
    /// Number of pixels that differ beyond tolerance.
    pub different_pixels: u64,
    /// Maximum per-channel difference across all pixels.
    pub max_channel_diff: u8,
    /// Mean per-pixel difference (averaged across all channels).
    pub mean_diff: f64,
}

impl DiffResult {
    /// Returns the fraction of pixels that differ (0.0 to 1.0).
    pub fn diff_ratio(&self) -> f64 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        self.different_pixels as f64 / self.total_pixels as f64
    }

    /// Returns whether the images match within the given tolerance.
    ///
    /// `max_diff_pixels` is the absolute number of pixels allowed to differ.
    /// `max_channel_diff` is the maximum allowed per-channel difference.
    pub fn matches(&self, max_diff_pixels: u64, max_channel_diff: u8) -> bool {
        self.different_pixels <= max_diff_pixels && self.max_channel_diff <= max_channel_diff
    }
}

/// Configuration for snapshot comparison.
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Maximum number of pixels allowed to differ (default: 0).
    pub max_diff_pixels: u64,
    /// Maximum per-channel color difference allowed (0-255, default: 0).
    pub max_channel_diff: u8,
    /// Whether to generate a diff image on mismatch (default: true).
    pub generate_diff_image: bool,
    /// Width for rendering (default: 800).
    pub width: u32,
    /// Height for rendering (default: 600).
    pub height: u32,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            max_diff_pixels: 0,
            max_channel_diff: 0,
            generate_diff_image: true,
            width: 800,
            height: 600,
        }
    }
}

impl SnapshotConfig {
    /// Creates a config with a specific viewport size.
    pub fn with_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Creates a mobile-sized config.
    pub fn mobile() -> Self {
        Self::with_size(390, 844)
    }

    /// Sets the maximum allowed differing pixels.
    pub fn with_tolerance(mut self, max_diff_pixels: u64, max_channel_diff: u8) -> Self {
        self.max_diff_pixels = max_diff_pixels;
        self.max_channel_diff = max_channel_diff;
        self
    }

    /// Returns the viewport as a kurbo Size.
    pub fn viewport(&self) -> Size {
        Size::new(self.width as f64, self.height as f64)
    }
}

/// Comparator for screenshot/bitmap snapshot testing.
pub struct SnapshotComparator {
    config: SnapshotConfig,
}

impl SnapshotComparator {
    /// Creates a new comparator with default config.
    pub fn new() -> Self {
        Self {
            config: SnapshotConfig::default(),
        }
    }

    /// Creates a new comparator with custom config.
    pub fn with_config(config: SnapshotConfig) -> Self {
        Self { config }
    }

    /// Returns the comparator's config.
    pub fn config(&self) -> &SnapshotConfig {
        &self.config
    }

    /// Compares two RGBA buffers pixel-by-pixel.
    ///
    /// Both buffers must have the same dimensions (`width * height * 4` bytes).
    /// Each pixel is 4 bytes: R, G, B, A.
    pub fn compare_rgba(
        &self,
        actual: &[u8],
        expected: &[u8],
        width: u32,
        height: u32,
    ) -> DiffResult {
        let total_pixels = width as u64 * height as u64;
        let expected_len = total_pixels as usize * 4;

        if actual.len() < expected_len || expected.len() < expected_len {
            return DiffResult {
                total_pixels,
                different_pixels: total_pixels,
                max_channel_diff: 255,
                mean_diff: 255.0,
            };
        }

        let mut different_pixels = 0u64;
        let mut max_channel_diff = 0u8;
        let mut total_diff = 0.0f64;

        for pixel_idx in 0..total_pixels as usize {
            let base = pixel_idx * 4;
            let mut pixel_differs = false;

            for ch in 0..4 {
                let a = actual[base + ch];
                let e = expected[base + ch];
                let diff = a.abs_diff(e);
                if diff > self.config.max_channel_diff {
                    pixel_differs = true;
                }
                if diff > max_channel_diff {
                    max_channel_diff = diff;
                }
                total_diff += diff as f64;
            }

            if pixel_differs {
                different_pixels += 1;
            }
        }

        let total_channels = total_pixels as f64 * 4.0;
        let mean_diff = if total_channels > 0.0 {
            total_diff / total_channels
        } else {
            0.0
        };

        DiffResult {
            total_pixels,
            different_pixels,
            max_channel_diff,
            mean_diff,
        }
    }

    /// Compares two `image::RgbaImage` buffers.
    pub fn compare_images(
        &self,
        actual: &image::RgbaImage,
        expected: &image::RgbaImage,
    ) -> DiffResult {
        let (w1, h1) = actual.dimensions();
        let (w2, h2) = expected.dimensions();

        if w1 != w2 || h1 != h2 {
            let total_pixels = std::cmp::max(w1 * h1, w2 * h2) as u64;
            return DiffResult {
                total_pixels,
                different_pixels: total_pixels,
                max_channel_diff: 255,
                mean_diff: 255.0,
            };
        }

        let width = w1;
        let height = h1;
        let total_pixels = width as u64 * height as u64;
        let mut different_pixels = 0u64;
        let mut max_channel_diff = 0u8;
        let mut total_diff = 0.0f64;

        for (actual_px, expected_px) in actual.pixels().zip(expected.pixels()) {
            let a = actual_px.0;
            let e = expected_px.0;
            let mut pixel_differs = false;

            for ch in 0..4 {
                let diff = a[ch].abs_diff(e[ch]);
                if diff > self.config.max_channel_diff {
                    pixel_differs = true;
                }
                if diff > max_channel_diff {
                    max_channel_diff = diff;
                }
                total_diff += diff as f64;
            }

            if pixel_differs {
                different_pixels += 1;
            }
        }

        let total_channels = total_pixels as f64 * 4.0;
        let mean_diff = if total_channels > 0.0 {
            total_diff / total_channels
        } else {
            0.0
        };

        DiffResult {
            total_pixels,
            different_pixels,
            max_channel_diff,
            mean_diff,
        }
    }

    /// Generates a diff image highlighting pixels that differ.
    ///
    /// Matching pixels are shown as-is; differing pixels are highlighted in red.
    pub fn generate_diff_image(
        &self,
        actual: &image::RgbaImage,
        expected: &image::RgbaImage,
    ) -> Option<image::RgbaImage> {
        let (w1, h1) = actual.dimensions();
        let (w2, h2) = expected.dimensions();
        if w1 != w2 || h1 != h2 {
            return None;
        }

        let mut diff = image::RgbaImage::new(w1, h1);
        for (x, y, actual_px) in actual.enumerate_pixels() {
            let expected_px = expected.get_pixel(x, y);
            let a = actual_px.0;
            let e = expected_px.0;

            let differs = (0..4).any(|ch| {
                let diff = a[ch].abs_diff(e[ch]);
                diff > self.config.max_channel_diff
            });

            if differs {
                // Highlight in red
                diff.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            } else {
                diff.put_pixel(x, y, *actual_px);
            }
        }

        Some(diff)
    }

    /// Asserts that two images match within tolerance, panicking with a detailed message otherwise.
    pub fn assert_matches(&self, actual: &image::RgbaImage, expected: &image::RgbaImage) {
        let result = self.compare_images(actual, expected);
        if !result.matches(self.config.max_diff_pixels, self.config.max_channel_diff) {
            panic!(
                "Snapshot mismatch: {}/{} pixels differ (max allowed: {}), max channel diff: {} (max allowed: {}), mean diff: {:.2}",
                result.different_pixels,
                result.total_pixels,
                self.config.max_diff_pixels,
                result.max_channel_diff,
                self.config.max_channel_diff,
                result.mean_diff,
            );
        }
    }
}

impl Default for SnapshotComparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders a Vello scene to an in-memory RGBA image without a platform window.
///
/// This uses a headless wgpu device and reads the offscreen render target back
/// into CPU memory, so it works in CI environments that do not expose a display
/// server. It still requires a usable wgpu adapter, such as a hardware adapter
/// or a software Vulkan/OpenGL implementation.
pub fn render_scene_to_image(
    scene: &Scene,
    width: u32,
    height: u32,
) -> Result<image::RgbaImage, UiAutomationError> {
    let rgba = render_scene_to_rgba(scene, width, height)?;
    image::RgbaImage::from_raw(width, height, rgba).ok_or(UiAutomationError::CaptureUnavailable)
}

/// Renders a Vello scene to PNG bytes without a platform window.
pub fn render_scene_to_png(
    scene: &Scene,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, UiAutomationError> {
    let image = render_scene_to_image(scene, width, height)?;
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|error| UiAutomationError::Internal(error.to_string()))?;
    Ok(bytes.into_inner())
}

fn render_scene_to_rgba(
    scene: &Scene,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, UiAutomationError> {
    if width == 0 || height == 0 {
        return Err(UiAutomationError::CaptureUnavailable);
    }

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .map_err(|error| UiAutomationError::Internal(error.to_string()))?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("tench-ui-test headless device"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .map_err(|error| UiAutomationError::Internal(error.to_string()))?;

    let mut renderer = Renderer::new(
        &device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    )
    .map_err(|error| UiAutomationError::Internal(error.to_string()))?;

    let texture_desc = wgpu::TextureDescriptor {
        label: Some("tench-ui-test headless capture target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    };
    let texture = device.create_texture(&texture_desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let render_params = RenderParams {
        base_color: tench_ui::core::types::Color::BLACK.into(),
        width,
        height,
        antialiasing_method: tench_ui::vello::AaConfig::Area,
    };
    renderer
        .render_to_texture(&device, &queue, scene, &view, &render_params)
        .map_err(|error| UiAutomationError::Internal(error.to_string()))?;

    read_texture_rgba(&device, &queue, &texture, width, height)
}

fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, UiAutomationError> {
    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
    let buffer_size = padded_bytes_per_row as u64 * height as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("tench-ui-test headless capture readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("tench-ui-test headless capture copy"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| UiAutomationError::Internal(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| UiAutomationError::Internal(error.to_string()))?
        .map_err(|error| UiAutomationError::Internal(error.to_string()))?;

    let mapped = buffer_slice.get_mapped_range();
    let mut rgba = vec![0; (width * height * bytes_per_pixel) as usize];
    for row in 0..height as usize {
        let src_start = row * padded_bytes_per_row as usize;
        let dst_start = row * unpadded_bytes_per_row as usize;
        let src_end = src_start + unpadded_bytes_per_row as usize;
        let dst_end = dst_start + unpadded_bytes_per_row as usize;
        rgba[dst_start..dst_end].copy_from_slice(&mapped[src_start..src_end]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(rgba)
}

/// Checks whether a rendered image is non-blank (not entirely one color).
///
/// Returns `true` if at least `min_unique_pixels` pixels differ from the
/// first pixel in the image.
pub fn is_nonblank(image: &image::RgbaImage, min_unique_pixels: u64) -> bool {
    if image.width() == 0 || image.height() == 0 {
        return false;
    }

    let first = image.get_pixel(0, 0).0;
    let mut unique_count = 0u64;

    for pixel in image.pixels() {
        if pixel.0 != first {
            unique_count += 1;
            if unique_count >= min_unique_pixels {
                return true;
            }
        }
    }

    unique_count >= min_unique_pixels
}
