use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// Media tools' measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaSizes {
    /// A crop box's handle, square.
    pub crop_handle: Rems,
    /// A mark's line on a picture.
    pub mark_stroke: Rems,
    /// An arrow's head, from its tip.
    pub arrow_head: Rems,
    /// A numbered pin, round.
    pub pin: Rems,
}

impl Theme {
    pub fn media(&self) -> MediaSizes {
        MediaSizes {
            crop_handle: px_to_rems(10.0),
            mark_stroke: px_to_rems(2.0),
            arrow_head: px_to_rems(12.0),
            pin: px_to_rems(20.0),
        }
    }
}
