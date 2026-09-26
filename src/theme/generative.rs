use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// Generative tools' measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerativeSizes {
    /// A style preset's picture, square.
    pub preset: Rems,
}

impl Theme {
    pub fn generative(&self) -> GenerativeSizes {
        GenerativeSizes {
            preset: px_to_rems(88.0),
        }
    }
}
