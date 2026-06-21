use super::*;

// ---------------------------------------------------------------------------
// Tab color presets (shared between painting and click handling)
// ---------------------------------------------------------------------------

pub(crate) const TAB_COLOR_PRESETS: &[(&str, Color)] = &[
    ("Red", Color::rgb8(0xEF, 0x44, 0x44)),
    ("Orange", Color::rgb8(0xF9, 0x73, 0x16)),
    ("Yellow", Color::rgb8(0xFB, 0xBF, 0x24)),
    ("Green", Color::rgb8(0x22, 0xC5, 0x5E)),
    ("Teal", Color::rgb8(0x14, 0xB8, 0xA6)),
    ("Blue", Color::rgb8(0x3B, 0x82, 0xF6)),
    ("Indigo", Color::rgb8(0x63, 0x66, 0xF1)),
    ("Purple", Color::rgb8(0xA7, 0x8B, 0xFA)),
    ("Pink", Color::rgb8(0xEC, 0x48, 0x99)),
    ("Gray", Color::rgb8(0x6B, 0x72, 0x80)),
];

mod find_replace;
mod handlers;
mod insert_function;
mod modal;
mod named_ranges;
mod paste_special;
mod settings;
mod sort;

pub(crate) use find_replace::paint_find_replace_dialog;
pub(crate) use insert_function::paint_insert_function_dialog;
pub(crate) use modal::{paint_sheets_modal, paint_toast};
pub(crate) use named_ranges::paint_named_ranges_dialog;
pub(crate) use paste_special::paint_paste_special_dialog;
pub(crate) use settings::paint_settings_dialog;
pub(crate) use sort::paint_sort_dialog;
