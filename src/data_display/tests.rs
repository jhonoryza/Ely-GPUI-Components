use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use gpui::{
    Bounds, Context, IntoElement, Modifiers, ParentElement, Pixels, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, point, px,
};

use super::{Carousel, DescriptionList, PropertyGrid, PropertyGroup};
use crate::{primitives::Measure, theme::Theme};

/// A property grid whose drawn height the test reads.
struct Inspector(Rc<Cell<Pixels>>);

impl Render for Inspector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let height = self.0.clone();
        let editor = || div().h(px(20.0));
        div().w(px(320.0)).child(
            Measure::new("measure", move |bounds, _, _| {
                height.set(bounds.size.height)
            })
            .child(
                PropertyGrid::new("grid")
                    .group(
                        PropertyGroup::new("Layout")
                            .row("Width", editor())
                            .row("Height", editor()),
                    )
                    .group(PropertyGroup::new("Export").folded().row("Scale", editor())),
            ),
        )
    }
}

/// Frames 2ms apart, past reduced motion's 1ms folds.
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

#[gpui::test]
fn a_property_group_folds_from_its_header_and_stays_folded(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
    let height = Rc::new(Cell::new(Pixels::ZERO));
    let seen = height.clone();
    let (_, cx) = cx.add_window_view(|_, _| Inspector(seen));
    settle(cx);
    let open = height.get();
    let header = point(px(4.0), px(12.0));
    cx.simulate_click(header, Modifiers::none());
    settle(cx);
    let folded = height.get();
    assert!(folded < open, "the rows fold away: {open:?} to {folded:?}");
    settle(cx);
    assert_eq!(height.get(), folded, "a new frame keeps the fold");
    cx.simulate_click(header, Modifiers::none());
    settle(cx);
    assert_eq!(height.get(), open, "a second press opens it");
}

/// A one-row description list, this wide, that reports where the list and its value sit.
struct Facts(Pixels, Rc<Cell<[Bounds<Pixels>; 2]>>);

impl Render for Facts {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (list, value) = (self.1.clone(), self.1.clone());
        let value = Measure::new("value", move |bounds, _, _| {
            value.set([value.get()[0], bounds])
        })
        .child("Apache-2.0");
        div().w(self.0).child(
            Measure::new("list", move |bounds, _, _| {
                list.set([bounds, list.get()[1]])
            })
            .child(DescriptionList::new().item("License", value)),
        )
    }
}

#[gpui::test]
fn a_value_drops_below_its_label_when_the_row_runs_short(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let mut place = |width: f32| {
        let seen = Rc::new(Cell::new([Bounds::default(); 2]));
        let shared = seen.clone();
        let (_, cx) = cx.add_window_view(move |_, _| Facts(px(width), shared));
        settle(cx);
        let [list, value] = seen.get();
        (value.left() - list.left(), value.top() - list.top())
    };
    let (wide_left, wide_top) = place(400.0);
    let (narrow_left, narrow_top) = place(240.0);
    assert_eq!(
        wide_left,
        px(176.0),
        "wide, the value sits beside its label"
    );
    assert_eq!(
        narrow_left,
        Pixels::ZERO,
        "narrow, the value starts under its label"
    );
    assert!(
        narrow_top > wide_top,
        "and a line lower: {wide_top:?} to {narrow_top:?}"
    );
}

/// A three-slide carousel, 200 wide, that turns every five seconds; each slide reports where it sits.
struct Deck(Rc<RefCell<[Pixels; 3]>>);

impl Render for Deck {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let slides = (0..3).map(|ix| {
            let seen = self.0.clone();
            Measure::new(("slide", ix), move |bounds, _, _| {
                seen.borrow_mut()[ix] = bounds.origin.x
            })
            .size_full()
        });
        Carousel::new("deck")
            .autoplay(Duration::from_secs(5))
            .w(px(200.0))
            .h(px(100.0))
            .children(slides)
    }
}

fn deck(cx: &mut TestAppContext) -> (Rc<RefCell<[Pixels; 3]>>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
    let places = Rc::new(RefCell::new([Pixels::ZERO; 3]));
    let seen = places.clone();
    let (_, cx) = cx.add_window_view(|_, _| Deck(seen));
    settle(cx);
    (places, cx)
}

/// The slide in view: the one at the viewport's left edge, inside its border.
fn front(places: &Rc<RefCell<[Pixels; 3]>>) -> Option<usize> {
    places.borrow().iter().position(|x| *x == px(1.0))
}

fn wait(time: Duration, cx: &mut VisualTestContext) {
    cx.executor().advance_clock(time);
    settle(cx);
}

#[gpui::test]
fn autoplay_turns_the_carousel_and_holds_while_pointed_at(cx: &mut TestAppContext) {
    let (places, cx) = deck(cx);
    assert_eq!(front(&places), Some(0));
    wait(Duration::from_secs(5), cx);
    assert_eq!(front(&places), Some(1), "five seconds turn it once");
    cx.simulate_mouse_move(point(px(100.0), px(50.0)), None, Modifiers::none());
    settle(cx);
    wait(Duration::from_secs(12), cx);
    assert_eq!(front(&places), Some(1), "held while pointed at");
    cx.simulate_mouse_move(point(px(600.0), px(400.0)), None, Modifiers::none());
    settle(cx);
    wait(Duration::from_secs(5), cx);
    assert_eq!(front(&places), Some(2));
}

#[gpui::test]
fn left_from_the_first_slide_wraps_to_the_last(cx: &mut TestAppContext) {
    let (places, cx) = deck(cx);
    cx.simulate_click(point(px(100.0), px(50.0)), Modifiers::none());
    settle(cx);
    cx.simulate_keystrokes("left");
    settle(cx);
    assert_eq!(front(&places), Some(2));
    cx.simulate_keystrokes("right");
    settle(cx);
    assert_eq!(front(&places), Some(0));
}
