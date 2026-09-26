use std::rc::Rc;

use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, ScrollHandle,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
};

use super::ListItem;
use crate::primitives::tab_stop;

/// How a press or a key picks rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    One,
    Toggle,
    Range,
    All,
}

/// The selection after picking row `at`, a range reaching back to `anchor`; in row order.
pub(crate) fn picked(
    keys: &[SharedString],
    selected: &[SharedString],
    anchor: usize,
    at: usize,
    pick: Pick,
) -> Vec<SharedString> {
    let (low, high) = (anchor.min(at), anchor.max(at));
    keys.iter()
        .enumerate()
        .filter(|(ix, key)| match pick {
            Pick::One => *ix == at,
            Pick::Toggle => selected.contains(key) != (*ix == at),
            Pick::Range => (low..=high).contains(ix),
            Pick::All => true,
        })
        .map(|(_, key)| key.clone())
        .collect()
}

/// The row the keyboard is on and where a Shift range starts, by index and by key so they follow their rows when the owner reorders them, and the scroll that follows them; the owner's last selection, and the one the list last sent.
#[derive(Default)]
struct Cursor {
    at: usize,
    anchor: usize,
    keys: Option<(SharedString, SharedString)>,
    scroll: ScrollHandle,
    seen: Option<Vec<SharedString>>,
    sent: Option<Vec<SharedString>>,
}

type OnSelect = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
type OnActivate = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type Picker = Rc<dyn Fn(usize, Pick, &mut Window, &mut App)>;

/// Rows you select. A press picks one; with `multiple`, Cmd-press adds or drops one and Shift-press takes the range from the last pick. Up and Down move, Shift with them extends, Space toggles, Cmd-A takes all. Keys pass over disabled rows, and no pick adds one.
#[derive(IntoElement)]
pub struct SelectableList {
    id: ElementId,
    base: Div,
    rows: Vec<(SharedString, ListItem)>,
    selected: Vec<SharedString>,
    multiple: bool,
    on_change: Option<OnSelect>,
    on_activate: Option<OnActivate>,
}

impl SelectableList {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            rows: Vec::new(),
            selected: Vec::new(),
            multiple: false,
            on_change: None,
            on_activate: None,
        }
    }

    /// A row under `key`, which the selection names it by.
    pub fn row(mut self, key: impl Into<SharedString>, item: ListItem) -> Self {
        self.rows.push((key.into(), item));
        self
    }

    pub fn selected(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected = keys.into_iter().map(Into::into).collect();
        self
    }

    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    /// Enter, or a double press, on a row.
    pub fn on_activate(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    /// Gets the keys now selected, in row order.
    pub fn on_change(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl Styled for SelectableList {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for SelectableList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.rows.len();
        let focus = tab_stop((self.id.clone(), "focus").into(), true, window, cx);
        let focused = focus.is_focused(window);
        let cursor =
            window.use_keyed_state((self.id.clone(), "cursor"), cx, |_, _| Cursor::default());
        let keys: Rc<[SharedString]> = self.rows.iter().map(|(key, _)| key.clone()).collect();
        if cursor.read(cx).seen.as_ref() != Some(&self.selected) {
            cursor.update(cx, |cursor, _| {
                let echo = cursor.sent.take().as_ref() == Some(&self.selected);
                if let Some(first) = self.selected.first().filter(|_| !echo) {
                    cursor.keys = Some((first.clone(), first.clone()));
                }
                cursor.seen = Some(self.selected.clone());
            });
        }
        let (at, anchor, scroll) = {
            let cursor = cursor.read(cx);
            let place = |key: Option<&SharedString>, ix: usize| {
                key.and_then(|key| keys.iter().position(|row| row == key))
                    .unwrap_or(ix.min(count.saturating_sub(1)))
            };
            let held = cursor.keys.as_ref();
            (
                place(held.map(|keys| &keys.0), cursor.at),
                place(held.map(|keys| &keys.1), cursor.anchor),
                cursor.scroll.clone(),
            )
        };
        if (at, anchor) != (cursor.read(cx).at, cursor.read(cx).anchor) {
            cursor.update(cx, |cursor, _| (cursor.at, cursor.anchor) = (at, anchor));
        }
        let off: Rc<[bool]> = self
            .rows
            .iter()
            .map(|(_, item)| item.is_disabled())
            .collect();
        let barred: Vec<SharedString> = self
            .rows
            .iter()
            .filter(|(_, item)| item.is_disabled())
            .map(|(key, _)| key.clone())
            .collect();
        let selected = self.selected;
        let multiple = self.multiple;
        let pick: Picker = {
            let (id, keys, selected, cursor, on_change) = (
                self.id.clone(),
                keys.clone(),
                selected.clone(),
                cursor.clone(),
                self.on_change,
            );
            Rc::new(move |ix, pick, window, cx| {
                let pick = match (multiple, pick) {
                    (true, pick) => pick,
                    (false, Pick::All) => return,
                    (false, _) => Pick::One,
                };
                let next: Vec<SharedString> =
                    picked(&keys, &selected, cursor.read(cx).anchor, ix, pick)
                        .into_iter()
                        .filter(|key| !barred.contains(key) || selected.contains(key))
                        .collect();
                cursor.update(cx, |cursor, cx| {
                    cursor.at = ix;
                    if pick != Pick::Range {
                        cursor.anchor = ix;
                    }
                    cursor.keys = Some((
                        keys[ix].clone(),
                        keys[cursor.anchor.min(keys.len() - 1)].clone(),
                    ));
                    cursor.scroll.scroll_to_item(ix);
                    cursor.sent = Some(next.clone());
                    cx.notify();
                });
                log::info!("selectable list {id:?}: {next:?}");
                if let Some(on_change) = &on_change {
                    on_change(&next, window, cx);
                }
            })
        };
        let activate: OnActivate = {
            let (id, on_activate) = (self.id.clone(), self.on_activate);
            Rc::new(move |key, window, cx| {
                log::info!("selectable list {id:?}: activated {key}");
                if let Some(on_activate) = &on_activate {
                    on_activate(key, window, cx);
                }
            })
        };
        let rows: Vec<_> = self
            .rows
            .into_iter()
            .enumerate()
            .map(|(ix, (key, item))| {
                let (pick, focus, activate) = (pick.clone(), focus.clone(), activate.clone());
                item.selected(selected.contains(&key))
                    .current(focused && ix == at)
                    .on_click(move |event, window, cx| {
                        let held = event.modifiers();
                        let how = match (held.shift, held.platform) {
                            (true, _) => Pick::Range,
                            (false, true) => Pick::Toggle,
                            _ => Pick::One,
                        };
                        window.focus(&focus);
                        pick(ix, how, window, cx);
                        if event.click_count() == 2 {
                            activate(&key, window, cx);
                        }
                    })
            })
            .collect();
        self.base
            .id(self.id)
            .track_focus(&focus)
            .track_scroll(&scroll)
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_0p5()
            .on_key_down(move |event, window, cx| {
                if count == 0 {
                    return;
                }
                let held = &event.keystroke.modifiers;
                let at = cursor.read(cx).at.min(count - 1);
                let extend = if held.shift { Pick::Range } else { Pick::One };
                let open = |ix: &usize| !off[*ix];
                let (to, how) = match event.keystroke.key.as_str() {
                    "down" => ((at + 1..count).find(open), extend),
                    "up" => ((0..at).rev().find(open), extend),
                    "home" => ((0..count).find(open), extend),
                    "end" => ((0..count).rev().find(open), extend),
                    "space" => (Some(at).filter(open), Pick::Toggle),
                    "enter" => {
                        cx.stop_propagation();
                        if open(&at) {
                            activate(&keys[at], window, cx);
                        }
                        return;
                    }
                    "a" if held.platform => (Some(at), Pick::All),
                    _ => return,
                };
                cx.stop_propagation();
                if let Some(to) = to {
                    pick(to, how, window, cx);
                }
            })
            .children(rows)
    }
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{Pick, picked};

    fn keys(list: &[&'static str]) -> Vec<SharedString> {
        list.iter().map(|key| SharedString::from(*key)).collect()
    }

    #[test]
    fn picks_one_toggle_a_range_or_all_in_row_order() {
        let rows = keys(&["a", "b", "c", "d"]);
        assert_eq!(picked(&rows, &keys(&["a"]), 0, 2, Pick::One), keys(&["c"]));
        assert_eq!(
            picked(&rows, &keys(&["c", "a"]), 0, 1, Pick::Toggle),
            keys(&["a", "b", "c"])
        );
        assert_eq!(
            picked(&rows, &keys(&["a", "c"]), 0, 0, Pick::Toggle),
            keys(&["c"])
        );
        assert_eq!(
            picked(&rows, &keys(&["a"]), 1, 3, Pick::Range),
            keys(&["b", "c", "d"])
        );
        assert_eq!(
            picked(&rows, &keys(&[]), 3, 1, Pick::Range),
            keys(&["b", "c", "d"])
        );
        assert_eq!(picked(&rows, &keys(&["b"]), 0, 0, Pick::All), rows);
    }
}
