# The opening, the buttons round the screen, and the settings

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## The settings' window opens on its first tab (2026-11-27)

Hajun, of the friends' panel: "each time u open the social panel, it
should be the friends tab not the last tab u opened ... this goes for the
settings page too". The gear opens the window on Graphics every time
(`settings`, `Choice::Open`), where it opened on the tab last shown. The
panel's half is in `docs/friends.md`, at the top. Not in an APK.

## The cover stays until the server has said what the town is

2026-11-24, late, after APK 53 (`docs/land.md`'s first entry). Where
everything in the town stands is thrown afresh as a channel opens, and
the town the device fills by itself with no server is in another order:
so with the cover gone before the server had spoken, you were stood on
another plot and every other plot filled again, in sight, a second or two
after the town came up. That was put to Hajun with the way round it
(holding the cover until the server answers, or two seconds pass), and
he: "yeah hold black cover until the server answers, rebuild apk an
dpush." `src/intro.rs`, with a word each from `net` and `land`. The app's
own: APK 54 is the first with it.

- **The cover waits for one thing more** (`Loading::answered`,
  `Wait::step`): the server's word on the town, which is whose every plot
  is (`land::Told`: `Tell::Plots` has come) and what stands on each
  (`bag::Theirs` has every one of those owners in it; the server says it
  of every plot, with something on it or not). What then stands on the
  plots is a model like any other, and the cover waits for those as it
  always waited for every model (`Loading::there`), and for the GPU's
  eight quiet frames after.
- **Only while there is a server that may answer** (`net::Link::hopeful`):
  it is being tried for the first time, or it is connected. Not with no
  server at all (`RT_SERVER=off`), not once every address has failed (no
  network: the login fails at once, and it rests before trying again),
  not with a server newer than the app, nor once the server has put you
  somewhere that is not the town (`net::Online::elsewhere`: back in a
  game of Bridge Wars your seat waited in).
- **And no longer than two seconds once everything else is there**
  (`SERVER_WAIT`): a slow network shows the town as the device has it and
  changes it when the server answers, as before. The cover's own most, ten
  seconds, is as it was.
- **This bends "the app never waits for the network"** (`CLAUDE.md`,
  "Playing online"), by those two seconds at the most, at his word. With
  no network the app opens exactly as it did.

**Calls that were mine.** Two seconds, counted from when everything else
is there and not from when the app opened (a phone takes longer than that
to load the town, and an answer comes in tenths of a second where there
is a network: it is nearly always there first). Not waiting again once
every address has failed, though it tries again four seconds on: someone
with no network is not to look at black for two seconds each time they
open the app. Not waiting for a town you are not in.

**Checked**, in the desktop build, hidden, by a throwaway that logged
every fifth of a second how solid the cover was, which plot you had and
whose the plots were, taken out again:

- **The VM**, this PC's own account: the server's word 1.67 s in, under
  the cover (from the device's plot 31 to the server's plot 15, the two
  real people's land come); the cover opened at 3.71 s, the buses' way
  being the last thing there, and nothing moved after it. Not a moment
  was spent waiting for the server alone.
- **No server** (`RT_SERVER=off`): covered 3.73 s, as it was.
- **A server that refuses at once** (nothing listening on this PC's
  port): no wait, covered as long as with none.
- **An address that swallows everything and says nothing**: everything
  else there at 2.77 s, the cover open at 4.97 s, having waited for the
  server alone its two seconds and the quiet frames after.
- **A server held still until after everything else was there** (one on
  this PC, its process stopped and let go): everything else at 3.67 s,
  the server's word at 4.38 s, the cover solid while you were moved from
  the device's plot 34 to the server's 38 and the plots became the
  server's, open at 4.90 s. Without this it would have opened at 3.8 s
  and the move been seen.
- The tests: the cover stays for the server past the frames it would have
  opened in, through the loading of what the server says stands, and
  opens after; never answered, it opens two seconds after everything
  else, and none of the time the town took counts; with none to wait for,
  as it always did.

**Not checked.** On a phone, where the town takes longer to load than on
this PC and the wait should matter less. The sign-up's pages, the first
time the app is run (the wait is theirs while they are up, as before). A
seat waiting in a game of Bridge Wars.

## The one button by the left edge moved up, and a cross beside the colour

2026-11-22, night (`docs/bag.md`, the entry marked "night", has Hajun's
words and what was checked). `src/hud.rs`, `src/customize.rs`.

- **The one button**, the house and pencil while you stand on your plot,
  is where the brush is among the three: from 38.75 to 49.75 of 100 down
  ("a bit too down, i think u can move it up a bit ... the location
  right now where the paint brush icon is, that is where the original
  house edit button is"). The three are where they were: the house and
  pencil over it, the tick under it.
- **The chat's lines begin there**, and step aside of the one button as
  they do of the three (`customize::aside`, `chat::Aside`).
- **Beside the brush, two round buttons**: the colour, and a cross that
  takes paint off; the one the brush paints with has a ring round it.
- **The edit menu's four buttons** are 10.5 across, as the two people and
  the gamepad are, and stand just clear of the thing as it is drawn
  (`src/editor.rs`).

## The door where the bag's button is, and edit mode's buttons by the left edge

2026-11-22, evening (`docs/bag.md`, the entry marked "evening", has
Hajun's words, all of it, and what was checked). `src/hud.rs`,
`src/customize.rs`, `src/chat.rs`.

- **The door out of your car** is among the buttons round the jump
  button, where your bag's button is while you are out of it
  (`hud::BAG_PLACE`), and as big: "the leave door/car button instead of
  left center, just put it where the inventory ui supposed to be cuz rn
  that slot is empty".
- **By the left edge** (`hud::LeftEdge`, a column that grows up from 62
  of 100 down, which `customize` fills): one button, 11 across, while you
  stand on your plot, the house and pencil; and in edit mode three, the
  house and pencil, the paintbrush and the tick, with a round button the
  brush's colour beside the brush. **The one is from 51 to 62 down**, not
  half way: the chat's lines are by that edge from 38.5 to 48.6, and the
  stick from 64 (mine). The three reach up to 26.5, under your gems.
- **The chat steps aside** to the right of the three for as long as they
  are up (`chat::Aside`, 22.5 shares): they stand where its lines begin.
- Three pictures drawn in code: `hud::customize`, `hud::brush`,
  `hud::tick`.
- **Not on a phone.**

## A box on the bag's button, and your coins and gems one size

2026-11-22, afternoon, after Hajun had seen APK 46 (`docs/bag.md`, the
entry marked "afternoon", has their words and what was checked).
`src/hud.rs`.

- **The button for your bag has the box of its window's first tab on it**
  (`hud::cube`): "i dont like the bag icon for that inventory panel. just
  reuse the box icon u used for the inventory tab". The sack is kept
  (`hud::bag`, `hud::SACK`).
- **Your coins' number is the gems' size**, 3.8 (`BALANCE_TEXT`; 5.0
  until then), and **the gem is as wide as the coin** (`GEM_WIDE`): its
  picture is nearly all gem where the coin's has clear round the coin, so
  drawn the same width it came out a twelfth wider. With the smaller
  number the row of your coins is as tall as the coin and no taller, so
  the coin stands half a share higher than it did and the gem a share
  higher (3 and 6 dots at 603 down, read off the runs before and after).
- **Not on a phone.**

## The face is a bag, the car has a door, and your gems are under your coins

2026-11-22, with the bag (`docs/bag.md` has Hajun's words and the whole of
it). `src/hud.rs`.

- **The face among the buttons round the jump button is a bag**
  (`hud::BagButton`): the same place and size, the sack that was drawn in
  code for Bridge Wars' first sketch (`hud::bag`). It opens your bag
  (`bag::panel`). Only in the town, as the face was, and **put away while
  you are in your car** (mine).
- **In your car a door stands half way down the left edge**
  (`hud::ExitButton`, `LeftEdge`; Hajun: "a new ui button around left side
  center of your screen that is like a leave door icon"): Material's
  `logout`, `icons/leave.png`, 12 across, as far in from the edge as your
  name and kept clear of a camera's hole as the bar is
  (`safe::KeepsClear`). A press gets you out (`drive::CarPressed`). Its
  bottom is eight shares over the stick's top on a phone, and it is put
  away with the buttons round the jump button while the camera is on
  someone you talk to.
- **Your gems are under your coins** (`hud::gems`, `GemsText`,
  `assets/gems.png`), the gem as wide as the coin and the number three
  quarters as big, as drawn in `bagandinventorypanel1.png`. They are your
  bag's (`bag::Bag::has`), ten to begin with.
- `icons/mood.png`, the face, is on nothing now, as `chat.png` is.
- **Checked**: unit tests (the bag up only in the town and out of the car,
  the door up only in it and clear of the stick and of your coins, the
  pictures drawn); the hidden desktop build, pictures looked at enlarged
  beside Hajun's (`docs/bag.md`). **Not on a phone.**

## The town's name goes at once when a window comes up over it

The evening of 2026-11-18, with the four games' asks (`docs/minigames.md`).
Hajun: "also the Donggeurami Town intro text when u open the app should
automatically fade right away when u open some menu like game or setting or
friends". Until then "Donggeurami Town" stood over the game selector, the
settings or the friends' panel for the rest of its three seconds.

- **As the app opens** (`src/intro.rs`): once you are let go, the first
  frame anything holds `hud::DialogUp`, the name goes from however strong
  it is then to nothing in `hud::HUSH_SECS`, a quarter of a second, and
  never comes back up (`Intro::hushed`, `title_solid`, `named`, a test);
  the opening is over as soon as it has gone. The sign-up's pages, which
  hold `DialogUp` too, do not count: the name never stands over them.
- **Back in the town** (`src/hud.rs`): the same name said again as a
  welcome as you come back from a game goes the same way under a window
  (`hush_welcome`, before `fade_headlines`; `Headline::out`, `solid`, a
  test). Nothing else said in a headline is hushed.
- **Mine**: a quarter of a second, half the name's own going or less
  (a test holds it so); every window that holds `DialogUp` hushes it, the
  chat's box and a talk too, not only the three Hajun named; and the
  welcome back in the town as well as the opening's.
- **Seen hidden** (`RT_BACKGROUND=1`, offline, a throwaway driver tapping
  the gamepad through the window's own touch events): the name full up
  over the town; 0.12 s after the tap, half gone over the selector, its
  white down from 12,706 pixels to 1,067 in its band (enlarged); at 0.6 s
  gone, the selector clear. Not seen on a phone.

## The cover is plain black, and fades

2026-11-17, with the four buses of Bridge Wars (the first entry of
`docs/bridge-wars.md`). Hajun: "Remove the initial loading sceen with the
black background and the red loading button. Remove the initial loading
falling down/landing animation or the circular expanding hole animation.
Just make it stay on a empty black screen until the map/world is loaded,
and then transition." `src/intro.rs` alone. The app's own: APK 40 is the
first with it.

- **The cover is black with nothing on it** from the first frame until the
  town is there, waiting for exactly what it waited for before
  (`Loading::there`, `Wait`: the town's shape, every model, the buses' way,
  your body, eight quiet frames of the GPU; at most 10 s). It no longer
  waits for a bar to fill, there being none.
- **Then it fades**, over half a second (`FADE_SECS`, `fade`: slowly away,
  quickly through the middle, slowly out), with you standing where you
  arrived; you can move once it has gone (`FREE_AT`). No circle, no drop,
  no squat, no ring of dust, no dip of the camera.
- **Turned off, not gone** ("remove" being "turn off" here): "Loading..."
  and its bar behind `SAYS_LOADING`, the circle and the drop behind
  `DROPS_IN`, both `false`; every number, word and colour of them is where
  it was (`assets/ui.ron`'s `words.loading` and `colours.intro` too).
- **Calls that were mine**: the fade, for "transition", and its half
  second; that the town's name still comes up as the cover goes, stays
  2.2 s and fades, as it did after the circle (it was the welcome, not the
  loading screen: say so and it goes too); you free once the cover has
  gone rather than as it begins to.
- **Checked**, in the desktop build, hidden (`RT_BACKGROUND=1`), by a
  throwaway that had the game save a picture every 50 ms of its first 8
  s, in a window of the phone's shape, taken out again: every picture
  pure black (0, 0, 0) until the town was there (4.25 s that run, the
  PC busy with 96 whole games at the time), then nine pictures through the
  fade, the town darkened and then clear, you where you arrived, the
  town's name over it, the buttons there. Unit-tested: no words and no
  drop, the fade whole to gone in `FADE_SECS` and never back, you free
  once it has gone. **Not on a phone.**

## The cover says it is loading

2026-11-16, with the buses of Bridge Wars (the first entry of
`docs/bridge-wars.md`). Hajun, with a picture of it put in the repo's
root, `loadingscreendesign.png`: "also use loadingscreendesign.png for when
u open the game - a minimalistic quick loading indicator (before the fall
from sky and circle out animation)". `src/intro.rs`, `src/ui.rs`,
`assets/ui.ron`. The app's own: nothing in `shared/`, no deploy. APK 38 is
the first with it.

- **The picture**: 852 by 393, a phone's screen on its side: near black,
  "Loading..." in white in the middle of it, and along the bottom a light
  grey bar with round corners, nearly as wide as the screen, filled red
  from the left, with "25%" in black in its middle.
- **It is on the cover the app opens under** ("The app opens under a
  cover, and you drop into the town", below), from the first frame until
  the cover begins to open: over the cover and under the sign-up's pages
  (`Loader`, layer 27). The word is `words.loading` and the four colours
  are `colours.intro.loading`, `bar`, `bar_fill` and `bar_text` in
  `assets/ui.ron`: white, `#dddddd`, `#ff4141` and black, read off the
  picture. The sizes are measured off the picture and are shares of the
  screen, so that it is the picture on a screen of any size
  (`LOADING_TEXT`, `BAR_SIDE`, `BAR_UP`, `BAR_THICK`, `BAR_ROUND`,
  `BAR_TEXT`): the bar 4.1% of the width in from either side, 9.4% of the
  height up from the bottom, 9.2% thick, its corners 4.1% round (35, 37,
  36 and 16 of the picture's dots); the word 12 of the short side and the
  share on the bar 5.8.
- **The bar is how much of what the cover waits for has come**
  (`Loading::share`, `SHARES`): the town's shape read is 15 of the
  hundred, every model spawned 50 (by how many of them are), the buses'
  way round worked out 15, your body on its feet 5, and the GPU with
  nothing left to compile 15 (by how many of its eight quiet frames have
  passed). The share written on it is the same, in whole hundredths, and
  the bar is laid out again only as that changes.
- **It is always seen to move, and never goes back** (`loaded`). What it
  shows goes toward what has come at 1.2 of the bar a second at the most
  (`BAR_FILLS`), a frame that hung counting as a thirtieth of a second:
  here everything but the buses' way is there in the first third of a
  second, in two long frames, before the GPU has what draws the bar, and
  the bar was first seen at 48%. Caught up, it creeps on while it waits,
  toward four fifths of what is still to come and never there
  (`CREEP_MOST`, `CREEP_SECS`): the buses' way is one piece of work, a
  second of it here, that says nothing of how far along it is, and the bar
  stood at 70% for that second before this. With everything there it runs
  to the end at 4 of the bar a second (`BAR_ENDS`), and the cover opens as
  it gets there: a quarter of a second later than it would have at the
  very most, and some three hundredths here, by the numbers.
- **What is seen here** (a dev build on this PC, three runs): black for a
  third of a second, the bar first seen at 8 to 20%, 70% at 0.8 s, 77 to
  80% at 1.4 s as the buses' way comes, the end in under a tenth of a
  second, and the circle opening at 1.45 to 1.6 s, as soon as it did.
- **The town's name comes up as the cover opens** now, in a quarter of a
  second (`TITLE_IN`), over the black round the circle, and stays and goes
  as it did. It stood on the cover while the town loaded; the picture has
  only the one word there.
- **The first time the app is run nothing says so.** The sign-up's pages
  are over the cover, the town loads under them and the bar fills under
  them, unseen. With the town all there as they go, which it is unless a
  name is typed in under a second, the cover opens 0.09 s after Save as it
  did, with nothing on it (`Intro::unseen`): "Loading... 96%" for a tenth
  of a second, between the last page and the circle, is what it did before
  that was put in. Should the cover not open within a third of a second
  after all (`UNSEEN_SECS`), or the town not be there yet, it says that it
  is loading as on any other start.
- **Calls that were mine.**
  - The background is the cover's plain black, as it was
    (`colours.intro.cover`). The picture's goes from `#111111` at its top
    to black at its bottom, a difference I could measure and hardly see.
    The circle the cover opens as is one colour, so a fade would have to
    be gone by the frame it begins; say so and it can be put in.
  - The red is as wide as the share written on it. The picture's is 18%
    of the bar under "25%".
  - The font is the game's, Inter, where the picture's is another, a
    narrower one: at the size that matches it best the word is 4% wider
    and 3% less tall than the picture's.
  - What each thing waited for counts for, the bar's two paces and the
    creep; the name off the cover; nothing on the cover after the
    sign-up's pages.
  - The picture is not in the app, which draws it from the numbers above:
    it is committed where Hajun put it, as the friends' panel's sketches
    are.
  - Bridge Wars' black screen, as a game loads, is as it was: "when u
    open the game".
- **Checked**, in the desktop build, hidden (`RT_BACKGROUND=1`), by a
  throwaway that had the game save a picture every 30 to 50 ms through
  its first seconds, taken out again: in a window of the picture's own
  shape, the frame at 25% laid over the picture at its size: the bar to
  the dot (rows 320 to 356, 35 to 817 across, the corners within a dot of
  the picture's), the word 209 by 46 dots where the picture's is 201 by
  47, the share 46 by 17 where it is 44 by 17, both in the middle; sixteen
  by nine too; by day and by night; the bar read off every picture of
  three runs, rising in every one; the first run from an empty `APPDATA`,
  through the sign-up's pages by keys and a tap, with the bar in no
  picture of it and the circle 0.09 s after Save; and `RT_INTRO=off`, with
  no cover and no bar. Unit-tested: the bar toward what has come at its
  pace, creeping for as long as the cover can be up, never back, the end
  within a quarter of a second, and the share as it is written (320 pass).
  **Type-checked for the iPhone on this PC.**
- **Not checked: anything on a phone or an iPhone.** How long a phone is
  covered and what the bar does through it (a phone's log says how long
  and for what, "the opening: covered for ... s"): if the buses' way takes
  several seconds there, the bar will be seen to slow toward 94% and wait;
  whether its first frames are black; the word and the bar beside a
  camera's hole or an island (by the numbers both are clear: the bar ends
  37 points above the bottom edge of a 16 Pro and lies below its island,
  and the word is in the middle); a change to the file's colours while
  the cover is up is not drawn until the next start.

## An iPhone's island: its safe area is read too

2026-11-15. Hajun, with the game on both phones: "also i noticed that for
android the OS safe area thing is working fine, but for iphone 16 pro, the
camera hole is not just a circle but a bar so it also covers the amethyst
icon and the chat text as well in the brdidge wars game. check for ios".
`src/safe.rs`, `src/hud.rs`, `src/quick.rs`, and for checking it
`mobile/ios/check_on_windows.sh`. The app's own: no change to `shared/`.
No APK and no iOS tag came of it (the last point says why).

- **Android is seen working on the phone**, by Hajun's words above: the
  first answer to "Not checked: anything on a phone" in the entry under
  this one ("Clear of the camera's hole").
- **An iPhone had never been asked** (that entry says so: "An iPhone is
  not asked"), so nothing moved on one, whatever shape its hole. What
  Hajun saw is what a window the size of a 16 Pro's screen shows with its
  island drawn on: 874 by 402 points lying on its side, the island a bar
  125 points long and 37 across, 14 in from the edge it is on (the numbers
  I have for a 16 Pro, not measured on one), so 138 to 264 points down and
  14 to 51 in. That lies over the amethyst's picture, the fourth of the
  counts, and over the first letters of every line of chat, and over
  nothing else on that side.
- **Now its window is asked** (`safe::read_the_window`, added on iOS
  alone; three times a second, as Android's activity says). winit 0.30
  gives a window an "outer" position and size and an "inner" one, and on
  iOS the outer are the whole window's and the inner its safe area's
  (`UIWindow.safeAreaInsets` taken off its bounds: read in winit's source,
  `platform_impl/ios/window.rs`). How far right of the one the other
  begins is the left inset, what is left of the width the right one, kept
  as shares of the window's width and brought to logical pixels
  (`safe::within`), which is what `safe::SafeArea` already was. Nothing in
  it is the iPhone's alone to compile: winit has all four on every
  platform, and only where the system is added is behind
  `target_os = "ios"`. What it read goes to the log to begin with and each
  time it changes (the iPhone's `log.txt`, "the safe area begins").
- **Both sides are moved on an iPhone.** iOS's safe area begins the same
  way in from both short edges whichever the island is on: 62 points on a
  16 Pro. So the name, the health, the four counts and the chat stand 68
  points in (the 62 and the 1.5 shares everything keeps), 17 clear of an
  island on their side, and the same with it on the other.
- **The right end of the bar is moved in no further than it has room**
  (`safe::RightRoom`, `hud::bar_room`). Moved the whole 52 points, the
  game button lay under "Bye!" and "Good game!", the top two plates of the
  things to say, when they are fanned out of the bubble. It is moved as
  far as leaves the plates a share clear, 4.5 shares on a phone, 18
  points: clear of a notch (33 points deep at the most, on the iPhones
  that have one), and an island lies lower than anything at that end (the
  minimap ends 127 points down, the island begins at 138). In Bridge Wars
  the minimap is at that end and there is no fan; it is moved the same, so
  that the bar is one thing.
- **The chat's small button steps in** (`safe::StepsIn`,
  `hud::CHAT_STEPS_IN`). It hangs 2 shares out of the right of the box
  round the jump button, half way down the screen, and with the phone held
  the other way up 7 points of it lay under the island. It steps in 2.6
  shares, 10 points: 4 clear of the island, and still more than 1.5 shares
  clear of every button about it (the nearest is the fourth for things to
  use, nearly straight over the sword). It is pressed where it is drawn.
  The dots, were they put back (`hud::DOTS`), hang as far out, higher up,
  and would want the same.
- **What does not move**, as before: the stick (61 points in on a 16 Pro,
  where the island ends at 51) and the other buttons round the jump
  button (52 in at the nearest, and those level with an island 56).
- **Calls that were mine**: moving both sides on an iPhone (iOS does not
  say which side the island is on, and asking how the phone is turned is
  UIKit called by hand, which cannot be run here); how far the right end
  of the bar and the chat's button go, and that both stop short of iOS's
  own safe area there, being clear of the island itself; that the stick
  and the jump button stay where a thumb knows them; putting the
  iPhone's type-check in the repo (`IOS.md`, "Before a tag").
- **Checked.** Unit tests: 314 pass, six of them new (what a window says
  as a safe area; the bar and the chat clear of an island on both sides;
  the bar's room on the right, and that with it the buttons down the right
  edge stay a share clear of every plate of the fan on screens of four
  shapes; the chat's button stepping in, and that stepped in it is clear
  of every button about it). The desktop build, scripted, in a window the
  size of a 16 Pro's screen in points, with the island and the screen's
  round corners drawn onto the pictures afterwards on both sides. Before:
  the amethyst and the chat's first letters under the island, as Hajun
  said, and the chat's button under one on the right. After, with the safe
  area an iPhone gives made up (62 and 62): the town (the bar, a line of
  chat, the fan out, the friends' panel, the game selector) and a game of
  Bridge Wars (the four counts, three lines of chat, the box typed into,
  all five buttons for things to use filled): nothing under an island on
  either side, nothing run into anything, and the chat's button pressed
  where it stands now. A safe area on one side alone (62 and 0, 0 and 62)
  moves that side alone, and none puts everything back. The reader itself
  was run on the desktop against its own window, by a scratch line: it
  read the window, said what it read, said so again when the window was
  resized, and not again after. **Type-checked for the iPhone on this
  PC**, the first change to be: no errors.
- **Not checked: anything on an iPhone.** That winit's inner position is
  there what its source says; the numbers iOS gives; that the island is
  where I have it. **If something is still under the island, read the
  phone's `log.txt` first**: nothing said, or noughts, is the reading;
  numbers that look right (some 62 and 62 on a 16 Pro) with something
  still covered is how far things are moved (`safe::CLEAR`,
  `hud::bar_room`, `hud::CHAT_STEPS_IN`).
- **No APK and no iOS tag.** As this was finished another session was
  changing Bridge Wars in the same tree and taking the protocol to 17 that
  night: an APK built here would have carried half of that, and an iOS
  build of protocol 16 would be told to update the moment the server
  moved. Android does nothing new with this unless a hole on the right is
  deeper than a phone's round one. The next APK and the next `ios-` tag
  carry it.

## The gift and the person are away; the settings are a window like the friends' panel

Later on the night of 2026-11-14, with the windows' new look ("Every window
is drawn as the friends' panel is", in `docs/ui-style.md`, which has what
the settings' window and the game selector look like now and why). Hajun,
as that was being looked at: "also there is too many lobby ui buttons now.
i think we can get rid of the gift icon for now, as well as the profile
icon next to the setting. u can shift the other buttons in place of
those." `src/hud.rs`. The app's own; no APK built, uncommitted as this is
written.

- **Two words turn them off** (`hud::GIFT`, `hud::PERSON`, both `false`):
  the gift is not spawned at the top of the right edge, nor the person
  beside your name. Each stood first in a row that lays itself out, so
  the others are in their places with nothing else changed: **the two
  people (friends) are at the top of the right edge and the gamepad under
  them; the gear is right beside your name.** Everything of both buttons
  is still in the code, and each word puts its button back.
- **The gift was the car's gun** on a phone (held in the car;
  `hud::GunButton`, `drive::shoot`). While it is away the gun is G on
  desktop and nothing on a phone. It hits nothing yet, and Hajun said
  "for now". The person did nothing.
- The test that keeps the chat's small button clear of the gamepad goes
  by where the gamepad is now, one place higher (and two with the gift
  back).
- **Checked** in the desktop build, scripted: the town's buttons on
  windows shaped sixteen by nine, as a phone and as a tablet; the gear,
  the two people and the gamepad each opening what it opens from where it
  is now, the friends' count in gold on the two people's corner. **Not on
  a phone.**

## Clear of the camera's hole; two people for friends; the chat's own button

Since the night of 2026-11-14. Hajun, having played on the phone: "Right
now I checked, and in my mobile Samsung phone, the chat box is covered
slightly, uh, and the input field too, by the, uh, small camera hole. Uh,
so I guess I need the UI elements to use the OS safe area, so that the
left margin is just for the UI elements is shifted to the safe area so
it's not covered. So right now in the UI, right side, there's 2 chat icons.
I like the top chat icon design better, so use that chat icon for the
bottom chat icon. And instead of the top chat icon, which was for friends,
uh, use a, uh, person, 2-person icon for the friend UI button." The rest of
what they said that night is the chat's ("The chat is its words alone",
in `docs/chat-and-text.md`). `src/safe.rs`, `src/hud.rs`,
`MainActivity.java`. APK 31; no change to `shared/`.

- **The camera's hole** (`src/safe.rs`). The game is drawn under it
  (`MainActivity.applyImmersive`: the window lies in the cutout along the
  short edges), and held on its side the phone has it half way down the
  left edge, where the chat is, or the right one, turned the other way up
  (`sensorLandscape`). `MainActivity.tellSafeInsets` reads the cutout's
  safe insets, left and right, off the decor view's root insets, as the
  window takes the focus and every 300 ms with the clipboard's poll, and
  hands them to the game by a native method (`safeInsets(int, int)`, the
  sixth; numbers only, as the keyboard's height is). The game brings them
  to logical pixels (`safe::SafeArea`).
- **What moves** (`safe::KeepsClear`): what stands 4 shares of the short
  side in from the sides (`hud::EDGE`): the bar across the top, in the
  town and in Bridge Wars (your name and what is under it on the left;
  the three buttons, or the minimap, on the right), and the chat. Each is
  given a margin that brings it 1.5 shares inside the safe area's edge on
  the side the hole is on, and none on a screen with no hole. **My
  reading**: "shifted to the safe area so it's not covered" is moved just
  clear of the hole, not the whole screen's worth of margins added to the
  hole's depth. **What does not move**: the stick and the buttons round
  the jump button (7 and 6 parts in a hundred of the width in, where a
  hole ends 3 or 4 in, by my reckoning from a status bar's height, not
  measured), and whatever comes up in the middle of the screen.
- **`RT_SAFE_AREA=<left>,<right>`**, in logical pixels, makes a hole up on
  desktop.
- **An iPhone is not asked**, and its island still lies over what is
  beside it: winit 0.30 has the safe area, but nothing behind
  `target_os = "ios"` can be checked on this PC.
- **The friends' button is two people** (`icons/friends.png`, Material's
  `group`, the friends' panel's own tab picture), and **the big bubble
  over the jump button is the one with three dots** that the friends'
  button had (`icons/social.png`, Font Awesome's comment-dots). Material's
  bubble (`icons/chat.png`) is on nothing now, and still in the APK.
- **The chat's own small button**, a keyboard, over the bubble and a
  little to the right, in the town and in Bridge Wars alike, and the
  bubble's fan of things to say: in `docs/chat-and-text.md`.
- **Checked.** A unit test of how far in a thing is moved for a hole
  (none with no hole, just clear of one, never back out). The desktop
  build, scripted, with a hole made up 70 pixels deep on the left and then
  on the right, in the town and in a game of Bridge Wars: the name, the
  balance (or the health and the four counts) and the chat moved in on the
  left; the three buttons, or the minimap, on the right; the stick and the
  buttons round the jump button where they were, and nothing run into
  anything. The new pictures, enlarged.
- **Not checked: anything on a phone.** The Java that reads the cutout has
  never run: that Android hands the hole over the way it is read, that it
  is told again when the phone is turned the other way up, and how deep
  the S26's hole is, are all still to be seen on it. **If the chat is
  still under the hole on the phone, read the phone's log first**: the
  game says what it was handed to begin with and each time it changes
  (`adb logcat`, a line with "the safe area begins", in logical pixels and
  in the screen's own), and the activity says once if it could not hand
  it over at all (`adb logcat -s Donggeurami`). Nothing handed over, or
  nothing but noughts, is the Java's side; numbers that look right and a
  chat still covered is `safe::CLEAR`, or the hole being deeper than its
  safe inset says.

## The app opens under a cover, and you drop into the town

Since 2026-11-13 (Hajun: "right now, when i open the game, there is no
smooth introduction animation. i need a short 0.5~1s animation in the
beginning. also when i open the game, some part of the map is not loaded
yet so it looks bad. only show the map after it gets loaded. im thinking
maybe it can be falling from an extreme height, and when it lands, maybe a
short sit on but fall landing animation, and then get up/standup and then u
can play"; of the four ways I put to them, a drop onto your bottom, a drop
into a squat, a circle opening and a dive from above the town, they chose
the circle with the squat: "ok C+B try it"), in `src/intro.rs`.

- **The cover.** From the first frame the screen is black with the town's
  name on it in white (`colours.intro.cover` and `title`, and
  `words.welcome`, in `assets/ui.ron`), over everything on the screen but
  the sign-up's pages (its layers are 25 to 27, theirs 30), and the town
  loads under it. It takes every press (`TakesPress`) and holds
  `hud::DialogUp`, so you stand still under it as under a dialog. While it
  is up the camera clears each frame to the cover's colour too: the first
  frames may be drawn before the GPU has what draws the cover itself
  (its pipeline is compiled in the background like any other), and would
  then be the bare sky; reasoned from Bevy's source, not seen. Android's
  own window is black before that
  (`windowBackground` in `themes.xml`, since APK 24).
- **It opens once the town is there to be seen** (`Loading::there`, `Wait`):
  the town's shape read (`Islands`), every model in the world spawned (every
  `WorldAssetRoot` with its instance ready: the islands, the buses,
  everyone's bodies), the buses' way round worked out (`car::Planned`), and
  your own limbs bound; and then only once the GPU has had nothing left to
  compile for eight frames running (`Compiling`: a system in the render
  world, `count_compiling`, counts `PipelineCache::waiting_pipelines` after
  each frame; Bevy compiles each material's pipeline in the background and
  leaves out whatever is not ready, which is one way a town comes up a
  piece at a time), and not on the heels of a frame that took over 50 ms.
  With anything never coming it opens anyway after 10 s (`MOST_COVERED`),
  and the log says what was missing. It logs one line as it opens, with
  when each thing was there: on this PC, a dev build, covered for 1.5 s,
  the town and its models at 0.3 s and the buses' way at 1.4 (it takes a
  second to work out here). The first time the app is run the sign-up's
  pages stand over it, the wait is theirs, and it opens as they go, 0.13 s
  after Save here.
- **The opening is 0.85 s** (`UP_AT`; `pose`, `hole`): the cover opens as a
  circle about where you stand, 동그라미, reaching every corner in 0.55 s
  (`OPEN_SECS`; one node, all border, with a round hole, as Bridge Wars'
  blindfold is, `gear::blindfold`); you drop in from 11 m up, over the top
  of the screen, arms up in a V and knees a little bent, faster as you
  come, and land at 0.4 s (`FALL_SECS`, `FALL_FROM`); you sink into a deep
  squat in 0.07 s, arms out at your sides, leaning in, drawn 14% shorter
  and 8% wider, stay down 0.06 s, and spring up over 0.32 s, through
  a little taller than you are; a white ring goes out over the ground from
  your feet as you land, to 2 m in 0.45 s, fading (`Dust`, drawn as the
  punch's range is, `range::look`, and as brightly as the sky lights the
  ground, `Daylight::unlit`, so that it does not shine at night; it waits
  at your feet under the cover, too small and faint to see, so that the
  GPU has it ready); and the camera dips 0.12 m as you land. From 0.85 s
  you can move, and the buttons are yours.
- **Nothing of the fall is your body's.** Your body stands where it
  arrived all along; what drops is its model, the node your limbs hang
  under, hung that much higher over it and put back as it was at the end
  (`Model`, `hang`). So it is no move to the island, to anyone else, or to
  the server, and nobody online sees it. The squat is your limbs'
  (`Landing` in `src/lib.rs`, a component as `Carrying` and `Drinking`
  are, `landing_offset`, read by `animate_walk` before anything else): a
  thigh turned 1.05 rad forward at the deepest and its shin bent back by
  twice that, an arm turned up to 2.2 rad from hanging; and the model
  comes down by what its bent legs take off its height, less what its
  heels come up (`Landing::sink`, from how high its hips are, measured off
  its thigh bones as the cover opens), so its feet stay on the ground.
- **The town's name stays where it was on the cover**, over the town now,
  for 2.2 s and fades in 0.8 (`TITLE_STAYS`, `TITLE_OUT`): it is the
  welcome, in white. `hud::welcome` leaves that one to it
  (`Intro::welcomes`); back from a game, the gold welcome fades in as
  before.
- **For trying it on desktop**: `RT_INTRO_SLOW=<times>` plays the opening
  that many times slower; `RT_INTRO=off` leaves all of it out, cover and
  held input and all, with the gold welcome as before, for a scripted run
  that presses or looks from its first frame.
- **What is mine**: every number; the white name, staying on in place of
  the gold welcome; the ring of dust and the dip; that the buttons come
  into sight with the circle rather than after you stand; and **that the
  cover waits for the buses**, where I had told Hajun I would let them
  drive in: it is the last 1.1 s of the 1.5 s the cover is up here, and a
  bus coming onto the ring road a second after the town is first seen is
  the kind of thing that was asked to go. Taking `car::Planned` out of
  `Loading::there` is all it takes to let them drive in instead.
- **Checked on 2026-11-13, scripted, in the desktop build** (a throwaway
  `src/intro/scratch.rs`, removed, that had the game save its own
  screenshots through the opening): at a fifth of its speed, from the
  game's camera and from the side close up, by day and by night: the
  cover from the first frame, the circle from nothing to past the
  corners, the drop, the squat with the feet on the ground and the knees
  forward, the ring, the name fading; at its own speed, three runs, every
  frame of the opening 16.2 to 17.6 ms on a 60 Hz screen, and the stick
  let go at 0.85 s; and the first run, with an empty `APPDATA`, through
  the sign-up's pages by keys and a tap, the cover opening as they went.
  Unit-tested: the poses on the models (`limbs_of`), the fall, the
  circle, the dust, and the wait. **Not by hand, and not on a phone**: how
  long a phone is covered, whether its first frames are black rather than
  sky, and the Adreno drawing the ring are all unseen. APK 24, built that
  night by the session that made Bridge Wars' five item slots, is the
  first with it, and was not installed, no phone being attached; a phone's
  log says how long it was covered and for what ("the opening: covered
  for ... s").

## The app opens in Round Town

(Since 2026-11-13 the app opens under a cover, and you drop into the town
as it lifts: the section above. What this one says of the first frame, and
of the town's name fading in as the app opens, is how it was until then.)

There is no menu. Once you have signed up (above), the app opens straight
into the town (`roundtownmap.glb`, from 2026-10-30, `circlemap1.glb`
before; for the day of 2026-11-07 the PvP map, `pvptownmap.glb`, which is
Bridge Wars' since that night, above). Everyone stands on the path round
its pond, facing the fountain in the middle of it, and the town's name,
"Donggeurami Town", fades in and out over the first few seconds. You can
walk, jump and look round from the first frame; on desktop the cursor is
taken for looking round at startup. The games, House Builder and Bridge
Wars, are chosen from the gamepad button (above, "Games").

- **The buttons round the screen are laid out as Hajun sketched them**
  (`assets/UIbuttonlayout.jpg`, 2026-10-31, kept out of the APK;
  `hud::show_top_bar`): top left your name on a plate (the buttons' grey, a
  little more solid, corners rounded a little), beside it a person button
  and the settings' gear ("Settings", below), and under it your balance,
  the coin and the number on nothing; down the right edge a gift, a
  speech bubble (social chat) and a gamepad (a leaderboard, a city at
  first, until Hajun said which it was, stood between the gift and a group
  of people until 2026-11-01, when Hajun had the leaderboard taken out
  and the group made a bubble); and round a phone's jump button a face
  (emotes, one day), a speech bubble (chat) and a fist. Every colour of it
  is `assets/ui.ron`'s since 2026-11-04 (above, "The screen's colours and
  words are in `assets/ui.ron`"). The person and the
  gear are 8% of the short side (`hud::SMALL_BUTTON`), the three down the
  right edge 10.5 (`SIDE_BUTTON`; 9.5 for an hour on 2026-11-01, then "very
  slightly" bigger) and 1.5 further in from the top and the right than the
  name is (`SIDE_INSET`, "very slightly increase margin"), the three round
  the jump button 14
  (`ACTION_BUTTON`), the stick 25 (`STICK_SIZE_VMIN`; 22 until Hajun
  asked for "slightly" bigger, 2026-11-01), 7% of the width in from the
  left (`STICK_LEFT_VW`; 8 until "very very slightly more left", the same
  day), and the jump button itself 22,
  6% of the width and 9% of the height in from the corner
  (`JUMP_SIZE_VMIN`, `JUMP_RIGHT_VW`, `JUMP_BOTTOM_VH` in `src/lib.rs`;
  since the evening of 2026-10-31, Hajun: "keep the size of the user icon
  and the settings icon but increase the size of every button the right
  hand side both top and bottom, especially jump button, and move the
  bottom right buttons slightly closer to the bottom right corner"; 8, 8,
  12 and 18 at 8% and 11% until then, then 5% and 7% for a night, until
  Hajun asked for "very slightly more margin", and 8% of the height for an
  hour, until "move them up very very slightly"). The three are placed from the
  jump button's corner so as to sit about 2.5% clear of it, the face up
  and to the left, the bubble over it and the fist beside it, as Hajun's
  sketch has them. **Only the gear, the stick, the jump button, the fist,
  the face, the gamepad and the speech bubble under the gift do anything**
  (Hajun: "u dont need to make any button
  functional yet other than jump, joystick, settings"; the fist since
  2026-11-01, below, "The fist throws a punch"; the face since 2026-11-05,
  the car's switch, below, "The car you drive", where the fist is the
  reverse and the jump button the boost; the gift since 2026-11-06,
  which held in the car is its gun, there too; the gamepad since the
  night of 2026-11-07, the game selector, above, "Games"; and the speech
  bubble under the gift since 2026-11-13, your friends, with how many
  things wait in your inbox counted in gold on its corner, below,
  "Friends, groups and messages"); the rest take a press and drop it. The three round the jump button are placed from where
  the jump button is, on every platform (until 2026-11-06 the desktop
  build showed the three with no stick or jump button among them; since
  then it has both, below, "The stick and the jump button are on Windows
  too"). Until then a
  button top right took you home (`testhomemap.glb`) and back (Hajun:
  "remove the top right home button"); nothing takes you home now, though
  it is still loaded at startup (`island::LOADED`) and the server still
  answers `Ask::Travel`. The town's name shows again every time you arrive
  back in the town, from a game of House Builder.
- **Every button is in one style, the stick's and the jump button's**
  (Hajun, 2026-10-31): a see-through grey disc with no rim, and on it a
  see-through white picture (`hud::round_button`, `TOUCH_DISC`,
  `TOUCH_MARK`: white at 0.52; "a little greyed", 0.92 at 0.38, until Hajun
  asked for "slightly more whiter"; held, the jump button is a little more
  solid). The jump button's arrow, the pencil and the fist are drawn in
  code (`hud::jump_arrow`, `pencil`, `fist`), each one filled shape edged
  by how far every pixel is from it, as the rating star is: nodes
  overlapping would show where they do through the see-through white. The
  fist is a closed hand from the front, four fingers over the palm with
  the thumb across them (Hajun, 2026-11-01: "more a hand fist rather than
  boxing glove"; Material has no closed hand); the gear was drawn in code
  too until that day, when Hajun's screenshot of it was Material's
  `settings`. The other pictures are 96 px PNGs in `assets/icons/`, white
  on clear (`hud::ICONS`): Google's Material icons (Apache 2.0, its
  licence beside them and out of the APK), turned white by hand (Hajun:
  "search online for that icon + png/transparent, download and modify
  it"): person, settings, videogame_asset, mood and textsms; and Font
  Awesome Free's gift (solid) and comment-dots (regular), the gift and the
  social chat's bubble top right (CC BY 4.0, `LICENSE-font-awesome.txt`
  beside them, out of the APK), the icons in Hajun's FlutterFlow
  screenshots of 2026-11-01, rendered white from their SVGs with an 8 px
  margin by a throwaway `resvg` tool (the gift and the bubble were drawn
  in code for an hour first). card_giftcard, leaderboard, groups and
  sports_mma are still in the folder, unused. Until 2026-10-31 the stick
  was white with a white rim, and the jump button yellow, its arrow three
  strokes. The stick and the jump button are put away while you talk to
  someone (below), and since 2026-11-01 the three round the jump button
  with them (`hud::hide_round_the_jump`; Hajun: "the punch, quickchat, and
  emote button is not hidden. hide those too"). Seen in the desktop build; none of it has been seen on a phone yet.
- **The stick's middle is set where the finger first goes down, and no
  further than half its radius from its own place** (`stick_origin`,
  `STICK_ORIGIN_REACH` in `src/lib.rs`, since 2026-11-14; Hajun: "if I tap
  the very bottom of the circle, the joystick's origin gets set too low, so
  it's hard to move downward at full speed. Limit the area where the origin
  can be set to 50% of the current radius. If the first tap lands outside
  that radius but still inside the joystick hitbox, set the origin on the
  edge of the 50% radius at the same angle. Movement magnitude should then
  be measured from that origin"). A finger put down at the very bottom of
  the disc has the middle half a radius down, and is pushing half way down
  from the start. From the small hours of that day the middle was wherever
  the finger went down (`versionCode` 28, "Building"). The mouse's on
  desktop the same. Unit-tested
  (`the_stick_s_middle_is_set_no_further_than_half_its_radius_from_its_own`);
  not seen under a thumb, and a script cannot hold the mouse on the stick.
- **The stick and the jump button are on Windows too** (since 2026-11-06,
  Hajun: "put on screen joystick and jump button on windows version too";
  until then they compiled for phones alone, `setup_hud` and the stick's
  and the jump button's visuals gated to them): drawn as on a phone, where
  a phone has them, and driven by the mouse with the cursor free, Escape
  (`read_mouse_controls` in `src/lib.rs`, before
  `editor::read_desktop_pointer`): pressed on the stick's disc, the mouse
  holds the stick, pushed toward wherever the cursor goes until the button
  comes up, as a finger is; pressed on the jump button it jumps, or boosts
  the car while held. The mouse is one more finger to `TouchControls`
  (`MOUSE_ID`), so `read_player_input` and `drive::steer` need no desktop
  path, and the keys and Space work as before. A press on either is not a
  tap on the town (`TouchControls::has_the_mouse`, which the editor's
  desktop pointer asks), though it dismisses an open menu, as on a phone;
  and one on a button over the world is the button's. Walking with the
  mouse on the stick, the camera comes round behind you after 1.5 s, as it
  does on a phone (`assist_camera`): the mouse is off the camera while the
  cursor is free. With the cursor taken for looking round there is no
  cursor to press with. Asked for to drive the car with a stick on the PC,
  the keys giving only eight ways. Seen in the desktop build the same day.
- **The fist throws a punch** (`Punching`, `start_punch`, `punch_offset` in
  `src/lib.rs`, since 2026-11-01; Hajun: "a very basic animation for the
  fist attack that activates each time u press the punch button. dont make
  the animation too long so that user can press the attack button like 4~8
  time per second"): each press of the fist, as it goes down
  (`hud::press_buttons`), or F on desktop, out of the car (in it the fist
  is the reverse, below, "The car you drive"), swings the right arm straight out
  level in front of you and back, 0.07 s out and 0.11 s back (`PUNCH_OUT`,
  `PUNCH_BACK`, `PUNCH_REACH` 1.7 rad, level in front of
  `playercharacter.glb`, whose arms hang a little back of the shoulder; 1.5
  until 2026-11-01; the two arms in turn for an hour,
  until Hajun asked for "only use right hand"); a press mid-punch starts
  the next at once, from the side. The arm punches in place of its walk;
  the rest of the body walks on. Nothing of it goes over the network, so
  nobody online sees it; it hits nothing. Seen from the side in the
  desktop build, the arm level in front (it first swung backward: forward
  is the same turn in both arms' bone spaces, the walk only negates the
  right's to swing it against the left); not yet on a phone.
- **On desktop, press `Escape` to free the cursor before clicking a button**,
  the same as for editing. On a phone you just tap it.
- **A press on a button is the button's alone.** Every button over the world
  is a `bevy_ui` node pressed through `bevy_picking`'s click, which a mouse
  and a finger both send, and carries `hud::TakesPress`. The touch controls
  and the editor's desktop pointer ask `hud::Presses::claimed` first, which
  tests the node's laid-out shape the way picking does, so the same press is
  not also taken for a look round or a tap on the town. A new button needs
  `TakesPress` and nothing else: there are no sizes to keep in step.
- **Every player has a balance** (`lobby::Balance`), shown top left next to
  `assets/coin.png`. Offline it starts at 1,000 (`lobby::STARTING_BALANCE`)
  and is not saved, so every launch starts at 1,000 again. Online it is the
  server's: kept with your account in Postgres, which a new account starts
  with 1,000 in, and sent to the device (`Tell::Balance`). The only things
  that change it are House Builder's entry fee and its prizes (below).
  `assets/coin.glb` is still in the repo, but nothing loads it.
- The text is Inter since 2026-11-13 ("The font is Inter", below); until
  then Bevy's built-in font, which is plain ASCII: no Hangul, and no ★,
  `…` or `·` either. Inter has no Hangul either. The rating star is drawn
  in code (`builder::star_image`).

## Settings

A gear beside your name, top left (`src/settings.rs`, since 2026-10-31;
Hajun: "a setting icon ... make it show a semi transparent greyish
rectangular popup which will have all sorts of different useful
settings"; top right for an hour, until their sketch of the screen put it
by the name), Material's `settings` icon (`assets/icons/settings.png`,
since 2026-11-01, Hajun's screenshot; drawn in code as the jump button's
arrow is until then), in the buttons' own style (`hud::round_button`),
after the person button
in `hud::BarLeft`. It opens a window of
grey glass over the dimmed screen, a dialog like the shop (`DialogUp`: you
stand still, every press is its, Escape or the cross on its corner or the
dark round it shuts it), with three tabs along its top: **Graphics** and
**Controls**, with nothing in them yet, and **Accounts**: the name you go
by, with a pencil beside it (`hud::pencil`) that opens a smaller dialog,
"Edit name", with the name in the sign-up's typing box (`signup::NameBox`,
`signup::name_box`) and Cancel and Save (Hajun: "a pencil/edit icon that if
u press, you can edit your name in a smaller new popup. and save so thats
ur new name"), which on a phone slides up out from under the keyboard as
the sign-up's name page does (`signup::LiftsForKeyboard`, above, "The first
time, you sign up"): Save makes it your name everywhere the sign-up did
(`signup::wear`: the town's room, over your head, and the server, through
the profile changing), and goes back to the tab; "Sign in with Google" and
"Sign in with Apple" blocked out until there is a sign-in to be had; and
**Delete account**, which asks once more, then has the login forget the
account; this device forgets its secret and its profile, and the sign-up's
pages come back. Offline, Delete says "Not online: try again". Seen in the
desktop build on 2026-10-31, scripted; not yet on a phone. Since 2026-11-13
an account deleted takes its friends, what it asked and was asked, and
what was written to and by it with it ("Friends, groups and messages",
below).
