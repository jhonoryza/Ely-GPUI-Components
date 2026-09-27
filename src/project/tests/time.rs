use std::time::Duration;

use gpui::{AnyElement, Entity, IntoElement, TestAppContext};
use jiff::civil::date;

use super::{Desk, desk, heard, note, press, settle, tab_to};
use crate::project::{DatabaseView, PomodoroTimer, Status, Task, TimeEntry, TimeTracker};

fn pomodoro(owner: Entity<Desk>) -> AnyElement {
    PomodoroTimer::new("pomodoro")
        .minutes(2, 1, 3)
        .rounds(2)
        .on_phase(move |phase, _, cx| note(&owner, phase.words().into(), cx))
        .into_any_element()
}

#[gpui::test]
fn a_paused_phase_keeps_its_time_and_one_run_out_hands_on_the_next(cx: &mut TestAppContext) {
    let (host, cx) = desk(pomodoro, cx);
    tab_to(2, cx);
    press("space", cx);
    cx.executor().advance_clock(Duration::from_secs(61));
    settle(cx);
    press("space", cx);
    cx.executor().advance_clock(Duration::from_secs(600));
    settle(cx);
    assert!(
        heard(&host, cx).is_empty(),
        "a paused minute of two stays short"
    );
    press("space", cx);
    cx.executor().advance_clock(Duration::from_secs(60));
    settle(cx);
    assert_eq!(heard(&host, cx), ["Short break"]);
}

fn tracker(owner: Entity<Desk>) -> AnyElement {
    let entry = TimeEntry::new("e1", "Brief", Duration::from_secs(600));
    TimeTracker::new("tracker", [entry])
        .on_start(move |task, _, _, cx| note(&owner, format!("start {task}"), cx))
        .into_any_element()
}

#[gpui::test]
fn a_named_task_starts_on_enter_and_a_blank_one_does_not(cx: &mut TestAppContext) {
    let (host, cx) = desk(tracker, cx);
    tab_to(1, cx);
    press("enter", cx);
    cx.simulate_input("  Site photos ");
    press("enter", cx);
    assert_eq!(heard(&host, cx), ["start Site photos"]);
}

fn database(owner: Entity<Desk>) -> AnyElement {
    let tasks = [
        Task::new("a", "Alpha")
            .starts(date(2026, 9, 20))
            .due(date(2026, 9, 30)),
        Task::new("b", "Beta").status(Status::Done),
    ];
    DatabaseView::new("db", tasks)
        .today(date(2026, 9, 27))
        .on_status(move |key, status, _, cx| note(&owner, format!("{key} {}", status.words()), cx))
        .into_any_element()
}

#[gpui::test]
fn the_switch_shows_each_way_and_the_board_moves_a_record(cx: &mut TestAppContext) {
    let (host, cx) = desk(database, cx);
    assert!(cx.debug_bounds("table-row a").is_some(), "a table first");
    tab_to(5, cx);
    press("space", cx);
    assert!(
        cx.debug_bounds("initiative a").is_some(),
        "a dated record on the timeline"
    );
    assert!(
        cx.debug_bounds("initiative b").is_none(),
        "an undated one left off"
    );
    tab_to(2, cx);
    press("space", cx);
    tab_to(6, cx);
    press("alt-right", cx);
    assert_eq!(heard(&host, cx), ["a In progress"]);
}
