use std::{cell::RefCell, rc::Rc};

use gpui::{
    Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, point, px,
};

use super::{Breakpoint, BreakpointList, Disassembly, Instruction};
use crate::theme::Theme;

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

/// Two plain instructions; it keeps the gutter presses.
struct Listing(Rc<RefCell<Vec<usize>>>);

impl Render for Listing {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let store = self.0.clone();
        let plain = |address| Instruction {
            address,
            bytes: vec![0x90],
            mnemonic: "nop".into(),
            operands: "".into(),
            comment: None,
            source: None,
        };
        div().w(px(400.0)).h(px(200.0)).child(
            Disassembly::new("listing", vec![plain(0x10), plain(0x11)])
                .on_breakpoint(move |ix, _, _| store.borrow_mut().push(ix)),
        )
    }
}

#[gpui::test]
fn an_empty_gutter_takes_a_press_across_its_row(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let store = seen.clone();
    let (_, cx) = cx.add_window_view(|_, _| Listing(store));
    settle(cx);
    cx.simulate_click(point(px(10.0), px(2.0)), Modifiers::none());
    settle(cx);
    assert_eq!(*seen.borrow(), [0]);
}

/// One breakpoint; it keeps what its toggle and open report.
struct Points {
    toggles: Rc<RefCell<Vec<(usize, bool)>>>,
    opens: Rc<RefCell<Vec<usize>>>,
}

impl Render for Points {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (toggles, opens) = (self.toggles.clone(), self.opens.clone());
        let point = Breakpoint {
            path: "src/lift.rs".into(),
            line: 3,
            enabled: true,
            condition: None,
            hits: 0,
        };
        div().w(px(400.0)).child(
            BreakpointList::new("points", [point])
                .on_toggle(move |ix, on, _, _| toggles.borrow_mut().push((ix, on)))
                .on_open(move |ix, _, _| opens.borrow_mut().push(ix)),
        )
    }
}

#[gpui::test]
fn a_breakpoints_box_toggles_without_opening_it(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (toggles, opens) = (
        Rc::new(RefCell::new(Vec::new())),
        Rc::new(RefCell::new(Vec::new())),
    );
    let view = Points {
        toggles: toggles.clone(),
        opens: opens.clone(),
    };
    let (_, cx) = cx.add_window_view(|_, _| view);
    settle(cx);
    cx.simulate_click(point(px(14.0), px(12.0)), Modifiers::none());
    settle(cx);
    assert_eq!(*toggles.borrow(), [(0, false)]);
    assert!(
        opens.borrow().is_empty(),
        "the box press stays with the box"
    );
    cx.simulate_click(point(px(200.0), px(12.0)), Modifiers::none());
    settle(cx);
    assert_eq!(*opens.borrow(), [0], "the row still opens");
}
