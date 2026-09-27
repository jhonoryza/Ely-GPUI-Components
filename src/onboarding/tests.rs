use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, IntoElement, KeyBinding, ParentElement, Render,
    Styled, TestAppContext, VisualTestContext, Window, div, px,
};

use super::{
    FeatureHighlight, Hotspot, OnboardingStep, OnboardingWizard, SetupChecklist, SetupTask,
};
use crate::{
    buttons::Button,
    forms,
    primitives::{FocusNext, FocusScope, IconName},
    theme::Theme,
};

type Part = fn(&Bench, Entity<Bench>) -> AnyElement;

/// A view that shows one onboarding part, keeps what it heard, and holds the owner's step, readiness and what it hid, with the root focus an app keeps.
struct Bench {
    part: Part,
    said: Vec<String>,
    root: FocusHandle,
    hidden: bool,
    step: usize,
    ready: bool,
    all_done: bool,
}

impl Render for Bench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(360.0))
            .p_4()
            .child((self.part)(self, cx.entity()))
    }
}

fn bench(part: Part, cx: &mut TestAppContext) -> (Entity<Bench>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
        forms::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
    let (host, cx) = cx.add_window_view(|_, cx| Bench {
        part,
        said: Vec::new(),
        root: cx.focus_handle(),
        hidden: false,
        step: 0,
        ready: true,
        all_done: false,
    });
    settle(cx);
    (host, cx)
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    cx.run_until_parked();
}

fn say(owner: &Entity<Bench>, words: String, cx: &mut App) {
    owner.update(cx, |bench, cx| {
        bench.said.push(words);
        cx.notify();
    });
}

fn said(host: &Entity<Bench>, cx: &mut VisualTestContext) -> Vec<String> {
    host.read_with(cx, |bench, _| bench.said.clone())
}

fn edit(host: &Entity<Bench>, cx: &mut VisualTestContext, change: impl FnOnce(&mut Bench)) {
    host.update(cx, |bench, cx| {
        change(bench);
        cx.notify();
    });
    settle(cx);
}

/// Focus on the `stops`th Tab stop from the top.
fn tab(stops: usize, cx: &mut VisualTestContext) {
    cx.update(|window, _| {
        window.blur();
        (0..stops).for_each(|_| window.focus_next());
    });
    settle(cx);
}

fn tap(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

fn wizard(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let steps = ["name", "invite", "theme"].map(|key| OnboardingStep {
        key: key.into(),
        title: format!("About {key}").into(),
        body: "A line on it".into(),
    });
    let [stepper, finisher, skipper] = [(); 3].map(|_| owner.clone());
    OnboardingWizard::new("welcome", steps, bench.step)
        .ready(bench.ready)
        .content(div().child("Fields"))
        .on_step(move |to, _, cx| {
            stepper.update(cx, |bench, cx| {
                bench.step = to;
                bench.said.push(format!("step {to}"));
                cx.notify();
            })
        })
        .on_finish(move |_, cx| say(&finisher, "finish".into(), cx))
        .on_skip(move |_, cx| say(&skipper, "skip".into(), cx))
        .into_any_element()
}

/// Stops: Skip, then Next on the first step; Skip, Back, then Next or Finish after it.
#[gpui::test]
fn the_wizard_walks_on_and_back_and_finishes(cx: &mut TestAppContext) {
    let (host, cx) = bench(wizard, cx);
    assert!(cx.debug_bounds("onboarding-Step 1 of 3").is_some());
    let (first, second) = (
        cx.debug_bounds("progress-fill-0")
            .expect("the bar's first part"),
        cx.debug_bounds("progress-fill-1")
            .expect("the bar's second part"),
    );
    assert!(
        first.size.width > px(0.0) && second.size.width == px(0.0),
        "the first of three parts is full: {first:?}, {second:?}"
    );
    tab(2, cx);
    tap("space", cx);
    tab(2, cx);
    tap("space", cx);
    edit(&host, cx, |bench| bench.step = 2);
    assert!(cx.debug_bounds("onboarding-Step 3 of 3").is_some());
    tab(3, cx);
    tap("space", cx);
    tab(1, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["step 1", "step 0", "finish", "skip"]);
}

/// While the step's fields are not ready, Next rests, so the second stop wraps to Skip.
#[gpui::test]
fn next_rests_until_the_step_is_ready(cx: &mut TestAppContext) {
    let (host, cx) = bench(wizard, cx);
    edit(&host, cx, |bench| bench.ready = false);
    tab(2, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["skip"]);
}

/// On the step before last, the stops are Skip, Back and Next: Next turns to Finish under focus, so a second press finishes.
#[gpui::test]
fn next_keeps_focus_as_it_turns_to_finish(cx: &mut TestAppContext) {
    let (host, cx) = bench(wizard, cx);
    edit(&host, cx, |bench| bench.step = 1);
    tab(3, cx);
    tap("space", cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["step 2", "finish"]);
}

/// Back from the second step leaves Back at rest on the first, so focus moves to Next, and a press walks on again.
#[gpui::test]
fn back_to_the_first_step_hands_focus_to_next(cx: &mut TestAppContext) {
    let (host, cx) = bench(wizard, cx);
    edit(&host, cx, |bench| bench.step = 1);
    tab(2, cx);
    tap("space", cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["step 0", "step 1"]);
}

fn hotspot(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    Hotspot::new("pins", "Pin a view", "Keep it one press away.")
        .on_dismiss(move |_, cx| say(&owner, "seen".into(), cx))
        .into_any_element()
}

/// Stops: the dot; Enter opens the tip, where Got it is the one stop.
#[gpui::test]
fn a_hotspot_opens_its_tip_from_the_keyboard_and_is_dismissed(cx: &mut TestAppContext) {
    let (host, cx) = bench(hotspot, cx);
    assert!(
        cx.debug_bounds("hotspot-tip").is_none(),
        "the tip rests shut"
    );
    tab(1, cx);
    tap("enter", cx);
    assert!(cx.debug_bounds("hotspot-tip").is_some());
    tap("escape", cx);
    tap("enter", cx);
    tap("tab", cx);
    tap("space", cx);
    assert_eq!(
        said(&host, cx),
        ["seen"],
        "Escape handed focus back to the dot"
    );
}

fn hidden_hotspot(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let after = owner.clone();
    let spot = (!bench.hidden).then(|| {
        Hotspot::new("pins", "Pin a view", "Keep it one press away.").on_dismiss(move |_, cx| {
            owner.update(cx, |bench, cx| {
                bench.said.push("seen".into());
                bench.hidden = true;
                cx.notify();
            })
        })
    });
    FocusScope::new(&bench.root)
        .root()
        .children(spot)
        .child(
            Button::new("after", "After").on_click(move |_, _, cx| say(&after, "after".into(), cx)),
        )
        .into_any_element()
}

/// Got it closes the tip before the owner hides the dot, so the root keeps focus and Tab reaches the next stop.
#[gpui::test]
fn after_got_it_tab_reaches_the_next_stop(cx: &mut TestAppContext) {
    let (host, cx) = bench(hidden_hotspot, cx);
    cx.update(|window, _| window.activate_window());
    tab(1, cx);
    tap("enter", cx);
    tap("tab", cx);
    tap("space", cx);
    tap("tab", cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["seen", "after"]);
}

fn highlight(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    let later = owner.clone();
    FeatureHighlight::new(
        "split",
        IconName::Columns2,
        "Split view",
        "Work on two files side by side.",
    )
    .on_try(move |_, cx| say(&owner, "try".into(), cx))
    .on_dismiss(move |_, cx| say(&later, "later".into(), cx))
    .into_any_element()
}

/// Stops: Try it, then Not now.
#[gpui::test]
fn a_feature_highlight_is_tried_or_put_off(cx: &mut TestAppContext) {
    let (host, cx) = bench(highlight, cx);
    tab(1, cx);
    tap("space", cx);
    tab(2, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["try", "later"]);
}

fn checklist(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let task = |key: &str, done: bool| SetupTask {
        key: key.to_string().into(),
        title: format!("Do {key}").into(),
        body: "Why it helps".into(),
        done: done || bench.all_done,
    };
    let hidden = owner.clone();
    SetupChecklist::new(
        "setup",
        "Get started",
        [
            task("profile", true),
            task("invite", false),
            task("connect", false),
        ],
    )
    .on_start(move |key, _, cx| say(&owner, format!("start {key}"), cx))
    .on_dismiss(move |_, cx| say(&hidden, "hide".into(), cx))
    .into_any_element()
}

/// Stops: Start on each task not done, so the third wraps to the first; once all are done, Hide.
#[gpui::test]
fn a_setup_checklist_starts_what_is_left_and_hides_when_done(cx: &mut TestAppContext) {
    let (host, cx) = bench(checklist, cx);
    assert!(cx.debug_bounds("setup-1 of 3 done").is_some());
    tab(2, cx);
    tap("space", cx);
    tab(3, cx);
    tap("space", cx);
    edit(&host, cx, |bench| bench.all_done = true);
    assert!(cx.debug_bounds("setup-3 of 3 done").is_some());
    tab(1, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["start connect", "start invite", "hide"]);
}
