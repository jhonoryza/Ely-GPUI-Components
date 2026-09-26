# AGENTS.md

Ely GPUI Component. A component library for GPUI, in light and dark.

## Stack

- Rust 1.95, edition 2024. One crate: `ely-gpui-component`.
- gpui 0.2.2 from crates.io. Feature `runtime_shaders` is on by default: machines without Xcode lack the Metal compiler.
- Assets, embedded with `rust-embed`: Lucide 1.48.0 icons (ISC), Inter 4.1 (regular, medium, semibold, italic) and JetBrains Mono 2.304 (OFL).
- Catalog data: `emojis` (Unicode emoji), `isolang` (ISO 639, native names), `isocountry` (ISO 3166), `iso_currency` (ISO 4217, also minor units for money).
- Codes: `qrcode` (QR) and `barcoders` (Code 128, EAN-13), both MIT OR Apache-2.0, default features off.
- macOS extras (tray icon, Dock badge) call AppKit through `cocoa` 0.26 and `objc` 0.2, the crates gpui already links.
- Terminal: `alacritty_terminal` 0.26 (Apache-2.0, default features off) for the grid, its parser and the pseudo-terminal; `futures` carries its events.
- Diffs: `similar` 3.2 (Apache-2.0) for line and word diffs and three-way merges; its `unicode` feature splits words at punctuation.
- Markdown: `pulldown-cmark` 0.13 (MIT, default features off) for CommonMark with tables, tasks, strikethrough, footnotes and math.
- Gallery: `examples/gallery`. Website: `frontend/` (Vite 8, pnpm), built after the components.

## Commands

- `cargo run --example gallery` opens the gallery. `-- --page <slug>` starts on a page.
- `cargo run --example gallery -- --capture <dir>` writes PNGs of every page, light and dark, top to bottom, then each page's scripted states, including windows the demos open. macOS only.
- `cargo test --lib --features test-support`, `cargo clippy --all-targets --features test-support -- -D warnings`, `cargo fmt --check`. `scripts/check.sh` runs them with the house rules.
- `rm -rf target/debug/incremental` after each task item keeps the disk lean.
- `scripts/icons.sh <lucide-name>...` adds icons. Then add a line to `IconName` in `src/primitives/icon.rs`.

## Layout

- `src/<chapter>/`: one folder per chapter of `gpui-components.md`.
- `src/theme`: palettes, tokens, `ActiveTheme`. `src/motion`: durations, easings, spring.
- `examples/gallery/pages/<chapter>.rs`: one page per chapter, in PRD order.
- `TASKS.md` and `tasks/`: progress. Every PRD entry has a line and a tag.

## Rules

- Colors, control and icon sizes, text sizes, radii, shadows come from `cx.theme()`. No raw `px()` literals inside components.
- Gaps and padding use gpui's rem scale helpers (`gap_2`, `px_3`). That scale is the spacing system.
- Borders use gpui's fixed-pixel helpers (`border_1` is 1px). Hairlines do not scale.
- One component, one home. Duplicates point to it.
- Fail fast. No silent fallbacks. Log state changes with `log`.
- Files stay at or under 500 lines. Comments are one short line, and rare.
- A repeating animation (spinner, breath, sweep) holds still under reduced motion instead of shortening, or it would spin at 1 ms a turn. An effect set off by a change (flash, shake) reads elapsed time rather than restarting an animation id, so what it holds keeps its state.
- `motion::duration()` honors reduced motion. Springs overshoot: use them inside animators, never as gpui easings, because gpui asserts eased values stay in `0..=1`.
- `svg()` paints only with its own `text_color`. `Icon` always sets one.
- Buttons stay out of focus on click (`prevent_default` on mouse down). Tab still reaches them.
- The focus ring is a 1px focus-colored border (`FocusRing`). gpui paints shadows under the element, so a shadow ring floods transparent elements. Focusable elements keep a 1px border, transparent at rest.
- `ely_gpui_component::init` binds Tab and Shift-Tab to `FocusNext` and `FocusPrev`. Wrap the root in `FocusScope` with `.root()` and focus its handle. A root scope takes focus when the focused element leaves the tree, as a toast's or an alert's button does.
- gpui's `Window::dispatch_event` returns a private type, so the gallery scripts mouse input by posting `NSEvent`s to its own queue.
- `hover()` needs an element id.
- `occlude()` blocks the pointer for everything painted before it, its own ancestors too. Put it on the outermost box that should stop the pointer, as the toast stack does, or the ancestors' hover never fires.
- gpui sends a drag's moves to every listener of its type. Each drag payload carries its owner's `EntityId`, and handlers check it.
- A component that takes a starting value from its owner keeps it in `layout::seeded`: a new value from the owner replaces local drags.
- The capture harness turns on reduced motion, so shots are still: repeating motion rests on one frame, and a shimmer shows no band.
- The capture harness finds gpui windows by handle through `raw-window-handle`. AppKit popups, which gpui does not own, come from the window list by process and level.
- Posted events cannot move a macOS window, so window drags are not scripted. `drag_region` follows Zed's title bar.
- An overlay takes focus with `primitives::take_focus` and returns it with `give_back` on every close path. While open, it takes focus back whenever the focused element leaves the tree (gpui's `on_focus_lost`).
- A state change made outside render calls `cx.notify()`, or nothing redraws. A notify made while drawing, as in a canvas's prepaint, marks the view but schedules no frame (gpui's `invalidate_view`), so it also calls `window.request_animation_frame()`.
- The macOS share picker draws its content in another process; a capture shows only its frame.
- NSColorSampler runs in another process too. The harness cannot pick or cancel with it; `read_srgb` is tested on a constructed `NSColor` instead.
- Every text field is a `forms::TextInput` entity. Its keys live under the `ElyInput` context, bound in `init`. Wrappers take Up, Down, Enter and Backspace first with `capture_action`.
- `TextInput::bounds_for` reads the last layout. Text changed this frame clamps to the laid-out end until the next paint.
- The harness types with `dispatch_keystroke` through `AnyWindowHandle`, which leaves the root view free to redraw.
- The macOS open panel runs out of process too. Escape does not reach it; the harness sends it `cancel:`.
- gpui's `test-support` swaps its executor: `block` fails on a real dispatcher. It stays behind this crate's `test-support` feature, never on for the gallery.
- A tracked `FocusHandle` is a Tab stop only when built with `.tab_stop(true)`. `tab_index` on the div reaches only handles gpui makes itself. `primitives::tab_stop` keeps one per id.
- Floating lists go below their anchor, or above when only above has room (`forms::options::float`). gpui's own switch keeps the anchor point, so a flipped list would cover its trigger.
- A trigger that opens a list goes through `forms::select::listing`: toggle, keys, blur and popup live there, for Select, TabOverflowMenu and Breadcrumb alike.
- A palette goes through `navigation::palette::Palette`: query field, grouped rows, cursor, keys and focus live there. CommandPalette, QuickOpen, QuickSwitcher, SearchPalette and QuickLauncher only build rows.
- A menu goes through `menus::menu`: rows, submenus, keys, focus and presses outside live there, for DropdownMenu, OverflowMenu, SplitButton, ContextMenu, SearchableMenu and MenuBar. A press on the host's own box is the host's to handle, so a trigger toggles and a right click moves a context menu without a close and reopen in one event.
- gpui focuses the nearest focusable ancestor on mouse down, and apps wrap everything in a `FocusScope`. A press that must leave focus alone calls `prevent_default`, as buttons and context regions do.
- gpui draws a window's first frame inside `open_window`. State the first frame reads, such as `cx.set_menus`, is set before it opens.
- gpui clicks a focused element when Enter or Space is released. An overlay that hands focus back picks on release too; a pick on press returns focus first, and the release clicks the opener again.
- A dialog goes through `overlays::Dialog`: scrim, focus trap, Escape and focus return live there. AlertDialog, ConfirmDialog and PromptDialog build on it; a fullscreen dialog fills the window below its title bar. A dialog's card fits the window, a title bar's height clear at each end; its body scrolls and its actions stay.
- `primitives::Severity` names info, success, warning and danger with their colors and icons, for alerts and feedback alike.
- Rows that move are keyed. `motion::Flip` places them absolutely from last frame's heights, so no frame shows a row at its new place before it glides there, and drives the glide from elapsed time: an animation id that changes per move would rebuild the row's keyed state. `Reorder` holds a row by key, drags with an empty ghost and moves the held row itself.
- A marker that slides between items (segment thumb, tab line) measures them with `motion::slide` and eases with `glide`.
- A picker hands focus to its popup on open and back to its trigger on pick or Escape; it closes once focus leaves both (`forms::date::picker`).
- Motion for a value change keys its animation on `motion::changes`, so it replays per change and stays still on first paint. Key only the part that moves: a keyed ancestor gives every descendant a new id, and focused buttons lose focus.
- A spotlight or tour takes its target's box from `primitives::Measure`; the lit box glides between targets.
- Thumbs stay inside their component's box: the track is padded by half a thumb (`Slider`, `ColorPicker`, `GradientEditor`).
- gpui's `Hsla::opacity` scales alpha; `alpha` sets it.
- A rounded box does not clip its children. Each layer inside takes the radius itself; `checker` takes one.
- A window keeps one `feedback::Toaster` and renders one `ToastViewport`; anything holding the entity pushes to it. Messages share `primitives::Severity`.
- Only gpui can build `ExternalPaths` with paths, so drop rules live in `forms::files::dropped`, where tests reach them.
- gpui runs key bindings before key listeners. A container takes a child's bound key through the action with `capture_action`, as `Form` takes `Submit`.
- `ScrollHandle::scroll_to_item` runs in the container's prepaint against last frame's bounds and overflow, so a call before a list's first frame does nothing. Call it from a child's prepaint once the handle has bounds, then request one frame (`navigation::editor_tabs`). A floating list reveals its cursor, and a strip its focused item, through `forms::options::reveal` and `revealer`. Never poll layout with `request_animation_frame`.
- gpui animations run on the wall clock; timers run on the executor's clock, which tests advance by hand. A test that needs an animation settled turns on the theme's reduced motion. Clocks read `background_executor().now()` and wake only their own view.
- Machine-read marks, QR codes and barcodes, paint `ink` on `paper`: dark on light in both themes, so cameras read them.
- A layer over pressable content that must still pass scrolls to its ancestors uses `block_mouse_except_scroll`; `occlude` hides them from scrolls too.
- Trackpad momentum arrives as `TouchPhase::Moved` after `Ended`, so a gesture acts only between `Started` and `Ended` (`lists::SwipeableListItem`).
- Quartz scroll events carry no window. The harness hands them to the window itself, placed in window points (`Step::Swipe`).
- A chevron that opens and closes is `primitives::Disclosure`.
- `uniform_list` does not stretch its rows; each row sets `w_full`.
- Chapters import only earlier chapters, so a select of tree nodes lives in `lists` (`TreeSelect`) and reaches `forms::listing`.
- Script clicks aim at a probe's corner, so a scripted target must fit in the window; a tall probe puts its top off screen.
- `scripts/check.sh` fails by its exit status; a pipe into `tail` hides it.
- Reduced motion shortens animations to 1ms on the wall clock. Tests sleep past it between refreshes to see where one ends.
- The test platform never runs next-frame callbacks, and its text metrics are simplified: each character takes a fixed advance. Tests refresh the window in place of the display link, and click where layout does not hang on text, such as padding and slots.
- An absolute element with no insets sits where it would flow: after the content in a block, inside the padding in a flex box. A canvas that measures its parent pins itself with `.top_0().left_0()`.
- Keyed state lives while its element renders in consecutive frames. An overlay rendered only while open starts fresh each time.
- Long grids scroll in a `uniform_list`. gpui has no nearest scroll, so a key move up scrolls with `Top` and down with `Bottom` (`forms::glyphs`).
- Masks reshape edits inside `TextInput::set_fit`, which sees the replaced range and the typed text. A diff after the fact cannot tell typed characters from kept ones.
- Ids inside a reusable component carry its owner's id or `EntityId`, animation ids too. Twin ids share focus, click and animation state.
- Text wraps at the width its box had when measured. A cross-axis `max_w` comes too late, and a flex column counts wrapped text as one line. Put prose in a plain block inside a flex row with `flex_1().max_w(..)` (`feedback::states`).
- gpui 0.2.2 keeps a no-wrap line's first measure, taken at full width inside flex and scroll boxes, so `.truncate()` clips there instead of ending in an ellipsis. A line that may overflow is a `typography::Ellipsis`.
- Set an explicit line height on any box that clips text. gpui's default leading is taller than a tight box, and the clip eats descenders.
- gpui's `img` takes its picture's pixel size for any `Auto` side, and its aspect ratio beats percent heights in flow. `Image` pins it absolute at full size, so an `Image` needs a sized box.
- Badges, tags and avatars live in `data_display`; forms and shell draw theirs from there, and `data_display` imports neither. Avatars take square pictures.
- `img()` keeps loading state only with an id. Content masks are rectangles, so rounded corners survive only when the image fills its box without cropping.
- Rise reads green and fall red unless a component's `red_up` swaps them, as markets in East Asia read. `finance::quotes::moves` picks the pair.
- Charts lay out in `f32` pixels of their own box, tested without gpui, and paint through `charts::paint::at`: a canvas's paint gets window coordinates, and `with_element_offset` works only in prepaint. Their sizes come from `theme.chart()`.
- A label pinned to a point sits in a zero-size absolute box that centers it with flex (`charts::axes`), so no offset guesses at text size.
- Tokens live in `src/theme`; the raw `px()` scan skips that folder only.
- `ScrollHandle` keeps its children's bounds unscrolled: a child paints at its bounds plus the offset. A press on a scrolled child subtracts both, and an offset that brings a child to a place is set whole, not added (`documents::find::reveal`).
- A find box is `editor::FindWidget`, over code, a terminal or a document's pages. It shows the replace fold and the option toggles only when the owner handles them.
- Taffy 0.9 hands negative free space to auto margins and centering alike, so a centered child wider than its scroll box starts past the scroll's reach. Center only what fits the measured box (`documents::DocumentViewer`).
- A mode that leaves on Escape holds its own focus through `primitives::hold_focus`: turned on while focus is elsewhere, it takes focus so Escape reaches it, and `hand_back` returns it (`documents::ZenMode`, `collab::FollowMode`).
- A chat holds at its newest message through gpui's `list` with `ListAlignment::Bottom`: it sticks while scrolled to the end, and `scroll_to` one past the last item sticks it again; that call sends no scroll event, so the owner clears its own away state (`chat::MessageList`).
- Anything pressable is a Tab stop: a row, card or toggle takes `tab_index(0)` and `focus_ring`, with a 1px border that stays transparent where it shows none; gpui presses it on the release of Enter or Space.
- A row that holds text beside fixed parts lets the text give way: the text takes `flex_1().min_w_0()` and an `Ellipsis`; counts, icons, dates and chevrons take `flex_none`.
- A mark over a text field reads the field at paint time, from a canvas laid over it (`collab::RemoteCursor`): by then `TextInput::bounds_for` holds this frame's layout, and a still page never renders again to catch up.
- A callback field holds a named `type` alias; clippy's type_complexity rejects `Option<Rc<dyn Fn(..)>>`. A chapter shares one alias per shape.
- A component that fills its container says `w_full` on its root. As a window's root, or in a flex row, it would shrink to its text's narrowest width.
- A field's `Highlight` sets color, wash, weight, slant and strike per span; measuring and painting share one set of runs, so wrapped heights match.
- A block editor keeps one undo for the document: text and structure together, as snapshots through `forms::History`; it takes Undo and Redo before its fields do.
- gpui calls focus listeners only while the window is active. A test that needs focus events activates its window with `window.activate_window()`.
- A terminal takes Tab and Shift-Tab through bindings under `ElyTerminal`, ahead of focus moves; other keys go through `terminal::keys` in a key listener, and typed text through its input handler.
- A shell in the gallery starts without startup files and with a fixed prompt, so captures show no user or host.
- gpui 0.2.2 has no `(ElementId, usize)` id. A child id names its index: `(self.id.clone(), format!("row-{ix}"))`.
- Clippy's `single_range_in_vec_init` rejects `[a..b]` and `vec![a..b]`; bind the range first.
- A component built on `lists::Tree` fills its box; the host gives it a height. A fixed-width column beside a `flex_1` one takes `flex_none`, or it shrinks.
- Editor rows read their text through `CodeEditor::row_text`: a multi-line ghost cuts its line, and the rest follows the ghost's last line.
- gpui styles a drag-over only on an element with its own hitbox. A veil that shows while files hover takes the drop itself (`chat::DragDropOverlay`).
- gpui 0.2.2 never clears `debug_bounds`: a test sees an element appear, not leave.
- The system time zone comes from `typography::format::system_zone`, which fails loud; jiff's `TimeZone::system()` falls back to UTC in silence.
- A list opens with its cursor on the current choice: Select on the chosen row, Combobox on its value, MultiSelect on the first one ticked.
- A search step, a tool call or an agent's step says where it stands with `chat::StepState` and draws it with `chat::step_mark`.
- A flex line whose free space goes to an auto margin loses its gaps in taffy 0.9. A row that parts left and right groups its sides and uses `justify_between`; `ml_auto` is safe only beside a `flex_1` item that leaves no free space.
- A header that holds a name beside counts and actions wraps: `flex_wrap`, the name `flex_1` with `min_w(label_width)`, the rest one `flex_none` group that drops below.
- Every header, row and strip works at 280px: a name keeps its minimum width, the rest wraps below or scrolls, and nothing draws past its box.
- The focus color belongs to the focus ring. A chosen or shown item takes accent, as `forms::glyphs` and `lists::ListItem` do.
- A box that shows a host's picture takes the picture's shape from the host (`ratio`), so nothing is cropped and a point given as a share lands on the picture.

## Decisions

- 2026-09-24: official gpui 0.2.2 over the `gpui-pre` snapshot. Older API, first-party publisher.
- 2026-09-24: one crate. Revisit if an incremental check passes 90 seconds.
- 2026-09-24: gpui 0.2.2 has no accessibility tree and no tray, badge, or notification API. Those entries carry `blocked` or `prove`.
- 2026-09-24: the gallery photographs its own window with `CGWindowListCreateImage`. No Screen Recording permission needed.
- 2026-09-24: license MIT OR Apache-2.0.
- 2026-09-25: layouts persist as versioned JSON through serde. Unknown fields are ignored. Newer versions, unknown panels and bad pane trees are refused, and restore is all or nothing.
- 2026-09-25: an entry built from a later chapter's parts lands with that chapter; its line points there. MenuBar goes to Menus, QuickLauncher to Navigation.
- 2026-09-25: SystemNotification is blocked. An unbundled app has no notification center (probed on macOS 27.2), and gpui 0.2.2 has no API.
- 2026-09-25: `unexpected_cfgs` declares `feature = "cargo-clippy"`, which the objc 0.2 macros test.
- 2026-09-26: Terminal proven with `alacritty_terminal` 0.26: its grid, parser and pseudo-terminal, drawn by Ely. A replayed grid serves recorded output and tests.
