use std::cell::{Cell, RefCell};

use gpui::{AnyElement, IntoElement, TestAppContext};

use super::{at_root, press, shown, stage};
use crate::misc::{ConsentDialog, CookieBanner, LicenseViewer, Package};

thread_local! {
    static SAID: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static OPEN: Cell<bool> = const { Cell::new(true) };
}

fn said() -> Vec<String> {
    SAID.with(|said| said.borrow().clone())
}

fn say(text: impl Into<String>) {
    SAID.with(|said| said.borrow_mut().push(text.into()));
}

fn licenses() -> AnyElement {
    LicenseViewer::new(
        "licenses",
        [
            Package::new("Alpha", "1.0", "MIT", "Permission is hereby granted"),
            Package::new("Beta", "2.3", "ISC", "Permission to use, copy, modify"),
        ],
    )
    .into_any_element()
}

/// The first package's license shows; a move down the list shows the next one's.
#[gpui::test]
fn a_license_viewer_shows_the_chosen_license(cx: &mut TestAppContext) {
    let (_, cx) = stage(licenses, cx);
    assert!(shown("license-Alpha", cx));
    press("tab", cx);
    press("down", cx);
    press("space", cx);
    assert!(shown("license-Beta", cx));
}

fn consent() -> AnyElement {
    let answer = |word: &'static str| {
        move |_: &mut gpui::Window, _: &mut gpui::App| {
            OPEN.set(false);
            say(word)
        }
    };
    match OPEN.get() {
        true => ConsentDialog::new(
            "consent",
            "Terms",
            "Be kind.",
            answer("accepted"),
            answer("declined"),
        )
        .into_any_element(),
        false => gpui::div().into_any_element(),
    }
}

/// Accept asks for the box until it is ticked, then answers once.
#[gpui::test]
fn consent_waits_for_the_box(cx: &mut TestAppContext) {
    let (_, cx) = stage(consent, cx);
    press("tab", cx);
    press("tab", cx);
    press("tab", cx);
    press("enter", cx);
    assert!(said().is_empty() && shown("consent-tick-first", cx));
    press("shift-tab", cx);
    press("shift-tab", cx);
    press("space", cx);
    press("tab", cx);
    press("tab", cx);
    press("enter", cx);
    assert_eq!(said(), ["accepted"], "Accept answers alone");
}

/// Escape declines, once.
#[gpui::test]
fn escape_declines_consent(cx: &mut TestAppContext) {
    let (_, cx) = stage(consent, cx);
    press("tab", cx);
    press("escape", cx);
    assert_eq!(said(), ["declined"]);
}

fn cookies() -> AnyElement {
    CookieBanner::new("cookies", "This page would set cookies.")
        .kind("Analytics", "Counts visits.")
        .kind("Marketing", "Shows offers.")
        .on_choose(|kinds, _, _| say(format!("{kinds:?}")))
        .into_any_element()
}

/// Accept all names every kind and Reject all none; Choose turns into Save choices on its focus, which names the switches turned on.
#[gpui::test]
fn a_cookie_banner_hands_over_the_kinds_chosen(cx: &mut TestAppContext) {
    let (_, cx) = stage(cookies, cx);
    press("tab", cx);
    press("enter", cx);
    press("tab", cx);
    press("enter", cx);
    press("tab", cx);
    press("enter", cx);
    assert!(!at_root(cx), "Save choices took Choose's focus");
    for _ in 0..3 {
        press("shift-tab", cx);
    }
    press("space", cx);
    for _ in 0..3 {
        press("tab", cx);
    }
    press("enter", cx);
    assert_eq!(
        said(),
        [r#"["Analytics", "Marketing"]"#, "[]", r#"["Marketing"]"#]
    );
}
