# The UI style file, assets/ui.ron

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## The screen's colours and words are in `assets/ui.ron`

Since 2026-11-04 (Hajun: "i think ai cannot rly implement my instructions
exactly and precisely how i want it. how can i make it so i can tweak it
myself and have more of my control?"; and, of a first file that held every
size and margin as well, "too overfilled with constants ... i just want to
be able to change the colors (including transparency) of UI objects like
buttons and windows and text. easily ... AND Text. so i want to change some
of the text easily like the 'A guest account, kept on this device.' in
settings"), every colour and every word of what is drawn over the world
lives in `assets/ui.ron`, for Hajun to edit by hand: the bar across the
top and its buttons, a phone's stick and jump button, the sign-up's pages,
the settings window and its dialogs, House Builder's board, stars, dialogs
and results, the shop, talking, the edit menu's buttons, the names over
heads, the headlines and the notices, and, since 2026-11-07, the punch's
hit range drawn round you (`colours.melee`; "The town is a PvP arena in
the sky", below). Sizes, margins and layout stay in the code. `src/ui.rs` reads it: `ui()` gives the file as it stands, a
global rather than a resource, so that the plain functions every screen is
built from (`hud::words`, `builder::label`, `builder::dimmed`) read it
without every caller threading it through; and `ui::fill` fills a word's
`{something}` in ("You came {place}").

- **The file is compiled in** (`include_str!`), so a phone shows whatever
  it said when its app was built, with nothing to copy or package
  (`ui.ron` is left out of the APK as a file, being built in). `cargo test`
  reads it; a field misspelt or missing is an error (`deny_unknown_fields`),
  not silently ignored, and so is a file with no small talk in it.
- **On Windows it is read again whenever it changes on disk**
  (`ui::reread`, every quarter second by its modified time; nothing on a
  phone), and `ui::UiChanged` changes that frame, on which whatever stays
  on the screen is drawn again from it: the bar is taken down and put up
  afresh (`hud::restyle`; `settings` puts its gear back on the new bar, and
  the name and balance are filled in again), the sign-up page that is up
  is drawn again as it stands, the name typed so far kept
  (`signup::redraw`), the settings window on its tab (`settings::redraw`;
  not while Edit name or the delete question is up in its place), and the
  names over heads take the new colour (`tags::recolour`). A dialog, the
  shop, House Builder's board and results take it when they next come up.
  So with the game running from `cargo run`, save the file and look: no
  rebuild, where one took 77 s on this PC (measured 2026-11-04). A file
  that will not read is logged once, with the line and column, and the
  last good one kept, as an editor may be caught halfway through writing
  it. `RT_SIGNUP=1` opens the sign-up's pages at launch whatever the
  profile, for working on them; saving on them sets the profile as it
  would.
- **A colour is written as on the web**, `"#rrggbb"` or `"#rrggbbaa"` with
  its alpha (`00` invisible, `ff` solid), so that VS Code's colour picker
  edits it (Hajun, 2026-11-04: "is there a notepad app or some extension on
  vscode so i can edit the colors using some rgb selector"): with
  `editor.defaultColorDecorators` set to `always` (`.vscode/settings.json`,
  which git ignores, has it for this PC) every colour gets a swatch, and
  hovering it opens the picker, whose hue, saturation and alpha sliders
  rewrite the string in place; with auto save on, the running game follows
  each drag. `rgb()`, `rgba()`, `hsl()` and `hsla()` read too
  (`ui::parse_colour`), whichever format the picker is switched to.
  `(r, g, b, a)` from 0 to 1 was the first form, for an hour. What every
  screen shares comes first (`text`, `text_soft`, `gold`, `on_gold`,
  `warn`, `dim`, `dialog`, `panel`, `quiet`, `off`, `close`), then each
  screen's own. The file's comments say what each is; the history of each
  (who asked for what, when) is in this file, under the screen it belongs
  to. The builder's and the sign-up's lit gold were a shade apart (1, 0.91,
  0.55 and 1, 0.9, 0.52) and are one `gold_lit` now; and the hex form
  rounds every channel to the nearest 1/255.
- **Adding one**: a field on the struct in `src/ui.rs` (a colour with
  `#[serde(deserialize_with = "colour")]`), read with `ui()` where the node
  is spawned, and the line in the file. A screen that stays up needs a
  system that draws it again when `Res<UiChanged>` `is_changed()` (and
  not `is_added()`); one spawned on demand reads `ui()` when it opens, and
  comes up new next time. A bundle returned from a function must own its
  words (`.clone()`, not `.as_str()`): in edition 2024 an `impl Bundle`
  captures the borrow, and the `Arc` `ui()` gives would not live long
  enough.
- **And every number of how the car you drive goes** (since 2026-11-06,
  Hajun: "put all the constants related to the car in the ron file. like
  related to movement. so the turning, the boost/reverse, acc/speed,
  multipliers, etc. i want to modify the numbers by myself while playing
  on pc"): the `car` section at the end of the file, `ui::Car` (below,
  "The car you drive"), in metres, seconds and degrees, with or without a
  point (`12` and `12.0` both read). `drive::steer` and `drive::settle`
  read it every frame (`ui()` is a handle), so a save while driving
  changes the car at once, and `drive::retune` puts the camera at a new
  `overhead` and keeps it between a new `overhead_nearest` and
  `overhead_furthest`. A number the car would break on (a rate under 0,
  which would panic a clamp; `easy_speed` or `full_lock_at` at 0, which
  it divides by; a range inside out) is refused as a misspelt colour is
  (`Car::check`), the last good file kept and the field named in the log.
  The speeds may go no faster than the server lets a body go
  (`roundtown_net::cars::DRIVEN_TOP_SPEED`, 40 since 2026-11-06, when
  Hajun asked for a top speed of 40 and the server was deployed with it;
  20 until then): a test fails on a file that does, saying to raise that
  and deploy the server. Checked in the
  running game the same day: a changed `cruise` taken within half a
  second, `easy_speed: 0.0` refused, the file restored and read again.
