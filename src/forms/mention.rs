use std::{ops::Range, rc::Rc};

use gpui::{
    App, Div, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Stateful, Styled, Window, div, prelude::*,
};

use super::{
    Highlight, Input, TextInput,
    options::{Choice, Pick, Popup},
    text::{Down, Enter, Up},
};
use crate::theme::ActiveTheme;

const SHOWN: usize = 6;

/// The trigger being typed: its byte offset and char, ending at `caret`.
pub(crate) fn active_trigger(text: &str, caret: usize, triggers: &[char]) -> Option<(usize, char)> {
    let before = &text[..caret];
    let (ix, ch) = before
        .char_indices()
        .rev()
        .take_while(|(_, ch)| !ch.is_whitespace())
        .find(|(_, ch)| triggers.contains(ch))?;
    let opens = before[..ix]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace);
    let word = before[ix + ch.len_utf8()..]
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '_');
    (opens && word).then_some((ix, ch))
}

/// Replaces the trigger and its query, `at..caret`, with `text`, as one undo step.
pub(crate) fn replace_trigger(
    field: &Entity<TextInput>,
    at: usize,
    caret: usize,
    text: &str,
    cx: &mut App,
) {
    field.update(cx, |input, cx| {
        input.select(at..caret, cx);
        input.insert(text, cx);
    });
}

/// The dismissed trigger, while it is still the one being typed.
fn dismissal(dismissed: Option<usize>, active: Option<usize>) -> Option<usize> {
    dismissed.filter(|at| active == Some(*at))
}

/// Spans of `@name` and `#tag` for a highlighter.
pub(crate) fn mention_spans(text: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut previous_space = true;
    let mut chars = text.char_indices().peekable();
    while let Some((ix, ch)) = chars.next() {
        if previous_space && (ch == '@' || ch == '#') {
            let mut end = ix + 1;
            while let Some(&(at, next)) = chars.peek() {
                if !(next.is_alphanumeric() || next == '_') {
                    break;
                }
                end = at + next.len_utf8();
                chars.next();
            }
            if end > ix + 1 {
                spans.push(ix..end);
            }
            previous_space = false;
            continue;
        }
        previous_space = ch.is_whitespace();
    }
    spans
}

/// Colors mentions and tags in the link tone over a faint wash.
pub fn mention_highlights(text: &str, cx: &App) -> Vec<(Range<usize>, Highlight)> {
    let link = cx.theme().colors.link;
    mention_spans(text)
        .into_iter()
        .map(|range| {
            (
                range,
                Highlight {
                    background: Some(link.opacity(0.08)),
                    ..Highlight::new(link)
                },
            )
        })
        .collect()
}

#[derive(Default)]
struct Picking {
    highlighted: usize,
    dismissed: Option<usize>,
}

/// Rows offered at a trigger in a field's text, and what a pick does: arrows choose, Enter or a press picks, Escape dismisses until the trigger ends.
pub(crate) struct Suggestions {
    pub id: ElementId,
    pub state: Entity<TextInput>,
    /// Where the trigger being typed starts.
    pub trigger: Option<usize>,
    pub rows: Vec<Choice>,
    pub pick: Pick,
}

impl Suggestions {
    /// Wraps the element that shows the field, with the rows under the trigger while it is typed.
    pub fn wrap(self, field: impl IntoElement, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let picking = window.use_keyed_state(self.id.clone(), cx, |_, _| Picking::default());
        let input = self.state.read(cx);
        let focused = input.focus().is_focused(window);
        let anchor = self.trigger.and_then(|ix| input.bounds_for(ix));
        let dismissed = dismissal(picking.read(cx).dismissed, self.trigger);
        if dismissed != picking.read(cx).dismissed {
            picking.update(cx, |picking, _| picking.dismissed = dismissed);
        }
        let open = focused
            && !self.rows.is_empty()
            && self.trigger.is_some_and(|ix| dismissed != Some(ix));
        let count = self.rows.len();
        let highlighted = picking.read(cx).highlighted.min(count.saturating_sub(1));
        log::debug!(
            "suggestions {:?}: open {open}, {count} rows, anchor {anchor:?}",
            self.id
        );
        let (up, down, escape) = (picking.clone(), picking.clone(), picking);
        let (enter, trigger) = (self.pick.clone(), self.trigger);
        div()
            .id(self.id)
            .relative()
            .w_full()
            .capture_action(move |_: &Up, _, cx| {
                if open {
                    cx.stop_propagation();
                    up.update(cx, |picking, cx| {
                        picking.highlighted = (picking.highlighted + count - 1) % count;
                        cx.notify();
                    });
                }
            })
            .capture_action(move |_: &Down, _, cx| {
                if open {
                    cx.stop_propagation();
                    down.update(cx, |picking, cx| {
                        picking.highlighted = (picking.highlighted + 1) % count;
                        cx.notify();
                    });
                }
            })
            .capture_action(move |_: &Enter, window, cx| {
                if open {
                    cx.stop_propagation();
                    enter(highlighted, window, cx);
                }
            })
            .on_key_down(move |event, _, cx| {
                if open && event.keystroke.key == "escape" {
                    cx.stop_propagation();
                    escape.update(cx, |picking, cx| {
                        picking.dismissed = trigger;
                        cx.notify();
                    });
                }
            })
            .child(field)
            .when_some(anchor.filter(|_| open), |field, anchor| {
                field.child(
                    Popup {
                        id: "suggestions".into(),
                        anchor,
                        rows: &self.rows,
                        highlighted: Some(highlighted),
                        checked: None,
                        pick: self.pick,
                        dismiss: None,
                        scroll: None,
                    }
                    .render(window, cx),
                )
            })
    }
}

/// A field where `@` or `#` opens suggestions at the caret. Arrows choose; Enter picks.
#[derive(IntoElement)]
pub struct MentionInput {
    id: ElementId,
    state: Entity<TextInput>,
    triggers: Vec<(char, Vec<SharedString>)>,
}

impl MentionInput {
    /// Give the state `mention_highlights` as its highlighter.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextInput>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            triggers: Vec::new(),
        }
    }

    /// Offers `handles` after `trigger`, such as `@` for people or `#` for tags.
    pub fn trigger(
        mut self,
        trigger: char,
        handles: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        let handles: Vec<SharedString> = handles.into_iter().map(Into::into).collect();
        for handle in &handles {
            assert!(
                !handle.is_empty() && handle.chars().all(|ch| ch.is_alphanumeric() || ch == '_'),
                "mention handle {handle:?} must be one word"
            );
        }
        self.triggers.push((trigger, handles));
        self
    }
}

/// The handles after `trigger` that hold the typed query, a few at most.
pub(crate) fn handles_matching(handles: &[SharedString], query: &str) -> Vec<SharedString> {
    let query = query.to_lowercase();
    handles
        .iter()
        .filter(|name| name.to_lowercase().contains(&query))
        .take(SHOWN)
        .cloned()
        .collect()
}

impl RenderOnce for MentionInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let input = self.state.read(cx);
        let (text, caret) = (input.text().to_string(), input.cursor());
        let marks: Vec<char> = self.triggers.iter().map(|(trigger, _)| *trigger).collect();
        let active = active_trigger(&text, caret, &marks);
        let matches: Vec<SharedString> = active
            .map(|(ix, trigger)| {
                let names = &self
                    .triggers
                    .iter()
                    .find(|(mark, _)| *mark == trigger)
                    .expect("from triggers")
                    .1;
                handles_matching(names, &text[ix + trigger.len_utf8()..caret])
            })
            .unwrap_or_default();
        let rows: Vec<Choice> = matches
            .iter()
            .map(|name| {
                let label = active.map_or(name.clone(), |(_, trigger)| {
                    SharedString::from(format!("{trigger}{name}"))
                });
                Choice::new(name.clone(), label)
            })
            .collect();
        let insert: Pick = {
            let state = self.state.clone();
            Rc::new(move |pick: usize, _: &mut Window, cx: &mut App| {
                let Some((ix, trigger)) = active else {
                    return;
                };
                let name = &matches[pick];
                log::info!("mention input: picked {trigger}{name}");
                state.update(cx, |input, cx| {
                    input.select(ix..caret, cx);
                    input.insert(&format!("{trigger}{name} "), cx);
                });
            })
        };
        Suggestions {
            id: self.id,
            state: self.state.clone(),
            trigger: active.map(|(ix, _)| ix),
            rows,
            pick: insert,
        }
        .wrap(Input::new(&self.state), window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{active_trigger, dismissal, mention_spans};

    #[test]
    fn triggers_open_after_space_and_close_on_space() {
        let marks = ['@', '#'];
        assert_eq!(active_trigger("hi @ad", 6, &marks), Some((3, '@')));
        assert_eq!(active_trigger("@", 1, &marks), Some((0, '@')));
        assert_eq!(active_trigger("mail@ad", 7, &marks), None);
        assert_eq!(active_trigger("hi @ada lovelace", 16, &marks), None);
        assert_eq!(active_trigger("go #rel", 7, &marks), Some((3, '#')));
    }

    #[test]
    fn a_dismissal_ends_with_its_trigger() {
        assert_eq!(dismissal(Some(0), Some(0)), Some(0));
        assert_eq!(dismissal(Some(0), None), None);
        assert_eq!(dismissal(Some(0), Some(4)), None);
    }

    #[test]
    fn spans_cover_whole_names() {
        assert_eq!(
            mention_spans("ping @ada and #release now"),
            vec![5..9, 14..22]
        );
        assert_eq!(
            mention_spans("a@b # c"),
            Vec::<std::ops::Range<usize>>::new()
        );
    }
}
