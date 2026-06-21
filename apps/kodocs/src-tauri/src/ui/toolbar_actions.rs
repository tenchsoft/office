// ---------------------------------------------------------------------------
// Toolbar action dispatch
// ---------------------------------------------------------------------------

/// Identifies which toolbar button was clicked, with semantic meaning.
#[allow(dead_code)] // Variants used via toolbar dispatch
pub(super) enum ToolbarAction {
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
    FontFamilySelect,
    FontSizeSelect,
    ParagraphStyleSelect,
    FontFamilyItem(usize),
    FontSizeItem(usize),
    ParagraphStyleItem(usize),
    InsertLink,
    InsertImage,
    InsertTable,
    TextColor,
    MarkColor,
}

/// Map toolbar x-position to a semantic action.
pub(super) fn toolbar_action_at(x: f64) -> Option<ToolbarAction> {
    let mut left = 12.0;
    // Group 1: Undo/Redo (2 buttons)
    left += 48.0 + 2.0 + 48.0 + 2.0 + 14.0; // buttons + separator

    // Group 2: Font family select
    let font_family_left = left;
    if x >= font_family_left && x < font_family_left + 120.0 {
        return Some(ToolbarAction::FontFamilySelect);
    }
    left += 120.0 + 2.0 + 14.0;

    // Group 3: Font size select
    let font_size_left = left;
    if x >= font_size_left && x < font_size_left + 62.0 {
        return Some(ToolbarAction::FontSizeSelect);
    }
    left += 62.0 + 2.0 + 14.0;

    // Group 4: Paragraph style select
    let para_left = left;
    if x >= para_left && x < para_left + 112.0 {
        return Some(ToolbarAction::ParagraphStyleSelect);
    }
    left += 112.0 + 2.0 + 14.0;

    // Group 4: Format buttons (B, I, U, S, <>, x², x_, 강조)
    let _format_start = left;
    let format_widths = [32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 36.0];
    for (idx, width) in format_widths.iter().enumerate() {
        if x >= left && x < left + *width {
            return Some(ToolbarAction::FormatButton(idx));
        }
        left += *width + 2.0;
    }
    left += 14.0; // separator

    // Group 5: List buttons (•, 1., ☑, ◀, ▶)
    let list_labels = [
        (28.0, ToolbarAction::BulletList),
        (28.0, ToolbarAction::NumberedList),
        (28.0, ToolbarAction::Checklist),
        (32.0, ToolbarAction::Outdent),
        (32.0, ToolbarAction::Indent),
    ];
    for (width, action) in list_labels {
        if x >= left && x < left + width {
            return Some(action);
        }
        left += width + 2.0;
    }
    left += 14.0; // separator

    // Group 6: Alignment buttons (좌, 중, 우, 양)
    let align_labels = [
        (28.0, ToolbarAction::AlignLeft),
        (28.0, ToolbarAction::AlignCenter),
        (28.0, ToolbarAction::AlignRight),
        (28.0, ToolbarAction::AlignJustify),
    ];
    for (width, action) in align_labels {
        if x >= left && x < left + width {
            return Some(action);
        }
        left += width + 2.0;
    }
    left += 14.0; // separator

    // Group 7: Insert buttons (링크, 그림, 표, 줄, 인용)
    let insert_labels = [
        (42.0, Some(ToolbarAction::InsertLink)),
        (38.0, Some(ToolbarAction::InsertImage)),
        (28.0, Some(ToolbarAction::InsertTable)),
        (28.0, Some(ToolbarAction::HorizontalRule)),
        (36.0, Some(ToolbarAction::BlockQuote)),
    ];
    for (width, action) in insert_labels {
        if x >= left && x < left + width {
            return action;
        }
        left += width + 2.0;
    }
    left += 14.0; // separator

    // Group 8: Color/Mark buttons (글자색, 배경색)
    let color_labels = [
        (50.0, Some(ToolbarAction::TextColor)),
        (50.0, Some(ToolbarAction::MarkColor)),
    ];
    for (width, action) in color_labels {
        if x >= left && x < left + width {
            return action;
        }
        left += width + 2.0;
    }
    let _ = left; // Group 8 end

    None
}
