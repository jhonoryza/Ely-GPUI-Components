use gpui::{
    AppContext as _, Context, Entity, IntoElement, KeyUpEvent, Keystroke, ParentElement, Render,
    TestAppContext, VisualTestContext, Window, div,
};

use super::setup;
use crate::{
    buttons::Button,
    forms::{FieldArray, Form, FormField, Input, InputEvent, TextInput},
};

struct Signup {
    name: Entity<TextInput>,
    sent: usize,
    field_submits: usize,
}

impl Render for Signup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        Form::new("signup")
            .on_submit(move |_, cx| {
                view.update(cx, |view, cx| {
                    view.sent += 1;
                    cx.notify();
                })
            })
            .child(
                FormField::new("signup-name", "Name")
                    .required()
                    .child(Input::new(&self.name)),
            )
            .child(Button::new("signup-save", "Save"))
    }
}

fn signup(cx: &mut TestAppContext) -> (Entity<Signup>, &mut VisualTestContext) {
    setup(cx);
    cx.add_window_view(|window, cx| {
        let name = cx.new(|cx| TextInput::new(window, cx));
        cx.subscribe(&name, |view: &mut Signup, _, event, _| {
            if *event == InputEvent::Submit {
                view.field_submits += 1;
            }
        })
        .detach();
        Signup {
            name,
            sent: 0,
            field_submits: 0,
        }
    })
}

#[gpui::test]
fn cmd_enter_in_a_field_submits_the_form_once(cx: &mut TestAppContext) {
    let (view, cx) = signup(cx);
    cx.update(|window, _| window.focus_next());
    cx.simulate_input("Ada");
    cx.simulate_keystrokes("cmd-enter");
    let counts = view.read_with(cx, |view, _| (view.sent, view.field_submits));
    assert_eq!(counts, (1, 0));
}

#[gpui::test]
fn cmd_enter_on_any_control_in_a_form_submits_it(cx: &mut TestAppContext) {
    let (view, cx) = signup(cx);
    cx.update(|window, _| {
        window.focus_next();
        window.focus_next();
    });
    cx.simulate_keystrokes("cmd-enter");
    assert_eq!(
        view.read_with(cx, |view, _| view.sent),
        1,
        "the button holds focus"
    );
}

struct Rows {
    rows: Vec<&'static str>,
}

impl Render for Rows {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (grow, shrink) = (cx.entity(), cx.entity());
        self.rows.iter().fold(
            FieldArray::new("rows")
                .on_add(move |_, cx| {
                    grow.update(cx, |view, cx| {
                        view.rows.push("new");
                        cx.notify();
                    })
                })
                .on_remove(move |ix, _, cx| {
                    shrink.update(cx, |view, cx| {
                        view.rows.remove(ix);
                        cx.notify();
                    })
                }),
            |array, row| array.row(div().child(*row)),
        )
    }
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).unwrap(),
    });
}

#[gpui::test]
fn field_array_buttons_remove_their_own_row_and_add_one(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Rows {
        rows: vec!["a", "b", "c"],
    });
    cx.update(|window, _| {
        window.focus_next();
        window.focus_next();
    });
    press("space", cx);
    assert_eq!(view.read_with(cx, |view, _| view.rows.clone()), ["a", "c"]);
    cx.update(|window, _| {
        window.blur();
        for _ in 0..3 {
            window.focus_next();
        }
    });
    press("space", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.rows.clone()),
        ["a", "c", "new"]
    );
}
