use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// The misc chapter's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiscSizes {
    /// A clock's face.
    pub face: Rems,
    /// A world clock row's face.
    pub small_face: Rems,
    /// A flashcard's least height.
    pub card: Rems,
}

impl Theme {
    pub fn misc(&self) -> MiscSizes {
        MiscSizes {
            face: px_to_rems(112.0),
            small_face: px_to_rems(32.0),
            card: px_to_rems(176.0),
        }
    }
}
