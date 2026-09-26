use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Bounds, Context, Entity, IntoElement, Modifiers, ParentElement, Pixels, Point,
    Render, ScrollDelta, ScrollWheelEvent, Styled, TestAppContext, TouchPhase, VisualTestContext,
    Window, div, point, px, size,
};

use super::{DocPage, DocumentViewer, PageHit};
use crate::{
    editor::FindWidget,
    forms::TextInput,
    theme::{ActiveTheme, ControlSize, Theme},
};

type Noted = Rc<RefCell<Option<(usize, Point<f32>)>>>;

/// Three tall pages with one hit on the second; a press while pinning records the note asked.
struct Reader {
    query: Entity<TextInput>,
    current: Option<usize>,
    noted: Noted,
}

impl Render for Reader {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let pages: Vec<DocPage> = (0..3)
            .map(|ix| DocPage {
                source: format!("page-{ix}.png").into(),
                width: 400.0,
                height: 1000.0,
            })
            .collect();
        let hit = PageHit {
            page: 1,
            bounds: Bounds::new(point(40.0, 400.0), size(60.0, 12.0)),
        };
        let noted = self.noted.clone();
        div().size_full().child(
            DocumentViewer::new("viewer", "lift.pdf", pages)
                .find(
                    FindWidget::new("find", &self.query, self.current, 1),
                    vec![hit],
                    self.current,
                )
                .notes(Vec::new(), move |page, at, _, _| {
                    *noted.borrow_mut() = Some((page, at))
                }),
        )
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Reader>, Noted, &mut VisualTestContext) {
    cx.update(Theme::init);
    let noted = Noted::default();
    let seen = noted.clone();
    let (reader, cx) = cx.add_window_view(move |window, cx| Reader {
        query: cx.new(|cx| TextInput::new(window, cx)),
        current: None,
        noted: seen,
    });
    settle(cx);
    (reader, noted, cx)
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

/// A point on the pages, below the header and the find bar, at the view's middle.
fn on_pages(cx: &mut VisualTestContext) -> Point<Pixels> {
    cx.update(|window, _| point(window.viewport_size().width / 2.0, px(300.0)))
}

fn wheel(down: f32, cx: &mut VisualTestContext) {
    let at = on_pages(cx);
    cx.simulate_event(ScrollWheelEvent {
        position: at,
        delta: ScrollDelta::Pixels(point(px(0.0), px(-down))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
}

/// Turns pinning on with the header's last button, then presses the pages; returns the note asked.
fn pin(cx: &mut VisualTestContext, noted: &Noted) -> (usize, Point<f32>) {
    let toggle = cx.update(|window, cx| {
        let header = cx
            .theme()
            .control_height(ControlSize::Lg)
            .to_pixels(window.rem_size());
        point(
            window.viewport_size().width - px(17.0),
            px(1.0) + header / 2.0,
        )
    });
    cx.simulate_click(toggle, Modifiers::none());
    settle(cx);
    let at = on_pages(cx);
    cx.simulate_click(at, Modifiers::none());
    settle(cx);
    noted
        .borrow_mut()
        .take()
        .expect("a press while pinning asks for a note")
}

#[gpui::test]
fn a_note_lands_where_the_press_is_on_a_scrolled_page(cx: &mut TestAppContext) {
    let (_, noted, cx) = open(cx);
    let (page, top) = pin(cx, &noted);
    wheel(150.0, cx);
    let at = on_pages(cx);
    cx.simulate_click(at, Modifiers::none());
    settle(cx);
    let (scrolled_page, scrolled) = noted.borrow_mut().take().expect("pinning stays on");
    assert_eq!((scrolled_page, page), (0, 0));
    assert_eq!(scrolled.x, top.x);
    assert_eq!(scrolled.y, top.y + 150.0, "the press counts the scroll");
}

/// Scrolls `down`, makes the hit current, then pins a note at the same window point.
fn revealed(down: f32, cx: &mut TestAppContext) -> (usize, Point<f32>) {
    let (reader, noted, cx) = open(cx);
    if down > 0.0 {
        wheel(down, cx);
    }
    reader.update(cx, |reader, cx| {
        reader.current = Some(0);
        cx.notify();
    });
    settle(cx);
    pin(cx, &noted)
}

#[gpui::test]
fn a_current_hit_comes_to_one_place_from_any_scroll(cx: &mut TestAppContext) {
    let from_top = revealed(0.0, cx);
    let from_below = revealed(300.0, cx);
    assert_eq!(from_top.0, 1, "the hit's page comes into view");
    assert_eq!(from_top, from_below);
}
