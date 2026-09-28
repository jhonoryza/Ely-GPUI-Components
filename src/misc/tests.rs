use std::time::Duration;

use gpui::{
    AnyElement, Context, FocusHandle, IntoElement, KeyUpEvent, Keystroke, ParentElement, Render,
    Styled, TestAppContext, VisualTestContext, Window, div, px,
};
use jiff::{Timestamp, tz::TimeZone};

use super::{Clock, Stopwatch, WorldClock};
use crate::{
    primitives::{FocusNext, FocusPrev, FocusScope},
    theme::Theme,
};

struct Stage {
    root: FocusHandle,
    part: fn() -> AnyElement,
}

impl Render for Stage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .child(div().w(px(400.0)).child((self.part)()))
    }
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    cx.run_until_parked();
}

fn stage(part: fn() -> AnyElement, cx: &mut TestAppContext) -> &mut VisualTestContext {
    cx.update(|cx| {
        Theme::init(cx);
        cx.bind_keys([
            gpui::KeyBinding::new("tab", FocusNext, None),
            gpui::KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
    });
    let (view, cx) = cx.add_window_view(|_, cx| Stage {
        root: cx.focus_handle(),
        part,
    });
    let root = view.read_with(cx, |stage, _| stage.root.clone());
    cx.update(|window, _| window.focus(&root));
    settle(cx);
    cx
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

fn wait(ms: u64, cx: &mut VisualTestContext) {
    cx.executor().advance_clock(Duration::from_millis(ms));
    settle(cx);
}

fn shown(selector: &'static str, cx: &mut VisualTestContext) -> bool {
    cx.debug_bounds(selector).is_some()
}

fn watch() -> AnyElement {
    Stopwatch::new("watch").into_any_element()
}

/// Start runs the time on the executor's clock, Pause holds it, and Reset clears it for the next run.
#[gpui::test]
fn a_stopwatch_runs_holds_and_resets(cx: &mut TestAppContext) {
    let cx = stage(watch, cx);
    press("tab", cx);
    press("enter", cx);
    wait(1_250, cx);
    assert!(shown("stopwatch-00:01.2", cx), "running");
    press("enter", cx);
    wait(2_000, cx);
    assert!(
        shown("stopwatch-00:01.2", cx) && !shown("stopwatch-00:03.2", cx),
        "Pause, on the same button, holds it"
    );
    press("tab", cx);
    press("enter", cx);
    press("shift-tab", cx);
    press("enter", cx);
    wait(300, cx);
    assert!(
        shown("stopwatch-00:00.3", cx),
        "Reset cleared it, and Start ran it from nothing"
    );
}

fn clocks() -> AnyElement {
    let now: Timestamp = "2026-09-28T03:00:00Z".parse().expect("a moment");
    let zone = |name: &str| TimeZone::get(name).expect("a zone");
    div()
        .child(
            Clock::new("clock")
                .zone(zone("Europe/Paris"))
                .now(now)
                .label("Paris"),
        )
        .child(
            WorldClock::new(
                "world",
                [
                    ("Tokyo", zone("Asia/Tokyo")),
                    ("Kolkata", zone("Asia/Kolkata")),
                ],
            )
            .home(zone("America/New_York"))
            .now(now),
        )
        .into_any_element()
}

/// Clocks shown at the owner's moment draw without a ticker.
#[gpui::test]
fn clocks_draw_at_the_owners_moment(cx: &mut TestAppContext) {
    let cx = stage(clocks, cx);
    wait(5_000, cx);
}
