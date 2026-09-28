use gpui::{Pixels, Rems, px};

use super::{Theme, tokens::px_to_rems};

/// A map's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapSizes {
    /// A tile's side at its own zoom, as tile servers cut them.
    pub tile: Pixels,
    /// A popup's width, less where the map is narrower.
    pub popup: Rems,
    /// Room a popup keeps from the map's edges and from its point.
    pub margin: Rems,
}

impl Theme {
    pub fn maps(&self) -> MapSizes {
        MapSizes {
            tile: px(256.0),
            popup: px_to_rems(240.0),
            margin: px_to_rems(8.0),
        }
    }
}
