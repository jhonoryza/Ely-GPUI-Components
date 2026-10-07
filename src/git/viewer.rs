use std::{ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, FontWeight, HighlightStyle, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    StyledText, Window, div, prelude::*, relative, transparent_black, uniform_list,
};

use super::{
    badges::DiffStat,
    diff::{DiffLine, LineKind, Stretch, diff, pair_places, side_numbers},
};
use crate::{
    buttons::{Button, ButtonVariant, SegmentedControl},
    editor::{code_colors, stack},
    i18n,
    primitives::FocusRing,
    theme::{ActiveTheme, ControlSize, Palette, TextSize},
    typography::LEADING,
};

/// How a diff lays out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLayout {
    Unified,
    Split,
}

/// Unchanged lines kept around each change.
const CONTEXT: usize = 3;

/// A line's place: its stretch, then its line there.
pub type Place = (usize, usize);

/// A row as the viewer draws it.
#[derive(Clone)]
enum Shown {
    /// A hunk's header and its place among the stretches.
    Header(usize, SharedString),
    Line(Place, DiffLine),
    Pair(Option<(Place, DiffLine)>, Option<(Place, DiffLine)>),
    /// A folded stretch: its place among the stretches and how many lines it hides.
    Fold(usize, usize),
}

fn shown(stretches: &[Stretch], layout: DiffLayout, open: &[usize]) -> Vec<Shown> {
    let mut rows = Vec::new();
    for (ix, stretch) in stretches.iter().enumerate() {
        match stretch {
            Stretch::Hunk { header, lines } => {
                rows.push(Shown::Header(ix, header.clone()));
                let at = |line: usize| ((ix, line), lines[line].clone());
                match layout {
                    DiffLayout::Unified => rows.extend((0..lines.len()).map(|line| {
                        let (place, line) = at(line);
                        Shown::Line(place, line)
                    })),
                    DiffLayout::Split => rows.extend(
                        pair_places(lines)
                            .into_iter()
                            .map(|(left, right)| Shown::Pair(left.map(at), right.map(at))),
                    ),
                }
            }
            Stretch::Folded(lines) if open.contains(&ix) => {
                rows.extend(lines.iter().enumerate().map(|(at, line)| {
                    let place = (ix, at);
                    match layout {
                        DiffLayout::Unified => Shown::Line(place, line.clone()),
                        DiffLayout::Split => {
                            Shown::Pair(Some((place, line.clone())), Some((place, line.clone())))
                        }
                    }
                }))
            }
            Stretch::Folded(lines) => rows.push(Shown::Fold(ix, lines.len())),
        }
    }
    rows
}

type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnLayout = Rc<dyn Fn(DiffLayout, &mut Window, &mut App)>;
type OnLine = Rc<dyn Fn(Place, bool, &mut Window, &mut App)>;

/// One file's changes: each change in its context with old and new numbers and its changed words marked, the stretches between folded until pressed; unified or side by side. Long diffs draw only the rows in view; it fills its box.
#[derive(IntoElement)]
pub struct DiffViewer {
    id: ElementId,
    path: SharedString,
    stretches: Rc<Vec<Stretch>>,
    stat: (usize, usize),
    layout: DiffLayout,
    open: Vec<usize>,
    on_open: Option<OnIndex>,
    on_layout: Option<OnLayout>,
    headless: bool,
    selected: Rc<Vec<Place>>,
    on_line: Option<OnLine>,
    hunk_action: Option<(SharedString, OnIndex)>,
}

impl DiffViewer {
    pub fn new(
        id: impl Into<ElementId>,
        path: impl Into<SharedString>,
        old: &str,
        new: &str,
    ) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            stretches: Rc::new(diff(old, new, CONTEXT)),
            stat: super::diff::stat(old, new),
            layout: DiffLayout::Unified,
            open: Vec::new(),
            on_open: None,
            on_layout: None,
            headless: false,
            selected: Rc::default(),
            on_line: None,
            hunk_action: None,
        }
    }

    /// Stretches made elsewhere, such as from Git's own diff.
    pub fn from_stretches(
        id: impl Into<ElementId>,
        path: impl Into<SharedString>,
        stretches: Rc<Vec<Stretch>>,
    ) -> Self {
        let stat = stretches
            .iter()
            .filter_map(|stretch| match stretch {
                Stretch::Hunk { lines, .. } => Some(lines),
                Stretch::Folded(_) => None,
            })
            .flatten()
            .fold((0, 0), |(added, removed), line| match line.kind {
                LineKind::Added => (added + 1, removed),
                LineKind::Removed => (added, removed + 1),
                LineKind::Same => (added, removed),
            });
        Self {
            stretches,
            stat,
            ..Self::new(id, path, "", "")
        }
    }

    /// Changed lines shown as picked.
    pub fn selected(mut self, places: impl IntoIterator<Item = Place>) -> Self {
        self.selected = Rc::new(places.into_iter().collect());
        self
    }

    /// Makes changed lines pressable; true when Shift was held.
    pub fn on_line(
        mut self,
        handler: impl Fn(Place, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_line = Some(Rc::new(handler));
        self
    }

    /// A button on each hunk's header, given the hunk's place.
    pub fn hunk_action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.hunk_action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// Leaves the file's header, its path, counts and layout, to the owner, as a change card draws it.
    pub fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    pub fn layout(mut self, layout: DiffLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Folded stretches shown in full, by their place among the stretches.
    pub fn open(mut self, stretches: impl IntoIterator<Item = usize>) -> Self {
        self.open = stretches.into_iter().collect();
        self
    }

    pub fn on_open(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    pub fn on_layout(
        mut self,
        handler: impl Fn(DiffLayout, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_layout = Some(Rc::new(handler));
        self
    }
}

/// The wash behind a line, and behind its changed words.
fn washes(kind: LineKind, colors: &Palette) -> (Option<Hsla>, Hsla) {
    match kind {
        LineKind::Same => (None, colors.hover),
        LineKind::Added => (
            Some(colors.success.opacity(0.08)),
            colors.success.opacity(0.25),
        ),
        LineKind::Removed => (
            Some(colors.danger.opacity(0.08)),
            colors.danger.opacity(0.25),
        ),
    }
}

/// A line's code, colored as code, its changed words washed.
fn code(line: &DiffLine, colors: &Palette, cx: &App) -> StyledText {
    let (_, word) = washes(line.kind, colors);
    let mut styles: Vec<(Range<usize>, HighlightStyle)> = code_colors(&line.text, cx);
    styles.extend(line.words.iter().map(|range| {
        (
            range.clone(),
            HighlightStyle {
                background_color: Some(word),
                ..HighlightStyle::default()
            },
        )
    }));
    styles.sort_by_key(|(range, _)| range.start);
    StyledText::new(line.text.clone()).with_highlights(stack(styles))
}

impl RenderOnce for DiffViewer {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let rows = Rc::new(shown(&self.stretches, self.layout, &self.open));
        let layout = SegmentedControl::new(
            (self.id.clone(), "layout"),
            match self.layout {
                DiffLayout::Unified => "unified",
                DiffLayout::Split => "split",
            },
        )
        .size(ControlSize::Sm)
        .segment("unified", i18n::text(cx, "git.diff.unified", &[]), None)
        .segment("split", i18n::text(cx, "git.diff.split", &[]), None);
        let layout = match self.on_layout {
            Some(on_layout) => layout.on_change(move |key, window, cx| {
                let next = if key.as_ref() == "split" {
                    DiffLayout::Split
                } else {
                    DiffLayout::Unified
                };
                on_layout(next, window, cx)
            }),
            None => layout,
        };
        let digits = self
            .stretches
            .iter()
            .flat_map(|stretch| match stretch {
                Stretch::Hunk { lines, .. } | Stretch::Folded(lines) => lines.iter(),
            })
            .flat_map(|line| [line.old, line.new])
            .flatten()
            .max()
            .unwrap_or(1)
            .to_string()
            .len()
            .max(2);
        let (id, on_open) = (self.id.clone(), self.on_open.clone());
        let (selected, on_line, hunk_action) = (
            self.selected.clone(),
            self.on_line.clone(),
            self.hunk_action.clone(),
        );
        let list = uniform_list(
            (self.id.clone(), "rows"),
            rows.len(),
            move |range, _, cx| {
                let colors = cx.theme().colors.clone();
                let number = |value: Option<usize>| {
                    div()
                        .flex_none()
                        .w(cx.theme().text_size(TextSize::Xs) * (digits as f32 * 0.62 + 0.8))
                        .text_right()
                        .text_color(colors.fg_subtle)
                        .children(value.map(|value| value.to_string()))
                };
                // Changed lines toggle when the owner picks lines.
                let pick = |element: gpui::Div,
                            own: ElementId,
                            place: Place,
                            line: &DiffLine,
                            cx: &App|
                 -> gpui::Stateful<gpui::Div> {
                    let picked = selected.contains(&place);
                    let element = element.when(picked, |element| element.bg(colors.selection));
                    match (&on_line, line.kind) {
                        (Some(on_line), LineKind::Added | LineKind::Removed) => {
                            let on_line = on_line.clone();
                            let sign = if line.kind == LineKind::Added {
                                "+"
                            } else {
                                "−"
                            };
                            element
                                .id((id.clone(), format!("line-{}-{}", place.0, place.1)))
                                .role(Role::ListItem)
                                .aria_label(SharedString::from(format!("{sign} {}", line.text)))
                                .aria_selected(picked)
                                .tab_index(0)
                                .focus_ring(cx)
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, |_, window, _| {
                                    window.prevent_default()
                                })
                                .on_click(move |event, window, cx| {
                                    on_line(place, event.modifiers().shift, window, cx)
                                })
                        }
                        _ => element.id(own),
                    }
                };
                let side = |line: Option<&(Place, DiffLine)>,
                            shown: Option<usize>,
                            at: &str,
                            cx: &App|
                 -> AnyElement {
                    let (wash, _) = line.map_or(
                        (Some(colors.hover.opacity(0.5)), colors.hover),
                        |(_, line)| washes(line.kind, &colors),
                    );
                    let own = ElementId::from(SharedString::from(at.to_string()));
                    let element = div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .gap_2()
                        .overflow_hidden()
                        .when_some(wash, |side, wash| side.bg(wash))
                        .child(number(shown))
                        .children(line.map(|(_, line)| {
                            div().whitespace_nowrap().child(code(line, &colors, cx))
                        }));
                    match line {
                        Some((place, line)) => {
                            pick(element, own, *place, line, cx).into_any_element()
                        }
                        None => element.id(own).into_any_element(),
                    }
                };
                range
                    .map(|ix| {
                        let row = div()
                            .w_full()
                            .flex()
                            .px_2()
                            .border_1()
                            .border_color(transparent_black())
                            .whitespace_nowrap()
                            .line_height(relative(LEADING));
                        match &rows[ix] {
                            Shown::Header(stretch, header) => {
                                let action = hunk_action.clone().map(|(label, run)| {
                                    let stretch = *stretch;
                                    Button::new((id.clone(), format!("hunk-{stretch}")), label)
                                        .variant(ButtonVariant::Ghost)
                                        .size(ControlSize::Sm)
                                        .on_click(move |_, window, cx| run(stretch, window, cx))
                                });
                                row.id(ix)
                                    .bg(colors.hover)
                                    .items_center()
                                    .text_color(colors.fg_muted)
                                    .child(div().flex_1().min_w_0().child(header.clone()))
                                    .children(action)
                                    .into_any_element()
                            }
                            Shown::Line(place, line) => {
                                let (wash, _) = washes(line.kind, &colors);
                                let sign = match line.kind {
                                    LineKind::Same => " ",
                                    LineKind::Added => "+",
                                    LineKind::Removed => "−",
                                };
                                let row = row.gap_2().when_some(wash, |row, wash| row.bg(wash));
                                pick(row, ElementId::from(ix), *place, line, cx)
                                    .child(number(line.old))
                                    .child(number(line.new))
                                    .child(
                                        div().flex_none().text_color(colors.fg_subtle).child(sign),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .overflow_hidden()
                                            .child(code(line, &colors, cx)),
                                    )
                                    .into_any_element()
                            }
                            Shown::Pair(left, right) => {
                                let (old, new) = side_numbers(
                                    left.as_ref().map(|(_, line)| line),
                                    right.as_ref().map(|(_, line)| line),
                                );
                                row.id(ix)
                                    .gap_1()
                                    .child(side(left.as_ref(), old, &format!("left-{ix}"), cx))
                                    .child(side(right.as_ref(), new, &format!("right-{ix}"), cx))
                                    .into_any_element()
                            }
                            Shown::Fold(stretch, count) => {
                                let (open, stretch) = (on_open.clone(), *stretch);
                                row.id((id.clone(), format!("fold-{stretch}")))
                                    .justify_center()
                                    .bg(colors.hover.opacity(0.5))
                                    .text_color(colors.fg_subtle)
                                    .when_some(open, |row, open| {
                                        row.tab_index(0)
                                            .focus_ring(cx)
                                            .cursor_pointer()
                                            .hover(|row| row.text_color(colors.fg))
                                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                                window.prevent_default()
                                            })
                                            .on_click(move |_, window, cx| {
                                                open(stretch, window, cx)
                                            })
                                    })
                                    .child(format!(
                                        "⋯ {}",
                                        i18n::text(
                                            cx,
                                            if *count == 1 {
                                                "git.diff.unchanged.one"
                                            } else {
                                                "git.diff.unchanged.other"
                                            },
                                            &[("n", &count.to_string())],
                                        )
                                    ))
                                    .into_any_element()
                            }
                        }
                    })
                    .collect()
            },
        )
        .size_full();
        div()
            .size_full()
            .flex()
            .flex_col()
            .text_size(theme.text_size(TextSize::Sm))
            .when(!self.headless, |view| {
                view.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_2()
                        .py_1p5()
                        .border_b_1()
                        .border_color(colors.border)
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.fg)
                                .child(self.path),
                        )
                        .child(DiffStat::new(self.stat.0, self.stat.1))
                        .child(layout),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .font_family(theme.mono_family.clone())
                    .text_size(theme.text_size(TextSize::Xs))
                    .child(list),
            )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn stretches_made_elsewhere_count_their_lines() {
        let stretches = std::rc::Rc::new(super::diff("a\nb\nc\n", "a\nx\ny\nc\n", 1));
        let viewer = super::DiffViewer::from_stretches("counted", "a.txt", stretches);
        assert_eq!(viewer.stat, (2, 1));
    }
}
