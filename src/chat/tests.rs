use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Context, Entity, Focusable, IntoElement, Modifiers, ParentElement, Render,
    ScrollDelta, ScrollWheelEvent, Styled, TestAppContext, TouchPhase, VisualTestContext, Window,
    div, point, px,
};

use super::{MessageEditor, MessageList};
use crate::{forms, forms::TextInput, theme::Theme};

type Seen = Rc<RefCell<Vec<usize>>>;

/// A conversation of tall messages, noting which ones each frame drew.
struct Talk {
    count: usize,
    seen: Seen,
}

impl Render for Talk {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let seen = self.seen.clone();
        seen.borrow_mut().clear();
        div()
            .size_full()
            .child(MessageList::new("talk", self.count, move |ix, _, _| {
                seen.borrow_mut().push(ix);
                div().h(px(80.0)).w_full().into_any_element()
            }))
    }
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn newest(seen: &Seen) -> usize {
    *seen.borrow().iter().max().expect("a frame draws messages")
}

#[gpui::test]
fn the_list_follows_the_newest_until_the_reader_scrolls_away(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let seen = Seen::default();
    let noted = seen.clone();
    let (talk, cx) = cx.add_window_view(move |_, _| Talk {
        count: 40,
        seen: noted,
    });
    settle(cx);
    assert_eq!(newest(&seen), 39, "it opens at the newest");
    talk.update(cx, |talk, cx| {
        talk.count = 42;
        cx.notify();
    });
    settle(cx);
    assert_eq!(newest(&seen), 41, "at the newest, it follows");
    let middle = cx.update(|window, _| {
        let size = window.viewport_size();
        point(size.width / 2.0, size.height / 2.0)
    });
    cx.simulate_event(ScrollWheelEvent {
        position: middle,
        delta: ScrollDelta::Pixels(point(px(0.0), px(1200.0))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
    let away = newest(&seen);
    assert!(away < 41, "scrolled up to {away}");
    talk.update(cx, |talk, cx| {
        talk.count = 45;
        cx.notify();
    });
    settle(cx);
    assert_eq!(newest(&seen), away, "away, it holds its place");
    let button = cx.update(|window, _| {
        let size = window.viewport_size();
        point(size.width / 2.0, size.height - px(28.0))
    });
    cx.simulate_click(button, Modifiers::none());
    settle(cx);
    assert_eq!(newest(&seen), 44, "the button goes back to the newest");
}

/// A sent message open to change, and what the reader asked.
struct Edit {
    field: Entity<TextInput>,
    saved: usize,
    cancelled: usize,
}

impl Render for Edit {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (save, cancel) = (cx.entity(), cx.entity());
        div().size_full().child(MessageEditor::new(
            "edit",
            &self.field,
            "Lift blends toward white.",
            move |_, cx| save.update(cx, |edit, _| edit.saved += 1),
            move |_, cx| cancel.update(cx, |edit, _| edit.cancelled += 1),
        ))
    }
}

#[gpui::test]
fn an_edit_saves_only_a_change_and_escape_cancels(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
    });
    let (edit, cx) = cx.add_window_view(|window, cx| Edit {
        field: cx.new(|cx| {
            let mut input = TextInput::new(window, cx);
            input.set_text("Lift blends toward white.", cx);
            input
        }),
        saved: 0,
        cancelled: 0,
    });
    cx.update(|window, cx| {
        window.activate_window();
        let field = edit.read(cx).field.focus_handle(cx);
        window.focus(&field);
    });
    settle(cx);
    cx.simulate_keystrokes("cmd-enter");
    settle(cx);
    assert_eq!(
        edit.read_with(cx, |edit, _| edit.saved),
        0,
        "nothing changed"
    );
    cx.simulate_input(" Gently.");
    settle(cx);
    cx.simulate_keystrokes("cmd-enter");
    settle(cx);
    assert_eq!(edit.read_with(cx, |edit, _| edit.saved), 1);
    cx.simulate_keystrokes("escape");
    assert_eq!(edit.read_with(cx, |edit, _| edit.cancelled), 1);
}

#[gpui::test]
fn the_editor_takes_focus_as_it_opens(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
    });
    let (edit, cx) = cx.add_window_view(|window, cx| Edit {
        field: cx.new(|cx| TextInput::new(window, cx)),
        saved: 0,
        cancelled: 0,
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    cx.simulate_keystrokes("escape");
    assert_eq!(
        edit.read_with(cx, |edit, _| edit.cancelled),
        1,
        "Escape reaches the editor"
    );
}
