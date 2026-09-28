# Tasks

Progress truth. Work runs top to bottom. No item is skipped.
Each `T` item ends with a review in a herdr split pane, three rounds at most: codex (`gpt-6-astra`, effort `max`) through T24d round 2; from T24d round 3, after codex ran out of quota on 2026-09-26, a Claude session on Fable 5.1 at max effort, at the user's word.

Per-component lines live in `tasks/`. Tags there:

| Tag | Meaning |
|-----|---------|
| `→ path` | Alias or duplicate. One home, at `path`. |
| `native` | gpui 0.2.2 ships it. The gallery shows the native call. |
| `host` | Ely draws the UI. The host app supplies the engine data. |
| `prove` | Needs a crate or platform API not yet proven with gpui 0.2.2. |
| `blocked` | Cannot be real on gpui 0.2.2. Reason given. |
| `(Txx)` | Built early in task `Txx` as a dependency. |

## Items

- [x] T00 Foundation: package, theme, motion, assets, Icon, Button, gallery shell, docs
- [x] T01 Primitives — `tasks/ch01-10.md`
- [x] T02 Typography
- [x] T03 Layout
- [x] T04 Window & Shell
- [x] T05 Buttons & Actions
- [x] T06a Forms · Text
- [x] T06b Forms · Selection
- [x] T06c Forms · Date & Time
- [x] T06d Forms · Color
- [x] T06e Forms · Files
- [x] T06f Forms · Other
- [x] T06g Forms · Structure
- [x] T07a Navigation · Tabs & paths
- [x] T07b Navigation · Places
- [x] T07c Navigation · Palettes & tour
- [x] T08a Menus · Menu & hosts
- [x] T08b Menus · Bar, search & pie
- [x] T09a Overlays · Popovers & dialogs
- [x] T09b Overlays · Guides & floats
- [x] T10a Feedback · Messages
- [x] T10b Feedback · States
- [x] T11a Loading · Spinners & skeletons — `tasks/ch11-20.md`
- [x] T11b Loading · Progress & states
- [x] T11c Motion · Presence & order
- [x] T11d Motion · Effects
- [x] T12a Data Display · People & numbers
- [x] T12b Data Display · Records
- [x] T12c Data Display · Media
- [x] T12d Data Display · Codes & measures
- [x] T13a Lists & Trees · Lists
- [x] T13b Lists & Trees · Trees
- [x] T13c Lists & Trees · Files
- [x] T14a Tables · Core
- [x] T14b Tables · Columns & rows
- [x] T14c Tables · Grids
- [x] T15a Charts · Cartesian
- [x] T15b Charts · Parts and spreads
- [x] T15c Charts · Flows and relations
- [x] T16a Finance · Market charts
- [x] T16b Finance · Technical analysis
- [x] T16c Finance · Quotes & book
- [x] T16d Finance · Trading
- [x] T16e Finance · Markets & assets
- [x] T17a Editor · Core
- [x] T17b Editor · Intelligence
- [x] T17c Editor · Search
- [x] T17d Editor · Panels
- [x] T17e Editor · Status items
- [x] T18 Terminal
- [x] T19 Git
- [x] T20 Debug
- [x] T21a Documents · Editing — `tasks/ch21-30.md`
- [x] T21b Documents · Reading
- [x] T21c Documents · Knowledge
- [x] T22 Collaboration
- [x] T23a AI Chat · Messages
- [x] T23b AI Chat · Citations
- [x] T23c AI Chat · Input
- [x] T23d AI Chat · Conversations
- [x] T23e AI Chat · Welcome
- [x] T24a Agent · Tool calls
- [x] T24b Agent · Progress
- [x] T24c Agent · Changes
- [x] T24d Agent · Previews
- [x] T24e Agent · Environment
- [x] T24f Agent · Control and cost
- [x] T25a Generative · Prompt and settings
- [x] T25b Generative · Queue and results
- [x] T25c Generative · Canvas
- [x] T25d Generative · Sound and motion
- [x] T25e Generative · Models
- [x] T25f Generative · Data and prompts
- [x] T26a Media · Images
- [x] T26b Media · Crop and annotate
- [x] T26c Media · Video
- [x] T26d Media · Audio and controls
- [x] T26e Media · Capture and devices
- [x] T27a Files · Items
- [x] T27b Files · Explorer
- [x] T27c Files · Find and look
- [x] T27d Files · Transfers
- [x] T28a Messaging · Channels
- [x] T28b Messaging · Messages
- [x] T28c Messaging · People
- [x] T28d Messaging · Calls
- [x] T28e Messaging · Stickers and GIFs
- [x] T29a Mail · Boxes and lists
- [x] T29b Mail · Reading
- [x] T29c Mail · Writing
- [x] T29d Mail · Sorting
- [x] T30a Calendar · Months
- [x] T30b Calendar · Time grid
- [x] T30c Calendar · Agenda and people
- [x] T30d Calendar · Editing
- [x] T31a Project · Tasks — `tasks/ch31-43.md`
- [x] T31b Project · Issues and boards
- [x] T31c Project · Plans
- [x] T31d Project · Time and databases
- [x] T32a Canvas · Plane — `tasks/ch31-43.md`
- [x] T32b Canvas · Tools — `tasks/ch31-43.md`
- [x] T32c Canvas · Panels — `tasks/ch31-43.md`
- [x] T32d Canvas · Graphs and boards — `tasks/ch31-43.md`
- [x] T33a DB & Dev Tools · Databases — `tasks/ch31-43.md`
- [x] T33b DB & Dev Tools · Data formats — `tasks/ch31-43.md`
- [x] T33c DB & Dev Tools · APIs — `tasks/ch31-43.md`
- [x] T33d DB & Dev Tools · Tools and machines — `tasks/ch31-43.md`
- [x] T34 Dashboard — `tasks/ch31-43.md` (its two maps lines wait for T41)
- [x] T35 Settings — `tasks/ch31-43.md`
- [x] T36a Account · Sign in — `tasks/ch31-43.md`
- [x] T36b Account · Profile and access — `tasks/ch31-43.md`
- [x] T36c Account · Billing and team — `tasks/ch31-43.md`
- [x] T37a Onboarding · Flows and highlights — `tasks/ch31-43.md`
- [x] T37b Onboarding · Help — `tasks/ch31-43.md`
- [x] T38a Interaction · Pointer — `tasks/ch31-43.md`
- [x] T38b Interaction · Keys, scroll and focus — `tasks/ch31-43.md`
- [x] T39a Theme · Editing — `tasks/ch31-43.md`
- [x] T39b Theme · Importing — `tasks/ch31-43.md`
- [x] T39c Theme · Platform — `tasks/ch31-43.md`
- [x] T40a i18n — `tasks/ch31-43.md`
- [x] T40b a11y — `tasks/ch31-43.md`
- [x] T41a Maps · View — `tasks/ch31-43.md`
- [x] T41b Maps · Layers — `tasks/ch31-43.md`
- [x] T41c Maps · World — `tasks/ch31-43.md`
- [x] T42a Misc · Time — `tasks/ch31-43.md`
- [x] T42b Misc · Numbers — `tasks/ch31-43.md`
- [x] T42c Misc · Codes and checks — `tasks/ch31-43.md`
- [x] T42d Misc · Asking and learning — `tasks/ch31-43.md`
- [x] T42e Misc · Terms and notices — `tasks/ch31-43.md`
- [x] T42f Misc · Web — `tasks/ch31-43.md`
- [x] T42g Misc · Export and import — `tasks/ch31-43.md`
- [x] T43a Tooling · Workbench — `tasks/ch31-43.md`
- [ ] T43b Tooling · Inspection — `tasks/ch31-43.md`
- [ ] T43c Tooling · Catalogs — `tasks/ch31-43.md`
- [ ] T44 Capture: every component, light and dark, plus motion clips; roots still on `w_full` drop it, checked at 280px and in padded cards
- [ ] T45 Website: `frontend/`, Vite 8, pnpm, light and dark: a home page in motion (three.js, smooth scroll), and a components page with every component, categories on the left and the chosen one's detail on the right, after astryx.atmeta.com
- [ ] T46 E2E: Playwright against the built site
- [ ] T47 Ship: GitHub repo (public), CI, Cloudflare deploy through `npx wrangler`
- [ ] T48 Acceptance: live URL and MVP checklist

## T00 detail

- [x] git, `.gitignore`, `LICENSE-MIT`, `LICENSE-APACHE`
- [x] `Cargo.toml`: gpui 0.2.2 with `runtime_shaders`, lints
- [x] Theme: light and dark palettes, high contrast, tokens, `ActiveTheme`, animated switch
- [x] Motion: durations, easings, spring
- [x] Assets: Lucide icons, Inter, JetBrains Mono, `AssetSource`
- [x] Icon, Button, IconButton
- [x] Gallery shell: chapter nav, theme toggle, self-capture
- [x] `AGENTS.md`, `README.md`

## Review log

| Item | Round | Verdict | Notes |
|------|-------|---------|-------|
| T00 | 1 | FAIL | capture swallowed errors; root not focused; raw px sizes; gallery copy; tag conflict |
| T00 | 2 | FAIL | AGENTS.md called `border_1` a rem helper |
| T00 | 3 | PASS | |
| T01 | 1 | FAIL | img loading needs an id; radii lost; focus loop cap; tooltip geometry literals |
| T01 | 2 | FAIL | Cover painted outside the box once the clip was removed |
| T01 | 3 | FAIL | two-line doc comment; fixed after the cap, verified by scan, no fourth review |
| T02 | 1 | FAIL | stale selection; strftime and zone fallbacks; \\text flattening; probes missed targets; ellipsis overflow; u64 precision; caret sizes; LineClamp copy; 500-line wording |
| T02 | 2 | PASS | |
| T03 | 1 | FAIL | restore kept cached split and float state; drags lacked an owner; weights unchecked; pane id overflow broke atomic restore; sticky copy escaped the clip; wheel propagated past the viewport; drawer pull lost the pointer; no top dock; grip sizes raw |
| T03 | 2 | FAIL | closing a pane made NaN or infinite weights; drag ghost text black in dark |
| T03 | 3 | FAIL | even split of a denormal weight underflowed; fixed after the cap by normalizing on close and split, verified by tests and capture, no fourth review |
| T04 | 1 | FAIL | system close skipped TitleBar::on_close; hosted windows had no Tab scope; switcher lost Tab to an ancestor, dropped focus on close, kept a stale index |
| T04 | 2 | FAIL | a removed on_close kept blocking the system close; switcher keys bubbled past it |
| T04 | 3 | PASS | |
| T05 | 1 | FAIL | hold task kept its state alive; a third click wedged the done state; border_1 undid the group seam |
| T05 | 2 | FAIL | AGENTS.md said no window is found by scanning; popups are |
| T05 | 3 | PASS | |
| T06a | 1 | FAIL | twin ids in clear and eye buttons; text fields not Tab stops; IME undo; stepping from a stale value; mask literals re-read as input; undo while disabled; Down past the last line; clamp before rounding; stuck mention dismissal; NaN swallowed by min and max |
| T06a | 2 | FAIL | float error in rounded bounds; composition commit skipped filter and length; a typed leading literal was eaten; PIN select-all collapsed; stepper state read the old value |
| T06a | 3 | FAIL | a mask diff misread a select-all replacement; grid tolerance too wide for large numbers; fixed after the cap by fitting masks inside `TextInput` with the exact edit, and a float-error tolerance; verified by tests and capture, no fourth review |
| T06b | 1 | FAIL | kept highlight and cascade trail went out of bounds; keyboard click reclosed MultiSelect and Cascader; disabled rows committed by keyboard; Combobox hid its owner's value; a disabled chosen radio left no Tab stop; a disabled Rating took arrows |
| T06b | 2 | FAIL | a free Combobox cleared the text it mounted with |
| T06b | 3 | PASS | |
| T06c | 1 | FAIL | range preview reseeded the month; disabled pickers still committed; month buttons left the cursor; hour column scrolled before layout; `*/1` day fields were not wildcards; next(0) ran on; clock fell back to UTC; zone offsets went stale; "never runs" overclaimed |
| T06c | 2 | FAIL | the description of an unstarred full day field ignored the OR rule |
| T06c | 3 | PASS | |
| T06d | 1 | FAIL | a zero-size checker hung the paint loop; a dragged stop lost its grip after passing another; `opacity(1.0)` kept alpha, so opaque pickers stayed see-through; inset thumbs drifted from the drawn gradients; swatch fills and checkers crossed rounded corners; the thumb ring was a literal white |
| T06d | 2 | PASS | |
| T06e | 1 | PASS | |
| T06f | 1 | FAIL | a stroke lost the move that started the drag and its release point; language codes were not searchable though the gallery said so; emoji search compared a lowercase query with mixed-case names |
| T06f | 2 | PASS | |
| T06g | 1 | FAIL | form errors shared one animation id, so a later error skipped its entrance; the demo left links out of its saved snapshot; the demo let a lone @ pass as an email |
| T06g | 2 | FAIL | a field's error still keyed its entrance by the label, so twin labels in one form shared it |
| T06g | 3 | PASS | |
| T07a | 1 | FAIL | editor tabs keyed their scroll on the count of changes: a far tab chosen at mount stayed off screen, and after one change every redraw pulled a manual scroll back |
| T07a | 2 | FAIL | in a hidden or zero-width strip the follow asked for a frame on every render, 126 redraws in 2 s |
| T07a | 3 | FAIL | AGENTS.md said the test platform gives text no width; it gives each character a fixed advance (fixed after the cap) |
| T07b | 1 | FAIL | an open navigation menu kept entry and link indexes past lists that shrank, and disabled links still fired by click or Enter |
| T07b | 2 | PASS | |
| T07c | 1 | FAIL | SearchPalette let a disabled result run on Enter, and the palette tests compiled without test-support |
| T07c | 2 | PASS | |
| T08a | 1 | FAIL | a marked row the owner removed panicked on Enter, the trigger's second click skipped give_back, Enter and Space reached parents on press, the harness never released keys so scripted picks failed, and the menus copy said key hints run rows |
| T08a | 2 | FAIL | Ctrl-Enter and Shift-Space reached keys() on press and picked at once |
| T08a | 3 | PASS | |
| T08b | 1 | FAIL | a shrinking pie kept a mark past its end, a hub press lost focus, pie confirm keys bubbled and modified releases picked, an open dropdown stayed put when its host moved, clearing the filter kept the old mark, and the ring sat half a slice off its hub |
| T08b | 2 | FAIL | the pie's opening right click let a root FocusScope take focus before take_focus recorded it |
| T08b | 3 | PASS | |
| T09a | 1 | FAIL | dialog buttons closed through the owner and skipped give_back, popover and hover panels let presses through, and the popover guessed its height |
| T09a | 2 | FAIL | the prompt's Enter release ran from anywhere in the dialog, so Enter on Cancel submitted and on Submit sent twice |
| T09a | 3 | PASS | |
| T09b | 1 | FAIL | the lightbox photo was cropped, the toolbar kept last frame's anchor, Escape missed a spotlight whose target held focus, a disabled Next dropped focus, and a dead arrow's press closed the lightbox |
| T09b | 2 | FAIL | Enter on a focused Back that returned the tour to its first step removed the button and lost focus, so the arrows stopped |
| T09b | 3 | FAIL | Left from a focused Back also removed it and lost focus; take_focus now takes focus back whenever the focused element leaves the tree (fixed after the cap) |
| T10a | 1 | FAIL | a second press on a leaving toast's Undo ran it again, EmptyState's prose width was a literal, and two gallery callouts described behavior the code lacks |
| T10a | 2 | FAIL | a gallery callout said reduced motion settles every animation, but the undo line keeps its real time |
| T10a | 3 | PASS | |
| T10b | 1 | FAIL | the state view's prose and detail widths were literals again, and pad put zeros ahead of a minus sign |
| T10b | 2 | FAIL | the error chain did not wrap inside its narrowed box, and ConnectionStatus's doc said OfflineIndicator shows only while offline |
| T10b | 3 | FAIL | a wrapped body did not push the actions down; the prose is now a plain block, and a test compares a short and a long body's height (fixed after the cap) |
| T11a | 1 | FAIL | the shimmer band crossed the container's rounded corners, and the skeleton card's picture height was a literal |
| T11a | 2 | FAIL | a rounded_full shimmer used the raw 9999px radius, so the band's path left the box |
| T11a | 3 | PASS | |
| T11b | 1 | FAIL | the sweeping bar grew in a tall flex column, the overlay's veil ignored rounded corners, the fail demo could not start a load, and two UploadList docs said more than the rows show |
| T11b | 2 | PASS | |
| T11c | 1 | FAIL | a drag kept a stale row index and panicked when the list shrank, a move changed each row's animation id and rebuilt its state, and repeated demo tags shared a key |
| T11c | 2 | FAIL | a row first seen hidden drew open and asked for no frame, so it lingered until something else redrew |
| T11c | 3 | PASS | |
| T11d | 1 | FAIL | the ripple drew inverted columns in a cut corner, motes crossed rounded corners, a narrow marquee jumped, confetti burst on a falling count, and the Lottie note misread velato |
| T11d | 2 | FAIL | Marquee's doc still said its content is drawn twice |
| T11d | 3 | PASS | |
| T12a | 1 | FAIL | the avatar upload took HEIC, which gpui cannot decode, an accent badge's dot matched its fill, and a square avatar's upload veil stayed round |
| T12a | 2 | PASS | |
| T12b | 1 | FAIL | the gpui test module compiled without test-support, and a long description value overflowed a narrow column |
| T12b | 2 | PASS | |
| T12c | 1 | FAIL | the gallery kept an open index past a shrunk list, and the lightbox asserted on it |
| T12c | 2 | FAIL | emptying the list removed the open lightbox without handing focus back |
| T12c | 3 | PASS | |
| T12d | 1 | FAIL | Code 128 took a set-switch character, a usage bar's float sum tripped its assert, and a dense QR floored its modules to nothing |
| T12d | 2 | PASS | |
| T13a | 1 | FAIL | Down and Cmd-A picked disabled rows, and Enter opened them |
| T13a | 2 | PASS | |
| T13b | 1 | FAIL | Down stuck on a loading row and never reached the nodes past it |
| T13b | 2 | PASS | |
| T13c | 1 | FAIL | the cursor kept its index through a re-sort, so Enter opened another file |
| T13c | 2 | PASS | |
| T14a | 1 | FAIL | numbers and words compared out of order in a mixed column, and the sort panicked |
| T14a | 2 | PASS | |
| T14b | 1 | FAIL | a removed filter rule left its typing to the next rule, and two sort rows could take one column |
| T14b | 2 | PASS | |
| T14c | 1 | FAIL | a shrunk grid kept its cursor outside, long column letters overflowed, presses landed in merged cells, Tab left the grid, and ranges counted text and blanks |
| T14c | 2 | FAIL | Tab stepped into a merge's hidden cells |
| T14c | 3 | PASS | |
| T15a | 1 | FAIL | an empty chart's SVG export panicked, and a huge flat domain looped forever making ticks |
| T15a | 2 | PASS | |
| T15b | 1 | FAIL | a pointed part outlived shrinking data, an empty heatmap made up a cell, a negative total flipped its bar, and a sunburst child filled its parent's arc |
| T15b | 2 | PASS | |
| T15c | 1 | FAIL | eight charts kept a pointed part past their data, and the Gantt today line ran past the timeline |
| T15c | 2 | PASS | |
| T16a | 1 | FAIL | an unknown system zone read as UTC, the view window kept its span when data shrank, and the volume profile dropped narrow and flat candles |
| T16a | 2 | FAIL | synced charts moved their shared window once per chart for one append |
| T16a | 3 | FAIL | a lagging synced chart moved the shared window again on every redraw; fixed after the cap by tracking each chart's own count, verified by a red-green test, no fourth review |
| T16b | 1 | FAIL | Fibonacci labels spilled out of their pane |
| T16b | 2 | FAIL | Fibonacci labels still ran past the price pane's top |
| T16b | 3 | PASS | |
| T16c | 1 | FAIL | the DOM ladder lost its tick's decimals, and the tape flashed only once |
| T16c | 2 | FAIL | ladder prices lost places their tick needed |
| T16c | 3 | FAIL | a step under 1e-10 showed no places; fixed after the cap by keeping places by magnitude, verified by a red-green test, no fourth review |
| T16d | 1 | FAIL | typed numbers reached the ticket only on blur, so a press sent the old value |
| T16d | 2 | PASS | |
| T16e | 1 | FAIL | a smaller loss than expected read as a miss |
| T16e | 2 | PASS | |
| T17a | 1 | FAIL | hidden numbers crashed the gutter; an edit with nothing to change tripped an assert; a read-only editor moved its caret and undid; a hint past a multi-line ghost fell off its row; the newest cursor lost the lead on merge; a slow composition left its pinyin in undo; gpui tests broke a featureless test build |
| T17a | 2 | FAIL | a selection touching the newest caret took the lead; a composition cleared to nothing stayed open and kept later edits out of undo |
| T17a | 3 | PASS | |
| T17b | 1 | FAIL | two calls of one name shared a tree key; references counted lines, not matches |
| T17b | 2 | PASS | |
| T17c | 1 | FAIL | a match on trailing spaces ran past its trimmed line; `**/` needed a folder |
| T17c | 2 | PASS | |
| T17d | 1 | FAIL | a path match marked a project name mid-character; pin and remove also opened the row; number settings showed no places |
| T17d | 2 | PASS | |
| T17e | 1 | PASS | |
| T18 | 1 | FAIL | a missing folder ran in the app's own; combining marks dropped; Tab skipped the terminal; its wheel scrolled the page too; clear kept find matches; New bash reused the old shell; the dark script typed into the replay |
| T18 | 2 | FAIL | a folder it could not enter still started the shell elsewhere; links read cluster cells as spaces |
| T18 | 3 | PASS | |
| T19 | 1 | FAIL | a merge rewrote line endings and added a last newline; the split view numbered its right side from the old file; an empty side started its hunk a line late; the subject counter covered long subjects; clipped code rows had no line height |
| T19 | 2 | FAIL | a lone return split lines for similar but not for regions, so a change was lost; taking both sides added a newline to an empty side |
| T19 | 3 | PASS | |
| T20 | 1 | FAIL | the demo picked past memory's end; an empty gutter had no height to press; a breakpoint's box also opened it; zoomed ticks lost their places; docs promised bound keys and batch actions |
| T20 | 2 | PASS | |
| T21a | 1 | FAIL | Enter kept a selection; undo of a split left focus on a gone field; italic inside bold ate the bold; keys went into merged cells; undo kept a changed kind's highlighter |
| T21a | 2 | FAIL | undo of typing lost the selection it replaced; Up from below a table landed in a hidden merged cell |
| T21a | 3 | FAIL | undo after a delete or a paste lost the selection they replaced; fixed after the cap by recording the replaced selection where every TextInput edit passes, verified by a red-green test, no fourth review |
| T21b | 1 | FAIL | a page zoomed wider than the view sat past its left edge; a new hit at the same index kept the old page in view; a hit current from the first frame only scrolled its page to the top; the chapter list did not scroll; Escape could not leave zen turned on from a button |
| T21c | 1 | FAIL | the gallery's page properties dropped a new due date and tags |
| T21b | 2 | PASS | |
| T21c | 2 | PASS | |
| T22 | 1 | FAIL | a remote cursor scrolled out sideways still drew its flag; presence avatars showed no focus ring; the access list hid roles it could not change; status dots used a fixed size |
| T22 | 2 | PASS | |
| T23a | 1 | FAIL | a long message of yours overflowed a narrow column; pictures and video sized to the theme, not their column; a finished stream kept revealing; avatar pictures from the web were read as files; the list stayed away after it fit again; the reasoning toggle had no Tab stop; a link's thumbnail lost its corners |
| T23a | 2 | PASS | |
| T23b | 1 | FAIL | source cards and web results with a handler were no Tab stops; in a narrow column the sources row and a passage's header ran past their edge |
| T23b | 2 | FAIL | the gallery's narrow passage built a vec of one range, which clippy refuses, so the checks failed |
| T23b | 3 | PASS | |
| T23c | 1 | FAIL | suggestion arrows landed on disabled rows and left the cursor out of view; a dot ended a file mention; the drop veil never showed, since gpui styles a drag-over only on an element with a hitbox; a parameter's places came from its step alone; the gallery's prompt menu and Reset only logged |
| T23c | 2 | FAIL | a slash inside a path read as a new trigger, closing the file suggestions; a list whose first rows were disabled opened with its cursor out of view |
| T23c | 3 | PASS | |
| T23d | 1 | FAIL | the list grouped days in a silent UTC fallback when the system zone was unknown; Combobox and MultiSelect opened a long list at its top, away from the choice |
| T23d | 2 | PASS | |
| T23e | 1 | PASS | |
| T24a | 1 | FAIL | tall arguments pushed the approval dialog's title and buttons off screen; a long tool name in a narrow column pushed the time and chevron out of the card; the dark capture closed the card the light one opened |
| T24a | 2 | FAIL | a dialog body's children shrank to the capped column instead of scrolling: a 2000px body drew at 860px and a scroll moved nothing |
| T24a | 3 | PASS | |
| T24b | 1 | FAIL | a task of one step crashed at render: its bar asked for one segment, and a segmented bar needs two |
| T24b | 2 | PASS | |
| T24c | 1 | FAIL | at 280px the path vanished and Accept all drew outside the box; the diff's fold rows took presses but no Tab |
| T24c | 2 | PASS | |
| T24d | 1 | FAIL | at 280px eight thumbnails ran past the browser card; the artifact panel lost its title and its controls ran out of the box |
| T24d | 2 | FAIL | Tab to a later thumbnail left the strip where it was, the focused frame and its ring clipped |
| T24d | 3 | FAIL | reviewed by Claude Fable 5.1 max after codex ran out of quota: the shown thumbnail wore the focus color, so focus and choice looked alike; frames were forced to 16:10 and cropped, and a pointer's share missed its target. Fixed after the cap: the shown frame takes accent, and BrowserPreview and ComputerUseViewer take the host's ratio (test a_screen_takes_its_pictures_shape); no fourth review |
| T24e | 1 | FAIL | cpu() clamped a share past 1 in silence; the registry's grouping by source had no test |
| T24e | 2 | PASS | |
| T24f | 1 | FAIL | Tab to a sortable header past a narrow table's edge left it out of view; a virtualized table still drew past its box at 280px |
| T24f | 2 | PASS | |
| T25a | 1 | FAIL | Cmd-Enter generated only while a text field held focus; the form set no key context of its own |
| T25a | 2 | PASS | |
| T25b | 1 | FAIL | a done result with nothing to open still took focus on a press and drew a ring the keys could not reach |
| T25b | 2 | PASS | |
| T25c | 1 | FAIL | a canvas with no handlers still took Tab, the pen and the ring; strokes from the host went unchecked |
| T25c | 2 | PASS | |
| T25d | 1 | FAIL | the timeline's strip never scrolled, its child stretched to the box; a focused shot past the edge stayed hidden; many voice tags ran past a 280px row |
| T25d | 2 | PASS | |
| T25e | 1 | FAIL | a description row never wrapped, so a 280px model card broke its facts mid-word; the fine-tune bar's share had no test |
| T25e | 2 | PASS | |
| T25f | 1 | FAIL | a drag on the turned embedding froze past the plot's edge; which version a diff sets against, and its heading, had no test |
| T25f | 2 | PASS | |
| T26a | 1 | FAIL | the viewer's picture had no id, so an animated picture never left its first frame |
| T26a | 2 | PASS | |
| T26b | 1 | FAIL | a shaped crop's corner ignored the pointer's move up or down |
| T26b | 2 | PASS | |
| T26c | 1 | FAIL | the caption faded with the controls; Space on a focused bar button acted twice; Command letters fired the player and the strip; a long caption, cue words and the scrubber's tip ran past their boxes at 280px |
| T26c | 2 | FAIL | the bar's fade restarted an animation id per rest and wake, which rebuilt the scrubber's drag, hover and focus under it |
| T26c | 3 | PASS | |
| T26d | 1 | FAIL | the playlist's keys started from the first row, not the song under way; the waveform's bar fit sat untested in paint |
| T26d | 2 | PASS | |
| T26e | 1 | FAIL | an option list had no width limit, so a long device name ran it past the window |
| T26e | 2 | PASS | |
| T27a | 1 | FAIL | a tile's long name leaned on gpui's line_clamp, which cut its second line at both ends and lost the extension |
| T27a | 2 | PASS | |
| T27b | 1 | FAIL | the listing's fixed columns left no room for names at 280px; a column keyed by its depth handed its cursor to the next folder there |
| T27b | 2 | PASS | |
| T27c | 1 | FAIL | recent and found files were keyed by their place, so the cursor left a file an open moved |
| T27c | 2 | PASS | |
| T27d | 1 | FAIL | a press on a finished download's Show button also opened the file through its row |
| T27d | 2 | PASS | |
| T28a | 1 | PASS | |
| T28b | 1 | PASS | |
| T28c | 1 | PASS | |
| T28d | 1 | PASS | |
| T28e | 1 | FAIL | a GIF focused past the fold stayed out of view |
| T28e | 2 | PASS | |
| T29a | 1 | FAIL | Space on a focused star also toggled the list's selection |
| T29a | 2 | PASS | |
| T29b | 1 | FAIL | a folded message lost focus as it opened; a long name beside a long date ran past a 280px fold |
| T29b | 2 | PASS | |
| T29c | 1 | FAIL | a long chip ran past its tag field at 280px |
| T29c | 2 | PASS | |
| T29d | 1 | FAIL | the time dialog's rule for a time ahead of now had no test; its test found that a picker opened in a dialog panicked on a deferred draw inside another |
| T29d | 2 | FAIL | a raise inside a raise took its place in the queue as its namespace, so one appearing before it rebuilt its state and dropped its focus |
| T29d | 3 | PASS | |
| T30a | 1 | PASS | |
| T30b | 1 | FAIL | the week and day views froze the line at now and today: the moment read once went down as given and turned the ticker off |
| T30b | 2 | PASS | |
| T30c | 1 | PASS | |
| T30d | 1 | FAIL | a chosen day outlived the owner's days, and the next render panicked |
| T30d | 2 | PASS | |
| T31a | 1 | PASS | |
| T31b | 1 | FAIL | a focused card past the board's fold stayed out of view; left and right steps that keep a card's place went untested |
| T31b | 2 | PASS | |
| T31c | 1 | FAIL | a roadmap bar's end and the workload bar's cap went untested |
| T31c | 2 | PASS | |
| T31d | 1 | FAIL | a focused segment past the strip's fold stayed hidden; the tracker lost focus as Start and Stop swapped; Stop and Skip went untested; percent did not refuse content |
| T31d | 2 | PASS | |
| T32a | 1 | FAIL | the plain wheel panned against every scroll box; the handlers panned from the render's viewport, so events within a frame overwrote each other; the fit's cap at one went untested |
| T32a | 2 | PASS | |
| T32b | 1 | FAIL | handles drawn for a multi-selection fell through to a marquee; the palette ran past a 280px box; an open pen path outlived the tool |
| T32b | 2 | PASS | |
| T32c | 1 | PASS | |
| T32d | 1 | FAIL | Tab in a mind map's rename field added a topic; the replaced wire, the connector released on its own shape and a deleted shape's links went untested |
| T32d | 2 | FAIL | Enter or Escape in a canvas field handed focus nowhere, so the map's and the layer's keys went dead until the next press |
| T32d | 3 | PASS | |
| T33a | 1 | FAIL | the plan, the Redis browser and the structure editor drew past a 280px box; the form's rules and a row button's press went untested |
| T33a | 2 | FAIL | the structure editor's cells kept the narrow box's first measure, and its Tab stops past the sideways fold stayed out of view |
| T33a | 3 | PASS | |
| T33b | 1 | FAIL | YAML lists at their key's indent and quoted `#` misread; a long note hid the key at 280px; XML text paths counted from 0 and entities stayed raw; odd-key paths and Format at rest untested |
| T33b | 2 | FAIL | quoted YAML scalars kept their escapes and single-quoted keys their quotes; the comment scanner's escape and after-a-space rules untested |
| T33b | 3 | FAIL | five escape arms untested and a quoted key holding `: ` split inside its quotes; both fixed after the last round with tests that go red on each mutation, no fourth round |
| T33c | 1 | FAIL | the socket log stayed at its top; a revealed secret stayed open across environments; the collection drew drags it never moved; the builder's refill, Send's rest and Bearer, the indented body, the eye and a field's pick path untested |
| T33c | 2 | FAIL | where an Inside drop lands in a folder that holds something went untested |
| T33c | 3 | PASS | |
| T33d | 1 | FAIL | text fields never painted a highlight's wash; a tile's value ran past 280px; a stray + read as a hex digit; the WCAG thresholds, an exit's badge, a stopped row's load and the meter's clamp untested |
| T33d | 2 | PASS | |
| T34 | 1 | FAIL | a focused tile's Left and Right, and the guard that leaves arrows to a focused control inside a card, untested |
| T34 | 2 | PASS | |
| T35 | 1 | FAIL | a row's control kept its full width past a 280px box; a notice's name shrank to nothing; privacy, notices and startup dropped edits with no handler; Apply's and the reset's rest and a box turned off untested |
| T35 | 2 | FAIL | an accent picker of nine or more presets still ran past a 280px box |
| T35 | 3 | PASS | |
| T36a | 1 | FAIL | Sign in waiting for a password and resting while busy, Create account waiting for a name, the other providers resting, and Send again resting while busy untested |
| T36a | 2 | PASS | |
| T36b | 1 | FAIL | a long key name hid its ends at 280px; the secret and ends skipped typography::literal; this device's place first and the name clearing after Create untested |
| T36b | 2 | PASS | |
| T36c | 1 | FAIL | a long quota name pushed its count past 280px; a month past 12 read as passed; the Amex shape, the address clearing, the clamped share and the paywall's benefits untested |
| T36c | 2 | PASS | |
| T37a | 1 | FAIL | the wizard's Next and Back dropped focus as the flow turned; Got it stranded focus on a closed tip; the words read and the bar's share untested apart from their selectors |
| T37a | 2 | PASS | |
| T37b | 1 | FAIL | the help panel's rows were bare Tab stops, not a list the keyboard walks; a long support address drew past 280px |
| T37b | 2 | PASS | |
| T38a | 1 | FAIL | the resize grip and edge handles hung half outside the box; the edges' sides, a free turn and a band with Shift untested |
| T38a | 2 | PASS | |
| T38b | 1 | PASS | |
| T39a | 1 | FAIL | a larger text size cut the density strip at 280px; ten palette colors had no name; the density and sliders untested |
| T39a | 2 | FAIL | segments split the strip equally and cut a longer label; the thumb lost its padding's height |
| T39a | 3 | PASS | |
| T39b | 1 | FAIL | a long theme name pushed its badge past 280px; four-digit colors refused the theme |
| T39b | 2 | PASS | |
| T39c | 1 | FAIL | Windows caption buttons took their row's height; a long Windows title pushed the buttons past the bar |
| T39c | 2 | FAIL | an untitled Windows bar lost its spacer; a centered title ran under the Linux buttons; caption buttons ignored the window's rem |
| T39c | 3 | PASS | |
| T40a | 1 | FAIL | a wrapped right-to-left line came out garbled, since gpui wraps glyphs in drawn order; a Hebrew minus sat right of its digits; two dates claimed CLDR's short year; two catalog guards untested |
| T40a | 2 | FAIL | a Hebrew line that opened with a signed number laid out left to right, since gpui sets no paragraph direction |
| T40a | 3 | PASS | |
| T40b | 1 | FAIL | the gallery's blocked note could not wrap, so the a11y note ran past its column and lost its dot; a check between rounds failed on the reviewer's mutant left in the shared target, so reviewers now build in their own |
| T40b | 2 | PASS | |
| T41a | 1 | FAIL | off-view pins took Tab and Enter; closing the popup stranded focus at the root; a tile that failed to load logged nothing; a tall popup ran past the map and under the zoom buttons |
| T41a | 2 | FAIL | a focused pin that panned out of view lost its handle, so focus fell to the root; the popup's height cap and scrolling body untested |
| T41a | 3 | PASS | |
| T41b | 1 | FAIL | the code held, but four cluster claims had no test of the item's own: a pan keeping a group, an off-view cluster out of Tab, a folded pin's hand-off, members far apart shown whole |
| T41b | 2 | PASS | |
| T41c | 1 | FAIL | a choropleth's points and lines drew in the page color; the legend covered the attribution under 332px; WorldMap's keying, credit and XK untested; a key given twice kept the last |
| T41c | 2 | PASS | |
| T42a | 1 | FAIL | Enter on Reset disabled it and stranded focus at the root; the tickers' wakes and the lap rows had no test that could see them |
| T42a | 2 | PASS | |
| T42b | 1 | FAIL | a result went on as its twelve-digit text, so 1 ÷ 3 = × 3 = read 0.999999999999; a long reading hid digits at 280px; Equals on a dangling operator said the formula ended early; Control, the = key and a point after a result untested |
| T42b | 2 | PASS | |
| T42c | 1 | FAIL | a verdict or a new ask disabled Verify while it held focus, so focus fell to the root; Enter handed a second answer over during the check; a passed field kept its ring and caret |
| T42c | 2 | PASS | three claims untested; tests for focus after an answer and a resting pass added in the pass commit |
| T42d | 1 | FAIL | a pointer's untick disabled Submit while it held focus, so focus fell to the root; six claims untested; radio labels ran past a 280px box |
| T42d | 2 | PASS | the poll's mark and its selector now share one binding |
| T42e | 1 | FAIL | the code held, but seven claims had no test: Decline, the scrim, focus back after Accept, necessary cookies on and resting, the list's mark and its rows' lines |
| T42e | 2 | PASS | a Tab-path check for the resting Necessary switch added in the pass commit |
| T42f | 1 | FAIL | a press on the web view gave WebKit the keyboard for good, since gpui's view never takes the first responder back; in_view warned off macOS; a straddling box untested |
| T42f | 2 | PASS | off-macOS imports and fields tidied in the pass commit |
| T42g | 1 | FAIL | the unread CSV dialog cut its reason at the card's edge, since InlineMessage laid its words unwrapped; nine ways out untested; a warning logged every render; an unclosed quote named the last line |
| T42g | 2 | PASS | |
| T43a | 1 | FAIL | the knobs' starting values sat outside layout::seeded, so a new one was lost and a renamed knob failed; the redraw untested behind a refreshing helper; twin names and an out-of-range number taken |
| T43a | 2 | PASS | |
