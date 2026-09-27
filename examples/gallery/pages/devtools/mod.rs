use gpui::{AnyElement, App, Window, div, prelude::*};

mod data;
mod db;

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 33,
    slug: "devtools",
    title: "DB & Dev Tools",
    summary: "Connections, schemas and queries, data to read, requests to send, and the machines that run them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(db::connections(window, cx))
        .child(db::schema(window, cx))
        .child(data::queries(window, cx))
        .child(db::structure(window, cx))
        .child(data::redis(window, cx))
        .into_any_element()
}
