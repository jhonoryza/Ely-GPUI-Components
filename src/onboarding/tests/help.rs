use gpui::{AnyElement, Entity, IntoElement, ParentElement, Styled, TestAppContext, div, px};

use super::{Bench, bench, edit, said, say, tab, tap, write};
use crate::{
    data_display::{Change, Release},
    forms::Choice,
    onboarding::{
        ContactSupport, FeedbackWidget, HelpArticle, HelpPanel, InlineHelp,
        KeyboardShortcutCheatsheet, WhatsNewDialog,
    },
    settings::Shortcut,
};

fn article(key: &str, title: &str, body: &str) -> HelpArticle {
    HelpArticle {
        key: key.to_string().into(),
        title: title.to_string().into(),
        summary: "A line on it".into(),
        body: body.to_string().into(),
    }
}

fn panel(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let articles = [
        article("start", "Getting started", "Make a project."),
        article("team", "Invite your team", "Send an invitation by email."),
    ];
    let [opener, contact, keys, close] = [(); 4].map(|_| owner.clone());
    div()
        .h(px(420.0))
        .child(
            HelpPanel::new("help", articles, &bench.search, bench.open.clone())
                .on_open(move |key, _, cx| {
                    let key = key.cloned();
                    opener.update(cx, |bench, cx| {
                        bench.said.push(format!(
                            "open {}",
                            key.as_ref().map_or("none", |key| key.as_ref())
                        ));
                        bench.open = key;
                        cx.notify();
                    })
                })
                .on_contact(move |_, cx| say(&contact, "contact".into(), cx))
                .on_shortcuts(move |_, cx| say(&keys, "shortcuts".into(), cx))
                .on_close(move |_, cx| say(&close, "close".into(), cx)),
        )
        .into_any_element()
}

/// Stops on the list: close, the search, each article, then the foot; on an article: close, All articles, then the foot. An article opened by key takes focus to All articles, and the way back takes it to the search, where words narrow the list.
#[gpui::test]
fn an_article_opens_from_the_list_and_focus_follows(cx: &mut TestAppContext) {
    let (host, cx) = bench(panel, cx);
    tab(3, cx);
    tap("space", cx);
    tap("space", cx);
    write("email", cx);
    tab(3, cx);
    tap("space", cx);
    tab(3, cx);
    tap("space", cx);
    tab(4, cx);
    tap("space", cx);
    tab(1, cx);
    tap("space", cx);
    assert_eq!(
        said(&host, cx),
        [
            "open start",
            "open none",
            "open team",
            "contact",
            "shortcuts",
            "close"
        ]
    );
}

fn cheatsheet(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let shortcut = |name: &str, keys: &str| Shortcut {
        group: "General".into(),
        name: name.to_string().into(),
        keys: keys.to_string().into(),
    };
    let dialog = bench.dialog.then(|| {
        KeyboardShortcutCheatsheet::new(
            "keys",
            [shortcut("Undo", "cmd-z"), shortcut("Find", "cmd-f")],
            &bench.search,
            move |_, cx| say(&owner, "closed".into(), cx),
        )
    });
    div().children(dialog).into_any_element()
}

/// The search takes focus as the dialog opens, and Escape closes it.
#[gpui::test]
fn the_cheatsheet_opens_on_its_search_and_escape_closes_it(cx: &mut TestAppContext) {
    let (host, cx) = bench(cheatsheet, cx);
    edit(&host, cx, |bench| bench.dialog = true);
    write("undo", cx);
    let typed = host.read_with(cx, |bench, cx| bench.search.read(cx).text().to_string());
    assert_eq!(typed, "undo");
    tap("escape", cx);
    assert_eq!(said(&host, cx), ["closed"]);
}

fn whats_new(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let dialog = bench.dialog.then(|| {
        WhatsNewDialog::new("new", move |_, cx| say(&owner, "closed".into(), cx))
            .release(Release::new("2.4", "27 Sep 2026").change(Change::Added, "Split view"))
    });
    div().children(dialog).into_any_element()
}

/// Stops: the close button, then Got it.
#[gpui::test]
fn whats_new_lists_the_releases_and_got_it_closes(cx: &mut TestAppContext) {
    let (host, cx) = bench(whats_new, cx);
    edit(&host, cx, |bench| bench.dialog = true);
    assert!(cx.debug_bounds("whats-new-releases").is_some());
    assert!(cx.debug_bounds("whats-new-done").is_some());
    tap("tab", cx);
    tap("tab", cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["closed"]);
}

fn feedback(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    FeedbackWidget::new("feedback")
        .on_send(move |sentiment, note, _, cx| {
            say(&owner, format!("{} {note}", sentiment.key()), cx)
        })
        .into_any_element()
}

/// Stops in the form: the four faces, the note, then Send once a face is picked.
#[gpui::test]
fn feedback_waits_for_a_face_and_says_thanks(cx: &mut TestAppContext) {
    let (host, cx) = bench(feedback, cx);
    tab(1, cx);
    tap("enter", cx);
    for _ in 0..5 {
        tap("tab", cx);
    }
    write("  Fast  ", cx);
    for _ in 0..3 {
        tap("tab", cx);
    }
    tap("space", cx);
    for _ in 0..3 {
        tap("tab", cx);
    }
    tap("space", cx);
    assert_eq!(
        said(&host, cx),
        ["good Fast"],
        "Send rested past the note, so the third Tab reached Good"
    );
    assert!(cx.debug_bounds("feedback-thanks").is_some());
}

fn support(bench: &Bench, owner: Entity<Bench>) -> AnyElement {
    let topics = [
        Choice::new("billing", "Billing"),
        Choice::new("bug", "Something broke"),
    ];
    ContactSupport::new("support", topics)
        .busy(bench.busy)
        .address("help@example.com")
        .on_send(move |topic, message, _, cx| say(&owner, format!("{topic}: {message}"), cx))
        .into_any_element()
}

/// Stops: the topic, the message, then Send once both are given; the message empties once sent, with focus back in it.
#[gpui::test]
fn a_message_goes_with_its_topic_and_empties(cx: &mut TestAppContext) {
    let (host, cx) = bench(support, cx);
    tab(2, cx);
    write("It froze", cx);
    tab(3, cx);
    tap("space", cx);
    assert!(
        said(&host, cx).is_empty(),
        "no topic yet, so Send rests and the third stop wraps to the topic"
    );
    tab(1, cx);
    for key in ["escape", "enter", "down", "enter"] {
        tap(key, cx);
    }
    tab(3, cx);
    tap("space", cx);
    write("Again", cx);
    tab(3, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["bug: It froze", "bug: Again"]);
}

/// While the owner sends, Send spins and takes no press, yet stays the third stop and keeps its focus: once sending ends, the same press sends.
#[gpui::test]
fn a_message_waits_while_busy(cx: &mut TestAppContext) {
    let (host, cx) = bench(support, cx);
    edit(&host, cx, |bench| bench.busy = true);
    tab(1, cx);
    for key in ["enter", "down", "enter"] {
        tap(key, cx);
    }
    tab(2, cx);
    write("It froze", cx);
    tab(3, cx);
    tap("space", cx);
    assert!(said(&host, cx).is_empty(), "Send takes no press while busy");
    edit(&host, cx, |bench| bench.busy = false);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["bug: It froze"]);
}

fn inline(_: &Bench, owner: Entity<Bench>) -> AnyElement {
    InlineHelp::new("hint", "Only admins can change billing.")
        .on_more(move |_, cx| say(&owner, "more".into(), cx))
        .into_any_element()
}

/// Stops: Learn more.
#[gpui::test]
fn inline_help_offers_more(cx: &mut TestAppContext) {
    let (host, cx) = bench(inline, cx);
    tab(1, cx);
    tap("space", cx);
    assert_eq!(said(&host, cx), ["more"]);
}
