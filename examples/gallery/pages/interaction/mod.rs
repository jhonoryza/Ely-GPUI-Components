use gpui::{AnyElement, App, Window, div, prelude::*};

mod pointer;

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 38,
    slug: "interaction",
    title: "Interaction",
    summary: "Drag and drop, resizing and turning, band selection, presses and gestures.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("interaction-tiles", 296.0, 6.0),
    Step::DragTo("interaction-tiles", 120.0, 120.0),
    Step::Wait(100),
    Step::Shot("band"),
    Step::UpAt("interaction-tiles", 120.0, 120.0),
    Step::DownAt("interaction-rotatable", 98.0, 22.0),
    Step::DragTo("interaction-rotatable", 116.0, 70.0),
    Step::UpAt("interaction-rotatable", 116.0, 70.0),
    Step::Wait(100),
    Step::Shot("turned"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(pointer::drag_and_drop(window, cx))
        .child(pointer::shapes(window, cx))
        .into_any_element()
}
