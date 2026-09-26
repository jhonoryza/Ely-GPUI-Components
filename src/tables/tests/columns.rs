use std::{cell::RefCell, rc::Rc};

use gpui::{
    Context, IntoElement, KeyUpEvent, Keystroke, Modifiers, Render, Styled, TestAppContext, Window,
    point, px,
};

use super::{edit_at, settle};
use crate::{
    tables::{Column, DataTable, Row},
    theme::Theme,
};

/// Three editable columns 200 wide in a table 280 wide, the first pinned or not; it keeps the column of each edit.
struct Wide(Rc<RefCell<Vec<String>>>, bool);

impl Render for Wide {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let heard = self.0.clone();
        let column = |key: &'static str| Column::new(key, key).width(gpui::rems(12.5)).editable();
        let first = if self.1 {
            column("a").pinned()
        } else {
            column("a")
        };
        DataTable::new("wide", [first, column("b"), column("c")])
            .rows(vec![Row::new("r", ["A".into(), "B".into(), "C".into()])])
            .on_edit(move |_, column, _, _, _| heard.borrow_mut().push(column.to_string()))
            .w(px(280.0))
    }
}

#[gpui::test]
fn columns_past_the_table_scroll_sideways_and_a_plain_wheel_leaves_them(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        crate::forms::bind_keys(cx);
    });
    for pinned in [false, true] {
        let heard = Rc::new(RefCell::new(Vec::new()));
        let seen = heard.clone();
        let (_, cx) = cx.add_window_view(|_, _| Wide(seen, pinned));
        settle(cx);
        let cell = point(px(250.0), px(54.0));
        for delta in [point(px(0.0), px(-2000.0)), point(px(-2000.0), px(0.0))] {
            cx.simulate_event(gpui::ScrollWheelEvent {
                position: cell,
                delta: gpui::ScrollDelta::Pixels(delta),
                modifiers: Modifiers::none(),
                touch_phase: gpui::TouchPhase::Moved,
            });
            settle(cx);
            edit_at(cell, "x", cx);
        }
        assert_eq!(*heard.borrow(), ["b", "c"], "pinned {pinned}");
    }
}

/// Two rows under a sortable, editable name; it keeps the row of each edit.
struct Sorting(Rc<RefCell<Vec<String>>>);

impl Render for Sorting {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let heard = self.0.clone();
        DataTable::new("sorting", [Column::new("name", "Name").editable()])
            .rows(vec![
                Row::new("bea", ["Bea".into()]),
                Row::new("ada", ["Ada".into()]),
            ])
            .on_edit(move |row, _, _, _, _| heard.borrow_mut().push(row.to_string()))
            .w(px(400.0))
    }
}

#[gpui::test]
fn tab_reaches_a_header_and_enter_sorts_by_it(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        crate::forms::bind_keys(cx);
    });
    let heard = Rc::new(RefCell::new(Vec::new()));
    let seen = heard.clone();
    let (_, cx) = cx.add_window_view(|_, _| Sorting(seen));
    cx.update(|window, _| window.activate_window());
    settle(cx);
    cx.update(|window, _| window.focus_next());
    cx.simulate_keystrokes("enter");
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").expect("a key"),
    });
    settle(cx);
    edit_at(point(px(120.0), px(54.0)), "x", cx);
    assert_eq!(*heard.borrow(), ["ada"], "Ada rises to the top");
}
