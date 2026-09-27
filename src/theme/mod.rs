mod calendar;
mod canvas;
mod chart;
mod dashboard;
mod files;
mod generative;
mod interaction;
mod media;
mod messaging;
mod onboarding;
mod palette;
mod project;
mod tokens;

#[cfg(all(test, feature = "test-support"))]
mod tests;

use std::time::{Duration, Instant};

use gpui::{App, Global, SharedString, WindowAppearance};

use crate::motion;
pub use calendar::CalendarSizes;
pub use canvas::CanvasSizes;
pub use chart::ChartSizes;
pub use dashboard::DashboardSizes;
pub use files::FileSizes;
pub use generative::GenerativeSizes;
pub use interaction::InteractionSizes;
pub use media::MediaSizes;
pub use messaging::MessagingSizes;
pub use onboarding::OnboardingSizes;
pub use palette::{HUE_NAMES, Mix, Palette, Syntax};
pub use project::ProjectSizes;
pub use tokens::{
    AvatarSize, ContainerSize, ControlSize, Density, Elevation, IconSize, Radius, TextSize,
};

const FRAME: Duration = Duration::from_millis(8);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Light,
    Dark,
}

impl From<WindowAppearance> for Mode {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
        }
    }
}

/// Design tokens. One global per app.
pub struct Theme {
    mode: Mode,
    high_contrast: bool,
    transition: u64,
    custom: [Option<Palette>; 2],
    pub colors: Palette,
    pub density: Density,
    pub radius_scale: f32,
    pub font_scale: f32,
    pub font_family: SharedString,
    pub mono_family: SharedString,
    pub reduced_motion: bool,
}

impl Global for Theme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

impl Theme {
    pub(crate) fn init(cx: &mut App) {
        cx.set_global(Theme {
            mode: Mode::Light,
            high_contrast: false,
            transition: 0,
            custom: [None, None],
            colors: Palette::light(false),
            density: Density::Standard,
            radius_scale: 1.0,
            font_scale: 1.0,
            font_family: "Inter".into(),
            mono_family: "JetBrains Mono".into(),
            reduced_motion: false,
        });
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn high_contrast(&self) -> bool {
        self.high_contrast
    }

    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }

    /// Cross-fades the palette to `mode`.
    pub fn set_mode(mode: Mode, cx: &mut App) {
        log::info!("theme: mode -> {mode:?}");
        cx.global_mut::<Theme>().mode = mode;
        Self::fade_to_target(cx);
    }

    pub fn set_high_contrast(on: bool, cx: &mut App) {
        log::info!("theme: high contrast -> {on}");
        cx.global_mut::<Theme>().high_contrast = on;
        Self::fade_to_target(cx);
    }

    /// The owner's palette for `mode`, as an edited or imported theme gives, or none to bring Ely's back. The colors fade to it when `mode` shows. High contrast strengthens Ely's own palettes alone.
    pub fn set_palette(mode: Mode, palette: Option<Palette>, cx: &mut App) {
        log::info!(
            "theme: {mode:?} palette -> {}",
            if palette.is_some() {
                "the owner's"
            } else {
                "Ely's"
            }
        );
        cx.global_mut::<Theme>().custom[mode as usize] = palette;
        Self::fade_to_target(cx);
    }

    /// The palette the shown mode fades to: the owner's, or Ely's.
    pub fn palette(&self) -> Palette {
        self.target()
    }

    /// Edits non-color tokens, then repaints.
    pub fn update(cx: &mut App, edit: impl FnOnce(&mut Theme)) {
        edit(cx.global_mut::<Theme>());
        cx.refresh_windows();
    }

    fn target(&self) -> Palette {
        if let Some(palette) = &self.custom[self.mode as usize] {
            return palette.clone();
        }
        match self.mode {
            Mode::Light => Palette::light(self.high_contrast),
            Mode::Dark => Palette::dark(self.high_contrast),
        }
    }

    fn fade_to_target(cx: &mut App) {
        let duration = motion::duration(motion::THEME, cx);
        let theme = cx.global_mut::<Theme>();
        theme.transition += 1;
        let (id, from, to) = (theme.transition, theme.colors.clone(), theme.target());
        let start = Instant::now();
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor().timer(FRAME).await;
                let t = (start.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0);
                let running = cx.update(|cx| {
                    let theme = cx.global_mut::<Theme>();
                    if theme.transition != id {
                        return false;
                    }
                    theme.colors = from.mix(&to, motion::ease_in_out_cubic(t));
                    cx.refresh_windows();
                    t < 1.0
                })?;
                if !running {
                    return anyhow::Ok(());
                }
            }
        })
        .detach_and_log_err(cx);
    }
}
