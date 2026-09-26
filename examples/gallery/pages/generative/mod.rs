mod canvas;
mod panel;
mod results;

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
    Step::DownAt("gen-queue", 426.0, 280.0),
    Step::UpAt("gen-queue", 426.0, 280.0),
    Step::Wait(400),
    Step::Shot("retried"),
    Step::DownAt("gen-ab", 181.0, 147.0),
    Step::UpAt("gen-ab", 181.0, 147.0),
    Step::Wait(300),
    Step::Shot("verdict"),
    Step::DownAt("gen-canvas", 300.0, 70.0),
    Step::UpAt("gen-canvas", 390.0, 220.0),
    Step::Wait(300),
    Step::Shot("masked"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(panel::panel(window, cx))
        .child(results::queue(window, cx))
        .child(results::results(window, cx))
        .child(results::variations(window, cx))
        .child(canvas::canvas(window, cx))
        .into_any_element()
}
