use gpui::{AppContext as _, Entity, Focusable, TestAppContext, VisualTestContext};

use super::{BlockData, BlockEditor, BlockKind};
use crate::{forms, theme::Theme};

fn open(
    blocks: Vec<BlockData>,
    cx: &mut TestAppContext,
) -> (Entity<BlockEditor>, &mut VisualTestContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
        super::bind_keys(cx);
    });
    let (editor, cx) = cx.add_window_view(|window, cx| BlockEditor::new(blocks, window, cx));
    settle(cx);
    (editor, cx)
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn blocks(
    editor: &Entity<BlockEditor>,
    cx: &mut VisualTestContext,
) -> Vec<(BlockKind, Vec<String>)> {
    cx.update(|_, cx| {
        editor
            .read(cx)
            .blocks(cx)
            .into_iter()
            .map(|block| (block.kind, block.texts))
            .collect()
    })
}

/// Puts the caret in block `ix`'s first field at byte `at`.
fn caret(editor: &Entity<BlockEditor>, ix: usize, at: usize, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let field = editor.read(cx).blocks[ix].fields[0].clone();
        field.update(cx, |field, cx| field.select(at..at, cx));
        window.focus(&field.focus_handle(cx));
    });
    settle(cx);
}

fn text(kind: BlockKind, text: &str) -> (BlockKind, Vec<String>) {
    (kind, vec![text.to_string()])
}

#[gpui::test]
fn enter_splits_prose_at_the_caret(cx: &mut TestAppContext) {
    let (editor, cx) = open(vec![BlockData::new(BlockKind::Bullet, ["lift"])], cx);
    caret(&editor, 0, 2, cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(
        blocks(&editor, cx),
        [text(BlockKind::Bullet, "li"), text(BlockKind::Bullet, "ft")]
    );
}

#[gpui::test]
fn backspace_at_the_start_joins_up_and_undo_parts_them(cx: &mut TestAppContext) {
    let (editor, cx) = open(
        vec![
            BlockData::new(BlockKind::Paragraph, ["tone "]),
            BlockData::new(BlockKind::Paragraph, ["lift"]),
        ],
        cx,
    );
    caret(&editor, 1, 0, cx);
    cx.simulate_keystrokes("backspace");
    settle(cx);
    assert_eq!(
        blocks(&editor, cx),
        [text(BlockKind::Paragraph, "tone lift")]
    );
    cx.simulate_keystrokes("cmd-z");
    settle(cx);
    assert_eq!(
        blocks(&editor, cx),
        [
            text(BlockKind::Paragraph, "tone "),
            text(BlockKind::Paragraph, "lift")
        ],
        "undo brings the joined block back"
    );
}

#[gpui::test]
fn markdown_typed_at_a_paragraphs_start_changes_its_kind(cx: &mut TestAppContext) {
    let (editor, cx) = open(vec![BlockData::new(BlockKind::Paragraph, [""])], cx);
    caret(&editor, 0, 0, cx);
    cx.simulate_input("## Lift");
    settle(cx);
    assert_eq!(blocks(&editor, cx), [text(BlockKind::Heading(2), "Lift")]);
}

#[gpui::test]
fn backspace_turns_a_heading_back_into_text(cx: &mut TestAppContext) {
    let (editor, cx) = open(vec![BlockData::new(BlockKind::Heading(1), ["Lift"])], cx);
    caret(&editor, 0, 0, cx);
    cx.simulate_keystrokes("backspace");
    settle(cx);
    assert_eq!(blocks(&editor, cx), [text(BlockKind::Paragraph, "Lift")]);
}

#[gpui::test]
fn a_press_in_a_blocks_text_puts_the_caret_there(cx: &mut TestAppContext) {
    let (editor, cx) = open(vec![BlockData::new(BlockKind::Paragraph, ["lift"])], cx);
    cx.simulate_click(
        gpui::point(gpui::px(300.0), gpui::px(10.0)),
        gpui::Modifiers::none(),
    );
    settle(cx);
    let focused = cx.update(|window, cx| editor.read(cx).focused(window, cx));
    assert!(focused.is_some(), "the press focuses the block's field");
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert_eq!(blocks(&editor, cx).len(), 2);
}

/// The editor at a fixed width inside a row, as a page holds it.
struct Hosted(Entity<BlockEditor>);

impl gpui::Render for Hosted {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        _: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::{ParentElement, Styled};
        gpui::div()
            .flex()
            .child(gpui::div().w(gpui::px(760.0)).child(self.0.clone()))
    }
}

#[gpui::test]
fn a_press_past_the_words_still_lands_in_the_block(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        forms::bind_keys(cx);
        super::bind_keys(cx);
    });
    let (host, cx) = cx.add_window_view(|window, cx| {
        let editor = cx.new(|cx| {
            BlockEditor::new(
                vec![BlockData::new(BlockKind::Paragraph, ["lift"])],
                window,
                cx,
            )
        });
        Hosted(editor)
    });
    settle(cx);
    cx.simulate_click(
        gpui::point(gpui::px(700.0), gpui::px(10.0)),
        gpui::Modifiers::none(),
    );
    settle(cx);
    let editor = cx.update(|_, cx| host.read(cx).0.clone());
    let focused = cx.update(|window, cx| editor.read(cx).focused(window, cx));
    assert!(focused.is_some(), "the press focuses the block's field");
}

#[gpui::test]
fn a_press_beside_a_blocks_text_puts_the_caret_at_its_end(cx: &mut TestAppContext) {
    let (editor, cx) = open(vec![BlockData::new(BlockKind::Paragraph, ["lift"])], cx);
    cx.simulate_click(
        gpui::point(gpui::px(300.0), gpui::px(1.0)),
        gpui::Modifiers::none(),
    );
    settle(cx);
    let focused = cx.update(|window, cx| editor.read(cx).focused(window, cx));
    assert_eq!(
        focused,
        Some((editor.read_with(cx, |editor, _| editor.blocks[0].key), 0))
    );
}
