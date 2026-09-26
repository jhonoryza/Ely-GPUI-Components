use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// File views' measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileSizes {
    /// A tile in a grid of files, wide.
    pub tile: Rems,
}

impl Theme {
    pub fn files(&self) -> FileSizes {
        FileSizes {
            tile: px_to_rems(96.0),
        }
    }
}
