use gpui::{
    Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window,
    anchored, div, point, prelude::*, px,
};

use super::raise;
use crate::theme::Theme;

/// A raised square holding a raised button, under a cover drawn after it in the same layer.
struct Stack {
    heard: Vec<&'static str>,
}

impl Render for Stack {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (inner, cover) = (cx.entity(), cx.entity());
        let button = div()
            .id("inner")
            .size(px(100.0))
            .occlude()
            .on_click(move |_, _, cx| inner.update(cx, |stack, _| stack.heard.push("inner")));
        let sheet = div()
            .id("cover")
            .absolute()
            .top_0()
            .left_0()
            .size(px(200.0))
            .on_click(move |_, _, cx| cover.update(cx, |stack, _| stack.heard.push("cover")));
        div().size_full().child(raise(
            anchored().position(point(px(0.0), px(0.0))).child(
                div()
                    .relative()
                    .size(px(200.0))
                    .child(raise(
                        anchored().position(point(px(50.0), px(50.0))).child(button),
                    ))
                    .child(sheet),
            ),
        ))
    }
}

#[gpui::test]
fn what_is_raised_inside_a_raise_lies_over_it(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (view, cx) = cx.add_window_view(|_, _| Stack { heard: Vec::new() });
    cx.run_until_parked();
    cx.simulate_click(point(px(100.0), px(100.0)), Modifiers::none());
    assert_eq!(
        view.read_with(cx, |stack, _| stack.heard.clone()),
        ["inner"]
    );
}
