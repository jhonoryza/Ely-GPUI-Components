use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div,
};

use crate::{
    forms::{Choice, Select, Slider, Switch},
    theme::{ActiveTheme, Radius, TextSize},
    typography::{format, tabular},
};

type Preview = Rc<dyn Fn(&Settings, &mut Window, &mut App) -> AnyElement>;

/// A property a playground edits: a switch, a choice among names, or a number in a range, each by its name.
#[derive(Clone, Debug)]
pub enum Knob {
    Toggle(SharedString, bool),
    Choice(SharedString, Vec<SharedString>, usize),
    Number(SharedString, f64, (f64, f64), f64),
}

impl Knob {
    fn name(&self) -> &SharedString {
        match self {
            Knob::Toggle(name, _) | Knob::Choice(name, _, _) | Knob::Number(name, _, _, _) => name,
        }
    }

    /// The setting it starts on.
    fn first(&self) -> Setting {
        match self {
            Knob::Toggle(_, on) => Setting::On(*on),
            Knob::Choice(name, options, at) => Setting::Picked(
                options
                    .get(*at)
                    .unwrap_or_else(|| panic!("playground knob {name}: no option {at}"))
                    .clone(),
            ),
            Knob::Number(_, value, _, _) => Setting::Number(*value),
        }
    }
}

/// Where a knob stands.
#[derive(Clone, Debug, PartialEq)]
pub enum Setting {
    On(bool),
    Picked(SharedString),
    Number(f64),
}

/// The knobs' settings by name; asking for a name the knobs lack, or in another kind, fails.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings(Vec<(SharedString, Setting)>);

impl Settings {
    fn get(&self, name: &str) -> &Setting {
        self.0
            .iter()
            .find(|(each, _)| each.as_ref() == name)
            .map(|(_, setting)| setting)
            .unwrap_or_else(|| panic!("playground: no knob named {name}"))
    }

    pub fn on(&self, name: &str) -> bool {
        match self.get(name) {
            Setting::On(on) => *on,
            other => panic!("playground knob {name} is {other:?}, not a switch"),
        }
    }

    pub fn picked(&self, name: &str) -> &SharedString {
        match self.get(name) {
            Setting::Picked(option) => option,
            other => panic!("playground knob {name} is {other:?}, not a choice"),
        }
    }

    pub fn number(&self, name: &str) -> f64 {
        match self.get(name) {
            Setting::Number(value) => *value,
            other => panic!("playground knob {name} is {other:?}, not a number"),
        }
    }

    fn set(&mut self, name: &str, setting: Setting) {
        let slot = self
            .0
            .iter_mut()
            .find(|(each, _)| each.as_ref() == name)
            .unwrap_or_else(|| panic!("playground: no knob named {name}"));
        log::info!("playground: {name} is {setting:?}");
        slot.1 = setting;
    }
}

/// A component beside the knobs that shape it: each knob edits one property as the preview redraws with the settings. Narrow, the knobs drop below.
#[derive(IntoElement)]
pub struct Playground {
    id: ElementId,
    knobs: Vec<Knob>,
    preview: Preview,
}

impl Playground {
    pub fn new(
        id: impl Into<ElementId>,
        knobs: impl IntoIterator<Item = Knob>,
        preview: impl Fn(&Settings, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            knobs: knobs.into_iter().collect(),
            preview: Rc::new(preview),
        }
    }
}

impl RenderOnce for Playground {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        assert!(!self.knobs.is_empty(), "playground {id:?} has no knobs");
        let settings = window.use_keyed_state((id.clone(), "settings"), cx, {
            let knobs = self.knobs.clone();
            move |_, _| {
                Settings(
                    knobs
                        .iter()
                        .map(|knob| (knob.name().clone(), knob.first()))
                        .collect(),
                )
            }
        });
        let now = settings.read(cx).clone();
        let preview = (self.preview)(&now, window, cx);
        let theme = cx.theme();
        let colors = &theme.colors;
        let rows = self.knobs.iter().enumerate().map(|(ix, knob)| {
            let (name, set) = (knob.name().clone(), settings.clone());
            let control = match knob {
                Knob::Toggle(..) => Switch::new((id.clone(), format!("knob-{ix}")), now.on(&name))
                    .on_change(move |on, _, cx| {
                        set.update(cx, |settings, cx| {
                            settings.set(&name, Setting::On(on));
                            cx.notify();
                        })
                    })
                    .into_any_element(),
                Knob::Choice(_, options, _) => Select::new(
                    (id.clone(), format!("knob-{ix}")),
                    options
                        .iter()
                        .map(|option| Choice::new(option.clone(), option.clone())),
                )
                .selected(now.picked(&name).clone())
                .on_change(move |option, _, cx| {
                    set.update(cx, |settings, cx| {
                        settings.set(&name, Setting::Picked(option.clone()));
                        cx.notify();
                    })
                })
                .into_any_element(),
                Knob::Number(_, _, (least, most), step) => div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div().flex_1().min_w_0().child(
                            Slider::new((id.clone(), format!("knob-{ix}")), now.number(&name))
                                .range(*least, *most)
                                .step(*step)
                                .on_change(move |value, _, cx| {
                                    set.update(cx, |settings, cx| {
                                        settings.set(&name, Setting::Number(value));
                                        cx.notify();
                                    })
                                }),
                        ),
                    )
                    .child(
                        tabular(div())
                            .flex_none()
                            .text_color(colors.fg_muted)
                            .child(format::number(
                                now.number(knob.name()),
                                format::decimals(*step),
                                format::Separators::EN,
                            )),
                    )
                    .into_any_element(),
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(theme.text_size(TextSize::Xs))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.fg_muted)
                        .child(knob.name().clone()),
                )
                .child(control)
        });
        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(
                div()
                    .flex_1()
                    .min_w(theme.label_width())
                    .flex()
                    .items_center()
                    .justify_center()
                    .p_6()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.sunken)
                    .child(preview),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(theme.label_width())
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(rows),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{Knob, Setting, Settings};

    #[test]
    fn settings_answer_by_name_and_kind() {
        let knobs = [
            Knob::Toggle("disabled".into(), true),
            Knob::Choice("size".into(), vec!["Sm".into(), "Md".into()], 1),
            Knob::Number("share".into(), 0.4, (0.0, 1.0), 0.1),
        ];
        let mut settings = Settings(
            knobs
                .iter()
                .map(|knob| (knob.name().clone(), knob.first()))
                .collect(),
        );
        assert!(settings.on("disabled"));
        assert_eq!(settings.picked("size").as_ref(), "Md");
        settings.set("share", Setting::Number(0.7));
        assert_eq!(settings.number("share"), 0.7);
    }

    #[test]
    #[should_panic(expected = "playground knob size is Picked")]
    fn a_setting_asked_in_another_kind_fails() {
        let settings = Settings(vec![("size".into(), Setting::Picked("Md".into()))]);
        settings.on("size");
    }
}
