use gpui::{
    AppContext as _, Context, FocusHandle, InteractiveElement, IntoElement, KeyUpEvent, Keystroke,
    ParentElement, Render, TestAppContext, VisualTestContext, Window, div, prelude::*, px, size,
};

use super::{WindowManager, WindowSwitcher};
use crate::{buttons::Button, primitives::FocusScope, theme::Theme};

struct Blank;

impl Render for Blank {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

struct Desk {
    root: FocusHandle,
    open: bool,
    enters_above: usize,
}

impl Render for Desk {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (opener, closer, above) = (cx.entity(), cx.entity(), cx.entity());
        let scope = FocusScope::new(&self.root)
            .size_full()
            .child(Button::new("opener", "Windows").on_click(move |_, _, cx| {
                opener.update(cx, |desk, cx| {
                    desk.open = true;
                    cx.notify();
                })
            }))
            .when(self.open, |desk| {
                desk.child(WindowSwitcher::new("switcher", move |_, cx| {
                    closer.update(cx, |desk, cx| {
                        desk.open = false;
                        cx.notify();
                    })
                }))
            });
        div()
            .size_full()
            .on_key_down(move |event, _, cx| {
                if event.keystroke.key == "enter" {
                    above.update(cx, |desk, _| desk.enters_above += 1);
                }
            })
            .child(scope)
    }
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("a key"),
    });
    settle(cx);
}

#[gpui::test]
fn the_switcher_switches_on_release_so_its_opener_stays_shut(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (desk, cx) = cx.add_window_view(|_, cx| Desk {
        root: cx.focus_handle(),
        open: false,
        enters_above: 0,
    });
    cx.update(|_, cx| {
        WindowManager::open("Notes", size(px(320.0), px(240.0)), cx, |_, cx| {
            cx.new(|_| Blank)
        })
        .expect("a managed window");
    });
    settle(cx);
    cx.update(|window, _| window.focus_next());
    press("enter", cx);
    assert!(desk.read_with(cx, |desk, _| desk.open));
    let above = desk.read_with(cx, |desk, _| desk.enters_above);
    press("enter", cx);
    assert!(!desk.read_with(cx, |desk, _| desk.open));
    assert_eq!(desk.read_with(cx, |desk, _| desk.enters_above), above);
}

struct Platformed;

impl Render for Platformed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .child(
                div()
                    .debug_selector(|| "controls".into())
                    .child(super::WindowControls::new("controls")),
            )
            .child(
                div()
                    .debug_selector(|| "caps".into())
                    .child(crate::typography::Kbd::new("secondary-s")),
            )
    }
}

/// Window buttons and key caps follow the theme's platform: Windows draws three wide caption buttons and spells Ctrl.
#[gpui::test]
fn the_themes_platform_draws_the_buttons_and_the_caps(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    cx.update(|cx| Theme::update(cx, |theme| theme.platform = crate::theme::Platform::Mac));
    let (_, cx) = cx.add_window_view(|_, _| Platformed);
    settle(cx);
    let width = |name: &'static str, cx: &mut VisualTestContext| {
        cx.debug_bounds(name).expect("a measured part").size.width
    };
    let (lights, command) = (width("controls", cx), width("caps", cx));
    cx.update(|_, cx| Theme::update(cx, |theme| theme.platform = crate::theme::Platform::Windows));
    settle(cx);
    assert!(
        width("controls", cx) > lights * 2.0,
        "caption buttons wider than lights"
    );
    assert!(
        width("caps", cx) > command,
        "Ctrl wider than the command mark"
    );
}
