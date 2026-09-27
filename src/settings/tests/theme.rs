use gpui::{AnyElement, App, Entity, IntoElement, TestAppContext, Window};

use super::{Desk, desk, said, say, tab, tap};
use crate::{
    settings::{ThemeDraft, ThemeEditor},
    theme::ActiveTheme,
};

fn editor(_: &mut Window, cx: &mut App, owner: Entity<Desk>) -> AnyElement {
    ThemeEditor::new("editor", ThemeDraft::of(cx.theme()))
        .on_change(move |draft, _, cx| {
            say(
                &owner,
                format!(
                    "{:?} contrast {} still {}",
                    draft.density, draft.high_contrast, draft.reduced_motion
                ),
                cx,
            )
        })
        .into_any_element()
}

/// Stops: the three densities, corners, text size, high contrast, reduced motion, then the colors. A switch hands on a draft with its own flag turned and the rest as it stood.
#[gpui::test]
fn a_switch_turns_its_own_flag(cx: &mut TestAppContext) {
    let (host, cx) = desk(editor, cx);
    tab(6, cx);
    tap("space", cx);
    tab(7, cx);
    tap("space", cx);
    assert_eq!(
        said(&host, cx),
        [
            "Standard contrast true still true",
            "Standard contrast false still false"
        ],
        "the desk runs with reduced motion on, so its switch turns it off"
    );
}

fn colors(_: &mut Window, cx: &mut App, owner: Entity<Desk>) -> AnyElement {
    ThemeEditor::new("editor", ThemeDraft::of(cx.theme()))
        .on_change(move |draft, _, cx| {
            let background = gpui::Rgba::from(draft.colors.bg);
            say(&owner, crate::forms::hex(background), cx)
        })
        .into_any_element()
}

/// The first color is the eighth stop; Enter opens its picker, where the third stop is the hex field, which takes a new background.
#[gpui::test]
fn a_color_well_sets_its_own_color(cx: &mut TestAppContext) {
    let (host, cx) = desk(colors, cx);
    tab(8, cx);
    tap("enter", cx);
    for _ in 0..3 {
        tap("tab", cx);
    }
    tap("cmd-a", cx);
    cx.simulate_input("#224466");
    tap("enter", cx);
    assert_eq!(said(&host, cx), ["#224466"]);
}
