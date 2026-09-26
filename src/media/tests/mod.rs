use std::path::PathBuf;

use gpui::{KeyBinding, KeyUpEvent, Keystroke, TestAppContext, VisualTestContext};

use crate::{primitives::FocusNext, theme::Theme};

mod edit;
mod images;

pub(super) fn setup(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
}

pub(super) fn settle(cx: &mut VisualTestContext) {
    for _ in 0..4 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    cx.run_until_parked();
}

pub(super) fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

/// A picture `w` by `h` written to a file named for its test, the same file each run.
pub(super) fn picture(name: &str, w: u32, h: u32) -> PathBuf {
    let path = std::env::temp_dir().join(format!("ely-media-{name}.png"));
    image::RgbaImage::from_pixel(w, h, image::Rgba([90, 120, 150, 255]))
        .save(&path)
        .expect("the picture writes");
    path
}
