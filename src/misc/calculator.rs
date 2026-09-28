use gpui::{
    App, ClickEvent, ElementId, Entity, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Styled, Window, div, prelude::*,
};

use crate::{
    buttons::{Button, ButtonVariant},
    forms::evaluate,
    primitives::{FocusRing, IconName, tab_stop},
    theme::{ActiveTheme, ControlSize, Radius, TextSize},
    typography::{
        format::{MINUS, Separators, group_digits, significant},
        tabular,
    },
};

/// A key on the pad or the keyboard.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Key {
    Digit(char),
    Point,
    Op(char),
    Sign,
    Back,
    Clear,
    Equals,
}

/// The pad, four keys a row, each with what it shows; zero takes two.
const PAD: [(Key, &str); 19] = [
    (Key::Clear, "AC"),
    (Key::Sign, "±"),
    (Key::Back, "Delete"),
    (Key::Op('/'), "÷"),
    (Key::Digit('7'), "7"),
    (Key::Digit('8'), "8"),
    (Key::Digit('9'), "9"),
    (Key::Op('*'), "×"),
    (Key::Digit('4'), "4"),
    (Key::Digit('5'), "5"),
    (Key::Digit('6'), "6"),
    (Key::Op('-'), "−"),
    (Key::Digit('1'), "1"),
    (Key::Digit('2'), "2"),
    (Key::Digit('3'), "3"),
    (Key::Op('+'), "+"),
    (Key::Digit('0'), "0"),
    (Key::Point, "."),
    (Key::Equals, "="),
];

const OPS: [char; 4] = ['+', '-', '*', '/'];

/// The entry as typed, in ASCII; the sum that gave it while it is a result; and why that sum failed.
#[derive(Default)]
struct Tape {
    entry: String,
    worked: Option<String>,
    problem: Option<String>,
}

impl Tape {
    /// Where the number being typed starts, its sign included.
    fn number_start(&self) -> usize {
        let bytes = self.entry.as_bytes();
        let start = self.entry.rfind(OPS).map_or(0, |at| at + 1);
        let signed = start > 0
            && bytes[start - 1] == b'-'
            && (start == 1 || OPS.contains(&(bytes[start - 2] as char)));
        if signed { start - 1 } else { start }
    }

    fn blank(&self) -> bool {
        self.entry.is_empty() && self.worked.is_none() && self.problem.is_none()
    }

    fn press(&mut self, key: Key) {
        let failed = self.problem.take().is_some();
        let result = self.worked.take().is_some() && !failed;
        if failed || (result && matches!(key, Key::Digit(_) | Key::Point)) {
            self.entry.clear();
        }
        let start = self.number_start();
        let number = &self.entry[start..];
        match key {
            Key::Digit(digit) => {
                if number.trim_start_matches('-') == "0" {
                    self.entry.pop();
                }
                self.entry.push(digit);
            }
            Key::Point if number.contains('.') => {}
            Key::Point => {
                if number.trim_start_matches('-').is_empty() {
                    self.entry.push('0');
                }
                self.entry.push('.');
            }
            Key::Op(op) => {
                let kept = self.entry.trim_end_matches(OPS).len();
                match kept {
                    0 if self.entry.is_empty() && op == '-' => self.entry.push('-'),
                    0 => {}
                    _ => {
                        self.entry.truncate(kept);
                        self.entry.push(op);
                    }
                }
            }
            Key::Sign if number.starts_with('-') => {
                self.entry.remove(start);
            }
            Key::Sign if !number.is_empty() => self.entry.insert(start, '-'),
            Key::Sign => {}
            Key::Back => {
                self.entry.pop();
            }
            Key::Clear => self.entry.clear(),
            Key::Equals if self.entry.is_empty() => {}
            Key::Equals => match evaluate(&self.entry, &[]) {
                Ok(value) => self.worked = Some(std::mem::replace(&mut self.entry, plain(value))),
                Err(problem) => {
                    self.worked = Some(self.entry.clone());
                    self.problem = Some(problem);
                }
            },
        }
    }
}

/// A result as the entry types it: twelve significant digits, no groups, an ASCII minus.
fn plain(value: f64) -> String {
    significant(value, 12, Separators::EN)
        .chars()
        .filter(|ch| *ch != ',')
        .map(|ch| if ch == MINUS { '-' } else { ch })
        .collect()
}

/// The entry as it reads: digits grouped, and ×, ÷ and a true minus between spaced terms.
fn shown(entry: &str) -> String {
    let (mut out, mut number, mut after_op) = (String::new(), String::new(), true);
    let flush = |out: &mut String, number: &mut String| {
        let (whole, part) = number.split_once('.').unwrap_or((number, ""));
        out.push_str(&group_digits(whole, ','));
        if number.contains('.') {
            out.push('.');
            out.push_str(part);
        }
        number.clear();
    };
    for ch in entry.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
            after_op = false;
            continue;
        }
        flush(&mut out, &mut number);
        match (after_op, ch) {
            (true, _) => out.push(MINUS),
            (_, '+') => out.push_str(" + "),
            (_, '-') => out.push_str(" − "),
            (_, '*') => out.push_str(" × "),
            _ => out.push_str(" ÷ "),
        }
        after_op = true;
    }
    flush(&mut out, &mut number);
    out
}

fn hit(tape: &Entity<Tape>, key: Key, cx: &mut App) {
    tape.update(cx, |tape, cx| {
        tape.press(key);
        log::info!("calculator: {key:?}, entry {:?}", tape.entry);
        cx.notify();
    });
}

/// A key typed on the keyboard; Enter is equals only while the calculator itself has focus, so a focused key presses on its own.
fn typed(event: &KeyDownEvent, own: bool) -> Option<Key> {
    let stroke = &event.keystroke;
    if stroke.modifiers.platform || stroke.modifiers.control {
        return None;
    }
    match (stroke.key.as_str(), stroke.key_char.as_deref()) {
        ("enter", _) if own => Some(Key::Equals),
        ("backspace", _) => Some(Key::Back),
        ("escape", _) => Some(Key::Clear),
        (_, Some("=")) => Some(Key::Equals),
        (_, Some(".")) => Some(Key::Point),
        (_, Some(op @ ("+" | "-" | "*" | "/"))) => op.chars().next().map(Key::Op),
        (_, Some(digit)) if digit.len() == 1 && digit.as_bytes()[0].is_ascii_digit() => {
            digit.chars().next().map(Key::Digit)
        }
        _ => None,
    }
}

/// A calculator: the sum as you type it over its result, and a pad of digits, the four operations, sign, delete and clear. Keys type too: digits, + − * /, = or Enter, Backspace and Escape. It works in precedence, × and ÷ before + and −, and shows twelve significant digits.
#[derive(IntoElement)]
pub struct Calculator {
    id: ElementId,
}

impl Calculator {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for Calculator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let tape = window.use_keyed_state((id.clone(), "tape"), cx, |_, _| Tape::default());
        let focus = tab_stop((id.clone(), "focus").into(), true, window, cx);
        let theme = cx.theme();
        let colors = &theme.colors;
        let (above, main, failed) = {
            let tape = tape.read(cx);
            let main = match (&tape.problem, tape.entry.is_empty()) {
                (Some(problem), _) => problem.clone(),
                (None, true) => "0".to_string(),
                (None, false) => shown(&tape.entry),
            };
            let above = tape
                .worked
                .as_deref()
                .map(|sum| format!("{} =", shown(sum)));
            (above, main, tape.problem.is_some())
        };
        let keys = PAD.iter().map(|&(key, label)| {
            let button = match key {
                Key::Back => Button::new((id.clone(), "back"), "").icon(IconName::Delete),
                _ => Button::new((id.clone(), label), label),
            };
            let tape = tape.clone();
            div()
                .debug_selector(|| format!("key-{label}"))
                .when(key == Key::Digit('0'), |cell| cell.col_span(2))
                .child(
                    button
                        .size(ControlSize::Lg)
                        .variant(match key {
                            Key::Equals => ButtonVariant::Primary,
                            Key::Digit(_) | Key::Point => ButtonVariant::Secondary,
                            Key::Op(_) => ButtonVariant::Outline,
                            _ => ButtonVariant::Ghost,
                        })
                        .full_width()
                        .on_click(move |_: &ClickEvent, _, cx| hit(&tape, key, cx)),
                )
        });
        div()
            .id(id.clone())
            .track_focus(&focus)
            .p_3()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .focus_ring(cx)
            .on_mouse_down(MouseButton::Left, {
                let focus = focus.clone();
                move |_, window, _| window.focus(&focus)
            })
            .on_key_down({
                let tape = tape.clone();
                move |event, window, cx| {
                    let Some(key) = typed(event, focus.is_focused(window)) else {
                        return;
                    };
                    if key == Key::Clear && tape.read(cx).blank() {
                        return;
                    }
                    cx.stop_propagation();
                    hit(&tape, key, cx);
                }
            })
            .child(
                div()
                    .pb_3()
                    .text_right()
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Sm))
                            .text_color(colors.fg_muted)
                            .child(above.unwrap_or_else(|| "\u{a0}".into())),
                    )
                    .child(
                        tabular(div())
                            .debug_selector(|| format!("calculator-{main}"))
                            .text_size(theme.text_size(TextSize::Xxl))
                            .text_color(if failed { colors.danger } else { colors.fg })
                            .child(main.clone()),
                    ),
            )
            .child(div().grid().grid_cols(4).gap_2().children(keys))
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Tape, plain, shown};

    fn typed(keys: &[Key]) -> Tape {
        let mut tape = Tape::default();
        keys.iter().for_each(|key| tape.press(*key));
        tape
    }

    fn sum(text: &str) -> Vec<Key> {
        text.chars()
            .map(|ch| match ch {
                '0'..='9' => Key::Digit(ch),
                '.' => Key::Point,
                '=' => Key::Equals,
                '~' => Key::Sign,
                '<' => Key::Back,
                'C' => Key::Clear,
                op => Key::Op(op),
            })
            .collect()
    }

    #[test]
    fn it_works_in_precedence_and_carries_the_result_on() {
        let tape = typed(&sum("12+3*4="));
        assert_eq!(
            (tape.entry.as_str(), tape.worked.as_deref()),
            ("24", Some("12+3*4"))
        );
        assert_eq!(
            typed(&sum("12+3*4=-4=")).entry,
            "20",
            "an operator goes on from the result"
        );
        assert_eq!(typed(&sum("12+3*4=7")).entry, "7", "a digit starts afresh");
        assert_eq!(
            typed(&sum(".1+.2=")).entry,
            "0.3",
            "twelve digits hide binary noise"
        );
        assert_eq!(typed(&sum("2/3=")).entry, "0.666666666667");
    }

    #[test]
    fn keys_keep_the_entry_a_sum() {
        assert_eq!(typed(&sum("007")).entry, "7", "no leading zeros");
        assert_eq!(typed(&sum("1..5.")).entry, "1.5", "one point a number");
        assert_eq!(
            typed(&sum("5+*")).entry,
            "5*",
            "an operator replaces the last"
        );
        assert_eq!(typed(&sum("+-")).entry, "-", "only minus starts, as a sign");
        assert_eq!(
            typed(&sum("5*3~")).entry,
            "5*-3",
            "sign turns the number typed"
        );
        assert_eq!(typed(&sum("5*3~~")).entry, "5*3");
        assert_eq!(
            typed(&sum("5*3~<+")).entry,
            "5+",
            "a dangling sign goes with the operator"
        );
        assert_eq!(typed(&sum("12+3C")).entry, "");
    }

    #[test]
    fn a_failed_sum_says_why_and_the_next_key_starts_afresh() {
        let tape = typed(&sum("5/0="));
        assert_eq!(tape.problem.as_deref(), Some("division by zero"));
        assert_eq!(typed(&sum("5/0=8")).entry, "8");
        assert_eq!(typed(&sum("5/0=+")).entry, "", "no result to go on from");
    }

    #[test]
    fn the_entry_reads_grouped_with_true_signs() {
        assert_eq!(shown("-1234.5*-6/7-8"), "−1,234.5 × −6 ÷ 7 − 8");
        assert_eq!(shown("0."), "0.");
        assert_eq!(plain(-1_234_567.0), "-1234567");
        assert_eq!(plain(1e-15), "0.000000000000001");
    }
}
