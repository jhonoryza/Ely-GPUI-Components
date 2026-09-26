mod blocks;
mod counts;
mod format;
mod link;
mod markdown;
mod modes;
mod render;
mod rich;
mod suggest;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod todo;
mod toolbar;
mod tree;

use gpui::App;

pub use blocks::{Align, BlockData, BlockEditor, BlockEvent, BlockKind, Media};
pub use counts::{ReadingTime, WordCount};
pub use format::{Format, format};
pub use link::LinkEditor;
pub use markdown::markdown_highlights;
pub use modes::{MarkdownEditor, MarkdownMode};
pub use render::MarkdownRenderer;
pub use rich::RichTextEditor;
pub use todo::{Checklist, TodoItem};
pub use toolbar::FixedFormatToolbar;

/// Binds the documents keys. `init` calls it.
pub(crate) fn bind_keys(cx: &mut App) {
    rich::bind_keys(cx);
}
