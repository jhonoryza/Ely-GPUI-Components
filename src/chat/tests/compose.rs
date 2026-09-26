use gpui::{
    AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement, Render, SharedString,
    Styled, TestAppContext, VisualTestContext, Window, div,
};

use super::settle;
use crate::{
    chat::PromptInput,
    forms::{self, Choice, TextInput},
    theme::Theme,
};

/// A composer over a field, and what it asked: sends, commands and context.
struct Compose {
    field: Entity<TextInput>,
    busy: bool,
    sent: usize,
    picked: Vec<SharedString>,
}

impl Render for Compose {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (send, command, mention) = (cx.entity(), cx.entity(), cx.entity());
        let input = PromptInput::new("compose", &self.field, move |_, cx| {
            send.update(cx, |compose, _| compose.sent += 1)
        })
        .commands(
            [Choice::new("summarize", "/summarize")],
            move |value, _, cx| command.update(cx, |compose, _| compose.picked.push(value.clone())),
        )
        .context([Choice::new("lift", "lift.rs")], move |value, _, cx| {
            mention.update(cx, |compose, _| compose.picked.push(value.clone()))
        });
        let input = if self.busy {
            input.busy(|_, _| {})
        } else {
            input
        };
        div().size_full().child(input)
    }
}

fn open(busy: bool, cx: &mut TestAppContext) -> (Entity<Compose>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
    });
    let (compose, cx) = cx.add_window_view(move |window, cx| Compose {
        field: cx.new(|cx| TextInput::new(window, cx).multi_line(1, 8)),
        busy,
        sent: 0,
        picked: Vec::new(),
    });
    cx.update(|window, cx| {
        window.activate_window();
        let field = compose.read(cx).field.focus_handle(cx);
        window.focus(&field);
    });
    settle(cx);
    (compose, cx)
}

fn text(compose: &Entity<Compose>, cx: &mut VisualTestContext) -> String {
    cx.update(|_, cx| compose.read(cx).field.read(cx).text().to_string())
}

#[gpui::test]
fn enter_sends_and_shift_enter_breaks_the_line(cx: &mut TestAppContext) {
    let (compose, cx) = open(false, cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(
        compose.read_with(cx, |compose, _| compose.sent),
        0,
        "nothing to send"
    );
    cx.simulate_input("lift");
    cx.simulate_keystrokes("shift-enter");
    settle(cx);
    assert_eq!(text(&compose, cx), "lift\n");
    cx.simulate_input("tone");
    settle(cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(compose.read_with(cx, |compose, _| compose.sent), 1);
    assert_eq!(
        text(&compose, cx),
        "lift\ntone",
        "Enter sends without a new line"
    );
}

#[gpui::test]
fn a_slash_picks_a_command_and_an_at_sign_picks_context(cx: &mut TestAppContext) {
    let (compose, cx) = open(false, cx);
    cx.simulate_input("/sum");
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(text(&compose, cx), "", "the command's words go");
    cx.simulate_input("see @li");
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(text(&compose, cx), "see ");
    compose.read_with(cx, |compose, _| {
        assert_eq!(compose.picked, ["summarize", "lift"]);
        assert_eq!(compose.sent, 0, "a pick is not a send");
    });
}

#[gpui::test]
fn a_busy_composer_holds_its_message(cx: &mut TestAppContext) {
    let (compose, cx) = open(true, cx);
    cx.simulate_input("wait");
    settle(cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(compose.read_with(cx, |compose, _| compose.sent), 0);
    assert_eq!(text(&compose, cx), "wait");
}
