use std::f32::consts::FRAC_PI_4;

use ely_gpui_component::{
    Assets,
    files::FileIcon,
    primitives::{FocusScope, Icon, IconName, IconTheme},
    theme::{ActiveTheme, IconSize},
};
use gpui::{
    App, AppContext, Context, FocusHandle, IntoElement, ParentElement, Radians, Render, Styled,
    Window, WindowOptions, div,
};

struct Icons {
    focus: FocusHandle,
}

impl Render for Icons {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let file = |name: &'static str| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(FileIcon::file(name))
                .child(name)
        };
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(Icon::new(IconName::Check))
                    .child(Icon::new(IconName::Rocket).size(IconSize::Xl))
                    .child(Icon::new(IconName::CircleCheck).color(theme.colors.success))
                    .child(Icon::new(IconName::Package).rotate(Radians(FRAC_PI_4))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(file("main.rs"))
                    .child(file("Cargo.toml"))
                    .child(file("notes.txt")),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx);
            IconTheme::default()
                .name("Cargo.toml", IconName::Package)
                .apply(cx);
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Icons { focus }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
