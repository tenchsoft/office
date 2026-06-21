//! Core types for the Tench UI framework.

use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique identifier for a single widget in the widget tree.
///
/// IDs are generated automatically via an atomic counter.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct WidgetId(NonZeroU64);

impl WidgetId {
    /// Allocates a new, unique `WidgetId`.
    pub fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id.try_into().unwrap())
    }

    /// Returns the raw u64 value.
    pub fn to_raw(self) -> u64 {
        self.0.into()
    }

    /// Recreates a widget ID from a raw value emitted by automation/debug APIs.
    pub fn from_raw(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }
}

impl std::fmt::Display for WidgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl From<WidgetId> for u64 {
    fn from(id: WidgetId) -> Self {
        id.to_raw()
    }
}

impl From<WidgetId> for accesskit::NodeId {
    fn from(id: WidgetId) -> Self {
        Self(id.0.into())
    }
}

/// An RGBA color.
///
/// Thin wrapper around `peniko::Color` (which is `AlphaColor<Srgb>`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(peniko::Color);

impl Color {
    /// Creates a color from 8-bit RGB values (fully opaque).
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self(peniko::color::AlphaColor::new([
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            1.0,
        ]))
    }

    /// Creates a color from 8-bit RGBA values.
    pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(peniko::color::AlphaColor::new([
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ]))
    }

    /// Creates a color from linear RGBA floats.
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(peniko::color::AlphaColor::new([r, g, b, a]))
    }

    /// Black.
    pub const BLACK: Self = Self::rgb8(0, 0, 0);
    /// White.
    pub const WHITE: Self = Self::rgb8(255, 255, 255);
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self::rgba8(0, 0, 0, 0);

    /// Returns the color as a packed RGBA u32 (8 bits per channel).
    pub fn to_u32(self) -> u32 {
        let rgba = self.0.components;
        let r = (rgba[0] * 255.0) as u8;
        let g = (rgba[1] * 255.0) as u8;
        let b = (rgba[2] * 255.0) as u8;
        let a = (rgba[3] * 255.0) as u8;
        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
    }

    /// Returns the red channel as u8 (0-255).
    pub fn r(self) -> u8 {
        (self.0.components[0] * 255.0) as u8
    }

    /// Returns the green channel as u8 (0-255).
    pub fn g(self) -> u8 {
        (self.0.components[1] * 255.0) as u8
    }

    /// Returns the blue channel as u8 (0-255).
    pub fn b(self) -> u8 {
        (self.0.components[2] * 255.0) as u8
    }

    /// Returns the alpha channel as u8 (0-255).
    pub fn a(self) -> u8 {
        (self.0.components[3] * 255.0) as u8
    }

    /// Creates a color from a packed RGBA u32 (8 bits per channel).
    pub fn from_u32(packed: u32) -> Self {
        let r = ((packed >> 24) & 0xFF) as f32 / 255.0;
        let g = ((packed >> 16) & 0xFF) as f32 / 255.0;
        let b = ((packed >> 8) & 0xFF) as f32 / 255.0;
        let a = (packed & 0xFF) as f32 / 255.0;
        Self(peniko::color::AlphaColor::new([r, g, b, a]))
    }

    /// Linearly interpolates between two colors.
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self(a.0.lerp_rect(b.0, t))
    }
}

impl From<Color> for peniko::Color {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<Color> for peniko::Brush {
    fn from(c: Color) -> Self {
        peniko::Brush::Solid(c.0)
    }
}

/// A 2D axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    /// Returns the cross axis.
    pub fn cross(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

/// Cursor icon for pointer hover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorIcon {
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    NotAllowed,
    ResizeColumn,
    ResizeRow,
    Grab,
    Grabbing,
}

/// Whether an event was handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handled {
    Yes,
    No,
}

impl From<bool> for Handled {
    fn from(b: bool) -> Self {
        if b {
            Self::Yes
        } else {
            Self::No
        }
    }
}

impl Handled {
    /// Returns true if the event was handled.
    pub fn is_handled(self) -> bool {
        self == Self::Yes
    }
}
