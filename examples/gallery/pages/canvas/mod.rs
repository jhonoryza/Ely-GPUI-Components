use gpui::{AnyElement, App, Window, div, prelude::*};

mod plane;

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 32,
    slug: "canvas",
    title: "Canvas & Design",
    summary: "An endless plane to lay work on, the tools and panels around it, and graphs of nodes.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("canvas-plane", 420.0, 300.0),
    Step::DragTo("canvas-plane", 300.0, 240.0),
    Step::UpAt("canvas-plane", 300.0, 240.0),
    Step::Wait(200),
    Step::Shot("panned"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div().child(plane::plane(window, cx)).into_any_element()
}
