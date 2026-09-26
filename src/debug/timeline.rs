use std::{cell::Cell, rc::Rc};

use gpui::{
    App, Bounds, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, RenderOnce, ScrollWheelEvent, SharedString, StatefulInteractiveElement,
    Styled, Window, canvas, div, prelude::*, relative,
};

use super::OnIndex;
use crate::{
    charts::nice_step,
    theme::{ActiveTheme, ControlSize, Radius, TextSize},
    typography::{Ellipsis, LEADING},
};

/// A stretch of work on a track, in milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub track: usize,
    pub name: SharedString,
    pub start: f64,
    pub end: f64,
}

/// Wheel pixels that zoom by a factor of e.
const ZOOM: f64 = 400.0;

/// Ticks along the ruler, about.
const TICKS: usize = 6;

/// `range` scaled by `factor` around the point at `at` across it, from 0 to 1; above 1 widens, below narrows.
pub fn zoomed(range: (f64, f64), at: f64, factor: f64) -> (f64, f64) {
    assert!(
        factor > 0.0 && range.1 > range.0,
        "a range zooms by a positive factor"
    );
    let pivot = range.0 + (range.1 - range.0) * at.clamp(0.0, 1.0);
    (
        pivot - (pivot - range.0) * factor,
        pivot + (range.1 - pivot) * factor,
    )
}

/// A steady color for a span's name.
fn tint(name: &str, palette: &[Hsla; 8]) -> Hsla {
    let hash = name.bytes().fold(0u32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(byte as u32)
    });
    palette[hash as usize % palette.len()]
}

type OnRange = Rc<dyn Fn((f64, f64), &mut Window, &mut App)>;

/// Work over time on tracks, such as threads: each span a bar from its start to its end under a time axis. The wheel zooms around the pointer and a sideways wheel pans; a press picks a span.
#[derive(IntoElement)]
pub struct TimelineProfiler {
    id: ElementId,
    tracks: Vec<SharedString>,
    spans: Rc<Vec<Span>>,
    range: (f64, f64),
    selected: Option<usize>,
    on_range: Option<OnRange>,
    on_select: Option<OnIndex>,
}

impl TimelineProfiler {
    /// `range` is the stretch of time in view, in milliseconds.
    pub fn new(
        id: impl Into<ElementId>,
        tracks: impl IntoIterator<Item = impl Into<SharedString>>,
        spans: impl Into<Rc<Vec<Span>>>,
        range: (f64, f64),
    ) -> Self {
        Self {
            id: id.into(),
            tracks: tracks.into_iter().map(Into::into).collect(),
            spans: spans.into(),
            range,
            selected: None,
            on_range: None,
            on_select: None,
        }
    }

    pub fn selected(mut self, span: usize) -> Self {
        self.selected = Some(span);
        self
    }

    pub fn on_range(
        mut self,
        handler: impl Fn((f64, f64), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_range = Some(Rc::new(handler));
        self
    }

    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TimelineProfiler {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let line = theme
            .control_height(ControlSize::Md)
            .to_pixels(window.rem_size());
        let (from, to) = self.range;
        let length = to - from;
        let at = move |time: f64| ((time - from) / length) as f32;
        let step = nice_step(length, TICKS);
        let first = (from / step).ceil() as i64;
        let ticks: Vec<f64> = (first..)
            .map(|n| n as f64 * step)
            .take_while(|time| *time < to)
            .collect();
        let lane = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let (measure, wheel_lane) = (lane.clone(), lane);
        let on_range = self.on_range.clone();
        let label = theme.label_width() * 0.6;
        let row = theme.control_height(ControlSize::Md);
        let selected = self.selected.and_then(|ix| self.spans.get(ix).cloned());
        div()
            .flex()
            .flex_col()
            .gap_2()
            .text_size(theme.text_size(TextSize::Xs))
            .child(
                div().flex().child(div().flex_none().w(label)).child(
                    div()
                        .relative()
                        .flex_1()
                        .h(theme.text_size(TextSize::Xs) * LEADING)
                        .line_height(relative(LEADING))
                        .overflow_hidden()
                        .border_b_1()
                        .border_color(colors.border)
                        .children(ticks.iter().map(|time| {
                            div()
                                .absolute()
                                .left(relative(at(*time)))
                                .top_0()
                                .pl_1()
                                .border_l_1()
                                .border_color(colors.border)
                                .text_color(colors.fg_subtle)
                                .child(format!("{time:.0} ms"))
                        })),
                ),
            )
            .child(
                div()
                    .id(self.id.clone())
                    .flex()
                    .flex_col()
                    .gap_px()
                    .on_scroll_wheel(move |event: &ScrollWheelEvent, window, cx| {
                        let Some(on_range) = &on_range else {
                            return;
                        };
                        let bounds = wheel_lane.get();
                        let delta = event.delta.pixel_delta(line);
                        let (x, y) = (f64::from(f32::from(delta.x)), f64::from(f32::from(delta.y)));
                        cx.stop_propagation();
                        let width = f64::from(f32::from(bounds.size.width)).max(1.0);
                        if x.abs() > y.abs() {
                            let shift = -x / width * length;
                            on_range((from + shift, to + shift), window, cx);
                        } else {
                            let pointer =
                                f64::from(f32::from(event.position.x - bounds.origin.x)) / width;
                            on_range(zoomed((from, to), pointer, (-y / ZOOM).exp()), window, cx);
                        }
                    })
                    .children(self.tracks.iter().enumerate().map(|(track, name)| {
                        let bars = self
                            .spans
                            .iter()
                            .enumerate()
                            .filter(|(_, span)| {
                                span.track == track && span.end > from && span.start < to
                            })
                            .map(|(ix, span)| {
                                let (left, right) =
                                    (at(span.start).max(0.0), at(span.end).min(1.0));
                                let lit = self.selected == Some(ix);
                                let pick = self.on_select.clone();
                                let color = tint(&span.name, &colors.chart);
                                div()
                                    .id((self.id.clone(), format!("span-{ix}")))
                                    .absolute()
                                    .top_0p5()
                                    .bottom_0p5()
                                    .left(relative(left))
                                    .w(relative(right - left))
                                    .px_1()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .rounded(theme.radius(Radius::Sm))
                                    .bg(if lit { color } else { color.opacity(0.55) })
                                    .border_1()
                                    .border_color(if lit {
                                        colors.focus
                                    } else {
                                        color.opacity(0.0)
                                    })
                                    .text_color(colors.fg)
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                                        window.prevent_default()
                                    })
                                    .when_some(pick, |bar, pick| {
                                        bar.on_click(move |_, window, cx| pick(ix, window, cx))
                                    })
                                    .child(Ellipsis::new(span.name.clone()))
                            });
                        div()
                            .flex()
                            .h(row)
                            .child(
                                div()
                                    .flex_none()
                                    .w(label)
                                    .pr_2()
                                    .text_color(colors.fg_muted)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(name.clone()),
                            )
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .h_full()
                                    .overflow_hidden()
                                    .bg(colors.hover.opacity(0.5))
                                    .when(track == 0, |lane| {
                                        let measure = measure.clone();
                                        lane.child(
                                            canvas(
                                                move |bounds, _, _| measure.set(bounds),
                                                |_, _, _, _| {},
                                            )
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .size_full(),
                                        )
                                    })
                                    .children(bars),
                            )
                    })),
            )
            .children(selected.map(|span| {
                div().text_color(colors.fg_muted).child(format!(
                    "{} · {:.1} ms · from {:.1} to {:.1} ms",
                    span.name,
                    span.end - span.start,
                    span.start,
                    span.end
                ))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_point_under_the_pointer() {
        assert_eq!(zoomed((0.0, 100.0), 0.5, 0.5), (25.0, 75.0));
        assert_eq!(
            zoomed((0.0, 100.0), 0.0, 0.5),
            (0.0, 50.0),
            "zooming at the start keeps the start"
        );
        assert_eq!(zoomed((10.0, 20.0), 1.0, 2.0), (0.0, 20.0));
    }
}
