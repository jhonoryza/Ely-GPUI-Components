use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// Project's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectSizes {
    /// A board's column, wide.
    pub column: Rems,
}

impl Theme {
    pub fn project(&self) -> ProjectSizes {
        ProjectSizes {
            column: px_to_rems(272.0),
        }
    }
}
