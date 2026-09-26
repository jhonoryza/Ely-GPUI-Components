use gpui::{Entity, TestAppContext, VisualTestContext};

use super::Terminal;
use crate::{editor::FindOptions, theme::Theme};

fn replayed<'a>(
    bytes: &'static [u8],
    cx: &'a mut TestAppContext,
) -> (Entity<Terminal>, &'a mut VisualTestContext) {
    cx.update(Theme::init);
    let (terminal, cx) = cx.add_window_view(move |_, cx| Terminal::replay(bytes, 30, 4, cx));
    cx.run_until_parked();
    (terminal, cx)
}

#[gpui::test]
fn replayed_output_shows_without_its_codes(cx: &mut TestAppContext) {
    let (terminal, cx) = replayed(b"\x1b[32mok\x1b[0m done\r\n\x1b]0;build\x07next", cx);
    terminal.read_with(cx, |terminal, _| {
        assert_eq!(terminal.text(), "ok done\nnext");
        assert_eq!(terminal.title().as_ref(), "build", "OSC 0 names it");
        assert_eq!(terminal.shown.cursor, None, "a replay shows no cursor");
    });
}

#[gpui::test]
fn find_counts_matches_and_steps_round(cx: &mut TestAppContext) {
    let (terminal, cx) = replayed(b"one two one\r\nthree one", cx);
    terminal.update(cx, |terminal, cx| {
        assert_eq!(terminal.find("one", FindOptions::default(), cx), Ok(3));
        assert_eq!(terminal.matches(), (3, Some(2)), "the newest match first");
        terminal.step(true, cx);
        assert_eq!(
            terminal.matches(),
            (3, Some(0)),
            "past the last comes the first"
        );
        assert!(
            terminal
                .find(
                    "(",
                    FindOptions {
                        regex: true,
                        ..FindOptions::default()
                    },
                    cx
                )
                .is_err()
        );
    });
}
