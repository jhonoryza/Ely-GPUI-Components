use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// The misc chapter's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiscSizes {
    /// A clock's face.
    pub face: Rems,
    /// A world clock row's face.
    pub small_face: Rems,
}

impl Theme {
    pub fn misc(&self) -> MiscSizes {
        MiscSizes {
            face: px_to_rems(112.0),
            small_face: px_to_rems(32.0),
        }
    }
}
