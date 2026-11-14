# The UI style file, assets/ui.ron

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Every window is drawn as the friends' panel is

Since the night of 2026-11-14. Hajun: "This is a design change. I think I
like the ui design/color choice/ close button layout/ etc. of the friends
tab. I want to use this same design style throughout the project. change
the other ui panels to match the current friends panel design. So i guess
change the settings tab, minigames tab, the shops in the bridge war game.
dont rebuild apk just change, test/screenshot to see if the design is
similar, fix if necessary, loop until it looks good and then finish"; and,
watching the scripted runs as they went, of the shops' cards, "the dark
part is too dark like the each items background box is bit too dark", then
"i meant too dark as in should be more transparent, not just become
grayer". `src/window.rs` (new), `src/friends.rs`, `src/settings.rs`,
`src/games.rs`, `src/market.rs`, `src/arena.rs` (the panel that says you
are out), `src/hud.rs`, `src/builder.rs`, `src/signup.rs`, `src/ui.rs`,
`assets/ui.ron`. The app's own: nothing of `shared/` or the server's, and
**no APK** (asked not to); uncommitted as this is written.

- **One module draws them all** (`src/window.rs`). What made the friends'
  panel look as it does was `friends.rs`'s own: its sizes, off Hajun's
  sketches of it (`friendtabdesign1.png` to `4`), and its small functions
  for a round button, a bare one, a button with words, the disc with
  someone's picture, a box up over the panel. All of that is `window`'s
  now, as it was written, less what a press does: each thing takes its
  press for itself (`hud::TakesPress`) and whoever spawns it puts a
  component of their own beside it. `friends.rs` draws from it as the
  others do. `pane` is the window, dark and see-through, its corners
  rounded 3 shares of the short side; `header` and `tab` the row along its
  top, the tab that is shown white with a white line under it; `cross`
  the bare white cross inside the top right corner, 8 shares across, with
  no disc behind it; then `words`, `picture`, `avatar`, `round`, `bare`,
  `choice` with its `Look`s, `link`, `card`, and `popup`, a darker box up
  over a window with the window darkened behind it. **The friends' panel
  itself is as it was**: a picture of it from before and one from after,
  offline, have every bright pixel of the panel's own in the same place
  (4,747 of them, none differing; the world behind it differs from run to
  run, so only those were compared).
- **One block of colours** (`colours.window` in `assets/ui.ron`,
  `ui::WindowColours`): the block that was `friends`, under a new name,
  with three more for cards. Change one and every window takes it. Gone,
  nothing reading them any more: `settings` (`window`, `blocked`,
  `blocked_text`), `dialog` and `warn_lit` (the settings' and the game
  selector's black boxes and the settings' red Delete were all that read
  them). `shop` is House Builder's shop's alone now, and its comment says
  so; `close` and `close_rim` are that shop's cross's.
- **The settings' window** (`settings`; "Settings" in
  `docs/town-hud.md`) is the friends' panel's size, 112 by 66, with
  nothing dimmed round it. Its tabs are words where that panel's are
  pictures, the one shown white over the white line (gold and grey pills
  until now). Accounts has your account on a row as a friend is on
  theirs: the disc with a person, its dot green online and grey not, your
  name over "(Signed in as guest)" or "Not online.", and the pencil on a
  round dark button at the row's end (it was beside the name); under it
  the two sign-ins side by side, blocked out as a card is that cannot be
  chosen; and at the foot of the window Delete account, the dark red of a
  no. **Edit name and the question before an account is deleted come up
  over the window**, as Add Friend does over the friends' panel (each was
  a black dialog in the window's place): the name in a box the grey of
  that panel's, "Pick another name." under it from the left, Cancel the
  dark red and Save the green, greyed while what is typed is no name
  (`Look::Greyed`). Cancel, Escape and a tap on the dark round such a box
  go back to the window; from the question that shut the settings
  altogether until now. Edit name slides up over a phone's keyboard as
  the friends' boxes do, its top never off the top of the screen
  (`LiftsForKeyboard::whole`).
- **The game selector** (`games`) is a window as wide, "Games" along its
  top from the left and the cross in its corner, its three cards sharing
  its width, 30 tall. "Need 100 coins" and "Only your group's leader can
  start a game." are the pale gold of the friends' line that says what
  came of something (`window.notice`; the red of a warning until now,
  which on the dark was hard to read), and the leader's line is along the
  window's bottom, where that panel's is.
- **Bridge Wars' two shops** (`market`) keep every card's size and place,
  and so the widths the tests hold them to. Their title is along the top
  from the left, in a row 6 tall, and the cross is in the corner, 7
  across with its middle 4.8 in: the window's margin stays 1.8, half the
  friends' panel's, three rows of cards having to fit between the time
  and the bottom of the screen short of the buttons on the right. **The
  panel that says you are out** (`arena::spawn_panel`) is laid out as one
  of the friends' questions: the headline and the line under it in the
  window's white, Exit the dark red of a no and Play again the green of a
  yes (a gold headline and round-ended grey and gold buttons on black
  until now).
- **A card is see-through** (`window.card`, `card_lit`, `card_off`: a
  game's in the selector, a thing's in a shop). The first cut had them
  the solid dark of the friends' round buttons, `#121212`; Hajun, seeing
  the shop go by: "the dark part is too dark like the each items
  background box is bit too dark". I made them a grey (`#2b2b2b`, then
  `#333333`), which was wrong: "i meant too dark as in should be more
  transparent, not just become grayer". They are that dark half
  see-through now, `#12121280`, so that the window and the island show
  through; one that cannot be chosen is `#12121233`, nearly clear, with
  faint words; lit under the pointer, `#55555580`. The alpha is the last
  two digits, there to be changed. A sign-in that is not to be had looks
  as a card does that cannot be chosen (`Look::Blocked`).
- **What is mine**: that there is one block of colours rather than a
  copy for each window, and its name; the settings' window being the
  friends' size and its Accounts tab laid out as a list of friends is,
  with the pencil at the row's end; that a tap off the settings or the
  selector shuts it as before, on the finger coming up, and is not also
  the stick's, where the friends' panel shuts as the finger lands and
  passes the press on (asked for that panel alone, 2026-11-13); **Delete
  and its question being dark red and a quiet grey, never the green of a
  yes** (the friends' own "Remove ... from your friends?" has a green
  Yes; an account going for good is not one to invite a press on); Exit
  red and Play again green; the shops' narrow margin and smaller cross;
  the pale gold for what says why something cannot be had; taking out
  what nothing uses any more rather than leaving it (`builder::dimmed`
  and `choices` with their six sizes, `hud::close_cross`,
  `signup::gold_button`, the colours above).
- **Left as they were, not being windows or not asked about**: House
  Builder's shop, board and results (stalled; `shop.rs` draws its own
  grey window and its own cross on the corner), the sign-up's pages,
  talking's paper, the plates over the world (your name, the time, the
  minimap: the grey of 2026-11-11), the chat, and every headline, gold
  as they were.
- **Checked.** Unit tests (the game's, 303; `window` has two of its own,
  and `settings` one). **In the desktop build, scripted**, a throwaway
  module pressing the buttons by touches and saving pictures, taken out
  again: the settings on each tab, Edit name typed into, the question,
  back out of both; the selector; both shops with things bought, owned and
  out of reach; the panel that says you are out (a game begun at 10:05
  and a fall); each on a window of sixteen by nine, on one shaped as a
  phone (19.5 by 9) and, the town's, as a tablet (4 by 3). **And every
  button by a touch on it**, what came of each read back from the game:
  the pencil, a name typed and Save (the name on the row and on the plate
  changed; Save grey while the box was empty), Cancel, a tap on the dark
  round the box, Delete account and its Cancel (back to the window), the
  cross, a tap on the window (nothing) and one off it (shut), for the
  settings and for the selector; Bridge Wars' card (into the waiting
  room); a card in the shop (an amethyst gone and a Builder on a button)
  and the shop's cross. **Against a server
  on this PC**: the friends' panel with friends, an inbox and a group put
  into it by that module (made up, never sent), its Add Friend, its
  question and its envelope, to hold the others against; the settings
  online, with the green dot and "(Signed in as guest)"; and the selector
  as someone in a group they do not lead sees it.
- **Not checked.** Anything on a phone. Edit name over a real keyboard
  (it slides as the friends' boxes do, which have not been seen on a
  phone either). A card or a button lit under the pointer (`card_lit`
  is a guess). The selector without the coins for Home Builders. The
  file saved while a box is up over the settings (it is drawn again with
  what is typed kept, by the code; not tried). On a tablet's four by
  three the shops' window lies over the minimap and the stick, as it did
  before this.

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
