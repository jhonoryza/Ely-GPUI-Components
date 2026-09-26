use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    Context, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext, Window, div,
    point, px,
};

use super::{press, settle, setup, tab};
use crate::generative::{
    AudioGenerationPlayer, Outcome, Shot, TTSVoicePicker, VideoGenerationTimeline, Voice,
};

type Heard = Rc<RefCell<Vec<String>>>;

/// A made sound a quarter played and one still being made, and what they heard.
struct Players(Heard);

impl Render for Players {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (sought, made, making) = (self.0.clone(), self.0.clone(), self.0.clone());
        let length = Duration::from_secs(40);
        div()
            .w(px(420.0))
            .child(
                AudioGenerationPlayer::new("made", "rain", "Orchid", vec![0.5; 8], length)
                    .played(Duration::from_secs(10), false)
                    .on_toggle(|_, _, _| {})
                    .on_seek(move |share, _, _| {
                        sought.borrow_mut().push(format!("seek {share:.2}"))
                    })
                    .on_regenerate(move |_, _| made.borrow_mut().push("regenerate made".into())),
            )
            .child(
                AudioGenerationPlayer::new("making", "cello", "Orchid", vec![0.5; 8], length)
                    .making(0.4)
                    .on_regenerate(move |_, _| {
                        making.borrow_mut().push("regenerate making".into())
                    }),
            )
    }
}

#[gpui::test]
fn the_waveform_seeks_with_keys_and_a_sound_being_made_waits(cx: &mut TestAppContext) {
    setup(cx);
    let heard = Heard::default();
    let store = heard.clone();
    let (_, cx) = cx.add_window_view(|_, _| Players(store));
    cx.update(|window, _| window.activate_window());
    settle(cx);
    tab(1, cx);
    press("enter", cx);
    tab(2, cx);
    press("right", cx);
    tab(1, cx);
    press("enter", cx);
    let heard = heard.borrow().clone();
    assert_eq!(
        heard,
        ["regenerate made", "seek 0.30", "regenerate made"],
        "Tab passes the waiting button"
    );
}

/// Two voices, the first chosen, and what they heard.
struct Voices(Heard);

impl Render for Voices {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (chosen, played) = (self.0.clone(), self.0.clone());
        let voice = |key: &str| Voice {
            key: key.to_string().into(),
            name: key.to_string().into(),
            about: "a voice".into(),
            tags: vec!["English".into()],
        };
        div().w(px(420.0)).child(
            TTSVoicePicker::new("voices", [voice("iris"), voice("theo")])
                .selected("iris")
                .on_select(move |key, _, _| chosen.borrow_mut().push(format!("choose {key}")))
                .on_play(move |key, _, _| played.borrow_mut().push(format!("play {key:?}"))),
        )
    }
}

#[gpui::test]
fn a_sample_plays_without_choosing_its_voice(cx: &mut TestAppContext) {
    setup(cx);
    let heard = Heard::default();
    let store = heard.clone();
    let (_, cx) = cx.add_window_view(|_, _| Voices(store));
    cx.update(|window, _| window.activate_window());
    settle(cx);
    tab(3, cx);
    press("enter", cx);
    cx.simulate_click(point(px(23.0), px(83.0)), Modifiers::none());
    settle(cx);
    assert_eq!(
        *heard.borrow(),
        ["choose theo", "play Some(\"theo\")"],
        "a sample's press stays off its row"
    );
}

/// Two shots, the first chosen, and what they heard.
struct Timeline(Heard);

impl Render for Timeline {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (chosen, extended) = (self.0.clone(), self.0.clone());
        let shot = |key: &str, outcome| Shot {
            key: key.to_string().into(),
            prompt: "a pan".into(),
            length: Duration::from_secs(4),
            outcome,
        };
        div().w(px(420.0)).child(
            VideoGenerationTimeline::new(
                "timeline",
                [
                    shot("a", Outcome::Done("missing.jpg".into())),
                    shot("b", Outcome::Pending(None)),
                ],
                Duration::ZERO,
            )
            .selected("a")
            .on_select(move |key, _, _| chosen.borrow_mut().push(format!("choose {key}")))
            .on_extend(move |_, _| extended.borrow_mut().push("extend".into())),
        )
    }
}

#[gpui::test]
fn a_shot_is_chosen_once_and_extend_asks_for_the_next(cx: &mut TestAppContext) {
    setup(cx);
    let heard = Heard::default();
    let store = heard.clone();
    let (_, cx) = cx.add_window_view(|_, _| Timeline(store));
    cx.update(|window, _| window.activate_window());
    settle(cx);
    for _ in 0..3 {
        tab(1, cx);
        press("enter", cx);
    }
    assert_eq!(*heard.borrow(), ["choose b", "extend"]);
}

/// A voice list and a timeline shown to read, with nothing to take a choice.
struct Shown;

impl Render for Shown {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let voice = Voice {
            key: "iris".into(),
            name: "Iris".into(),
            about: "a voice".into(),
            tags: Vec::new(),
        };
        let shot = Shot {
            key: "a".into(),
            prompt: "a pan".into(),
            length: Duration::from_secs(4),
            outcome: Outcome::Pending(None),
        };
        div()
            .w(px(420.0))
            .child(TTSVoicePicker::new("voices", [voice]).selected("iris"))
            .child(VideoGenerationTimeline::new("timeline", [shot], Duration::ZERO).selected("a"))
    }
}

#[gpui::test]
fn lists_shown_to_read_take_no_focus(cx: &mut TestAppContext) {
    setup(cx);
    let (_, cx) = cx.add_window_view(|_, _| Shown);
    cx.update(|window, _| window.activate_window());
    settle(cx);
    tab(1, cx);
    assert!(cx.update(|window, cx| window.focused(cx).is_none()));
}

#[test]
#[should_panic(expected = "a sound's peaks lie within 0 to 1")]
fn a_peak_past_full_fails_loud() {
    let _ =
        AudioGenerationPlayer::new("sound", "rain", "Orchid", vec![1.5], Duration::from_secs(1));
}
