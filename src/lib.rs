mod assets;

pub mod buttons;
pub mod charts;
pub mod data_display;
pub mod debug;
pub mod documents;
pub mod editor;
pub mod feedback;
pub mod finance;
pub mod forms;
pub mod git;
pub mod layout;
pub mod lists;
pub mod menus;
pub mod motion;
pub mod navigation;
pub mod overlays;
pub mod primitives;
pub mod shell;
pub mod tables;
pub mod terminal;
pub mod theme;
pub mod typography;

pub use assets::Assets;

use gpui::{App, KeyBinding};

use crate::primitives::{FocusNext, FocusPrev, IconName};

/// Loads fonts and the theme, binds Tab and text keys. Call once, first.
pub fn init(cx: &mut App) {
    match cx.asset_source().load(IconName::Check.path()) {
        Ok(Some(_)) => {}
        Ok(None) => panic!("ely: pass `ely_gpui_component::Assets` to `Application::with_assets`"),
        Err(error) => panic!("ely: asset source failed: {error:#}"),
    }
    assets::load_fonts(cx).expect("ely: embedded fonts failed to register");
    theme::Theme::init(cx);
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrev, None),
    ]);
    forms::bind_keys(cx);
    editor::bind_keys(cx);
    terminal::bind_keys(cx);
    documents::bind_keys(cx);
}
