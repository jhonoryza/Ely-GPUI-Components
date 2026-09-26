mod diffs;
mod people;
mod work;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 19,
    slug: "git",
    title: "Git",
    summary: "Diffs and merges, changes and commits, the commit graph, branches, blame and review.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[Step::Rest];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(diffs::viewer(window, cx))
        .child(diffs::merges(window, cx))
        .child(work::changes(window, cx))
        .child(work::commits(window, cx))
        .child(work::refs(window, cx))
        .child(people::blame(window, cx))
        .child(people::review(window, cx))
        .into_any_element()
}
