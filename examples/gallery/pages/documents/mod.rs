mod editing;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 21,
    slug: "documents",
    title: "Documents",
    summary: "Markdown written and read, blocks like a notebook's, and a knowledge base around them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("doc-rich", 400.0, 203.0),
    Step::UpAt("doc-rich", 400.0, 203.0),
    Step::Wait(200),
    Step::Key("cmd-shift-left"),
    Step::Type("- [x] Light and dark @gr"),
    Step::Wait(300),
    Step::Shot("mention"),
    Step::Key("escape"),
    Step::Key("alt-shift-left"),
    Step::Wait(300),
    Step::Shot("floating"),
    Step::Key("cmd-k"),
    Step::Wait(300),
    Step::Shot("link"),
    Step::Key("escape"),
    Step::Wait(200),
    Step::DownAt("doc-markdown", 804.0, 16.0),
    Step::UpAt("doc-markdown", 804.0, 16.0),
    Step::Wait(300),
    Step::Shot("split"),
    Step::DownAt("doc-markdown", 696.0, 16.0),
    Step::UpAt("doc-markdown", 696.0, 16.0),
    Step::Wait(300),
    Step::DownAt("doc-markdown", 38.0, 66.0),
    Step::UpAt("doc-markdown", 38.0, 66.0),
    Step::Wait(300),
    Step::Shot("open-block"),
    Step::Key("escape"),
    Step::DownAt("doc-blocks", 740.0, 60.0),
    Step::UpAt("doc-blocks", 740.0, 60.0),
    Step::Wait(200),
    Step::Key("enter"),
    Step::Type("/"),
    Step::Wait(300),
    Step::Shot("slash"),
    Step::Key("escape"),
    Step::Key("cmd-z"),
    Step::Key("cmd-z"),
    Step::Wait(200),
    Step::HoverAt("doc-blocks", 120.0, 20.0),
    Step::Wait(300),
    Step::Shot("handle"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div().child(editing::editing(window, cx)).into_any_element()
}
