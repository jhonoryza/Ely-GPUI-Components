mod agent;
mod buttons;
mod calendar;
mod charts;
mod chat;
mod collab;
mod data;
mod debug;
mod documents;
mod editor;
mod feedback;
mod files;
mod finance;
mod forms;
mod generative;
mod git;
mod layout;
mod lists;
mod mail;
mod media;
mod menus;
mod messaging;
mod motion;
mod navigation;
mod overlays;
mod primitives;
mod project;
mod shell;
mod tables;
mod terminal;
mod theme;
mod typography;

use gpui::{AnyElement, App, Window};

use crate::script::Step;

pub struct Page {
    pub number: u8,
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub render: fn(&mut Window, &mut App) -> AnyElement,
    /// Interactions shot after the static pages.
    pub script: &'static [Step],
}

pub const ALL: &[Page] = &[
    primitives::PAGE,
    typography::PAGE,
    layout::PAGE,
    shell::PAGE,
    buttons::PAGE,
    forms::PAGE,
    navigation::PAGE,
    menus::PAGE,
    overlays::PAGE,
    feedback::PAGE,
    motion::PAGE,
    data::PAGE,
    lists::PAGE,
    tables::PAGE,
    charts::PAGE,
    finance::PAGE,
    editor::PAGE,
    terminal::PAGE,
    git::PAGE,
    debug::PAGE,
    documents::PAGE,
    collab::PAGE,
    chat::PAGE,
    agent::PAGE,
    generative::PAGE,
    media::PAGE,
    files::PAGE,
    messaging::PAGE,
    mail::PAGE,
    calendar::PAGE,
    project::PAGE,
    theme::PAGE,
];

pub use shell::{open_about, open_managed};

pub fn find(slug: &str) -> Option<usize> {
    ALL.iter().position(|page| page.slug == slug)
}
