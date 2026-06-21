// ---------------------------------------------------------------------------
// Toolbar action dispatch
// ---------------------------------------------------------------------------

/// Identifies which toolbar button was clicked, with semantic meaning.
#[allow(dead_code)] // Variants used via toolbar dispatch
pub(super) enum ToolbarAction {
    Undo,
    Redo,
    FormatButton(usize),
    BulletList,
    NumberedList,
    Checklist,
    Outdent,
    Indent,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    HorizontalRule,
    BlockQuote,
    ToggleThumbnails,
    ZoomIn,
    ZoomOut,
    ToggleStylePanel,
    ToggleComments,
    ToggleTrackChanges,
    FontSizeSelect,
    FontFamilySelect,
    ParagraphStyleSelect,
    FontSizeItem(usize),
    FontFamilyItem(usize),
    ParagraphStyleItem(usize),
    InsertLink,
    InsertImage,
    InsertTable,
    TextColor,
    MarkColor,
}

/// A single item in the toolbar layout.
pub(super) struct ToolbarItem {
    pub label: &'static str,
    pub width: f64,
    pub group: usize,
    pub action: ToolbarAction,
    pub tooltip: &'static str,
}

/// The canonical toolbar layout. Both painting and hit-testing iterate over this.
pub(super) static TOOLBAR_LAYOUT: &[ToolbarItem] = &[
    // Group 0: Undo/Redo
    ToolbarItem {
        label: "U",
        width: 32.0,
        group: 0,
        action: ToolbarAction::Undo,
        tooltip: "Undo",
    },
    ToolbarItem {
        label: "R",
        width: 32.0,
        group: 0,
        action: ToolbarAction::Redo,
        tooltip: "Redo",
    },
    // Group 1: Font size select (special - handled separately)
    // Group 2: Font family select (special - handled separately)
    // Group 3: Paragraph style select (special - handled separately)
    // Group 4: Format buttons
    ToolbarItem {
        label: "B",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(0),
        tooltip: "Bold",
    },
    ToolbarItem {
        label: "I",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(1),
        tooltip: "Italic",
    },
    ToolbarItem {
        label: "U",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(2),
        tooltip: "Underline",
    },
    ToolbarItem {
        label: "S",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(3),
        tooltip: "Strikethrough",
    },
    ToolbarItem {
        label: "<>",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(4),
        tooltip: "Code",
    },
    ToolbarItem {
        label: "x2",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(5),
        tooltip: "Superscript",
    },
    ToolbarItem {
        label: "x_",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(6),
        tooltip: "Subscript",
    },
    ToolbarItem {
        label: "H/L",
        width: 32.0,
        group: 4,
        action: ToolbarAction::FormatButton(7),
        tooltip: "Highlight",
    },
    // Group 5: List buttons
    ToolbarItem {
        label: "\u{25CF}",
        width: 34.0,
        group: 5,
        action: ToolbarAction::BulletList,
        tooltip: "Bullet List",
    },
    ToolbarItem {
        label: "1.",
        width: 34.0,
        group: 5,
        action: ToolbarAction::NumberedList,
        tooltip: "Numbered List",
    },
    ToolbarItem {
        label: "\u{2611}",
        width: 34.0,
        group: 5,
        action: ToolbarAction::Checklist,
        tooltip: "Checklist",
    },
    ToolbarItem {
        label: "\u{21E4}",
        width: 36.0,
        group: 5,
        action: ToolbarAction::Outdent,
        tooltip: "Outdent",
    },
    ToolbarItem {
        label: "\u{21E5}",
        width: 32.0,
        group: 5,
        action: ToolbarAction::Indent,
        tooltip: "Indent",
    },
    // Group 6: Alignment
    ToolbarItem {
        label: "L",
        width: 32.0,
        group: 6,
        action: ToolbarAction::AlignLeft,
        tooltip: "Align Left",
    },
    ToolbarItem {
        label: "C",
        width: 32.0,
        group: 6,
        action: ToolbarAction::AlignCenter,
        tooltip: "Align Center",
    },
    ToolbarItem {
        label: "R",
        width: 32.0,
        group: 6,
        action: ToolbarAction::AlignRight,
        tooltip: "Align Right",
    },
    ToolbarItem {
        label: "J",
        width: 32.0,
        group: 6,
        action: ToolbarAction::AlignJustify,
        tooltip: "Justify",
    },
    // Group 7: Insert
    ToolbarItem {
        label: "Link",
        width: 44.0,
        group: 7,
        action: ToolbarAction::InsertLink,
        tooltip: "Insert Link",
    },
    ToolbarItem {
        label: "Img",
        width: 38.0,
        group: 7,
        action: ToolbarAction::InsertImage,
        tooltip: "Insert Image",
    },
    ToolbarItem {
        label: "Tbl",
        width: 38.0,
        group: 7,
        action: ToolbarAction::InsertTable,
        tooltip: "Insert Table",
    },
    ToolbarItem {
        label: "Hr",
        width: 32.0,
        group: 7,
        action: ToolbarAction::HorizontalRule,
        tooltip: "Horizontal Rule",
    },
    ToolbarItem {
        label: "Quote",
        width: 48.0,
        group: 7,
        action: ToolbarAction::BlockQuote,
        tooltip: "Block Quote",
    },
    // Group 8: Color/Mark
    ToolbarItem {
        label: "Color",
        width: 52.0,
        group: 8,
        action: ToolbarAction::TextColor,
        tooltip: "Text Color",
    },
    ToolbarItem {
        label: "Mark",
        width: 48.0,
        group: 8,
        action: ToolbarAction::MarkColor,
        tooltip: "Highlight Color",
    },
];

/// Gap between toolbar buttons within a group.
pub(super) const BTN_GAP: f64 = 2.0;

/// Width of the separator between toolbar groups.
pub(super) const SEPARATOR_W: f64 = 14.0;

/// Width of the font size select dropdown.
pub(super) const FONT_SIZE_SELECT_W: f64 = 62.0;

/// Width of the font family select dropdown.
pub(super) const FONT_FAMILY_SELECT_W: f64 = 112.0;

/// Width of the paragraph style select dropdown.
pub(super) const PARAGRAPH_SELECT_W: f64 = 112.0;

/// Left padding at the start of the toolbar.
pub(super) const TOOLBAR_LEFT_PAD: f64 = 12.0;

/// Compute the x-position just after group 0 (Undo/Redo).
pub(super) fn after_group0_x() -> f64 {
    let mut left = TOOLBAR_LEFT_PAD;
    for item in TOOLBAR_LAYOUT.iter() {
        if item.group != 0 {
            break;
        }
        left += item.width + BTN_GAP;
    }
    left
}

/// Compute the x-position where the font size select starts.
pub(super) fn font_size_select_x() -> f64 {
    after_group0_x() + SEPARATOR_W
}

/// Compute the x-position where the font family select starts.
pub(super) fn font_family_select_x() -> f64 {
    font_size_select_x() + FONT_SIZE_SELECT_W + BTN_GAP + SEPARATOR_W
}

/// Compute the x-position where the paragraph style select starts.
pub(super) fn paragraph_style_select_x() -> f64 {
    font_family_select_x() + FONT_FAMILY_SELECT_W + BTN_GAP + SEPARATOR_W
}

/// Map toolbar x-position to a semantic action.
pub(super) fn toolbar_action_at(x: f64) -> Option<ToolbarAction> {
    // Check special dropdown groups first (groups 1-3)
    let fs_x = font_size_select_x();
    if x >= fs_x && x < fs_x + FONT_SIZE_SELECT_W {
        return Some(ToolbarAction::FontSizeSelect);
    }

    let ff_x = font_family_select_x();
    if x >= ff_x && x < ff_x + FONT_FAMILY_SELECT_W {
        return Some(ToolbarAction::FontFamilySelect);
    }

    let ps_x = paragraph_style_select_x();
    if x >= ps_x && x < ps_x + PARAGRAPH_SELECT_W {
        return Some(ToolbarAction::ParagraphStyleSelect);
    }

    // Iterate over TOOLBAR_LAYOUT items
    let mut left = TOOLBAR_LEFT_PAD;
    let mut prev_group = 0;

    for item in TOOLBAR_LAYOUT.iter() {
        // Add separator between groups
        if item.group != prev_group {
            left += SEPARATOR_W;
            // When transitioning from group 0 to group 4+, account for the
            // dropdown selects (font size, font family, paragraph style) that
            // the paint code inserts between these groups.
            if prev_group == 0 && item.group >= 4 {
                left += FONT_SIZE_SELECT_W + BTN_GAP;
                left += SEPARATOR_W;
                left += FONT_FAMILY_SELECT_W + BTN_GAP;
                left += SEPARATOR_W;
                left += PARAGRAPH_SELECT_W + BTN_GAP;
                left += SEPARATOR_W;
            }
            prev_group = item.group;
        }

        if x >= left && x < left + item.width {
            return Some(clone_action(&item.action));
        }
        left += item.width + BTN_GAP;
    }

    None
}

/// Clone a ToolbarAction for return.
fn clone_action(action: &ToolbarAction) -> ToolbarAction {
    match action {
        ToolbarAction::Undo => ToolbarAction::Undo,
        ToolbarAction::Redo => ToolbarAction::Redo,
        ToolbarAction::FormatButton(i) => ToolbarAction::FormatButton(*i),
        ToolbarAction::BulletList => ToolbarAction::BulletList,
        ToolbarAction::NumberedList => ToolbarAction::NumberedList,
        ToolbarAction::Checklist => ToolbarAction::Checklist,
        ToolbarAction::Outdent => ToolbarAction::Outdent,
        ToolbarAction::Indent => ToolbarAction::Indent,
        ToolbarAction::AlignLeft => ToolbarAction::AlignLeft,
        ToolbarAction::AlignCenter => ToolbarAction::AlignCenter,
        ToolbarAction::AlignRight => ToolbarAction::AlignRight,
        ToolbarAction::AlignJustify => ToolbarAction::AlignJustify,
        ToolbarAction::HorizontalRule => ToolbarAction::HorizontalRule,
        ToolbarAction::BlockQuote => ToolbarAction::BlockQuote,
        ToolbarAction::ToggleThumbnails => ToolbarAction::ToggleThumbnails,
        ToolbarAction::ZoomIn => ToolbarAction::ZoomIn,
        ToolbarAction::ZoomOut => ToolbarAction::ZoomOut,
        ToolbarAction::ToggleStylePanel => ToolbarAction::ToggleStylePanel,
        ToolbarAction::ToggleComments => ToolbarAction::ToggleComments,
        ToolbarAction::ToggleTrackChanges => ToolbarAction::ToggleTrackChanges,
        ToolbarAction::FontSizeSelect => ToolbarAction::FontSizeSelect,
        ToolbarAction::FontFamilySelect => ToolbarAction::FontFamilySelect,
        ToolbarAction::ParagraphStyleSelect => ToolbarAction::ParagraphStyleSelect,
        ToolbarAction::FontSizeItem(i) => ToolbarAction::FontSizeItem(*i),
        ToolbarAction::FontFamilyItem(i) => ToolbarAction::FontFamilyItem(*i),
        ToolbarAction::ParagraphStyleItem(i) => ToolbarAction::ParagraphStyleItem(*i),
        ToolbarAction::InsertLink => ToolbarAction::InsertLink,
        ToolbarAction::InsertImage => ToolbarAction::InsertImage,
        ToolbarAction::InsertTable => ToolbarAction::InsertTable,
        ToolbarAction::TextColor => ToolbarAction::TextColor,
        ToolbarAction::MarkColor => ToolbarAction::MarkColor,
    }
}

/// Compute the tooltip text and x-position for a toolbar x-coordinate.
/// Returns (tooltip_text, tooltip_x) if a tooltip should be shown.
pub(super) fn toolbar_tooltip_at(x: f64) -> Option<(&'static str, f64)> {
    // Check special dropdown groups - no tooltips for these
    let fs_x = font_size_select_x();
    if x >= fs_x && x < fs_x + FONT_SIZE_SELECT_W {
        return None;
    }
    let ff_x = font_family_select_x();
    if x >= ff_x && x < ff_x + FONT_FAMILY_SELECT_W {
        return None;
    }
    let ps_x = paragraph_style_select_x();
    if x >= ps_x && x < ps_x + PARAGRAPH_SELECT_W {
        return None;
    }

    // Iterate over TOOLBAR_LAYOUT items
    let mut left = TOOLBAR_LEFT_PAD;
    let mut prev_group = 0;

    for item in TOOLBAR_LAYOUT.iter() {
        if item.group != prev_group {
            left += SEPARATOR_W;
            // When transitioning from group 0 to group 4+, account for the
            // dropdown selects that the paint code inserts between these groups.
            if prev_group == 0 && item.group >= 4 {
                left += FONT_SIZE_SELECT_W + BTN_GAP;
                left += SEPARATOR_W;
                left += FONT_FAMILY_SELECT_W + BTN_GAP;
                left += SEPARATOR_W;
                left += PARAGRAPH_SELECT_W + BTN_GAP;
                left += SEPARATOR_W;
            }
            prev_group = item.group;
        }

        if x >= left && x < left + item.width {
            if item.tooltip.is_empty() {
                return None;
            }
            return Some((item.tooltip, left));
        }
        left += item.width + BTN_GAP;
    }

    None
}
