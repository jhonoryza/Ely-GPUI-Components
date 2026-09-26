use std::rc::Rc;

use gpui::{
    App, Context, ElementId, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, Styled, Subscription, Window, div, prelude::*,
};

use super::{InputEvent, TextInput, text::Backspace};
use crate::{
    data_display::{Tag, Tone},
    theme::{ActiveTheme, ControlSize, Radius, TextSize},
};

type OnChange = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

struct Tags {
    input: Entity<TextInput>,
    tags: Vec<SharedString>,
    on_change: Option<OnChange>,
    _events: Subscription,
}

impl Tags {
    /// Adds the field's text as a tag, unless blank or already there.
    fn take(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let typed = self
            .input
            .read(cx)
            .text()
            .trim()
            .trim_end_matches(',')
            .trim()
            .to_string();
        self.input.update(cx, |input, cx| input.set_text("", cx));
        if typed.is_empty() || self.tags.iter().any(|tag| tag.as_ref() == typed) {
            return;
        }
        let mut next = self.tags.clone();
        next.push(typed.into());
        log::info!("tag input: {} tags", next.len());
        if let Some(on_change) = self.on_change.clone() {
            on_change(next, window, cx);
        }
    }
}

/// A chip per tag, then a field. Enter or a comma adds; Backspace on empty drops the last.
#[derive(IntoElement)]
pub struct TagInput {
    id: ElementId,
    tags: Vec<SharedString>,
    placeholder: SharedString,
    check: Option<fn(&str) -> bool>,
    on_change: Option<OnChange>,
}

impl TagInput {
    pub fn new(
        id: impl Into<ElementId>,
        tags: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            id: id.into(),
            tags: tags.into_iter().map(Into::into).collect(),
            placeholder: "Add a tag".into(),
            check: None,
            on_change: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Marks each tag that fails `check` as danger, as text that is not an address.
    pub fn check(mut self, check: fn(&str) -> bool) -> Self {
        self.check = Some(check);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TagInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let placeholder = self.placeholder.clone();
        let state =
            window.use_keyed_state(self.id.clone(), cx, |window, cx: &mut Context<Tags>| {
                let input = cx.new(|cx| TextInput::new(window, cx).placeholder(placeholder));
                let events = cx.subscribe_in(&input, window, |tags, input, event, window, cx| {
                    let comma =
                        *event == InputEvent::Changed && input.read(cx).text().ends_with(',');
                    if comma || *event == InputEvent::Submit {
                        tags.take(window, cx);
                    }
                });
                Tags {
                    input,
                    tags: Vec::new(),
                    on_change: None,
                    _events: events,
                }
            });
        state.update(cx, |tags, _| {
            tags.tags = self.tags.clone();
            tags.on_change = self.on_change.clone();
        });
        let input = state.read(cx).input.clone();
        let focused = input.read(cx).focus().is_focused(window);
        let focus = input.read(cx).focus().clone();
        let theme = cx.theme();
        let colors = &theme.colors;
        let owner = self.id.clone();
        let chips = self.tags.iter().enumerate().map(|(ix, tag)| {
            let (tags, on_change) = (self.tags.clone(), self.on_change.clone());
            let failed = self.check.is_some_and(|check| !check(tag));
            Tag::new((owner.clone(), format!("tag-{ix}")), tag.clone())
                .when(failed, |tag| tag.tone(Tone::Danger))
                .on_remove(move |window, cx| {
                    let mut next = tags.clone();
                    next.remove(ix);
                    log::info!("tag input: removed one, {} left", next.len());
                    if let Some(on_change) = &on_change {
                        on_change(next, window, cx);
                    }
                })
        });
        let (tags, on_change, empty_check) =
            (self.tags.clone(), self.on_change.clone(), input.clone());
        div()
            .id(self.id)
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1()
            .w_full()
            .min_h(theme.control_height(ControlSize::Md))
            .px_1()
            .py_0p5()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(if focused {
                colors.focus
            } else {
                colors.border_strong
            })
            .bg(colors.surface)
            .text_size(theme.text_size(TextSize::Base))
            .text_color(colors.fg)
            .on_mouse_down(MouseButton::Left, move |_, window, _| window.focus(&focus))
            .capture_action(move |_: &Backspace, window, cx| {
                if !empty_check.read(cx).is_empty() || tags.is_empty() {
                    return;
                }
                cx.stop_propagation();
                let mut next = tags.clone();
                next.pop();
                log::info!("tag input: backspace dropped the last tag");
                if let Some(on_change) = &on_change {
                    on_change(next, window, cx);
                }
            })
            .children(chips)
            .child(
                div()
                    .flex_1()
                    .min_w(theme.pane_min() / 2.0)
                    .px_1()
                    .child(input),
            )
    }
}
