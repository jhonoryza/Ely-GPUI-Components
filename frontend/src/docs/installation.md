# Installation

## Requirements

- Rust 1.95 or later, the version Ely's `Cargo.toml` names.
- macOS, where Ely is tested. Windows and Linux builds are not tested yet.
- No full Xcode: GPUI compiles its Metal shaders at run time.

## Add the dependencies

Ely is not on crates.io. Depend on it by git, with GPUI from Zed's repository at the revision Ely pins:

```toml
[dependencies]
ely-gpui-component = { git = "https://github.com/ZacharyZhang-NY/Ely-GPUI-Components" }
gpui = { git = "https://github.com/zed-industries/zed", rev = "1a28cff4b409169bac058bca40dfbfeb7621d19b" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "1a28cff4b409169bac058bca40dfbfeb7621d19b", features = ["font-kit"] }
```

| Line | Why |
| --- | --- |
| `ely-gpui-component` | The library. `Cargo.lock` records the commit; add `rev = "<commit>"` to hold it in `Cargo.toml` as well. |
| `gpui` | The same revision as Ely's. Another revision builds a second GPUI, and its types do not match the ones Ely takes. |
| `gpui_platform` | Opens windows on each platform. Without `font-kit`, macOS falls back to GPUI's no-op text system and draws no text. |

## Shaders

Ely's default feature, `runtime_shaders`, turns on the same feature in `gpui_platform`, so GPUI compiles its Metal shaders at run time. With `default-features = false` the build runs `xcrun metal` instead, which needs Xcode's Metal compiler.

## Check the install

Copy the [first window](/docs/#a-first-window) into `src/main.rs` and run it:

```sh
cargo run
```

A window opens with a counter and a Press button. From a clone of the repository, `cargo run --example gallery` opens every chapter.

## When it fails

| Symptom | Cause and fix |
| --- | --- |
| Panic: `` ely: pass `ely_gpui_component::Assets` to `Application::with_assets` `` | `init` found no Ely assets. Start the app with `gpui_platform::application().with_assets(Assets)`. |
| Panic: `ely: asset source failed` | The app's asset source returned an error. The panic carries it. |
| Panic: `ely: embedded fonts failed to register` | GPUI's text system refused Ely's fonts. File a bug with the log. |
| Windows open with no text on macOS | `gpui_platform` lacks the `font-kit` feature. |
| `mismatched types`, with the note ``there are multiple different versions of crate `gpui` in the dependency graph`` | `gpui` points at a revision other than Ely's. Use `1a28cff4b409169bac058bca40dfbfeb7621d19b`. |
| `metal shader compilation failed` | Default features are off and Xcode's Metal compiler is missing. Keep `runtime_shaders`. |
