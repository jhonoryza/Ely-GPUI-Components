use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*,
};

use crate::{
    forms::{Checkbox, OnFlag},
    motion::ProgressBar,
    theme::{ActiveTheme, TextSize},
    typography::tabular,
};

type OnItem = Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;

/// A to-do: its box and its words; done, the words go quiet and struck through.
#[derive(IntoElement)]
pub struct TodoItem {
    id: ElementId,
    label: SharedString,
    done: bool,
    on_toggle: Option<OnFlag>,
}

impl TodoItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, done: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            done,
            on_toggle: None,
        }
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TodoItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let check = Checkbox::new(self.id, self.done);
        let check = match self.on_toggle {
            Some(on_toggle) => check.on_change(move |done, window, cx| on_toggle(done, window, cx)),
            None => check,
        };
        div().flex().items_center().gap_2().child(check).child(
            div()
                .flex_1()
                .min_w_0()
                .when(self.done, |label| {
                    label.line_through().text_color(colors.fg_subtle)
                })
                .child(self.label),
        )
    }
}

/// To-dos under a title, with how many are done as a count and a bar.
#[derive(IntoElement)]
pub struct Checklist {
    id: ElementId,
    title: SharedString,
    items: Vec<(SharedString, bool)>,
    on_toggle: Option<OnItem>,
}

impl Checklist {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        items: impl IntoIterator<Item = (impl Into<SharedString>, bool)>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            items: items
                .into_iter()
                .map(|(label, done)| (label.into(), done))
                .collect(),
            on_toggle: None,
        }
    }

    /// Tells which item changed, by index, and whether it is now done.
    pub fn on_toggle(
        mut self,
        handler: impl Fn(usize, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checklist {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let done = self.items.iter().filter(|(_, done)| *done).count();
        let total = self.items.len();
        let share = if total == 0 {
            0.0
        } else {
            done as f32 / total as f32
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child(self.title))
                    .child(
                        tabular(
                            div()
                                .text_size(theme.text_size(TextSize::Sm))
                                .text_color(theme.colors.fg_muted),
                        )
                        .child(format!("{done} of {total}")),
                    ),
            )
            .child(ProgressBar::new((self.id.clone(), "done"), share))
            .children(
                self.items
                    .into_iter()
                    .enumerate()
                    .map(|(ix, (label, is_done))| {
                        let item =
                            TodoItem::new((self.id.clone(), format!("item-{ix}")), label, is_done);
                        match &self.on_toggle {
                            Some(on_toggle) => {
                                let on_toggle = on_toggle.clone();
                                item.on_toggle(move |done, window, cx| {
                                    on_toggle(ix, done, window, cx)
                                })
                            }
                            None => item,
                        }
                    }),
            )
    }
}
