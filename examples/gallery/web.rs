use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};

use anyhow::{Context as _, Result, anyhow, bail};
use ely_gpui_component::Assets;
use gpui::{Application, AsyncApp, SharedString, WindowHandle};
use gpui_web::{CanvasFontFallback, WebBackendPreference, WebPlatform};
use wasm_bindgen::prelude::*;
use web_sys::UrlSearchParams;

use crate::{Start, choose, launch, pages, shell::Choice, shell::Gallery};

/// The i18n page's Hebrew: gpui_web asks the browser for emoji and CJK alone.
const HEBREW: &[u8] = include_bytes!("fonts/NotoSansHebrew-Regular.ttf");

thread_local! {
    /// The running app and its window, for the host page's calls.
    static HOST: RefCell<Option<(AsyncApp, WindowHandle<Gallery>)>> = const { RefCell::new(None) };
}

fn js(error: JsValue) -> anyhow::Error {
    anyhow!("{error:?}")
}

fn choice(theme: &str) -> Result<Choice> {
    match theme {
        "light" => Ok(Choice::Light),
        "dark" => Ok(Choice::Dark),
        other => bail!("theme is light or dark, not {other}"),
    }
}

/// `?page=<slug>&story=<title or slug>&theme=light|dark`, each optional; a story needs its page.
fn read(search: &str) -> Result<Start> {
    let assets = asset!("");
    if !(assets.starts_with("https://") || assets.starts_with("http://")) || !assets.ends_with('/')
    {
        bail!("ELY_GALLERY_ASSETS is an address ending in /, not {assets}");
    }
    let mut start = Start {
        page: 0,
        story: None,
        choice: Choice::Light,
        narrow: None,
        failure: None,
    };
    let params = UrlSearchParams::new_with_str(search).map_err(js)?;
    for key in js_sys::try_iter(&params.keys())
        .map_err(js)?
        .context("the address's parameters do not iterate")?
    {
        let key = key
            .map_err(js)?
            .as_string()
            .context("a parameter without a name")?;
        let value = params.get(&key).context("a parameter without a value")?;
        match key.as_str() {
            "page" => {
                start.page = pages::find(&value).with_context(|| format!("no page {value}"))?
            }
            "story" => start.story = Some(value.into()),
            "theme" => start.choice = choice(&value)?,
            other => bail!("unknown parameter {other}; the gallery reads page, story and theme"),
        }
    }
    if start.story.is_some() && params.get("page").is_none() {
        bail!("a story needs its page: ?page=<slug>&story=<title>");
    }
    Ok(start)
}

/// The gallery in a browser: one window over the page's canvas.
#[wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    let search = web_sys::window()
        .expect("a browser window")
        .location()
        .search()
        .expect("the page's address");
    let start = read(&search).unwrap_or_else(|error| {
        log::error!("gallery: {search} -> {error:#}");
        Start {
            page: 0,
            story: None,
            choice: Choice::Light,
            narrow: None,
            failure: Some(SharedString::from(format!("{search}: {error:#}"))),
        }
    });
    // One thread: a static site sends no cross-origin isolation, so there is no shared memory.
    let platform = Rc::new(WebPlatform::new_with_backend_and_font_fallback(
        false,
        WebBackendPreference::Auto,
        CanvasFontFallback::EmojiAndCjk,
    ));
    let http = Arc::new(platform.fetch_http_client());
    Application::with_platform(platform)
        .with_http_client(http)
        .with_assets(Assets)
        .run(move |cx| {
            cx.text_system()
                .add_fonts(vec![Cow::Borrowed(HEBREW)])
                .expect("the gallery's Hebrew face registers");
            let window = launch(start, cx);
            HOST.with(|host| *host.borrow_mut() = Some((cx.to_async(), window)));
        });
}

/// Switches Ely's mode from the host page: `light` or `dark`.
#[wasm_bindgen]
pub fn set_theme(theme: &str) -> Result<(), JsError> {
    let choice = choice(theme).map_err(|error| JsError::new(&format!("{error:#}")))?;
    HOST.with(|host| {
        let host = host.borrow();
        let (app, window) = host
            .as_ref()
            .ok_or_else(|| JsError::new("the gallery's window is not open yet"))?;
        app.update(|cx| choose(*window, choice, cx));
        Ok(())
    })
}
