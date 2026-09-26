use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*,
};
use jiff::Timestamp;

use super::{ListItem, SelectableList};
use crate::{
    navigation::{Breadcrumb, Crumb},
    primitives::{Icon, IconName, file_icon},
    theme::{ActiveTheme, IconSize, TextSize},
    typography::{format, tabular},
};

/// One entry of a directory: a folder, or a file with its size; both with when they last changed.
#[derive(Clone, Debug, PartialEq)]
pub struct DirEntry {
    name: SharedString,
    size: Option<u64>,
    modified: Timestamp,
}

impl DirEntry {
    pub fn folder(name: impl Into<SharedString>, modified: Timestamp) -> Self {
        Self {
            name: name.into(),
            size: None,
            modified,
        }
    }

    pub fn file(name: impl Into<SharedString>, size: u64, modified: Timestamp) -> Self {
        Self {
            name: name.into(),
            size: Some(size),
            modified,
        }
    }

    pub fn name(&self) -> &SharedString {
        &self.name
    }

    pub fn is_folder(&self) -> bool {
        self.size.is_none()
    }

    /// A file's size in bytes; none for a folder.
    pub fn size(&self) -> Option<u64> {
        self.size
    }

    pub fn modified(&self) -> Timestamp {
        self.modified
    }
}

/// A column a listing sorts by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Name,
    Size,
    Modified,
}

/// Entries sorted by `column`, folders always first, ties by name.
fn sorted(mut entries: Vec<DirEntry>, column: Column, ascending: bool) -> Vec<DirEntry> {
    let by_name = |a: &DirEntry, b: &DirEntry| a.name.to_lowercase().cmp(&b.name.to_lowercase());
    entries.sort_by(|a, b| {
        let chosen = match column {
            Column::Name => by_name(a, b),
            Column::Size => a.size.cmp(&b.size),
            Column::Modified => a.modified.cmp(&b.modified),
        };
        let chosen = if ascending { chosen } else { chosen.reverse() };
        b.is_folder()
            .cmp(&a.is_folder())
            .then(chosen)
            .then_with(|| by_name(a, b))
    });
    entries
}

type OnEntry = Rc<dyn Fn(&DirEntry, &mut Window, &mut App)>;
type OnClimb = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A directory's entries in columns: name, size, and when each changed. A header sorts by its column and a second press reverses it; folders stay first. Enter or a double press opens an entry; the path above climbs to an earlier level.
#[derive(IntoElement)]
pub struct DirectoryListing {
    id: ElementId,
    path: Vec<SharedString>,
    entries: Vec<DirEntry>,
    on_open: Option<OnEntry>,
    on_climb: Option<OnClimb>,
}

impl DirectoryListing {
    /// `path` names each level from the root down to this directory.
    pub fn new(
        id: impl Into<ElementId>,
        path: impl IntoIterator<Item = impl Into<SharedString>>,
        entries: impl IntoIterator<Item = DirEntry>,
    ) -> Self {
        let path: Vec<SharedString> = path.into_iter().map(Into::into).collect();
        assert!(!path.is_empty(), "a listing needs the directory it lists");
        Self {
            id: id.into(),
            path,
            entries: entries.into_iter().collect(),
            on_open: None,
            on_climb: None,
        }
    }

    pub fn on_open(mut self, handler: impl Fn(&DirEntry, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    /// Gets the level pressed in the path, zero at the root.
    pub fn on_climb(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_climb = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DirectoryListing {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let order =
            window.use_keyed_state((self.id.clone(), "order"), cx, |_, _| (Column::Name, true));
        let picked = window.use_keyed_state((self.id.clone(), "picked"), cx, |_, _| {
            Vec::<SharedString>::new()
        });
        let (column, ascending) = *order.read(cx);
        let entries = Rc::new(sorted(self.entries, column, ascending));
        let theme = cx.theme();
        let colors = &theme.colors;
        let (size_width, date_width) = (theme.label_width() * 0.5, theme.label_width() * 0.75);
        let now = Timestamp::now();
        let header = |label: &'static str, this: Column| {
            let order = order.clone();
            div()
                .id((self.id.clone(), label))
                .flex()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .hover(|style| style.text_color(colors.fg))
                .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                .on_click(move |_, _, cx| {
                    order.update(cx, |order, cx| {
                        *order = if order.0 == this {
                            (this, !order.1)
                        } else {
                            (this, true)
                        };
                        log::info!(
                            "directory listing: sorted by {:?}, {}",
                            order.0,
                            if order.1 { "rising" } else { "falling" }
                        );
                        cx.notify();
                    })
                })
                .child(label)
                .when(column == this, |header| {
                    header.child(
                        Icon::new(if ascending {
                            IconName::ArrowUpNarrowWide
                        } else {
                            IconName::ArrowDownWideNarrow
                        })
                        .size(IconSize::Xs)
                        .color(colors.fg_muted),
                    )
                })
        };
        let heading = div()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py_1p5()
            .border_b_1()
            .border_color(colors.border)
            .text_size(theme.text_size(TextSize::Xs))
            .text_color(colors.fg_muted)
            .child(div().flex_1().child(header("Name", Column::Name)))
            .child(
                div()
                    .w(size_width)
                    .flex()
                    .justify_end()
                    .child(header("Size", Column::Size)),
            )
            .child(
                div()
                    .w(date_width)
                    .flex()
                    .justify_end()
                    .child(header("Modified", Column::Modified)),
            );
        let list = entries.iter().enumerate().fold(
            SelectableList::new((self.id.clone(), "entries")).selected(picked.read(cx).clone()),
            |list, (ix, entry)| {
                let size = entry
                    .size
                    .map(|bytes| format::file_size(bytes, false))
                    .unwrap_or_else(|| "—".into());
                let cells = div()
                    .flex()
                    .gap_3()
                    .child(
                        tabular(div())
                            .w(size_width)
                            .flex()
                            .justify_end()
                            .child(size),
                    )
                    .child(
                        div()
                            .w(date_width)
                            .flex()
                            .justify_end()
                            .child(format::relative(entry.modified, now)),
                    );
                let item =
                    ListItem::new((self.id.clone(), format!("entry-{ix}")), entry.name.clone())
                        .leading(
                            Icon::new(if entry.is_folder() {
                                IconName::Folder
                            } else {
                                file_icon(&entry.name)
                            })
                            .size(IconSize::Md)
                            .color(colors.fg_muted),
                        )
                        .trailing(cells);
                list.row(entry.name.clone(), item)
            },
        );
        let (store, shown, on_open) = (picked.clone(), entries.clone(), self.on_open);
        let crumbs = self.path.iter().enumerate().map(|(ix, name)| {
            let crumb = Crumb::new(SharedString::from(ix.to_string()), name.clone());
            if ix == 0 {
                crumb.icon(IconName::HardDrive)
            } else {
                crumb
            }
        });
        let on_climb = self.on_climb;
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                Breadcrumb::new((self.id.clone(), "path"), crumbs).on_select(
                    move |level, _, window, cx| {
                        if let Some(on_climb) = &on_climb {
                            on_climb(level, window, cx);
                        }
                    },
                ),
            )
            .child(heading)
            .child(
                list.on_change(move |keys, _, cx| {
                    store.update(cx, |picked, cx| {
                        *picked = keys.to_vec();
                        cx.notify();
                    })
                })
                .on_activate(move |name, window, cx| {
                    let entry = shown
                        .iter()
                        .find(|entry| entry.name == *name)
                        .expect("a listed entry");
                    if let Some(on_open) = &on_open {
                        on_open(entry, window, cx);
                    }
                }),
            )
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::{Column, DirEntry, sorted};

    fn names(entries: Vec<DirEntry>) -> Vec<String> {
        entries
            .into_iter()
            .map(|entry| entry.name.to_string())
            .collect()
    }

    #[test]
    fn folders_stay_first_whichever_way_it_sorts() {
        let at = |seconds: i64| Timestamp::from_second(seconds).expect("a time");
        let entries = vec![
            DirEntry::file("notes.md", 900, at(30)),
            DirEntry::folder("Photos", at(10)),
            DirEntry::file("archive.zip", 50_000, at(20)),
            DirEntry::folder("apps", at(40)),
        ];
        assert_eq!(
            names(sorted(entries.clone(), Column::Name, true)),
            ["apps", "Photos", "archive.zip", "notes.md"]
        );
        assert_eq!(
            names(sorted(entries.clone(), Column::Size, false)),
            ["apps", "Photos", "archive.zip", "notes.md"]
        );
        assert_eq!(
            names(sorted(entries.clone(), Column::Modified, true)),
            ["Photos", "apps", "archive.zip", "notes.md"]
        );
        assert_eq!(
            names(sorted(entries, Column::Name, false)),
            ["Photos", "apps", "notes.md", "archive.zip"]
        );
    }
}
