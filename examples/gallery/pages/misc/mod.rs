use ely_gpui_component::misc::{Clock, Stopwatch, WorldClock};
use gpui::{AnyElement, App, IntoElement, ParentElement, Styled, Window, div, px};
use jiff::tz::TimeZone;

use super::Page;
use crate::probe::probe;
use crate::script::Step;
use crate::ui::{section, specimen, specimens};

pub const PAGE: Page = Page {
    number: 42,
    slug: "misc",
    title: "Misc",
    summary: "Small tools that stand alone: clocks, a stopwatch.",
    render,
    script: &[
        Step::DownAt("stopwatch-start", 30.0, 62.0),
        Step::UpAt("stopwatch-start", 30.0, 62.0),
        Step::Wait(1_600),
        Step::DownAt("stopwatch-start", 30.0, 62.0),
        Step::UpAt("stopwatch-start", 30.0, 62.0),
        Step::Wait(300),
        Step::Shot("stopwatch"),
    ],
};

fn zone(name: &str) -> TimeZone {
    TimeZone::get(name).unwrap_or_else(|error| panic!("the zone {name}: {error}"))
}

fn render(_: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(clocks(cx))
        .child(stopwatch(cx))
        .into_any_element()
}

fn clocks(cx: &App) -> impl IntoElement + use<> {
    section(
        "Clock / WorldClock",
        "The time in a zone, a face over its digits, ticking each second. A world clock lays places in rows: how each one's day and hour stand against home.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("clock", Clock::new("clock").label("Here"), cx))
            .child(specimen(
                "world clock",
                div().w(px(320.0)).child(WorldClock::new(
                    "world-clock",
                    [
                        ("San Francisco", zone("America/Los_Angeles")),
                        ("New York", zone("America/New_York")),
                        ("London", zone("Europe/London")),
                        ("Mumbai", zone("Asia/Kolkata")),
                        ("Tokyo", zone("Asia/Tokyo")),
                    ],
                )),
                cx,
            )),
    )
}

fn stopwatch(cx: &App) -> impl IntoElement + use<> {
    section(
        "Stopwatch",
        "Time that runs while started and holds while paused, in tenths; Lap marks the time so far, Reset clears it once paused.",
        cx,
    )
    .child(probe("stopwatch-start", div().w(px(320.0)).child(Stopwatch::new("stopwatch"))))
}
