use gpui::{
    AnyElement, Context, FocusHandle, InteractiveElement, IntoElement, KeyUpEvent, Keystroke,
    ParentElement, Render, Styled, TestAppContext, VisualTestContext, Window, div, px,
};

use super::{Knob, Playground};
use crate::{
    forms,
    primitives::{FocusNext, FocusPrev, FocusScope},
    theme::Theme,
};

struct Stage {
    root: FocusHandle,
}

impl Render for Stage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .child(div().w(px(600.0)).child(playground()))
    }
}

fn playground() -> AnyElement {
    Playground::new(
        "playground",
        [
            Knob::Toggle("disabled".into(), false),
            Knob::Choice(
                "size".into(),
                vec!["Sm".into(), "Md".into(), "Lg".into()],
                1,
            ),
            Knob::Number("share".into(), 0.4, (0.0, 1.0), 0.1),
        ],
        |settings, _, _| {
            let shown = format!(
                "preview-{}-{}-{:.1}",
                settings.on("disabled"),
                settings.picked("size"),
                settings.number("share")
            );
            div().debug_selector(|| shown).into_any_element()
        },
    )
    .into_any_element()
}

fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    cx.run_until_parked();
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

/// Each knob's edit redraws the preview with the new settings.
#[gpui::test]
fn knobs_redraw_the_preview(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
        cx.bind_keys([
            gpui::KeyBinding::new("tab", FocusNext, None),
            gpui::KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
    });
    let (view, cx) = cx.add_window_view(|_, cx| Stage {
        root: cx.focus_handle(),
    });
    let root = view.read_with(cx, |stage, _| stage.root.clone());
    cx.update(|window, _| window.focus(&root));
    settle(cx);
    let shown =
        |selector: &'static str, cx: &mut VisualTestContext| cx.debug_bounds(selector).is_some();
    assert!(shown("preview-false-Md-0.4", cx));
    press("tab", cx);
    press("space", cx);
    assert!(shown("preview-true-Md-0.4", cx), "the switch turned");
    press("tab", cx);
    for key in ["down", "down", "enter"] {
        press(key, cx);
    }
    assert!(shown("preview-true-Lg-0.4", cx), "the choice moved");
    press("tab", cx);
    press("right", cx);
    assert!(shown("preview-true-Lg-0.5", cx), "the number stepped");
}
