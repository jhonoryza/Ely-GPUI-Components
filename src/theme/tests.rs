use gpui::{TestAppContext, hsla};

use super::{ActiveTheme, Mode, Palette, Theme};

/// An owner's palette serves its own mode; the other keeps Ely's, and none brings Ely's back.
#[gpui::test]
fn an_owners_palette_serves_its_mode_alone(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let mut own = Palette::light(false);
    own.accent = hsla(0.6, 0.5, 0.5, 1.0);
    cx.update(|cx| Theme::set_palette(Mode::Light, Some(own.clone()), cx));
    assert_eq!(cx.read(|cx| cx.theme().palette()), own);
    cx.update(|cx| Theme::set_mode(Mode::Dark, cx));
    assert_eq!(cx.read(|cx| cx.theme().palette()), Palette::dark(false));
    cx.update(|cx| {
        Theme::set_mode(Mode::Light, cx);
        Theme::set_palette(Mode::Light, None, cx);
    });
    assert_eq!(cx.read(|cx| cx.theme().palette()), Palette::light(false));
}

#[test]
fn every_named_color_answers() {
    let mut palette = Palette::light(false);
    *palette.token_mut("accent") = hsla(0.1, 0.2, 0.3, 1.0);
    assert_eq!(palette.accent, hsla(0.1, 0.2, 0.3, 1.0));
    *palette.syntax.token_mut("comment") = hsla(0.4, 0.2, 0.3, 1.0);
    assert_eq!(palette.syntax.comment, hsla(0.4, 0.2, 0.3, 1.0));
}

#[test]
#[should_panic(expected = "no palette color glow")]
fn an_unknown_color_fails() {
    Palette::light(false).token_mut("glow");
}
