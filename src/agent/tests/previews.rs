use std::{cell::Cell, rc::Rc};

use gpui::{
    Context, IntoElement, KeyBinding, ParentElement, Render, Styled, TestAppContext, Window,
    canvas, div, point, px,
};

use super::{press, settle};
use crate::{
    agent::{ArtifactPanel, BrowserPreview, ComputerUseViewer},
    forms,
    primitives::{FocusNext, IconName},
    theme::Theme,
};

fn setup(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
}

/// A browser with three frames, and the frames asked for.
struct Browsing {
    asked: Vec<usize>,
}

impl Render for Browsing {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        div().w(px(480.0)).child(
            BrowserPreview::new("browser", "localhost:5173", ["a.png", "b.png", "c.png"], 2)
                .on_show(move |ix, _, cx| view.update(cx, |browsing, _| browsing.asked.push(ix))),
        )
    }
}

#[gpui::test]
fn a_frame_shows_from_the_keyboard(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Browsing { asked: Vec::new() });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    cx.update(|window, _| window.focus_next());
    cx.update(|window, _| window.focus_next());
    press("enter", cx);
    assert_eq!(
        view.read_with(cx, |browsing, _| browsing.asked.clone()),
        [1]
    );
}

#[test]
#[should_panic(expected = "frame 3 of 3")]
fn a_frame_past_the_last_fails_loud() {
    let _ = BrowserPreview::new("browser", "localhost", ["a.png", "b.png", "c.png"], 3);
}

#[test]
#[should_panic(expected = "a pointer inside the frame")]
fn a_pointer_off_the_frame_fails_loud() {
    let _ = ComputerUseViewer::new("screen", "a.png").pointer(point(1.2, 0.5));
}

/// A panel with a preview and a source, noting which one drew.
struct Panel {
    drew_source: Rc<Cell<bool>>,
}

impl Render for Panel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let drew = self.drew_source.clone();
        div().w(px(480.0)).h(px(320.0)).child(
            ArtifactPanel::new("panel", "Notes", IconName::FileText)
                .preview(div().h(px(40.0)))
                .source(
                    div().relative().h(px(40.0)).child(
                        canvas(move |_, _, _| drew.set(true), |_, _, _, _| {})
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full(),
                    ),
                ),
        )
    }
}

#[gpui::test]
fn the_source_shows_when_asked(cx: &mut TestAppContext) {
    setup(cx);
    let drew = Rc::new(Cell::new(false));
    let seen = drew.clone();
    let (_, cx) = cx.add_window_view(|_, _| Panel { drew_source: seen });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    assert!(!drew.get(), "the preview shows first");
    cx.update(|window, _| window.focus_next());
    cx.update(|window, _| window.focus_next());
    press("enter", cx);
    assert!(drew.get(), "the source shows once asked");
}
