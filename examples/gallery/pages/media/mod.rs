mod images;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 26,
    slug: "media",
    title: "Media",
    summary: "Pictures, video and sound: looking closely, cropping and marking up, playing, and the controls and devices around them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("media-viewer", 320.0, 240.0),
    Step::UpAt("media-viewer", 320.0, 240.0),
    Step::Key("="),
    Step::Key("="),
    Step::Wait(300),
    Step::Shot("zoomed"),
    Step::Key("r"),
    Step::Wait(500),
    Step::Shot("turned"),
    Step::DownAt("media-thumbs", 237.0, 50.0),
    Step::UpAt("media-thumbs", 237.0, 50.0),
    Step::Wait(500),
    Step::Shot("lightbox"),
    Step::Key("escape"),
    Step::Wait(300),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(images::viewer(cx))
        .child(images::thumbnails(window, cx))
        .into_any_element()
}
