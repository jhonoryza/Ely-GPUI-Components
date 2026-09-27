use gpui::Rems;

use super::{Theme, tokens::px_to_rems};

/// Calendar's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarSizes {
    /// An event's chip in a day, tall.
    pub chip: Rems,
    /// Rows of events a month's day holds under its number.
    pub day_rows: usize,
    /// A day in a year's small months, square.
    pub mini_day: Rems,
}

impl Theme {
    pub fn calendar(&self) -> CalendarSizes {
        CalendarSizes {
            chip: px_to_rems(20.0),
            day_rows: 3,
            mini_day: px_to_rems(28.0),
        }
    }
}
