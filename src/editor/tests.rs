use gpui::{Entity, TestAppContext, VisualTestContext};

use super::{CodeEditor, GhostText, layout::Row};
use crate::theme::Theme;

fn editor<'a>(
    text: &str,
    cx: &'a mut TestAppContext,
) -> (Entity<CodeEditor>, &'a mut VisualTestContext) {
    cx.update(Theme::init);
    let text = text.to_string();
    cx.add_window_view(move |window, cx| CodeEditor::new(text, window, cx))
}

#[gpui::test]
fn typed_pairs_close_and_one_undo_takes_back_the_burst(cx: &mut TestAppContext) {
    let (editor, cx) = editor("let x = ", cx);
    editor.update(cx, |editor, cx| {
        let end = 8..8;
        editor.select([end], cx);
        for ch in ["(", "1", ")"] {
            editor.type_text(ch, cx);
        }
        assert_eq!(
            editor.text(),
            "let x = (1)",
            "the typed close steps over its pair"
        );
        editor.undo(cx);
        assert_eq!(editor.text(), "let x = ");
        editor.redo(cx);
        assert_eq!(editor.text(), "let x = (1)");
    });
}

#[gpui::test]
fn select_next_takes_the_word_then_its_next_match(cx: &mut TestAppContext) {
    let (editor, cx) = editor("a foo b foo c", cx);
    editor.update(cx, |editor, cx| {
        let inside = 3..3;
        editor.select([inside], cx);
        editor.select_next(cx);
        editor.select_next(cx);
        let ranges: Vec<_> = editor.selections().iter().map(|s| s.range()).collect();
        assert_eq!(ranges, [2..5, 8..11]);
        editor.type_text("bar", cx);
        assert_eq!(editor.text(), "a bar b bar c", "every cursor types");
    });
}

#[gpui::test]
fn toggling_comments_twice_restores_the_lines(cx: &mut TestAppContext) {
    let (editor, cx) = editor("fn a() {\n    b();\n}", cx);
    editor.update(cx, |editor, cx| {
        editor.select_all(cx);
        editor.toggle_comment(cx);
        assert_eq!(editor.text(), "// fn a() {\n//     b();\n// }");
        editor.toggle_comment(cx);
        assert_eq!(editor.text(), "fn a() {\n    b();\n}");
    });
}

#[gpui::test]
fn a_multi_line_ghost_moves_the_rest_of_its_line_below_it(cx: &mut TestAppContext) {
    let (editor, cx) = editor("call(x)\nnext", cx);
    editor.update(cx, |editor, cx| {
        editor.set_ghost_text(
            Some(GhostText {
                offset: 5,
                text: "a,\n  b, ".into(),
            }),
            cx,
        );
        assert_eq!(
            editor.row_text(0),
            "call(",
            "the row stops where the ghost starts"
        );
        assert_eq!(editor.row_text(1), "next");
        let caret = 5..5;
        editor.select([caret], cx);
        editor.indent(cx);
        assert_eq!(
            editor.text(),
            "call(a,\n  b, x)\nnext",
            "Tab takes the ghost"
        );
        assert_eq!(editor.row_text(0), "call(a,", "a taken ghost cuts nothing");
    });
}

#[gpui::test]
fn a_selection_inside_a_fold_opens_it(cx: &mut TestAppContext) {
    let (editor, cx) = editor("fn a() {\n    b();\n}\nc", cx);
    editor.update(cx, |editor, cx| {
        editor.toggle_fold(0, cx);
        assert!(!editor.frame_shows(1), "the body folds away");
        let body = 13..13;
        editor.select([body], cx);
        assert!(editor.frame_shows(1), "a cursor inside unfolds it");
    });
}

impl CodeEditor {
    /// Whether line `line` has a row now.
    fn frame_shows(&self, line: usize) -> bool {
        let hide = super::layout::hidden(
            self.buffer.lines(),
            &super::syntax::folds(&self.buffer),
            &self.folded,
        );
        super::layout::rows(&self.buffer, &hide, &self.marks).contains(&Row::Line(line))
    }
}
