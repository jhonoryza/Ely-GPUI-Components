/// A file under the gallery's assets, by name.
macro_rules! asset {
    ($name:literal) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/gallery/assets/",
            $name
        )
    };
}

mod capture;
mod pages;
mod probe;
mod script;
mod shell;
mod swipe;
mod ui;

use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};
use ely_gpui_component::Assets;
use gpui::{
    App, AppContext, Bounds, KeyBinding, Menu, MenuItem, TitlebarOptions, WindowBounds,
    WindowHandle, WindowOptions, actions, point, px, size,
};

actions!(gallery, [Quit, About, LightMode, DarkMode, NewWindow]);

/// The gallery's menu bar and Dock menu: NativeMenu and JumpList at work. Set before the window opens, whose first frame may draw them.
fn menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "Ely".into(),
            items: vec![
                MenuItem::action("About Ely", About),
                MenuItem::separator(),
                MenuItem::action("Quit Ely", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::action("Light", LightMode),
                MenuItem::action("Dark", DarkMode),
            ],
            disabled: false,
        },
    ]);
    cx.set_dock_menu(vec![MenuItem::action("New Window", NewWindow)]);
}

/// What the menus' actions do; the theme ones need the window.
fn menu_actions(window: WindowHandle<shell::Gallery>, cx: &mut App) {
    cx.on_action(|_: &About, cx| pages::open_about(cx));
    cx.on_action(|_: &NewWindow, cx| pages::open_managed(cx));
    cx.on_action(move |_: &LightMode, cx| choose(window, shell::Choice::Light, cx));
    cx.on_action(move |_: &DarkMode, cx| choose(window, shell::Choice::Dark, cx));
}

fn choose(window: WindowHandle<shell::Gallery>, choice: shell::Choice, cx: &mut App) {
    if let Err(error) = window.update(cx, |gallery, window, cx| gallery.choose(choice, window, cx))
    {
        log::error!("gallery: menu could not reach the window: {error:#}");
    }
}

struct Args {
    page: Option<usize>,
    capture: Option<PathBuf>,
    narrow: Option<f32>,
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        page: None,
        capture: None,
        narrow: None,
    };
    let mut iter = std::env::args().skip(1);
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .with_context(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--page" => {
                args.page = Some(pages::find(&value).with_context(|| format!("no page {value}"))?)
            }
            "--capture" => args.capture = Some(PathBuf::from(value)),
            "--narrow" => {
                args.narrow = Some(
                    value
                        .parse()
                        .with_context(|| format!("bad width {value}"))?,
                )
            }
            other => bail!("unknown flag {other}"),
        }
    }
    Ok(args)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = parse_args()?;
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx: &mut App| {
            ely_gpui_component::init(cx);
            #[cfg(debug_assertions)]
            ely_gpui_component::tooling::install_inspector(cx);
            pages::bind_keys(cx);
            pages::install_i18n(cx);
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.on_action(|_: &Quit, cx| cx.quit());
            menus(cx);
            let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Ely".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(16.0), px(18.0))),
                }),
                window_min_size: Some(size(px(880.0), px(560.0))),
                // Keeps drawing while a demo window is key.
                inactive_frame_interval: None,
                ..Default::default()
            };
            let window = cx
                .open_window(options, |window, cx| {
                    cx.new(|cx| {
                        shell::Gallery::new(args.page.unwrap_or(0), args.narrow, window, cx)
                    })
                })
                .expect("gallery window failed to open");
            menu_actions(window, cx);
            cx.activate(true);
            if let Some(dir) = args.capture {
                capture::run(window, dir, args.page, args.narrow.is_none(), cx);
            }
        });
    Ok(())
}
