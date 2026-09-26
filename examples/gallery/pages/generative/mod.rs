mod panel;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 25,
    slug: "generative",
    title: "Generative",
    summary: "Making pictures, sound and video with a model: the prompt and its settings, the queue and what comes back, the canvas, and the models behind them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("gen-enhancer", 356.0, 124.0),
    Step::UpAt("gen-enhancer", 356.0, 124.0),
    Step::Wait(1400),
    Step::Shot("suggested"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div().child(panel::panel(window, cx)).into_any_element()
}
