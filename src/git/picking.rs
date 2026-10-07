//! Long lists draw in view, load more, pick by key.

use std::{cell::RefCell, rc::Rc};

use gpui::{
    Context, IntoElement, KeyBinding, KeyUpEvent, Keystroke, ParentElement, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, px,
};

use super::{Blame, BlameView, Commit, CommitList, FileHistory};
use crate::{forms, primitives::FocusNext, theme::Theme};

fn commit(n: usize) -> Commit {
    Commit {
        id: format!("{n:040}").into(),
        parents: vec![format!("{:040}", n + 1).into()],
        subject: format!("change {n}").into(),
        author: "Ann".into(),
        when: "today".into(),
        refs: Vec::new(),
    }
}

/// What a list asked for.
#[derive(Default)]
struct Heard {
    picked: Vec<String>,
    ends: usize,
}

struct Lists {
    count: usize,
    heard: Rc<RefCell<Heard>>,
    which: &'static str,
}

impl Render for Lists {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let commits: Vec<Commit> = (0..self.count).map(commit).collect();
        let (pick, end) = (self.heard.clone(), self.heard.clone());
        let frame = div().w(px(600.0)).h(px(240.0));
        match self.which {
            "commits" => frame.child(
                CommitList::new("commits", Rc::new(commits))
                    .on_pick(move |id, _, _| pick.borrow_mut().picked.push(id.to_string()))
                    .on_end(move |_| end.borrow_mut().ends += 1),
            ),
            "history" => frame.child(
                FileHistory::new(
                    "history",
                    Rc::new(commits.into_iter().map(|c| (c, None)).collect()),
                )
                .on_pick(move |id, _, _| pick.borrow_mut().picked.push(id.to_string())),
            ),
            _ => {
                let blame = |commit: &str| Blame {
                    commit: commit.into(),
                    author: "Ann".into(),
                    when: "today".into(),
                    subject: "first".into(),
                    age: 0.0,
                };
                frame.child(
                    BlameView::new(
                        "blame",
                        "one\ntwo\nthree",
                        [0, 0, 1],
                        [blame("a1"), blame("b2")],
                    )
                    .on_commit(move |id, _, _| pick.borrow_mut().picked.push(id.to_string())),
                )
            }
        }
    }
}

fn shown<'a>(
    cx: &'a mut TestAppContext,
    count: usize,
    which: &'static str,
) -> (Rc<RefCell<Heard>>, &'a mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
    let heard = Rc::new(RefCell::new(Heard::default()));
    let held = heard.clone();
    let (_, cx) = cx.add_window_view(move |_, _| Lists {
        count,
        heard: held,
        which,
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (heard, cx)
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

/// Tab to the first stop and press Enter.
fn press_first(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.focus_next(cx));
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").expect("a key"),
    });
    settle(cx);
}

#[gpui::test]
fn a_long_list_asks_for_more_only_near_its_end(cx: &mut TestAppContext) {
    let (heard, _) = shown(cx, 500, "commits");
    assert_eq!(heard.borrow().ends, 0, "500 rows, few in view");
}

#[gpui::test]
fn a_short_list_asks_for_more(cx: &mut TestAppContext) {
    let (heard, _) = shown(cx, 12, "commits");
    assert!(heard.borrow().ends > 0);
}

#[gpui::test]
fn a_commit_picks_from_the_keyboard(cx: &mut TestAppContext) {
    let (heard, cx) = shown(cx, 500, "commits");
    press_first(cx);
    assert_eq!(heard.borrow().picked, [format!("{:040}", 0)]);
}

#[gpui::test]
fn a_file_history_row_picks_from_the_keyboard(cx: &mut TestAppContext) {
    let (heard, cx) = shown(cx, 300, "history");
    press_first(cx);
    assert_eq!(heard.borrow().picked, [format!("{:040}", 0)]);
}

#[gpui::test]
fn a_blame_run_picks_its_commit_from_the_keyboard(cx: &mut TestAppContext) {
    let (heard, cx) = shown(cx, 0, "blame");
    press_first(cx);
    cx.update(|window, cx| window.focus_next(cx));
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").expect("a key"),
    });
    settle(cx);
    assert_eq!(
        heard.borrow().picked,
        ["a1", "b2"],
        "each run's head, no stop between"
    );
}
