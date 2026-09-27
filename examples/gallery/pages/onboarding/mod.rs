use gpui::{AnyElement, App, Window, div, prelude::*};

mod flows;

use super::Page;
use crate::script::Step;

pub const PAGE: Page = Page {
    number: 37,
    slug: "onboarding",
    title: "Onboarding & Help",
    summary: "A first run in steps, what is new called out, and first things to do.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("onboarding-wizard", 200.0, 167.0),
    Step::UpAt("onboarding-wizard", 200.0, 167.0),
    Step::Type("Acme"),
    Step::Key("tab"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("wizard-step-2"),
    Step::DownAt("onboarding-hotspot", 10.0, 10.0),
    Step::UpAt("onboarding-hotspot", 10.0, 10.0),
    Step::Wait(200),
    Step::Shot("hotspot-tip"),
    Step::Key("escape"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(flows::wizard(window, cx))
        .child(flows::highlights(window, cx))
        .child(flows::checklist(window, cx))
        .into_any_element()
}
