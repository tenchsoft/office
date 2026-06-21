// ---------------------------------------------------------------------------
// Toolbar action dispatch
// ---------------------------------------------------------------------------

use tench_document_core::{Alignment, BlockType, MarkType};

use super::state::{
    LinkModalState, ParagraphStyle, TableGridState, ToolbarDropdown, FONT_FAMILIES, FONT_SIZES,
};
use super::toolbar_actions::ToolbarAction;
use super::KodocsApp;

impl KodocsApp {
    pub(super) fn handle_toolbar_action(&mut self, action: ToolbarAction) -> bool {
        match action {
            ToolbarAction::FormatButton(idx) => {
                self.state.hovered_btn = Some(idx);
                match idx {
                    0 => {
                        let result = self.engine().toggle_mark(MarkType::Bold);
                        self.state.bold = !self.state.bold;
                        self.state.apply_edit_result(result);
                    }
                    1 => {
                        let result = self.engine().toggle_mark(MarkType::Italic);
                        self.state.italic = !self.state.italic;
                        self.state.apply_edit_result(result);
                    }
                    2 => {
                        let result = self.engine().toggle_mark(MarkType::Underline);
                        self.state.underline = !self.state.underline;
                        self.state.apply_edit_result(result);
                    }
                    3 => {
                        let result = self.engine().toggle_mark(MarkType::Strikethrough);
                        self.state.strikethrough = !self.state.strikethrough;
                        self.state.apply_edit_result(result);
                    }
                    4 => {
                        let result = self.engine().toggle_mark(MarkType::Code);
                        self.state.code = !self.state.code;
                        self.state.apply_edit_result(result);
                    }
                    5 => {
                        let result = self.engine().toggle_mark(MarkType::Superscript);
                        self.state.superscript = !self.state.superscript;
                        if self.state.superscript {
                            self.state.subscript = false;
                        }
                        self.state.apply_edit_result(result);
                    }
                    6 => {
                        let result = self.engine().toggle_mark(MarkType::Subscript);
                        self.state.subscript = !self.state.subscript;
                        if self.state.subscript {
                            self.state.superscript = false;
                        }
                        self.state.apply_edit_result(result);
                    }
                    7 => {
                        self.state.open_dropdown =
                            if self.state.open_dropdown == Some(ToolbarDropdown::MarkPicker) {
                                None
                            } else {
                                Some(ToolbarDropdown::MarkPicker)
                            };
                    }
                    _ => return false,
                }
                true
            }
            ToolbarAction::BulletList => {
                let result = self.engine().toggle_list(false);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::NumberedList => {
                let result = self.engine().toggle_list(true);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Checklist => {
                let result = self.engine().toggle_task_list();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Outdent => {
                let result = self.engine().outdent();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Indent => {
                let result = self.engine().indent();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignLeft => {
                let result = self.engine().set_alignment(Alignment::Left);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignCenter => {
                let result = self.engine().set_alignment(Alignment::Center);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignRight => {
                let result = self.engine().set_alignment(Alignment::Right);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignJustify => {
                let result = self.engine().set_alignment(Alignment::Justify);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::HorizontalRule => {
                let result = self.engine().insert_horizontal_rule();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::BlockQuote => {
                let result = self.engine().set_block_type(BlockType::BlockQuote);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::ToggleThumbnails => {
                self.state.show_thumbnails = !self.state.show_thumbnails;
                true
            }
            ToolbarAction::ZoomIn => {
                self.state.zoom = (self.state.zoom + 10.0).min(200.0);
                true
            }
            ToolbarAction::ZoomOut => {
                self.state.zoom = (self.state.zoom - 10.0).max(50.0);
                true
            }
            ToolbarAction::ToggleStylePanel => {
                self.state.show_style_panel = !self.state.show_style_panel;
                true
            }
            ToolbarAction::ToggleComments => {
                self.state.show_comments = !self.state.show_comments;
                true
            }
            ToolbarAction::ToggleTrackChanges => {
                self.state.track_changes = !self.state.track_changes;
                true
            }
            ToolbarAction::FontFamilySelect => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::FontFamily) {
                        None
                    } else {
                        Some(ToolbarDropdown::FontFamily)
                    };
                true
            }
            ToolbarAction::FontFamilyItem(idx) => {
                if let Some(&name) = FONT_FAMILIES.get(idx) {
                    self.state.current_font_family = name.to_string();
                    let result = self.engine().set_font_family(name.to_string());
                    self.state.apply_edit_result(result);
                }
                self.state.open_dropdown = None;
                true
            }
            ToolbarAction::FontSizeSelect => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::FontSize) {
                        None
                    } else {
                        Some(ToolbarDropdown::FontSize)
                    };
                true
            }
            ToolbarAction::FontSizeItem(idx) => {
                if let Some(&size) = FONT_SIZES.get(idx) {
                    self.state.current_font_size = size;
                    let result = self.engine().set_font_size(size);
                    self.state.apply_edit_result(result);
                }
                self.state.open_dropdown = None;
                true
            }
            ToolbarAction::ParagraphStyleSelect => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::ParagraphStyle) {
                        None
                    } else {
                        Some(ToolbarDropdown::ParagraphStyle)
                    };
                true
            }
            ToolbarAction::ParagraphStyleItem(idx) => {
                if let Some(&style) = ParagraphStyle::all().get(idx) {
                    self.state.current_paragraph_style = style;
                    let block_type = match style {
                        ParagraphStyle::Paragraph => BlockType::Paragraph,
                        ParagraphStyle::Heading1 => BlockType::Heading(1),
                        ParagraphStyle::Heading2 => BlockType::Heading(2),
                        ParagraphStyle::Heading3 => BlockType::Heading(3),
                        ParagraphStyle::Heading4 => BlockType::Heading(4),
                        ParagraphStyle::Heading5 => BlockType::Heading(5),
                        ParagraphStyle::Heading6 => BlockType::Heading(6),
                        ParagraphStyle::BlockQuote => BlockType::BlockQuote,
                        ParagraphStyle::CodeBlock => BlockType::CodeBlock,
                    };
                    let result = self.engine().set_block_type(block_type);
                    self.state.apply_edit_result(result);
                }
                self.state.open_dropdown = None;
                true
            }
            ToolbarAction::InsertLink => {
                self.state.link_modal = Some(LinkModalState::default());
                self.state.active_modal = Some("하이퍼링크 삽입".into());
                true
            }
            ToolbarAction::InsertImage => {
                self.insert_image_dialog();
                true
            }
            ToolbarAction::InsertTable => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::TableGrid) {
                        None
                    } else {
                        self.state.table_grid = TableGridState::default();
                        Some(ToolbarDropdown::TableGrid)
                    };
                true
            }
            ToolbarAction::TextColor => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::ColorPicker) {
                        None
                    } else {
                        Some(ToolbarDropdown::ColorPicker)
                    };
                true
            }
            ToolbarAction::MarkColor => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::MarkPicker) {
                        None
                    } else {
                        Some(ToolbarDropdown::MarkPicker)
                    };
                true
            }
        }
    }
}
