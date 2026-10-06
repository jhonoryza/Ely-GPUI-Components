use gpui::{AppContext, Context, IntoElement, Render, TestAppContext, Window};

use super::setup;
use crate::forms::{TextInput, UnitInput};

#[gpui::test]
fn a_selection_outside_the_text_or_inside_a_character_is_left(cx: &mut TestAppContext) {
    setup(cx);
    let (_, cx) = cx.add_window_view(|_, _| gpui::Empty);
    let input = cx.update(|window, cx| cx.new(|cx| TextInput::new(window, cx)));
    input.update(cx, |input, cx| {
        input.set_text("héllo", cx);
        input.select(0..1, cx);
        input.select(0..2, cx);
        input.select(3..99, cx);
        assert_eq!(input.selection(), 0..1);
    });
}

/// A unit field whose list may drop the chosen unit.
struct Units {
    shortened: bool,
}

impl Render for Units {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let units = if self.shortened {
            vec!["em"]
        } else {
            vec!["px", "em"]
        };
        UnitInput::new("length", 12.0, units, "px")
    }
}

#[gpui::test]
fn a_unit_gone_from_its_list_keeps_the_field(cx: &mut TestAppContext) {
    setup(cx);
    let (host, cx) = cx.add_window_view(|_, _| Units { shortened: false });
    cx.run_until_parked();
    host.update(cx, |host, cx| {
        host.shortened = true;
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("number-root").is_some());
}
