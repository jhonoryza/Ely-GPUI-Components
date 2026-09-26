use std::{ops::Range, rc::Rc};

use emojis::Group;
use gpui::{
    AnyElement, App, Context, ElementId, Entity, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, ScrollStrategy, SharedString, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window, div, prelude::*, uniform_list,
};

use super::{
    Input, TextInput,
    text::{Down, Enter},
};
use crate::{
    primitives::{Icon, IconName, Tooltip, tab_stop},
    theme::{ActiveTheme, ControlSize, IconSize, Radius, TextSize},
    typography::Emoji,
};

const COLUMNS: usize = 8;

/// A row of a glyph grid: a group's title, or a run of glyphs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Row {
    Title(&'static str),
    Glyphs(Range<usize>),
}

/// Rows for glyph groups given as (title, count), laid in order.
pub(crate) fn rows(groups: impl IntoIterator<Item = (Option<&'static str>, usize)>) -> Vec<Row> {
    let (mut out, mut start) = (Vec::new(), 0);
    for (title, count) in groups.into_iter().filter(|(_, count)| *count > 0) {
        out.extend(title.map(Row::Title));
        let end = start + count;
        while start < end {
            let stop = (start + COLUMNS).min(end);
            out.push(Row::Glyphs(start..stop));
            start = stop;
        }
    }
    out
}

fn row_of(rows: &[Row], ix: usize) -> usize {
    rows.iter()
        .position(|row| matches!(row, Row::Glyphs(range) if range.contains(&ix)))
        .expect("every glyph sits in a row")
}

/// The glyph an arrow key moves to from `at`: same column in the next row, or the last one there.
pub(crate) fn stepped(rows: &[Row], at: usize, key: &str) -> Option<usize> {
    let row = row_of(rows, at);
    let Row::Glyphs(here) = &rows[row] else {
        unreachable!("row_of finds glyph rows");
    };
    let column = at - here.start;
    let land = |row: &Row| match row {
        Row::Glyphs(range) => Some((range.start + column).min(range.end - 1)),
        Row::Title(_) => None,
    };
    let last = rows
        .iter()
        .rev()
        .find_map(|row| match row {
            Row::Glyphs(range) => Some(range.end - 1),
            Row::Title(_) => None,
        })
        .expect("a grid with a cursor has glyphs");
    match key {
        "left" => at.checked_sub(1),
        "right" => (at < last).then_some(at + 1),
        "up" => rows[..row].iter().rev().find_map(land),
        "down" => rows[row + 1..].iter().find_map(land),
        _ => None,
    }
}

/// A grid's search field, its cursor, and where its list has scrolled.
struct Finder {
    search: Entity<TextInput>,
    query: String,
    cursor: usize,
    scroll: UniformListScrollHandle,
}

fn finder(id: &ElementId, hint: &'static str, window: &mut Window, cx: &mut App) -> Entity<Finder> {
    let finder = window.use_keyed_state(
        (id.clone(), "finder"),
        cx,
        |window, cx: &mut Context<Finder>| Finder {
            search: cx.new(|cx| TextInput::new(window, cx).placeholder(hint)),
            query: String::new(),
            cursor: 0,
            scroll: UniformListScrollHandle::new(),
        },
    );
    let query = {
        let search = finder.read(cx).search.clone();
        search.read(cx).text().trim().to_lowercase()
    };
    if finder.read(cx).query != query {
        finder.update(cx, |finder, _| {
            finder.query = query;
            finder.cursor = 0;
            finder.scroll.scroll_to_item(0, ScrollStrategy::Top);
        });
    }
    finder
}

type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type Draw = Rc<dyn Fn(usize, &App) -> AnyElement>;
type OnIcon = Rc<dyn Fn(IconName, &mut Window, &mut App)>;
type OnEmoji = Rc<dyn Fn(&str, &mut Window, &mut App)>;

/// What a glyph grid shows: names for tooltips, rows, a painter, and what a pick does.
struct Grid {
    id: ElementId,
    names: Rc<Vec<SharedString>>,
    rows: Rc<Vec<Row>>,
    selected: Option<usize>,
    draw: Draw,
    pick: Pick,
    none: &'static str,
}

fn grid(grid: Grid, finder: &Entity<Finder>, window: &mut Window, cx: &mut App) -> AnyElement {
    let focus = tab_stop((grid.id.clone(), "grid").into(), true, window, cx);
    let focused = focus.is_focused(window);
    let count = grid.names.len();
    let (search, cursor, scroll) = {
        let finder = finder.read(cx);
        let cursor = finder.cursor.min(count.saturating_sub(1));
        (finder.search.clone(), cursor, finder.scroll.clone())
    };
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let cell = theme.control_height(ControlSize::Lg);
    let (into, enter, keys) = (focus.clone(), grid.pick.clone(), grid.pick.clone());
    let field = div()
        .capture_action(move |_: &Down, window, cx| {
            if count > 0 {
                cx.stop_propagation();
                window.focus(&into);
            }
        })
        .capture_action(move |_: &Enter, window, cx| {
            if count > 0 {
                cx.stop_propagation();
                enter(cursor, window, cx);
            }
        })
        .child(
            Input::new(&search).prefix(
                Icon::new(IconName::Search)
                    .size(IconSize::Sm)
                    .color(colors.fg_subtle),
            ),
        );
    let body = if count == 0 {
        div()
            .h(theme.list_max_height())
            .pt_6()
            .text_center()
            .text_color(colors.fg_subtle)
            .child(grid.none)
            .into_any_element()
    } else {
        let (rows, names, draw, pick) = (grid.rows.clone(), grid.names, grid.draw, grid.pick);
        let selected = grid.selected;
        let (moved, key_rows, place) = (finder.clone(), grid.rows, scroll.clone());
        let clicked = finder.clone();
        let list = uniform_list((grid.id, "rows"), rows.len(), move |range, _, cx| {
            let theme = cx.theme();
            rows[range]
                .iter()
                .map(|row| match row {
                    Row::Title(title) => div()
                        .h(cell)
                        .flex()
                        .items_end()
                        .pb_1()
                        .text_size(theme.text_size(TextSize::Xs))
                        .text_color(colors.fg_subtle)
                        .child(*title)
                        .into_any_element(),
                    Row::Glyphs(glyphs) => div()
                        .flex()
                        .children(glyphs.clone().map(|ix| {
                            let (pick, name, clicked) =
                                (pick.clone(), names[ix].clone(), clicked.clone());
                            let ring = if focused && ix == cursor {
                                colors.focus
                            } else if selected == Some(ix) {
                                colors.accent
                            } else {
                                gpui::transparent_black()
                            };
                            div()
                                .id(("glyph", ix))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(cell)
                                .rounded(theme.radius(Radius::Md))
                                .border_1()
                                .border_color(ring)
                                .when(selected == Some(ix), |cell| cell.bg(colors.active))
                                .cursor_pointer()
                                .hover(|style| style.bg(colors.hover))
                                .tooltip(Tooltip::text(name))
                                .on_click(move |_, window, cx| {
                                    clicked.update(cx, |finder, cx| {
                                        finder.cursor = ix;
                                        cx.notify();
                                    });
                                    pick(ix, window, cx);
                                })
                                .child(draw(ix, cx))
                        }))
                        .into_any_element(),
                })
                .collect()
        })
        .track_scroll(scroll)
        .h(theme.list_max_height());
        div()
            .track_focus(&focus)
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if matches!(key, "enter" | "space") {
                    cx.stop_propagation();
                    keys(cursor, window, cx);
                    return;
                }
                let Some(to) = stepped(&key_rows, cursor, key) else {
                    return;
                };
                cx.stop_propagation();
                let strategy = if to < cursor {
                    ScrollStrategy::Top
                } else {
                    ScrollStrategy::Bottom
                };
                place.scroll_to_item(row_of(&key_rows, to), strategy);
                moved.update(cx, |finder, cx| {
                    finder.cursor = to;
                    cx.notify();
                });
            })
            .child(list)
            .into_any_element()
    };
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w(cell * COLUMNS as f32)
        .child(field)
        .child(body)
        .into_any_element()
}

/// A searchable grid of the library's icons.
#[derive(IntoElement)]
pub struct IconPicker {
    id: ElementId,
    selected: Option<IconName>,
    on_change: Option<OnIcon>,
}

impl IconPicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            on_change: None,
        }
    }

    pub fn selected(mut self, icon: IconName) -> Self {
        self.selected = Some(icon);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(IconName, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for IconPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let finder = finder(&self.id, "Search icons", window, cx);
        let query = finder.read(cx).query.clone();
        let found: Rc<Vec<IconName>> = Rc::new(
            IconName::ALL
                .iter()
                .copied()
                .filter(|icon| icon.name().replace('-', " ").contains(&query))
                .collect(),
        );
        let (painted, picked) = (found.clone(), found.clone());
        let (id, on_change) = (self.id.clone(), self.on_change);
        let pick: Pick = Rc::new(move |ix, window, cx| {
            let icon = picked[ix];
            log::info!("icon picker {id:?}: {}", icon.name());
            if let Some(on_change) = &on_change {
                on_change(icon, window, cx);
            }
        });
        let fg = cx.theme().colors.fg;
        grid(
            Grid {
                id: self.id,
                names: Rc::new(found.iter().map(|icon| icon.name().into()).collect()),
                rows: Rc::new(rows([(None, found.len())])),
                selected: self
                    .selected
                    .and_then(|icon| found.iter().position(|f| *f == icon)),
                draw: Rc::new(move |ix, _| {
                    Icon::new(painted[ix])
                        .size(IconSize::Md)
                        .color(fg)
                        .into_any_element()
                }),
                pick,
                none: "No icons match",
            },
            &finder,
            window,
            cx,
        )
    }
}

/// Emoji whose name or shortcode holds `query`, a lowercase string; all for an empty one.
pub(crate) fn emoji_found(query: &str) -> Vec<&'static emojis::Emoji> {
    emojis::iter()
        .filter(|emoji| {
            query.is_empty()
                || emoji.name().to_lowercase().contains(query)
                || emoji.shortcodes().any(|code| code.contains(query))
        })
        .collect()
}

fn title(group: Group) -> &'static str {
    match group {
        Group::SmileysAndEmotion => "Smileys & Emotion",
        Group::PeopleAndBody => "People & Body",
        Group::AnimalsAndNature => "Animals & Nature",
        Group::FoodAndDrink => "Food & Drink",
        Group::TravelAndPlaces => "Travel & Places",
        Group::Activities => "Activities",
        Group::Objects => "Objects",
        Group::Symbols => "Symbols",
        Group::Flags => "Flags",
    }
}

/// A searchable grid of emoji, grouped as Unicode groups them.
#[derive(IntoElement)]
pub struct EmojiPicker {
    id: ElementId,
    selected: Option<SharedString>,
    on_change: Option<OnEmoji>,
}

impl EmojiPicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            on_change: None,
        }
    }

    pub fn selected(mut self, emoji: impl Into<SharedString>) -> Self {
        self.selected = Some(emoji.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for EmojiPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let finder = finder(&self.id, "Search emoji", window, cx);
        let query = finder.read(cx).query.clone();
        let found = emoji_found(&query);
        let layout = if query.is_empty() {
            rows(Group::iter().map(|group| (Some(title(group)), group.emojis().count())))
        } else {
            rows([(None, found.len())])
        };
        let found = Rc::new(found);
        let (painted, picked) = (found.clone(), found.clone());
        let (id, on_change) = (self.id.clone(), self.on_change);
        let pick: Pick = Rc::new(move |ix, window, cx| {
            let emoji = picked[ix];
            log::info!("emoji picker {id:?}: {}", emoji.name());
            if let Some(on_change) = &on_change {
                on_change(emoji.as_str(), window, cx);
            }
        });
        grid(
            Grid {
                id: self.id,
                names: Rc::new(found.iter().map(|emoji| emoji.name().into()).collect()),
                rows: Rc::new(layout),
                selected: self.selected.and_then(|glyph| {
                    found
                        .iter()
                        .position(|emoji| emoji.as_str() == glyph.as_ref())
                }),
                draw: Rc::new(move |ix, _| {
                    Emoji::new(painted[ix].as_str())
                        .size(TextSize::Lg)
                        .into_any_element()
                }),
                pick,
                none: "No emoji match",
            },
            &finder,
            window,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Row, emoji_found, rows, stepped};

    #[test]
    fn emoji_are_found_whatever_the_case_of_their_names() {
        let glyphs = |query| {
            emoji_found(query)
                .into_iter()
                .map(|emoji| emoji.as_str())
                .collect::<Vec<_>>()
        };
        assert!(glyphs("united states").contains(&"🇺🇸"));
        assert!(glyphs("sparkles").contains(&"✨"));
        assert_eq!(glyphs("").len(), emojis::iter().count());
    }

    #[test]
    fn groups_break_into_titled_rows_of_eight() {
        let laid = rows([(Some("A"), 10), (None, 0), (Some("B"), 3)]);
        assert_eq!(
            laid,
            [
                Row::Title("A"),
                Row::Glyphs(0..8),
                Row::Glyphs(8..10),
                Row::Title("B"),
                Row::Glyphs(10..13),
            ]
        );
    }

    #[test]
    fn arrows_keep_the_column_and_cross_titles() {
        let laid = rows([(Some("A"), 10), (Some("B"), 3)]);
        assert_eq!(stepped(&laid, 6, "down"), Some(9));
        assert_eq!(stepped(&laid, 9, "down"), Some(11));
        assert_eq!(stepped(&laid, 11, "up"), Some(9));
        assert_eq!(stepped(&laid, 1, "up"), None);
        assert_eq!(stepped(&laid, 12, "right"), None);
        assert_eq!(stepped(&laid, 0, "left"), None);
        assert_eq!(stepped(&laid, 8, "left"), Some(7));
    }
}
