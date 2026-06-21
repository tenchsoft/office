mod comments;
mod modals;
mod status;
mod style;
mod thumbnails;

pub use comments::paint_comment_panel;
pub use modals::{
    paint_goto_modal, paint_print_preview, paint_special_char_modal, paint_word_count_modal,
};
pub use status::paint_status_bar;
pub use style::paint_style_panel;
pub use thumbnails::paint_thumbnails;
