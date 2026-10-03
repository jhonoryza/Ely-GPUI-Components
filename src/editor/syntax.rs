use std::ops::Range;
use std::rc::Rc;

use gpui::{App, Global, HighlightStyle};

use super::buffer::Buffer;
use crate::forms::{Kind, code_highlights, lex};

/// A host-installed syntax highlighter, consulted before the built-in
/// one-pass highlighter.
///
/// Receives the editor's language name (as set with `CodeEditor::language`),
/// the line's byte range in the buffer, and the whole buffer text. Returns the
/// line's highlight spans — ranges relative to the line start — or `None` to
/// fall back to the built-in highlighter (unknown language, parse failure).
#[derive(Clone)]
pub struct HighlightFn(
    pub Rc<dyn Fn(&str, Range<usize>, &str, &App) -> Option<Vec<(Range<usize>, HighlightStyle)>>>,
);

impl Global for HighlightFn {}

/// Install the process-wide syntax highlighter. Call once at startup; the
/// editor falls back to its built-in highlighter wherever this returns `None`.
pub fn set_highlighter(cx: &mut App, highlight: HighlightFn) {
    cx.set_global(highlight);
}

/// Each line that opens a block, and the last line the block holds, read from indentation.
pub(crate) fn folds(buffer: &Buffer) -> Vec<(usize, usize)> {
    let blank = |line: usize| buffer.line(line).trim().is_empty();
    let mut out = Vec::new();
    for line in 0..buffer.lines() {
        if blank(line) {
            continue;
        }
        let indent = buffer.indent(line);
        let mut end = None;
        for next in line + 1..buffer.lines() {
            if blank(next) {
                continue;
            }
            if buffer.indent(next) <= indent {
                break;
            }
            end = Some(next);
        }
        if let Some(end) = end {
            out.push((line, end));
        }
    }
    out
}

/// A bracket in the code, how deep it sits, and where its partner is when it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bracket {
    pub offset: usize,
    pub depth: usize,
    pub partner: Option<usize>,
}

fn opener(close: char) -> char {
    match close {
        ')' => '(',
        ']' => '[',
        _ => '{',
    }
}

/// Every bracket outside strings and comments; a closer takes its opener's depth.
pub(crate) fn brackets(buffer: &Buffer) -> Vec<Bracket> {
    let mut out: Vec<Bracket> = Vec::new();
    let mut open: Vec<(usize, char)> = Vec::new();
    for line in 0..buffer.lines() {
        let start = buffer.line_range(line).start;
        let text = buffer.line(line);
        for (range, kind) in lex(text) {
            if kind != Kind::Mark {
                continue;
            }
            let Some(ch) = text[range.clone()].chars().next() else {
                continue;
            };
            let offset = start + range.start;
            match ch {
                '(' | '[' | '{' => {
                    open.push((out.len(), ch));
                    out.push(Bracket {
                        offset,
                        depth: open.len() - 1,
                        partner: None,
                    });
                }
                ')' | ']' | '}' => match open.last() {
                    Some((ix, wanted)) if *wanted == opener(ch) => {
                        let ix = *ix;
                        open.pop();
                        let partner = out[ix].offset;
                        out[ix].partner = Some(offset);
                        out.push(Bracket {
                            offset,
                            depth: open.len(),
                            partner: Some(partner),
                        });
                    }
                    _ => out.push(Bracket {
                        offset,
                        depth: open.len(),
                        partner: None,
                    }),
                },
                _ => {}
            }
        }
    }
    out
}

/// The pair beside a caret: the bracket just after it, or else the one just before.
pub(crate) fn matched(brackets: &[Bracket], caret: usize) -> Option<(usize, usize)> {
    let at = |offset: usize| {
        brackets
            .binary_search_by_key(&offset, |bracket| bracket.offset)
            .ok()
            .map(|ix| brackets[ix])
    };
    let bracket = at(caret).or_else(|| caret.checked_sub(1).and_then(at))?;
    Some((bracket.offset, bracket.partner?))
}

/// A line's syntax colors as highlight styles.
///
/// When the host installed a highlighter with [`set_highlighter`] and it
/// returns spans for this line, those win; otherwise the built-in one-pass
/// highlighter runs on the line's text.
pub(crate) fn colors(
    language: &str,
    line_range: Range<usize>,
    buffer: &str,
    cx: &App,
) -> Vec<(Range<usize>, HighlightStyle)> {
    if let Some(spans) = cx
        .try_global::<HighlightFn>()
        .and_then(|highlight| (highlight.0)(language, line_range.clone(), buffer, cx))
    {
        return spans;
    }
    let text = &buffer[line_range];
    code_highlights(text, cx)
        .into_iter()
        .map(|(range, highlight)| {
            (
                range,
                HighlightStyle {
                    color: Some(highlight.color),
                    background_color: highlight.background,
                    ..HighlightStyle::default()
                },
            )
        })
        .collect()
}

/// Highlight for non-editor surfaces (chat code blocks, diff viewers, hover
/// cards): no language context, so the installed hook only applies when it
/// handles the empty language name; otherwise the built-in highlighter runs.
pub(crate) fn code_colors(text: &str, cx: &App) -> Vec<(Range<usize>, HighlightStyle)> {
    colors("", 0..text.len(), text, cx)
}

/// Where `needle` appears in the lines of `range`, as offsets, whole words only when it is a word.
pub(crate) fn occurrences(buffer: &Buffer, needle: &str, lines: Range<usize>) -> Vec<Range<usize>> {
    if needle.is_empty() || needle.contains('\n') {
        return Vec::new();
    }
    let word = needle.chars().all(|ch| ch.is_alphanumeric() || ch == '_');
    let boundary = |ch: Option<char>| ch.is_none_or(|ch| !(ch.is_alphanumeric() || ch == '_'));
    let mut out = Vec::new();
    for line in lines.start..lines.end.min(buffer.lines()) {
        let start = buffer.line_range(line).start;
        let text = buffer.line(line);
        for (at, _) in text.match_indices(needle) {
            let end = at + needle.len();
            let whole =
                boundary(text[..at].chars().next_back()) && boundary(text[end..].chars().next());
            if !word || whole {
                out.push(start + at..start + end);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_fold_by_indentation_and_skip_blank_lines() {
        let buffer = Buffer::new("fn a() {\n    one();\n\n    two();\n}\nfn b() {}");
        assert_eq!(folds(&buffer), [(0, 3)]);
    }

    #[test]
    fn brackets_pair_by_depth_outside_strings_and_comments() {
        let buffer = Buffer::new("f(a[0], \"(\") // )\n{ }");
        let found = brackets(&buffer);
        let offsets: Vec<usize> = found.iter().map(|bracket| bracket.offset).collect();
        assert_eq!(
            offsets,
            [1, 3, 5, 11, 18, 20],
            "the quoted and the commented ones do not count"
        );
        assert_eq!(found[1].depth, 1);
        assert_eq!(found[3].partner, Some(1));
        assert_eq!(matched(&found, 12), Some((11, 1)), "just before the caret");
        assert_eq!(matched(&found, 18), Some((18, 20)), "just after the caret");
    }

    #[test]
    fn occurrences_of_a_word_skip_longer_words() {
        let buffer = Buffer::new("let total = total_cost + total;");
        let found = occurrences(&buffer, "total", 0..1);
        assert_eq!(found, [4..9, 25..30]);
    }
}
