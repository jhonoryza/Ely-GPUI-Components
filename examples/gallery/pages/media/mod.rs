mod edit;
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
    Step::DownAt("media-cropper", 458.0, 317.0),
    Step::DragTo("media-cropper", 400.0, 280.0),
    Step::UpAt("media-cropper", 400.0, 280.0),
    Step::Wait(200),
    Step::Shot("cropped"),
    Step::DownAt("media-annotator", 500.0, 80.0),
    Step::UpAt("media-annotator", 500.0, 80.0),
    Step::Key("v"),
    Step::DownAt("media-annotator", 168.0, 310.0),
    Step::UpAt("media-annotator", 168.0, 310.0),
    Step::Wait(200),
    Step::Shot("chosen"),
    Step::Key("b"),
    Step::DownAt("media-annotator", 60.0, 90.0),
    Step::DragTo("media-annotator", 200.0, 200.0),
    Step::UpAt("media-annotator", 200.0, 200.0),
    Step::Wait(200),
    Step::Shot("marked"),
    Step::DownAt("media-upload", 388.0, 307.0),
    Step::UpAt("media-upload", 388.0, 307.0),
    Step::Wait(300),
    Step::Shot("emptied"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(images::viewer(cx))
        .child(images::thumbnails(window, cx))
        .child(edit::cropper(window, cx))
        .child(edit::upload(window, cx))
        .child(edit::annotator(window, cx))
        .into_any_element()
}
