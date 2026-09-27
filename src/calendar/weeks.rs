use std::cmp::Reverse;

use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};

use super::event::{Event, When};
use crate::forms::week_start;

/// An event that spans days, laid across a week row: its lane from the top, its first and last column, and whether it runs on past either end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bar {
    pub event: usize,
    pub lane: usize,
    pub from: usize,
    pub to: usize,
    pub before: bool,
    pub after: bool,
}

/// The Mondays of the week rows that hold `month`'s days: four to six.
pub(crate) fn weeks(month: Date) -> Vec<Date> {
    let last = month.last_of_month();
    let mut monday = week_start(month.first_of_month());
    let mut mondays = Vec::with_capacity(6);
    while monday <= last {
        mondays.push(monday);
        monday = monday.checked_add(1.week()).expect("a week follows");
    }
    mondays
}

fn column(monday: Date, day: Date) -> usize {
    let days = monday.until(day).expect("days apart").get_days();
    usize::try_from(days).expect("a day in its week")
}

/// Bars for the week from `monday`: earliest first and longer first, each in the first lane free across its days.
pub(crate) fn bars(events: &[Event], monday: Date, zone: &TimeZone) -> Vec<Bar> {
    let sunday = monday.checked_add(6.days()).expect("a week ends");
    let mut spans: Vec<(usize, Date, Date)> = events
        .iter()
        .enumerate()
        .filter(|(_, event)| event.spans(zone))
        .map(|(ix, event)| {
            let (first, last) = event.days(zone);
            (ix, first, last)
        })
        .filter(|&(_, first, last)| first <= sunday && last >= monday)
        .collect();
    spans.sort_by_key(|&(ix, first, last)| (first, Reverse(last), ix));
    let mut ends: Vec<Date> = Vec::new();
    spans
        .into_iter()
        .map(|(event, first, last)| {
            let (from, to) = (first.max(monday), last.min(sunday));
            let lane = ends
                .iter()
                .position(|end| *end < from)
                .unwrap_or(ends.len());
            match ends.get_mut(lane) {
                Some(end) => *end = to,
                None => ends.push(to),
            }
            Bar {
                event,
                lane,
                from: column(monday, from),
                to: column(monday, to),
                before: first < monday,
                after: last > sunday,
            }
        })
        .collect()
}

fn start(event: &Event) -> Timestamp {
    match event.when {
        When::Timed { start, .. } => start,
        When::AllDay { .. } => panic!("event {}: an all-day event spans", event.key),
    }
}

/// The events on `day` alone, by start.
pub(crate) fn singles(events: &[Event], day: Date, zone: &TimeZone) -> Vec<usize> {
    let mut on: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, event)| !event.spans(zone) && event.days(zone).0 == day)
        .map(|(ix, _)| ix)
        .collect();
    on.sort_by_key(|&ix| (start(&events[ix]), ix));
    on
}

/// What a day shows under its week's bars: how many of its `singles` fit in `room` rows, and how many more wait, counting `hidden` bars past the lanes shown.
pub(crate) fn day_rows(singles: usize, hidden: usize, room: usize) -> (usize, usize) {
    if hidden == 0 && singles <= room {
        return (singles, 0);
    }
    let shown = room.saturating_sub(1).min(singles);
    (shown, singles - shown + hidden)
}

#[cfg(test)]
mod tests {
    use jiff::{Timestamp, civil::date, tz::TimeZone};

    use super::{Bar, bars, day_rows, singles, weeks};
    use crate::calendar::Event;

    fn at(day: i8, hour: i8) -> Timestamp {
        date(2026, 9, day)
            .at(hour, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .expect("a UTC time")
            .timestamp()
    }

    fn bar(event: usize, lane: usize, from: usize, to: usize) -> Bar {
        Bar {
            event,
            lane,
            from,
            to,
            before: false,
            after: false,
        }
    }

    #[test]
    fn a_month_takes_the_weeks_that_hold_its_days() {
        assert_eq!(
            weeks(date(2026, 9, 15)),
            [
                date(2026, 8, 31),
                date(2026, 9, 7),
                date(2026, 9, 14),
                date(2026, 9, 21),
                date(2026, 9, 28)
            ]
        );
        assert_eq!(weeks(date(2027, 2, 1)).len(), 4, "a February from a Monday");
        assert_eq!(
            weeks(date(2026, 8, 1)).len(),
            6,
            "an August from a Saturday"
        );
    }

    #[test]
    fn bars_take_the_first_free_lane_and_break_at_the_week() {
        let events = [
            Event::all_day(
                "offsite",
                "Offsite",
                date(2026, 9, 22),
                date(2026, 9, 24),
                0,
            ),
            Event::all_day("trip", "Trip", date(2026, 9, 24), date(2026, 9, 29), 1),
            Event::all_day("fair", "Fair", date(2026, 9, 25), date(2026, 9, 26), 2),
            Event::timed("night", "Night shoot", at(21, 22), at(22, 2), 3),
        ];
        let utc = &TimeZone::UTC;
        assert_eq!(
            bars(&events, date(2026, 9, 21), utc),
            [
                bar(3, 0, 0, 1),
                bar(0, 1, 1, 3),
                Bar {
                    after: true,
                    ..bar(1, 0, 3, 6)
                },
                bar(2, 1, 4, 5),
            ]
        );
        assert_eq!(
            bars(&events, date(2026, 9, 28), utc),
            [Bar {
                before: true,
                ..bar(1, 0, 0, 1)
            }]
        );
    }

    #[test]
    fn a_day_shows_what_fits_and_counts_the_rest() {
        assert_eq!(day_rows(3, 0, 3), (3, 0));
        assert_eq!(day_rows(5, 0, 3), (2, 3), "two chips, then 3 more");
        assert_eq!(day_rows(0, 1, 1), (0, 1), "a hidden bar still counts");
        assert_eq!(day_rows(2, 1, 2), (1, 2));
    }

    #[test]
    fn a_days_own_events_come_by_the_hour() {
        let events = [
            Event::timed("late", "Late", at(22, 14), at(22, 15), 0),
            Event::timed("early", "Early", at(22, 9), at(22, 10), 0),
            Event::all_day("all", "All", date(2026, 9, 22), date(2026, 9, 22), 0),
            Event::timed("next", "Next", at(23, 9), at(23, 10), 0),
        ];
        assert_eq!(singles(&events, date(2026, 9, 22), &TimeZone::UTC), [1, 0]);
    }

    #[test]
    fn an_end_at_midnight_leaves_the_next_day_alone() {
        let late = Event::timed("late", "Late", at(22, 20), at(23, 0), 0);
        assert_eq!(
            late.days(&TimeZone::UTC),
            (date(2026, 9, 22), date(2026, 9, 22))
        );
        assert!(!late.spans(&TimeZone::UTC));
    }
}
