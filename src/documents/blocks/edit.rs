use std::collections::HashMap;

use gpui::{Context, Window};

use super::{Block, BlockData, BlockEditor, BlockEvent, BlockKind, Snapshot};

impl BlockEditor {
    /// Splits a prose block at the caret: the text after it moves to a new block of the kind that follows.
    pub(crate) fn split(&mut self, key: u64, window: &mut Window, cx: &mut Context<Self>) {
        let ix = self.index(key);
        let kind = self.blocks[ix].kind.clone();
        let field = self.blocks[ix].fields[0].clone();
        if field.read(cx).is_empty()
            && kind != BlockKind::Paragraph
            && kind.next() != BlockKind::Paragraph
        {
            self.turn_into(key, BlockKind::Paragraph, window, cx);
            return;
        }
        let caret = field.read(cx).cursor();
        let tail = field.read(cx).text()[caret..].to_string();
        log::info!("block editor: block {key} splits at {caret}");
        self.before_change(cx);
        field.update(cx, |field, cx| {
            let end = field.text().len();
            field.select(caret..end, cx);
            field.insert("", cx);
        });
        let key = self.fresh_key();
        let next = self.build(
            BlockData {
                kind: kind.next(),
                texts: vec![tail],
            },
            key,
            window,
            cx,
        );
        self.blocks.insert(ix + 1, next);
        self.after_change(cx);
        self.focus_field(key, 0, Some(0), window, cx);
    }

    /// Backspace at a prose block's start: another kind turns back into text; text joins the prose above, or an empty block goes.
    pub(crate) fn join(&mut self, key: u64, window: &mut Window, cx: &mut Context<Self>) {
        let ix = self.index(key);
        if self.blocks[ix].kind != BlockKind::Paragraph {
            self.turn_into(key, BlockKind::Paragraph, window, cx);
            return;
        }
        let text = self.blocks[ix].fields[0].read(cx).text().to_string();
        let Some(above) = ix.checked_sub(1).map(|ix| &self.blocks[ix]) else {
            return;
        };
        if !above.kind.is_prose() {
            if text.is_empty() {
                self.remove(key, window, cx);
            }
            return;
        }
        let (above_key, above_field) = (above.key, above.fields[0].clone());
        log::info!("block editor: block {key} joins block {above_key}");
        self.before_change(cx);
        let joint = above_field.read(cx).text().len();
        above_field.update(cx, |field, cx| {
            field.select(joint..joint, cx);
            field.insert(&text, cx);
        });
        self.blocks.remove(ix);
        self.after_change(cx);
        self.focus_field(above_key, 0, Some(joint), window, cx);
    }

    pub(crate) fn undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settle(true, cx);
        if let Some(previous) = self.history.undo(self.current.clone()) {
            log::info!("block editor: undo");
            self.restore(previous, window, cx);
        }
    }

    pub(crate) fn redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next) = self.history.redo(self.current.clone()) {
            log::info!("block editor: redo");
            self.restore(next, window, cx);
        }
    }

    /// Puts the document back as `snapshot` holds it, keeping fields whose blocks stay.
    fn restore(&mut self, snapshot: Snapshot, window: &mut Window, cx: &mut Context<Self>) {
        let mut kept: HashMap<u64, Block> = self
            .blocks
            .drain(..)
            .map(|block| (block.key, block))
            .collect();
        for (key, data) in &snapshot.0 {
            let block = match kept.remove(key) {
                Some(mut block) if block.fields.len() == data.texts.len() => {
                    block.kind = data.kind.clone();
                    for (field, text) in block.fields.iter().zip(&data.texts) {
                        if field.read(cx).text() != text {
                            field.update(cx, |field, cx| field.set_text(text.clone(), cx));
                        }
                    }
                    block
                }
                _ => self.build(data.clone(), *key, window, cx),
            };
            self.blocks.push(block);
        }
        self.current = snapshot;
        cx.emit(BlockEvent::Changed);
        cx.notify();
    }
}
