use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Div, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Point, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::*, relative,
};

use crate::{
    buttons::{ButtonVariant, IconButton},
    documents::source,
    forms::{Pick, Run},
    motion::{self, Spinner},
    primitives::{FocusRing, Icon, IconName, Image},
    theme::{ActiveTheme, AvatarSize, ControlSize, IconSize, Radius, TextSize},
    typography::Ellipsis,
};

/// Width over height of a frame from the host.
const FRAME: f32 = 16.0 / 10.0;

/// A box that keeps a frame's shape at the width it is given.
fn framed(cx: &App) -> Div {
    let mut frame = div()
        .relative()
        .w_full()
        .overflow_hidden()
        .bg(cx.theme().colors.sunken);
    frame.style().aspect_ratio = Some(FRAME);
    frame
}

/// The bar over a page: a lock, the address, a spinner while it loads, and the owner's actions.
fn address_bar(
    id: &ElementId,
    url: SharedString,
    loading: bool,
    actions: Option<AnyElement>,
    cx: &App,
) -> Div {
    let theme = cx.theme();
    let colors = &theme.colors;
    div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1p5()
        .border_b_1()
        .border_color(colors.border)
        .text_size(theme.text_size(TextSize::Xs))
        .text_color(colors.fg_muted)
        .child(
            Icon::new(IconName::Lock)
                .size(IconSize::Xs)
                .color(colors.fg_subtle),
        )
        .child(div().flex_1().min_w_0().child(Ellipsis::new(url)))
        .when(loading, |bar| {
            bar.child(Spinner::new((id.clone(), "loading")).size(IconSize::Xs))
        })
        .children(actions)
}

/// The card a page sits in.
fn card(cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.surface)
}

/// A browser an agent drives, as the host shows it: the address, a spinner while a page loads, the frame shown, and the frames so far along the foot; a press on one shows it.
#[derive(IntoElement)]
pub struct BrowserPreview {
    id: ElementId,
    url: SharedString,
    frames: Vec<SharedString>,
    shown: usize,
    loading: bool,
    on_show: Option<Pick>,
}

impl BrowserPreview {
    /// `frames` are the host's screenshots, oldest first, as paths or addresses; `shown` indexes them.
    pub fn new(
        id: impl Into<ElementId>,
        url: impl Into<SharedString>,
        frames: impl IntoIterator<Item = impl Into<SharedString>>,
        shown: usize,
    ) -> Self {
        let frames: Vec<SharedString> = frames.into_iter().map(Into::into).collect();
        assert!(shown < frames.len(), "frame {shown} of {}", frames.len());
        Self {
            id: id.into(),
            url: url.into(),
            frames,
            shown,
            loading: false,
            on_show: None,
        }
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Gets the frame pressed along the foot.
    pub fn on_show(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_show = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for BrowserPreview {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let thumb = theme.avatar_size(AvatarSize::Lg);
        let shown = self.shown;
        let strip = (self.frames.len() > 1).then(|| {
            div()
                .id((self.id.clone(), "strip"))
                .overflow_x_scroll()
                .flex()
                .gap_1p5()
                .p_2()
                .border_t_1()
                .border_color(colors.border)
                .children(self.frames.iter().enumerate().map(|(ix, frame)| {
                    let show = self.on_show.clone();
                    let mut tile = div()
                        .id((self.id.clone(), format!("frame-{ix}")))
                        .relative()
                        .flex_none()
                        .h(thumb)
                        .rounded(theme.radius(Radius::Sm))
                        .border_1()
                        .border_color(if ix == shown {
                            colors.focus
                        } else {
                            colors.border
                        })
                        .child(
                            Image::new((self.id.clone(), format!("thumb-{ix}")), source(frame))
                                .rounded(theme.radius(Radius::Sm))
                                .size_full(),
                        );
                    tile.style().aspect_ratio = Some(FRAME);
                    tile.when_some(show, |tile, show| {
                        tile.tab_index(0)
                            .focus_ring(cx)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.prevent_default()
                            })
                            .on_click(move |_, window, cx| {
                                log::info!("browser preview: frame {ix}");
                                show(ix, window, cx)
                            })
                    })
                }))
        });
        card(cx)
            .child(address_bar(&self.id, self.url, self.loading, None, cx))
            .child(
                framed(cx)
                    .when(strip.is_none(), |frame| {
                        frame.rounded_b(theme.radius(Radius::Lg))
                    })
                    .child({
                        let image =
                            Image::new((self.id.clone(), "frame"), source(&self.frames[shown]))
                                .size_full();
                        if strip.is_none() {
                            image.rounded_b(theme.radius(Radius::Lg))
                        } else {
                            image
                        }
                    }),
            )
            .children(strip)
    }
}

/// A screen an agent works on, as the host shows it: the frame, where the agent points, and what it does there; each new point rings once.
#[derive(IntoElement)]
pub struct ComputerUseViewer {
    id: ElementId,
    frame: SharedString,
    pointer: Option<Point<f32>>,
    action: Option<SharedString>,
}

impl ComputerUseViewer {
    /// `frame` is the host's latest screenshot, a path or an address.
    pub fn new(id: impl Into<ElementId>, frame: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            frame: frame.into(),
            pointer: None,
            action: None,
        }
    }

    /// Where the agent points, each side a share of the frame from its top left.
    pub fn pointer(mut self, at: Point<f32>) -> Self {
        assert!(
            (0.0..=1.0).contains(&at.x) && (0.0..=1.0).contains(&at.y),
            "a pointer inside the frame, not {at:?}"
        );
        self.pointer = Some(at);
        self
    }

    /// What the agent does there, such as "click Save".
    pub fn action(mut self, text: impl Into<SharedString>) -> Self {
        self.action = Some(text.into());
        self
    }
}

impl RenderOnce for ComputerUseViewer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let moves = motion::changes(
            (self.id.clone(), "pointer"),
            self.pointer.map(|at| (at.x.to_bits(), at.y.to_bits())),
            window,
            cx,
        );
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let ring = theme.icon_size(IconSize::Lg);
        let pointer = self.pointer.map(|at| {
            let mark = div()
                .flex_none()
                .size(ring)
                .rounded_full()
                .border_2()
                .border_color(colors.focus)
                .bg(colors.focus.opacity(0.16));
            let mark = if moves == 0 {
                mark.into_any_element()
            } else {
                mark.with_animation(
                    (self.id.clone(), format!("ring-{moves}")),
                    Animation::new(motion::duration(motion::SLOW, cx))
                        .with_easing(motion::ease_out_cubic),
                    move |mark, t| mark.bg(colors.focus.opacity(0.16 + 0.24 * (1.0 - t))),
                )
                .into_any_element()
            };
            div()
                .absolute()
                .left(relative(at.x))
                .top(relative(at.y))
                .size_0()
                .flex()
                .items_center()
                .justify_center()
                .child(mark)
        });
        card(cx)
            .child(
                framed(cx)
                    .rounded_t(theme.radius(Radius::Lg))
                    .when(self.action.is_none(), |frame| {
                        frame.rounded_b(theme.radius(Radius::Lg))
                    })
                    .child({
                        let image = Image::new((self.id.clone(), "frame"), source(&self.frame))
                            .rounded_t(theme.radius(Radius::Lg))
                            .size_full();
                        if self.action.is_none() {
                            image.rounded_b(theme.radius(Radius::Lg))
                        } else {
                            image
                        }
                    })
                    .children(pointer),
            )
            .children(self.action.map(|action| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .border_t_1()
                    .border_color(colors.border)
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(colors.fg_muted)
                    .child(
                        Icon::new(IconName::MousePointer2)
                            .size(IconSize::Sm)
                            .color(colors.fg_subtle),
                    )
                    .child(div().flex_1().min_w_0().child(Ellipsis::new(action)))
            }))
    }
}

/// A page as it renders while the agent builds it: its address, a spinner while it updates, reload, and the host's element that draws it.
#[derive(IntoElement)]
pub struct LivePreview {
    id: ElementId,
    url: SharedString,
    updating: bool,
    content: AnyElement,
    on_reload: Option<Run>,
}

impl LivePreview {
    pub fn new(
        id: impl Into<ElementId>,
        url: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            updating: false,
            content: content.into_any_element(),
            on_reload: None,
        }
    }

    pub fn updating(mut self, updating: bool) -> Self {
        self.updating = updating;
        self
    }

    pub fn on_reload(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reload = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for LivePreview {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let reload = self.on_reload.map(|reload| {
            IconButton::new((self.id.clone(), "reload"), IconName::RotateCw)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip("Reload")
                .on_click(move |_, window, cx| {
                    log::info!("live preview: reload");
                    reload(window, cx)
                })
                .into_any_element()
        });
        card(cx)
            .size_full()
            .child(address_bar(&self.id, self.url, self.updating, reload, cx))
            .child(
                div()
                    .id((self.id.clone(), "page"))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(self.content),
            )
    }
}
