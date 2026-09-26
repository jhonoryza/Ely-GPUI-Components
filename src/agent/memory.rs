use std::rc::Rc;

use gpui::{
    App, ElementId, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::*,
};
use jiff::Timestamp;

use crate::{
    buttons::{Button, ButtonVariant, IconButton},
    data_display::{Timeline, TimelineItem, Tone},
    forms::{Enter, Input, Pick, TextInput},
    primitives::IconName,
    theme::{ActiveTheme, ControlSize, TextSize},
    typography::{RelativeTime, format::plural},
};

/// Something an agent keeps between conversations: its key and words, where it came from, and when.
#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    pub key: SharedString,
    pub text: SharedString,
    pub source: Option<SharedString>,
    pub at: Timestamp,
}

type OnAdd = Rc<dyn Fn(&str, &mut Window, &mut App)>;
type OnForget = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// What an agent keeps between conversations: a field to add a memory, then each one with where it came from, when, and a way to forget it.
#[derive(IntoElement)]
pub struct MemoryPanel {
    id: ElementId,
    field: Entity<TextInput>,
    memories: Vec<Memory>,
    on_add: Option<OnAdd>,
    on_forget: Option<OnForget>,
}

impl MemoryPanel {
    /// `field` is the owner's line for a new memory.
    pub fn new(
        id: impl Into<ElementId>,
        field: &Entity<TextInput>,
        memories: impl IntoIterator<Item = Memory>,
    ) -> Self {
        Self {
            id: id.into(),
            field: field.clone(),
            memories: memories.into_iter().collect(),
            on_add: None,
            on_forget: None,
        }
    }

    /// Gets the words to keep; the field empties after.
    pub fn on_add(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_add = Some(Rc::new(handler));
        self
    }

    /// Gets the key of the memory to forget.
    pub fn on_forget(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_forget = Some(Rc::new(handler));
        self
    }
}

/// Hands the field's words to `add` and empties it; blank words are kept out.
fn keep(field: &Entity<TextInput>, add: &OnAdd, window: &mut Window, cx: &mut App) {
    let text = field.read(cx).text().trim().to_string();
    if text.is_empty() {
        return;
    }
    log::info!("memory: kept {} characters", text.len());
    add(&text, window, cx);
    field.update(cx, |input, cx| input.set_text("", cx));
}

impl RenderOnce for MemoryPanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let ready = !self.field.read(cx).text().trim().is_empty();
        let count = self.memories.len();
        let adding = self.on_add.map(|add| {
            let (enter, press, typed, pressed) =
                (add.clone(), add, self.field.clone(), self.field.clone());
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .capture_action(move |_: &Enter, window, cx| {
                            cx.stop_propagation();
                            keep(&typed, &enter, window, cx)
                        })
                        .child(Input::new(&self.field)),
                )
                .child(
                    Button::new((self.id.clone(), "add"), "Remember")
                        .variant(ButtonVariant::Secondary)
                        .disabled(!ready)
                        .on_click(move |_, window, cx| keep(&pressed, &press, window, cx)),
                )
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .text_size(theme.text_size(TextSize::Sm))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Memory"))
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Xs))
                            .text_color(colors.fg_subtle)
                            .child(plural(count as u64, "memory", "memories")),
                    ),
            )
            .children(adding)
            .when(count == 0, |panel| {
                panel.child(div().text_color(colors.fg_subtle).child("Nothing kept yet"))
            })
            .children(self.memories.into_iter().map(|memory| {
                let forget = self.on_forget.clone().map(|forget| {
                    let key = memory.key.clone();
                    IconButton::new(
                        (self.id.clone(), format!("forget-{}", memory.key)),
                        IconName::Trash2,
                    )
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
                    .tooltip("Forget")
                    .on_click(move |_, window, cx| {
                        log::info!("memory: forget {key}");
                        forget(&key, window, cx)
                    })
                });
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .py_2()
                    .border_t_1()
                    .border_color(colors.border)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_color(colors.fg).child(memory.text))
                            .child(
                                div()
                                    .mt_0p5()
                                    .flex()
                                    .flex_wrap()
                                    .gap_x_1()
                                    .text_size(theme.text_size(TextSize::Xs))
                                    .text_color(colors.fg_subtle)
                                    .children(memory.source.map(|source| format!("{source} ·")))
                                    .child(RelativeTime::new(
                                        (self.id.clone(), format!("when-{}", memory.key)),
                                        memory.at,
                                    )),
                            ),
                    )
                    .children(forget.map(|button| div().flex_none().child(button)))
            }))
    }
}

/// A point an agent can go back to: its key, what it had done, when, and how many files it had touched.
#[derive(Clone, Debug, PartialEq)]
pub struct Checkpoint {
    pub key: SharedString,
    pub label: SharedString,
    pub at: Timestamp,
    pub files: usize,
}

/// Points an agent can go back to, newest first: each with what it had done, when and how many files it had touched; the one it stands on marked, Rewind on the rest.
#[derive(IntoElement)]
pub struct CheckpointList {
    id: ElementId,
    checkpoints: Vec<Checkpoint>,
    current: usize,
    on_rewind: Option<Pick>,
}

impl CheckpointList {
    /// `checkpoints` oldest first; `current` indexes the one the agent stands on.
    pub fn new(
        id: impl Into<ElementId>,
        checkpoints: impl IntoIterator<Item = Checkpoint>,
        current: usize,
    ) -> Self {
        let checkpoints: Vec<Checkpoint> = checkpoints.into_iter().collect();
        assert!(
            current < checkpoints.len(),
            "checkpoint {current} of {}",
            checkpoints.len()
        );
        Self {
            id: id.into(),
            checkpoints,
            current,
            on_rewind: None,
        }
    }

    /// Gets the index of the checkpoint to go back to.
    pub fn on_rewind(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_rewind = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for CheckpointList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let current = self.current;
        let items: Vec<(usize, Checkpoint)> =
            self.checkpoints.into_iter().enumerate().rev().collect();
        items
            .into_iter()
            .fold(Timeline::new(), |timeline, (ix, point)| {
                let rewind = self
                    .on_rewind
                    .clone()
                    .filter(|_| ix != current)
                    .map(|rewind| {
                        Button::new((self.id.clone(), format!("rewind-{ix}")), "Rewind to here")
                            .variant(ButtonVariant::Ghost)
                            .size(ControlSize::Sm)
                            .icon(IconName::RotateCcw)
                            .on_click(move |_, window, cx| {
                                log::info!("checkpoints: rewind to {ix}");
                                rewind(ix, window, cx)
                            })
                    });
                let item = TimelineItem::new(point.label)
                    .time(RelativeTime::new(
                        (self.id.clone(), format!("when-{ix}")),
                        point.at,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_x_2()
                            .child(plural(point.files as u64, "file", "files"))
                            .when(ix == current, |meta| {
                                meta.child(div().text_color(colors.accent).child("Here now"))
                            })
                            .children(rewind),
                    );
                timeline.item(if ix == current {
                    item.tone(Tone::Accent)
                } else {
                    item
                })
            })
    }
}
