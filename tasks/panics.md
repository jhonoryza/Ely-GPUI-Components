# Panics

Started 2026-10-06. A report counted the crate's panics: handlers left out, keys missing from their lists, outside failures, asserts, `expect`, `unreachable!`. Measured in library code, tests aside: 158 `panic!`, 477 `assert!`, 23 `assert_eq!`/`assert_ne!`, 343 `expect`, 14 `unreachable!`.

Rules, at the owner's word:

- A handler a component needs is an argument of `new`. Leaving it out fails to compile, not at render.
- A state live data or the outside world can reach (a key gone from a list that changed, a failed load) logs an error naming the owner and draws a visible state: nothing chosen, or an error line in the component's box. The app keeps running.
- An assert on a caller's argument, or on a fact true by construction, stays, as std's do. One that runtime data or input can trip is fixed. Reachable means what a component meets on its own: input inside it, its own later state, a call to the outside, or two arguments that agree at one time and not the next (a selected key and its list). One value checked against itself (a ratio above zero) is the caller's contract. A current index or key checked against its list is two arguments even in one call: an owner that rebuilds from live data reaches the mismatch. A list's own emptiness is one value.
- An unknown system time zone logs once and reads as UTC; every time shown in the system zone goes through `format::datetime`, which says "UTC" then. Each item routes its own times there.
- A paint call that fails inside gpui (glyphs, the sprite atlas) logs an error naming the component and skips that draw; drawing an error state would meet the same fault.

Each item reads every site in its folders, fixes what the rules name, updates callers in the gallery, docs examples and tests, passes `scripts/check.sh`, and passes a codex review (`gpt-6.1-sol`, effort `high`) in a herdr split, five rounds at most. Then it commits and pushes.

## Items

- [x] P00 Policy in AGENTS.md
- [x] P01 Primitives — `src/primitives`, `src/lib.rs`, `src/assets.rs`
- [x] P02 Typography — `src/typography`
- [x] P03 Layout — `src/layout`
- [x] P04 Window & Shell — `src/shell`
- [x] P05 Buttons & Actions — `src/buttons`
- [x] P06 Forms — `src/forms`
- [x] P07 Navigation — `src/navigation`
- [x] P08 Menus — `src/menus`
- [x] P09 Overlays — `src/overlays`
- [x] P10 Feedback — `src/feedback`
- [x] P11 Loading & Motion — `src/motion`
- [x] P12 Data Display — `src/data_display`
- [x] P13 Lists & Trees — `src/lists`
- [ ] P14 Tables — `src/tables`
- [ ] P15 Charts — `src/charts`
- [ ] P16 Finance — `src/finance`
- [ ] P17 Code Editor — `src/editor`
- [ ] P18 Terminal — `src/terminal`
- [ ] P19 Git — `src/git`
- [ ] P20 Debug — `src/debug`
- [ ] P21 Documents — `src/documents`
- [ ] P22 Collaboration — `src/collab`
- [ ] P23 AI Chat — `src/chat`
- [ ] P24 Agent — `src/agent`
- [ ] P25 Generative — `src/generative`
- [ ] P26 Media — `src/media`
- [ ] P27 Files — `src/files`
- [ ] P28 Messaging — `src/messaging`
- [ ] P29 Mail — `src/mail`
- [ ] P30 Calendar — `src/calendar`
- [ ] P31 Project — `src/project`
- [ ] P32 Canvas & Design — `src/canvas`
- [ ] P33 DB & Dev Tools — `src/devtools`
- [ ] P34 Dashboard — `src/dashboard`
- [ ] P35 Settings — `src/settings`
- [ ] P36 Account — `src/account`
- [ ] P37 Onboarding — `src/onboarding`
- [ ] P38 Interaction — `src/interaction`
- [ ] P39 Theme — `src/theme`
- [ ] P40 i18n & a11y — `src/i18n`
- [ ] P41 Maps — `src/maps`
- [ ] P42 Misc — `src/misc`
- [ ] P43 Library Tooling — `src/tooling`
- [ ] P44 Rendering — `src/rendering`
- [ ] P45 Recount, acceptance table

## Log

One line per item: what changed, what stayed and why, review rounds.
- P00: AGENTS.md holds the three rules; this list follows the PRD's chapters. Review: 2 rounds (round 1 found the runtime clause missing).
- P01: `init` returns `anyhow::Result`; a missing app icon logs once and shows `Icon::broken`. Kept: `layer.rs` expects (gpui's element order), `checked_ratio` (one value, the caller's). Review: 2 rounds.
- P02: `Link::new` takes its handler; `system_zone` logs once and reads as jiff's unknown zone, which `datetime` marks UTC; an oversized highlight query logs and marks nothing; a cut line's paint fault logs. Kept: caller contracts (speed, digits, step, keystroke, heading level, pattern) and the exponent parse. Review: 3 rounds.
- P03: `PaneGroup` leaves a gone or last pane and logs (`split` returns `Option`); a stale split path sets nothing; `SplitPane` splits evenly when sizes lag or fail, seeds by the owner's raw sizes, and drops a drag over another pane count; masonry sorts with `total_cmp`. Kept: dock ids (panels never leave), one-value contracts, pane-id overflow, workspace JSON. Review: 2 rounds.
- P04: no change. Kept: splash progress and column count (caller contracts), the objc class under `Once`, `Tray::pick` (items fixed at creation). Review: 1 round.
- P05: a `SegmentedControl` whose choice left its segments marks none; the cause, `motion::slide`, logs and returns no marker (P11 inherits it). Kept: one-value contracts (group and quick-action counts, toggle label or icon, share payload). Review: 1 round.
- P06: a root path shows whole; `TextInput::select` logs a range past the text or inside a character; the text element logs shaping and paint faults; a unit gone from its list logs; cron runs in the system zone go through `datetime`. Kept: empty choice lists (the owner disables, as AGENTS.md says), one-value contracts, facts by construction. Review: 2 rounds.
- P07: `Wizard::new` takes `on_step` and `on_finish`; tabs, crumbs, history, steps, wizards and pagination with a place their list no longer holds log and mark none; an empty or all-disabled tab strip ignores arrows; a gone recent command is skipped. Kept: non-empty lists, disabled-free switcher and search items, row index parse. Review: 3 rounds.
- P08: no change. Kept: a bar's menus and a pie's slices (one value each); the panel's level lookup, which `settle` fits to the menu before every draw. Review: 1 round.
- P09: a `Lightbox` or `Tour` past its last item logs and closes, handing focus back. Kept: dialog Enter without `focus_first` and popover face builders on an owner's opener (static builder misuse). Review: 1 round.
- P10: no change. Kept: a toast's stay above zero (one value). Review: 1 round.
- P11: a buffer at or behind the value draws none; `slide` was fixed in P05. Kept: values in 0..=1, segment and skeleton counts (one value each), builder misuse, reorder's permutation. Review: 1 round.
- P12: a featured column past the columns features none; a gauge past its scale pegs; usage parts past the total fill it and repeat hues past eight; stars above max log and fill. Kept: one-value contracts, a comparison row's shape, construction. Review: 3 rounds (round 2 met an edit that had not applied).
- P13: a directory listing's selected name gone from its entries picks none. Kept: one-value contracts; activation, folder split, single-select sections and tree rows hold by construction. Review: 1 round.
