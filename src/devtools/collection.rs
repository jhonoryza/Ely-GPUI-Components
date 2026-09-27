use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window};

use super::request::Method;
use crate::{
    lists::{Tree, TreeNode},
    primitives::IconName,
};

/// A folder of saved requests, or a request with its method.
#[derive(Clone, Debug, PartialEq)]
pub enum Saved {
    Folder {
        key: SharedString,
        name: SharedString,
        items: Vec<Saved>,
    },
    Request {
        key: SharedString,
        name: SharedString,
        method: Method,
    },
}

impl Saved {
    pub fn folder(
        key: impl Into<SharedString>,
        name: impl Into<SharedString>,
        items: impl IntoIterator<Item = Saved>,
    ) -> Self {
        Saved::Folder {
            key: key.into(),
            name: name.into(),
            items: items.into_iter().collect(),
        }
    }

    pub fn request(
        key: impl Into<SharedString>,
        name: impl Into<SharedString>,
        method: Method,
    ) -> Self {
        Saved::Request {
            key: key.into(),
            name: name.into(),
            method,
        }
    }

    fn node(&self) -> TreeNode {
        match self {
            Saved::Folder { key, name, items } => TreeNode::new(key.clone(), name.clone())
                .icon(IconName::Folder)
                .children(items.iter().map(Saved::node)),
            Saved::Request { key, name, method } => TreeNode::new(key.clone(), name.clone())
                .icon(IconName::Send)
                .note(method.name()),
        }
    }
}

type OnKey = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type OnKeys = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;

/// Saved requests in folders, each with its method. Enter or a double press opens a request; a drag moves one into another folder or among its own. It fills its box, which the host sizes.
#[derive(IntoElement)]
pub struct CollectionTree {
    id: ElementId,
    items: Vec<Saved>,
    open: Vec<SharedString>,
    selected: Vec<SharedString>,
    on_open: Option<OnKey>,
    on_select: Option<OnKeys>,
}

impl CollectionTree {
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = Saved>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            open: Vec::new(),
            selected: Vec::new(),
            on_open: None,
            on_select: None,
        }
    }

    /// The folders open at first.
    pub fn open(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.open = keys.into_iter().map(Into::into).collect();
        self
    }

    pub fn selected(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected = keys.into_iter().map(Into::into).collect();
        self
    }

    /// Gets the request to open.
    pub fn on_open(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for CollectionTree {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut tree = Tree::new(self.id, self.items.iter().map(Saved::node))
            .open(self.open)
            .selected(self.selected)
            .size_full();
        if let Some(on_open) = self.on_open {
            tree = tree.on_activate(move |key, window, cx| {
                log::info!("collection tree: open {key}");
                on_open(key, window, cx)
            });
        }
        if let Some(on_select) = self.on_select {
            tree = tree.on_select(move |keys, window, cx| on_select(keys, window, cx));
        }
        tree
    }
}
