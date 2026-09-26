use std::{path::PathBuf, time::Duration};

use gpui::{
    Context, Entity, IntoElement, Modifiers, MouseButton, ParentElement, Pixels, Point, Render,
    Styled, TestAppContext, VisualTestContext, Window, div, point, px,
};

use super::{picture, press, settle, setup};
use crate::media::{Cue, Scrubber, SubtitleEditor, VideoPlayer, VideoThumbnailStrip};

const SECOND: Duration = Duration::from_secs(1);

/// Where a share of a box sits in the window, halfway down.
fn across(selector: &'static str, share: f32, cx: &mut VisualTestContext) -> Point<Pixels> {
    let bounds = cx.debug_bounds(selector).expect("the box draws");
    point(bounds.left() + bounds.size.width * share, bounds.center().y)
}

/// A scrubber over a hundred seconds, 400 wide, and the times it was sent to.
struct Scrubbing {
    at: Duration,
    sought: Vec<Duration>,
}

impl Render for Scrubbing {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        div().w(px(400.0)).mt(px(40.0)).child(
            Scrubber::new("scrub", SECOND * 100, self.at)
                .chapter(Duration::ZERO, "Intro")
                .chapter(SECOND * 40, "Stair")
                .on_seek(move |time, _, cx| {
                    owner.update(cx, |view, cx| {
                        (view.at, view.sought) = (time, [view.sought.clone(), vec![time]].concat());
                        cx.notify();
                    })
                }),
        )
    }
}

fn scrubbing(cx: &mut TestAppContext) -> (Entity<Scrubbing>, &mut VisualTestContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| Scrubbing {
        at: Duration::ZERO,
        sought: Vec::new(),
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (view, cx)
}

#[gpui::test]
fn a_press_seeks_and_a_drag_past_the_end_stops_there(cx: &mut TestAppContext) {
    let (view, cx) = scrubbing(cx);
    let quarter = across("scrubber-rail", 0.25, cx);
    let none = Modifiers::none();
    cx.simulate_mouse_down(quarter, MouseButton::Left, none);
    cx.simulate_mouse_move(quarter + point(px(20.0), px(0.0)), MouseButton::Left, none);
    cx.simulate_mouse_move(quarter + point(px(900.0), px(0.0)), MouseButton::Left, none);
    cx.simulate_mouse_up(quarter + point(px(900.0), px(0.0)), MouseButton::Left, none);
    settle(cx);
    let sought = view.read_with(cx, |view, _| view.sought.clone());
    let first = sought.first().expect("a press seeks");
    assert!(first.abs_diff(SECOND * 25) < SECOND, "{sought:?}");
    assert_eq!(sought.last(), Some(&(SECOND * 100)), "{sought:?}");
}

#[gpui::test]
fn the_arrows_step_five_seconds_and_end_goes_to_the_end(cx: &mut TestAppContext) {
    let (view, cx) = scrubbing(cx);
    let tenth = across("scrubber-rail", 0.1, cx);
    cx.simulate_click(tenth, Modifiers::none());
    settle(cx);
    let start = view.read_with(cx, |view, _| view.at);
    press("right", cx);
    assert_eq!(view.read_with(cx, |view, _| view.at), start + SECOND * 5);
    press("end", cx);
    assert_eq!(view.read_with(cx, |view, _| view.at), SECOND * 100);
}

#[gpui::test]
fn the_pointer_shows_its_time_over_the_track(cx: &mut TestAppContext) {
    let (_, cx) = scrubbing(cx);
    assert!(cx.debug_bounds("scrubber-tip").is_none());
    let half = across("scrubber-rail", 0.5, cx);
    cx.simulate_mouse_move(half, None, Modifiers::none());
    settle(cx);
    let tip = cx
        .debug_bounds("scrubber-tip")
        .expect("a tip over the track");
    assert!((tip.left() - half.x).abs() < px(1.0), "{tip:?}");
}

/// A player of a hundred seconds, what it was told, and whether it plays.
struct Watching {
    path: PathBuf,
    at: Duration,
    playing: bool,
    captions: bool,
}

impl Render for Watching {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (play, seek, caption) = (cx.entity(), cx.entity(), cx.entity());
        div().w(px(480.0)).child(
            VideoPlayer::new(
                "player",
                Some(self.path.clone().into()),
                1.6,
                SECOND * 100,
                self.at,
            )
            .playing(self.playing)
            .captions(self.captions, Some("Light falls on the stair".into()))
            .on_play(move |on, _, cx| {
                play.update(cx, |view, cx| {
                    view.playing = on;
                    cx.notify();
                })
            })
            .on_seek(move |time, _, cx| {
                seek.update(cx, |view, cx| {
                    view.at = time;
                    cx.notify();
                })
            })
            .on_captions(move |on, _, cx| {
                caption.update(cx, |view, cx| {
                    view.captions = on;
                    cx.notify();
                })
            }),
        )
    }
}

fn watching(cx: &mut TestAppContext) -> (Entity<Watching>, &mut VisualTestContext) {
    setup(cx);
    let path = picture("frame", 160, 100);
    let (view, cx) = cx.add_window_view(|_, _| Watching {
        path,
        at: SECOND * 30,
        playing: false,
        captions: false,
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    let frame = cx.debug_bounds("video-player").expect("the player draws");
    cx.simulate_click(frame.origin + point(px(8.0), px(8.0)), Modifiers::none());
    settle(cx);
    (view, cx)
}

#[gpui::test]
fn space_plays_l_and_j_jump_ten_seconds_and_c_turns_captions(cx: &mut TestAppContext) {
    let (view, cx) = watching(cx);
    press("space", cx);
    assert!(view.read_with(cx, |view, _| view.playing));
    press("l", cx);
    press("l", cx);
    press("j", cx);
    assert_eq!(view.read_with(cx, |view, _| view.at), SECOND * 40);
    press("c", cx);
    assert!(view.read_with(cx, |view, _| view.captions));
}

#[gpui::test]
fn controls_rest_once_the_pointer_is_still_while_it_plays(cx: &mut TestAppContext) {
    let (view, cx) = watching(cx);
    cx.update(|window, _| window.blur());
    view.update(cx, |view, cx| {
        view.playing = true;
        cx.notify();
    });
    let frame = cx.debug_bounds("video-player").expect("the player draws");
    cx.simulate_mouse_move(frame.center(), None, Modifiers::none());
    settle(cx);
    assert!(
        cx.debug_bounds("video-controls-resting").is_none(),
        "a moving pointer keeps them up"
    );
    cx.executor().advance_clock(Duration::from_millis(2600));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("video-controls-resting").is_some(),
        "a still one lets them rest"
    );
}

/// A strip of four frames over a hundred seconds with a trim, and what it was told.
struct Trimming {
    path: PathBuf,
    at: Duration,
    trim: (Duration, Duration),
}

impl Render for Trimming {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (seek, cut) = (cx.entity(), cx.entity());
        let frames: Vec<PathBuf> = (0..4).map(|_| self.path.clone()).collect();
        div().w(px(400.0)).child(
            VideoThumbnailStrip::new("strip", frames, SECOND * 100, self.at)
                .trim(self.trim.0, self.trim.1)
                .on_seek(move |time, _, cx| {
                    seek.update(cx, |view, cx| {
                        view.at = time;
                        cx.notify();
                    })
                })
                .on_trim(move |trim, _, cx| {
                    cut.update(cx, |view, cx| {
                        view.trim = trim;
                        cx.notify();
                    })
                }),
        )
    }
}

fn trimming(cx: &mut TestAppContext) -> (Entity<Trimming>, &mut VisualTestContext) {
    setup(cx);
    let path = picture("strip", 160, 100);
    let (view, cx) = cx.add_window_view(|_, _| Trimming {
        path,
        at: Duration::ZERO,
        trim: (SECOND * 10, SECOND * 50),
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (view, cx)
}

#[gpui::test]
fn a_press_seeks_and_a_handle_drag_moves_its_end_to_the_length(cx: &mut TestAppContext) {
    let (view, cx) = trimming(cx);
    let press_at = across("video-strip", 0.3, cx);
    cx.simulate_click(press_at, Modifiers::none());
    settle(cx);
    let at = view.read_with(cx, |view, _| view.at);
    assert!(at.abs_diff(SECOND * 30) < SECOND, "{at:?}");
    let out = across("video-strip", 0.5, cx);
    let none = Modifiers::none();
    cx.simulate_mouse_down(out, MouseButton::Left, none);
    cx.simulate_mouse_move(out + point(px(10.0), px(0.0)), MouseButton::Left, none);
    cx.simulate_mouse_move(out + point(px(900.0), px(0.0)), MouseButton::Left, none);
    cx.simulate_mouse_up(out + point(px(900.0), px(0.0)), MouseButton::Left, none);
    settle(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.trim),
        (SECOND * 10, SECOND * 100)
    );
}

#[gpui::test]
fn i_and_o_set_the_trim_at_the_playhead(cx: &mut TestAppContext) {
    let (view, cx) = trimming(cx);
    let press_at = across("video-strip", 0.3, cx);
    cx.simulate_click(press_at, Modifiers::none());
    settle(cx);
    let at = view.read_with(cx, |view, _| view.at);
    press("i", cx);
    assert_eq!(view.read_with(cx, |view, _| view.trim), (at, SECOND * 50));
    press("right", cx);
    press("o", cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.trim),
        (at, at + SECOND * 25)
    );
}

/// A subtitle editor over two cues, its playhead, and the cues it sent.
struct Captioning {
    cues: Vec<Cue>,
    at: Duration,
}

impl Render for Captioning {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        div().w(px(420.0)).child(
            SubtitleEditor::new("subs", self.cues.clone(), self.at).on_change(
                move |cues, _, cx| {
                    owner.update(cx, |view, cx| {
                        view.cues = cues.to_vec();
                        cx.notify();
                    })
                },
            ),
        )
    }
}

fn captioning(cx: &mut TestAppContext) -> (Entity<Captioning>, &mut VisualTestContext) {
    setup(cx);
    let cue = |key: &str, start: u64, end: u64| Cue {
        key: key.to_string().into(),
        start: SECOND * start as u32,
        end: SECOND * end as u32,
        text: key.to_string().into(),
    };
    let (view, cx) = cx.add_window_view(|_, _| Captioning {
        cues: vec![cue("a", 1, 3), cue("b", 5, 8)],
        at: SECOND * 6,
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (view, cx)
}

/// Tabs to the `nth` stop and types over it.
fn retype(nth: usize, text: &str, cx: &mut VisualTestContext) {
    cx.update(|window, _| {
        window.blur();
        for _ in 0..nth {
            window.focus_next();
        }
    });
    press("enter", cx);
    cx.simulate_input(text);
    press("enter", cx);
}

#[gpui::test]
fn a_start_typed_in_place_moves_its_cue_into_order(cx: &mut TestAppContext) {
    let (view, cx) = captioning(cx);
    retype(5, "00:00.500", cx);
    let keys: Vec<String> = view.read_with(cx, |view, _| {
        view.cues.iter().map(|cue| cue.key.to_string()).collect()
    });
    assert_eq!(keys, ["b", "a"]);
}

#[gpui::test]
fn a_time_that_does_not_read_keeps_the_cue_and_says_why(cx: &mut TestAppContext) {
    let (view, cx) = captioning(cx);
    let before = view.read_with(cx, |view, _| view.cues.clone());
    retype(1, "1:x", cx);
    assert_eq!(view.read_with(cx, |view, _| view.cues.clone()), before);
    assert!(cx.debug_bounds("subtitle-complaint").is_some());
}
