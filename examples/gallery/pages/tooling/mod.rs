use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    motion::ProgressRing,
    theme::ControlSize,
    tooling::{Knob, Playground},
};
use gpui::{AnyElement, App, IntoElement, ParentElement, Styled, Window, div, px};

use super::Page;
use crate::probe::probe;
use crate::script::Step;
use crate::ui::section;

pub const PAGE: Page = Page {
    number: 43,
    slug: "tooling",
    title: "Library Tooling",
    summary: "Tools for building with the library: this gallery, and a playground that edits a component live.",
    render,
    script: &[
        Step::DownAt("button-playground", 472.0, 207.0),
        Step::UpAt("button-playground", 472.0, 207.0),
        Step::Shot("playground-loading"),
    ],
};

fn render(_: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(section(
            "ComponentGallery / Storybook",
            "This app is the gallery: a page per chapter of the library, light and dark, each with scripted states. The --page flag opens one page, and --capture photographs every page and state.",
            cx,
        ))
        .child(
            section(
                "PlaygroundPanel",
                "A component beside the knobs that shape it: a switch, a choice or a number edits one property, and the preview redraws.",
                cx,
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(probe(
                        "button-playground",
                        div().w(px(720.0)).child(Playground::new(
                        "button-playground",
                        [
                            Knob::Choice(
                                "variant".into(),
                                ["Primary", "Secondary", "Outline", "Ghost", "Danger"]
                                    .map(Into::into)
                                    .into(),
                                0,
                            ),
                            Knob::Choice("size".into(), ["Sm", "Md", "Lg"].map(Into::into).into(), 1),
                            Knob::Toggle("disabled".into(), false),
                            Knob::Toggle("loading".into(), false),
                        ],
                        |settings, _, _| {
                            let variant = match settings.picked("variant").as_ref() {
                                "Primary" => ButtonVariant::Primary,
                                "Secondary" => ButtonVariant::Secondary,
                                "Outline" => ButtonVariant::Outline,
                                "Ghost" => ButtonVariant::Ghost,
                                "Danger" => ButtonVariant::Danger,
                                other => panic!("gallery playground: no variant {other}"),
                            };
                            let size = match settings.picked("size").as_ref() {
                                "Sm" => ControlSize::Sm,
                                "Md" => ControlSize::Md,
                                "Lg" => ControlSize::Lg,
                                other => panic!("gallery playground: no size {other}"),
                            };
                            Button::new("played", "Save changes")
                                .variant(variant)
                                .size(size)
                                .disabled(settings.on("disabled"))
                                .loading(settings.on("loading"))
                                .into_any_element()
                        },
                    ))))
                    .child(div().w(px(720.0)).child(Playground::new(
                        "ring-playground",
                        [
                            Knob::Number("value".into(), 0.62, (0.0, 1.0), 0.01),
                            Knob::Toggle("percent".into(), true),
                        ],
                        |settings, _, _| {
                            let ring = ProgressRing::new("played-ring", settings.number("value") as f32);
                            match settings.on("percent") {
                                true => ring.percent(),
                                false => ring,
                            }
                            .into_any_element()
                        },
                    ))),
            ),
        )
        .into_any_element()
}
