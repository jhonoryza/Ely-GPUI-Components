mod blocks;
mod cite;
mod controls;
mod talk;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 23,
    slug: "chat",
    title: "AI Chat",
    summary: "A conversation with an assistant: messages that stream in, code and media inside them, and what can be done with each.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("chat-thinking", 12.0, 10.0),
    Step::UpAt("chat-thinking", 12.0, 10.0),
    Step::Wait(300),
    Step::Shot("reasoning"),
    Step::DownAt("chat-thinking", 12.0, 10.0),
    Step::UpAt("chat-thinking", 12.0, 10.0),
    Step::DownAt("chat-edit-open", 20.0, 12.0),
    Step::UpAt("chat-edit-open", 20.0, 12.0),
    Step::Wait(300),
    Step::Shot("editing"),
    Step::Key("escape"),
    Step::Wait(200),
    Step::HoverAt("chat-cite-1", 8.0, 8.0),
    Step::Wait(700),
    Step::Shot("citation"),
    Step::DownAt("chat-sources", 20.0, 10.0),
    Step::UpAt("chat-sources", 20.0, 10.0),
    Step::Wait(300),
    Step::Shot("sources"),
    Step::DownAt("chat-sources", 20.0, 10.0),
    Step::UpAt("chat-sources", 20.0, 10.0),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(talk::conversation(window, cx))
        .child(talk::streaming(window, cx))
        .child(blocks::code(window, cx))
        .child(blocks::rich(window, cx))
        .child(blocks::media(window, cx))
        .child(controls::actions(window, cx))
        .child(controls::status(window, cx))
        .child(controls::thinking(window, cx))
        .child(cite::citations(window, cx))
        .child(cite::search(window, cx))
        .into_any_element()
}
