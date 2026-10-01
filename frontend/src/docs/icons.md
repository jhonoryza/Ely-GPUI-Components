# Icons, fonts and assets

`Assets` embeds Ely's icons and fonts in the binary. Pass it to `with_assets`; `init` registers the fonts and checks that `icons/check.svg` loads.

## Icons

Ely bundles 306 Lucide 1.48.0 icons, each an `IconName` variant: `IconName::Check` is `icons/check.svg`, and `name()` returns the Lucide name, `check`. `IconName::ALL` lists them all.

| Call | Does |
| --- | --- |
| `Icon::new(name)` | Draws the icon at `IconSize::Md` in the theme's `fg`. |
| `.size(IconSize::Xl)` | `Xs` to `Xxl`, from the theme's icon sizes. |
| `.color(color)` | Any color; components pass theme colors. |
| `.rotate(Radians(..))` | Turns the glyph inside its box. |
| `.scale(factor)` | Draws the glyph larger or smaller in the same box. |

GPUI's `svg()` paints with its own text color alone, so `Icon` always sets one.

## Adding an icon

From a clone of the repository:

1. `scripts/icons.sh <lucide-name>...` fetches Lucide 1.48.0 with npm and copies each SVG into `assets/icons/`.
2. Add a line to the `icons!` list in `src/primitives/icon.rs`, such as `Rocket => "rocket",`.

## File icons

`FileIcon::file(name)` shows the icon a file's name suggests, and `FileIcon::folder(open)` a folder's. Ely maps extensions to icons: `rs` and `ts` to `FileCode`, `toml` to `FileCog`, `json` to `FileJson`, images, audio, video, archives and more. An app lays its own rules over that map:

| Call | Does |
| --- | --- |
| `IconTheme::default().name("Cargo.toml", IconName::Package)` | A file of that exact name, in any case. |
| `.extension("md", IconName::BookOpen)` | Files with that extension, given without its dot. |
| `.apply(cx)` | Serves every window from then on, and repaints. |

A name rule beats an extension rule, and both beat Ely's map.

## Fonts

`init` registers seven fonts from `Assets`: Inter Regular, Medium, SemiBold and Italic; JetBrains Mono Regular and Medium; IBM Plex Sans Regular. The theme draws text in Inter and code in JetBrains Mono, set by its `font_family` and `mono_family` fields. On the web, GPUI answers the system UI font with IBM Plex Sans, and text inside SVGs falls back to it.

## An app's own assets

GPUI takes one asset source. An app with files of its own writes an `AssetSource` that answers its own paths and hands every other path to `Assets`. `init` panics unless `icons/check.svg` loads through it.

## A window of icons

```rust example=icons
```
