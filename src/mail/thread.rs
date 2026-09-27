use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*,
};
use jiff::{Timestamp, tz::TimeZone};
use smallvec::SmallVec;

use super::reader::{Asks, Message, open_letter};
use crate::{
    data_display::Avatar,
    primitives::FocusRing,
    theme::{ActiveTheme, AvatarSize, Radius, TextSize},
    typography::{Ellipsis, format},
};

/// A conversation: its subject over its messages, oldest first. The newest stands open; the rest fold to a line of the sender, the first words and the date, which a press, Enter or Space opens.
#[derive(IntoElement)]
pub struct MailThreadView {
    id: ElementId,
    subject: SharedString,
    messages: Vec<(Message, SmallVec<[AnyElement; 2]>)>,
    zone: Option<TimeZone>,
    asks: Asks,
}

impl MailThreadView {
    pub fn new(id: impl Into<ElementId>, subject: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            messages: Vec::new(),
            zone: None,
            asks: Asks::default(),
        }
    }

    /// The next message, and its words.
    pub fn message(mut self, message: Message, body: impl IntoElement) -> Self {
        let twice = self
            .messages
            .iter()
            .any(|(other, _)| other.key == message.key);
        assert!(!twice, "message {} twice", message.key);
        self.messages
            .push((message, SmallVec::from_iter([body.into_any_element()])));
        self
    }

    /// The zone dates read in; the system's otherwise.
    pub fn zone(mut self, zone: TimeZone) -> Self {
        self.zone = Some(zone);
        self
    }

    /// Gets the key of the message to answer.
    pub fn on_reply(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.asks.reply = Some(Rc::new(handler));
        self
    }

    /// Offered on a message that went to more than one; gets its key.
    pub fn on_reply_all(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.asks.reply_all = Some(Rc::new(handler));
        self
    }

    /// Gets the key of the message to forward.
    pub fn on_forward(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.asks.forward = Some(Rc::new(handler));
        self
    }

    /// Gets a message's key and the file's name.
    pub fn on_download(
        mut self,
        handler: impl Fn(&SharedString, &SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.asks.download = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for MailThreadView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        assert!(!self.messages.is_empty(), "a thread holds a message");
        let opened = window.use_keyed_state((self.id.clone(), "opened"), cx, |_, _| {
            Vec::<SharedString>::new()
        });
        let zone = self
            .zone
            .unwrap_or_else(|| format::system_zone("mail thread"));
        let now = Timestamp::now();
        let newest = self.messages.len() - 1;
        let open_now = opened.read(cx).clone();
        let theme = cx.theme();
        let colors = &theme.colors;
        let messages = self
            .messages
            .into_iter()
            .enumerate()
            .map(|(ix, (message, body))| {
                let open = ix == newest || open_now.contains(&message.key);
                let part = if open {
                    open_letter(&self.id, &message, body, &zone, &self.asks, cx).into_any_element()
                } else {
                    let (key, opening, id) = (message.key.clone(), opened.clone(), self.id.clone());
                    let date = format::relative(message.at, now);
                    div()
                        .id((self.id.clone(), format!("fold-{}", message.key)))
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_2()
                        .py_1p5()
                        .rounded(theme.radius(Radius::Md))
                        .border_1()
                        .border_color(gpui::transparent_black())
                        .tab_index(0)
                        .focus_ring(cx)
                        .cursor_pointer()
                        .hover(|style| style.bg(colors.hover))
                        .text_size(theme.text_size(TextSize::Sm))
                        .on_click(move |_, _, cx| {
                            log::info!("mail thread {id}: open {key}");
                            opening.update(cx, |opened, cx| {
                                opened.push(key.clone());
                                cx.notify();
                            })
                        })
                        .child(
                            div().flex_none().child(
                                Avatar::new(
                                    (self.id.clone(), format!("folded-{}", message.key)),
                                    message.from.name.clone(),
                                )
                                .size(AvatarSize::Sm),
                            ),
                        )
                        .child(
                            div()
                                .flex_none()
                                .max_w(theme.label_width())
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.fg)
                                .child(Ellipsis::new(message.from.name.clone())),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_color(colors.fg_muted)
                                .child(Ellipsis::new(message.snippet.clone())),
                        )
                        .child(
                            div()
                                .flex_none()
                                .text_size(theme.text_size(TextSize::Xs))
                                .text_color(colors.fg_subtle)
                                .child(date),
                        )
                        .into_any_element()
                };
                div()
                    .when(ix > 0, |row| {
                        row.pt_3().border_t_1().border_color(colors.border)
                    })
                    .child(part)
            });
        div()
            .debug_selector(|| "mail-thread".into())
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Xl))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.fg)
                    .child(self.subject),
            )
            .children(messages)
    }
}
