# GPUI Kit

Started 2026-10-07. Ely and Longbridge's GPUI Kit (`gpui-kit` on crates.io) share one app and one window. Measured before work:

- Ely took gpui from Zed's git at `1a28cff`; Kit 0.7.1 takes `gpui-pre =0.3.8`, a snapshot of Zed at `279fe07`, 109 commits on. Two sources make two crates, and their elements do not mix. Ely on `gpui-pre` 0.3.8 builds and passes all 1,685 tests.
- Actions and key contexts do not collide: Ely's namespaces and contexts all start with `ely`/`Ely`.
- Ely binds Tab with no context, which gpui ranks at the deepest level. In a Kit code editor inside an Ely scope, Tab ties with Kit's indent, and the later `init` wins.
- An app has one asset source. Of Ely's 306 icons, Kit's source holds 290 paths, 289 with the same drawing; `credit-card` differs by one stroke (Lucide 1.48 against 1.43). Ely's `init` checked one icon, so Kit's source alone passed and Ely's other files went missing later.
- Kit's `init` installs its inspector in debug builds; Ely's `install_inspector` replaces it when called after.
- Kit can run on `gpui-fast` instead; Ely could not follow.

Rules: the work order follows dependencies, since the PRD has no chapter for it. Each item passes `scripts/check.sh` and a codex review (`gpt-6.1-sol`, effort `high`) in a herdr split, five rounds at most, then commits and pushes.

## Items

- [x] K01 gpui from `gpui-pre =0.3.8`, Kit's pin: crate, gallery, web build, docs, CI
- [ ] K02 Tab and Shift-Tab bound under `FocusScope`'s own context, so a deeper context's Tab wins whatever the order of `init`
- [ ] K03 Assets: `init` checks every embedded file through the app's source; one source serves Ely's files and another's
- [ ] K04 `compat/kit`: a crate on the newest Kit that runs both libraries in one window, in `scripts/check.sh` and CI
- [ ] K05 `gpui-fast`: Ely builds and tests on Kit's other engine, or the item records why not
- [ ] K06 Docs: README, the site's guide, AGENTS.md
- [ ] K07 Acceptance table

## Log

One line per item: what changed, what stayed and why, review rounds.
- K01: gpui, gpui_platform and gpui_web come from `gpui-pre` `=0.3.8`; the lock holds no git source and one copy of each. `gpui-pre` split leak detection from `test-support`, so this crate's `test-support` turns on `gpui/leak-detection`, and a test that leaks on purpose proves it. The check, the wasm check, the web build (20.0 MB), a headless Chromium shot and the site build pass. README and the install guide name the snapshot. Review: 2 rounds.
