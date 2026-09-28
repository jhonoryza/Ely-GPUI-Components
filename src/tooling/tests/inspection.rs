use std::time::Duration;

use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyUpEvent, Keystroke, Modifiers,
    ParentElement, Render, ScrollDelta, ScrollWheelEvent, Styled, TestAppContext, TouchPhase,
    VisualTestContext, Window, div, point, px,
};

use crate::{
    primitives::FocusScope,
    theme::Theme,
    tooling::{EventLogger, FpsMeter, RenderCounter, install_inspector},
};

fn shown(selector: &'static str, cx: &mut VisualTestContext) -> bool {
    cx.debug_bounds(selector).is_some()
}

fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

struct Sample;

impl Render for Sample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            div()
                .id("sample")
                .mt(px(4.0))
                .ml(px(4.0))
                .p(px(8.0))
                .border_2()
                .w(px(120.0))
                .h(px(40.0))
                .child("Sample"),
        )
    }
}

/// Hovering picks a box and the panel gives its size and layers; a press holds it, and Pick picks again.
#[gpui::test]
fn the_inspector_shows_the_picked_box(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        install_inspector(cx);
    });
    let (_, cx) = cx.add_window_view(|_, _| Sample);
    cx.update(|window, cx| window.toggle_inspector(cx));
    cx.run_until_parked();
    assert!(shown("inspector-picking-true", cx));
    cx.simulate_mouse_move(point(px(60.0), px(24.0)), None, Modifiers::none());
    assert!(shown("inspector-size-120x40", cx), "the hovered box");
    assert!(shown("inspector-margin-4-0-0-4", cx));
    assert!(shown("inspector-border-2-2-2-2", cx));
    assert!(shown("inspector-padding-8-8-8-8", cx));
    assert!(
        shown("inspector-content-100x20", cx),
        "less border and padding"
    );
    cx.simulate_click(point(px(60.0), px(24.0)), Modifiers::none());
    // gpui holds a pick without a redraw; the panel's own frame requests do not run in tests.
    redraw(cx);
    assert!(shown("inspector-picking-false", cx), "a press holds it");
    let pick = cx
        .debug_bounds("inspector-pick")
        .expect("Pick shows once held");
    cx.simulate_click(pick.center(), Modifiers::none());
    assert!(
        cx.update(|window, cx| window.is_inspector_picking(cx)),
        "Pick picks again"
    );
}

struct Meter;

impl Render for Meter {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(FpsMeter::new("meter"))
    }
}

/// The meter counts a frame per draw on the executor's clock, and names the worst.
#[gpui::test]
fn the_meter_counts_frames(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (_, cx) = cx.add_window_view(|_, _| Meter);
    for _ in 0..60 {
        cx.executor().advance_clock(Duration::from_millis(20));
        redraw(cx);
    }
    assert!(shown("fps-50", cx), "a frame each 20 ms");
    assert!(shown("fps-worst-20.0", cx));
    cx.executor().advance_clock(Duration::from_millis(100));
    redraw(cx);
    assert!(shown("fps-worst-100.0", cx), "one slow frame");
}

struct Counted;

impl Render for Counted {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(RenderCounter::new("counter", "Page"))
    }
}

/// Each render of the view that draws it counts once.
#[gpui::test]
fn the_counter_counts_renders(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (_, cx) = cx.add_window_view(|_, _| Counted);
    cx.run_until_parked();
    assert!(shown("renders-Page-1", cx), "the first frame");
    redraw(cx);
    redraw(cx);
    assert!(shown("renders-Page-3", cx));
}

struct Logged {
    root: FocusHandle,
    target: FocusHandle,
}

impl Render for Logged {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .p(px(10.0))
            .child(
                EventLogger::new("logger").child(
                    div()
                        .id("target")
                        .track_focus(&self.target)
                        .h(px(100.0))
                        .capture_any_mouse_down(|_, _, cx| cx.stop_propagation()),
                ),
            )
    }
}

/// Pointer events inside the box, even ones a child stops as they arrive, and keys while focus is inside, list with what they carried, placed in the box.
#[gpui::test]
fn the_logger_lists_events(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (view, cx) = cx.add_window_view(|_, cx| Logged {
        root: cx.focus_handle(),
        target: cx.focus_handle(),
    });
    let target = view.read_with(cx, |logged, _| logged.target.clone());
    cx.update(|window, _| window.focus(&target));
    cx.run_until_parked();
    assert!(shown("event-logger-empty", cx));
    cx.simulate_mouse_move(point(px(20.0), px(20.0)), None, Modifiers::none());
    assert!(shown("event-move to 10, 10", cx));
    cx.simulate_mouse_move(point(px(30.0), px(40.0)), None, Modifiers::none());
    assert!(shown("event-move to 20, 30 ×2", cx), "moves run together");
    cx.simulate_click(point(px(30.0), px(40.0)), Modifiers::none());
    assert!(
        shown("event-press left at 20, 30", cx),
        "a press the child stops"
    );
    assert!(shown("event-release left at 20, 30", cx));
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(30.0), px(40.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-24.0))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    assert!(shown("event-wheel 0, -24 px", cx));
    cx.simulate_keystrokes("a");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("a").expect("a key"),
    });
    assert!(shown("event-key down a", cx));
    assert!(shown("event-key up a", cx));
    cx.simulate_modifiers_change(Modifiers::shift());
    assert!(shown("event-modifiers shift", cx));
    cx.simulate_click(point(px(30.0), px(410.0)), Modifiers::none());
    assert!(
        !shown("event-press left at 20, 400", cx),
        "a press outside the box"
    );
}
