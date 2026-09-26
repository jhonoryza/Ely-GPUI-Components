mod blocks;
mod live;
mod panels;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 18,
    slug: "terminal",
    title: "Terminal",
    summary: "A real shell drawn cell by cell, with tabs, splits, find and links; command blocks and history; colored output, logs and processes.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("terminal", 200.0, 150.0),
    Step::UpAt("terminal", 200.0, 150.0),
    Step::Wait(400),
    Step::Type("ls -G src"),
    Step::Key("enter"),
    Step::Wait(800),
    Step::Shot("shell"),
    Step::Key("ctrl-l"),
    Step::DownAt("terminal-tabs", 155.0, 16.0),
    Step::UpAt("terminal-tabs", 155.0, 16.0),
    Step::Wait(400),
    Step::Shot("find"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(live::live(window, cx))
        .child(blocks::blocks(window, cx))
        .child(blocks::ansi(cx))
        .child(panels::logs(window, cx))
        .child(panels::process_list(window, cx))
        .into_any_element()
}
