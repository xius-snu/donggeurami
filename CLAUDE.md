# 동그라미타운 (RoundTown)

A third-person 3D world in Bevy 0.19, zoomed into first person all the way
in. One codebase, three targets: Windows
(`src/main.rs`), Android (`mobile/android`, built as a cdylib via cargo-ndk),
iOS (`mobile/ios`, built as a plain Rust binary that Xcode bundles).

## Before touching rendering, read `RENDERING.md`

It documents why the APK used to look different from the Windows build, what was
tested on the actual phone, and what not to try again. The short version:

- **Windows and Android run the same rendering code.** `src/sky.rs` has no
  `#[cfg(target_os = ...)]` in it. Keep it that way — materials, lights and
  geometry are shared, with real `AlphaMode::Blend` and `AlphaMode::Add` on both.
- **`NoIndirectDrawing` on the camera (`setup_world` in `src/lib.rs`) is load-bearing.**
  Without it the Adreno 840 silently drops Bevy's sorted transparent phase and
  the water, clouds and sun glow all disappear — with no error in logcat. Do
  not remove it as a cleanup.
- **A day and a night pass every twenty minutes** (`src/sky.rs`, since
  2026-10-16, Hajun: "re-add daynight cycle"; from 2026-09-23 the sun stood
  still at mid-morning): ten minutes of day, a minute and a half of sunset,
  seven of night and a minute and a half of sunrise. It goes by the town's
  clock (`src/clock.rs`, below, "Playing online"), not the game's time, as
  the fountain's jet and the town's buses do, so every device online shows
  the same sky. Noon is exactly the light Hajun chose on 2026-10-13
  (a test keeps it so); the sun climbs to it quickly and stays near it most
  of the day, then goes down orange into a pink and a deep blue sky, with a
  moon and stars through the night. The sun and the moon are one directional
  light, so the night has no more lights to draw than the day. Only what
  has changed is written, a quarter of a second at a time. `RT_TIME_OF_DAY`
  (seconds into the cycle, or `day`, `sunset`, `night`, `sunrise`) stops it
  at one time, for trying things out. Shadows are still off (`SUN_SHADOWS`).
  **The sky is drawn round the camera** (since 2026-10-30): the sun, the
  moon, the stars and the clouds hang from one `SkyDome`, kept where the
  camera is and three times as big as they are built (`SKY_SCALE`): the sun
  510 m out, the clouds 276 to 414. Drawn as built round the middle of the
  town, they were in the new town, 420 m across: the sun went down among its
  houses, its glow lighting up the road under it (Hajun: "the sun appeared
  through the road"), and the lowest clouds hung over its roofs. The sea is
  4,000 m across (`SEA_SIZE`; 1,200 until then, whose corners bent the
  horizon seen from the edge of the new town).
- **Don't fake transparency with opaque geometry.** An earlier attempt added an
  opaque haze dome and giant "sun wash" spheres; that is what made the sky look
  grey and washed out. `RENDERING.md` has the details and the anti-patterns.
- **Test on the device, don't infer from the engine source.** The adb
  build-install-screenshot loop is in `RENDERING.md` and takes about ten seconds
  per cycle.
- **Lights are sorted into clusters on the CPU, on every platform**
  (`cluster_lights_on_the_cpu` in `src/lib.rs`, since 2026-10-08). Bevy 0.19
  does it on the GPU everywhere but Android and the iOS simulator. The only
  lights it sorts are the street lamps', and only at night (below): by day
  there is nothing to sort, which on the CPU is nothing at all, and every
  platform takes Android's path. It was turned off as the suspect for the
  iPhone's black screen, which it was not (`RENDERING.md`). The camera has
  Bevy's own cluster grid since the lamps came; before them it had one cell
  (`ClusterConfig::Single`).
- **Street lamps light the night** (`src/lamps.rs`, since 2026-10-16): every
  bulb in a model, a part whose material is `lightbulb`, glows from dusk to
  dawn, and each lamp, its bulbs taken together, throws a point light from
  between them (5,000,000 lm, out to 40 m: right under it the ground is lit
  a little more brightly than by the noon sky, and 16 m out as brightly as
  by the moon; 2,000,000 and 16 m until 2026-10-30, when Hajun asked for each
  lamp to light a larger area, "maybe 2~3x": the lit ground round each lamp
  is about three times what it was, and the ring round the pond is lit all
  the way round). The shop's Street Lamp,
  and any in a map: Hajun's new town has 20 since its third version
  (2026-10-30), each an object of its own (12 in one object, `streetlam`,
  in its second version, which is no matter, a lamp being its bulbs
  wherever they are, `lamps_of`); the old town's 23 went with it. By day there are no
  lights at all; at night at most
  24 at once, the nearest the camera on your island. No shadows. On the
  laptop the night cost about a third more GPU than the day (8% to 11%)
  and no more CPU with the lamps reaching 16 m (measured 2026-10-16); at
  40 m each spot is lit by two or three, which costs more, not yet
  measured; not yet seen or measured on a phone, where Bevy's CPU
  clustering has point lights in it for the first time. The night's own light, the moon's, is a twentieth of the noon
  sky's on the ground, enough to find your way on an island with no lamps.
- **Bridge Wars' respawn anchors glow and light their islands by night**
  (`src/lamps.rs`, since 2026-11-11; Hajun, whose map of that day took the
  lamps off the islands: "for now, just make the spawn anchor purple
  crystal thing emit some light so that its not completely dark, while
  still remaining purple"). What gives off light is a list by material
  now (`lamps::SOURCES`): a street lamp's `lightbulb`, as before, and the
  anchors' `spawn`. From dusk to dawn, as the lamps (`Daylight::lamps`),
  each pillar glows in its own purple (`ANCHOR_GLOW`, one and a half times
  its colour: drawn (182, 101, 209), the purple its sunlit faces are by
  day, (200, 117, 213)) and throws a pale purple point light
  (`ANCHOR_COLOR`, 0.78, 0.62, 1.0) of 1,000,000 lm out to 25 m
  (`ANCHOR_LUMENS`, `ANCHOR_RANGE`: a fifth of a street lamp's) from
  inside its tip, 3.5 m up (`ANCHOR_LIFT`): a pool of light round the
  anchor that reaches the island's middle, 6 m away, about twice as
  brightly as the moon, and leaves the far half to the moon. My numbers;
  kept pale and modest so that the island keeps its colours. By day
  nothing glows and there is no light, as with the lamps. A broken
  anchor's pillar is hidden, and its light goes out with it; it comes
  back as the pillar is stood up again. The map has 24 lights with them,
  20 lamps and the four, which is `MOST_LIT`. **Two things found on the
  way**: a light over the pillar's tip lit its faces white, so the light
  is inside it, where no face of it is lit and the glow alone colours it;
  and **Bevy 0.19 does not take the camera's exposure to emitted light**
  unless the material's `emissive_exposure_weight` says so (0 by default;
  the alpha of `emissive` is not read), so a glow is so many times the
  colour, as it is: at 1,500 the pillar was white. The bulbs' 4,000,
  which a comment had as "four times its colour", is far past white and
  left as Hajun has seen it since 2026-10-16. Seen in the desktop build,
  scripted, with `RT_TIME_OF_DAY=night` and `day`, from 10, 24 and 60 m
  over the island, and with the anchor broken; a test keeps one light to
  each anchor, inside it (`each_of_bridge_wars_anchors_is_one_light_inside_its_tip_and_glows_purple`).
  Not on a phone; the APK was not rebuilt. **Since 2026-11-13 each anchor
  is its team's colour, red, blue, green or yellow, and no longer
  purple** (Hajun: "make it so the color is your team's color. so red blue
  green yellow glow"; `arena::paint_anchors`, `colours.arena.anchor_*` in
  `assets/ui.ron`; "Games", below): each team's pillar has a material of
  its own in that colour, glows in it by night as above, and throws a
  pale shade of it where all four threw the pale purple. Nothing in
  `lamps` changed for it: a bulb glows in whatever colour its material
  is, and `ANCHOR_COLOR` is only what a pillar's light is for the frame
  or so before it is painted.
- **The town's fountain has a material and a shader of its own**
  (`src/fountain.rs`, `src/fountain.wgsl`): see-through running water whose
  streaks the GPU moves by the frame's time. It is the one custom shader;
  Bevy compiles it on each platform like its own, built into the binary
  rather than loaded from `assets/`.
- **Standing in a house in House Builder, or in one of the town's houses,
  what of it stands between the camera and you is drawn see-through**
  (`src/see_through.rs`, since 2026-10-09; everywhere until 2026-10-25, only
  in House Builder's houses until 2026-10-30): a copy of its own
  material at a quarter, drawn with alpha to coverage since 2026-10-13 (blended until
  then), so in the opaque phase, writing depth, under the camera's 4x MSAA.
  Not yet seen on a phone, neither way; the Adreno has never drawn alpha to
  coverage for this game. If it looks wrong there, `see_through::see_through`
  is where to go back to `AlphaMode::Blend`, which needs `NoIndirectDrawing`
  as the sea and the clouds do.
- **Unlit things are drawn brighter than their colours** (since 2026-10-13).
  Bevy's tonemapping (TonyMcMapface, its default) runs on unlit materials
  too, and shows white as a light grey: the clouds were grey and the sun a
  dull mustard. The clouds and the sun's disc are drawn at five times their
  colour (`sky::UNLIT_GAIN`), the fountain's running water at two and a half
  (`fountain::WATER_GAIN`). `ClearColor`, the sky, is not tonemapped. The
  same day the light was rebalanced, the blue fill turned down: Hajun found
  the whole world a greyish blue, "like a privacy screen filter".
  `RENDERING.md`, "Colour", has what was tried and what Hajun turned down.
  Since the night came back, the clouds and the sun's disc take the colours
  of the time of day, and the fountain's running water is drawn as brightly
  as the sky lights the ground, never less than 0.3 of noon
  (`sky::Daylight::unlit`): unlit, it would shine through the night.

## It aims for 120 frames a second on Windows and Android

Bevy draws a frame every time the screen refreshes, and every frame is the
whole scene. On the 240 Hz laptop this is built on, that was 240 frames a
second, 168% of a CPU core and 45 W on the GPU, and the machine ran hot
(measured 2026-09-27). 120 is the target on both platforms instead.

- **Desktop: `src/pace.rs`** draws a frame every so many refreshes — every 2nd
  at 240 Hz — so each frame is shown for as long as every other. It uses
  Bevy's `UpdateMode::Reactive` with every wake-up but the clock turned off:
  input is read on the next frame and never starts one of its own. A screen
  at 144 Hz or slower is left alone.
- **Android: `MainActivity`** asks for the screen mode closest to 120 Hz and
  calls `Surface.setFrameRate(120)` on the game's surface, so the phone paces
  the frames itself. Without an explicit request Android decides the rate:
  `adb shell dumpsys display | grep frameRateOverride` shows what it chose.
- **iOS** holds apps to 60, because `Info.plist` does not set
  `CADisableMinimumFrameDurationOnPhone`. Setting it is how iOS would get 120.
- **On the S26 the GPU is what limits 120.** It holds 120 while the phone is
  cool; once it is hot (sooner while charging) Android caps the GPU clock at
  about 750 MHz and frames start arriving late. That is why the sun has two
  shadow maps rather than four (`src/sky.rs`): four cost about 40% of each
  frame's GPU work there. `RENDERING.md` has the measurements and how to take
  them.
- **On trial since 2026-10-03: no shadows, and the phone draws at three
  quarters size.** Hajun's phone ran hot, and they asked to see what taking
  the shadows out and a slightly lower resolution would save. `SUN_SHADOWS`
  in `src/sky.rs` is off (on Windows too, as the rendering code is one), so
  Bevy makes no shadow maps at all. `RENDER_SCALE`, 0.75, in `MainActivity`
  and in `src/lib.rs` (keep the two the same) has the phone draw 1080x2340
  and its display hardware stretch it over the 1440x3120 screen
  (`SurfaceHolder.setFixedSize`), with touches scaled down in
  `processMotionEvent` to match and the UI's scale factor lowered to keep
  every button its size. Neither is measured or tried on the phone yet.
- **There are no real mirrors** (Hajun's call, 2026-10-02). A reflecting
  Mirror was built and worked: a second camera behind the glass, on only
  while the glass was in sight. But every active camera gets its own pair of
  the sun's shadow maps in Bevy 0.19, with no way to turn them off for one
  camera, so a mirror in sight would cost about what going from four maps to
  two saved (estimated, not measured). Hajun had it taken out as too costly.
  The shop's Mirror went back to a shiny pale disc, and since 2026-10-03 the
  shop no longer sells it (it sells only Hajun's pieces). `RENDERING.md` has
  how it was built, should it come back.
- **Every schedule runs its systems on one thread** (`one_thread_per_world` in
  `src/lib.rs`). Handing them to Bevy's worker pool cost more than the work:
  one thread per world took the game from 121% to 87% of a core on the phone
  and from 87% to 46% on the laptop, at 120 frames a second. **Except the
  render world on an iPhone** (and a Mac): there the system that makes the
  window's surface must run on the main thread, which only Bevy's own
  executor arranges. On one thread it ran on the render thread and panicked
  on the first frame ("can only access UIView on the main thread"), and iOS
  1.0.1 to 1.0.3 opened on a black screen and stayed there.
- **The gamepad plugin is off** (`src/lib.rs`). Nothing uses it, and left
  in, it polled for controllers every 8 ms. Turn it back on when there is a
  controller to read. Audio was off as well, keeping a stream open playing
  silence, until there was something to hear (2026-10-30: "The town has
  music", below).
- **Anything that runs every frame writes only what has changed**
  (`set_if_neq`, or compare first). Writing a `Transform` with the value it
  already had still marks it changed, and for a player that means placing
  their whole skeleton again; writing a UI `Node` has the whole screen laid
  out again.

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

## Building

Desktop: `cargo run`.

The game server: `server/deploy.sh` builds it for Linux on this PC and puts it
on the Vultr VM (`SERVER.md`). `cargo build` at the root builds the game
alone; the server is `cargo build -p roundtown_server`. Never `--workspace`,
which would build the game with the server's half of replicon in it.

Android needs the `release-android` profile, a manual strip step, and Gradle
(`versionCode` in `mobile/android/app/build.gradle` is the build number: 28
later the same night, the stick's middle wherever the finger holding it
first goes down, its disc going there and back to its own place as it is
let go (Hajun: "wherever the user tap first should be the middle";
`TouchControls::stick_origin`, `JoystickBase`, `stick_corner` in
`src/lib.rs`; protocol 15 still, no server change; built and not
installed, APK 27 kept beside it as
`donggeurami_town-release.prev-2026-11-14b.apk`); 27
in the small hours of 2026-11-14, the dynamite's arch drawn up the screen
with an X where it will land, its brown fuse, no three dots and no empty
buttons, the computer's players standing their ground against whoever
keeps coming at their base ("Games"), and the friends' panel's changes of
the night before ("Friends, groups and messages"; protocol 15 still, the
server deployed for the bots at 15:45 UTC on 2026-11-13, the binary before
kept on the VM as `roundtown-server.prev-2026-11-14`); 26
on the night of 2026-11-13, every word in Inter, a tag after every name,
and the friends' panel with its requests, groups and messages ("Friends,
groups and messages", "The font is Inter"), and with it what another
session had in the tree, Bridge Wars' dynamite and the computer's players
that no longer fall for three easy tricks ("Games"; protocol 15, the
server deployed with both at 14:07 UTC that day, its tables given the
friends' columns and tables as it started; the binary before kept on the
VM as `roundtown-server.prev-2026-11-13c`, and the database as it was
before tags as `/var/backups/roundtown/before-tags-2026-11-13.dump`); 25
an hour after 24, the compass's arrow pointing out with its words clear of
it, the anchors in their teams' colours, and the computer's players
carrying up to five kinds of thing and sending Bridge Builders (protocol
14 still; the server deployed with the bots); 24
in the small hours of 2026-11-13, five buttons for things to use and
several kinds held at once (protocol 14, the server deployed with it), and
with it what two other sessions had in the tree: the computer's players'
second round, the teams' colours, the compass, and the app's opening; 23
on the night of 2026-11-12, the computer's players that play the game in
a game played offline, the line about your anchor, and what a bot takes
flying to it (protocol 13 still; the server deployed with the bots
online that night); 22
in the afternoon of 2026-11-12, the sword held straighter at the ready
(protocol 13 still, no server change); 21
at midday on 2026-11-12, dying taking everything you hold, the fireball's band
seen over the void, and a sword held on its edge and cut downward (protocol
13 still; the server deployed for the first); 20
in the small hours of 2026-11-12, Bridge Wars' two shops and everything they sell, the swords,
the second button, and games of two to four people with no bots (protocol
13, the server deployed with it); 19
late on the night of 2026-11-11, what the generators make dropping onto
its pad, the level and the count written by the sapphire and amethyst
generators, grey panels, 25 m of bridge for 4 silver, and the camera at
one height over you in Bridge Wars (protocol 12, the server deployed with
it); 18
earlier that night, Bridge Wars' generators, its shop, its
timetable and its zombies, on Hajun's map of that day, with the anchors
lit by night (protocol 11, the server deployed with it); 17
on the afternoon of 2026-11-10, the metres of bridge left written in front of you
and running out of them stopping you where you stand (protocol 10 still,
no server change); 16
in the afternoon of 2026-11-10, a fork's root and a landing's end following the
bent edge they meet, no sliver off a curve of bridge (protocol 10 still,
no server change); 15 before it that day, a bridge that is built staying
as it was built
whatever is built past it, and unfinished ends square (protocol 10 still,
no server change); 14 earlier that day, sharp turns as L corners, short
forks square, 500 m
of bridge to start and text shadows in proportion to the letters; 13
earlier that day, the bridges cut flush along the edges they leave and
meet, at any slant, and a fork off your own bridge's side (protocol 10
still, no server change); 12
on 2026-11-09, the bridges a mitred ribbon, the minimap, the metres of
bridge, the resources under the health bar, half a second past hurting on
the server, the anchors at 120 (protocol 10 still; the server deployed the
same day with its two-tick game end, the binary before kept on the VM as
`roundtown-server.prev-2026-11-09`); 11
on the night of 2026-11-08, Bridge Wars online with eight seats in teams,
respawn anchors and bridges built by walking (protocol 10); 10 that
afternoon, the fight's minor things (dead bots hidden, fleeing, regen); 9
in the small hours of 2026-11-08, Bridge Wars' fight, its bots that go for
you, running, arrivals in the islands' middles and the jump beside the
sword; 8
late on the night of 2026-11-07, the town `roundtownmap.glb` again with the
game selector, Bridge Wars on the PvP map with Hajun's waiting room, and
its own buttons (protocol 9); 7
at the end of 2026-11-07, the map's fourth version of the day, every
island furnished and every arrival at its own marker (protocol 8); 6 half
an hour before: you arrive on the furnished island at its spawn marker
(protocol 7), the punch's reach 2 m and its range fainter, and out of the
car you walk again; 5 an hour before, the map's third version of
the day, one island furnished; 4 a quarter of an hour before that, the
punch's range band, the islands at 130 m and the planked bridges; 3 an
hour before that, with the range as a ring sector; 2 that morning, the PvP
town's; 1 for every APK before) —
see the "Build gotchas" section of `RENDERING.md`, or the commands in
`mobile/android/app/build.gradle`'s closing comment. The APK carries only
what the game loads: whatever `mobile/android/app/assets-left-out.txt` names
stays out (Blender files, reference pictures, retired models), and
`cargo test` fails if that would leave out a piece the shop sells, or
either of the town's buses.

The app icon, on both phones, is Hajun's picture `mobile/appiconroundtown.png`
(since 2026-10-08). `mobile/make_icons.ps1` makes the iOS icon and Android's
adaptive icon from it; run it again after changing the picture.

iOS cannot be built on Windows at all — it needs macOS. `codemagic.yaml` rents
one per build and ships the result to TestFlight. **Read `IOS.md` before
touching anything under `mobile/ios`**; it covers the Apple-side setup, why the
`.xcodeproj` is generated rather than committed, and what breaks if the shell
scripts are checked out with CRLF line endings. A TestFlight build is made by
pushing a tag, `ios-<version>`, after bumping `CFBundleShortVersionString`
in `mobile/ios/Info.plist`; the build number is Codemagic's own. Code behind
`target_os = "ios"` cannot even be type-checked here (`ring` wants Apple's
SDK), so keep it to call sites and put the logic in code every platform
compiles and tests, as `log_file` and `map::kept_in_library` do.

**An iPhone writes its own log** (`src/log_file.rs`, since 1.0.2): `log.txt`
in the app's Documents, which the Files app shows under On My iPhone >
Donggeurami Town. Everything Bevy logged in the last launch, and any panic
with its thread. iOS keeps the app's real log where only a Mac can read it,
so this is how a black screen says why: the first `log.txt` Hajun sent
(2026-10-08) named the black screen's cause on its own, after two builds of
guessing. Ask for it before guessing.

## Models

Authored in Blender and exported by `blender/export_glb.py` into `assets/`. A
metre in Blender is a metre in the world: anything the wrong size gets resized in
Blender and re-exported rather than scaled at spawn time (`map::ObjectDef`).

- **Every body is Hajun's `playercharacter.glb`** (`lobby::MODEL`, since
  2026-11-01: "try using playercharacter in asset folder instead of both
  boy/girl model for the main player and all the AI's"): every boy, the
  AIs, the House Builder and everyone online, whatever look they wear, but
  the girls, who are their `playercharactergirl.glb` since that night
  (`lobby::GIRL_MODEL`: "now there is playercharacter girl as well. use
  those as the player choice in the beginning of the game character
  selection"; the same rig, body and rest, with hair to the waist and a
  skirt, 3,440 triangles), and the AIs, who wear the two and Hajun's two
  recolourings of each (`roundtown_net::BOY_LOOKS`, `GIRL_LOOKS`, since
  2026-11-02; above, "The first time, you sign up") (`lobby::LOOKS` maps
  each look to its model and any other to the boy's; `lobby::models()` is
  every model, each once, which the APK's test keeps in and the walk's test
  reads the limbs of). A Rigify human metarig, scaled to 0.616 in the file (the
  game draws it as the file has it), under a skinned skin-coloured body with
  shoes, a blue top and dark trousers, and a mouth and two eyes hung rigid
  on the head bone: about 4,150 triangles in seven materials, hair among them,
  1.64 m to the top
  of a round head 0.53 m across, its eyes 1.38 m up, facing Blender's -Y
  as every model does, resting in an A-pose with its upper arms 60 degrees
  under level. That is Hajun's fourth export of the day (23:17, with hair;
  22:53 dressed and bare-headed); the first
  was a bare 1.70 m figure of 1,468 triangles, flat-shaded, with the arms
  28 degrees under level, the second the same smooth-shaded. The boy,
  `limbperson.glb`, the girl, `limbpersongirl.glb`,
  and the AIs' `AIpersonred.glb`, `AIpersonryellow.glb` and
  `AIpersonrpurple.glb` stay in `assets/` and out of the APK
  (`assets-left-out.txt`; a test keeps whatever `LOOKS` names in it).
- **The walk and the punch turn limbs about the body's own axes, whatever
  way the rig's bones lie** (`WalkBone::turned`, `bind_limbs` in
  `src/lib.rs`, since 2026-11-01). Each limb is bound by its bone's name
  (`walk_bone_kind`: `thigh.L`, `shin.L`, `upper_arm.L` and their `.R`;
  `Bone.005.L.001` and the like until then) with how it lies at rest in the
  body's space, the turns of every node from the body down to it, the
  loader's half turn among them; a swing is a turn about the body's
  side-to-side axis and the arms hang about its front-to-back axis, each
  turned down from however the model rests it to 75 degrees under level
  (`ARM_HANG_BELOW`, `stance_from_rest`, worked out as the arm is bound; a
  fixed 0.8 rad for the first export's arms, 1.02 for the old models'
  straight out, until that evening), and each thigh turned in the same
  way to stand 3.5 degrees out from straight down (`LEG_SPLAY`; Hajun,
  that night: "the legs are too wide apart while standing/walking": the
  model's rest 7 degrees out, its ankles 0.27 m apart, 0.19 stood so; the
  shin goes in with its thigh), put on the bone through its frame.
  Until then each turn was about an axis
  of the old rig's own bone space, and a new rig meant working them out
  again. Another rig needs only its bone names in `walk_bone_kind`; a test
  reads the model and checks every limb is found once and swings the right
  way (`every_limb_of_the_player_s_model_is_bound_and_swings_the_right_way`).
  Seen in the desktop build, scripted, from the front and the side: standing
  with the arms at the sides, walking with the legs striding, the back shin
  kicking up and the arms swinging against the legs, and the punch level in
  front. The sign-up's pictures (`assets/signup/boy.png`, `girl.png`) are
  rendered from the two models (`blender/render_person.py`, the same
  night). The colliders (`LEG_RADIUS`, `PLAYER_RADIUS`, "A
  body is the shape of the model") still bump as wide as the old model,
  0.42 m at the legs and 0.9 at the body, where this one is 0.35 and 0.45;
  what follows the model is the see-through's points on you
  (`see_through::RINGS`, `CROWN`), your eyes (`EYE_HEIGHT`, 1.38, "Your
  eyes are inside your head") and where the camera looks at a face while
  you talk (`talk::FACE`, 1.38). Names and the chat bubble still stand
  1.85 m up (`tags::OVER_HEAD`), 0.21 m over this head where they were
  0.14 over the old. Not yet on a phone.

## The first time, you sign up

Since 2026-10-31 (Hajun), the first time the app is run, or whenever the name
and the character are not set, three pages come before the town
(`src/signup.rs`): the town's name over a Start button, with "Already have an
account? Log In" under it; "Choose Username", "Shown to other players.
2-16 characters." on one line (a hyphen: the font has no en dash), a box
to type a name into and Save; and "Choose Character", "You may change this
later.", a boy and a girl on two cards with their pictures and nothing
else, since 2026-11-02 with a white arrowhead either side of each card
that turns it through that side's colours (`signup::ColourArrow`,
`Action::Turn`, `turn_cards`; Hajun: "put some arrow thingy next to each
guy and girl character selection in the start so u can choose which color
guy or which color girl"; pressing an arrow chooses that side as well; the
pictures, `assets/signup/<look>.png`, one to each look, are rendered from
the models by `blender/render_person.py`, which takes a `.glb` since the
colourings have no `.blend`; until then `boy.png` and `girl.png` from
`limbperson.blend` and `limbpersongirl.blend`), and Save (Hajun,
2026-11-01; until then three lines of explanation, "Save Username", "Save
Character" and "Boy" and "Girl" under the pictures). Solid colours, no
outlines, no words but those (Hajun:
"as minimalistic as possible"): black pages, the box and the cards dark
grey, the chosen card lighter, and white buttons with black words (since
the evening of 2026-10-31, Hajun: "black for the initial pages ... white
instead of yellow for the buttons"; dark blue, blue and gold until then).

- **It is kept on the device, `profile.json` beside `town.json`**, so the
  pages never come again; and online it is the account's: sent as soon as
  the server has let you in, if the account has you otherwise, and whenever
  it changes (`Ask::Profile`, the last of the asks, so that older apps are
  not told to update), kept in Postgres in `players.name` and `players.look`,
  which were there already, and sent to everyone in your room as a new
  member record. No other database: the pages are two columns on the
  accounts table. A native sign-in later (Apple, Google) is one more way to
  find that row, by the token's subject, and is where "Log In" goes; for now
  Log In says "Log in is not available yet".
- **The look saved is any of either side's** (`roundtown_net::BOY_LOOKS`,
  `GIRL_LOOKS`, `profile::look`; since 2026-11-02, `boy` or `girl` alone
  until then): the server checks the same list, so **it has to be deployed
  with the app**, or a profile in a new colour is thrown out whole, name and
  all, and the account keeps its generated name online.
- **A name is 2 to 16 characters of plain ASCII** (`roundtown_net::profile`,
  checked on both sides, the server ignoring any other): the game's font has
  no Hangul. It need not be unique: since 2026-11-13 every account has a
  tag after its name, `#1000` to `#9999`, and no two have both the same
  ("Friends, groups and messages", below). Typing: on desktop the keys and Enter;
  on a phone the soft keyboard, asked for by the window's `ime_enabled`,
  which winit shows on both phones. An iPhone's sends each letter as a key
  press and Return as a newline; Android's sends no keys at all, and
  writes into the activity's own text (`GameTextInput`), which the name is
  read off every frame (`signup::android_text`, through
  `bevy::android::ANDROID_APP`). Neither has been tried on a phone yet; the
  Android build type-checks. A phone's keyboard, with the phone on its
  side, covers most of the screen, so the name's page, and the settings'
  Edit name, slide up out from under it (`signup::Keyboard`,
  `signup::LiftsForKeyboard`, `lift_for_keyboard`; Hajun, 2026-10-31: "the
  keyboard hides the edit name box so u cant see the text anymore. make it
  go up in reaction to keyboard. as well as when u first set ur name"):
  laid out as with no keyboard and slid up by their own transform, by as
  much as puts their bottom 2% of the short side over the keyboard, and
  never so far that the box the name goes in is nearer the top than that
  (the heading may go off the top). On Android the keyboard's height is
  what the window's insets say: `MainActivity.onApplyWindowInsets` hands
  the IME inset's bottom to the game through a native method the game
  exports (`imeInset`, `Java_town_donggeurami_MainActivity_imeInset` in
  `signup`), kept in an atomic the game reads every frame, in the screen's
  pixels: times `RENDER_SCALE`, over the window's scale factor. **Not
  through `GameActivity`'s native side**: the first build of this called
  `GameActivity_getWindowInsets` on `AndroidApp::activity_as_ptr()`, which
  is a JNI reference to the Java activity and not the native `GameActivity`
  struct, and the app crashed on launch (Hajun, 2026-10-31: "app is
  crashing on launch"). An iPhone's winit says nothing of its keyboard, so
  there it is taken to cover 0.55 of the screen's height while a box is up
  (`IOS_KEYBOARD`, my guess). Seen in the desktop build with a keyboard of
  0.55 written in by a script, both pages; not yet with a real keyboard on
  a phone.
- **The character is a look** (`lobby::LOOKS`, `roundtown_net::PEOPLE_LOOKS`):
  `boy` is Hajun's `playercharacter.glb` and `girl` their
  `playercharactergirl.glb` (`lobby::MODEL`, `GIRL_MODEL`; below, "Every
  body is Hajun's `playercharacter.glb`"; since the night of 2026-11-01:
  for the day's first hours both were the boy's, and until that day
  `limbperson.glb` and `limbpersongirl.glb`, the same rig and bones, a
  pink top). The cards' pictures, `assets/signup/boy.png` and `girl.png`,
  are rendered from the two by `blender/render_person.py` (run again the
  same night). Every account from before wears `blue`,
  which is drawn as the boy, as is any look an app has not heard of. Saving
  swaps your body's model on the spot (`lobby::dressed_as`; the scene
  spawner takes the old one down) and your name in the town's room, so a
  game of House Builder offline is played under it; online, `net::redress`
  swaps anyone else's as their record changes, and their name over their
  head follows (`tags::rename`). The AIs are a boy or a girl by the toss of
  a coin, and then in any of that side's colours (`roundtown_net::ai_look`,
  `BOY_LOOKS`, `GIRL_LOOKS`: the boy and the girl people choose and Hajun's two
  recolourings of each, `playercharacterblackbrown.glb`,
  `playercharacterwhitejeans.glb`, `playercharactergirlwhiteblue.glb` and
  `playercharactergirlyellowpink.glb`, looks `boy_blackbrown` and the like
  in `lobby::LOOKS`; on this device in `lobby::ai_member` and for the
  server's bots in `server/src/builder.rs`, since 2026-11-02; Hajun: "make
  the ai's also be random girl/guy 50/50", "use variations of the colors. i
  made various colors for both guy and girl for the ais"); until then they
  took turns at red, yellow and purple, which stay known, drawn as the boy,
  for the server's bots until it is deployed again and for older apps, as
  any look an app has not heard of is.
- **While the pages are up the world waits as under a dialog**
  (`hud::DialogUp`): you stand still, every press is the page's, the cursor
  is free on desktop, and the town's name waits to fade in until the pages
  have gone. Deleting your account (`settings`) forgets the profile and
  brings the pages back.
- Seen in the desktop build on 2026-10-31, scripted with posted clicks and
  keys: all three pages, Log In's notice, Save greyed until there is a name
  or a choice, the girl chosen and walking in the town under the typed name,
  the name and look on a local server, and a relaunch going straight to the
  town.

## Games: the gamepad opens a selector, and Bridge Wars is the PvP map

Since the night of 2026-11-07 (Hajun: "i want the default map to open when
u open the app to be roundtownmap.glb, and when u press the game icon ui
button, it will open a popup game selector menu with 3 games (the first one
is called 'Bridge Wars' the second is called 'Home Builders' and third is a
??? coming soon. you select one and you get a quick black loading screen
and then when the new map is loaded, the player is teleported to either
the pvp map's (waitingroom box above the map until 4 ppl is queued, which
at that point it will count down from 5 and then teleport each player to
each of the 4 island), or the build battle island for queueing - same as
before ... if there is no player joining for 10sec, fill it with bots ...
just make it so the pvpmap game (small) timer of 10 minute (tap to skip)
and after it, the players go back to the main roundtownmap. pvpmap doesnt
need buses and doesnt need npcs. also when in the roundtownmap or the
build battle, it will be freecam, but in the pvpmap game it will switch to
top down cam and the button ui should change to pvpmapbuttonui.jpg
layout"), the town is `roundtownmap.glb` again, with the sea round it and
everyone on the path round its pond, the camera behind you as it was, and
the PvP map, `pvptownmap.glb`, is a game of its own, Bridge Wars, on an
island of its own, `Venue::Arena` (`roundtown_net::Venue`, `island::model`,
loaded as the app opens with the rest; `PROTOCOL_VERSION` 9, the server
deployed with it, the binary before kept on the VM as
`roundtown-server.prev-2026-11-07d`). For the day of 2026-11-07 the PvP map
was the town itself; everything below about the map, its islands, the
void, the punch's knockback and its range held for the town then, and
holds for the arena now.

- **Since the small hours of 2026-11-14 the dynamite's arch rises and falls
  and leans to neither side, with an X where it will land; its fuse is seen
  to be brown; the three dots are gone, and a button for a thing to use is
  shown only with something on it; and whoever keeps coming at a team's
  base is stood against, sooner or later, by who each of them is** (Hajun,
  having played APK 26, in three messages late on the night of 2026-11-13:
  "i want the dynamites hit indicator's curved part to be curved only in
  the Y axis (up and down) not the sideways axis. its a arch projectile not
  a boomarang. also i want there to be some indicator of where the dynamite
  will land, maybe a x in the middle of the circle, and the arched hit
  indicator rectangular part should be connected to the middle of the
  circle. also read BridgewarsAIbottleneck. i just tested the new version
  and found another ai interaction that could solved. fix and
  rebuild+repush"; "also i think the dynamite's string part should be
  brown, like a string."; and "also i dont think the UI ... button is
  necessary. just remove it and keep the default as the only option - only
  showing the button if the slot is filled and not showing empty
  buttons"). **`PROTOCOL_VERSION` 15 still**: nothing that is sent has
  changed (a stick is sent as where it was thrown from and to, as before,
  and how it is drawn between the two is each device's), so APK 26 plays
  on against the server. The bots' part is the server's online, **deployed
  at 15:45 UTC on 2026-11-13** (00:45 on the 14th here; the binary before
  kept on the VM as `roundtown-server.prev-2026-11-14`); the rest is the
  app's, **APK 27**, which has the friends' panel's changes of the same
  night as well ("Friends, groups and messages", below).
  - **The arch** (`bridge_wars::lob_track`, `Lob::track`, `Lob::rise`,
    `DYNAMITE_LEAN`, `UP_THE_MAP`). A stick is thrown straight at where it
    comes down, and rises and falls on the way (`Lob::apex`: 0.8 m over
    the straight line of the nearest throw, 2 m over that of the
    furthest). The camera is straight over you, and from there a throw
    that only rose and fell would be seen going along a straight line,
    however high; so it is drawn as an arch is on a map, **up the screen
    as it rises and back down it as it falls**, by as much as it is high
    (`DYNAMITE_LEAN`, 1; north, the game's -Z, whichever way it is
    thrown). Thrown to the left or the right, the whole arch is seen;
    thrown up the screen or down it, it goes along a straight line. For
    its first day it swung out to the thrower's right by 0.18 of the
    throw's length (`DYNAMITE_BULGE`, gone), which is what Hajun saw as a
    boomerang. Where it comes down, when, and what it blows are as they
    were, and the server's online; the leaning is only where each device
    draws it on the way.
  - **The band** (`utility::lob_arch`, `range::arch`): the same arch, 0.6
    m wide as before, built in the air: each of its 24 stretches as high
    as the stick will be there, and as far up the screen, from the hand to
    **the middle of the ring** (it stopped at the ring's edge, and in the
    nearest of throws was not drawn at all), with its fill, its outline
    and its dark rim (`range::arch` takes a line through the air;
    `range::ribbon`, along the ground, is gone). Built again as the throw
    changes by 4 cm or turns by half a degree, which way the top of the
    screen is from it changing as it turns; its sheets are never left out
    for where they were a moment ago (`NoFrustumCulling`). **And an X in
    the middle of the ring** (`range::cross`, `LANDING_CROSS`): two
    strokes a metre long and 17 cm thick, in the band's white with its
    dark rim, aslant on the screen whichever way you aim (the ring is
    turned back by as much as the band is turned with the aim).
  - **The fuse** (`dynamite::FUSE_RADIUS`, `FUSE_LENGTH`): 0.42 m long and
    0.14 across where it was 0.26 and 0.09. It was brown all along
    (`colours.arena.dynamite_fuse`), and from the camera over you a dark
    hair at the sticks' end, of no colour at all. **Its picture is in two
    colours** (`hud::painted`, `Picture::in_its_own_colours`,
    `hud::repaint`): the sticks red and the fuse brown, the file's own
    two, on the button and on the shop's card, where the whole picture
    was tinted the sticks' red, with a dot for a spark at the fuse's end,
    which is gone. It is the one picture not tinted where it is shown
    (`market::look_of` gives it white), so it is painted again as the
    file changes.
  - **The buttons** (`hud::DOTS`, `utility::SECOND_ALWAYS`, both `false`,
    the code behind each kept): the three dots are not put up, and
    nothing shows an empty button. **A button is shown only with
    something on it**, the second one too, which until then stood empty
    up and to the left of the sword from the start: with nothing to use,
    the sword has the jump and the bridge buttons round it and no more.
    That the second one goes with the rest is my reading of "not showing
    empty buttons"; `SECOND_ALWAYS` is the one word to have it back.
  - **What Hajun found** (`BridgewarsAIbottlenecks.txt`, its one item
    now: "When i go to invade another base, i go near their island, and
    attack the spawner, then the 2 enemies of that island come toward me
    to defend, and i retreat back a slight bit to the bridge, and then
    they both go back to what they were doing (going to their generator)
    so i go back to attack the spawner a little bit, then i retreat back,
    and i keep repeating this until their spawner destroyed"; it asks for
    a patience that differs from one to the next, standing ground after
    the first, second or third "agitation", "until he rly rly leaves",
    and says "if some thing becomes a definite if-then, then its not a
    real human, but a program"). A bot had no memory of anyone: it went
    home for whoever stood on its island, and back to what it was doing
    the moment they stepped off it, as often as they liked.
  - **Standing their ground** (`shared/src/bots.rs`: `Vigil`, `Stand`,
    `Squad::keep_vigil`, `guard_spot`, `Goal::Guard`, `GUARD_SCORE`
    0.79, over anything a bot does about its island and under seeing
    someone off it and backing off hurt). Whoever of another team is on
    the land of a bot's island, or within 5 m of its anchor as that loses
    health, it has in mind from then on, and counts each time they come
    again after having stepped back (`Vigil::comings`). **Each bot has
    its patience, 1 to 3 comings** (`Persona::patience`, `patience_of`:
    its kind's number and a bell's throw, rounded: ISFJ 1.0, INFJ 1.1,
    ISTJ 1.2, ESTJ 1.3, ESTP 1.4, ENTJ and ESFJ 1.5, ENFJ 1.6, ISTP 1.7,
    INTJ 1.8, ENTP 2.2, ESFP 2.4, INTP 2.5, ISFP 2.6, ENFP 2.7, INFP
    2.8; thrown again each time it has forgotten someone). When they step
    back and it has not had enough of them, it looks after them for 0.8
    to 4 s (`VIGIL_LOOK`) and goes back to what it was doing; once they
    have come as many times as it has the patience for, **it stands its
    ground**: on its own land between them and its anchor, on the line
    from them to it, 3 to 5 m from them and 0.9 to 2.2 m in from the
    shore (`VIGIL_OFF`, `VIGIL_INSIDE`, the more for one that minds being
    thrown off), or, with too little land for that, right in front of the
    anchor; the two of a team 0.9 m to either side of the line; turned to
    them, and moving as they move. **At them if they come near**: within
    5 to 12 m of where it stands, times how bold it is (`VIGIL_WATCH`:
    measured from where it stands, not from where it is, so that it is
    not led out a step at a time), it goes out onto the bridge after
    them, and back to its place when they are further. **There until
    they are really gone**: 24 to 38 m from the middle of its island
    (`VIGIL_GONE`; the bridge's far end is 44 m off) for 2.5 to 8 s
    (`VIGIL_CALM`), when it goes back to what it was doing. **And not for
    ever**: after 12 to 36 s of it (`VIGIL_STAND`) with them still about,
    it either goes out at them, as far as they are and 6 m more, for 6 to
    14 s (the likelier the bolder and the better at a fight: 0.25 and 0.3
    of its `pvp` and 0.35 for each 1 of `bold` over 1, between 0.1 and
    0.85), or lets them be, and has not forgotten them either way: the
    next time they come it stands at once. Left alone 50 to 110 s
    (`VIGIL_FORGET`), it has forgotten them. Every number is thrown from
    its spread each time it is needed, so no two stands are alike.
  - **And nothing else it does about a fight begins or ends on the same
    line twice** (`KEEPS_ON`, `KEEN`, `KEEN_LASTS`, `STING`, `NERVE`):
    how near it lets people come before it goes for them is 0.72 to 1.35
    times its usual, thrown again every 4 to 11 s; someone it is on
    already it keeps after 1.3 times as far; a blow smarts for 2.2 to
    6.5 s, in which it goes for whoever is within 9 m (3 s, always,
    until then); and where it backs off hurt is up to 0.08 of its health
    either side of its usual, thrown again after each fright.
  - **Two bots after each other across a hole fell into it, and do not
    now** (`Squad::act`, `Squad::fight`, `Bot::awaits`, `Nav::leap`,
    `Nav::nearest_walkable_within`). Found by the server's whole games:
    with the dice falling otherwise for the throws above, in one of its
    eight a bot walked off an edge. Two bots of different teams, either
    side of a hole a stick had left in a bridge, each jumped it at the
    other at the same moment, passing in the air, three times running,
    and the third time one fell. Four things, each mended: **someone on
    their way over a gap to it is waited for** (in the air over the
    void, or running at it with no straight way between them: it jumps
    none itself until they are down or have stopped); **a jump comes
    down a step in from the edge** (0.9 m, wherever whoever it is after
    stands: with them on the very edge, the place to come down was the
    edge itself); **in the air it keeps on for where it means to come
    down and stops over that very spot** (the jump was over as soon as it
    was within 0.4 m of there, in the air or not); and **what someone in
    the air steers back for has ground under it** (the squares walked
    along a plank reach round its end, out over the hole: it steered for
    the middle of the one it was in, to and fro about it, down into the
    void).
  - **Whole games, again** (a hundred and twenty of bots alone, as
    deployed): over at 12:32 on average, none drawn; won by island 33,
    26, 26 and 35; before 10:00 twelve and a half deaths by punches and
    eleven by falls a game, and 0.5 anchors broken by bots: much as it
    was (12:29, twelve and a half and ten and a half, 0.4).
  - **Checked on 2026-11-14.** Unit-tested: the arch (to neither side of
    its throw, up the map by its height, straight thrown up or down it),
    the band and the X, the two-coloured picture, that no empty button
    is shown; someone who keeps coming stood against at the coming its
    kind is like (of 24 each: ISFJ at the first 21 times, INTJ at the
    second 21, ENFP at the second 8 and at the third 16), a bot keeping
    between them and its anchor until they are gone, and what it does
    when it has stood long enough (of thirty stand-offs the lone wolf
    went out at them 19 times and let them be 11, the dreamer 4 and 26);
    the jump past someone on the edge of a hole, 24 of 24 down on the
    planks, and the wait for someone coming over, which fails without it.
    **Scripted in the desktop build**, offline (a throwaway
    `src/arena/scratch_run.rs`, removed): the band aimed nine ways, to
    the right at three lengths, to the left, up the screen, down it and
    across the corners, by day and by night, the X an X in each and the
    band meeting the ring's middle; a near throw and a far one watched
    along their arch; the fuse close by and on the button; the buttons
    with nothing held and with one kind; and a team baited as Hajun did
    it: both at the head of their bridge between the baiter and their
    anchor, one out at the baiter standing 4 m off, both back to their
    places, and both back to what they were doing 7 to 16 s after the
    baiter had gone 31 m off. **On the VM after the deploy**, nobody
    having been on it for an hour, this PC's own account was let in and
    seated with seven of the computer's, three of their bridges across
    by half a minute; the script then stood on another team's island,
    whose two bots had it dead in six seconds; the journal clean. APK 27
    is built, its 85 asset files checked byte for byte, its
    library the stripped one with its entry points, and its number read
    back, and **not installed, no phone being attached** (APK 26 is kept
    beside it as `donggeurami_town-release.prev-2026-11-14.apk`). **Not
    by hand, and not on a phone**: the arch and its X under a thumb, and
    a real person baiting the server's bots, are unseen.
  - **What is mine**: that the arch is drawn up the screen, and by as
    much as it is high; that the band is built in the air; the X's size;
    the longer, thicker fuse; hiding the empty second button; every
    kind's patience and every number of a stand; that a bot which has
    stood long enough goes out at them or lets them be by chance; that it
    forgets; the spreads of `KEEN`, `STING` and `NERVE`; and that a bot
    waits for someone already on their way over a gap.

- **Since the night of 2026-11-13 there is dynamite, which cuts bridges;
  the computer's players no longer fall for three easy tricks, and are not
  all as good as each other at a fight; and one whose teammate is a person
  is out and about** (Hajun, in three messages: "read the
  bridgewarsgameevent.md, i made some update like adding the dynamite
  item. add the dynamite item and functionality to the minigame, test it
  thoroughly. dont need to build apk or push to git when ur done"; "also
  read BridgewarsAIbottlenecks.txt. those are some of the things i noticed
  while playtesting against the current ai bots. try to fix them as well";
  and "the ai teammate right now when ur in a team with a bot, the bot
  stays on the iron/gold generator for toooo long. reduce the time it
  spends on the generator and make it go out more or use shop longer or
  bridging or raiding an island more and just come back to pick up
  resource and leave again quickly not stay"). **Hajun's rules file is
  `bridgewarsgameevent.md` since that day** (`bridgewarsgameevent.txt`
  wherever this file says so of an earlier day); besides the dynamite it
  has the healing potion giving back half of your health where it gave a
  quarter (`bridge_wars::HEALING`; a green "+50"), and so does the game.
  **`PROTOCOL_VERSION` 15, the number it shares with the friends system of
  the same day** ("Friends, groups and messages", below): a `Kit` with a
  stick of dynamite on a button is nothing an app from before can read,
  so the server and the app go out together, and the two pieces of work
  with each other. **The session that made this neither deployed, built an
  APK nor committed**, as asked: it goes out with the friends system, by
  the session that made that ("Building", above, has the build).
  - **The rules** (`roundtown_net::bridge_wars`; Hajun's file: "Dynamite: 1
    amethyst (similar to default dynamite attack for dynamike in
    brawlstars, hold&drag&letgo mechanic like fireball, but instead of one
    straight rectangular hit range indicator its like an long arched
    curved rectangular, with ability to change the total distance with the
    magnitude of the hold&drag&letgo, and when fired, it will throw an
    arched dynamite (2 minimalistic mini red cylinder shape with smaller
    brown cylinder sticking out of one face which is the string), and
    after landing on the ground/bridge, it will show a circular blast
    radius range around it and it will shine bright red 3 time before it
    explode, and when it explode, it will do 20 damage to player in that
    blast radius and destroy that blast radius region of the bridge - can
    stack up to 3 times max at once)"). `Utility::Dynamite` is the sixth
    thing for the buttons: 1 amethyst at the item shop, three to a button
    (`most_held`), aimed as the fireball is (`Utility::aimed`). A stick
    thrown is a `Lob`: from the hand, 1.2 m up and half a metre in front
    of the thrower (`DYNAMITE_HAND`, `DYNAMITE_AHEAD`), to the ground 2.5
    to 10 m from them (`DYNAMITE_NEAREST`, `DYNAMITE_FURTHEST`; whatever
    is asked for is brought within those), in 0.45 s and 0.04 s more for
    each metre (`Lob::secs`: 0.55 s the nearest, 0.85 s the furthest),
    along an arch (for its first day one that also swung out to the
    thrower's right by 0.18 of its length, `DYNAMITE_BULGE`, so that the
    camera straight over you saw an arch at all; since 2026-11-14 it is
    drawn up the screen as it rises, above). Down, it lies 1.2 s
    (`DYNAMITE_FUSE`), shining three times (`DYNAMITE_FLASHES`,
    `Lob::shine`), and blows: 20 (`DYNAMITE_DAMAGE`), less what a shield
    keeps off, to everyone of another team than the thrower's whose feet
    are within 3 m of it and half a body's width more (`DYNAMITE_RADIUS`,
    `in_dynamite`), zombies among them, with no knock; whoever it hurts is
    seen again if they were unseen, as by any blow. And **every plank
    whose middle is within 3 m of it is gone** (`nav::plank_in_blast`),
    whoever laid it, from under whoever stands on it: mostly four planks,
    5.45 m of bridge, sometimes five. One that comes down with nothing to
    lie on within half a metre (`DYNAMITE_CATCH`, `caught`) falls on into
    the void and never blows. The server's online (`Game::lob`,
    `Game::dynamites` in `server/src/arena.rs`), this device's own in a
    game played offline (`arena::dynamites_offline`).
  - **Aiming it** (`utility::aim`, `Hold::reach`, `Hold::far`): held and
    dragged as the fireball is, and **the further the finger is dragged
    the further the throw**: from 4 shares of the short side from where it
    went down (`AIM_DEAD`, as ever), the nearest of throws, to 22
    (`AIM_FULL`, about a button and a half), the furthest. What is drawn
    is a band 0.6 m wide that bends as the stick will fly (for its first
    day out to your right and back along the ground, `range::ribbon`
    along `lob_line`, ending at the ring's edge; since 2026-11-14 an arch
    in the air to the ring's middle, with an X there, above; `LobBand`),
    and a ring as wide as the blast, round where it will come down
    (`LandingRing`, `range::disc`: mine, as Brawl Stars shows where a
    thrower's shot lands), in the fireball band's own colours with its
    dark rim (`colours.arena.aim_*`). In the nearest of throws the ring
    reaches back past your hand (and for that first day the ring alone
    was drawn). Dragged back to where the finger went down
    it is cancelled, a quick tap that never dragged throws 4.5 m the way
    you face (`DYNAMITE_TAP`), and so does its key. Half a second between
    two, as with the fireball.
  - **What you see** (`src/dynamite.rs`): two red sticks side by side, 0.8
    m long and 0.64 across the pair, with a brown fuse out of one end
    (`colours.arena.dynamite`, `dynamite_fuse`), turning end over end one
    and a half times on their way; lying, a red ring on the ground as far
    as the blast will reach (`blast_outline`, `blast_fill`, with the
    band's dark rim, so that it is seen over the void), and the sticks
    shining three times: five times their own red given off (`SHINE`) and
    a red glow round them out to 1.1 m; then the fireball's own ball of
    fire (`fire::burst`). One that found nothing to lie on tumbles on
    down and is gone 35 m below. A plank blown leaves its bridge with a
    hole: the ends left are capped, what is left of the bridge stays as
    it was built (`Chain::gone`, `holed`; `bridge::Bridges::blast`,
    `break_planks`), the plank you are laying goes too if the blast
    reaches its middle (`blast_tip`), the minimap shows the gap, and
    planks laid across a hole afterwards meet the ends left flush, as a
    bridge meets a shore. The button's picture and the shop's card are
    the two sticks and the fuse (`hud::Picture::Dynamite`).
  - **Online** each stick is an entity the server sends as it is thrown
    (`Dynamite`: who threw it, from where and to where), which every
    device lobs on by itself (`dynamite::follow_server`), and takes away
    as it blows, sending in its place where it blew and which planks went
    (`Blasted`, kept in the game's room with its planks, so that someone
    who comes late has the holes; `dynamite::hear_blasts`); one taken
    away with no blast fell into the void there, and falls on here.
    `Ask::Use(Use::Dynamite { heading, far })` throws one: the server
    takes it off the button, throws it from where it has you standing,
    and refuses numbers that are none.
  - **The shop**: the item shop's third row is six cards now, the
    dynamite's last ("Dynamite", "Cuts bridges", `words.market.dynamite`
    and `dynamite_stat`: the line is mine), and a card in a row of six is
    narrower, 14 shares of the short side where the rest are 15.5
    (`market::NARROW_CARD`, `card_width`), so that the row ends short of
    the second button on a screen of sixteen by nine: six of the other
    width reached under it. **"Slots full" comes up now**: with five other
    kinds on the five buttons, the sixth cannot be bought.
  - **The computer's players and dynamite** (`shared/src/bots.rs`). Each
    kind has its leaning to it as to the other five things
    (`Persona::carries`, six numbers now: the trickster and the tinkerer
    most, 0.4, the captain and the sentinel least, 0.05), and one a fifth
    of whose leanings are to it keeps two, the rest one
    (`Persona::loadout`). **It is thrown at whoever of another team is out on a
    bridge** a throw away, at where they will be when it blows if that is
    still on a bridge, to take the bridge from under them or from in
    front of them: from the land, or from a bridge itself if what would
    go is well clear of where it stands; never where a teammate stands on
    the planks that would go; 6 to 12 s between two (`Order::blast`).
    **A stick thrown at one it gets out from under** (`Squad::mind_sticks`,
    `Stick`, `View::sticks`): once it has woken up to it, 0.75 to 2 s
    after the throw for the worst of them at a fight and 0.35 to 1 s for
    the best, a quarter of a second later for anyone in a fight
    (`STICK_WAKES`, `STICK_WAKES_SOONER`, `STICK_BUSY`), it runs for the
    nearest ground clear of the blast and keeps out until it has blown,
    whoever it was fighting in there; one lying on a bridge ahead it
    waits short of; its own team's is nothing to it on the land. Of
    eighty sticks thrown from close by to come down right where one
    stood: the four best kinds were never caught, the middling ones
    hardly ever in the open and up to 15 times in a fight, the worst
    (INFP) 34 times in the open and 55 in a fight; a test has every
    kind's
    (`a_bot_gets_out_from_under_a_stick_of_dynamite_and_waits_short_of_one_on_its_way`). For its first hours every kind was out in half a second, and
    then walked back in after whoever it was fighting. **A hole in a
    bridge it jumps or mends** (`nav::Nav::blast`, `gaps`, `crossing`,
    `leap`; `Body::leap`; `Squad::reach_of_a_jump`): one of up to 5.8 m,
    the four planks a stick mostly takes, a bot at a run jumps, on its
    way anywhere and after whoever it fights; a longer one, in its team's
    way to the middle, either of the team lays planks across, from
    whichever side it stands on, half a metre inside what stands to half
    a metre inside what stands on the far side (`mend_for`, `MEND_INSET`),
    with the metres it has or buys.
  - **The three easy ways to beat them** (Hajun's
    `BridgewarsAIbottlenecks.txt`: "These are the list of my personally
    observed "easy" optimal ways to defeat AI's"). **Running round in a
    circle** ("the AI always is one step behind so he just keeps
    following in a circle behind me in an endless loop"): a bot goes for
    where whoever it is after will be, not where they are (`Squad::meet`,
    `foresee`, `Track`: their pace and how fast they turn, watched from
    frame to frame; no further ahead than 1.6 s, and no further than there
    is ground under them), so someone going round and round is cut off
    across the round. **Jumping from one bridge to another beside it**
    ("instead of the ai also jumping across, it goes all the way back to
    the island"): with a gap it could jump between it and whoever it
    fights, no walking straight to them and the way round far or none, it
    stands a moment and jumps, the likelier the bolder and the better it
    is (about one time in two for the worst of them, nearly always for
    the best), or thinks better of it, goes round, and thinks of it
    again in a few seconds. **All of them
    equally moderate** ("when i spam click the attack button while
    fighting, i always seem to hit them first, and they seem to get
    knocked back more than they knock me back, so i can always win even
    in 1v2 ... some bots are like pro-er, and some bots are moderate like
    now"): each kind has how good it is at a fight, from 0, as they all
    were, to 1 (`Persona::pvp`, each bot's own a little off its kind's,
    `Temper::pvp`): the lone wolf 1, the daredevil 0.9, the commander 0.8,
    the enforcer 0.7, the trickster 0.65, the mastermind 0.6, the troll
    0.55, the free spirit 0.5, the captain and the sentinel 0.4, the
    watchman 0.35, the drifter 0.25, the tinkerer and the host 0.2, the
    defender 0.1, the dreamer 0. The better it is, the sooner its first
    punch comes once it has someone in reach and the nearer its punches
    come to as fast as they count (down to one every 0.53 s, a body being
    past hurting for half a second); the oftener it steps back out of
    reach between two and comes in again on the run timed to the next,
    since a punch thrown running in knocks further than one thrown
    standing, which is why they were knocked back more than they knocked;
    the sooner it is on its feet after a knock and the harder it steers
    against one; and the more of where you are going it reads.
  - **A bot whose teammate is a person is no keeper of the pad** any
    more. It comes to the pad for what lies there once that is worth the
    walk (as many things as it wants, six at the most) and you are not
    there for it yourself, is on it no longer than 1.4 s, and is not back
    for 9 to 16 s; between visits it shops, wanders its island away from
    the pad, and goes out: from 45 s, with nobody of another team within
    25 m of its island, for the amethyst on your side of the middle and
    the sapphire of the smaller islands beside yours; from 30 s it builds
    the bridge to the middle if nothing joins your island to it, unless
    you are out over the void building one yourself (40 s until then),
    and from 75 s one to a smaller island (two minutes until then); and
    from a little after the raids begin, 2:50 at the soonest, it raids by
    itself, which it never did. In a test of seven minutes beside a
    person who stays in the middle of their island, three times over, it
    is by the pad 14 to 18 times, 6 to 10 s of it in all and never longer
    than 2.8 s at a time, off the island for 249 to 299 s, and has raided
    and fetched every time
    (`a_person_s_bot_teammate_comes_to_the_pad_for_what_lies_there_and_is_off_again`).
  - **Three faults found on the way.** A bot's body that came down to
    within a hair of the ground without quite touching it stood there
    still falling, as far as it knew, for as long as it stayed on ground
    of that height: neither in the air nor its own to move, so it never
    jumped nor drank (`Body::step`). **A mesh with no corner in it is
    not to be put in `Assets<Mesh>`**: Bevy's mesh allocator logs "Use-
    after-free: attempted to copy element data for an unallocated key"
    for each, as the nearest of throws did for the band's three sheets,
    none of it being outside the ring; they are put away as they are
    instead (`LobSheet`). And a bot that had got out of a blast walked
    straight back into it, above.
  - **Whole games, again** (two hundred of bots alone, as it stands):
    over at 12:29 on average, none drawn; won by island 43, 40, 63 and
    54; before 10:00 twelve and a half deaths by punches and ten and a
    half by falls a game, where it was nine and five (the ones that are
    good at a fight punch on the run, which throws whoever they hit
    further), and 0.4 anchors broken by bots; each kind's team winning 7%
    (the sentinel) to 55% (the lone wolf) of its games, where every
    kind's won 20% to 31%: a lone wolf dies 2.0 times a game and a
    captain 4.4. The eight of a game together bought 1.9 sticks of
    dynamite and threw 0.6, which took 2.5 planks, the most in one game
    four sticks and eighteen planks; and 12.9 fireballs, 2.3 speed
    potions, 1.0 invisibility potions, 25.0 healing potions and 1.4
    Bridge Builders, of which they used 11.1, 2.0, 0.6, 8.8 and 1.2.
    `many` prints all of it (`Chronicle::blasts`, `planks_blown`).
  - **Checked on 2026-11-13.** Unit-tested: the lob, its arch, its fuse
    and its three flashes, whom a blast reaches, and what a kit and a
    shop make of a sixth kind; planks taken out of a map and out of a
    drawn bridge, the ends left, the plank being laid, and a hole mended
    flush; the server's stick from throw to blast; the band and the ring;
    a bot out from under a stick kind by kind, short of one on its way,
    over a hole of four planks and mending one of five, throwing at
    whoever comes over a bridge and at nobody else, cutting off someone
    who runs in circles, and jumping after someone on the next bridge;
    and the teammate's visits to the pad. **Scripted in the desktop
    build** (a throwaway `src/arena/scratch_run.rs`, removed), offline,
    by day, at sunset and at night: the card bought three times and
    refused a fourth, "Slots full" with five other kinds; the band at
    three lengths and four ways, over the land and over the void, and
    dragged back; a throw watched through its flight, its ring, its three
    flashes and its blast from the game's camera and from close by; one
    thrown off the island, falling; a bridge of fifteen rungs cut from the
    shore, four planks gone, the gap on the minimap, standing on what was
    left past it, the planks under my own feet blown by my own stick and
    the fall, and walking off the cut end into the hole; sticks thrown at
    me: 20 off, 18.8 through a Basic Shield, nothing from a teammate's,
    and dead of one, with everything held gone; sticks thrown at two of
    the computer's players on their island, one of which got out every
    time and the other was caught twice, 100 to 80 to 60; and their
    bridge to the middle cut, on which, within six seconds, the one
    coming home jumped the hole while its teammate came out from the
    island and laid six rungs across it. **Against a server on this PC,
    with two copies** seated as teammates: the sticks bought from the
    server, the fourth refused; the second copy seeing the first's stick
    fly, lie, flash and blow, and four planks go from the bridge on its
    own screen; one thrown over the void, which the server had fall; the
    near end of the bridge blown from under the second copy, which took
    no harm from its teammate's stick and fell with the planks, the
    server logging the fall; and one of the server's bots taking 20 from
    a stick, the number floating off it. **Not by hand, and not on a
    phone**; no stick thrown by one of the computer's players has been
    seen on a screen, they throw so few, and that is the tests' alone.
    Known and left: thrown the full 10 m straight up or down the screen,
    half of the ring is off it, and a far throw up the screen lands under
    the time; and on a window of sixteen by nine the four more buttons,
    when they are shown, lie over the right end of the shop's bottom row,
    as they have since there were five.
  - **What is mine, the file not saying**: that a blast harms neither its
    thrower's team nor an anchor, and knocks nobody; that a plank goes
    whole, by its middle; the 3 m of the blast (the fireball's), the 2.5
    to 10 m of a throw and the 4.5 m of a tap; how long it flies and the
    1.2 s it lies; that the arch leant to the thrower's right (for its
    first day); the ring
    where it will land; that one over the void falls and never blows;
    the sticks' size and their glow; "Cuts bridges"; the narrower cards;
    everything about the computer's players and it; every kind's number
    for a fight and what it changes; and every number of the teammate.

- **Later on 2026-11-13: the compass's arrow points out, the anchors are
  their teams' colours, and the computer's players fill their buttons,
  each as it is like** (Hajun, having played APK 24, in three messages:
  "the player compass tracker, i think the arrow need to be flipped? the
  arrow icon is pointing inward at the player rather than outward. fix and
  push to git and build apk again"; "also now, there is 4 more item slot,
  so the ai should be adjusted so it can buy up to 5 even. match it with
  the personality so more diverse interaction"; and "also make the color
  of your spawn anchor (right now its purple) but make it so the color is
  your team's color. so red blue green yellow glow"). **The bots' part is
  the server's online, deployed at 16:43 UTC on 2026-11-12** (01:43 on the
  13th here; the binary before kept on the VM as
  `roundtown-server.prev-2026-11-13b`; `PROTOCOL_VERSION` 14 still, what a
  bot is ordered to do being the brain's own and nothing sent, so APK 24
  plays on against it). The arrow, the anchors and the bots of a game
  played offline are the app's: **APK 25**.
  - **The compass's arrow was drawn upside down** (`hud::Picture::Compass`).
    `compass::point` turns the picture from pointing up the screen to
    pointing at whoever it tracks, and the picture had its tip at the
    bottom: `hud::drawn` has y up, as every picture's comment says, and I
    wrote its tip at y -0.92. So round you the arrow sat on the side its
    target was on and pointed back at you. I had looked at it the day
    before in a screenshot at full size and taken it for right; enlarged,
    it plainly points in. **Look at a small thing enlarged.** It is drawn
    tip up now (its card in the shop too), and a test keeps the tip at the
    top (`the_compass_s_needle_points_up_its_picture`). **And its words
    keep clear of it** (`compass::past`, `WORDS_GAP`): their middle was a
    fixed 15.5 shares of the short side out, which up and down the screen
    cleared the arrow and across it did not, a name and a distance being
    eight times as long as they are tall ("Dori 76m" written over the
    arrow, seen in the run that checked the flip); they begin one share
    past the arrow's tip now, whichever way it points. Seen from four
    places: up and to the left at "Ddol 179m" from home, left, right and
    straight up from the middle of the map, each time pointing out at the
    island whoever it named was on, in that team's colour.
  - **Each team's anchor is its team's colour** (`arena::paint_anchors`,
    `pale`; `colours.arena.anchor_red`, `anchor_blue`, `anchor_green` and
    `anchor_yellow` in `assets/ui.ron`, `ArenaColours::anchor`): each
    team's pillar wears a material of its own, the map's `spawn` material
    in its team's colour (`#e0352b`, `#2f6fe8`, `#2fbf4a`, `#f2c21e`, more
    solid than the names' `team_*`, which are tints for words), where all
    four shared the one purple. By night each glows in its colour, `lamps`
    glowing a bulb in whatever colour its material is (`ANCHOR_GLOW`, one
    and a half times it), and lights its island in a pale shade of it:
    three parts of white to two of the colour, its strongest channel 1
    (`pale`; the purple's light, chosen by eye, was much what this makes
    of purple). Painted once the map has been read and the pillars' lights
    are there, and again as the file changes. The bar over an anchor is
    as it was, green for yours and red for the rest. Seen from over each
    island by day and by night: red at the bottom of the map, blue on its
    left, green at its top, yellow on its right, and four tinted pools of
    light.
  - **What each bot carries** (`bots::Persona::carries`, `pockets`,
    `loadout`, `KINDS`; `Bot::loadout`). A kind has its leaning to each of
    the five things there are for a button, a fireball, a speed potion, an
    invisibility potion, a healing potion and a Bridge Builder, and how
    many kinds it carries at the least and the most: the daredevil, the
    trickster, the mastermind and the tinkerer three to five; the troll,
    the commander, the free spirit and the drifter two to four; the
    captain and the enforcer two or three; the host, the watchman, the
    defender and the lone wolf one to three; the dreamer and the sentinel
    one or two. Each life it throws which, in what order it buys them (the
    likeliest mostly first), and how many of each it keeps: three
    fireballs for one that leans hard to them and two for one that leans
    some, two healing potions for one that leans on those, one of anything
    else. `shopping` gives each kind its turn among everything else it
    buys (0.76 of its kind's leaning to its buttons for the first, 0.05
    less for each after), a second or third of a kind well after the first
    of the next, and a keeper with nobody to fetch it amethyst a healing
    potion in place of what amethyst buys. One kind a life until then.
  - **Each is used in its own time** (`Squad::intend`; `Bot::drinking`
    knows which potion is at its mouth): a healing potion whenever it has
    less than 55% of its health and nobody of another team is within 9 m,
    or backing off with nobody within 6; an invisibility potion within
    62 m of the middle of the island it raids; a speed potion setting out
    on a raid or a hunt, for home from over 30 m off when someone is on
    its island, or backing off with whoever is after it 8 to 22 m behind;
    and its fireballs as before, one every 6 s at whoever is a throw away
    with land under them.
  - **A bot sends a Bridge Builder** (`Order::build`, `Sent`, `build_on`):
    with one on a button and a bridge to lay, it walks to where the bridge
    is to begin and sends the builder off from there, and the bridge goes
    up straight, a plank or two ahead of where the builder would be, at
    the pace anyone builds at, while the bot goes about something else: no
    metres of its own, no plank in its arms. Nobody sees the builder, as
    nobody sees a person's online: the bridge grows by itself. Whoever
    runs the game takes the builder off its buttons (`Outfits::spend`;
    `Squad::unsend` if it was not there after all); its owner dying, it
    stops where it is. **And there are bridges worth one**
    (`Squad::spare_span`), which a bot lays only by builder, never with
    metres, and buys a builder for when it carries them and has the
    amethyst: from the middle to **the other of the two smaller islands**
    beside its own, for twice the sapphire; and, once the raids are on,
    **a side door**, from a smaller island its team has reached straight
    onto the island of the other team that island lies beside, about 75 m
    of water, to come at them from where no bridge of theirs lands. A
    kind that likes a raid opens the side door first, the rest the
    sapphire; not beside a person. Without these no bot would ever have
    bought one: its own bridges are built before the first amethyst
    comes, a minute in. Every way a bot finds goes over a side door as
    over any bridge, so a raid comes by whichever way is shorter.
  - **Two faults found on the way.** A bot put the same potion to its
    mouth twice: in the frame a drink was done, the potion was still on
    its button as that frame saw its kit, and it began again, 1.6 s at
    half its pace for nothing, since the bots first drank. And **a
    person's bot teammate could be given the wrong bridge** since the
    kinds came: `lay` asked for its bridge again as if it were the one of
    two bots that goes out, so one whose kind opens to a smaller island
    was sent to build that, with the metres it had bought for the middle,
    and one whose kind opens late was given none for up to two minutes. It
    lays the bridge its last thought settled on now (`Bot::planned`).
  - **Whole games, again** (two hundred, as deployed): over at 13:03 on
    average, none drawn; won by island 50, 53, 58 and 39; before 10:00
    nine deaths by punches and five by falls a game; each kind's team
    winning 20% to 31% of its games. The eight of a game together bought
    16 fireballs and threw 14, drank 2.5 speed potions, 0.7 invisibility
    potions and 16 healing potions of the 29 they bought, and sent two
    Bridge Builders; the most kinds on one of them at once was two in 87
    games, three in 88, four in 23 and five in 2: they can fill five, and
    amethyst, one a minute from each pad at first, is what mostly keeps
    them from it. `many` prints all of it (`Chronicle::carried`, `used`,
    `fullest`).
  - **Checked on 2026-11-13.** Unit-tested: the picture's tip, the words'
    place, each anchor's colour and its light, what each kind carries,
    what a bot buys of several kinds and when a builder, a builder's
    bridge laid rung by rung across a gap while the bot stays ashore, the
    potion drunk being the one the moment calls for, and, on Hajun's map
    itself, the second island and the side door, which laid rung by rung
    joins the two lands. Scripted in the desktop build, offline, by day
    and by night (the throwaway `src/arena/scratch_run.rs` again,
    removed): the arrow and the anchors, as above. On the VM, deployed
    with nobody on it for twenty minutes (Hajun's own game had ended at
    16:23), this PC's account was seated and played half a minute with
    seven of the computer's, two of their bridges across with their bends
    by then, the journal clean. APK 25 is built, its 77 asset files
    checked byte for byte, its library the stripped one and its number
    read back, and **not installed, no phone being attached** (APK 24 is
    kept beside it as `donggeurami_town-release.prev-2026-11-13b.apk`).
    **Not by hand, not on a phone, and no bot's builder or side door has
    been seen on a screen**: those are the tests' alone.
  - **What is mine**: the four colours and how pale their light is; every
    number of what a kind carries and how many it keeps; when each potion
    is drunk; that bots send builders at all, the two bridges worth one,
    and who opens a side door first; and where the compass's words stand.
- **Since 2026-11-13 there are five buttons for things to use, and you
  hold several kinds at once** (Hajun, with `new4morebuttonlayout.jpg`, a
  screenshot with four circles drawn on it: "i want to update the ui
  buttons on the minigame so that it has 4 more buttons that are hidden by
  default until either u press the ... toggle button or u buy an item that
  fills up another slot", and: "update the item shop purchase mechanic
  correspondingly so that it fills up the slots according to buy order so
  u can buy more than 1 item and hold it at once and adjust
  bridgewarsgameevent.txt file accordingly to remove any of the 'only can
  fill 1' mentions"). It is what makes room for more items to come.
  **`PROTOCOL_VERSION` 14: what a kit is on the wire has changed, so the
  server was deployed with it** (2026-11-12 15:49 UTC, the binary before
  kept on the VM as `roundtown-server.prev-2026-11-13`), **and APK 23 is
  told to update: APK 24 is the one that plays online** (below).
  - **The rules** (`shared/src/lib.rs`): a `Kit` has `slots:
    [Option<Stack>; UTILITY_SLOTS]`, five, a `Stack` being a kind and how
    many, where it had one `utility` and a `count`. Buying a kind you
    hold adds to its button (three fireballs, nine of anything else, as
    before, `most_held`); a new kind goes on the first empty button
    (`Kit::free_slot`), so they fill in the order things are bought; a
    button keeps its place until it is empty, and nothing moves up to
    fill a gap, so that what is under your thumb stays there. A potion
    whose effect still lasts keeps its own button with none left on it,
    the speed's while you run fast and the invisibility's while you are
    unseen, each by itself (`Outfits::tick`). `Outfits::drink` and
    `Use::Potion` say which potion. Dying empties every button, as it
    emptied the one. With five kinds sold and five buttons, "Slots full"
    (`words.market.taken`, "Slot full" until then) cannot come up yet: it
    is there for the sixth kind (which the dynamite is, since that
    night: above). `Kit::held`, `slot_of`, `potion` and
    `holding` are what everything else reads a kit by.
  - **Where they are** (`hud::slot_place`, `MORE_RING`, `MORE_ANGLES`):
    the second button where it was, and the four more on a second ring
    round the sword's middle, 38 shares of the short side out where the
    jump, second and bridge buttons are 20.5 out, at 0, 26.7, 53.3 and 80
    degrees up from straight left of the sword, each the others' size,
    3.5 clear of each other and of the first ring, the last ending beside
    the dots. Measured off the sketch, whose circles are 37 to 40 out at
    0, 25, 52 and 80 degrees, and evened out as the first three were. A
    test keeps every button there clear of every other.
  - **When they are seen** (`utility::fill_slots`, `slot_shown`,
    `MoreSlots`): the second button always; each of the four more while
    something is on it; and the empty ones too while the three dots are
    on (`hud::MoreButton`, which did nothing until now), which are lit
    while they are, as the bridge button is, and off again at each new
    game (`fold_away`). A button put away takes no press (`Visibility`;
    Bevy's picking and `hud::Presses` both pass over what is hidden).
    (Since 2026-11-14 there are no dots, and a button is shown only with
    something on it, the second one too: above.)
  - **Using them** (`utility::buttons`, `aim`, `drink`): each as the one
    was, by what is on it: a hold and a drag to aim a fireball or a
    builder, a press to drink, the count in its corner, a potion's
    seconds over its own button. One thing at a time: while a finger
    aims, no other button aims or drinks; with a potion at your mouth, a
    press on its button puts it down and one on another potion's does
    nothing. On desktop Q is the second button, as it was, and 1 to 5
    the five.
  - **The bots keep to one kind a life** (`shopping` in
    `shared/src/bots.rs`: nothing on any button but what it fancies, and
    none of that left), so the games they were tuned by are the games
    they play: the session that made them ran the same sixty seeded
    games before and after, number for number. (For an hour: later that
    day each fills its buttons as it is like, up to all five, above.)
  - **What is mine**: that the dots never put away a button with
    something on it (it could not be used); which of the four fills
    first, the one beside the second button and then outward (53, 27, 80
    and 0 degrees), so that two or three things lie together under the
    thumb, where a plain sweep along the arc is `MORE_ANGLES` in another
    order; that an empty button looks as the empty second button did;
    the keys; and the bots' one kind.
  - **Hajun's file** says so now: the Utilities heading has the five
    buttons, the order and the stacking, and the fireball's and the speed
    potion's lines say "its button" where they said the 2nd.
  - **Checked on 2026-11-13, scripted, in the desktop build** (a throwaway
    `src/arena/scratch_run.rs`, removed, pressing the buttons with
    touches through picking). Offline: the dots showing four empty
    outlines and putting them away; healing, two fireballs, speed,
    invisibility and a builder bought in turn, a button appearing for
    each in that order; both potions drunk off outer buttons, "43s" and
    "31s" each over its own; a fireball thrown by a quick tap and one by
    a hold and a drag from an outer button, its band on the ground; that
    button gone once empty, the others where they were, and the next
    fireball bought back on it; a fall into the void emptying all five.
    Against a server on this PC: the same purchases sold by the server,
    the buttons as its kits have them, both drinks and both fireballs in
    its log. Unit-tested: the order, the stacking, a freed button filled
    first, each potion's button outlasting its count, death, the bytes,
    the layout. **Deployed and built at Hajun's word that night** ("Deploy
    and build now"), from the working tree as it stood, which held two
    other sessions' uncommitted work as well (the bots' second round with
    the compass and the teams' colours, and the app's opening, both said
    by their sessions to be finished): nobody had been on the VM for half
    an hour; after it this PC's own account was let in and seated in a
    game there, which ran half a minute with seven of the computer's and
    a clean journal. APK 24 is built, its 77 asset files checked byte for
    byte and its library the stripped one, and **not installed, no phone
    being attached** (APK 23 is kept beside it as
    `donggeurami_town-release.prev-2026-11-13.apk`). **Not by hand and not
    on a phone**: whether the outer ring lies well under a thumb, and how
    much of the island the buttons cover on the phone's screen, is unseen.
    Committed and pushed with the other two sessions' work as one commit
    (`ad147fd`) when Hajun said "push to git".

- **Since late on the night of 2026-11-12 each of the computer's players
  is someone, a knock no longer stuns them, and no two fights with them
  go the same way; the teams are red, blue, green and yellow; and the item
  shop sells a compass** (Hajun, playing the bots of that evening, in four
  messages: "when im fighting the bot, i attack the bot, and the bot is
  like 'stunned' or stop moving so they get knocked back, and while they
  are being knocked back they are not running toward me so its so easy to
  knock them off the map. make the bot run toward u even while being
  knocked back. also add some random bell curve variation to the
  'straightness' of their bridging. straight line bridge look too ai. also
  add variation to strategy ... your ai teammate going to idle when u step
  on the generator ... seem a bit weird. just let it stay on the generator
  even while ur collecting but the item collection goes to the player ...
  there is a very easy way to kill the bots which is make them chase you
  on the bridge to the center island, and then right when u reach the
  center island, turn around and knock them off ... think of some way to
  randomize this interaction and also reverse this interaction ... so that
  there is no 'optimal' way to kill bots"; "team should be red, blue,
  green, yellow. and the name color should indicate the team. and add the
  Player compass tracker item in the bridgewarsgameevent.txt as well. also
  i want ai to jump while running/ and also randomly while attacking. more
  nonlinear/curve/slightly random not efficient movements to make it seem
  like a real player"; "it should be like a personality/playstyle, and at
  game start the bots each take a certain personality/playstyle that is a
  predefined set of probability distribution for the if ___ then do ___
  thing ... there should be more personalty than 8"; and "check if the
  enemy/team character's health bar is displaying correctly after they
  take damage"). The bots are `shared/src/bots.rs` still, the server's
  online and the device's own offline, so **online all of the bots' part
  is the server's, deployed at 15:17 UTC** (the binary before kept on the
  VM as `roundtown-server.prev-2026-11-12h`; `PROTOCOL_VERSION` 13 still
  then: the compass is a ware added after the rest and a kit's word of it
  is not sent, so APK 23 played on against it as it was). The colours, the
  compass and the health bars are the app's, **and APK 24 has them**: half
  an hour later, at 15:49 UTC, the five item buttons went out from the
  same tree as protocol 14 (their own account is beside this one; the
  15:17 binary kept on the VM as `roundtown-server.prev-2026-11-13`), and
  the APK built with them carries this round and the app's opening as
  well. From that deploy on APK 23 is told to update, so on a phone all of
  this comes with APK 24, built and not installed.
  - **Who each is** (`bots::PERSONAS`, `Persona`): sixteen ways of
    playing, called by the four letters people sort each other by. Each
    bot takes one by chance as it is seated (`Squad::seat`), two of a game
    the same as likely as not, and plays the whole game by it. Every number
    of one is odds or a leaning about 1, not a rule, and what a bot does at
    each turn is still thrown for: how much it wants to be the one that
    goes out (`drive`), its odds of each opening (`opening`), what it
    spends on first (`spends`: its sword, its shield, its second button,
    its team's generator, its team's defences, its team's swords), what it
    puts on its second button (`button`: a fireball, a speed potion, an
    invisibility potion, a healing potion, thrown again each life,
    `Bot::fancy`), how readily it goes in, how much it jumps and how widely
    it weaves (`bold`, `bouncy`, `weave`, each bot's own a little off its
    kind's, `Temper`), its leaning to each way of fighting (`fights`), how
    much it goes for throwing people off an edge and minds being thrown
    off (`shoves`, `minds`), and how it raids (`eager`, seconds sooner or
    later than 2:30; `raids`, added to a raid's score; `lingers`, times
    the 45 s; `anchors`, how likely it is there for the anchor rather than
    for whoever is home and their pad). ESTP the daredevil, the action
    items and into every fight; ESFP the troll, an invisibility potion and
    over to someone's island to lurk by its edge; ENTJ the commander,
    straight for the anchors; ENTP the trickster, odd openings, a fireball,
    in and out; ENFJ the captain, its team's swords and shields; ENFP the
    free spirit, a speed potion, all over the map, jumping; ESTJ the
    enforcer, head on and on to the anchor; ESFJ the host, traps and a pool
    for whoever comes; INTJ the mastermind, everything bought first and
    then to battle; INTP the tinkerer, its generator and no hurry to fight;
    INFJ the watchman, waiting beside where your bridge comes ashore; INFP
    the dreamer, out of everyone's way; ISTJ the sentinel, by the book;
    ISFJ the defender, the smaller islands and not the middle, its
    generator and its team's defences; ISTP the lone wolf, a sword, round
    you and off the edge with you; ISFP the drifter, a potion in hand and
    never mind the anchors. The server logs who is who as a fight begins
    ("Bridge Wars game 0: 1 Kimbap is INTP, 2 Nana is ESTP, ...; the teams
    open Middle, Late, Middle, Middle"). Nothing on the screen says who is
    which: it shows in how they play.
  - **Who keeps the base, and how a team opens.** Of a team's two bots
    the one that wants to go out more goes (`Persona::drive`; the higher
    seat until then), the other keeps the base; beside a person the bot
    keeps, as before. The one that goes out settles how its team opens
    (`Plan`, `Persona::plan`, `Squad::settle`; `Opening`): **to the
    middle** at once, as every team did until then; **straight to one of
    the two smaller islands beside its own** (75 m, four buys, for its
    sapphire, the middle left until its kind's `middle_from`, a minute for
    some and five for others); or **later**, arms first, both of the team
    buying as the one that keeps does, and no bridge until 45 to 120 s in.
    In two hundred games of bots 62% of teams opened to the middle, 17% to
    a smaller island and 21% late, and won 26%, 23% and 22% of the time.
    From 4:00 a team that has built no bridge still has one built to it.
  - **What they buy** (`shopping`): everything a bot might buy next keeps
    its place in the order they all bought in, and its kind's leaning to
    that sort of thing moves it up or down: one puts a fireball before its
    sword, another its team's defences before its generator. **The speed
    and the invisibility potions are used now**: a speed potion drunk as
    it sets out on a raid or a hunt, an invisibility potion within 62 m of
    the middle of the island it raids, to walk onto it unseen, each with
    nobody of another team within 9 m. The
    Bridge Builder is still not for bots (it is since 2026-11-13, above).
  - **A knock carries a bot for a moment and no more** (`Body::step`,
    `Squad::act`, `Skill::reel`, `Skill::grip`). Until then a bot knocked
    was carried and nothing else until it had come to rest, which is what
    Hajun saw as stunned. Its legs are its own in the air and with a knock
    on it now, as anyone's are: 0.06 to 0.36 s after a knock lands
    (`reel`, mostly about 0.2) it is going about what it was doing again,
    which in a fight is at you; in the air over the void it steers for the
    nearest ground within 8 m (`REACH_BACK`); and carried toward an edge it
    walks against what carries it. How hard it steers is from half its
    pace to all of it (`grip`), and both are thrown again at every knock:
    now it is back on its feet at once, now it goes over. On the ground it
    still walks only where a body can walk, and never off an edge by
    itself.
  - **How it fights is thrown for every few seconds** (`Stance`,
    `Squad::fight`, `take_stance`, `spot`). Until then every bot ran
    straight at whoever it fought and stood in front of them, wherever
    that was: on the last plank of a bridge, with them to one side of its
    end, where one punch put it in the void. Six ways now: **press**, up
    to them and at them; **circle**, round them as it punches; **poke**,
    in for a punch and back out of their reach to 3.6 m (`POKE_BACK`);
    **wait**, standing off 4.2 m (`HOLD_OFF`) for them to come; **ambush**,
    on the land beside where their bridge comes ashore, to one side
    (`lair`), so that whoever steps off it is thrown back over its side,
    which is Hajun's own trick turned on people and on other bots; and
    **rush**, off the bridge it is on and past them onto the land at a
    run. Which, and for how long (1.4 to 4.2 s, an ambush 5 and a rush 3),
    goes by
    where the two of them stand, on the land or out on a bridge, by how
    bold it is and by its kind's leanings: on the land it presses, circles
    or pokes; with them coming over a bridge it mostly lies in wait or
    waits; out on a bridge with them on the land ahead it stops short of
    their reach or runs for the land past them; and it is thought of again
    whenever either steps onto a bridge or off one. Having stood off
    someone on the land from out on a bridge, three times in ten it gives
    them up for 3 to 8 s (`Bot::shun`) and goes about something else.
    **Where it stands to punch** is the best of twelve places 1.45 m round
    them (`RING`): better for being near, much better if a punch from
    there would carry them off the land (`flung`, by `shoves`), worse if
    one of theirs would carry it off (by `minds`), and a little by chance.
    Its punches are not to the beat either (every 0.6 to 1.04 s, the first
    0.15 to 0.48 s after it has someone in reach), now and then thrown
    with a jump, and it faces whoever it fights whichever way it walks.
  - **It jumps and it does not walk a ruler's line.** Running somewhere,
    it jumps now and then, the more the bouncier it is (`Body::jump`, the
    game's own jump; not with a plank in its arms or a potion at its
    mouth); in a fight it jumps to no purpose now and then, and with one
    punch in ten to four in ten. Walking a way, it weaves to one side and
    the other of it (`Squad::walk_to`: up to about 14 degrees either side
    for a plain one, half that for the straightest kind and nearly twice
    for the widest weavers; a third of that on a bridge, and none close to
    where it is going). With nothing to do it wanders its island from one
    thing on it to another, the keeper mostly by its anchor, where until
    then it stood on one spot.
  - **Their bridges bend** (`nav::Span::bend`, `Squad::bends`): a sweep to
    one side, a wave and a ripple along its length (so much of the sine of
    one, two and three half turns, each nothing at both ends, so that it
    leaves and meets the land where the straight one would), each as a
    bell rings it, and all of them by how straight this bridge is to be,
    thrown as a bell rings too: most a little wavy, a few all but
    straight, a few all over the place; on a line a little to one side or
    the other of the one between the two lands' middles. Never slanting
    more than 0.45 to the side for each metre on where it leaves and
    meets the land (`MOST_SLANT`, 24 degrees), nor more than a twentieth
    longer than the straight one (`MOST_BENT`), which is the metres a bot
    buys for it. Its planks are laid rung by rung along the curve, each
    headed as the curve runs there (`Span::rung`, `heading_at`), and cut
    and drawn by `bridge` as anyone's.
  - **Your bot teammate stays on the pad with you** (`Bot::yields`,
    `Squad::wants`): while you are within 3.5 m of your team's pad it
    stands on it and takes nothing, so what the pad makes is yours; when
    you leave, it takes what its own purchases need again. Until then it
    stepped off to the anchor as you came and back on as you left.
  - **The teams are red, blue, green and yellow** (`colours.arena.
    team_red`, `team_blue`, `team_green`, `team_yellow` in
    `assets/ui.ron`, `ArenaColours::team`): red the island at the bottom
    of the map, blue the one on its left, green the one at its top and
    yellow the one on its right. Everyone else's name over their head is
    in their team's colour (`arena::spawn_tag`; a teammate's green or an
    enemy's red until then, `ally` and `enemy`, gone from the file); a
    zombie's in its own (`zombie`). The bar under a name is still green
    for your team and red for any other (`hp_ally`, `hp_enemy`), which is
    what says friend or foe at a glance: mine. A team is written by its
    colour, "Team Green" where it was "Team 3" (`arena::team_name`;
    `words.arena.team_red` and the rest), and "You are on Team Red" is
    said over "Fight!" as the game begins (`your_team`), your own name not
    being over your own head. The minimap's dots are as they were, you red
    and your teammate green, whatever your team.
  - **The Player Compass Tracker** (`src/compass.rs`; Hajun's file: "2
    amethyst (one time per life purchase - lost on death, which is a non
    slot filling item that basically shows a small white arrow around
    your character that is pointing at the direction of the nearest
    nonteammate player + name (color of team) + '123m'"): the fourth card
    of the item shop's second row, "Tracker", "Nearest enemy"
    (`Ware::Compass`, `Kit::compass`, `words.market.compass`,
    `hud::Picture::Compass`), "Owned" once you have it. With it, a white
    arrow 10 shares of the short side out from you, about two metres of
    the map, points the way whoever of another team is nearest stands,
    with their name by it in their team's colour and how far off they are
    in whole metres. Yours alone, nothing of it sent. It points at nobody
    unseen, dead, or a zombie. (In APK 24 the arrow pointed back at you,
    its picture drawn upside down, and across the screen its words lay
    over it: both put right the next day, above.) **Close by it gets out
    of the way**: the
    words are put away with them nearer than 6 m, where their own name is
    on the screen (seen in a run: "Yeppi 2m" written across two names),
    and the arrow nearer than 3 m, where it would lie on them; each comes
    back three quarters of a metre further off than it went. Online the
    server sells it as it sells anything, to someone within 5 m of the
    item shop, and says so (`Tell::Bought`); **a kit's word of it is not
    sent** (`#[serde(skip)]`), so what the room is sent is what it was,
    and this device keeps its own word of yours (`arena::Game::compass`,
    put back on your kit as the room's kits come, `follow_round`) until
    you die. The computer's players do not buy it.
  - **The health bars over heads never shrank, and do now**
    (`arena::place_tags`, `fills_of`). Hajun was right. Since 2026-11-08,
    when the name went over the bar and the two were put in a column, the
    bar's fill was looked for among the tag's own children; it is the
    bar's child, so none was ever found, and every bar stayed full
    whatever its body's health, online and offline. The numbers floating
    off a body, the health itself and the deaths were right all along,
    and so was your own bar top left.
  - **`Skill::MODERATE` has two numbers more**, `reel` (0.06, 0.36) and
    `grip` (0.5, 1.0); the constants of the fights and the bridges are at
    the top of `bots.rs` (`RING`, `POKE_BACK`, `HOLD_OFF`, `CARRY`,
    `REACH_BACK`, `MOST_SLANT`, `MOST_BENT`), and everything of a kind is
    its line of `PERSONAS`. `Squad::cast(seat, name)` makes a bot someone
    for trying one out, and `Squad::set_plan` a team's opening.
  - **Whole games, again** (`server/src/arena.rs`): the eight-game test
    asks that someone be across to the middle within a minute, a smaller
    island reached within four, a sword within two, an upgrade within
    five, and no anchor broken before 4:00; no longer that every team be
    across within two minutes, nor that no runner stand 20 s, teams
    opening late or to a smaller island as they do. `many` prints each
    kind's games, wins and deaths and each opening's, and `RT_WHO=1` who
    was who. **Over two hundred games** (as deployed): over at 13:00 on
    average, the soonest at 10:07 and the latest at 13:42, none drawn; won
    by island 46, 64, 43 and 47; before 10:00 ten deaths by punches and
    six by falls a game, where it was six and fourteen: they are thrown
    off the map less than half as often, and kill each other more; 0.3
    anchors broken by bots before 10:00; each kind's team winning 19% to
    34% of its games, an ISFJ dying 1.6 times a game and an ESTP 3.9. And
    someone standing still beside the far end of a bridge, punching
    whatever comes off it twice a second, throws 217 of 960 bots into the
    void in half a minute, where 916 of them get onto the land
    (`crossings_to_someone_waiting_at_a_bridge_s_end`, an ignored test,
    by kind).
  - **Checked on the night of 2026-11-12, scripted, in the desktop
    build** (the throwaway `src/arena/scratch_run.rs` again, removed).
    Offline: "You are on Team Red" over "Fight!"; a teammate's name red
    over a green bar, the bots of the island on the left blue over red
    bars; the compass bought for 2 amethyst, its arrow up and to the right
    at "Ssal 181m", in yellow, from the red island, turning as I flew
    toward the blue one, and gone, with everything held, at a death; two
    bots' bars shrinking under my punches while theirs took me from 100 to
    16. Against a server on this PC: the compass refused 5.002 m from the
    shop, where a seat arrives, and sold standing at it; kept through the
    room's kits coming, and gone at the death the server logged; the
    server's bots' bars shrinking as it logged each hit; my bot teammate
    standing on the pad. Unit-tested: the compass's rules and whom it
    points at, a bent span's ends, length and planks, a knocked body
    brought back by its legs, and what each kind buys first. The VM,
    deployed with nobody on it for eleven minutes, let this PC's build in.
    **Not by hand, not on a phone, and no game has been played on the VM
    with it by me** (after the 15:49 deploy this PC's account was seated
    there and played half a minute with seven of the computer's, the
    journal clean). The cost of the brain on a phone is no more measured
    than it was.
  - **What is mine, Hajun having asked for the thing and not the
    numbers**: the sixteen kinds, their names and every number of them;
    that the one that wants to go out more goes; the three openings' odds
    and that a late team arms first; the six ways of fighting and their
    odds; the reel and the grip; how bridges bend and how far; that the
    bars stay green and red; the caption over "Fight!"; where the compass
    sits in the shop, its size and where it is drawn, that it leaves out
    the unseen and the zombies, that it gets out of the way close by, and
    that bots do not buy it.
- **Since the night of 2026-11-12 the computer's players play the
  game** (Hajun: "right now the other bots dont do anything other than
  stay in their base. i need the bots to be intelligent enough. think of a
  way to make this game enjoyable. the difficulty need to moderate - not
  too hard, not too easy ... maybe one way is every 1~2 seconds the bot
  scores a short list of goals and does the highest one ... and a bonus
  for the current goal so the bot doesn't flip between two every tick ...
  maybe treat islands as graph nodes and every bridge as an edge and bots
  pathfind over the graph and when no path exist, they buy meters for the
  straight line distance and bridge it"; the plan Hajun approved is
  `~/.claude/plans/right-now-when-2-lexical-goblet.md`). They collect,
  shop, bridge to the middle and to the smaller islands, buy their team's
  upgrades, raid, defend, back off, break anchors and, at the end, hunt.
  **Online all of it is the server's** (deployed three times that night:
  at 12:02 UTC, at 12:44 with a fix, and at 13:19 as it stands, the
  binaries before kept on the VM as `roundtown-server.prev-2026-11-12e`,
  `f` and `g`; `PROTOCOL_VERSION` 13 still, nothing that is sent having
  changed, so APK 22 plays against them as it is, and **tuning them
  online is a server deploy, not an app update**). `RT_ARENA_BOTS=old`
  in the VM's `/etc/roundtown/env`, and a restart, goes back to the bots
  that kept to their islands, without a deploy (`Config::arena_old_bots`,
  `Game::old_bots`). **A game played offline has the same bots, by the
  same brain, since APK 23** ("Offline", below); an app from before has
  the old ones offline, and these online.
  - **The map the server was missing** (`shared/src/nav.rs`,
    `roundtown_net::nav`). The server has no map, which is why its bots
    kept to the disc of their islands. `Grid` is Hajun's
    `pvptownmap.glb` as a body walks it, half a metre to a square
    (`CELL`): each square's floor, whether a body fits there and can walk
    to it (`Grid::clear`: its whole footprint on ground, nothing reaching
    into it, no step higher than a step), and which piece of land it is
    (`Region`: a team's island, one of the four smaller islands, the
    middle, or the pond, `Pocket`, which is standing room nobody walks
    into). **It is baked out of the map by the game's own collision**
    (`src/arena/nav_bake.rs`, a test: flooded from the arrivals and the
    pads with `Island::floor` and `Island::blocked`, as a body is moved)
    into `shared/src/arena.nav`, 39,832 bytes, built into both binaries
    (`Grid::arena`). A team's island comes to 294 m² a body can walk, a
    smaller one 422, the middle 15,883. **After any re-export of the map:
    `RT_BAKE_NAV=1 cargo test --lib nav_bake`, which writes the file
    again, and deploy the server**; without it
    `the_grid_built_in_is_the_map_s_as_it_is` fails, saying so. **The map
    is a star**: every island reaches every other through the middle (a
    team's island to the middle 43.6 m, a smaller island to the middle
    37 m; a team's island straight to a smaller one is 75 m, never worth
    it), so "no way" always means one of eight straight spans is missing.
    `Nav` is the grid with every bridge laid on it as its rungs come
    (`Nav::hear`, from people's `Ask::Rung` and the bots' own, in index
    order as `bridge` takes them): the squares within a plank's half
    width less a body's of a plank's middle line are walked (`deck`),
    which pieces can be walked between is worked out again at each plank
    (`relabel`, `joined`, `connected`), a way is the shortest over the
    squares, straightened (`Nav::path`, A*), and `on_plank` and `ground`
    say what holds a body knocked there. `Span` is the straight bridge
    across one of the star's gaps (`Grid::span`): from half a metre
    inside the one shore to half a metre inside the other, 33 planks and
    44.6 m on an axis, which `bridge` cuts along both shores as it does
    anyone's (a test lays one on the real map, rung by rung,
    `a_span_the_computer_lays_is_a_bridge_like_any_other_on_hajuns_map`;
    the plank's measures are `nav`'s now, `PLANK_HALF_WIDTH`,
    `PLANK_SPACING`, `PLANK_TOP`, which `bridge` reads). **A plank laid
    costs next to nothing** (`Nav::lay_deck`, `joined_to`, `piece`): a
    plank only ever adds squares to walk, so it joins pieces and never
    parts them, and its own squares take the number of whatever is beside
    them (the four beside: a step cornerwise needs both squares at the
    corner) and join what they lie between. For its first hours every
    plank had the whole map gone over again, 318,096 squares, 3.3 ms on
    this laptop and a dozen times a second as a game begins; a 33-plank
    span is 26 millionths of a second now where it was 113 thousandths.
    **A way is searched for greedily** (`GREED`, 2: what is left weighed
    twice what is done, each square gone on from once): from a team's
    island round the pond to the far side of the middle in 0.09 ms where
    the shortest took 2.1, and to the island across the map in 0.35 ms
    where it took 4.8, the ways found, once straightened, a metre longer
    in 274 and 3.6 m shorter in 161 (`timing_of_the_arena_s_map`, an
    ignored test, prints them). And **a bot keeps the way it has while bridges are built**: a
    plank only adds ground, so only one with no way looks again as one is
    laid (every bot did, at every plank anyone laid). All three were
    found measuring for a phone, where the bots' frame is the game's.
  - **The brain** (`shared/src/bots.rs`, `roundtown_net::bots`, plain
    code with dice of its own, so the same seed is the same game).
    `Squad` is every bot of a game. Each has **a slow layer** (`think`,
    every 1 to 1.5 s, not all at once): what it might be doing, each with
    a score, the best taken, the one it is on getting 0.15 more
    (`Goal`: `Retreat` 0.95, `Defend` 0.88, `Hunt` 0.8, `Lay` 0.78,
    `Shop` 0.72 at home, `Raid` 0.6 to 0.79, `Collect` about 0.5,
    `Fetch` about 0.5 to 0.8, `Idle`), a goal it gets nowhere with for
    3 s, or asks the shop for three times and is not given, left out for
    a while (`barred`); and **a fast layer** every frame (`act`), which
    follows its way there, fights whoever comes into reach, lays its
    planks and keeps to the ground. `Body` is its body: walked only
    where a body can walk, so **it never walks off an edge**; carried by
    a knock as anyone is, landing on whatever ground is under it, land
    or plank, and falling where there is none, which is how one is
    beaten on a bridge, and how it beats you; knocked into the pond, it
    hops out over the rim. What a frame comes to that the game has to
    carry out is an `Order` (a punch, a strike at an anchor, a purchase,
    rungs laid, a throw, a drink), which the server does as it would for
    a person who asked (`Game::play_bot`): **nothing is free, and
    everything is lost on death**, the metres of bridge with it.
  - **How they play.** (Since later that night who keeps and who goes
    out, how a team opens, what each buys first, how each fights and how
    each raids go by who the bot is, above; what follows is how they all
    played for those hours, and is still what the kinds lean away from.)
    Each team's lower seat keeps the base (`Role::
    Keeper`) and the other goes out (`Runner`); one alone does both.
    **The opening**: the one that goes out has the pad's first 8 silver
    (the keeper wants none until the bridge is across), buys 50 m and
    lays the span to the middle at the pace anyone builds at, 5 m/s, each
    plank a rung or two ahead of its feet: across by 25 to 29 s. The
    keeper buys a Basic Sword and a shield. **Then** the one that goes
    out takes the amethyst from the pad on its side of the middle,
    bridges from the middle to one of the two smaller islands beside its
    own (by 65 to 69 s) and brings sapphire home. **What they buy, in
    order** (`shopping`): the metres its bridge is short of; with
    sapphire the Heal Pool, the generator, Stronger Swords, Better
    Shields, a trap (the first upgrade at 83 to 89 s); the Aura Sword
    with 4 amethyst and the Aura Shield with 6; the Silver Sword once
    the gold is near, and until then the basic one; a shield; a healing
    potion for the second button, or, the one that goes out, from 5:00
    and with 40 silver, a fireball; the Advanced Shield. **At a pad a
    bot takes only what its next purchases need** (`Squad::wants`,
    `Mint::pick_up_where`) and leaves the rest lying; people are served
    first, whatever their seats, and of a team's two bots whichever it
    matters most to. **Fights** (`foe`): it goes for whoever of another
    team comes within 6 m at first, growing to its full sight, 24 m,
    between 2:30 and 10:00 (`Skill::aggro_at`), or within 9 m for 3 s
    after it is hit; anyone on its island, from anywhere (`Defend`: it meets
    them on the land, it does not hold the end of a bridge); and on an
    island it raids, whoever is home. Its first punch comes 0.3 s after
    it has someone in reach, since it is drawn a moment behind on your
    screen, then every 0.8 s; it knocks as a punch on the run only when
    it really was running in (every bot punch did, until then). Whom a
    team's runner raids first is thrown differently in every game
    (`Squad::salt`). **Under
    35% of its health with someone on it, it backs off** to its pad
    until it has 60%, and drinks its healing potion once nobody is
    within 6 m; 50% once its anchor is gone and dying is for good.
    **Raids** (`Goal::Raid`): none before 2:30. Then the one that goes
    out, with a sword and 70% of its health, goes to another team's
    island for no more than 45 s, fights whoever is home and takes from
    their pad until it holds 20 silver and 7 gold; home 40 s after, or
    after dying on one. One team raids a team at a time, a team raided is left alone
    for 90 s, and **a team with a person on it gets no more than its
    share**, one raid in three while there are two other teams to go to
    (`Squad::raids`, `spared_until`, `raided`). **Anchors** are left
    alone before 4:00, struck by one bot at a time every 2 s (a person
    can strike twice a second), and the bots break no more once two are
    broken, before 8:00. From 5:30 the keeper goes along with its
    teammate. From 4:00 a team that has built no bridge has one built to
    it from the middle: nobody sits a game out behind the void. **From
    13:00 everyone hunts** whoever of another team is nearest that can
    be walked to, three minutes after the timetable breaks the anchors,
    so that the end of a game is minutes and not moments. **A
    fireball** only at someone 4 to 10.5 m off with land under them,
    never at someone out on a bridge. Not for bots yet: invisibility,
    the speed potion, the Bridge Builder (the two potions since later
    that night, above; the builder still not).
  - **A bot whose teammate is a person keeps the base** whichever its
    seat: it leaves the pad's first silver to you for 40 s, then, if
    nothing joins your island to the middle, buys the 50 m and lays the
    bridge itself (seen on the VM: begun at 40 s, across at 51 s); then
    it stands at the pad taking what its own purchases need, the rest
    piling up for you, steps off to the anchor when you come within
    3.5 m of the pad (until later that night: it stays on the pad with
    you since, taking nothing while you are there, above), defends, and
    from 2:00, with nobody of another
    team within 40 m, bridges to a smaller island and fetches its
    sapphire, and the amethyst on your side of the middle, for the
    team's upgrades. It never raids by itself.
  - **Every number of how hard they play is `Skill::MODERATE`**, in one
    place: 1 to 1.5 s between thoughts, the 0.15 for the goal it is on,
    0.3 s and 0.8 s for a punch, 2 s for an anchor, 6 m growing to 24
    between 2:30 and 10:00, 35% and 60%, raids from 2:30, anchors from
    4:00, the keeper along from 5:30, two anchors spared until 8:00, 45 s
    a raid, 40 s after, 90 s for the raided, a third for people, the
    hunt from 13:00, 50% once the anchor is gone, 6 s between fireballs.
    All mine, from the plan, but 13:00: with the hunt at 10:00, as
    planned, every game ended within half a minute of the anchors
    breaking.
  - **How the server shows them**: a bot is moved by the game and sent
    as it moves (`flush`), and once more as standing still when it stops
    (`Spot::still`), so that nobody watching carries it on the way it
    was going; its rungs go to the room as a person's do (`Event::Rung`,
    a `Plank` each), so its bridges are drawn, stood on and on the
    minimap as anyone's; what it takes from a pad is taken from the
    room. **In APK 22 a thing a bot takes does not fly to it** (it
    looked only for people, the computer's picking up nothing until
    now): it goes where it lay, or to a person standing within reach.
    `loot::follow_server` takes anyone but a zombie for the taker since,
    which APK 23 has. Nobody sees a bot carry a plank or
    drink, as nobody sees anyone else do either.
  - **Whole games are played in tests** (`server/src/arena.rs`,
    `bots_alone(seed, old)`, `play`, `chronicle`):
    `the_computer_s_players_play_whole_games_sensibly`, eight games of
    bots alone in about four seconds, keeps every team across to the
    middle within two minutes, a smaller island reached within four, a
    sword within two and an upgrade within five, nobody ever walking off
    an edge, no runner idle for 20 s, no anchor broken and nobody out
    before 4:00, every game over before 20:00, three of the four islands
    winning at least once, and the same seed the same game; two more put
    stand-in people in (`with_people`): a team of people raided no more
    than its share with its anchor left alone at first, and a person's
    bot teammate keeping the base, bridging for them and never raiding.
    `cargo test -p roundtown_server many -- --ignored --nocapture` prints
    sixty games as a table to tune by, or as many as `RT_GAMES` says,
    with who won by island, and `watch` one game's raids and deaths
    (`RT_SEED`, `RT_BRIEF=1`). **Over three hundred** (2026-11-12, as
    deployed at 13:19): over at 11:04 on average, the soonest at 8:31
    and the latest at 13:19, none drawn; won by island 90, 69, 68 and
    73; before 10:00 six deaths by punches and fourteen by falls a game
    (a punch knocks 4.5 m, and an island is 20.8 across) and 1.5 anchors
    broken by bots; every team across to the middle by 30 s, a smaller
    island reached by 69 s, a sword by 55 s and an upgrade by 88 s, in
    every one of them.
  - **Two things the first deploy had wrong, found that night and
    deployed at 12:44.** **A body knocked off an edge and carried in
    under an island or a plank as it fell hung there**, neither on the
    ground nor in the air, for the rest of the game: unseen under the
    land, never dying, its team never out. Three of forty games of bots
    went to 25:00 as a draw for it, which I had first put down to two
    teams never finishing each other; with it mended, none of three
    hundred did (`Body::step`: under the ground is in the air, and goes
    on falling; a test drops one under an island). And **a body stood a
    little under the ground it is on is put onto it**: a device's
    arrivals are at the land's own height, a quarter of a metre under an
    island's top, and offline every bot stood there for good, neither
    free nor falling. Found by playing the bots offline, where they are
    seen.
  - **Checked on 2026-11-12**: with a server on this PC and a scripted
    desktop copy watching from above (a throwaway `scratch_run`, role
    `w`, removed since), a runner's bridge growing from its island and landing flush on
    the middle's rim, on the minimap too; my bot teammate on its bridge;
    a smaller island bridged, with sapphire on its pad; swords, shields,
    potions, the Heal Pool and a trap bought; at 3:12 a raider on my
    island, blinded by the trap, knocking the watcher off the edge. And
    **on the VM after the deploy**, with this PC's own account: seated
    with seven of the computer's, three bridges across at 28 s, my bot
    teammate's begun at 40 s and across at 51 s, bridges to two of the
    smaller islands by 96 s, the journal clean, and 1.1 to 1.6 KB/s out
    to a player in a game; and again after each of the two deploys
    after, a minute of a game each. Not by hand, and not on a phone.
  - **Offline, the same bots** (APK 23; `arena::drive_bots`, `Brains`,
    `Brained`). A game played offline has a map and a brain of its own
    (`Nav::arena`, `Squad`, with `Skill::MODERATE`, the server's own),
    and each of the computer's seven is **a body put where the brain has
    it each frame, in nobody's way** (`lobby::spawn_bot_at`: no `Intent`
    and no `Collider`, so `move_bodies` neither walks it nor bumps you
    into it): you walk through them, as you do online, where until now
    offline they bumped. It keeps a `Shove` and a `PlayerJump`, which is
    what your punch, a zombie's and a blast knock (`Shove::knock`): the
    knock is handed to the brain (`Squad::shoved`), which carries the
    body by it, off an edge if it must, and `reap` finds it fallen as it
    does anyone. What a frame of a bot comes to is carried out as the
    server carries it out: its punch knocks whoever it lands on and
    lands (`Landed`, so the damage, the number and the death are
    `deal_damage`'s, as yours are), an anchor is struck (the file's
    `arena.damage` and a sword's, `anchor_hurt`), a purchase made at the
    stand out of what it holds (`Brains::purses`, as yours is
    `Resources`), its rungs laid as someone else's
    (`bridge::Bridges::lay_remote`, so its bridges are drawn, stood on
    and on the minimap), a fireball thrown, a potion drunk. Every bridge
    is told to the brains' map as it is fixed, yours and a Bridge
    Builder's among them (`Bridges::laid`, by `Bridges::version`). It
    takes what its purchases need of what lies in its reach after you
    have had yours that frame, each thing flying to it; dead, it loses
    what it held, and stands again as the brain's too. **Laying a
    bridge, it carries a plank** (`Carrying`, `bridge::carry_for_bots`,
    `BotPlank`), which online nobody is seen to. `hunt` is the zombies'
    mind alone now, and they go for the computer's players as for you
    (its `Intent` is optional for that). The bots walk by the brain's
    numbers (a run of 10 m/s, a builder's 5), and are hurt and knocked
    by the file's (`assets/ui.ron`'s `arena` and `melee`).
  - **"Your anchor is under attack!"** (`arena::warn_of_the_anchor`,
    `words.arena.anchor_attacked` in `assets/ui.ron`; APK 23, online and
    offline): said in the line the timetable's are said in whenever your
    team's anchor has lost health and still stands, no oftener than
    every 8 s (`ANCHOR_WARN_EVERY`). Nothing said it until now, and
    until now only a person ever hit one. Broken, it has its headline,
    as before. Mine, with the plan.
  - **`loot::follow_server` flies a thing to whoever took it, a bot
    too** (APK 23), anyone but a zombie being looked for.
  - **Checked offline on 2026-11-12, scripted, in the desktop build**
    (`RT_SERVER=off`, the throwaway again, with a role that watches from
    above and punches whoever comes into reach): three bridges across in
    half a minute, the planks in their arms; my bot teammate's at 40 to
    51 s; smaller islands reached within 75 s; swords, shields, potions
    and the team's upgrades bought; at 3:00 a raider on my island, its
    name and bar in red, taking 11 through its shield from a punch of
    mine and thrown up by it,
    my teammate coming home over our bridge to see it off, and the
    raider killed and standing again on its own island with nothing,
    back at its pad for a sword; a raider's three strikes of 22 on
    another team's anchor; "Your anchor is under attack!" across the screen as
    mine lost health; from `RT_ARENA_FROM=890`, the twelve zombies and
    the bots at each other, and seven bodies knocked into the void in
    twenty seconds, which with the anchors gone ended the game. No
    error in any log. **Not by hand, and not on a phone**: a phone has
    never run the brain, and the cost of it there is worked out, above,
    not measured. APK 23 is built, its 77 asset files checked byte for
    byte, and not installed, none being attached; APK 22 is kept beside
    it as `donggeurami_town-release.prev-2026-11-12d.apk`.
  - **What is mine, Hajun having said only "moderate"**: every number
    above; that the lower seat keeps and the other goes out; the order
    they buy in; that they take only what they need at a pad; that the
    raids are steered (one at a time on a team, a rest, people no more
    than their share, anchors slow and capped early); that a person's
    bot teammate bridges for them after 40 s and never raids; that a bot
    throws no fireball at someone on a bridge; that bot teams fight each
    other as well as you; that offline you walk through them; and the
    line about your anchor, its words and its 8 s.
- **Since the evening of 2026-11-12 every game online is 2v2v2v2, people
  teamed up with each other first and the computer in every seat left**
  (Hajun: "right now, when 2 real players queue for the bridge wars game,
  it becomes a 1v1. i want to change it so that this game is always
  2v2v2v2, with bots filling the remaining slots, and the 2 player become
  same team. and if its 4 players, then those 2 players and 2 players are
  same team too, and so on, so that real players will be teamed up with
  each other first, and bots on remaining teams"; server deployed, the
  binary before kept on the VM as `roundtown-server.prev-2026-11-12d`;
  `PROTOCOL_VERSION` 13 still, nothing that is sent having changed, so APK
  22 plays it as it is).
  - **Seats are taken a team at a time** (`SEAT_ORDER` in
    `server/src/arena.rs`, 0, 4, 2, 6, 1, 5, 3, 7): both seats of the first
    island, both of the one across the map from it, then both of each of
    the two at its sides. Two people are a team; four are two teams on
    opposite islands; the odd one out has the computer for a teammate.
    Pairs go by the order people join in: mine, there being no parties.
    Ten seconds after the last to join, the computer takes **every** seat
    left, however many people there are (for half a day two to four
    people began with no bots, an island each).
  - **A seat left before the fight begins is anyone's again**
    (`Game::leave`, `vacate`): in the waiting room, the next person to
    come or the computer takes it; left during the countdown, the
    computer has it as the fight begins, on its island from the start
    (`seat_bot(.., landed)`). Until then it stayed its leaver's, empty,
    and a team began short. Once the fight is on, a seat left stays
    empty: mine.
  - **The server's fight is a plain function of the game now**
    (`Game::step`; nothing a player sees has changed by it): where
    everyone is is kept in the `Game` itself (`at`, `yaw`: a person's
    copied in each frame from what their device last said, `sync`; a
    bot's the game's own, to the last of a metre where a `Spot` is whole
    centimetres), and whatever a step does that takes the world, an
    entity for the room or a word to a device, it only says (`Event`:
    `Made`, `Taken`, `Thrown`, `Burst`, `Knocked`, `Land`, `Zombie`),
    for `flush` to do after. So a whole 25-minute game of the
    computer's players is played in a test, in a few hundredths of a
    second (`a_whole_game_of_the_computer_s_players_plays_itself_out`),
    which is what the bots to come are tuned by. Each game throws its
    own seeded dice.
  - **Checked on 2026-11-12 with a server on this PC and scripted desktop
    copies** (a throwaway `src/arena/scratch_run.rs`): two people in
    seats 0 and 4, on one island 4 m apart, with six of the computer's
    players; one of two leaving the waiting room, and the computer in
    their seat; what the generators make picked up, a fireball and a
    sword bought, a bot's punches knocking a person off its island, the
    fall, standing again with nothing held; a fireball thrown at a zombie
    bursting on it for 20; the twelve zombies risen, on their own team,
    at `RT_ARENA_FROM=895`; the game skipped to its end, and a second
    with nobody left over from the first. The VM, deployed with nobody
    on it, let this PC's build in. Not on a phone.
  - **This was the first part of a larger job, the computer's players
    learning to play** (collect, shop, bridge, raid, defend), which came
    that night (above). For those hours the computer's players kept to
    their islands, as before.
- **Later on 2026-11-12: dying loses everything you hold, what is aimed
  is seen over the void, and a sword is held on its edge and cut
  downward** (Hajun: "for the bridge pvp minigame, make it so death make u
  lose all ur silver, gold, sapphire, amethyst, bridge meters. also make
  the hit indicator for fireball appear even when ur on a bridge (the hit
  indicator should appear even on void area) and shootable on bridge. also
  instead of removing the code for the sword animation now (cuz i might
  wanna change it back after trying this new way), make a new sword
  animation, this time instead of holding the sword flat, hold it narrow
  and slash downward(?) and use that animation instead. try to choose the
  holding angle/slashing animation look good enough even when its like
  this way and look nice enough in a top down view. then rebuild apk and
  push to git"; `PROTOCOL_VERSION` 13 still, nothing that is sent having
  changed; the server deployed for the first of the three, the binary
  before kept on the VM as `roundtown-server.prev-2026-11-12c`; APK 21).
  - **Dying loses everything you hold**: your silver, gold, sapphire and
    amethyst with the metres of bridge, the sword and what is on the
    second button; the shield and what your team has bought stay, as
    before. Online it is the server's (`Game::die` in
    `server/src/arena.rs` empties the seat's purse, and the room is sent
    it with the rest, `Purses`), so an app from before, APK 20, loses
    them online too; offline it is this device's (`arena::lose_on_death`,
    `lost_on_death`), as the metres are online as well. Until then what
    you held was kept through a death, which was my call, Hajun's file
    saying only "all items are lost upon death" of the item shop. They
    are lost, not dropped: nobody else gets them. Checked in the desktop
    build, scripted, with a sword, a shield, a fireball and 25 m bought
    and then walked off the island: offline, [172, 80, 60, 60] to nothing;
    against a server on this PC, [122, 200, 200, 200] to nothing as the
    server logged the fall, the shield kept both times.
  - **The band of what is aimed has a dark rim round it** (`range::rim`,
    `RIM`, 0.07 m just outside its edge, in `colours.arena.aim_rim` of
    `assets/ui.ron`, black at a half; `utility::spawn_band`): the
    fireball's and the builder's. The band was drawn over the void all
    along, at your feet's height wherever you stand, but it is white,
    and by day so is the sky, nearly: measured in a screenshot of the
    desktop build, the sky (140, 209, 247), the band's fill over it (163,
    213, 243) and its outline (212, 224, 233). Off the edge of an island
    it was all but gone, and from a bridge nearly all of it lies over the
    void. With the rim it is seen over anything; over land it is a thin
    shadow round the white. **Throwing from a bridge was never refused,
    and I could not make it fail**: in the same scripted run, standing on
    a bridge 13 m out over the void, three fireballs were thrown, by a
    drag with building on, by a drag with it off and by Q, and each flew
    and burst; and the VM's log has Hajun's three of that morning bought
    and gone from the button within half a minute. If it still will not
    throw from a bridge on the phone, it is something this did not find:
    ask what was pressed, and for the log. The punch's own range has no
    rim, nobody having asked: over the void it is as faint as the band
    was.
  - **A sword is held on its edge and cut downward** (`arena.sword_swing:
    Chop` in `assets/ui.ron`, `ui::SwordSwing`; `chop_offset`,
    `chop_blade`, `chop_angle` in `src/lib.rs`; `gear::edge_on`, `Cut`,
    `Cutting`). **The first way is all still there, and `sword_swing:
    Sweep` is it** (`sword_offset`, `blade_way`, `gear::pointing`,
    `Trail`): the one word in the file goes back, on desktop as the file
    is saved, on a phone at its next build. The blade moves in one plane
    with its edge leading, as a sword is cut with: an upright plane
    through the way you face, leant over to the right by `CHOP_LEAN`,
    0.62 rad, 36 degrees. Upright altogether, the camera over you would
    see the blade and its cut edge on, a hair; flat is what Hajun asked
    to leave; leant so, it sees a slim blade, about half its breadth, and
    the cut as a crescent. At the ready the blade was out in front and up,
    0.5 rad over level in that plane, which from above points a little out
    to the right (straighter since that afternoon, below); a press raises the arm (`CHOP_ARM_FROM`, 2.35 rad
    forward of hanging, where level is 1.7) with the blade up and back
    over the right shoulder (`CHOP_FROM`, 2.0 rad over level), and brings
    both down through straight ahead to low on the left (`CHOP_TO`, -0.5;
    `CHOP_ARM_TO`, 1.15, and across, `CHOP_ARM_OUT` to `CHOP_ARM_IN`) in
    the time a punch goes out, eased, and back to the ready in the time
    it comes back. From over you: the sword flicks out to your right and
    back, whips over to the front, and ends a little left of straight
    ahead, inside the range drawn there. It is drawn thicker through its
    flat than the other way (`gear::CHOP_SCALE`, 3.0 where the flat
    sword's is 1.7), that being what the camera sees most of. **Its trail
    is the ribbon the blade itself has cut** (`Cutting::follow`,
    `gear::ribbon`): from a third of the way up the blade to its tip
    (`BLADE`, each model's own length, which a test reads off the
    models), a sample to every 5 degrees the blade has turned whatever
    the frame rate (`CUT_STEP`), the mesh built again each frame of the
    cut and faded after as the other trail is; fuller than that one along
    its length (`CUT_RAMP`), the camera seeing it from nearer its edge. A
    press mid-cut begins the next, as with a punch. What a punch lands
    on, and when, is as it was, and so is that nobody else sees your
    swing: someone else's sword is at the ready, the new way. All the
    angles are mine. Seen in the desktop build, scripted, each of the
    three swords: from the game's camera in bursts of twelve shots
    through a cut, and close from the front, the side and behind, at the
    ready, cutting, and walking and cutting; the first numbers (a lean of
    0.55, a thickness of 2.6 and the other trail's ramp) left the trail
    too faint from above, and were raised.
  - **At the ready the sword is held straighter than it cuts** (that
    afternoon, Hajun, of the hold above, playing APK 21: "i think the
    sword holding is too tilted now (both forward tilt and side tilt).
    make it more straight"; APK 22, no server change): 30 degrees forward
    of straight up and 10 over to the right, where it was 61 and 36 (the
    blade 0.5 rad over level in the plane of the cut). My reading of
    "straight" is upright, as "hold it narrow and slash downward" was of
    a sword held up and brought down; the 30 and the 10 are mine, and
    **are the file's to change**: `arena.sword_tilt_forward` and
    `sword_tilt_side` in `assets/ui.ron`, degrees from straight up, read
    as the file is saved on desktop (`crate::Guard`, `Guard::of`;
    `Arena::check` refuses anything outside 0 to 90). Straight up
    altogether the camera over you sees the sword end on, a dot: at 30
    and 10 it sees a blade about half its length, pointing nearly
    straight ahead, 0.7 to 0.9 m of it. The cut is as it was, in its own
    plane (`CHOP_LEAN`, `cut_angle`, `cut_blade_at`): the sword goes from
    the hold to the cut as it is raised, and on the way back both its
    angle and its plane's lean come back to the hold's (`chop_angle`).
    The arm at the ready is where it was. Seen in the desktop build,
    scripted, each sword at the ready from above, from the side, the
    front and behind, and through a cut and back; unit-tested with a hold
    of its own, not the file's. Not by hand or on a phone: APK 22 is
    built, its assets checked byte for byte, and not installed (APK 21 is
    kept as `donggeurami_town-release.prev-2026-11-12c.apk`).
  - **Unit-tested**: the server's purse at a death, and the device's;
    the rim round the band; the cut's poses on the model, its blade's
    path, that the flat stays square to the cut, the ribbon at 3 and at
    30 frames to a cut, and the blades' lengths against the models. The
    VM, deployed while nobody was on it, let this PC's build in. **Not
    by hand, and not on a phone: none was attached, so APK 21 is built,
    its assets checked byte for byte, and not installed** (APK 20 is kept
    beside it as `donggeurami_town-release.prev-2026-11-12b.apk`). The Adreno has
    not drawn the rim or the ribbon (both blended, as the band and the
    other trail are).
- **Since 2026-11-12 Bridge Wars has two shops, and everything in Hajun's
  file is in the game** (Hajun: "i updated the bridgewarsgameevent.txt.
  update the generator rates, then add all the itemshop/upgrade shop items,
  make them functional, and test each one to see if they work functioanlly
  properly and visually pleasingly. you may organize shoplayout in your own
  way, but keep it easy for people to be able to understand right away
  without too much text ... you may also need to make the sword animation
  for the swords rather than using punch swing animation for it";
  `PROTOCOL_VERSION` 13, the server deployed with it, the binary before
  kept on the VM as `roundtown-server.prev-2026-11-12b`; APK 20). The file
  was saved twice more while this was being built, last at 02:58 that
  night, and the game is as that one has it: **read the file again before
  changing a number**, Hajun edits it as they go. The rules are all in
  `roundtown_net::bridge_wars`, as the generators' are: `Outfits`, what
  everyone has and what each team has bought, beside the `Mint`, the
  server's online and this device's own in a game played offline. What
  there is, and where:
  - **The rates**: an amethyst every 60 s, 40 from 4:00 and 25 from 8:00
    (`interval`; 40, 30 and 20 until then), so at 60, 120, 180 and 240 s,
    every 40 s to 480, and every 25 after.
  - **What everyone has** (`roundtown_net::Kit`: the sword and the shield
    by tier, what is on the second button and how many, until when a
    potion lasts, and whether a trap has them blinded; `Upgrades`, a
    team's: its generator's step, stronger swords, better shields' step,
    the heal pool, the traps waiting). Online both are on the game's own
    entity beside the `Purses` (`Kits`, `Upgraded`), sent to the room as
    they change, so every device knows what everyone carries; a purchase
    is `Ask::Buy(Ware)`, answered `Tell::Bought(Ware)`, and using what is
    on the button `Ask::Use(Use)`.
  - **The item shop, at the red stand** (`Shop::Items`; the prices and the
    numbers are the file's): Bridge, 25 m for 4 silver; the Basic, the
    Silver and the Aura Sword, 10 silver, 7 gold and 4 amethyst, which add
    5, 10 and 20 to a punch's 12 (`SWORD_DAMAGE`); the Basic, the Advanced
    and the Aura Shield, 24 silver, 12 gold and 6 amethyst, which keep 6%,
    14% and 22% of every blow off (`SHIELD_SHARE`); and five things for
    the second button, one kind at a time ("you can only buy if the 2nd ui
    button is either empty or its filled by the same item u already
    have"): Fireball, 40 silver, three on the button at the most ("cap at
    3 stack"); Bridge Builder, 1 amethyst; Speed Potion, 1 amethyst;
    Invisibility Potion, 2 amethyst; Healing Potion, 3 gold. Only a better
    sword or shield than the one you have is sold. **Dying loses the
    sword, what is on the second button, a potion still lasting and the
    metres of bridge; the shield stays** ("sword is lost on death, shield
    carries on", `Outfits::die`, `arena::lose_on_death`); and, since
    later that day, everything you hold of the four things as well
    (above).
  - **The upgrade shop, at the blue stand** (`Shop::Upgrades`, your whole
    team's, in sapphire): the generator, 2, 4, 6 and 8, half as fast again,
    twice, amethyst too, and three times as fast (`FORGE_SPEED`, "150%",
    "200%", "starts spawning amethyst too", "300%": the team's own silver
    and gold; the amethyst comes on the team's own pad); Stronger Swords,
    4, once, 5 more on every sword of the team; Better Shields, 2, 3, 4
    and 6, 6% more kept off each time; Heal Pool, 1, 5 health every 2.5 s
    to whoever of the team stands on its island; Blindness Trap, 1, 2 and
    4 for up to three waiting.
  - **The windows** (`src/market.rs`): standing in the outline at a stand
    opens that stand's shop, the red-roofed one's (`RoofTiles_Red`) the
    item shop and the blue one's (the material `bluemarket`) the upgrade
    shop, as the map has them (`market::Shops`, `shop_zones`); the server
    sells within 5 m of where they stood when it was built (`ARENA_SHOPS`,
    `ARENA_UPGRADE_SHOPS`, `at_a_shop`; a test keeps the two together). A
    window is a card to each thing: its picture, drawn in code
    (`hud::Picture`) and tinted as the thing is, a name of a word or two,
    one short line ("+5 dmg", "-14% dmg", "+40% 45s", "Blinds 8s", each
    number filled in from the rules), a pip for each step of an upgrade
    that goes by steps, and its price, the picture of what it is paid in
    and how many; or, in the price's place, "Owned", "Max" or "Slot full".
    What cannot be had is blocked out; a tap on a card buys it at once. The
    item shop is three rows, the bridge and the swords, the shields, and
    the five things for the button; the upgrade shop one row of five taller
    cards. The window is put up again as what you hold or have changes. It
    is still not a dialog, and covers the middle of the screen, you with
    it.
  - **The second button** (`src/utility.rs`, `hud::utility_slot`, the empty
    slot up and to the left of the sword until then): the picture of what
    is on it, how many in its corner, and the seconds a potion still lasts
    over it ("45s"). **What is aimed**, the fireball and the builder, is
    held and dragged ("rather than a push button, its a hold & drag / letgo
    to shoot or cancel, and shows a long narrow rectangular hit range
    indicator similar to Brock in brawlstars"): once the finger is 4% of
    the short side from where it went down a band lies on the ground from
    you that way (`range::band`, in `colours.arena.aim_outline` and
    `aim_fill`, and since later that day with a dark rim round it,
    `aim_rim`, which is what shows it over the void, above), as long as a
    fireball flies and a metre wide, or as wide
    as a bridge; let go and it goes that way, and you turn to face it;
    dragged back to where it went down the band is gone, and letting go
    there cancels. A quick tap that never dragged, under 0.3 s, uses it
    the way you face (mine, as Brawl Stars fires at a tap), and so does Q
    on desktop. Up on the screen is north, the camera being fixed over you.
    Half a second has to pass between two. **What is drunk**, all three
    potions (the healing one too in the file's last version: "after
    drinking time 1.6s, speed reduct, same as speed potion"), is a press,
    or Q: 1.6 s (`DRINK_SECS`) with the time left written in front of you
    in tenths (`utility::show_drink`), at half your pace, with no punch,
    your right hand at your mouth and your sword put away
    (`crate::Drinking`); a press again, any hit that takes health off you,
    or dying stops it with nothing drunk; run out, the potion is drunk
    (`arena::Used::Potion`). The punch's range is put away meanwhile, as
    while you build.
  - **The fireball** (`src/fire.rs`, `bridge_wars::Flight`): a red glowing
    ball thrown from in front of your chest, flying level and straight at
    22 m/s for 12 m (`FIREBALL_SPEED`, `FIREBALL_RANGE`), through whatever
    stands on the map; where it meets anyone of another team, or at the end
    of its range, it bursts, and everyone of another team within 3 m takes
    20, less what their shield keeps off, and is knocked away from where
    it burst as a punch knocks. The blast is a ball of fire that swells to
    as far as it reaches and fades in a third of a second. Online each is
    an entity the server sends as it is thrown (`Fireball`), which every
    device flies on by itself, and takes away as it bursts. The range was
    20 m for its first hours, which bursts off the screen: the camera over
    you shows 11 m up and down.
  - **The Bridge Builder** (`src/bridge/builder_bot.rs`; "it spawns a bot
    character that walks in bridge mode toward that direction, creating a
    bridge in a straight line for 100m or until it reaches a island/land
    and then the bot pops/disappears"): a body in your own look carrying a
    plank, sent off from where you stand at the pace anyone builds at, 5
    m/s. It walks straight over the ground, through whatever stands there,
    to where the ground ends; there its bridge begins, a chain like any of
    yours (`Chain`, its root on the very edge and cut along it, heading the
    way it walked or as near that as a first plank may slant, 45 degrees),
    laid a plank or two ahead of its feet; ground within a plank's reach
    ahead, land or anyone's bridge, the bridge lands on it as yours do
    (`land_on`); out of its own 100 m (`BUILDER_METRES`, not yours) it
    ends where it is; and at the end of its bridge the builder shrinks to
    nothing. Off the side of a bridge it forks. Walking 170 m without the
    ground ending it gives up, and it is sent only from the ground. It is
    only there to be seen (`lobby::puppet`, `crate::ForShow`: in nobody's
    way, no punch lands on it) and is this device's alone, as the plank in
    your arms is; online the server takes the builder off your button and
    its bridge goes up a rung at a time (`Ask::Rung`), so everyone else
    sees the bridge grow and not the builder.
  - **The potions**: speed, 45 s at 1.4 times your pace (measured in the
    desktop build: 10.0 m/s before, 5.0 drinking, 14.0 after); the healing
    one, a quarter of your health back as it is drunk, with a green "+25"
    floating off you (half of it, and "+50", since 2026-11-13, as the
    file has it since); and invisibility, 30 s in which nobody of another
    team draws you, your name or your bar (`island::show_where_i_am`,
    `arena::place_tags`), and no bot goes for you (`arena::hunt`, the
    server's `think`), while you and your own team see you see-through
    (`gear::ghost`). **Being hit, or hitting, ends it** ("being hit or
    attacking reveals u", `Outfits::reveal`): a blow that takes health off
    you, or a punch of yours that lands on a body or an anchor, which is
    all of a punch the server hears of. While a potion lasts with none
    left, the button keeps its picture, faint, and its time, and takes
    nothing else; then it is empty.
  - **Swords and shields are seen on everyone** (`src/gear.rs`, by
    `arena::Wears`, which every body in a game carries from the game's
    kits). The sword is Hajun's model for its tier, `basicsword.glb`,
    `silver_sword.glb` and `aura_sword.glb` (theirs of 2026-11-12, each a
    blade up its own +y from a grip at its origin, 0.9 to 1.1 m; a test
    keeps them in the APK), a child of the body put in its right hand every
    frame once the limbs are posed (the rig's `hand.R`, followed up to the
    body), its flat side up. **It is drawn bigger than modelled**, 1.7
    times and twice across (`gear::SWORD_SCALE`): from the camera 15 m over
    you a metre of thin blade was a hair. (Since later that day a sword
    is held on its edge and cut downward instead, above, and what follows
    is the first way, `arena.sword_swing: Sweep`, kept to go back to.)
    **With a sword the punch is a
    slash** (`sword_offset`, `blade_way` in `src/lib.rs`): the right arm
    holds it at the ready, a little forward of hanging, the blade out in
    front, and at a press it is raised to level and swept about the body's
    upright from the right of straight ahead across to the left in the
    time a punch goes out, and back to the ready in the time it comes
    back; and the slash leaves a trail (`gear::Trail`), the arc the blade
    has swept, in the sword's colour, growing behind it and gone a sixth of
    a second after. What a punch lands on, and when, is as it was: the
    range drawn round you, at the press. A slash, like a punch, is this
    device's alone: someone else's sword is seen at the ready and their
    swing not at all. The shield is a ball of force round the body ("a
    very transparent (slightly semitransparent) spherical forcefield"),
    1.05 m out from its middle, in its tier's colour and as see-through as
    that colour's alpha (`colours.arena.shield_*`).
  - **The team's upgrades, seen**: the heal pool's 5 floats off whoever it
    heals as a green "+5" (`arena::spawn_healed`; online any jump up in
    health of 4 or more is taken for a heal). **A trap** springs on an
    enemy stepping onto your island: it is used up, they are blinded for 8
    s wherever they go (`BLIND_SECS`; the file's first version had it until
    they had been off the island 5 s), your team is told "An enemy walked
    into your trap!" and they "A trap has blinded you!", in the line the
    timetable's are said in. **Blinded** (`gear::blindfold`; "they can only
    see a very small circle around their character and the rest of their
    screen is black except their ui menu/buttons/minimap"), the screen is
    black all but a circle 2.6 m round you (`BLIND_SIGHT`): one UI node
    with a round hole kept over you, under the buttons, the minimap and
    the rest of the screen's own, over the world and the names and numbers
    written over it. A blinded bot sees no further than that.
  - **Who is in a game online, until the evening of 2026-11-12** (every
    game is 2v2v2v2 since, above) (Hajun, the same night: "when ur the only
    real person in the queue for the bridge wars, it can be same as now
    where 7 bots join u. but when there is atleast one other real person in
    that same queue lobby, no bots will join, and after the wait time of no
    more new joins, it will start the game as a 1v1 (2 side island being
    empty), or 1v1v1 (with one island being empty) or 1v1v1v1, (or 2v2v2v2
    with each bot teammate for missing slots if >=5 players)"): people are
    seated in the order 0, 2, 1, 3, 4, 6, 5, 7 (`SEAT_ORDER` in
    `server/src/arena.rs`), so that two are on opposite islands and the
    first four each on their own; ten seconds after the last to join, one
    person alone, or five or more, have the seats left filled with bots as
    before, and two to four begin as they are, the seats nobody took kept
    empty (a `Member` with no id, out from the start), so that a zombie's
    seat is still past the players' eight. Offline it is you and seven
    bots, as it was.
  - **`assets/ui.ron`** has the words (`words.market`: the two titles,
    "Owned", "Max", "Slot full", and each thing's name and line, in which
    a `{number}` comes out of the rules; `words.arena.blinded` and
    `trap_sprung`) and the colours (`colours.arena.sword_*`, `shield_*`,
    `fireball`, `potion_*`, `heal_pool`, `trap`, `pip`, `pip_off`,
    `aim_outline`, `aim_fill`, `aim_rim`, `drink`, `blind`). No number of
    the rules is in it; `arena.sword_swing` says which way a sword is
    swung.
  - **For trying it out**: `RT_ARENA_PURSE=<count>` begins a fight with
    that many of each thing in every person's purse, on the server for its
    games and on a desktop build for one played offline, as
    `RT_ARENA_FROM` begins it late.
  - **What is mine, the file not saying**: (what you hold of the four
    things was kept through a death, until Hajun had it lost, later that
    day;) Stronger Swords and Better Shields do nothing for a bare hand or
    someone with no shield; the generator's third step keeps the second's
    pace, and its amethyst is one a minute, two on the pad at the most;
    nine of anything but a fireball on the button; the fireball's speed,
    range, size and blast, that it harms neither its thrower's team nor an
    anchor, and that it flies through walls (the server has no map); the
    quick tap; that a trap springs only on stepping onto the island, and
    not again on the same enemy until they have stepped off and on, and
    never on a zombie; that your own team sees you while you are unseen;
    the builder's pace, and that only its owner sees it; and how big the
    swords are drawn, how they are held and swung, and the trail.
  - **Checked on 2026-11-12, scripted, in the desktop build** (a throwaway
    `src/arena/scratch_run.rs`, removed, with touches written in for the
    cards and the second button). Offline, each thing: both windows, every
    card bought by a tap, the prices off the counts, a fourth fireball and
    a second kind refused, the cards blocked out with nothing to pay; each
    sword from above and close, at the ready, slashing and walking, and
    each shield; a punch with the Aura Sword and Stronger Swords taking 37
    off a bot, and a bot's 12 taking 9.4 through the Aura Shield; the
    fireball aimed, dragged back and cancelled, thrown, flying, bursting,
    20 off a bot, and thrown by a quick tap; a builder bridging the 44 m
    from the island to the town and landing, a second forking off that
    bridge and reaching a sapphire island 83 m on, and standing on its
    planks; a drink stopped by a second press and by a hit, the paces
    above, a bot six metres from someone unseen leaving them alone, a punch
    ending it, the healing taking 28 to 53; the pool's 5 every 2.5 s; a bot
    stepping into a trap, and being blinded for 8 s on an enemy's island.
    Online, a server on this PC and two copies: two people begun as a 1v1
    with no bots; everything bought from the server at both stands; the
    second copy seeing the first's sword and shield, walking into its trap
    and being blinded, not seeing it while it was unseen and seeing it
    again as it punched (22 off), a fireball flying in and taking 20, and
    the builder's bridge growing on its minimap. Unit-tested: the prices,
    the tiers, the caps, the potions, the traps and the pools; the
    builder's bridge to a far shore, its hundred metres with none, a slant
    and a fork; the poses on the model; the band, the trail, the drag's
    aim. The VM, deployed, let this PC's build in. **Not by hand, and not
    on a phone: none was attached, so APK 20 is built, its assets checked
    byte for byte, and not installed.** The Adreno has not drawn the
    swords, the blended bubble and ghost, the trail, the bands or the
    blindfold.
- **Since the night of 2026-11-11 Bridge Wars has generators, a shop and a
  timetable** (Hajun: "i want to now make the generators functional in the
  pvpmap, using the 4 item models in the asset folder as the items being
  generated - they should drop in semi random location on top of the stone
  pad on each island ... the items should bob up and down and rotate
  slowly, and when the user can collect it by walking in it (short magnet)
  ... make it so that the red shop if the user goes inside the activation,
  it opens a closable popup shop menu, and right now just have 'bridge' as
  the option, which should cost 4 silver for 50m of bridge ... read the
  bridgewarsgameevent.txt for events and generator rates. it should show a
  small alert in around middle of the upper half of screen that tells the
  event ... these generators/items in map need to be server synced so
  other players see the same number of items on map at the same times and
  other player taking the item from the generators make it so you cant
  take it too ... if there is any other info ur not sure try to think for
  yourself what would be ideal, cuz this is a game similar to minecraft
  bedwars"; `PROTOCOL_VERSION` 11, the server deployed with it, the binary
  before kept on the VM as `roundtown-server.prev-2026-11-11b`; APK 18).
  Hajun's file, `bridgewarsgameevent.txt` at the root of the repo, is the
  timetable and the rates. Everything it says is in
  `roundtown_net::bridge_wars` (`shared/src/lib.rs`), which the server
  plays by online and the device by itself offline, and none of it in
  `assets/ui.ron`, which has the colours and the words. What there is, and
  where:
  - **The generators** (`bridge_wars::Mint`, `GENERATORS`, `generator`,
    `interval`, `cap`): sixteen, a silver one and a gold one on the pad at
    the back of each team's island (`ARENA_PADS`, the map's `gen1` to
    `gen4`), a sapphire one on each of the four smaller islands on the
    diagonals (`SAPPHIRE_PADS`, `sapphiregen` to `sapphiregen4`) and four
    amethyst ones in the middle of the map, 35.8 m out on the axes
    (`AMETHYST_PADS`, `amethystgen` to `.003`), as the file has them.
    Silver one every 2 s, half a second more or less, 48 to a pad at the
    most; gold every 7.5 s, a second more or less, 6; sapphire every 30 s,
    23 from 2:00 and 12 from 6:00, 4; amethyst every 40 s, 30 from 4:00 and
    20 from 8:00, 2 (every 60, 40 and 25 since 2026-11-12, above). A
    generator's time comes round whether or not it is
    full: holding its most, it makes nothing then, and its count starts
    over (mine: the file says only "max held on generator"; until late
    that night a full one's count stood still, and its next thing was a
    whole wait on from when one was taken, which a count written over it
    could not have shown). The sapphire and the amethyst ones keep time
    to the second, by the time alone (`bridge_wars::steady`, `next_at`: a
    sapphire at 30, 60, 90 and 120 s, then every 23 s to 350, then at 372,
    the third tier having come at 360 and cut the wait in hand to its own
    12 s, and every 12 s after; an amethyst every 40 s to 240, every 30 to
    480, every 20 after), which is what lets every device write the same
    count over them with nothing sent ("The sapphire and the amethyst
    generators say", below). Each thing comes up at the best of six
    places tried at random within 0.7 m of the pad's middle each way, the
    one furthest from everything lying there (`PAD_SPREAD`: my reading of
    "semi random location"). The pads' places are constants, the server
    having no map; a test reads them off the map, by what Hajun calls the
    objects (`the_generators_pads_are_where_the_map_has_them`).
  - **Picking up** (`MAGNET`, 1.5 m across the ground from your middle to
    the thing's: Hajun's "short magnet"; `Mint::pick_up`), **wherever the
    thing is, dropping or lying, once it is within your reach** (since
    2026-11-12; Hajun: "the delay is a bit too much for the delay between
    the item dropping from generator and the user collecting - user
    should be able to collect in the air too maybe when they jump"): no
    further over your feet than `REACH_OVER`, 2.14 m, half a metre over
    your head, as Minecraft picks a thing up half a block over a
    player's, nor under them than `REACH_UNDER`, 1.5 m, so that a jump
    across a pad still takes what lies on it and the waiting room and a
    fall past the island do not. A thing is where its drop has it
    (`Piece::height`, `drop_height`: the rules' now, with `HOVER`,
    `DROP_FROM` and `DROP_SECS`, as whoever runs the generators goes by
    them): standing on its pad it is yours 0.45 s after it is made, still
    dropping, and jumping as it appears, 0.33 s after, 2 m up; the
    numbers but Minecraft's half metre are mine. For the evening before
    it could not be had until it had landed and lain 0.3 s, 0.85 s after
    it was made (`PICK_UP_AFTER`, gone), which Hajun found too long.
    Measured in the desktop build, scripted, standing pinned on a pad
    (0.449 to 0.451 s) and jumping as each thing was made (0.324 to
    0.329 s, the feet 2.05 m up); not online, where it is the server's
    and unit-tested, nor by hand. Online the server gives a thing to
    whoever of the people in the game comes within reach of it, where
    they last said they stood, the lower seat first within a tick, and it
    is gone for everyone that tick (`server/src/arena.rs`, `mint`,
    `Game::gather`); offline the game does the same for you
    (`arena::gather`). The computer's players picked up nothing (mine: a
    teammate that emptied the pad would be no help, and it bought
    nothing) until 2026-11-12, since when they take what their purchases
    need, people first ("the computer's players play the game", above).
    Online each thing is an entity the server sends to the game's room
    (`Loot`: what it is and where it lies), and what everyone holds is one
    component on the game's own entity (`Purses`, four counts a seat).
  - **What you see** (`src/loot.rs`): Hajun's four models,
    `silver_item.glb`, `gold_item.glb`, `sapphire_item.glb` and
    `amethyst_item.glb`, cut stones of three, four, five and six sides,
    0.7 m across, each hung 0.42 m over its pad's top, bobbing 0.1 m
    either way every 2.4 s and turning once in seven, out of step with
    the one beside it, and taken, flying to whoever took it in 0.16 s,
    shrinking (online, to the nearest person to it, the server not saying
    who). **Made, it drops onto its pad** (since late that night; Hajun:
    "the minerals spawning animation is like a growing big animation. i
    want it to be a drop from a certain height animation. so that even
    when player is standing on top of the generator, u can see the mineral
    for a bit before the player pick it up"): it appears 5 m over where it
    will hang (`loot::DROP_FROM`: over the head of anyone standing on the
    pad, and well under the camera, so drawn nearly half as big again
    as it lies), and falls there in 0.55 s, faster as it comes
    (`loot::height`), so that someone standing on a pad sees each thing
    come down before it flies to them from where it is, as it comes
    within their reach ("Picking up", above; for its first evening it
    had to land and lie 0.3 s first); until then
    it came up from nothing to its size in a fifth of a second where it
    lay, and was theirs the frame after. Online the drop begins as the
    server's entity for it arrives, and who has it is the server's. Seen
    in a burst of shots a tenth of a second apart, offline and against a
    server on this PC, standing on the pad. Only those within
    48 m of the camera are moved: a full map has 240. At night they glow
    in their own colours, 0.9 times, as far as the lamps are on: a
    sapphire is nearly black by moonlight, and the smaller islands have
    no lamps. All my numbers.
  - **What you hold is counted top left**, under your health bar, where
    the log, the ingot, the gem and the emerald were: silver, gold,
    sapphire and amethyst, each pictured as its model is seen from above,
    a stone of three, four, five or six sides with its corners cut off as
    the model's are and the outline of its table cut through
    (`hud::item_picture`; Hajun: "change the icon to match the model
    shape"), tinted its own colour (`colours.arena.item_silver`,
    `item_gold`, `item_sapphire`, `item_amethyst` in `assets/ui.ron`).
    The metres of bridge are no longer there ("remove the log icon - the
    bridge meter only shows as in map text in front of the bridge like
    now"): they are written in front of you as you build, as before, in
    the shop, and, since 2026-11-12, inside the bridge button, under its
    plank, small and white, building or not (`hud::bridge_button`,
    `BridgeCount`, written by `show_resources`; 2.8 shares of the short
    side, in `colours.arena.bridge_count`; Hajun: "above the ui bridge
    button add a small white text "123m"", and a few minutes later,
    "instead of above the button, put it inside the button, below the
    icon": the plank is moved up 13% of the button to make room). Seen in
    the desktop build with 123 m written in.
  - **The shop** (`src/market.rs`): standing in the white outline on the
    ground in front of the red-roofed stand on an island, anyone's (the
    map's `activation` material, at a stand roofed in `RoofTiles_Red`;
    read off the map once it has spawned, `market::shop_zones`), opens a
    small window over the middle of the screen: "Shop", a card with a
    plank, "Bridge +25m" (50 until late that night; Hajun: "make it so 4
    silver gets you 25m of bridge", `bridge_wars::BRIDGE_A_BUY`) and its
    price, a silver and "4", the line "You have 0m of bridge", and a cross
    on its corner; all of it white on grey since late that night ("The
    panels are grey", below: its title was gold, its cross red and a
    price you could not pay red). It is not a dialog:
    you can walk, and the fight goes on round it. A tap on the card buys
    one at once, as often as you can pay: offline here, online asked of
    the server (`Ask::Buy`), which takes the 4 silver if you hold it, are
    standing, and are within 5 m of a shop where you last said you stood
    (`ARENA_SHOPS`, `SHOP_SLACK`; a test keeps those where the map's red
    stands are), and says so (`Tell::Bought`), on which the 25 m are
    yours. Without 4 silver the card is blocked out, its price with it.
    The cross shuts it, and so does a press anywhere else on the screen,
    which is not taken, so that touching the stick walks you away, and
    Escape; shut by hand it stays shut until you have stepped out of the
    outline and in again; stepping out of it, dying and the game's end
    shut it too. The blue stands' outlines were nothing until 2026-11-12,
    when they became the upgrade shop and this one the item shop, with a
    card for everything it sells ("two shops", above); its words were
    `words.arena.shop`, `shop_bridge` and `shop_have`, and are
    `words.market` since.
  - **Everyone starts with no bridge** (`arena.bridge_metres` in
    `assets/ui.ron`, 0 where it was 500 since 2026-11-10): my call, Hajun
    not having said, as a game of Bedwars begins with no blocks. Four
    silver, eight seconds of a pad, is 25 m, and an island is 44 m from
    the middle of the map: two buys. It is that one number to change
    back. Online
    the metres are still the device's own; it is the silver the server
    keeps.
  - **The timetable** (`bridge_wars::TIMETABLE`, `PLAY_SECS`): a game
    lasts 25 minutes where it lasted 10. At 2:00 and 6:00 the sapphire
    generators quicken, at 4:00 and 8:00 the amethyst ones; at 10:00
    every anchor left breaks, so that whoever dies after is out; at
    15:00, sudden death, the zombies rise; and at 25:00 it is a draw
    ("Time's up!" over "The game is a draw."). Each is said as it happens
    in one line across the screen a quarter of the way down, small (3.6
    shares of the short side), white, up five seconds (`arena::alert`;
    `words.arena.tier_up`, `anchors_broken` and `sudden_death`, and
    `colours.arena.alert`: "Sapphire generators have been upgraded to Lvl
    II."), over the shop's window if that is up; and since late that
    night a line about an item has that item's picture right after its
    name, the one its count has top left, in its colour (`arena::Line`,
    which cuts the file's line after `{item}`, and `AlertPicture`; Hajun:
    "add the item icon after that item name in that game event text"),
    coming and going with the words. The four headlines four
    anchors breaking at once would put on top of each other are left out
    for the one line.
  - **The sapphire and the amethyst generators say their level and count
    down to their next thing** (`arena::write_over_generators`, since
    late that night; Hajun: "for the sapphire and the amethyst
    generators, put a white text above it that shows the generator level
    like Amethyst Generator Lvl 2, and below it, put 'Next Amethyst in
    24sec...' that counts down. text should be small enough so it doesnt
    cover the amethysts or teh generator but large enough to be
    readable"): two lines of white words, 3.0 shares of the short side
    (`GENERATOR_TEXT`, mine: between the names over heads, 2.4, and the
    alert's line, 3.6; about 2 mm letters on the phone), one ending 0.55
    m north of each of the eight pads and one beginning 0.55 m south of
    it (`GENERATOR_GAP`), over and under it on the screen under the
    camera fixed over you, centred on it, shown while the pad is on the
    screen. The words are `words.arena.generator` ("{item} Generator Lvl
    {level}", the level as 1, 2 or 3 where the alert writes I, II, III,
    as Hajun wrote each) and `generator_next` ("Next {item} in
    {seconds}sec..."), the colour `colours.arena.generator`. **Nothing
    is sent for them**: the level is the timetable's and the count is
    `bridge_wars::next_at`, both by the time, which online is the
    server's to the second (`ArenaRound::left`, rounded up, so the count
    reads 1 through the last second and the whole wait the moment a
    thing comes, never 0). It counts on while the generator is full, at
    the end of which nothing comes and it starts over: mine. Seen in the
    desktop build, scripted, at a sapphire pad and an
    amethyst pad, and across 2:00, "Lvl 1" and "2sec" to "Lvl 2" and
    "23sec" as the sapphire dropped and the line came up with its
    picture.
  - **The camera stands at one height in a game and nothing zooms it**
    (`OrbitCamera::top_down`, `locked`; since late that night, Hajun:
    "make the camera in that mode's default zoom to be zoom in a bit more
    so the borders of each players island just fits around perfectly on
    the camera on my mobile phone, and lock that zoom so u cannot zoom in
    or out"): `arena.camera_height` in `assets/ui.ron`, 15.5 m over you
    (as `car.overhead` is, over your chest: 16.7 m over your feet), where
    an island, 20.8 m across, takes up 96% of the screen's height with
    you in its middle, its edge just inside the top and the bottom. The
    camera sees 66 degrees top to bottom on every screen (`CAMERA_FOV`),
    so that is so on any screen wider than it is tall, the phone on its
    side among them, with the sides to spare; worked out, measured in a
    1600 by 900 window (the island from 18 to 882 of 900 pixels), and
    not seen on the phone. A pinch and the wheel do nothing there
    (`zoom_by_ratio`); the file saved again moves the camera to its new
    number (`drive::retune`, `OrbitCamera::relock`); out of the game the
    camera is as you had it. Until then it stood the car's 24 m up,
    where the island was 64% of the height, and zoomed between 7 and 60.
  - **The panels are grey** (since late that night; Hajun: "the shop
    panel is too colorful. make it all shades of white/black/gray - the
    text, the red close button, the background, etc - no blue tint. same
    for the timer background and minimap background, and also the
    roundtownmap setting panel. i dont want any blue tint. and try to
    reuse the exact grey color if possible"). The grey is your name's
    plate's and the buttons', `#666666`: the glass behind the time
    (`colours.panel`, `panel_lit`, House Builder's too) and the minimap's
    background (`arena.map_back`) are the plate's very colour, alpha and
    all (`#66666673`), so the three across the top of the screen are one;
    the shop's window and the settings' are that grey as solid as each
    was (`shop.window` `#666666db`, House Builder's shop with it, and
    `settings.window` `#666666d1`); the cross on a window's corner is
    that grey, solid, where it was red (`close`, every window's); and
    the shadow under every word is black where it was a dark blue
    (`text_shadow`). They were a navy (`#0a1a2e`, `#102033`) and a
    bluish grey (`#5c5e66`). In the shop the title is white where it was
    gold, a price you cannot pay is greyed where it was red, and the
    silver's picture is a plain light grey (`item_silver`, which had a
    little violet in it). **The minimap's picture is clear round the
    land** now, its background the node's alone: it was filled with
    `map_back` as well, so a see-through one was laid on twice and came
    out more solid than the file said. A grey as see-through as the
    plate's takes the colour of what is behind it, the sky's blue by day
    over the void: the plate always has, and the time and the minimap do
    now as it does; each is one value in the file, its alpha how solid.
    **Left as they were, being neither blue nor asked about**: the
    settings' chosen tab, gold, and its Delete account, red; the
    headlines ("Fight!"), gold; the items' own colours beside their
    counts. Seen in the desktop build, scripted: the board and the
    minimap over the sky and over the town's paving, the shop's window,
    and the settings' window in the town.
  - **`PROTOCOL_VERSION` 12, the server deployed with it** (the binary
    before kept on the VM as `roundtown-server.prev-2026-11-11c`): a
    bridge bought is 25 m where an app from before would count 50. The
    drop's wait and the generators' counts are the server's online. This
    PC's build was let in by the VM after. APK 19 is built, its assets
    checked byte for byte, and **not installed: no phone was attached**;
    none of this has been seen on the phone, the camera's fit least of
    all.
  - **On 2026-11-12, after APK 19** (Hajun: "dont rebuild apk yet"): a
    thing is picked up as it drops, the metres of bridge are written
    inside the bridge button, and the minimap marks the sapphire and
    amethyst generators (each under its own heading above). The server
    was deployed with the new reach (the binary before kept on the VM as
    `roundtown-server.prev-2026-11-12`); `PROTOCOL_VERSION` is still 12,
    nothing sent having changed, so APK 19 plays online and has the new
    pick-up there, the server deciding it, and keeps the old 0.85 s wait
    in a game played offline, and has neither the number in the button
    nor the marks, until an APK is built again. Not committed either.
    Hajun's `bridgewarsgameevent.txt` had by then been changed by hand
    (amethyst every 60, 50 and 40 s where the game has 40, 30 and 20; an
    item shop and an upgrade shop begun): nothing of that was in the game,
    nobody having asked, until Hajun did, later that night ("two shops",
    above).
  - **The zombies are all mine but that they come**: the file says
    "Sudden death: zombies spawn all over the map", and no more. Twelve
    (`ZOMBIES`): one on each team's island, 4 m to the blue stand's side
    of its middle, one on each smaller island, 4 m from its pad, and four
    on the path round the pond (`zombie_spot`). Each is the body everyone
    has with its skin and its hair green (`arena::paint_zombies`; the look
    `zombie`), called "Zombie", on a team of its own (`ZOMBIE_TEAM`), so
    that every player and every bot is its to go for, and it theirs: with
    everyone's 100 of health and 12 a punch, a punch every 0.8 s. It
    comes on at 6 m/s where everyone runs at 10, never runs away, takes
    three tenths of a punch's knock (`ZOMBIE_KNOCK`: knocked as anyone
    is, the server's bots had every island's zombie over the edge within
    three seconds of its rising), and 12 s after it is put down another
    stands where it rose. Online they are the server's, seated past the
    players' eight seats (`arena_team` gives a seat from 8 up the
    zombies' team; `Game::zombie`, `seat_zombie`), each kept to a disc as
    the server's bots are (`zombie_home`: its island, or the middle of
    the map out to 74 m, where it walks through the lamps, the planters
    and the pond's rim, the server having no map); offline this device's
    (`arena::raise_zombie`, `Zombie`), bodies like the bots', which walk
    the map as it is. A zombie's punch knocks a player as any punch
    thrown standing does: four of them had me off my island in a scripted
    run. With every anchor gone by then, that is out.
  - **For trying it out**: `RT_ARENA_FROM=<seconds>` begins a fight that
    far into its timetable, with everything before then already done: on
    the server for its games, and on a desktop build for a game played
    offline. 595 is five seconds before the anchors break, 895 before the
    zombies.
  - **Checked on 2026-11-11, scripted, in the desktop build.** Offline:
    7 silver and a gold picked up off the pad 13 s in, the counts top
    left; the shop opened in the outline, two taps on its card taking 8
    silver for 100 m, a tap elsewhere shutting it, stepping out and in
    opening it again, "100m" in front of you building; the line at 2:00;
    the anchors gone at 10:00 under one line; twelve zombies at 15:00,
    one of them punched to death and back up. Online with a server on
    this PC: every thing the server had was on the screen, to the count,
    at every look; a purchase taken by the server; and with two copies in
    one game, both stepping onto one pad with ten things on it at the
    same moment, the server gave all ten to one and none to the other,
    both screens showed the pad empty at once, and each copy's counts
    were the server's; a punch of yours takes 12 off a server's zombie,
    and its punches knock you. The VM, deployed, let the new build in.
    **Not by hand, and not on a phone: none was attached, so APK 18 is
    built and not installed.** The item models, their glow, the shop's
    window and the alert's line have not been seen on the Adreno.
- **Since the night of 2026-11-08 Bridge Wars is the server's online, eight
  to a game in four teams of two, with respawn anchors and bridges you
  build** (Hajun: "the bridge battle minigame doesnt allow queing with
  other players i think even tho we join the game at the same time. fix
  that. also make the purple rotated cube thingy on each island the respawn
  anchor (similar to bed in bedwars) ... make it 8 player rather than 4
  player, so its 2v2v2v2 ... make it so u cannot hurt your own teammate ...
  display the ai bot/other players name above the healthbar, and make the
  healthbar and the nametag not distracting ... some indication wether they
  are teammate or enemy ... add a 'autobridge' feature"; `PROTOCOL_VERSION`
  10, the server deployed with it, the binary before kept on the VM as
  `roundtown-server.prev-2026-11-08b`). What changed, and where:
  - **Online a game is a room of the server's** (`server/src/arena.rs`,
    `Room::Arena`, as House Builder's games are): choosing Bridge Wars asks
    `Ask::Arena`, the server seats you in the game filling now
    (`Tell::Seated`), puts you in the waiting room at your seat's spot
    (`roundtown_net::bridge_wars::waiting_spot`, the room's floor, 150.5 m
    up since the map of 2026-11-11 and 100.5 until then, as a shared
    constant, which a test checks the map agrees with), fills
    the seats left with its own bots after 10 s (since 2026-11-12 only for
    one person alone or five or more, "Who is in a game online", above),
    counts down, sends everyone
    to their islands, and runs the clock; the device follows its
    `ArenaRound` and `Seats` (`arena::follow_round`): the stage and the
    time, everyone's health in whole points, each team's anchor and who is
    out. Your moves go to the server in the arena now (`net::tell_where_i_am`
    no longer leaves it out), and its arrivals move you (`net::hear`).
    **Health is the server's online**: a punch that lands sends `Ask::Punch`
    as before, and the server takes 12 off whoever it landed on, if they are
    not on your team and within 6.5 m of where you each last said you stood,
    knocks a person through `Tell::Knocked` and a bot itself; a fall into
    the void is `Ask::Respawn`, which the server answers with a death. **A
    body punched is past hurting for half a second on the server too**
    (`HURT_TIME`, `Game::wound`, and an anchor the same, `anchor_hurt`;
    since 2026-11-09, Hajun: the bots "seem to get knocked back too much
    by me maybe cuz i can hit them multiple times before they process it
    networkly": for a night the server took every punch that landed, as
    many as the presses came, where the device had kept to two knocks a
    second since 2026-11-07, "The punch knocks whoever it lands on back"). The
    rules both sides play by are `roundtown_net::bridge_wars` (health 100,
    damage 12, anchor 120, hurt 0.5 s, respawn 4 s, regen 4 a second after
    5 s, run 10, sight 24, a bot's punch every 0.8 s, flee under 20%, fall
    30 m, the end's 6 s); `assets/ui.ron`'s `arena` has the same numbers
    for offline, with `anchor_health` and `bridge_metres` added (and, for
    the number written in front of you as you build, `bridge_low_at` and
    `bridge_red_at`), and rules
    offline alone. **A game ends over two ticks** (`close`, then `end`, in
    `server/src/arena.rs` and `server/src/builder.rs` alike, since
    2026-11-09): its bots, planks or pieces and board are despawned first,
    while everyone in it is still in its room to be told so, and everyone
    is moved back to the town the tick after. Done in one tick, as for a
    night, the devices were never told: replicon 0.44 writes a despawn
    only to a device the entity is not hidden from by then, and a loss of
    sight only for an entity still there (`collect_despawns`), and the
    seven bots of the game before stood on the islands through the next
    game, bodies and health bars, with a teammate's green dot on the
    minimap; House Builder's bots had stood on its plots the same way
    since 2026-10-08, out of sight in the town. Checked with a scripted
    second game: no body left. **The server's bots**
    (`server/src/arena.rs`, `Mind`, `think`) have no map: each keeps to the
    disc of its island (`ARENA_ISLAND_RADIUS` 10.4 m), runs at whoever of
    another team comes within 24 m on or beside it, punches in reach,
    runs away under 20%, strolls when alone, is knocked up and across as a
    body is (`KNOCK_BACK`, `KNOCK_UP`, the drags) and falls off the edge if
    knocked over it; it walks through the island's desk and slab (its
    market stands, since the map of 2026-11-11), and never crosses a
    bridge. (Those were the server's bots until 2026-11-12; `think` moves
    only its zombies since, and the bots it would be gone back to with
    `RT_ARENA_BOTS=old`.) Offline the game is this device's own as
    before, with seven bots. Seen on this PC with a local server (`RT_PORT=5443`,
    `RT_ALLOW_SKIP=1`): seated, the bots in, the fight, a bridge to the west
    island, its two bots knocking you to 16 and off the edge, your punches
    taking 12 off them and their health coming back. **And with two copies
    on this PC and a local server, scripted (2026-11-10, Hajun: "check
    again if the enemy / enemy ai bots health bar decreases on attack and
    they can actually die from damage not just falling, and also check if
    other players can stand on top of your bridge / see u bridging")**: the
    second player, seated and sent to the island at -X, bridged to the
    town, walked across it and onto the first's bridge, and stood on it at
    the planks' height; the first punched them from 100 to 4 by twelves,
    the same on both devices, the server logged the kill, and they stood
    again on their island; the punches knocked them 24 m along the bridge
    and never off it; the server's bots at the island a bridge led to
    killed the player arriving on it ("seat 2 killed seat 0"), by punches,
    not a fall; and offline a bot put beside you went from 100 to 4 and
    died of the punches while taking you to 76. Others see your planks as
    they are fixed, not the plank in your arms (`Carrying` is this
    device's alone) nor the tip. Not yet on a phone with two people.
  - **Eight seats, two to a team, a team to an island** (`ARENA_SEATS`,
    `ARENA_TEAMS`, `roundtown_net::arena_team`, seat modulo four: seats 0 and
    4 share island 0; `arrival` puts the second of each team 4 m from the
    first). Every body in the game carries `Team` (`src/lib.rs`): a punch
    never lands on a teammate (`start_punch` passes them by for whoever is
    behind), a bot never goes for one (`arena::hunt`, the server's `think`),
    and the server refuses a hit on one.
  - **The purple pillar on each island is its team's respawn anchor**
    (`roundtown_net::ARENA_ANCHORS`, read by hand off the `spawn` material's
    pillars, which a test checks, `ANCHOR_RADIUS` 0.85): a bar over it
    (`arena::AnchorBar`, your team's in `colours.arena.anchor_mine`, the
    rest in `anchor_enemy`), 120 of health, ten punches (240 and twenty for
    a night, until Hajun asked for "1/2 of now", 2026-11-09). A punch of
    yours lands on an enemy anchor in its range as on a body that wide
    (`arena::hit_anchors`, off a new `Swung` message `start_punch` writes
    for every punch), never your own; broken, the pillar is hidden and
    taken out of what is stood on (`arena::break_anchors`, `Anchors`,
    `Islands::reshape` without its triangles, `is_pillar`; stood up again
    as the game ends), with a headline. **Dying with your anchor standing,
    you come back in 4 s as before; dying with it broken, you are out for
    good** (`Game::die`, checked at the moment of death, as Hajun asked):
    the body stays hidden and a panel comes up in the middle of the screen
    (`arena::Panel::Out`: "You're out!", "Your respawn anchor is gone.",
    Play again and Exit). The last team with anyone in play wins, or nobody
    when the time runs out: a headline across the middle of the screen
    ("Victory!", "Defeat" or "Time's up!", with who won under it,
    `arena::announce_result`, `RESULT_SECS` 4 s), and 6 s after the end
    (`bridge_wars::RESULTS_SECS`) everyone is back in the town. For a night
    the end was a panel (`Panel::End`, up 15 s, "Back to the town in" and
    the two buttons), which Hajun took for a stray popup (2026-11-09: "idk
    why there is a times up popup that appears near the end of the bridge
    game and it appears on the left side of the screen. remove that"). Play
    again is another game (offline a new one at once; online `Ask::Arena`
    again), Exit the town (`Ask::Leave` online), both on the Out panel
    alone. The words are `words.arena`'s.
  - **Everyone else's name is over their health bar** (`arena::TagOf`,
    `spawn_tag`; `tags` shows no names in the arena): small (2.4 shares of
    the short side) and faint, a teammate's in `colours.arena.ally` over a
    bar in `hp_ally` (green), an enemy's in `enemy` and `hp_enemy` (red),
    2.2 m over the feet, not scaled by distance under the fixed camera.
    (Since late on 2026-11-12 the name is in its team's colour, red, blue,
    green or yellow, and the bar alone says friend or foe; and the bar,
    which until then never shrank with its body's health, does: above.)
    Its shadow, like every text's, falls under the letters by 0.12 of
    their size since 2026-11-10 (`hud::shade`, `SHADOW_DROP`; 3 logical
    pixels whatever the size until then, which under these small names on
    a phone, whose logical pixels are large, was about a third of the
    letters' height: Hajun, "the shadow of the nametag is too far below
    the text").
  - **The bridges are built as you walk** (`src/bridge.rs`): the plank over
    the sword (`hud::BridgeButton`, the slot there until then; B on desktop)
    toggles building (`bridge::Bridging`, the button lit while on). On, you
    carry a plank (`Carrying` in `src/lib.rs`: both arms held out a little
    under level, `CARRY_REACH` 1.45 rad, with a board 1.1 by 0.5 m lying
    across them, `HeldPlank`, a child of your body), walk at half your pace
    (`bridge::CARRY_SLOW`, in `read_player_input`), throw no punch
    (`start_punch`) and show no hit range (`range::follow`; the three
    since 2026-11-09, Hajun: "when bride mode is toggled, decrease player
    speed to 1/2 and also remove the hit indicator and also dont allow
    punching"), and every frame before you move (`bridge::lay`, between
    `start_punch` and `move_bodies`) the ground `AHEAD` of your step is
    looked at: over the void, a plank is laid under it, as Hajun's own
    notes had it. A bridge is a chain of rungs (a point and a heading
    each), a plank between every two: 2.726 m wide and 0.25 m thick,
    Hajun's `bridge` object's measure from their export of 2026-11-08, its
    top at 0.24 m (the islands' 0.25 less a seam, so that the overlap at
    the root hides under the island and the join is the edge itself; and
    up to 7 mm under that by chain, `top_of`, the same on every device, so
    that two chains crossing never flicker through each other), each rung
    half a plank's width on from the last (`SPACING`, 1.363 m: the boards
    of a dock, which follow a curve closely; a whole width for a night,
    Hajun's `s >= 2w`). **The bridge is a ribbon laid under where you
    walk** (since 2026-11-09, Hajun, of the night's version: "when i am
    like on the edge of the bridge or when i turn too much it looks weird
    and choppy. think of a smart way to make briding feel good and look
    good no matter what"): the chain you are building has a tip plank not
    fixed yet, from the last fixed rung to `AHEAD` (1.05 m) past you,
    re-aimed at you every frame and drawn again every frame it moves, so
    that whichever way you step over the void there is a plank under you;
    each time you are a plank's length past the last fixed rung, a rung is
    fixed on the line toward you, so the fixed planks trace the path you
    walked; each joint is mitred (`corners_of`: the corners across the
    bisector of the two planks meeting there, `1 / cos(turn / 2)` wider,
    so the planks share their edges and neither gap nor overlap), turned
    no more than 35 degrees a rung (`MAX_TURN`: a plank with a mitre at
    both ends folds, its inner edge gone, once `2 * HALF_WIDTH * tan(turn /
    2)` passes `SPACING`; 35 degrees leaves a tenth; the bridge turns
    round in about 2.3 m, and walking a tighter circle you walk off the
    tip's side), and a mitre that would reach past a short plank's end is
    drawn in instead. Leaving the ground, the first rung lies on its very
    edge headed straight out from it (the edge found by bisection, the way
    out by sampling the ground round it); walking off a plank's side,
    anyone's, a fork begins flush against that side (a chain of nothing
    but its root is forgotten once you are `FORGET_BEHIND`, 1.55 m, back
    behind it, further in than a chain is ever begun, so brushing an edge
    builds nothing; at half a metre, for a day, a chain was begun and
    forgotten every other frame as you came up to an edge, its tip
    flickering and a root sent to the server each time); walking the far
    end of your own unfinished chain grows it;
    and where you step onto land, or onto another bridge, past where the
    chain began, it ends on that edge, headed at where you stepped onto
    it, with a rung there if the edge is half a plank's length or more
    from the last (`MIN_LAND`: nearer, the chain ends on its last rung, as
    a plank shorter than its own mitre would fold). A corner over the void
    at the root or at the end is pushed out onto the shore (`onto_edge`,
    `shore_ahead`, within `ONTO_EDGE` 2.7 m), so that the bridge meets the
    edge exactly however it curves; one over land stays, hidden under the
    land, which is higher: for a night the corners were pulled onto the
    edge both ways, and pulled in past a short last plank's start they
    folded it (a test caught a top face turned down). Turning building
    off, or dying, fixes the tip where it is if it is a plank's worth.
    **Every plank costs its length in metres of bridge** (Hajun: "players
    will buy the bridge by meter count ... for now make each person start
    with 100m of briding so show '100m' as the number next to the log"):
    `arena.bridge_metres` in `assets/ui.ron` to start, 500 (since
    2026-11-10, Hajun: "increase the default bridge meter to 500m instead
    of 100"; 100 until then; and none since 2026-11-11, when the shop
    began to sell it, above),
    `arena::Resources::bridge_metres`, shown by the log top left until
    2026-11-11 and taken
    as each rung is fixed; with less than a plank's length left no chain
    begins and no rung is fixed, and the tip, never shorter than `AHEAD`,
    runs out under you. The planks are ground: their shape is laid over
    the island's (`Island::lay_over`, `Islands::lay_over`, `overlay`: a
    second shape every question of the island is asked of too, so that
    50,000 triangles are never read again for a plank; laid again once
    the tip has moved 0.2 m or turned 0.05 rad, `OVERLAY_MOVE`,
    `OVERLAY_TURN`, as the minimap is drawn again, `Bridges::version`),
    and you, the bots (offline) and everyone stand on them and fall off
    their sides. Online every rung goes to the server as it is fixed
    (`Ask::Rung`), and comes back to everyone as a `Plank`
    (`bridge::hear_planks`, out of order allowed), yours left out; the tip
    is yours alone. Each chain is one mesh coloured by vertex
    (`colours.arena.plank` on top, every other plank a shade lighter,
    `plank_side` under and at the sides), the ends open but for an
    unfinished tip's cap. Nothing is taken up again; the bridges go with
    the game. Hajun's `bridge` object was taken out of `pvptownmap.blend`
    (saved without a `.blend1` of mine) and the map exported again with
    Apply Modifiers (62 objects, 50,130 triangles, no image: the plank
    texture went with the bridges). Seen in the desktop build, scripted
    (2026-11-09): a bridge off the island's edge wobbling and turning
    sharply with the walk, smooth all the way, the feet on it at 0.236 the
    whole way, and one toward the west island running out with the metres
    at 100 m and dropping you; unit-tested that a chain turning the most it
    may every rung never folds; not yet by hand or on a phone.
    **Since 2026-11-10 a chain's root and its end are cut along the edge
    they meet, at whatever slant** (Hajun, with two screenshots: "unwanted
    gaps and bridge formation when the tilt angle is a bit too large,
    especially when branching off of an existing bridge, and also sometime
    when approaching the island at a weird angle (not tangent) then it
    seemed to not connect properly to the island too"): the first plank
    heads where you walk, up to 45 degrees off straight out of the edge
    (`MAX_SLANT`; until then it stood square to the edge and bent a plank
    later, a stub and a bend off the side of a bridge), its root corners on
    the edge of the ground it leaves, the island's or a plank's
    (`corners_of`, `to_edge`: a corner over the void pushed back onto it,
    as before; one over another bridge's plank pulled in to that plank's
    edge, which a fork's first plank is made long enough for,
    `first_length`, nearly three metres at 45 degrees; one over land left,
    hidden under it); a fork's root is square to the parent plank's own
    nearest edge (`Chain::edge_normal`; the 16 points round it were out by
    up to 11 degrees, and diagonal in the corner between two bridges), an
    island's to the chord of a ring round it (`outward`). Landing, a chain
    comes in no more than 50 degrees off straight into the shore
    (`MAX_END_SLANT`, `land_on`: turning toward the shore a rung at a time
    first, up to three), its end corners pushed up to 3 m to meet it
    (`END_PUSH`; 2.7 before, and past 60 degrees a corner missed a round
    island altogether and the end hung off it) or pulled back onto another
    bridge's plank, and the plank that lands runs at least half a plank
    into the ground, under the land, so that its end edge always crosses
    the shore (it ended on its last rung, with the corners pushed, which
    missed when the last plank pointed past the shore); heading at ground
    within a plank past the next rung, along the plank or along your way,
    no rung is fixed (`STRETCH`): the tip stretches to the ground, so that
    the plank that lands is a whole one, with room for its cut; the rungs
    aim two planks ahead of you along your way (`LEAD`), so that after a
    sharp turn they settle onto your line within a few degrees rather than
    swinging 25 degrees past it; turning back over your own planks and off
    their side ends the chain where it is and begins a fork there, and off
    them onto land or another bridge ends it with no landing
    (`Bridges::abandon`, `Chain::behind`; the tip stayed frozen where you
    had turned back, and off your own bridge's side there was nothing under
    you: a scripted zigzag fell into the void); you have left the ground a
    root is on 0.2 m past its edge line (`GONE_OUT`; half a plank, which
    held back a landing on a bridge half a metre from a fork's root until a
    rung had been fixed on the way); and edges are felt to 5 mm (`EDGE`;
    2 cm left a hairline of void along a cut now and then). **A turn too
    sharp for the tip to follow makes an L corner** (later that day,
    Hajun's third and fourth screenshots, "it happens when i do a close to
    90 degree turn": a rung was fixed a plank on, past where you had
    turned, you walked off that plank's side, and the fork began there,
    leaving the rest of the plank sticking out past the fork and the
    fork's root cut, a long diagonal for a slanted fork, hanging past the
    plank's end; landing within a metre the same fork came out a
    triangle): stepping off the tip through its side
    (`Chain::tip_side_exit`), the tip is fixed as the chain's last plank
    cut off flush with the far edge of the fork that begins off its side,
    `HALF_WIDTH / cos(slant)` past where you stepped off and a hair more
    (`Bridges::corner`, `CORNER_ROOM`), so that nothing sticks out and
    nothing of the fork hangs; and a fork landing within a plank or two
    slants no more than that length can cut cleanly along the plank it
    leaves (`land_on`, from `CUT_SHARE`), so a short one is a square
    gusset rather than a triangle. What stays: turning back over your own
    fixed planks and off their side still leaves the plank past the turn
    as a dead end (it was sent as it was fixed), and crossing a junction
    of bridges at a slant lays a plank across each wedge of void between
    them. Seen in the desktop build, scripted (a throwaway
    `src/scratch_bridge.rs`, removed; 17 legs, no fall): off the island at
    40 degrees, onto the town at 60, a 35-degree fork off a bridge, a
    crossing onto the town, back along a chain and off its side, turns of
    113, 107 and 63 degrees; then right angles over the void, a right
    angle a metre off the town's shore and onto it, and a zigzag of right
    angles, no fall; and unit-tested: a fork cut flush at 0 to 45
    degrees, landings at 0 to 85 degrees meeting the shore whole, drifting
    off a bridge nearly along its edge, a 63-degree turn settling within
    10 degrees, a landing on another bridge cut flush, an L corner at 0,
    30 and 60 degrees with nothing sticking out, and a short fork landing
    with a slant its length can cut. Hajun played the first of these on
    the phone (APK 13) and found the right-angle turns; the rest not yet
    by hand.
    **A bridge that is built stays as it was built** (still later that
    day, Hajun, playing APK 14, with four screenshots, `errorframe1.jpg`
    to `4`: "when there is a unfinised end, and then i build another
    bridge that passes near that unfinished end, suddenly that unfinished
    end's vertices try to connect to my current bridge. i think bridge
    that is already made should not change"). A chain's corners are
    worked out only when the chain itself changes, a rung fixed (or come
    from the server) or its tip moved, against the ground as it stands
    then, and never because another chain has (`Chain::stale`, by how
    many rungs it has and its tip; `Chain::cut`; `Bridges::recut`, the
    chains in the order they were begun); and its root is cut for good
    once its third rung is fixed (`ROOT_KEPT`: its first plank and the
    joint after it both settled; `Chain::root_cut`). Until then `rebuild`
    worked every chain out again, against the land and every other
    chain's planks as they then stood, each time anything was fixed or a
    tip had moved 0.2 m: whatever was built near an old root or end
    afterwards recut it. **And only an end that lands is cut along what
    it lands on** (`corners_of`: its last rung on ground, or ground within
    0.15 m ahead): an end left over the void, let go of, turned back from
    or ended at an L corner, is square where it was left. Its corners had
    been pushed on like a landing's, up to `END_PUSH`, 3 m, so an end
    left within that of a shore or a bridge grew a spike out to it, as
    one in Hajun's first screenshot had to the town's edge before the new
    bridge was begun. So someone else's chain, online, shows a square end
    at each rung as it comes, and is cut along the shore by the rung that
    lands. The meshes follow: a chain is drawn again only when it has
    changed (or the file's colours have), where every chain's mesh was
    made and uploaded again each time; the shape laid over the island is
    still made of all of them each time, from triangles each chain keeps.
    **And a chain forgotten takes its tip's plank down with it**
    (`Bridges::forget`, `orphans`): a chain of nothing but its root,
    walked back from, was dropped and what its tip had drawn left where
    it was, out over the edge with nothing to stand on (found reading
    the code for this; not something Hajun reported). Seen in the desktop
    build, scripted (a throwaway `src/bridge/scratch_bridge.rs`, removed):
    two bridges left unfinished off the island, a third built a metre or
    two past the end of each, every chain's corners logged before, during
    and after. On the code before, the first end's corners moved up to
    0.77 m onto the new bridge and the second's grew a 2 m spike to it,
    as in the screenshots; on this, both chains' corners were the same
    to the millimetre in all five logs, and no fall. Unit-tested: an old
    bridge unchanged as others are built past its end, over its end and
    beside its root, where worked out afresh it would change; an end left
    2 m short of a shore square; a root cut afresh until its third rung
    and kept from then on; a forgotten chain's mesh handed over to be
    taken down. Not yet on a phone.
    **A root and an end follow the edge they meet all the way across**
    (the same day, Hajun, of a sliver in that first screenshot that I
    had pointed out: "fix the sliver gap on curved forks too"). Each was
    one straight line between its two corners, and only the corners were
    cut onto the edge: a chord across an edge that bends. Off the inside
    of a curve of bridge that left void between the fork and the bridge
    it left: a sliver on a gentle curve, and on one turning 20 degrees a
    plank 0.4 m deep in the middle, right where you step off, with
    nothing under the step; off the outside the fork lay 0.18 m over the
    bridge instead. Now the line is tried all the way across
    (`cut_along`): at 9 places evenly (`STATIONS`), each pushed or
    pulled onto the edge as a corner is (`to_edge`), and halfway between
    any two again, up to three times over (`REFINEMENTS`, so as fine as
    4.3 cm), wherever the edge there is 3 mm off the straight line
    between them (`BENT`); of those, the ones it takes to follow the
    edge within 4 mm are kept (`bends`, `STRAIGHT`), none along a
    straight edge. They are the root's edge and the end's (`Edge`,
    `Shape::root`, `Shape::end`, `Chain::root_edge`, `end_edge`, the
    root's kept with its corners at the third rung), and the first
    plank's near edge, or the last one's far edge, is built along them,
    its top and underside in strips side by side (`strips`, in `planks`
    and in `Chain::ground_quads`, which `plank_under` and the ground
    other chains are cut against go by, so what is stood on, what is
    drawn and what is cut against are one shape); a plank between
    straight edges is the one quad it always was. Every place is set
    1.5 cm in under the ground it meets (`TUCK`; the corners too, which
    lay on the very edge, to 2.5 mm, until then), so that no hairline is
    left along the cut: under the land, or under or over another
    bridge's plank, whose top is a millimetre or more off. A plank cut
    at both its ends, a fork of one plank, has its end pulled back no
    further than its root was pulled in leaves it. `corners_of` is
    `shape_of` now, and what it did the tests' alone. Off the inside of
    a bend as tight as a bridge turns (35 degrees a plank, 0.8 m round
    inside) a root is wider than the bend: it follows the edge where it
    can and lies over the bridge either side of that, no void. Cutting
    asks only after other chains' planks within 6 m of the chain's root
    or its last rung (`NEAR`), where it scanned every plank in the game
    for each question. The cost, measured in the desktop build while a
    chain is being begun, its root worked out afresh every frame: 22 µs
    a frame on average and 153 µs at the most, of a frame's 8,333; not
    measured on a phone. Seen in the desktop build, scripted (the
    throwaway again, removed), on the code before and on this: a bridge
    off the island round a bend, a fork off its inside, one off its
    outside and one off its inside at a slant; before, a dark sliver
    along each inside fork's root, and on this none, and no fall.
    Unit-tested: forks off the inside and the outside of a 20 degree
    bend lie along its bent edge all the way across, neither void nor
    overlap, where the chord stood 0.4 m out or 0.18 m in; stepping off
    bends of 10 to 35 degrees a plank, near a joint and between two,
    inside and outside, square and slanted 0.6 rad either way, 72 ways,
    nothing but bridge within half a metre of the step and no strip
    folded; landing on a curve the same; a straight edge still one quad.
    Not yet by hand or on a phone.
  - **The metres left are written in front of you as you build, and
    running out stops you** (since the afternoon of 2026-11-10, Hajun: "i
    think i need another number indicator in front of where ur bridging
    that is the same '### m' number of bridge wood left, and it should be
    white font but orange -> red when its 10m->5m left. and when it become
    0m, it should exit bridge mode (not able to go back to bridge mode
    when u dont have any bridging m left) and pause (even if the user is
    holding the joystick, so the user need to let go of the screen and do
    the joystick again to move) so that they dont fall off when their
    bridge ends"). **The number** (`bridge::show_left`, `LeftAhead`): the
    one that was by the log top left until 2026-11-11, "500m" (`Resources::metres`), as a line of UI
    text 3.6 shares of the short side (`LEFT_TEXT`) placed 2.4 m ahead of
    you the way you face (`LEFT_AHEAD`: past the end of the plank being
    laid, which reaches 1.05 m), after `follow_camera`, while building is
    on. Its colours are `assets/ui.ron`'s `colours.arena.bridge_left`
    (white), `bridge_low` (orange) and `bridge_out` (red), and where it
    turns the file's `arena.bridge_low_at`, 10, and `bridge_red_at`, 5,
    both Hajun's (`Arena::check` refuses them inside out): white with
    more than 10 m, orange at 10, shading evenly to red at 5 and under
    (`bridge::left_colour`), by the number as it is written, to the
    nearest whole metre, so "10m" is orange from 10.4. The shading
    between is my reading of "orange -> red"; the size and the 2.4 m are
    mine. Yours alone, nothing of it online. **Running out**
    (`bridge::lay`, `RanOut`): the frame the plank being laid can no
    longer keep 1.05 m ahead of you for want of metres (`Grown::spent`,
    set by `grow`: the tip is as long as the metres go and never shorter
    than `AHEAD`), or what is left is under half a metre, which is
    written "0m" (`OUT_BELOW`), what is being built is fixed where it is
    (`let_go`), the metres are set to none, building is turned off, and
    you are stopped: your walk is zeroed that frame, and
    `read_player_input` gives none while `RanOut::halted`, whatever the
    stick or the keys say, until a frame comes with neither pushed, which
    lets you go; pushed again, you walk. So you stand on the bridge with
    a plank's worth still ahead of you, about a metre (0.87 m at the
    least, at 60 frames a second: the tip is aimed at you only once you
    are a hand's width past its rung, and a frame's step goes by), and
    the last plank takes all that was left, up to a plank and that
    reach. Dying lets you go too. The "0m", red, stays in front of
    you for as long as you stand stopped and a second and a half
    (`OUT_SHOWN`), my addition, so that the stop says why. **And building
    stays off**: `bridge::toggle` turns it on only with half a metre or
    more, and a press with none shows the "0m" for that second and a
    half instead. Until then the tip ran out under you and you walked
    off the end of it; and with less than a plank's worth left (1.363 m)
    no chain began at all, building on or not, so that off an edge you
    fell with a plank in your arms: a chain begins now with anything not
    written "0m", a stub of 1.05 m that stops you at the edge. Seen in
    the desktop build, scripted through the keys (the throwaway again,
    removed), with 13 m: white "13m" on the island, orange at 10.3 m,
    red at 4.8, stopped on the bridge at none with 1.03 m of plank ahead
    and the key still held, not moved in 2 s, the plank button refused,
    and, the key let go and another pressed, walking again. Unit-tested:
    the bridge running out with 4.5, 12 and 0.6 m, a plank under you
    every frame and one ahead as it ends; the colours; standing until
    the stick, or the keys, are let go. Not yet by hand or on a phone,
    where the stick is a finger's.
  - **A minimap top right** (`src/minimap.rs`, since 2026-11-09, Hajun:
    "put a minimap that only shows the current player's location as a red
    dot and your teammate as a green dot, and the full map in top down view
    including the live bridges but not show the other players"): a picture
    256 pixels square of the map from above, 300 m across with north up,
    the land in `colours.arena.map_land` over `map_back` (the node's
    background, the picture clear round the land since late on
    2026-11-11), drawn once from
    the map's own triangles that face up and lie under 2 m (`draw_land`:
    the islands, the town's middle, the desks, and since the map of
    2026-11-11 the barrels; not the stands' roofs, the trees' leaves, the
    lamps' heads or the waiting room), the bridges drawn over it again in
    the planks' colour
    whenever they change (`draw_bridges`, `Bridges::version`), and over the
    picture a dot for you (`map_me`, red) and one for each teammate
    (`map_mate`, green), placed as you move and hidden when dead
    (`place_dots`). Nobody else is on it. **Each sapphire and each
    amethyst generator is marked on it** (`minimap::mark_generators`,
    since 2026-11-12; Hajun: "in the minimap, add a smaller sapphire and
    amethyst logo onto the minimap where the generators are. so the icon
    need to be smaller so it fits inside the smaller sapphire island in
    the minimap"): the item's stone in its colour, 5.5% of the map across
    (`MARK`) where the smaller islands are 8%, so each sits inside its
    island with the island showing round it, under the dots. It is the
    stone alone, filled, drawn 32 pixels across for the size
    (`hud::item_mark`): the picture the counts have, 128 pixels with the
    line round its table, came out ragged with a ring of flecks at 13
    pixels. Seen in the desktop build: 11 pixels across on islands 18
    across, in a 900-pixel-high window. `hud::spawn_arena_bar` puts it
    up (`hud::MinimapNode`, 26 shares of the short side, `hud::MINIMAP`);
    the picture is made as the app opens (`minimap::plugin`), clear, so
    that the bar can hold it before the map is read.

- **The gamepad down the right edge opens the game selector**
  (`src/games.rs`, `hud::GamesButton`): a dialog like the settings'
  window, "Games" over three cards, Bridge Wars ("Free"), Home Builders
  (its 100 coins, or "Need 100 coins" and blocked out without them) and
  "???" ("Coming soon", blocked out), the cross on its corner, the dark
  round it or Escape to shut it; only in the town, with nothing else up.
  Choosing a game, a black screen comes down over everything
  (`games::Loading`, you standing still under it as under a dialog):
  half a second later the game is gone into under it, Bridge Wars'
  waiting room (`arena::Ask::Enter`) or House Builder's lobby
  (`builder::Ask::Join`, paying its fee as the House Builder's Play did),
  and once the island you are on has loaded and a moment more has passed
  it lifts over 0.3 s; should nothing come of the choice (the server
  turning it down), it lifts after 3 s. The words are `assets/ui.ron`'s
  (`words.games`). House Builder's own flow from the lobby on is as it
  was ("House Builder", below); the House Builder NPC stands at a house
  only in a town that has one, and `roundtownmap.glb` has none.
- **Bridge Wars** (`src/arena.rs`): you are stood in the waiting room,
  Hajun's glass box floating over the middle of the map, the object
  called `waiting room` in `pvptownmap.glb` (their export of 23:21 that
  night: 30 m square, its floor a metre thick with its top at 100.5 m, its
  walls 1.5 m thick, thicker than the lattice, and 5 m high, more than a
  jump, open to the sky, in `glass` at a tenth; 50 m higher, its floor's
  top at 150.5 m, since their map of 2026-11-11, "The PvP map itself",
  below), found by name once the
  map has spawned (`arena::read_map`, `WaitingRoom`: the box
  round its meshes; its floor is read off the map's shape under its
  middle, so it is stood on as any of the map is) and on a ring 3 m round
  its middle, facing in; with no such object in a map, everyone waits on
  their island. The board top centre (small, as asked) counts "1/4
  Players", "Waiting for players..."; nobody else can join yet, the game
  being this device's own, so after `BOT_WAIT`, 10 s with nobody joining
  (`roundtown_net::bridge_wars`, with `COUNTDOWN` 5 and `PLAY_SECS` 600,
  so that a server playing it one day plays by the same), the computer
  takes the seats left, all at once (`ARENA_SEATS`, 4; the town's AI
  names and looks, `lobby::seat_ai`, `spawn_player_at`), the board counts
  down 5 to 1 over "Match found!", and everyone is sent to the island
  their seat says (`roundtown_net::arrival`, as the town's arrivals were
  that day), "Fight!" across the screen; then the board shows 10:00
  (25:00 since 2026-11-11, with a timetable on the way, above)
  counting down over "Bridge Wars", a tap on it skips (any stage: the
  bots at once, the countdown over, the game over), and at 0:00 everyone
  is back in the town where their seat arrives, the bots gone
  (`arena::InArena` on everything the game puts up), the camera as it
  was. Nothing is scored; `arena::Health` stays full and
  `arena::Resources` at nothing. A game of House Builder is entered the
  same way and runs as before. Seen in the desktop build, scripted (taps
  through `WindowEvent::TouchInput` at the buttons' laid-out places,
  `arena::Ask::Skip` written to hurry it): the dialog, the black screen,
  the waiting room from above, four bodies at the countdown, the island
  with "Fight!", and the town again with the camera and the buttons
  restored; and Home Builders to its lobby with the fee taken. Not yet by
  hand or on a phone.
- **The fight** (later that night, Hajun: "make the ai, if there is
  another ai/player within its visible radius, it will go towards them
  and try to fight/punch them ... make punching deal damage, and show
  healthbar for enemies. make default hp = 100 ... make damage numbers
  appear on each hit. default damage = 12. so when health <0 u die too
  ... 'You died. Respawning in 4...' -> 'You died. Respawning in 3.5...'
  until it goes to 0 and then you respawn and the camera goes to where u
  respawned ... for falling make it so u need to drop a bit more z to
  actually die. increase knockback slightly ... make it running rather
  than walking"); every number is `assets/ui.ron`'s `arena` (`ui::Arena`,
  read as the car's are, hot-reloaded): `run_speed` 10 m/s (a walk is 7),
  `health` 100, `damage` 12, `respawn_secs` 4, `regen_after` 5 and
  `regen_per_sec` 4, `hunt_radius` 24, `bot_punch_every` 0.8 (0.5 for an
  hour, until Hajun found the bots "so strong": "decrease ai hit
  frequency slightly"), `flee_below` 0.2 and `fall_to_die` 30 (`void_y`
  in `src/lib.rs`, the land less that; 15 m until then). Everyone runs in the arena
  (`Intent::sprint`, the run over a walk; the legs swing that much faster
  and up to half as far again, `animate_walk`; set by `read_player_input`
  for you and `ai::steer` for a bot, by the island they are on). Every
  body there has `arena::Health`, you included (given on entering, taken
  away on leaving); a punch that lands and takes (`Landed`, written by
  `start_punch` and by a bot's punch) takes `damage` off the victim while
  the game is on, never in the waiting room (`arena::deal_damage`), a
  number floating up off them for 0.9 s (`DamageNumber`, in
  `colours.arena.damage`), and everyone but you has a bar over their head
  (`BarOf`, placed as names are, `tags`), yours being the bar top left;
  both drawn in steps of half a percent, so that health coming back a
  little every frame does not lay the screen out again every frame.
  **Health comes back** once nobody has hit a body for `regen_after`
  seconds, `regen_per_sec` a second up to full (`arena::regen`, `Health::
  mend`; the while since the last hit counted without marking the health
  changed; later that night, Hajun: "make players health regen slowly
  after they have not been attacked for some period"); bots' too. At none,
  or fallen `fall_to_die` below the map (`arena::reap`), a body is `Dead`
  (`src/lib.rs`): hidden where it died (`show_where_i_am` for everyone
  else, `follow_camera` for you; for an hour a dead bot stood where it
  died in plain sight, Hajun: "the ai body doesnt disappear"), neither
  moving nor moved nor in anyone's way nor hit (`move_bodies`,
  `start_punch`, `ai`), the punch's range band put away with you
  (`range::follow`; it stayed over the void for an hour, Hajun: "the hit
  indicator is not gone"), for `respawn_secs`, you with "You died.
  Respawning in 4..." across the screen counting down in halves
  (`words.arena.died`, `arena::show_death`), and then put back where its
  seat arrives with its health full (`arena::respawn`; the camera, fixed
  over you, is there with you). (What follows was every bot's mind
  offline until 2026-11-12, and is the zombies' alone since; the
  computer's players have the brain the server's have, above.) **A bot
  goes for whoever is nearest within `hunt_radius`** (`arena::hunt`, after `ai::steer`, over its
  wandering, only while the game is on): at a run up to them, pressing in
  at `PRESS_IN` of a push once within the punch's reach, and a punch
  whenever they are in the punch's own range (`range::Sector`) and its
  `bot_punch_every` is up, which knocks and lands as yours does; a hit on
  the move is a sprint hit, as ever. **With less than `flee_below` of its
  health it runs away instead** (Hajun: "make the ai run away when its
  low health (below 20%)"; `arena::flight`): straight away from them, or,
  with no dry land that way for `FLIGHT` 3 m, the nearest way round that
  has it, to either side and then across (`ai::dry_all_the_way`), and
  stands with none; so beside the island's middle it runs to the edge and
  then along it, and seen on the map it ran to the bridge's end, where it
  stood a while. Once its health is back over the line it fights again.
  It sees the dead not at all, and is kept from walking off its island
  as ever (`StaysAshore`), so a target across the void it stands at the
  edge for; bots on four islands 130 m apart never see each other until
  someone crosses a bridge. The knockback is `melee.knock_back` 10 (8,
  Minecraft's, until then). Seen in the desktop build, scripted
  (`arena::Ask::Enter` and `Skip` written, a bot put beside you with
  `lobby::put`, its health and yours written to): the bot coming, "12"
  off each hit on either, its bar over its head, you knocked off the
  island into the void, the words counting down over the dark with the
  band gone, and you standing again at the island's middle with the bar
  full; a bot's hit on you right after; a bot at 15% running off to the
  edge and round; a dead bot gone, bar and all, and back on its island
  4 s later; health climbing 4 a second from 5 s after the last hit. Not
  yet by hand or on a phone.
- **Online, a game of Bridge Wars is a room of your own**, as home is:
  entering it asks the server `Ask::Travel(Venue::Arena)`, which seats
  you alone there (`server/src/players.rs`, `room_for`, as `Venue::Home`),
  so that you vanish from the town's channel rather than stand frozen in
  it, and leaving asks `Travel(Venue::Town)`. Nothing of where you stand
  in the arena goes to the server (`net::tell_where_i_am`), and its
  `Tell::Arrive` for the arena never moves you (`net::hear`): the game is
  the device's, bots and all. The server's `Ask::Respawn` is not asked
  there (`lobby::respawn_the_fallen`).
- **The camera is fixed straight down over you only in the arena**
  (`OrbitCamera::top_down`, which keeps how it was to go back to,
  `arena::Game::view`; the car's `overhead` keeps its own, and going into
  the arena gets you out of the car, `drive::toggle` on the change of
  island, the camera going back to how it was before the car rather than
  to the car's, `Driving::take_view`): the file's `arena.camera_height`
  up, 15.5 m, with the top of the screen north, nothing turning it and,
  since late on 2026-11-11, nothing zooming it ("The camera stands at one
  height in a game", above; the car's `overhead`, 24 m, zooming between
  `overhead_nearest` and `overhead_furthest`, until then). Everywhere else it is behind
  you as it was before 2026-11-07, "The camera stands where you zoomed
  it" below, and on desktop the cursor is taken for looking round as the
  app opens (`lock_cursor`, unless `RT_FREE_CURSOR`), and let go for as
  long as the camera is fixed over you, in the car or the arena
  (`cursor_for_dialogs`), where the stick and the buttons want the mouse.
- **In the arena the buttons are Hajun's `pvpmapbuttonui.jpg`**
  (`hud::Layout::Arena`, `hud::spawn_arena_bar`, put up as you arrive on
  the arena and the town's again as you leave, `hud::relayout`): top left
  your name on its plate and under it your health bar, 30 by 3.6 shares of
  the short side with a rim, filled red (`colours.arena` in
  `assets/ui.ron`: `hp`, `hp_back`, `hp_rim`), and under that what you
  hold of the four things the generators make, silver, gold, sapphire and
  amethyst, each a picture of it, tinted its colour, with its count
  beside it (`hud::RESOURCE_ICON`, `RESOURCE_TEXT`, `hud::item_picture`,
  since 2026-11-11, above; until then wood, gold, diamond and emerald,
  white, the wood's count your metres of bridge and the rest 0) (there since 2026-11-09,
  Hajun: "move the 4 item icon + number thing on the top right side of
  that screen to the left side of the screen below the hp bar ... remove
  the inventory icon"; for a night they were top right under a bag, the
  inventory, which did nothing); top right the minimap in the bag's place
  ("A minimap top right", above); bottom right, in the jump button's very place (the jump button
  is put away there, `hide_touch_hud`), the sword, which punches as the
  fist does (`hud::PunchButton`, so in the car it is the reverse), and
  round it on an arc 20.5 shares out from its middle three buttons the
  size of the town's action buttons, alike and evenly spaced (later that
  night, Hajun: "make the 3 circles surrounding the sword icon in that
  map the same size ... and even the distances out"; for an hour two were
  that size and one over the sword small): beside it the jump button
  (`hud::ARENA_JUMP`, which `hud_layout` in `src/lib.rs` reads presses
  on in the arena, lit while held as the one in its own place is; Hajun:
  "i want the bottom left one to be the jump icon and a jump button"),
  and up and to the left of it and over it two empty slots
  (`hud::ARENA_SLOTS`, `hud::empty_slot`: the buttons' disc with a faint
  rim, `colours.arena.slot_rim`, which takes a press and does nothing),
  where a bow and dynamite go once bought (the one over the sword is the
  bridge button since 2026-11-08, and the other the second button, for
  what the item shop sells to use, since 2026-11-12: `hud::utility_slot`,
  `utility`); and over those a smaller
  button of three dots (`hud::ARENA_DOTS`), which does nothing yet (it
  showed the empty item buttons from 2026-11-13, and is not put up at all
  since 2026-11-14, `hud::DOTS`: "Games", above); the
  stick stays, and nothing else of the town's bar is there (no balance,
  person, gear, gift, social, face or chat). The pictures are drawn in code
  (`hud::sword`, `item_picture`, `plank`, `dots`, and the `bag`
  nothing shows now, through `hud::drawn` and a polygon distance,
  `polygon_distance`; tests sample each; the `log`, the `ingot`, the `gem`
  and the `emerald` went on 2026-11-11). Space still jumps on desktop. The punch's range band is drawn
  only in the arena now (`range::follow`).
- **Falling off the map** is the arena's alone (`island::bottomless`):
  the town has its sea back. The bots are kept on by `strands` as the
  town's AIs are; a punch knocks them off all the same, and they are dead
  `fall_to_die` down, as you are ("The fight", above).

### The PvP map itself

(Since the night of 2026-11-08 the map has no bridges: Hajun's `bridge`
object was taken out of `pvptownmap.blend`, and every bridge is built by
walking, "Games" above. What follows describes the bridges as they were.)

**The map as it is since 2026-11-11** (Hajun: "i updated the pvp map
slightly. adjust the spawn locations/etc accordingly. dont rebuild apk.
just fix"; their export of 18:50, with the modifiers applied, the islands'
arrays whole: 89 objects, 78,380 triangles, three PNG textures embedded,
the trees'). What the game goes by, and what changed in it:

- **The waiting room is 50 m higher**: its floor's top 150.5 m up, its
  walls to 155.5, its glass white at a twentieth. From it, under the
  camera 24 m over you, the whole middle of the map and the islands either
  side are in sight. `roundtown_net::bridge_wars::WAITING_FLOOR` is 150.5
  (100.5 until then); offline the floor is read off the map as ever.
- **The anchors** are the objects `spawn` to `spawn.003`, still in the
  material `spawn`: two pyramids base to base, 8 triangles, standing on
  their point on the slab's top, 0.75 m up, 1.41 m square at 2.25 and
  ending at 3.75 (a cube on its corner, 0.54 to 3.96, until then), 124 m
  out and 0.16 m off each axis, each a quarter turn on from the last
  about the middle. `ARENA_ANCHORS` is (0.16, 124), (-124, 0.16), (-0.16,
  -124), (124, -0.16); the bar over each stands 4.4 m up
  (`arena::ANCHOR_BAR_OVER`, 4.6 until then). They had stood up to 0.92 m
  from there, placed one by one; left at the old numbers, six of the
  eight triangles of team 0's pillar would have stayed in the way after
  it broke, being further from where the game had it than `is_pillar`
  reaches (the test that reads the pillars off the map failed on the
  distance).
- **Each island's middle is open ground**: the desk is gone, and in its
  place two market stands (`MarketStand_2` to `.007`, the model in
  `assets/polypizza/`, blue-roofed and red-roofed) 4.65 to 5.94 m either
  side of the middle, a barrel behind each (`Cylinder` to `.007`), and
  round the front of each a white outline on the ground (`Plane.003` to
  `.006`, in a material called `activation`, 2 cm over the island: drawn,
  and walked over as the road's paint is; the one at the red stand is
  where the shop opens, since that night, `market`). Everyone still
  arrives in the middle (`ARENA_ISLANDS`), and is no
  longer shoved to a desk's edge; the second of a team, 4 m toward +X,
  stands on islands 0 and 2 0.65 m from a stand, at the edge of its
  outline, and on islands 1 and 3 between the middle and the anchor's
  slab or the pad. The anchor's slab is where it was, and the low pad at
  the far edge is `gen1` to `gen4`. **No lamp stands on an island any
  more** (the four at their far edges went: 20 lamps, all in the middle),
  so at night an island had the moon alone, until its anchor was made to
  glow and light it the same evening ("Bridge Wars' respawn anchors glow
  and light their islands by night", with the street lamps, near the top
  of this file).
- **The rest**: the four smaller islands, `SapphireIslands`, 24 m across
  and 125 m out on the diagonals, each with a pad (`sapphiregen` to `4`)
  and two palm trees; four more pads in the middle, 35.8 m out on the
  axes (`amethystgen` to `.003`); a pine and two trees on the planters;
  the fountain's tower and bowls gone from the pond, which stays; the
  islands' tops `stone`. None of it is anything to the game but land and
  scenery. Hajun's `amethyst_item.glb`, `gold_item.glb`,
  `sapphire_item.glb` and `silver_item.glb` in `assets/` were loaded by
  nothing, and the pads were scenery, until that night, when they became
  what the generators make and where ("Games", above).
- **Checked on the map itself** (a throwaway test drawing what a body
  bumps into, removed): the stands, the barrels, the slabs and the trees'
  trunks are solid and nothing else is; no invisible wall under a stand's
  roof or a tree's leaves, which a part left open over a walkable place
  would make ("A solid has to be a closed shape"). A test keeps every
  arrival on the island's own top with nothing reaching into a body there
  (`every_arrival_is_dry_land_in_the_map_with_the_void_beyond_the_islands`).
  The server's bots, which have no map, walk through the stands as they
  did the desk.
- **`PROTOCOL_VERSION` is 10 still, and the server was deployed with the
  new numbers** (2026-11-11, the binary before kept on the VM as
  `roundtown-server.prev-2026-11-11`): without it the server put everyone
  100.5 m up, under the new room, over nothing. An app with the map
  before (APK 17, which was not rebuilt) is put 50 m over its own room
  online, which is open to the sky, and falls into it; the server lets
  any fall go (`Pace::take`), so nothing is put back.
- Seen in the desktop build, scripted (`arena::Ask::Enter` and `Skip`
  written by a throwaway module, removed), offline and online against the
  VM: waiting at (0, 150.5, 3), the map under the glass; arriving at
  (0, 0.25, 130) and staying there; the anchor with its bar over it, the
  stands, the barrels and the outlines drawn, the minimap with all eight
  islands; and back in the town. Not by hand, and not on a phone.

The map (Hajun, 2026-11-07: "use pvptownmap instead of roudntownmap ...
make the player start in the center of one of the 4 bridge connected
island, remove the ocean and it will be a void that if you step over, u
fall and then u 'die' and respawn back at your spawn ... pressing melee
(the punch) actually deals 'knockback' (minecraft 1.8.9 bedwars style
melee knockback) ... make the camera top down for even the player mode")
is Hajun's `pvptownmap.glb`: Round Town's middle, the pond and its
fountain (the fountain's water is drawn but does not run there, the
fountain being read off the town's model alone), the ring road round it
in its two halves with the strip of grass between, the lamps and the
paving out to 76 m, and no houses, trees or outer road; and out of it
four wooden bridges, one along each axis, each to a round island at 130 m
(`roundtown_net::ARENA_ISLANDS`; since the evening of 2026-11-07, when
Hajun scaled the islands' object to 0.65 across the ground, its array of
four with it, and shortened the bridges to meet them: 200 m, 32 m across,
over a smaller island at 125 m, with 14 m bridges, that morning), 20.8 m
across and 3 m thick, its top 0.25 m up, the smaller island's far half
under it; the bridges planked, 3.5 m wide, their deck 0.5 m up with a lip
either side to 0.75; and, since 23:21 that night, the waiting room over
the middle (above). All of it stands in the sky. No buses drive it (the
buses are the town's, `car`) and no AI of the town walks it, only the
game's bots. **Everyone arrives in the very middle of their island**
(`roundtown_net::ARENA_ISLANDS`, 130 m out on each axis; later that
night, Hajun: "make the spawnpoint in the center of the island so that u
dont need to hide the purple rotated cube"): the `spawn` pillar and its
slab are drawn and solid as anything else now (the game hid the pillar
and arrived at it for the evening before, `island::SPAWN_MATERIAL`,
gone). The desk across each island's middle (there until the map of
2026-11-11) begins 0.1 m past the
middle, so a body arriving is shoved to its edge, toward the bridge; and
the pillar's slab, 0.5 m high, is a wall to walking (as the fountain's
rim is, "A body steps up anything under 0.5 m"), to be jumped onto or
gone round: a scripted run north from the middle stopped at it.

- **The islands and the bridges are Geometry Nodes arrays in
  `pvptownmap.blend`**, which Hajun's exports leave off (that morning,
  and again at 16:44 that evening): their `.glb` has one island, one
  small island and one bridge (124, 124 and 44 triangles, where applied
  they are 496, 496 and 176). `assets/pvptownmap.glb` is the file
  exported again with Apply Modifiers ticked (`export_apply`, and from
  `assets/` itself: exported from a copy elsewhere, the bridges' wood
  texture, `assets/woodplanktexture.jpg` by a path relative to the
  `.blend`, is silently dropped), 41,006 triangles since that evening
  (40,878 that morning), and that is how to export it after a change.
  That evening's change: the islands pulled in to 130 m (above), the
  bridges planked, the `wood` material a texture embedded in the `.glb`
  (the game's second texture, after the Soft Couch's) on a bridge of 44
  triangles, its deck 0.5 m up and a lip either side to 0.75 (a
  see-through blue box of 12 until then), and the islands' tops `path2`,
  the paving's brown, in place of `stone`. **The texture is a JPEG, which
  Bevy decodes only with its `jpeg` feature**, on in `Cargo.toml` since:
  without it the whole town fails to load ("invalid image mime type:
  image/jpeg" in the log) and everyone waits in the air over nothing.
  Every part has a material and every walkable top faces up (checked both
  times). **The map's third version of the day (17:31), exported by Hajun
  with the modifiers applied** (their `.glb` is byte for byte the applied
  export: so it is the file as they made it, 41,642 triangles, 46
  objects), furnishes one island, the one at the game's +Z
  (`TOWN_ISLANDS[0]` since late that evening, `[2]` for an hour;
  Blender's +Y, the top of Hajun's screen in Blender and the bottom of
  the game's, the camera's top being -Z): two stone slabs (`Cube`, 2 m
  square, 0.25 m high at the far edge; `Cube.001`, 0.5 m high, a jump,
  toward the bridge), on the second a purple pillar 3.4 m tall in a
  material called `spawn` (`Cube.002`, 5 m from the island's middle
  toward the town), and a cafe desk 11.3 m long and 0.9 m high
  (`Cube.003`) lying across the island's middle. **A `spawn` part marks
  where that island's first arrival stands** (`island::SPAWN_MATERIAL`,
  since Hajun's "make the user spawn in the bottom one with the
  furnishes"): the game neither draws it nor stands anyone on it, and
  `TOWN_ISLANDS[0]` is set by hand to its spot, on the slab (the server
  has no map to read it from); the ring of seven round it, 4 m out,
  clears the slab, the desk and the bridge. For an hour seat 0 arrived
  on the bare island opposite. **The fourth version (18:27) furnishes
  all four islands alike**, each with its slabs, desk and `spawn` pillar
  turned a quarter on from the last, so every island's arrival is its
  own marker (`roundtown_net::ARENA_ISLANDS`, `TOWN_ISLANDS` that day, all four by hand, 123.8 to 124.9 m out),
  and stands a street lamp at the far edge of each (24 lamps now):
  50,270 triangles, 62 objects. Hajun's export of it again had one small
  island and one bridge, so `assets/pvptownmap.glb` is the applied
  export. **A glTF position is not a game
  position**: the game turns every model half round about the upright
  ("Models"), so a point read off the `.glb`'s JSON is at (-x, y, -z) in
  the game, and a dump that forgets it puts a thing on the opposite
  island (as mine did for an hour that evening); the game's own reader,
  `island::glb::read`, has it right. **The
  pond has a floor under its water**, 0.1 m up, out to 24.8 m, inside a
  rim 0.7 m high: standing room that nobody walks into or out of, the
  rim being higher than a step, and that a knock can put a body in, to
  jump out of. Until 2026-11-12 this said it had none, and that under
  the pond was the void, which was wrong of this map: found baking it
  for the computer's players ("The computer's players play the game",
  above).
- **There is no sea round the town** (`island::bottomless`, which only
  `Venue::Town` is; the home, the lobby and the plots keep theirs):
  `spawn_island` spawns none for it, and `support` in `src/lib.rs` has
  nothing for a body off the land to come down on (`Ground::bottomless`),
  so it falls, at `FALL_GRAVITY`, until it is `VOID_Y` down, 15 m, a
  second's fall, and dead: `lobby::respawn_the_fallen` puts it back where
  its seat arrives (`send`), you and this device's AIs alike. The sky's
  clear colour is what shows below. Online you are put back at once, no
  move is sent, and the server is asked to put you there too
  (`Ask::Respawn`, `net::ask_to_respawn`, `Online::venue` cleared until its
  `Tell::Arrive` comes), as a fresh arrival: the leap back would not pass
  its check on moves, and be undone. AIs never walk off (`strands`, as
  ever): only a punch takes one over the edge.
- **Everyone arrives on an island, by seat** (`roundtown_net::arrival`,
  `arena_island`, `town_island` that day: seat modulo four, the first of each at the island's
  `spawn` marker's spot, 5 to 6 m from its middle toward the town (its
  middle until the markers came), the next seven on a ring 4 m out, all
  facing the town down the
  bridge; the server works out the same), so that the first four seats of a
  town channel, four people, are on four different islands (Hajun: "when
  there is 2 players+ you will not spawn in the same bridge connected
  island"), and the AIs fill the islands round them, eight to each
  (`lobby::TOWN_SPOTS`, 32). Offline you are seat 0: since late on
  2026-11-07 the island at the game's +Z, the bottom of the screen (the
  top of Hajun's in Blender, its +Y), the one they furnished first, arriving on
  the stone slab its `spawn` marker stands on, 5 m from its middle toward
  the town (Hajun: "make the user spawn in the bottom one with the
  furnishes"; the bare island at -Z until then). Coming online, the
  server seats you, and a
  seat on another island than the one you arrived on sends you there as if
  you had fallen (`net::hear`, `Tell::Arrive` with a `spot`, which until
  then never moved you), with the same `Ask::Respawn` after. **This is
  `PROTOCOL_VERSION` 5**: the arrivals, `Ask::Respawn`, and the punch's
  `Ask::Punch` and `Tell::Knocked` (below); **and 6** since that evening,
  the islands at 130 m, which an app with them at 200 m would arrive
  beside, in the void, and have every move put back; **and 7** late that
  evening, the first island the furnished one at +Z, its arrival at its
  `spawn` marker; **and 8** at the day's end, every island's arrival at
  its own marker. The server was deployed with each (2026-11-07; the
  binaries before kept on the VM as `roundtown-server.prev-2026-11-06`,
  `.prev-2026-11-07`, `.prev-2026-11-07b` and `.prev-2026-11-07c`).
- **The punch knocks whoever it lands on back** (`start_punch`,
  `punch_lands`, `Shove`, `Shove::knock` in `src/lib.rs`, the numbers in
  `assets/ui.ron`'s `melee`, `ui::Melee`, Minecraft 1.8.9's throughout, its
  blocks a tick made metres a second): a press of the fist, or F, lands on
  the nearest body in front of you within `reach`, 3 m from your middle to
  their side, and `cone`, 60 degrees all told, no higher or lower than a
  body (Hajun: "if done while both facing the other player/ai and pressed
  when within range"; Minecraft's three blocks from the eyes to the hitbox
  and the crosshair on it), at the press, as Minecraft lands a click, and
  there is no cooldown on the punch itself, as 1.8.9 had none. The swing
  is 0.12 s out and 0.18 back (`swing_out`, `swing_back`: its six ticks;
  0.07 and 0.11 until then). Whoever it lands on is thrown up at `knock_up`
  (8 m/s, 0.4 blocks a tick; their rise halved first and the whole no more
  than that) and carried away from you at `knock_back` (8 m/s, 0.4) and
  `sprint_bonus` more (10 m/s, 0.5, its sprint hit) if you were on the
  move (`Intent::walk` half pushed), what already carried them halved
  first; the carry dies away by `air_drag` (0.91) each twentieth of a
  second in the air and `ground_drag` (0.546: 0.6 friction times 0.91) on
  the ground, in `Motion::step`, through walls and bodies as a walk goes,
  without turning them, and off any edge, `strands` or no, which is what
  knocks an AI into the void. One punch standing still carries a body
  about 3 m; on the move, about 7. Once a punch has landed on a body no
  other moves it for `hurt_time`, 0.5 s (its ten ticks of hurt resistance),
  so at most two knocks a second however fast the presses come. An AI, or
  anyone this device moves, is knocked on the spot. Someone online is
  told through the server (`Hit`, `net::relay_hits`, `Ask::Punch { who,
  running }`; the server checks the two are within 6.5 m where each last
  said, `PUNCH_REACH` in `server/src/players.rs`, and tells the other
  device `Tell::Knocked { by, from, running }`), and their own device
  knocks their body, whose moves then show everyone; nothing is predicted
  on yours. A punch on an AI online is this device's own, as the AIs are.
  `Melee::check` refuses a swing of no time, a cone wider than all round,
  or a drag over 1, as `Car::check` does. Seen in the desktop build:
  unit-tested (the knock, its dying away, whom a punch lands on, the fall);
  not yet by hand or on a phone.
- **The punch's range is drawn round you, as Brawl Stars draws an
  attack's** (`src/range.rs`, since 2026-11-07; Hajun: "right now there
  is no hit range indicator for the melee. i want to make it similar to
  brawlstars - a default of 1/8 circular sector hit indicator with white
  outline and less white area that is both semi transparent, projected
  only for the client's view - which revolves around the player depending
  on player facing direction"): a flat band lying on the ground in front
  of you, turned the way you face, from `melee.near` out to `melee.reach`
  (0.45 m, your own body's radius, so that it begins at your edge; and
  2 m: Hajun, late that evening, "make the range 2m instead of 1m"; 1 m
  for an hour, after "decrease the reach to 1m"; 3 m, Minecraft's,
  through the day), `melee.width` across where it begins (1.5 m, my
  number, for "make it much more wider near the player") and opening out
  by `melee.cone` from there (45 degrees, an eighth of a circle; 60 until
  that day): the sector of a circle whose sides meet behind you, where an
  arc that wide at that angle has its middle (`range::Sector::of`, 1.5 m
  behind your middle as the file stands); with a `width` of 0, one coming
  to a point `near` in front of you. It is the very range `punch_lands`
  lands a punch in: a body is hit when any of it is within the band, its
  middle within the cone from where the sides meet and no further than
  its own radius past the far arc or short of the near one
  (`Sector::holds`). For its first hour it was a wedge from a point at
  your middle (Hajun: "its not a pointy corner near the player so the hit
  region need to be in front of the player not connected to it. the round
  edge near the player need to be wider"), then a ring sector about your
  middle from `near`, 0.8 m for a minute ("the gap between player and the
  hit indicator is too much now") and then 0.45. A white outline 0.1 m
  wide runs just inside its edge (`range::WIDTH`) over a fainter white
  fill, both see-through, in `colours.melee` of `assets/ui.ron`
  (`outline`, `fill`; my alphas, 0.6 and 0.17 since Hajun asked for
  "slightly more transparent" late that evening, 0.7 and 0.22 before),
  taken again as the file changes, the shape too. Yours alone, offline and online: nothing of it
  goes over the network, nobody else's is drawn, nor an AI's; put away in
  the car. It lies as high over your feet as a step (`range::HEIGHT`,
  `LAND_STEP_UP`), so that under the camera over you the paving beside a
  road or the strip of grass never hides it; the pond's rim does. Unlit,
  drawn two and a half times its colours (`range::GAIN`, as the
  fountain's water is), blended, so `see_through` never takes it for
  something in the way, and so needing the camera's `NoIndirectDrawing`
  on the Adreno, as the sea does. Seen in the desktop build, scripted,
  from over you at 24 and 10 m, facing three ways, and unit-tested (the
  band is the punch's own, arcs, sides and width, the outline inside it,
  every triangle facing up); not yet by hand or on a phone.
- **The camera is fixed straight down over you, in and out of the car**
  (`OrbitCamera::top_down`, the resource as the app opens; Hajun: "make the
  camera top down for even the player mode"): the file's `car.overhead` up
  (24 m) with the top of the screen north (the game's -Z, Blender's +Y),
  the same wherever you are, so that the map is always the same way up;
  it zooms between `overhead_nearest` and `overhead_furthest`, and nothing
  turns it: not a finger, the mouse, the stick, a double tap, a bus, nor
  getting into or out of the car (`drive::toggle` keeps no view to go back
  to). The stick's up is the top of the screen (`camera_ahead`), and you
  face the way you walk. Your eyes, the orbit behind you and everything in
  "The camera stands where you zoomed it" below are still in the code
  behind `OrbitCamera::overhead`, and come back by not starting it over
  you. On desktop the cursor is free from the start, there being nothing
  to look round with (`lock_cursor`, `grab_cursor`: Escape only ever lets
  it go), for the stick, the buttons and the town; `RT_FREE_CURSOR` is
  moot. Talking to someone still comes in to their face and back.
- **The buses drive the outer half of the ring round the pond, round and
  round** (Hajun: "the bus now just goes around the inner circle"): the
  town's only roads are the ring's two halves, 44 to 56 m out and 59 to
  71, each a ring nothing comes into, and `roads::network` makes such a
  ring one stretch from a join at its first point back to it, with its
  own join, never one place with another's (the two halves' first points
  are 15 m apart on one axis, and as one place the bus crossed between
  them, a figure of eight). Of the two the planner takes the longer, the
  outer half, on its outside lane, 67.9 m out, keeping right, which is
  anticlockwise seen from above with north up: 426 m, 71 s with its stops
  short of two of the four crossings, four buses a quarter apart. A test
  keeps it so
  (`in_the_pvp_town_the_buses_go_round_the_ring_round_the_pond_and_nowhere_else`).
  Seen in the log of the desktop build; not yet watched.
- **No houses, so no House Builder in the town** (`build::town` finds
  none and says so), and nobody to talk to but the AIs. The town's name
  still shows as the app opens.

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

## Friends, groups and messages

Since 2026-11-13 (Hajun, with four sketches of the panel at the root of the
repo, `Friendtabdesign1.png` and `friendtabdesign2.png` to `4`: "I want to
add a friend system. First every account need a random number between
1000~9999 that is gona serve as a #1234 tag for identification when there
is duplicate name and your friend need to know which one u are, and
whenever u change ur name to sth, it will first check if there is someone
else that has your #1234 tag is also that same name, and so that there can
be same names, but never BOTH same name AND same number. The friend panel
popup will open when u tap the top right corner chat bubblt icon under the
gift icon ... you should be able to search the username, and it will give u
a list of everyone with that username and u can choose which one to add,
and when u add the other person, will get a small 1 badge next to the chat
icon, and the "message" tab inside the panel, andthey can accept or reject
the add friend request. and when youre in the group with ur friend, only
the leader can queue for a minigame, and everyone get queued at the same
time in the same team (unless group is >2 and ur playing 2v2v2v2 bridge
wars, then it will need to be 2 ppl same team 1 ppl other team i guess).
and inviting to group should also land in the same panel's messaging icon
tab with the small 1 circular badge in gold background as a subscript of
the chat icon. and they can accept or reject the group invite too"; and of
the sketches, "just the concept + the general vibe/colorscheme/design
vibe. you can adjust the actual layout and add functions that are not
shown in the design if u think its necessary. but atleast have all the
features that are shown in the designs"). What the two sides say is
`roundtown_net::social` (in `shared/src/lib.rs`: `Ask::Social(Act)`,
`Tell::Social(News)`, the last of each, and `Refusal::NotLeader`); what the
server keeps and does is `server/src/social.rs`; the panel is
`src/friends.rs`. **`PROTOCOL_VERSION` 15**: a `Tell` an app from before
cannot read, so the server went out with the app: **deployed at 14:07 UTC
on 2026-11-13** (23:07 here), with Bridge Wars' dynamite of the same day
("Games"), nobody having been on it for three and a half hours; **APK 26
is the one that plays online**, and an APK from before is told to update.
The binary before is kept on the VM as `roundtown-server.prev-2026-11-13c`
and a dump of the database as it was before tags as
`/var/backups/roundtown/before-tags-2026-11-13.dump`, out of the nightly
backups' rotation.

- **Everything about friends is the server's, and there are none offline**
  (the panel says "You are offline. Friends need a connection."). The
  server holds all of it in memory (`social::Social`): what every account
  is called and its tag, who is whose friend, every inbox, every group. It
  is read from the database once, as the server starts
  (`accounts::Snapshot`, out of `accounts::set_up`), and from then on the
  game's loop goes by its own copy, so nothing about friends waits on a
  query; whatever is to outlast the server is written down after the fact,
  in the order it happened (`accounts::Keep`, `Bank::keep`, `keep_it`,
  through the accounts' thread). Without a database (`RT_DB` unset, a
  server on this PC) it all lasts as long as the server runs. `Social`
  knows nothing of the world: it takes what is asked (`Social::act`) and
  where everyone is (`saw`, `left`, `tick`), and says what is to be told
  and to whom and what is to be kept (`told`, `kept`), which three small
  systems carry out (`hear`; `track`, four times a second; `flush`), so
  all of it is unit-tested with no server running. **A device is told
  everything of its account's as it comes** (`Social::greet`: its tag and
  level, its friends, its inbox, its group), and nothing is kept on the
  device: it forgets all of it the moment it is offline
  (`friends::forget_offline`). That has to hold for a device let in while
  the server still thinks the one before it connected, a phone's app put
  away and opened again within the few seconds a dead connection takes
  to be given up: `players::admit` says so (`Social::connected`), or the
  panel came back empty until something changed.
- **The tag** is `players.tag`, `#1000` to `#9999`
  (`social::TAG_FIRST`, `TAG_LAST`). No two accounts have both the same
  name, whatever its capitals, and the same tag: the server's copy keeps to
  it (`Social::enroll`, `rename`, `taken`), and the table has a unique
  index on `(lower(name), tag)` (`players_name_tag`, `SCHEMA_AFTER`),
  which holds the login to it as well. A new account is given a tag nobody
  of its generated name has (`login::account_of`, trying again up to 16
  times should the index refuse it); every account from before was given
  one the first time the server started with tags (`accounts::give_tags`).
  **Taking a name that someone with your tag already has gives you another
  tag**, at random, that nobody of that name has (`Social::rename`, from
  `Ask::Profile`): mine. Hajun asked that the two never both be the same;
  the other way is to refuse the name, which would make a name something
  that can be taken. The device is told its tag and level whenever either
  changes (`News::Me`), and so are its friends (`News::Friend`).
- **The tables** (`accounts::SCHEMA`, made as the server starts, each
  statement leaving what is there alone): `players.tag` and `players.xp`;
  `friends (a, b)`, the lesser number first; `friend_requests (sender,
  receiver)`; `messages (id, sender, receiver, body)`. Every one goes with
  its account (`ON DELETE CASCADE`): the login's `/delete` takes them with
  the row, and tells the game's loop which account has gone
  (`login::Setup::gone`, `social::forget_deleted`), so that its own copy
  lets go of it too. **The server from before runs on the tables as they
  are now** (its own INSERT and UPDATE of `players` tried on a migrated
  copy: a row made without a tag is allowed), so going back to a
  `.prev` binary needs nothing undone in the database; a rename that
  lands on someone's name and tag would be refused by the index there,
  and that is all.
- **A level** is what an account has earned playing (`players.xp`,
  `Social::award`, `social::level`): 20 for playing a game to its end
  (`XP_PLAYED`), 50 more on the team that wins Bridge Wars (`XP_WON`), 50,
  30 and 15 more for House Builder's first three places (`XP_PLACED`),
  and a level for every 100 (`XP_A_LEVEL`), from 1. All mine: the sketch
  has "Level 53" by each name and nothing says what a level is. A game
  skipped to its end (`RT_ALLOW_SKIP`) earns as it pays its prizes.
- **The panel** (`src/friends.rs`; every colour and word of it
  `assets/ui.ron`'s `colours.friends` and `words.friends`): the speech
  bubble under the gift (`hud::FriendsButton`, the "social" button that did
  nothing) opens it, a dialog (`hud::DialogUp`: you stand still, and a
  press on it is its own; the cross or Escape shuts it), dark and
  see-through as sketched, 112 by 66 shares of the short side, which is
  the sketch's own size on a phone and fits a 4:3 tablet (a test keeps it
  inside both). Along its top two tabs and the cross. It is drawn again
  whole whenever what it shows
  changes (`redraw`, as the settings' window is), keeping what is typed
  (`signup::NameBox`) and how far each list is scrolled (`Scrolled`); a
  list scrolls under a finger or the mouse held (`drag_list`), and a
  press dragged more than 16 px is no tap on the button it began on
  (`TAP_SLOP`).
  - **A press anywhere off the panel shuts it as it goes down, and is
    whatever it landed on's as well** (since the night of 2026-11-13,
    Hajun: "it should close just from the DOWN. and also when i for
    example touch the joystick as the way of closing it or jump button,
    it should register the joystick tap/drag or the jump button rather
    than needing to press twice"; for its first evening a tap, down and
    up, on the dark round it shut it, and that tap was the panel's
    alone). The panel's root is over the whole screen, takes no press for
    itself (`TakesPress` is on the panel and on what comes up over it)
    and lets what is under it be pressed (`Pickable` that blocks nothing):
    `press_outside` hears a press that lands on it through picking and
    puts the panel away, so a thumb on a button under it presses that
    button too, the gamepad opening the games with the one tap; and
    `shut_under_a_press`, ordered before `read_player_input`, puts it
    away in the very frame a press lands on nothing that takes one, so
    that the stick has that touch and a press on the jump button jumps
    (a jump is read only in the frame its press lands, and you stand
    still under a dialog). The speech bubble itself, pressed with the
    panel up, shuts it and does not open it again with the same press
    (`Pressed::shut_it`).
  - **Your tag is after your name on its plate, top left**
    (`hud::YourTag`, which `friends::show_tag` writes as the server says
    it; `hud::show_name`: "Hajooon#5571" online, your name alone
    offline), in the town and in Bridge Wars alike, and nothing of your
    own is in the panel's top; **your own level is shown nowhere but on
    your row of a group's list** (Hajun, the same night: "i dont need
    that text there. just put that number in the main Ui top left corner
    name tag ... Your own Level doesnt need to be shown anywhere for
    now"; for its first evening the panel's top had your name, tag and
    level beside the cross). The cross is 8 shares across (`CLOSE`; 7
    until "increase size of the close button slightly") and has no disc
    come up behind it under the pointer or a finger, as the panel's other
    bare buttons have ("it should just close it no hover/click
    background"; `bare`'s `lit`).
  - **Friends**, the first tab: your group, if you are in one ("Group
    (2/4):", Leave, each member with "(Leader)" and "(You)", and for the
    leader a cross that puts a member out); then "Friends (1/2):", how
    many are here of how many, with the person and plus that opens Add
    Friend; and a row to each friend, whoever is here first and then by
    name: a disc with a dot, green here and grey gone, the name, where
    they are ("In Main Lobby #1", the channel of the town; "At home";
    "Bridge Wars: In Queue" and "In Game"; "Offline for 15 hours", drawn
    again each minute), "Level 53", a button that asks them into your
    group (the sketch's question first, "Would you like to invite ... to
    your group?", Cancel and Yes; faint when they cannot be asked: gone,
    in your group already, or you in a group you do not lead), and a
    gear, under which Message and Remove Friend (which asks first).
  - **Add Friend**: a name typed into the sign-up's box
    (`NameBox::open_for`), with "Friend's Username" standing faint in it
    until the first letter (`Hint`, `show_hint`, `words.friends.
    search_hint`, `colours.friends.hint`; a line under the box said to
    type a name and tap the glass, until Hajun had it put in the box);
    the glass, or Enter, looks for it (`Act::Find`):
    everyone of that name, whatever its capitals, and whoever has that
    name and tag if it ends in one (`social::split_handle`), up to 20,
    whoever is here first and then whoever was here last, never yourself.
    Beside each, "Add friend" (`Act::Befriend`), or "Requested",
    "Friends", or "Accept" for one who has asked you. Under them "My ID"
    and your own name and tag, the tag fainter, with a button that copies
    it. **Until something has been looked for, "My ID" stands right under
    the search bar**: the list of who was found is as many rows tall as
    it has to show, none before a search, one for "Nobody goes by that
    name.", and up to three, past which it scrolls (`found_rows`; Hajun:
    "there should be barely any gap between the search bar and the "My
    ID" box. that gap only appears after you press enter/click the search
    icon"; three rows' room was kept from the start until then). **On a
    phone the box, like the one a message is written in, slides up over
    the keyboard only until its own top is at the top of the screen**
    (`signup::LiftsForKeyboard::whole`, `lifting_whole`; Hajun: "the Add
    Friend popup's top edge should touch the top of the screen when
    keyboard is up. right now the top area is hidden by going over the
    border"), so its title and the way back out of it stay in sight and
    whatever does not fit is under the keyboard; the sign-up's name page
    and the settings' Edit name still slide as far as keeps the box typed
    into on the screen, their headings going off the top.
  - **The envelope**, the second tab, is your inbox, newest first: who
    wants to be your friend (Accept, Reject), who invites you to their
    group (Join, Reject), and what your friends have written (Reply, OK);
    with nothing in it, "All Caught Up." (and an empty list of friends
    says "No friends."; both Hajun's words, that night).
    How many wait there is counted in gold on the envelope's corner and
    on the speech bubble's among the buttons (`hang_badge`, `count_badge`;
    "9+" past nine). A request and a message wait for someone who is not
    here, and are there when they next come.
  - What came of something asked is said in a line along the panel's
    bottom for 3.5 s ("Friend request sent.", `Outcome`, `words.friends.
    said`); what happens to your group while the panel may be shut is said
    across the lower screen (`toast`).
- **A message** (`Act::Write`) is up to 80 characters of what the game's
  keyboard types (`social::message`, `MESSAGE_LONGEST`), to a friend, here
  or not; it waits in their inbox until they press OK, 30 at the most
  (`LETTERS_MOST`), and is kept in the database until then. Mine: the
  sketch's gear has "Message" under it and nothing more is said of it.
  **It is the first thing one player writes that another reads**: friends
  only, no filter, no block and no report, which a shop's review may ask
  for (Apple does of user-generated content) before a release to
  strangers. Hajun's call.
- **A group** (`Act::Invite`, `Answer`, `LeaveGroup`, `Kick`; `News::Group`)
  has a leader, whoever first asked someone into it, and up to four in it
  (`GROUP_MOST`: two teams of Bridge Wars; mine). Being asked stands for
  ten minutes and only while whoever asked is here (`INVITE_SECS`); saying
  yes to one group leaves the one you were in; the leader leaving, the
  first of the rest who is here leads; one alone is no group. Whoever's
  connection goes is kept in their group for two minutes (`GROUP_GRACE`: a
  phone closes its connection whenever the app is put in the background).
  Groups are never written down: the server starting again ends them all.
- **Only a group's leader starts a game.** Anyone else in a group finds
  the game selector's cards blocked out over "Only your group's leader can
  start a game." (`games`, `Friends::follows`), and the server refuses
  them (`Refusal::NotLeader`). The leader choosing a game takes with them
  everyone of the group who is free to go: connected, and in the town or
  in a game that is over for them, and, for House Builder, with its 100
  coins, each paying their own (`Social::party`; `Ask::Arena` in
  `server/src/arena.rs`, `Ask::Play` in `server/src/builder.rs`). Whoever
  is not is told "Your group started a game without you." and stays in
  the group. **In Bridge Wars they are seated two by two onto a team**
  (`Game::seats_for`, `seat_together`): two on one team; of three, two on
  one team and the third on the island across the map, as Hajun has it;
  four, two teams; in the game filling now if it has the seats for all of
  them, and in a new one if not (`Arenas::filling`). In House Builder they
  are all in one game (`Games::filling`). Whoever is taken along has the
  panel put away and "Your group is playing Bridge Wars" across the
  screen.
- **Copying your name and tag**: on desktop to the system's clipboard
  (Bevy's `system_clipboard` feature, on for desktop alone in
  `Cargo.toml`). On Android the game cannot reach the clipboard, so it
  keeps what is to be copied until `MainActivity` has taken it, asked
  every 300 ms by numbers alone (`clipLength`, `clipByte`, `clipTaken`,
  exported from `friends` as the keyboard's `imeInset` is from `signup`:
  nothing of Java's is touched from the game's side, which is what
  crashed the first build of the keyboard's lift). An iPhone has no copy
  button: the name and tag are there to read.
- **What is mine, Hajun having drawn the panel and said the rules above**:
  another tag rather than a refused name; what a level is; messages, their
  80 characters and 30 waiting; the 100 friends and 50 unanswered
  requests an account may have (`FRIENDS_MOST`, `REQUESTS_MOST`); four to
  a group, its ten minutes and two; that whoever cannot go is left behind
  rather than holding the group back; that a request turned down says
  nothing to whoever asked, who may ask again; "9+"; no copy button on an
  iPhone; and, of the night's changes, that a press off the panel is a
  button's under it as well as the stick's and the jump button's, that
  the list of who was found is as tall as what it shows rather than three
  rows once anything has been looked for, and that your own row of a
  group's list still has your level, as every row of it has.
- **Not built**: blocking or reporting someone; anything said to you while
  the app is shut (a request waits for when you next open it); a group's
  own chat; Hangul in a name (below, "The font is Inter").
- **Checked on 2026-11-13.** Unit-tested: the rules both sides share (56
  in `shared`), the server's `Social` and a group's seats in Bridge Wars
  (45 in `server`, nineteen of them new for this), the panel's own (276
  in the game). **The database, on the VM's own Postgres, against scratch
  databases made and dropped for it** (the server's own never written
  to, its accounts' names, looks and coins hashed before and after): two
  ignored tests that will run only on a database whose name ends in
  `_scratch` (`RT_TEST_DB`; `accounts::tests::scratch`), three times
  each: everything `social` asks to be kept written and read back, the
  login making forty accounts, each with a tag nobody of its name has,
  and **a copy of the server's own accounts taken through starting up**,
  each given its tag with nothing else of it changed. **End to end with a
  real Postgres**: the new server on this PC against a scratch database
  on the VM (reached through SSH as the VM's own `roundtown` user, with
  nothing opened there and nothing left), three scripted desktop copies:
  accounts from before given their tags as it started, two requests, one
  answered, two messages, one seen off, a group taken into a game of
  Bridge Wars and its 20 earned; the server stopped and started; and
  every tag, the friendship, the request still unanswered and the
  message not yet seen off were as they had been, in the table and on
  the three screens, the request then answered. That run found accounts
  numbered past the
  middle of what the table counts read back as one account (ids clamped
  at nought where everything else casts them: `Snapshot::read`, and
  `keep_it` ordering two friends other than the table does), which only
  a server run without the login can have, and which is mended. **In the
  desktop build with a server on this PC, scripted, six runs of two or
  three copies**, one of them by touches on the buttons alone and two
  more in part, each run again as the code changed: a search finding two
  players of one name, one gone; asked, accepted, rejected and asked
  again; the gold count on both corners; a group of three with its
  leader, a member put out, one leaving; a message written, read,
  answered and seen off, seven of them scrolled with a finger, a drag
  begun on a button pressing nothing; the leader taking three into
  Bridge Wars, two on one team and one on another; a follower refused,
  and the selector blocked for them; a group into House Builder, each
  paying; a friend gone and back, with what was written meanwhile
  waiting; and a copy killed outright and opened again at once, which
  the server took for never having left ("is back"), with its friend and
  its message there again. **Not by hand, and not on a phone**: the keyboard's lift for
  the two boxes typed into, the copy on Android, and how the panel sits
  under a thumb are all unseen. The desktop's copy was read back off the
  system's clipboard after a scripted press of the button ("Copied").
  With a phone's keyboard written in by the script, over the lower 55% of
  the window, both boxes slid up clear of it: Add Friend with its box,
  its results and "My ID" all in sight and its title off the top, and the
  message's box with Cancel and Send.
- **As it went out.** The server's own tables were given the columns, the
  three tables and the index as the new server first started, and its
  three accounts their tags ("3 accounts were given their tags"), with
  every account's name, look and coins as they were (hashed before and
  after). This PC's own account was then let in through the login, shown
  its tag and level in the panel, told "Nobody goes by that name." of a
  name nobody has, and written down as seen when it left. **Nothing
  between two accounts has been done on the VM itself**: no request, no
  group and no message, there being one account to try it with and no new
  ones made for it; that is the tests', the scratch databases' and the
  runs' on this PC alone. APK 26 is built, its 85 asset files checked
  byte for byte, its library the stripped one with the four entry points
  `MainActivity` calls (`imeInset`, `clipLength`, `clipByte`,
  `clipTaken`), and its number read back, and **not installed, no phone
  being attached** (APK 25 is kept beside it as
  `donggeurami_town-release.prev-2026-11-13c.apk`).
- **The changes Hajun asked for later that night, having tried APK 26**
  (said where each is, above: the tag on the plate and the panel's bare
  top, the bigger cross with no disc behind it, "No friends." and "All
  Caught Up.", Add Friend's hint and its "My ID" right under the search
  bar, the boxes' tops kept on the screen over a keyboard, and a press off
  the panel shutting it as it lands and passing on) **are not in APK 26**:
  the app's own, nothing of the server's or of what is sent, so they went
  out with the next build, **APK 27**, in the small hours of 2026-11-14,
  with Bridge Wars' changes of that night ("Games"; built by the session
  that made those, and not installed, no phone being attached). Seen in
  the desktop build, scripted, against a
  server on this PC and offline: the tag on the plate online and the name
  alone offline; Add Friend with its hint, one row for nobody found and
  two for two; with a phone's keyboard written in over the lower 55% of
  the window, the short box whole above it and the taller one with its
  top at the top of the screen; a finger put down off the panel shutting
  it before the finger came up, where the stick is, where the jump button
  is, on the gamepad (the games up with that one tap) and on the speech
  bubble (shut, and not opened again); and a tap on the panel leaving it
  up. **That the stick and the jump button have that touch is the code's
  and a test's of the frame's order, not a run's**: on desktop they go by
  the mouse, which a script cannot hold down there, and a phone has not
  been tried.

## The font is Inter

Since 2026-11-13 (Hajun: "I want to change the font of most things to the
font "Inter" Regular"), every word on the screen is written in Inter
Regular (`src/font.rs`), in place of the font Bevy comes with, FiraMono, a
fixed-width font cut down to plain ASCII.

- **Nothing that draws words names a font.** Each takes Bevy's default
  (`TextFont::default()`), which is whatever font sits under the default
  handle; `font::plugin` puts Inter there as the app is built
  (`AssetId::default()`, after `DefaultPlugins`, whose text plugin puts
  its own there), so every word is Inter's from the first frame, the
  opening's cover included, with nothing loaded later to swap in. So
  "most things" came to everything: the bar, the dialogs, the shops, the
  names over heads, the headlines, Bridge Wars' numbers. Should one thing
  want another font, it names it in its own `TextFont`.
- **The file is compiled in** (`include_bytes!`), as `assets/ui.ron` is:
  `assets/fonts/Inter-Regular.ttf`, 411,640 bytes, the static Regular of
  Inter 4.1 (`extras/ttf/Inter-Regular.ttf` of the release at
  github.com/rsms/inter), under the SIL Open Font License, which is
  beside it (`LICENSE-inter.txt`). A phone needs nothing packaged for it:
  both files are left out of the APK (`assets-left-out.txt`; a test
  keeps it so, and that the font built in is Inter's Regular).
- **Every text asks for fixed-width digits** (`font::sized`, `tnum`,
  which every text's `TextFont` is made by: `hud::words`, `builder::label`
  and the rest). Inter's digits are of different widths, a 1 narrower
  than an 8, where FiraMono's were all one: a clock or a count would
  shift a hair as its digits changed, the whole line with it where it is
  centred. A test keeps every text asking.
- **No Hangul.** Inter has Latin, Greek and Cyrillic, and the signs
  FiraMono's subset had not (an ellipsis, a middle dot, dashes, arrows).
  Names are plain ASCII still (`roundtown_net::profile`), though Hajun's
  sketches of the friends' panel have "동그라미<3" for a name: a font with
  Hangul in Inter's own shapes for the rest (Pretendard is one, under the
  same licence, several times Inter's 0.4 MB a weight) and a name's
  rule that lets Hangul through, with a phone's keyboard composing it,
  are what that takes. Not built; Hajun's call.
- Inter is proportional and a little narrower than FiraMono was at one
  size, so every line is shorter than it was; nothing was resized for it.
  Seen in the desktop build, scripted, on the screens that day's runs went
  through: the bar, the sign-up's three pages, the settings, the game
  selector, House Builder's lobby, Bridge Wars' board, counts, buttons
  and both its shops' windows, and the friends' panel. Not opened in it:
  House Builder's shop, stars and results, and what is said when you talk
  to someone. Not on a phone.

## Playing online

Since 2026-10-08 the game plays online, against a server in Seoul (a $10
Vultr VM). `MULTIPLAYER.md` is the plan and the why, `SERVER.md` how the
server is run. The rest of this file holds online too: being online changes
where things come from, not what they are.

- **The app never waits for the network.** It opens on the offline town as
  it always has; `src/net.rs` connects in the background, IPv6 first, then
  IPv4. Once the server has let you in, whoever else is in your channel of
  the town (16 to a channel) walks into it, and one of this device's AIs
  leaves for each, keeping the town at 32 bodies
  (`lobby::fill_the_town`; Hajun's choice, 2026-10-08; eight until
  2026-11-01). Each device runs its
  own AIs, so two players see them in different places. With no server to be
  had it is the offline game exactly, and `net` keeps trying, less and less
  often. Never mid-game offline: a game of House Builder started offline is
  finished offline.
- **Three crates.** The game is the root; `shared/` (`roundtown_net`) is
  everything the two sides say, registered in one order by
  `roundtown_net::protocol`; `server/` is the server, a headless Bevy app
  with replicon and renet over UDP 443. **Deploy the server with every change
  to `shared/`**: an app whose protocol differs from the server's is told "A
  new version is out" and stays offline. replicon's protocol hash is only the
  names of the types registered and their order (read in its source,
  2026-10-08), so it misses a field added to one or a field meaning
  something new: bump `PROTOCOL_VERSION` for those, or an old app connects
  and breaks. A change that leaves what is sent as it was keeps the apps
  already on phones playing online, as the prizes did.
- **Your body is yours.** It is moved here, at once, as offline; where it is
  goes to the server 15 times a second while it moves and not at all while
  it stands (`net::tell_where_i_am`). The server checks a move could have been
  made (no faster than a run, no higher than a jump) and passes it on.
  Everyone else is drawn two updates behind, smoothed between them
  (`net::smooth`). **Players walk through each other** (Hajun's choice): a
  remote body has no collider; AIs and walls still bump. Riding one of the
  town's buses, where you are goes as a place on the bus, in its own space
  (`Spot::aboard`, since `PROTOCOL_VERSION` 2), and everyone else draws you
  on the bus as they see it (below, "The town's buses").
- **One clock for the whole town** (`src/clock.rs`, since 2026-10-24,
  `PROTOCOL_VERSION` 3). Everything that goes by the clock goes by the
  town's clock: the buses round the roads, the fountain's jet and the day.
  Offline it is the device's own time of day; online it is the server's.
  Until then each device went by its own, and phones keep theirs to the
  network's only roughly: on Hajun's two phones the cars were in different
  places, and a rider, drawn on the car as each phone had it, jumped onto it
  on the other phone and back off it (Hajun: "i want the car location to be
  synced too as well as the fountain ... in a very network cost efficient
  way"). As a device comes online it asks the server the time eight times,
  0.1 s apart, and then once a minute (`net::ask_the_time`, `WhatTime`); the
  server answers at once (`server/src/clock.rs`, `TheTime`, sent
  independent of replication, which would hold it to the next tick); and
  the answer that came back quickest sets the clock, put forward by half
  the time it took (`TownClock::heard`). Between answers it counts on the
  device's steady clock (`Instant`), which nothing puts forward or back; the
  app going to the background goes back to the time of day, put out by as
  much as the server's was ahead of it, until it is heard again. That is
  one number, and 8 bytes there and 16 back an answer. The first answer
  logs how far the device's own clock was out. Checked with two
  copies on this PC and a server on it, one copy's clock put 5 s out
  (`RT_CLOCK_OFF=5`): it logged "-5.000 s on from this device's time of day,
  to within 8 ms", and both copies' town clocks were within 3.2 ms of the
  machine's, their buses within a centimetre of each other. Not yet tried on
  phones.
- **Getting in.** The device makes up a secret the first time it runs
  (`device.id`, beside `town.json`), sends it over HTTPS to the login on the
  VM (`src/login.rs`, TCP 443, the server's own certificate pinned by its
  SHA-256, `login::PINNED`: no domain), and gets a signed connect token. A new
  device is a new guest account in Postgres with a generated name ("Mochi
  482"), since 2026-11-13 a tag after it that nobody of that name has
  ("Friends, groups and messages", above), 1,000 coins, and the blue look;
  the name and the character chosen
  on signing up replace those as soon as it is let in ("The first time, you
  sign up"). **The VM's addresses and certificate are compiled into the app**
  (`net::SERVERS`, `login::PINNED`): a new VM means a new app.
- **House Builder online is the server's** (`server/src/builder.rs`): the
  queue (10 s for people, then the computer one seat at a time; Hajun's
  choice), the stages and their clock, the fee and every prize (each in a
  Postgres transaction, with its line in `coin_changes`), the stars, the
  places, and the results' 15 s, after which it sends whoever is left back
  to the town. The device follows its `Round` and `Seats`
  (`builder::follow_round`) and is moved by it. The places are not sent:
  both sides work them out from the stars with `roundtown_net::standings`,
  the lot being the game's `Round::theme`, so that the apps already out
  could keep playing online when the prizes came in. Pieces put down go to the
  server as they are (`build::report_pieces`), and everyone else's are built
  from what it says (`build::raise_shadows`, `build::Shadow`), on their plots,
  for the visits. The computer's players stand where they arrive: the server
  has no islands to walk them on. A dropped connection mid-game waits 15 s to
  come back (`builder::RECONNECT_WAIT`) before the game is given up.
- **Names over heads** (`src/tags.rs`): everyone else's online, AIs
  included, and never your own (since the evening of 2026-10-31, Hajun:
  "remove the in world player name for the player's own character"; until
  then yours was always over you, online or not: it is on its plate top
  left). Each is a line of UI text laid out once (3.6% of the short side
  of the screen; 6% for an hour, which Hajun found "too big, especially
  your own name": "reduce by more than 50%", so 2.8, then, with their own
  gone, "slightly bigger", 3.2, and on 2026-11-01 "slightly increase"
  again), and scaled down by
  how far its head is from the camera, as words standing in the world would
  be: full size 8 m off and nearer, half at 16, a tenth at 80 (Hajun,
  2026-10-31: "like a in the world text rather than a on screen text so
  further away the text look small naturally and closer look larger"; until
  then one size, 3.2%, at every distance). Read
  out to 75 m (25 until then; Hajun: "like 3x"), which is further than the
  scaling leaves one readable. **Town
  editing is off** while online (`map::hands_off_online`): nobody else would
  see it. **Your account** is under Settings (below, "Settings"), where
  Delete account is, which Apple asks for even for accounts nobody signed
  up for. Until 2026-10-31 it was a round button beside the balance, online
  only (`src/account.rs`, now `src/settings.rs`).
- **Testing on desktop.** Two copies side by side need their own `APPDATA`
  (their own `device.id`, so their own accounts) and `RT_FREE_CURSOR=1`, so
  that neither takes the mouse. `RT_SERVER=<address>[,<address>]` points a
  desktop build at another server, `RT_SERVER=off` keeps it offline, and in a
  debug build `RT_UNSECURE=1` skips the login, for a server run without
  `RT_KEY`. `RT_TIME_OF_DAY=night` (or `day`, `sunset`, `sunrise`, or
  seconds into the twenty minutes) stops the day at one time, and
  `RT_CAR_AT=<seconds>` the town's buses at one point of their loop.
  `RT_ARENA_FROM=<seconds>` begins a game of Bridge Wars' fight that far
  into its timetable (the server for its games, a desktop build for one
  played offline), and `RT_ARENA_PURSE=<count>` with that many of each
  thing in every person's purse, for trying the shops out.
  `RT_CLOCK_OFF=<seconds>` puts a copy's own clock that far out, for
  trying the town's clock out on one machine, whose clock both copies
  share. `RT_INTRO=off` leaves out the opening's cover, which otherwise
  holds every press for the first second or two of a run (above, "The app
  opens under a cover"). A
  server on this PC for two copies to meet on: `cargo build -p
  roundtown_server`, then `target/debug/roundtown-server.exe` with
  `RT_PORT=5443` and no `RT_KEY`, and the copies with
  `RT_SERVER=127.0.0.1:5443 RT_UNSECURE=1`. The VM allows
  the tap on the time that skips House Builder's waits
  (`RT_ALLOW_SKIP=1`): take that out of `/etc/roundtown/env` before real
  players come.

## The islands are Blender models

`assets/roundtownmap.glb` is the town (Hajun's, 2026-10-30: "i made a new map
in replace of circlemap"; `circlemap1.glb`, the old town, stays in the repo
and out of the APK), `assets/testhomemap.glb` your home, and
`assets/lobbymap.glb` House Builder's lobby. Each is a Blender model (`.blend`
beside it), not code; `island::Venue` names the files. Since 2026-09-27 your
home is the plain `homemap.glb` with a 3 m high room (a doorway facing where
you arrive) and a 6 m wall to your right, for trying the camera against
ceilings and walls; House Builder's plots are still the plain `homemap.glb`. `src/island.rs` reads
every triangle back out of each spawned scene, and that is what players stand
on and bump into — no bounds are kept by hand anywhere. Reshape the land,
re-export, and the collision follows.

- **All three are loaded at startup and all stand at the origin.** A game of
  House Builder adds eight more, `Venue::Plot(0..8)`, a copy of `homemap.glb`
  for each builder, and takes them away when it ends. What keeps them all apart
  is `island::Venue`, carried by everything that moves or collides — players
  and town objects alike. Bodies only collide with what shares their venue,
  and only the island you are on is drawn (`show_where_i_am`). Anything added
  to the world without a `Venue` is drawn everywhere and left out of collision
  altogether.
- **A shape is read once per model** (`island::Islands`), so the eight plots
  share one.
- **Everyone arrives on a ring round the middle**, seat by seat, facing in
  (`island::arrival`, through `roundtown_net::arrival`, which the server
  goes by too): in the town 31.5 m out (`TOWN_ARRIVAL_RADIUS`), on the path
  round its pond, and 6 m out on the grass of the home, the lobby and the
  plots. 6 m out in the new town is in the pond, 0.4 m deep, inside a rim a
  jump gets out over and the computer's players never would.
- **Someone modelled into a map is not land.** Skinned meshes are left out of
  the shape: they are drawn where their skeleton puts them, not where their
  transform does, so read as land they stood somewhere else entirely. (The
  House Builder read that way was an invisible wall 23 m out from the
  fountain, on the road to them.) Give such a person a collider in code, as
  `builder::settle_house_builder` does.
- **The sea is everywhere the model has nothing underfoot,** at `WATER_Y` — z = -0.5
  in Blender. The land's flat top is z = 0 (`LAND_TOP`); the lowest ground on
  any island, the home's sand, is z = -0.2 (the new town's roads are at 0, its
  paving 0.25 over them), so keep anything walkable above about -0.25 or it
  counts as wading. Each island has
  its own sheet of it drawn (`sky::sea`), hidden and shown along with the island.
- **Water inside a map is a Blender object with a material called `water`**
  (`island::WATER_MATERIAL`), like `Water_fountain_water` in the town's fountain
  basin. The game draws it with the sea's own material, whatever it looks like
  in Blender, and leaves it out of collision, so players stand on the bottom,
  not the surface. The fountain's is 0.45 m up, over a floor at 0.05 and under
  a rim at 0.55: from inside, the rim looks a hand's width over the water, but
  it is half a metre over your feet. Since 2026-10-08 (Hajun asked for water
  in the "second and third smaller circle" too) the tower's two bowls hold
  water as well, `Water_fountain_water_middle` at 0.915 m and
  `Water_fountain_water_top` at 1.44 m, each 1 cm under its brim, with the
  tower's own 32 corners so its edge lies on the bowl's wall. They were added
  to `circlemap1.blend` by script and exported with the exporter's defaults,
  which reproduce Hajun's export of the town byte for byte (checked first);
  nothing else in the file changed. In the new town the fountain stands in
  the middle of a pond 60 m across (`water`, 0.5 m up over a floor at 0.1,
  inside a rim 0.7 m high), and its bowls' water came over from the old
  town as `water.001`, as Blender names a copy: a material called as
  another is, with a dot and a number after, is that one
  (`island::is_material`), for water, road, paint and bulbs alike.
- **Fixed in `roundtownmap.blend` on 2026-10-30** (my doing, before it was
  played): `Road`, the road under all the town's paving, and
  `peoplepathoutsideoffountain`, the path round the pond, were inside out,
  their tops facing down, which the game falls through; their faces were
  turned round (no triangle moved, checked over 3,000 points). The road's
  paint (`Circle.001`-`.004`, `Plane`-`Plane.004`) had no material, which
  glTF draws white and wholly metallic, near black, and the buses' way round
  is read off: it was given `roadwhite`, the old town's (0.86 grey). The
  water tower's legs and the booth under it, on the island east of the town
  (`Cylinder.001`, `Cube.001`), had none either: they were given
  `default_grey`, Blender's own look (0.8 grey, rough 0.5), as they showed in
  Blender. A default export of Hajun's file reproduced their glb byte for
  byte first; the export after differs in those materials and those faces'
  winding alone. Saved without a `.blend1` of mine, so that theirs is theirs.
- **The town's second version, later on 2026-10-30**, is Hajun's
  `roundtownmapbackup.blend` (untracked), the file in which they keep their
  modifiers (Hajun: "i dont want to apply the modifers in that file"):
  roundabouts with 8 m of road round their islands rather than 6, white
  lines down the edges of the roads, 12 street lamps, 10 fewer trees (14),
  and its houses grey with blue or pink roofs. At their asking it was copied
  over `roundtownmap.blend` and every modifier applied there, each object
  checked triangle for triangle against what its modifiers made; the
  boolean cutters (`Road Cutters`) deleted, as Hajun deleted them from the
  first version, or a default export has them as walls; the edge lines down
  the straight roads (`Plane.005`), which had no material, given `roadwhite`
  in `roundtownmap.blend` only (the backup was open in Blender); and a
  linked node group nothing used dropped. A default export of it is the
  backup's export with Apply Modifiers and Visible Objects ticked, node for
  node, but for that material: 49,584 triangles (33,696 before), 20,160 of
  them the lamps. The fixes above came over in the backup.
- **The town's third version, that evening**, made the same way from the
  backup (Hajun saved at 18:14), and the same check passes: 20 street lamps,
  each an object of its own (`streetlam`, `.001` to `.019`): a pair either
  side of each of the four gaps in the ring's strip of grass and of each of
  its corners, 72 m out, and four 43 m out, inside the ring, in line with
  the straight roads; 4 trees
  (14); the strip of grass down the middle of the ring (`Circle`) 0.25 m
  high rather than 0.5, under a bus's underside, and the planters 0.5 rather
  than 0.75; and the road under the town (`Road`) and round the roundabouts'
  islands (`Circle.007`) made slabs 2 m deep by a Solidify, the sides of the
  town's edge going down into the sea. 58,736 triangles, 33,600 of them the
  lamps. Their top and bottom are cut into triangles each its own way, which
  found a flaw in how a vertical line was tried against a triangle: its
  slack was a share of the triangle, and half a millimetre outside the
  road's edge a line met its bottom and missed its top, and was taken for
  inside it (a bus test failed there). The slack is a distance since
  (`island::EDGE`, 0.1 mm). Edge lines `Plane.005` still have no material
  in the backup.
- **What the old town taught, for any town**: a part with no material loads
  near black; keep a lamp an object of its own (the old town's 23 came joined
  three to an object, and were split by script, while whatever hid you faded:
  a lamp in the way faded its whole object; since nothing in the town fades
  (2026-10-25), the new town's 12 are one object as Hajun made them); keep a
  sign's font simple
  (Calibri Bold was 1,500 triangles a letter, Blender's own a few hundred);
  and the exporter leaves modifiers off by default (the old town's outer
  road dashes came as one dash and a Geometry Nodes Array): apply them, or
  tick Apply Modifiers when exporting.
- **Normals must point out of the land** (Mesh > Normals > Recalculate Outside).
  Up and down are read from the winding, so a top turned inside out is fallen
  through, even though the double-sided material hides it on screen.
- Faces steeper than 50° are walls, not floor. Anything thinner than 0.25 m can
  be walked through. Players wait where they arrive until their island has
  loaded.
- **A body steps up anything under 0.5 m** (`LAND_STEP_UP` in `src/lib.rs`,
  0.495; it was 0.75 until 2026-09-30, when Hajun made it 0.5). Half a metre
  itself, and anything higher, is a jump: later that day the fountain's rim,
  exactly 0.5 m over the floor of its basin, could be stepped out of, and
  Hajun wanted a jump. It is also where the band a body is tested against
  walls starts, so what stands between it and the head is in the way.
  Nothing else walkable in any map is exactly 0.5 m up from its
  neighbour (checked: only the fountain tower's tiers and some furniture
  tops, like the bookshelf's rows). An AI cannot jump, so it never steps
  down anything it could not step back up (`strands`, and the walks it
  picks in `ai::dry_all_the_way`): it keeps out of the fountain's basin.
- **A body is the shape of the model** (since 2026-10-08; of the old model,
  `limbperson.glb`: `playercharacter.glb`, since 2026-11-01, is narrower,
  0.35 m across the legs and 0.45 the shoulders, and bumps as wide as
  before): its legs, 0.42 m
  across (`LEG_RADIUS`), up to its hips at 0.65 m, then rounded out to its
  body and arms, 0.9 m across (`PLAYER_RADIUS`), from 0.8 m up to its head
  (`resolve_player_solids`, through `Island::walls_round`, which looks down
  each square of the lattice once for all three widths). Until then it was
  the full width from half a metre up, and the fountain's middle bowl could
  not be stood on: the top bowl, at the thighs of anyone on it, shoved them
  off into the basin.
- **A body stands on whatever is under its legs** (`support`): the ground
  under its middle, unless the ground under its legs, round that, is higher
  by more than a slope makes it (`EDGE_RISE`, 0.15 m), and then it is
  standing on an edge. Until 2026-10-08 only the middle counted: coming down
  with the legs on an edge and the middle just past it, a body sank past
  the edge, half into it, and was shoved out sideways once it reached its
  knees (Hajun: it "glitch steps", "any time" onto an edge). Missing an
  edge by more than the legs, the wider body above them is still eased off
  it, 0.12 m at most in a frame. Standing on the ground under its middle,
  as it nearly always is, a body does not look under its legs at all.
- **A solid has to be a closed shape.** Bodies are tested by dropping a
  vertical line through every triangle (`Island::blocked`), and where the
  surfaces it meets do not add up, the space is taken for the inside of
  something. A part left open — a gable with no bottom over a doorway — fills
  the doorway. Model roofs, gables and posts as closed shells.
  On a plot, each piece down on it is counted on its own, and only at the
  heights it reaches (since 2026-10-03). Before that the whole plot was
  counted as one, and a piece that is not a closed shape (most furniture: a
  blanket that is only a sheet, the cafe chair) threw out the count for
  everything over or under it. A chair downstairs put invisible walls on
  the upper floor over it, and a bed upstairs filled the room under it: in
  Hajun's two floor house the upper floor was sticky to drag pieces over,
  with spots nothing could go and players stuck. The islands themselves are
  still counted whole, but for the walls the game makes again round holes,
  each counted on its own, on a plot and in the town, and the town's window
  panes (since 2026-10-30). Counted with the rest of its house, the wall
  under the Square House's front gable was open to walk through: the gable
  is inside out, its underside facing up at the very height of the top of
  the wall, and two surfaces facing the same way at one height are taken
  for one, met on the seam between two of its triangles. Every town house
  had one wall a body walked out through (Hajun: "make sure player cannot
  go through the transparent house walls"); a test keeps it so
  (`a_wall_counted_on_its_own_is_solid_under_a_gable_inside_out`), and
  walking eight ways out of a town house in the desktop build, only the
  doorway lets you out.
- Anything baked into a map is scenery: the tap menu cannot move it. Only
  objects from the town save below, and House Builder's pieces while you
  build, can be edited.

## The fountain runs

Water wells out of the top of the fountain's tower into its top bowl, spills
over each bowl's brim into the one below and into the basin, with foam where
it lands; and every 4 s a jet shoots 6 m up out of the top, for a second and
a half (`src/fountain.rs`; Hajun, 2026-10-08; every 8 s until Hajun asked
for 4 on 2026-10-10).

- **Stand in the top bowl when the jet comes up and it throws you** 9 m up
  over the bowl, out of the top of the jet (`THROW_HEIGHT`; Hajun asked for
  more than the jet's own 4.5 m it was at first), with `throw`, which sets
  your jump's speed: it is a jump, as far as everything else is concerned,
  and so does standing on its
  brim or on its edge with the legs. Getting up there takes a running jump
  from the basin's rim, or one from the middle bowl, which can be stood on
  between 0.7 and 0.95 m out from the middle: any nearer and the legs are on
  the top bowl's rim, and stand there. Only while the jet goes up and stays
  up, less time than a throw takes to come down, so nobody is thrown twice
  by one jet. AIs never get up there.
- **The jet goes by the town's clock** (`jet_phase`, the seconds since 1970
  modulo 4 s, the server's online: "Playing online"), not by the game's
  time, so every device's goes up together and everyone online sees the
  same jet throw the same player. Until 2026-10-24 it went by each device's
  own clock, and two phones whose clocks were out went up at different
  moments. A throw is the thrower's
  own device's, like a jump, and the server takes it as one: it rises over
  twice as fast as a jump, inside the server's slack for one, which allows
  a throw up to about 13 m (`MOST_RISE` and `BURST` in
  `server/src/players.rs`, which a test there keeps in step).
- **Everything is read off the model** once the town has spawned
  (`Fountain::read`): the tower, by its name `Water_fountain_tower`, for its
  axis and its top; every piece of `water` on its axis for the pools (in the
  new town the bowls' `water.001` and, lowest, the whole pond); and for
  each bowl, the widest the tower is within 12 cm of its water for the brim
  the water spills over. Move or reshape the fountain in Blender, and the
  water follows; take its water out, and nothing runs.
- **The running water is see-through sheets turned round the axis** (`lathe`),
  a few thousand triangles in eight draws, drawn with `FlowingWater`: unlit,
  a few broad, soft white streaks on pale blue, a gentle swell passing down
  each, which the GPU moves along by the frame's time, so nothing about them
  is written from one frame to the next. A sheet has as many streaks as fit
  round it 0.28 m apart (`STREAK`; the foam's patches 0.45 m): until Hajun
  found them "tooooo detailed" for the rest of the town (2026-10-08), every
  sheet had 40 fine ones, and the foam was a ring of dots. Only the jet's
  two parts are, while it is up (`spout`). See-through things are drawn
  furthest first by the middle of their bounds, which is why foam floats 6 mm
  over its pool: so that it comes after it.

## The town has music

Since 2026-10-30 (Hajun: "add the background music loop Pelican Breeze in
the sounds folder"), in `src/sound.rs`, from `assets/sounds/`. Bevy's `mp3`
feature decodes it (`Cargo.toml`); it decodes only Ogg Vorbis by default.
**Off for now** (since 2026-11-02, Hajun: "just disable the music from
playing temporarily"): `sound::PLAYING` is false, so `play_music` starts
nothing; everything else, the module, Bevy's audio plugin and `mp3`
feature, the file in the APK and the test that decodes it, stays as it is,
to be turned on again by flipping it.

- **The music plays everywhere, and again from the start whenever it
  ends.** It is played once and started over (`play_again`), not looped:
  Bevy loops a sound by keeping every sample of it as it plays, 56 MB for
  this one's 158 s.
- **It is paused while the app is in the background** (`AppLifecycle`), as
  a phone would play it on otherwise.
- At -30 dBFS on average (`MUSIC_VOLUME`, 0.25): at first -24, my guess,
  and Hajun found it too loud. `cargo test` decodes it, as the game does,
  and fails if the APK would leave it out.
- **The buses had an engine for an evening**, the same day (Hajun: "add
  bus_engine_loop sound effect on the bus"), heard from the camera (a
  `SpatialListener` on it), from the side of you each bus was on, louder
  the nearer, and paused anywhere but the town, where every island stands
  at the origin. Hajun found it too quiet, then had it taken out: "remove
  engine sound just keep music". `assets/sounds/bus_engine_loop.wav` stays,
  left out of the APK; Bevy's `wav` feature is still on, decoding nothing.

## The town's buses

(For the day of 2026-11-07 the town was the PvP map, whose only roads are
the two halves of the ring round the pond: the buses went round the outer
half, which `roads::network` makes one stretch of a ring nothing comes
into, from a join at its first point back to it, with its own join, never
one place with another's; the two halves' first points are 15 m apart on
one axis, and as one place the bus crossed between them, a figure of
eight. A test keeps it so on that map,
`in_the_pvp_town_the_buses_go_round_the_ring_round_the_pond_and_nowhere_else`,
though no bus drives it now: the buses are the town's, `car`, and Bridge
Wars' map has none.)

Four buses drive round the town's roads by themselves, one behind the other
round the same loop, a quarter of it apart (two, half apart, until
2026-11-01: Hajun, "4 buses in the map rather than 2"), and anyone can get
on any of them and ride it. The cars
came first (Hajun, 2026-10-23: "the car that automatically goes around in
the circlemap1 ... it goes on every part of the road ... both lane 1 and
lane 2"; "dont hardcode the moving logic", as the model is to get seats
and windows; and later that day "put both car and car2 on the road at
different points on the same path"); on 2026-10-24 Hajun had them swapped
for buses ("remove the car and car2 from the circlemap and put 2 bus
instead"). All are their bus, `assets/mybuilds/bus.glb`. **`car::CARS` is
the cars, and each goes by its place in it**, 0 to 3: four of the bus. The
code still calls them cars (`src/car.rs`, `src/roads.rs`, `Spot::aboard`,
`RT_CAR_AT`). The first car, `car.glb`, a red slab with its seats sunk into
it, and the Roadster, `car2.glb`, are still in `assets/mybuilds/` and left
out of the APK (`assets-left-out.txt`); the APK test keeps whatever is in
`CARS` in. Every device online has to have the same cars in the same order,
as a rider is sent as a place on one, by its number. Neither is sold in the
shop. Nothing about the buses is in the code but their speeds: their shape
is their model's, and their way round is the town's roads.

- **The bus** is 12.5 m long, 3.7 m wide (wider than a real bus, 2.55 m)
  and 3.3 m to its roof, 4.1 m with what stands on it: a red coach with a
  windscreen and headlights at the front, windows down both sides, and two
  doorways on its right, the kerb side, each with a step, into a hollow
  inside whose floor is 0.45 to 0.49 m over the road, just under a step
  (0.495), so that you walk in. Its body is one object, `Cube`, a box made
  a shell by a Solidify modifier, its sides about 5 cm thick, and its four
  wheels another, `wheels`, one wheel and a Mirror and an Array. **Since
  Hajun's save of 2026-10-25 every modifier is applied in `bus.blend`**,
  so an export with the defaults has all of it. That night Hajun raised the
  body 0.1 m (its floor was 0.33 m over the road, its sides 1.5 cm thick),
  and three exports came without the Solidify, the first with one wheel
  too: with no Solidify the bus is one surface facing out, its floor facing
  down, nothing stands in it and the bus tests fail. **Its roof is
  two faces 1.6 cm apart**, the top facing up and the ceiling under it
  facing down, and it needs both: later that night the ceiling was deleted,
  and the inside of the bus counted as solid, shoving anyone in it out
  through a doorway; then the top was flipped to face down, and what landed
  on the roof fell through it. Hajun's save of 01:44 is their 01:11 one
  again, which passes every test. Run `cargo test --lib` after exporting it.
- **Fixed in `bus.blend` on 2026-10-24** (my doing, at the swap): Hajun
  modelled it facing Blender's +X, so the game would have driven it
  backwards. It was turned a quarter round about the origin to face -Y,
  the front every model faces, its reference pictures with it; its wheel
  object, `Cylinder`, was named `wheels`, which the game turns; and it was
  exported with Apply Modifiers, as Hajun's export of that day had one
  wheel (3,556 triangles with all four). Nothing else in the file changed,
  and its `.blend1` is Hajun's save from before. Should a modifier come back
  on it, export it with Apply Modifiers ticked, or the game gets one wheel
  again.
- **A bus is the shape of its model**, every triangle of it, read once it
  has spawned (`car::Hull`) and moved about with it as an island of its own:
  bodies stand on it and bump into it as they do the island (`Deck`,
  `Ground` in `src/lib.rs`). **Each object in Blender is one solid, all its
  materials together**: the glTF loader makes a mesh of each material, and
  the first car's red alone, without the dark grey of its seats, was a box
  with no top over them, which `Island::blocked` took for the inside of
  something and shoved everyone out of. Keep each solid a closed shell, as
  anywhere. **The bus's walls are a few centimetres thick, and still keep
  you in and out** (`keep_off_deck_walls`, since 2026-10-24): the lattice that keeps
  bodies out of walls only stops what is 0.25 m thick or more ("The islands
  are Blender models"), and you walked out of the bus through its side, and
  in the same way (Hajun: "i think its cuz the bus's walls are rly thin").
  So a body walking on or near a car has its middle, as wide as it is, at
  waist to head height, swept against every triangle of the car in the
  car's own space, and slides along what it meets, the bus's sides, front
  and back being square to its two ways across the ground. Its doorways,
  about 1.2 m wide on both sides, near the front and in the middle, are the
  way in and out, a step up from the road onto its floor; thickening the
  walls to the lattice's 0.25 m would have left them too narrow for a body
  0.9 m across. Only a body's own walking is swept: standing in the bus's
  way, it scoops you up onto its floor rather than shoving you aside, as
  the cars did with their sides over a step high, and an AI it drives into
  is scooped up the same, and rides until it walks to a doorway. Checked
  on the model in a test that fails without the sweep
  (`the_bus_s_thin_walls_keep_you_in_and_out_but_its_doorways_let_you_through`);
  not yet tried by hand.
- **Its front is along its longer side**, toward Blender's -Y, Blender's
  front, as the bus and the Roadster face and every other model does, or
  toward -X, as the first car's hood is (`car::front`). Its wheels' bottoms
  stand on the road.
- **Its wheels turn**: every part of an object with `wheel` in its name,
  but a steering wheel (`car::is_wheel`), is broken into its loose pieces,
  those about one middle taken as one wheel, and each turned about its own
  middle as far as the bus has gone over its radius (`split_wheels`,
  `turn_wheels`). The wheels as modelled are hidden, and copies of them,
  one to each, turn.
- **Its front wheels keep to the middle of its lane, and its back wheels
  follow them as a car's do** (`roads::trailing`, since 2026-10-30): each
  step the front ones take, the back ones are pulled after them along the
  way the bus faces, and never sideways, so that round a bend they keep
  inside it, and the bus turns about them as a bus does. Its body faces from
  its back wheels to its front ones (`roads::Footprint`, `Route::on_wheels`,
  `car::place_at`); the loop holds where its back wheels are as its front
  ones pass each point, worked out going round it twice. The bus's front
  wheels are 3.8 m ahead of its middle and its back ones 2.8 m behind
  (`axles`, read off where its wheels are). From 2026-10-24 to 2026-10-30
  both keep to the line, as a train's to its rails, and round a tight bend
  the bus slid sideways, up to 17°; Hajun, 2026-10-30, asked for "wide turns
  in tight corner so the turning animation is not weird this time". Before
  that a car turned about its middle.
- **The way round is read off the town** (`roads`): a road is where the top
  of the town is `road`, its middle the dashes painted down it in
  `roadwhite` (`roads::ROAD`, `MARKING`). A dash is a line, no wider than a
  metre: the stripes of the new town's crossings, 1.6 m wide and lying along
  the road like dashes, two of them on the very middle of the ring round the
  pond, are not (`WIDEST_DASH`). Each dash in line with the next, up to 9
  dash lengths on (`GAP`: the ring's dashes stop 12 m either side of each
  crossing), makes a road's middle, round or open, and a gap in it is filled
  round the circle its dashes are on (`roads::bridge`): across a crossing
  the line through the dashes alone sagged 0.4 m. An open middle runs on
  past its last dash for as long as there is road (`ACROSS`, 40 m), across
  any road it meets to the furthest it reaches: the new town's straight
  roads run across the outer half of the ring round the pond, through the
  gap in the strip of grass between its halves, to the inner half. Ends
  that run off the road beside each other round an island meet round it, a
  **roundabout** (`roads::roundabout`: the island's edge felt for all the way
  round and a circle drawn through it, and the road round it as far out as
  it goes in a quarter of the ways round): the new town's four, round
  islands 9.1 m across from their middles, the road round them 8 m wide (6
  m in the town's first version). Each road that runs into one goes on to
  the middle of its island round the bend it is on (`roads::on_to`): the
  road round the edge of the town stops its dashes 23 m short of each
  roundabout's middle since the roundabouts grew, and a straight line on
  from its last dash turned 3.4° off its 194 m bend there, which the bus
  coming onto the roundabout took as a kink, 5.75 m round.
  Joins nearer each other than 20 m are one place (`ONE_PLACE`): the ring's
  two halves meet the straight roads 15 m apart, and turns there go from the
  one half to the other. Each road is as wide as the road is either side of
  its middle at its dashes, and roads are cut into stretches where they
  join. Round Town is 16 stretches between 8 joins: the ring's two halves, 4
  stretches each, the 4 straight roads and the road round the edge of the
  town, 4 stretches between its roundabouts. Repaint or move the roads in
  Blender and the buses drive the new ones; a road with no dashes down it is
  not one to them.
- **A road's dashes are evened out before its middle is drawn through
  them** (`roads::even_out`). Round the old town's rings every other one of
  Hajun's dashes was a few centimetres in from the rest, and a line through
  every dash wove in and out by as much: the car on it turned out and in,
  back and forth, all the way round (Hajun, 2026-10-23: "it shouldnt rotate
  outward ever just constantly rotate inward bit by bit"). Each dash is
  moved halfway to the middle of the two either side of it, and back out by
  a little more (Taubin's smoothing), which takes out what goes from side to
  side from dash to dash and leaves a circle the circle it was. Round a ring
  a bus turns one way (a test keeps it so, over a metre and a half at a
  time: the lanes are drawn through points a metre apart and wave by a
  millimetre). A turn leaves and joins a lane along the bend there
  (`heading_at`).
- **They keep to the right**, as Korea drives: two lanes to a stretch, one
  each way, each halfway out from the middle to its edge (on the new town's
  12 m roads, 3 m out). Every turn at every join is worked out first
  (`Route::read`), and the loop drives **every lane the bus can get onto and
  off gently, from and to lanes it can**: of the lanes it can go round from
  any to any other turning gently, the most road (`round_together`), each
  once if that makes one loop (`roads::plan`), never back the way it came
  but at a dead end. In the new town that is every lane but the inner half
  of the ring's, clockwise: the bus would get onto it and off it only
  turning right through a gap in the strip of grass, which at the town's
  sharp corners and 12 m roads it does only swinging round on its back
  wheels. 28 lanes, 4,851 m, 633 s round, each once (the log says so as the
  town loads, with how far behind the first each bus goes and how long
  working it out took). It is worked out away from the frames
  (`car::Planning`, `AsyncComputeTaskPool`): about 0.8 s on the laptop in a
  debug build, which on a phone would have held up a frame for a second or
  more; the buses come onto the road once it is done.
- **Turning gently, swinging wide at a tight corner, all of a bus on the
  road where it can be, and its wheels where it cannot** (`roads::turning`).
  For the cars, Hajun, 2026-10-23: "when its turning a corner into the inner
  circle, it kind of almost steps onto the rock". Every turn is tried as the
  bus would go round it, its front wheels on the line and its back ones
  following, felt for the road every 0.25 m round its edge at every point
  along it, from the lane before, which brings its back wheels to where
  they would be, to the lane after (`LEAD_IN`, `LEAD_OUT`). At an ordinary
  join, round corners: arcs of 8.5 to 19.6 m between the lanes' lines
  (`fillets`, `ROUNDS`, only where they meet the lanes themselves), or from
  the middle of the road it leaves, or into the middle of the one it joins,
  moving across to its lane over 8 m (`CROSS_OVER`), as a bus turns wide;
  and curves from the room left at each lane's end, nearer the join, and up
  to 15 m further back (`corner_shapes`, `FURTHER`). At a roundabout, round
  its island keeping it on the left, as a roundabout is driven where cars
  keep to the right: off the lane round an arc to the right onto the road
  round the island, round it, and off round another arc to the right, each
  arc running on from the last along it (`round_shapes`, `merge`, `arc`),
  round at a few ways out from the island (`ROUND_AT`), on and off round
  arcs of a few sizes (`MERGES`). Of all of them, the first tried that keeps
  all of the bus 0.25 m inside the road's edge (`KERB`); failing that, the
  first that keeps its wheels so, or failing that on the road, its body
  swinging out over nothing higher than its underside (`UNDERSIDE`, 0.4 m:
  over a kerb or paving, and since the town's third version the strip of
  grass down the middle of the ring, 0.25 m high, but not a wall or a
  lamp); and failing even that, the one that takes the least of
  its wheels off it. **Where the first bends tighter than 10.4 m
  (`RELAXED`), the gentlest that does as well: at a tight corner the bus
  swings wide.** **Never a turn tighter than 8.5 m round (`TIGHTEST`, 1.3
  times how far apart its wheels are)**, if there is another that keeps its
  wheels on the road: its back wheels would swing round rather than roll.
  Round a roundabout only its wheels count, and they may run up onto the
  edge of the island by a metre (`APRON`), as a bus's do round a small
  roundabout. In Round Town all of it stays on the road round the
  roundabouts' islands, since Hajun made them bigger (8 m of road round an
  island 18 m across; with 6, its wheels ran up onto the island and its
  body swung out over the sea on the far side of each, by about a metre).
  Its wheels leave the road only coming onto a roundabout and off it: its
  back wheels cut the square corners where the roads meet the road round
  the island, by a metre at the most (0.85 m; 1.05 m with 6), and its body
  swings out over them, at the two on the edge of the town over the sea;
  never into anything taller than the paving (a test keeps it so,
  `round_town_s_buses_swing_into_nothing_and_never_meet`). A 12.5 m bus
  turning right off a 12 m road round a square corner cuts it: rounding
  those corners off in the town, as a real roundabout's are, is what would
  keep its wheels on. Round the ring round the pond and through its gaps it
  stays on the road. The route waits for both buses' models as well as the town's
  (`plan_the_way`) and is worked out for the longest, and a bus is on the
  town, drawn and met, only once the route has put it on the road
  (`drive`).
- **Their speed is worked out from how sharply the loop turns**: at most
  `roundtown_net::cars::TOP_SPEED`, 7 m/s, as fast as a run, so that you can
  keep up beside a bus and no more; no harder round a bend than 2.5 m/s²
  sideways; speeding up at 1.5 m/s² and slowing at 2.5. All my numbers.
  **On an open road, from one roundabout to the next, twice that**, 14 m/s
  (`roads::OPEN_ROAD`, `Lane::speed`; Hajun, 2026-10-30: "make the bus go
  faster when going around the outer circle (like 2x)"): in Round Town the
  road round its edge, and nowhere else. The server lets a rider off as far
  from where they got on as 18.2 m/s takes them (`MOST_RIDE`), so it needed
  no change. **And it stops short of every other crossing it comes to**, its
  front 1.5 m short, waits 3 s and goes on (`STOP_SHORT`, `PAUSE`,
  `Route::waits`; Hajun, the same day: "when in the inner circle, make it
  pause before some of the crosswalks and then go"): a crossing is read off
  the paint as stripes, pieces as wide all along as no line is, side by side
  (`crossings_painted`), and Round Town's are one at each corner of the ring
  round its pond, across both its halves, so the bus meets one on each of
  the 12 lanes of the ring it drives, and stops at 6. Seen in the desktop
  build the same day: 14.4 m/s on the road round the edge and no more than
  7.2 anywhere else, and standing 3 s, 10° to 14° short of a crossing's
  corner, on three lanes of the ring. The loop, its waits and all, is then
  made to take a whole number of seconds.
- **They go by the town's clock** (`place_at`, the seconds since 1970 into
  the loop, less how far behind the first a bus goes, `Route::behind`), as
  the fountain's jet does, and online the town's clock is the server's
  ("Playing online"), so every device has each in the same place, and
  nothing about them goes over the network but the time. Until 2026-10-24
  they went by each device's own clock, and on Hajun's two phones the cars
  were in different places. The loop being a whole number of seconds is
  what keeps devices from drifting apart over the years since 1970 on
  arithmetic that differs in its last digit. They never stop for anybody,
  as a device could not tell the others it had: only short of crossings,
  where they would on every device alike. **Each goes its share of the
  loop behind the one before**, a quarter with four buses, or as near it,
  a second at a time, as keeps every two a metre apart all the way round,
  where the loop crosses itself and passing each other (`Route::spread`),
  or failing that, out of each other: with two buses in the new town, 317
  s behind (half is 316.5), at which they never touched (checked every
  0.1 s round the loop in the test above, which checks every pair of
  however many there are).
  `RT_CAR_AT=<seconds>` stops both at one point of the loop, the first that
  far into it and the second 317 s behind, for trying things out. Where the
  loop starts goes by the order the town's paint is read in, which every
  device reads alike, but a test reading the file itself starts elsewhere.
- **Riding one**: standing on a bus, or in the air off it, a body is carried
  along with it, turning as it turns (`Riding`, `move_bodies`), until it
  comes down on anything else. So a jump on it comes down where it went up,
  and walking off it, you come down on the road with nothing carrying you
  on. Its floor is a step up from the road, and its roof, at 3.3 m, out of
  a jump's reach. **The camera comes round after it** (since 2026-11-01;
  until then nothing turned it but you, and the bus turned under you as the
  island does): riding one, once your fingers, or the mouse, have been off
  the camera for 1.5 s, it eases round behind the way the bus faces,
  standing still too, so that you look the way the bus goes without turning
  the camera yourself as it goes round ("The camera stands where you zoomed
  it").
- **A bus in the way does not fade** (since 2026-10-25, Hajun: "not on the
  bus"): nothing outside a house in House Builder does ("The camera stands
  where you zoomed it"), so riding one with the camera outside, or behind
  one from the camera, you are hidden. From 2026-10-24 it faded to three
  quarters of itself (Hajun, 2026-10-24: "the bus should become
  transparent just not that much as the other ones so slightly higher
  opacity but still transparent", and
  then of half, "increase the bus opacity slightly more";
  `see_through::BUS_SEEN`, 0.75, against a quarter for everything else,
  still in the code, unused, should the town's fading come back). It
  faded whole, wheels and glass with it, whichever part of it hid any of
  you, as a town object did. **Only a quarter, a
  half and three quarters come out smooth**: with alpha to coverage a pixel
  shows it in one, two or three of its four MSAA samples, and anything
  between a GPU makes up, if at all, by using more samples in some pixels
  than in those beside them. Seen the same day in the desktop build, the bus
  stopped by day with you behind it: at a half and at three quarters
  smooth, at 0.65 a checkerboard, neighbouring pixels 16 levels of red
  apart on average (up to 31). Not yet seen on a phone, whose GPU may not
  make up the steps between at all. From 2026-10-23, at Hajun's asking, the
  car never faded at all ("make it so that the car never become
  transparent even if camera blocks the player through the car"), and the
  bus took that over until the first of these.
- **Online, a rider is a place on their bus** (`Spot::aboard`, the bus's
  number): where you are goes to the server in the bus's own space, and
  standing still on it is standing still, sending nothing. Everyone else
  draws a rider where they were on the bus two updates ago, on the bus as
  it is now (`net::drawn`), so that a rider is never left behind it by the
  time an update takes. Getting on and off is eased over 0.25 s
  (`net::GETTING_ON`). The server checks moves on a bus as anywhere, in its
  space, takes a jump from one bus onto the other as getting on, and
  getting off, no further from where you got on than a bus could have
  taken you (`Pace`, `MOST_RIDE`, `cars::REACH` 15 m): it cannot tell where
  they are. Riding was checked with two copies on this PC and a server on it
  (2026-10-23, one car). **This is `PROTOCOL_VERSION` 4** since 2026-10-30
  (2 from 2026-10-23: riding; 3 from 2026-10-24: the town's clock, and a
  place on car 0 being a place on a bus; 4: the new town, where a place is
  somewhere else entirely, and its arrivals round the pond): an app from
  before it is told to update.
- **Not yet tried**: on a phone, or by hand, or riding the bus, nor the new
  town's way round by Hajun. Seen in the desktop build on 2026-10-30 from
  above: round the east roundabout's island and through a gap in the ring's
  strip of grass, smoothly, its back wheels inside its front ones; and on
  the town's second version, from above the north roundabout, a bus coming
  along the road round the edge of the town and taking the first way off
  it, toward the pond, on its right, its tail swinging over the corner
  there. Dropped
  anywhere over it, a body comes to rest on its roof
  (`a_body_dropped_onto_the_car_anywhere_stays_on_it`), and its wheels keep
  to its way round (`the_cars_stand_on_the_road_facing_the_way_they_go`).
  Seen in the desktop build on 2026-10-24, from beside the road by day and
  from above in two copies at once, in the same place in both. The cost is
  not measured: only bodies within a bus's reach of it test its triangles,
  its 3,604 (3,556 until 2026-10-25).

## The car you drive

Since 2026-11-05 (Hajun: "add this car1.glb model in the asset folder into
the game as a user spawnable. so for now, just make the emote button to be
a toggle on/off switch for car. when on, the player model disappears and u
are suddenly now a 'car'"), in `src/drive.rs`: press the face among the
buttons round the jump button, or C on desktop, and your body is gone and
you are Hajun's `car1.glb` (`drive::MODEL`; a red sports car 6.5 m long,
3.1 m wide, 3.84 m between its axles, 2,258 triangles, three textures),
where you stood and facing the way you faced; press it again and you are
on your feet where the car is. The face is lit while you are in it, as
the jump button is held, and the fist shows the jump button's arrow turned
down (`hud::repicture`).

**Every number of how it goes is in `assets/ui.ron`, under `car`**
(`ui::Car`, since 2026-11-06; above, "The screen's colours and words are
in `assets/ui.ron`"; Hajun: "put all the constants related to the car in
the ron file ... i want to modify the numbers by myself while playing on
pc"). The names below are the file's fields in lower case: `cruise`,
`boost`, `reverse`, `accel`, `boost_accel`, `brake`, `tightest`,
`easy_speed`, `steer_rate` and `full_lock_at` (in degrees, 286 a second
and 14.3, where the code had 5 rad/s and 0.25), `creep`, `overhead`,
`overhead_nearest`, `overhead_furthest`, `end_radius`, `end_back`,
`felt_low`, `felt_high` and `stopped_short`, read every frame, so a save
while driving changes the car at once. The values below are what the
file started with; whatever it says now is what the car does. A phone
drives by what the file said when its app was built. The constants of
2026-11-05 are gone from `src/drive.rs` and `src/lib.rs`.

- **The camera is fixed straight down over the car** (Hajun: "the camera
  gets fixed as top down (can still zoom in/out, but no auto follow
  rotate/direction/ no panning. its fixed just follow the car's plane
  position no rotation.)"): `OrbitCamera::overhead` tips it to straight
  down, `OVERHEAD`, 24 m, up, and from then on a finger or the mouse does
  not turn it, the stick does not bring it round, nor a double tap, nor a
  bus (`assist_camera`); it zooms between `OVERHEAD_NEAREST`, 7 m, and
  `OVERHEAD_FURTHEST`, 60, never into your eyes. The top of the screen
  stays the way it was looking along the ground as you got in, and the
  stick's up is that way (`camera_ahead`). Getting out, it goes back to
  exactly where you had turned and zoomed it (`OrbitCamera::back_to`),
  your eyes included.
- **It drives as a car does** (`Drive::go`): its front wheels steer and
  its back wheels follow them, at the radius the steering gives
  (`TIGHTEST`, 4.5 m round at the back wheels, up to `EASY_SPEED`, 8 m/s,
  and wider in proportion from there, 6.75 m cruising and 11.25 boosting),
  never on the spot. The roads are 12 m wide: a U-turn takes 12 m outside
  to outside, and the corners where the straight roads meet the ring are
  taken at cruising speed; boosting, a corner is taken wide (Hajun: "make
  the car turning angle so that it fits the map roads"). The stick says
  which way to go, across the screen, and how hard; the car steers toward
  it the shorter way round, all the way at `FULL_LOCK_AT`, 0.25 rad, off
  it, so that pushed behind the car it turns round in a circle (Hajun:
  "there is a pivot so when ur going forward and then u turn the joystick
  the other way, you will 'turn' like a real car in the closer direction").
  Let go, it brakes to a stop (`BRAKE`, 12 m/s²). Stopped against a wall
  with the stick pushed, it steers as if creeping at `CREEP`, 3 m/s, so
  that it comes round rather than standing with its wheels turned (the
  first version stood).
- **The fist held, or F, is the reverse** (`hud::Held`, held for as long
  as the finger that pressed it is down, wherever it slides): the car
  slows (`BRAKE`) and backs away as fast as it boosts (`REVERSE`, 40 m/s
  since 2026-11-06 with the boost, 20 until then,
  gaining `BOOST_ACCEL`), straight back with the stick let go or held
  where the nose points, and with the stick to one side its nose comes
  round toward the stick as it backs, the wheels turned the other way for
  the same stick, as a car reversing turns them, until it faces the stick
  and backs straight away from it. Since the evening of 2026-11-05 (Hajun:
  "when u press the reverse button without touching joystick, it should do
  a straight reverse backward. also the reverse button should accelerate
  backward same speed as boost ... when reversing, the direction of the
  joystick should be opposite ... it should just deaccellerate and then
  wheels start towards rotating the other way. so straight back. so its
  like real reverse"); for an hour it backed toward where the stick
  pointed at 6 m/s, so that held where you were driving it swung round
  instead of backing straight. **The jump button held, or Space, is the boost** (Hajun:
  "similar to rocket league's car boosting - it just makes ur car faster,
  but u dont need to press this to move it still goes when u just do the
  joystick"): `BOOST`, 40 m/s since 2026-11-06 (Hajun: "change the max
  speed to 40m/s", once the numbers were in the file; 20 until then), and
  the reverse with it, instead of `CRUISE`, 12 (a run and a bus
  are 7, a bus on the open road 14), gaining `BOOST_ACCEL`, 14 m/s², rather
  than `ACCEL`, 8, so about 3 s from standing to 40; and boost alone
  drives it straight on. At 40 the turning circle is 22.5 m
  (`tightest` times 40 over `easy_speed`), wider than a 12 m road: a
  corner at full boost is not to be had, which `easy_speed` or
  `tightest` in the file would change. No jump, no
  punch, and nobody to talk to from it (`talk::choose`). All the numbers
  but the model's are mine.
- **The car is your own body, moved by the car**: `drive::steer`, between
  `read_player_input` and `move_bodies`, writes your `Intent` with the
  car's move (`Intent::pace`, metres a second in place of a walk's, and
  `Intent::face`, the way the car faces, reversing too), so the body bumps
  into the island, the buses and everyone as you do, steps up what you
  step up (a kerb, the strip of grass), and, as an AI does, never goes
  down anything it could not drive back up, into the pond or off the edge
  of the town (`strands`); and after `move_bodies`, `drive::settle` stops
  it: stopped short by what the body met, it loses its speed, and its
  nose, going forward, or its tail, reversing, each felt as a circle 1.2 m
  across between 0.55 and 1.3 m over your feet (`END_RADIUS`, `FELT_LOW`,
  `FELT_HIGH`: over a kerb or a planter, under its roof), is kept out of
  anything taller as the body is kept out of walls (`resolve_walls`), the
  whole car moving back with it and losing its speed by how squarely it
  went in. Then `drive::place` puts the model at your feet, facing the way
  the car does, its wheels rolled as far as it has gone and the front ones
  turned as it steers (`car::hang_wheels`, which the buses' wheels go
  through too, with `Turns::front`). The body is hidden (`follow_camera`),
  and the car's model, with no `Venue`, is drawn wherever you are. Sent to
  another island, you are out of it. **Out of it, your legs are your own
  again**: `read_player_input` clears the pace and the facing every frame,
  and only `steer`, after it, writes them (since the evening of
  2026-11-07; Hajun: "if you leave the car u glitch": until then they
  stayed as the car last wrote them, so out of a moving car you walked at
  its speed facing its way, and out of one standing still you could not
  walk at all, its pace being 0). A test keeps it so
  (`out_of_the_car_your_legs_are_your_own_again`), and a scripted run
  drove, got out and walked at a walk's pace.
- **The gift button held, or G, is the gun** (`drive::shoot`, `drive::fly`,
  `hud::GunButton`, `hud::Held::gun`, since 2026-11-06; Hajun: "make the
  gift icon, while riding a car, so that it shoots small fast spherical
  bullets out of the front of the car while pressed at semifast intervals
  that disappear after a certain defined distance range from initial
  starting point"): in the car, held, it fires a bullet out of the car's
  nose as it goes down and then one every `bullet_every` seconds (0.12 as
  first written), each a ball of `bullet_radius` (0.12 m), a lit yellow,
  going `bullet_speed` (60 m/s) level the way the car faces, from the
  height the nose is felt at, and gone once `bullet_range` (40 m) from
  where it was fired; the four are in `assets/ui.ron`'s `car` with the
  rest, and `Car::check` refuses a `bullet_every` or `bullet_radius` of 0.
  A bullet hits nothing: no collider, no `Venue` (drawn wherever you are),
  nothing over the network, so nobody else sees it. Out of the car the
  gift takes a press and drops it, as before. Seen in the desktop build
  the same day.
- **Nothing of it goes over the network.** Online everyone else sees your
  body go where the car goes, legs walking; nobody sees the car. **The
  server lets every body go as fast as the car boosts**
  (`server/src/players.rs`, `MOST_SPEED`, `roundtown_net::cars::DRIVEN_TOP_SPEED`
  times 1.3: 52 m/s since 2026-11-06, 26 from 2026-11-05, a run times 1.3
  until then): nothing says a body is in a car, and a server from before
  puts a car faster than its limit back where it was, every second or so
  (`Tell::PutBack`). Deployed with each: the top speed of 40 on
  2026-11-06, the earlier binary kept on the VM as
  `roundtown-server.prev-2026-11-05`.
- Seen in the desktop build on 2026-11-05, scripted: from standing at the
  pond's edge with the stick behind, a U-turn in about 2 s and off across
  the paving at 12 m/s; a right turn at speed; boosted with the stick
  behind, a wide turn to 20 m/s; let go, a stop in 1.7 s; reversing (then
  at 6 m/s toward the stick; the reverse as it is now is unit-tested
  only); out again with the camera back. Stopped nose first
  against the pond's rim, the first version could not turn away (every
  move that left the nose in the rim was undone); the push-out and the
  creep are the fix, unit-tested only. Not yet on a phone.

## The town's houses have doors that open, and windows

Every house in the new town is Hajun's Square House, House Builder's own
(`mybuilds/square_house.glb`), put down whole in Blender: 64 as it is and 32
with a pink roof (`plain_*_2`), a few to an object (`House`, `House.001` to
`.004`), and none with a way in. In the game, not the model (Hajun,
2026-10-30: "put doors that open and a few windows in each house in the game
(not in the blend) using the various door models"), each has a door that
swings open and five windows (`src/build/town.rs`).

- **Nothing about where they are is in the code.** Once the town has spawned
  and the Square House has, each house is found by its floor, a box the size
  of the Square House's `Floor_1` (`floors`), and turned the way its roof's
  ridge runs. The Square House's `Wall...` boxes, read off its own model,
  stand on it. The town's own walls there are taken out of what is drawn and
  what is stood on (`walled`: every triangle of the town inside one of a
  house's wall boxes, but a gable's underside on top of it and the floor
  under it), and each is made again as House Builder makes a wall, the boxes
  left round the holes in it (`build::Wall`, `wall_mesh`), in the material
  most of its own walls were. A house that is not a Square House is left
  as it is.
- **The door is in the wall nearest a road** (felt for out from the middle
  of each wall, 60 m at the most), one of Hajun's five (Classic, Classic
  Window, Classic Double, Glass, Glass Double), with a square window either
  side of it; the wall opposite has a wide window, and the other two a
  square one each or a wide one each. Which door, and which windows on the
  sides, goes by a mix of where the house stands, the same on every device
  (`mix`). Each is fitted to its wall as
  House Builder fits one (`Wall::fit`): a door on the floor, a window 22% up
  its storey. The houses beside the straight roads face them; the eight
  nearest the ring round the pond face it.
- **The doors swing open for anyone who comes up to them**, as House
  Builder's do (`build::swing_doors`, the leaves found as House Builder
  finds them, `build::doorway`), and a window's hole is shut to bodies by its
  pane (`build::pane`), as House Builder's are: walk in through a door, never
  a window. Each wall and pane is solid on its own ("A solid has to be a
  closed shape").
- **Standing in one, what of it is between the camera and you goes
  see-through, as in a house in House Builder** (Hajun, 2026-10-30: "when
  the player goes inside one of the houses in roundtownmap it does the
  similar transparent thing as in the build minigame"): a wall in the way,
  with its window frames, glass and door, and the roof and gables with it;
  the roof in the way of a camera looking down at a slant, with the walls on
  the camera's side; from straight above, the roof alone. Only that house:
  the town round it, its other houses, trees and lamps, stays as it is. A
  bus never fades.
- **Each house is its own, as a house in House Builder is**
  (`build::TownHouse`, since 2026-10-30): a root with the town's `Venue`,
  `Rooms` (its floor) and `Bounds`, which `untappable_from_inside` marks
  `YouAreIn` as for a plot's house; its four walls (`Wall`, `TownWall`,
  which `carve_walls` leaves as cut); each wall's window frames under one
  `OnWall`; its door, `OnWall` on its wall; its glass, as one, but wall by
  wall while you are in it (`open_the_house`), to fade with its wall; and
  its roof and gables, the Square House's meshes in the town's own colours
  there, which are taken out of what of the town is drawn, though still
  stood on. The floor stays the town's. A wall, a wall's frames or a
  house's glass the same in many houses is one mesh, drawn together: 148
  meshes for 96 houses; raising them takes about 100 ms on the laptop,
  once. Until then the walls and frames of all the houses were one mesh to
  each material and the roofs the town's own, so one wall faded would have
  faded every house's. About a thousand entities more than then, not
  measured on a phone.
- Seen in the desktop build on 2026-10-30, from the road and inside a house:
  the doors, the windows, the House Builder at theirs; and standing in the
  House Builder's house, from the side, at a slant and from straight above,
  the walls and the roof going see-through as above. Not yet on a phone.

## The camera stands where you zoomed it

(For the day of 2026-11-07 the camera was fixed straight down over you
from the start, in and out of the car, the town being the PvP map; since
that night it is so only in a game of Bridge Wars, `OrbitCamera::top_down`,
"Games", above, and in the car, and everywhere else it is as this section
says.)

There is one view (Hajun, 2026-10-09): the camera orbits you as you turn and
zoom it (`OrbitCamera`, `follow_camera` in `src/lib.rs`), 1.5 to 18 m out
behind you, and zoomed all the way in it is your own eyes. Nothing in the
world moves it: walls, ceilings and trees never bring it in nearer you.
That is Hajun's call (2026-10-09): "at all times u should be able to zoom
out to the current max zoom out even when ur inside a room or covered by
some wall and the camera should not automatically go near the player".
Standing in a house in House Builder, whatever stands between it and you is
drawn see-through instead, and a tap goes through it (`src/see_through.rs`):
zoomed out, the camera is outside the house looking in, and you tap what is
in there with you through its wall. Anywhere else, what stands between hides
you.

- **Only in a house in House Builder, or in one of the town's houses, is
  anything see-through** (Hajun, 2026-10-25: "i want that effect to only
  happen when the player is inside the home in the home build battle game,
  not anywhere else not in circlemap1, not on the bus, not in home"; and
  2026-10-30, of the town's houses: "it does the similar transparent thing
  as in the build minigame"), and in a town house, only that house. In the
  town out of its houses, on or behind one of its buses, at home, in House
  Builder's lobby and on a plot outside its house, a wall, a tree, a lamp
  or a bus between the camera and you stays as it is and hides you, and
  nothing is out of a tap's reach for being in the way. In a house is over one of its floors and under its top
  (`build::YouAreIn`, set in `build::untappable_from_inside`), in a house
  that is down, not a ghost: yours while you build, anyone's on a visit or
  at the results. Stepping out of it, what was faded comes back over 0.15
  s. From 2026-10-09 until then it was so everywhere, and what the rest of
  this section says of the town and its buses is how it was there. A test
  keeps it so (`nothing_is_in_the_way_but_in_a_house_of_house_builder`);
  not yet tried by hand or on a phone.
- **It always points exactly the way you turned it, and stands as far out as
  you zoomed it**: 1.5 to 18 m (`CAMERA_DISTANCE_MIN`, `_MAX`; 2.5 at the
  nearest until 2026-11-01, when Hajun found "the max zoom in is not zoomed
  in enough"; looking up past you it comes no nearer than 0.8 m,
  `CAMERA_LOOK_UP_NEAREST`, which keeps it out of your body; 15 m at most
  until Hajun asked for "a slight bit more", 2026-10-09). It tips down as far
  as straight down (`PITCH_MAX`, a right angle; about 69° until Hajun asked,
  the same day), for a view of the tops of things, a roof fading to show the
  rooms under it. Until 2026-10-09 the island brought it in: a
  ball was swept out from you to where you zoomed it and the camera stood
  where the ball stopped, eased in and out (`CameraFit`), with a screen
  across every doorway to keep it in a house (`build::screen`). All of that
  is gone. Before that, a version slid it along walls and climbed it up them,
  and Hajun found it glitchy (2026-09-27): every bit of a swipe has to turn
  the camera by as much, wherever you are. **Keep both**: nothing in the
  world moves it off where you turned and zoomed it. What turns it for you is
  only what Hajun asked for on 2026-10-31: the stick's recentering and a
  double tap (next), and talking to someone ("You can talk to anyone in the
  town"), and since 2026-11-01 riding a bus (next); none of them while a
  finger is on the camera. A finger dragged
  across a phone's screen turns it a quarter more than the mouse does for
  the same distance (`TOUCH_LOOK_GAIN`, 1.25, since 2026-11-01; Hajun:
  "increase the camera panning ... sensitivity slightly so each drag
  rotates the camera bit more"; the same as the mouse, `LOOK_SENSITIVITY`,
  until then).
- **In the car it is fixed straight down over you** (since 2026-11-05,
  `OrbitCamera::overhead`; above, "The car you drive"): it zooms and
  nothing turns it, and getting out it is back exactly where you had it.
- **On a phone, walking with the stick brings it round behind you, a
  double tap swings it there, and riding a bus brings it round behind the
  way the bus faces** (`assist_camera`; Hajun, 2026-10-31: "Soft
  auto-recenter: when the stick is pushed and the player hasn't touched the
  camera for ~1.5 s, the camera eases back behind the movement direction.
  Double-tap on empty screen = snap camera behind the character"). Once no
  finger has been on the camera, turning or zooming it, for 1.5 s
  (`RECENTER_AFTER`), walking with the stick eases it round behind the way
  you walk: eased in over 0.6 s, then at the angle left a second at full
  speed (`RECENTER_RATE`), more slowly walking slowly, never faster than 46° a
  second (`RECENTER_MOST`). The way you walk is how you moved, not how the
  stick points: walking into a wall, going nowhere, it stays; and what a bus
  carries you is not walking. Walking toward the camera it does not turn: as
  readily as ever up to 100° off straight away from it, and not at all from
  150°. With the stick held to one side you walk round a circle about 17 m
  across as it comes round, which pushing the stick up steers out of. In
  your eyes it never turns. Two taps on nothing (not the stick, the jump
  button, a button or anything the editor takes), the second down within 0.3
  s of the first coming up and within 8% of the short side of it
  (`DoubleTap`), swing it round behind the way you face **as you stand at
  the tap** (`CameraAssist::snap_behind` keeps that yaw, `snap_to` goes to
  it), tipped as the app opens it (`CAMERA_PITCH`), or looking straight
  ahead from your eyes, most of the way in a fifth of a second. Until the
  evening of 2026-10-31 it went behind the way you faced each frame, and
  walking with the stick right after a double tap, you face where the
  camera's ahead and the stick point, so the camera coming round behind
  you turned you with it, and chased you round for ever (Hajun: "the
  camera starts spinning ... it supposed to just move the camera behind
  the user at that exact moment only, not after"); a test keeps the mark
  fixed. The double tap also counts the 1.5 s wait, and its easing in, as
  served: walking with the stick brings the camera round behind where you
  walk as soon as the swing is over (Hajun, 2026-11-01: "make that become
  0s instantly so it starts autofollowing right away"; the taps, being
  fingers on the camera, had started the wait over). **Riding a bus, it
  comes round behind the way the bus faces** (`ride_turn`, since
  2026-11-01; Hajun: "when the player standing still but on the bus, the
  camera auto follow doesnt activate ... make it so riding bus is an
  exception where user doesnt need to move ... camera will auto follow in
  direction of bus too while theyre on the bus so user doesnt need to
  constantly move the camera while riding on a bus to look a particular
  direction according to where the bus is facing"): standing still on it,
  or walking about it, and in your eyes as well, once the camera has been
  left alone 1.5 s, eased in over 0.6 s from then or from getting on,
  whichever came later, at the stick's rate and no faster, and from a
  camera turned to look out of the front of the bus as readily as from one
  beside it (walking at the camera never brings it round; the bus turned,
  you did not walk). The bus turns under you and the camera comes round
  after it: round a bend it is left behind the bus by about as many
  degrees as the bus turns in a second, and catches up once the bend is
  over. Until then nothing turned it on a bus, and looking the way the
  bus went round the ring meant turning the camera all the way round with
  it. A finger on the
  camera stops any of them at once. On desktop no double tap, and the
  stick's only with the mouse on the on-screen stick, the cursor free
  (since 2026-11-06; until then the desktop had no stick): the keys walk
  you without bringing it round, and the mouse is the camera's while the
  cursor is taken; riding a
  bus brings it round on desktop too, once the mouse has been still 1.5 s
  (moving it counts as a finger on the camera, `read_mouse_look`). All the
  numbers but the 1.5 s are mine. Seen in the
  desktop build with the stick written in by a script, walking one way across
  the town with the camera off to the side: it stood still for 1.5 s, then
  came round behind in about 3 s. The double tap and the bus are
  unit-tested only; the recentering has not been tried on a phone, and the
  spin's fix not yet.
- **In a house, whatever hides any of you from it fades to a quarter of itself**
  (`SEEN`, 0.25, as much as Roblox's Invisicam leaves; one of the town's
  buses, until 2026-10-25, to three quarters, `BUS_SEEN`, above) over 0.15 s, and
  back once it hides none of you (`FADE_SECS`): "even little bit of the
  player is covered, that wall becomes invisible" (Hajun, 2026-10-13).
  Every frame the straight lines from the camera to 73 points on you
  (`RINGS`: eight rings from 0.2 m up to your head, each as wide and deep
  as the model is there, a little inside it, fitted to `playercharacter.glb`
  since 2026-11-01, its middle and eight points
  round it, and the top of your head) are tried against everything drawn on
  your island: the box round all the lines first, then the box round each
  part, then its triangles, read once per mesh into boxes inside boxes
  (`island::Faces`) and again whenever the mesh changes, like a wall a door
  has just cut. From 2026-10-09 until then it was three points down your
  middle, at 1.15, 1.4 and 1.65 m (`SEEN_AT`): a wall that hid your side or
  your legs stayed, and Hajun saw one fade only once it covered your
  middle. The rings are inside what you bump into things with
  (`LEG_RADIUS`, `PLAYER_RADIUS`) and higher than any edge your legs stand
  over (`EDGE_RISE`), so a wall you stand against, from your side of it,
  and the step you stand on are never in the way; a bed you stand behind
  now is. It costs, in an optimized build on the laptop (release-android),
  11 to 15 µs a frame in the open, 28 to 37 next to a house of the town and
  40 to 48 while one is faded (measured 2026-10-13; 5 to 13 in a debug build
  with the three points). Since 2026-10-25, out of a house of House Builder
  it tries no lines at all.
- **A faded part is one even pane, however its parts overlap.** It is drawn
  with a copy of its own material that shows only its near side, with
  alpha to coverage (`see_through::see_through`): a quarter of it covers
  one of the four MSAA samples of a pixel and writes its depth there, so
  wherever two faded faces overlap only the nearer shows. Blended, as it
  was until 2026-10-13, each overlapping face added a layer, and Hajun
  found "the edges of the models get revealed sometime in a slightly dirty
  way": compared in a scripted run the same day, a faded Square House's
  roof showed three shades where its panels overlapped, and the faded door
  sat on its faded wall with its handles; with alpha to coverage, one
  layer. The same day, faded parts drawn not at all were tried first, and
  Hajun found that "a bit too much". Glass and water stay blended, and so
  does anything with a texture, which alpha to coverage would show all or
  nothing (`RENDERING.md`): the Soft Couch.
- **What fades:** a piece of furniture, a door, a window, a town object or
  a bus fades whole; a house and the island a part at a time, each as modelled in
  Blender: a wall, the roof, a floor, a tree's trunk or its leaves. Whatever
  is in a part fades with it, so that no door stands solid in a wall you see
  through (`see_through::inside_of`): of a house or the island, the rest of it that
  is inside the box round the part, like the doors, windows, handles and
  sign of a house in the town, which are objects of their own, or the water
  in the fountain's basin; and the doors, windows and hangings in a wall in
  House Builder (`build::OnWall`). Glass and water fade from as see-through
  as they are. Bodies never fade (skinned meshes are left out), nor
  ghosts or the sky, nor glass or water for being in the way (but for the glass of
  a door or a window in a wall in House Builder: below), nor a piece being
  put down, nor whatever the edit menu is open on. **Nor the ground**
  (`see_through::ground`): the island's own parts no higher than a step
  over the land, like the town's paving, roads and grass. What the ground
  hides of you is your feet below a kerb or your legs in the sea, and the
  town's paving is one piece 140 m across. Seen through a doorway in the
  town, the door fades and the wall round it does not; in House Builder the
  wall fades too (below).
- **Standing in a house in House Builder, any of its walls in the way takes
  its whole ceiling with it** (Hajun, 2026-10-16): everything of the house
  wholly over your head (`see_through::over_your_head`), its roof and
  gables, and downstairs the floor upstairs, fades with the wall and comes
  back with it, so the room you are in is open to the camera from above as
  well. The house you stand in carries `build::YouAreIn`, set where it is
  made untappable from inside. A wall of a house you are outside of takes
  no ceiling.
- **And the ceiling, when the camera looks down into the room at a slant,
  takes the walls on the camera's side** (`see_through::on_the_camera_side`;
  Hajun, 2026-10-16): standing in a house with its ceiling see-through,
  every wall of it whose way in is within 60° of the way the camera looks
  (`ON_THE_CAMERA_SIDE`) fades too, so the room is open the way the camera
  looks in: looking square on, the near wall; across a corner, the two near
  walls. Looking down steeper than 75° (`STRAIGHT_DOWN_DEG`), the ceiling
  goes alone; looking level or up, only a wall in the way goes. Both angles
  are my reading of "slightly angled down" and "rly top down". Checked in a
  scripted run the same day, standing in the middle of a Square House: 40°
  down over the front, the roof and the front wall faded; 83° down, the
  roof alone; 40° down across a corner, the roof and that corner's two walls.
- **A door or a window in a wall is in the way as its wall is, glass and
  all** (`see_through::in_a_wall`; Hajun, 2026-10-16: seen from outside
  through a Glass Door or a window, nothing faded, as glass was never in
  the way). Its glass, its frame or its leaf between the camera and you
  fades the wall it is in, with everything in that wall, and from inside
  the house the ceiling. Before, a door or window frame in the way faded
  only itself. Checked in a scripted run the same day: standing 2.6 m inside
  a Square House behind a shut Glass Door (further in than the 1.5 m that
  opens it), the camera 9 m out, nearly level, the front wall, the door and
  the roof faded. Not yet tried by hand or on a phone.
- **Checked in a scripted run (2026-10-13)**, beside the front corner of
  the town's red house (it stands turned 30°), the camera swept round past
  the corner a degree at a time: 10° short of where the corner lines up
  with your middle it hid 3 of the 73 points, and the house faded, with its
  doors, windows, handles and sign; the three points of before saw nothing
  until 3° short. Swimming 2 m off the edge of the town with the camera
  low, the edge hid 36 points of your legs and the island stayed. Standing
  in the fountain's basin with the camera low outside it, the basin faded,
  and the water in it. In House Builder, standing in a Square House with a
  door in its front wall and a table before you: from outside the front
  only the door and the table faded (you were in sight through the
  doorway), from a corner the two near walls, the roof and the door in its
  wall, from above the roof, from behind the back wall.
- **A tap goes through whatever is out of the way** (`see_through::InTheWay`,
  which `editor::pick` skips as it does `Untappable`, for as long as any of
  it is faded): only what is not in the way of you can be tapped, and since
  2026-10-13 that includes a door or window whose wall is in the way.
  Checked in a scripted run (2026-10-09): standing in the Square House with
  the camera 15 m out, its front wall faded, and a tap on the bed through it
  opened the menu on the bed. A house you are standing in still cannot be
  tapped (`build::untappable_from_inside`), whatever the camera sees.
- **Zoomed all the way in, it is your eyes, and that is as far in as it
  zooms** (Hajun, 2026-10-09: "the max zoom point is the point at which it
  becomes first person view ... as soon as you zoom out u are back to third
  person view"). Zoomed in past 1.5 m it goes into them, and zoomed out of
  them it is straight back behind you at 1.5 m; either way it takes a tenth
  more zoom than the camera can follow (`SWAP_ZOOM`), a notch of the wheel,
  so that fingers that waver as a pinch ends do not swap it back and forth.
  There is no view button any more (from 2026-09-29 it swapped first and
  third person while you built and rated), and no zooming the view in your
  eyes (it narrowed to 23°). It takes in 66° top to bottom (`CAMERA_FOV`),
  near and far: first person's width; behind you it was 45° until then.
- **Your eyes are inside your head, as high as the model's eyes, 1.38 m**
  (`EYE_HEIGHT`; 1.57 until the evening of 2026-11-01, under the old
  model's head top at 1.71 m, `playercharacter.glb`'s eyes being 1.38 m up
  in a head 0.53 m across to 1.64; Hajun, 2026-10-09).
  They were 2.2 m up from 2026-10-03, over your head, for the view of a
  taller person, and 1.44 m before that. Under a ceiling lower than that,
  the eyes stop just under it (`rise`). Your body is not drawn while the
  camera is in it (`inside_you`), which is why `island::show_where_i_am`
  leaves your body to the camera; through your eyes nothing is in the way.
- **Ahead is the way the camera faces along the ground** (`camera_ahead`):
  where the stick's up walks you and where a piece from the shop comes up.
  Looking straight down, it is the top of the screen.
- `cargo test --lib` checks that walls never bring it in, that you are in
  sight of it behind you at every tilt and as near as it comes, that zoomed
  all the way in it is your eyes, that the zoom goes into them and back out
  as it should, that it looks straight down with ahead still ahead, which
  of a house's walls, its roof and a bed are in the way from inside the house
  and from outside it, that nothing is in the way but in a house of House
  Builder that is down, that a wall hiding only your side is in the way and
  one you stand against is not from your side of it, that every point
  tried on you is inside you, over the step you are on, and turns with you,
  that walking with the stick brings it round behind where you walk only once
  it has been left alone, and never walking at it, that riding a bus brings
  it round behind the way the bus faces, standing still, from in front of
  the bus too, and keeps up with the bus turning, that a double tap swings
  it behind you, and that talking to someone it looks at their face from near
  you and goes back out after.
- **A jump stops the head just under a ceiling** (`headroom`, in
  `move_bodies`). Before 2026-09-28 the head went into it at the top of the
  jump, the body's collision read the ceiling as a wall there, and anyone
  jumping in a room lower than about 3.25 m was shoved sideways. The fix has
  not been tried by hand yet.

## The town is data, not code

Nothing that stands on the town is spawned by hand. `src/map.rs` holds a
catalogue of kinds — model, collision, tap volume, whether the player may pick
it up — and a save file listing what is placed where. `src/editor.rs` is the
tap-to-edit menu that writes to it.

- **Tap something and four buttons come up round it**: a bin above, a green
  tick below, and a 45° turn either side. To move it, drag the thing itself
  with your finger (a press on it is the editor's, not a look round). The tick
  says it is where you want it; in the town it only shuts the menu. The same
  menu puts House Builder's pieces down (below): it works on anything
  `editor::Editable`, and only says what was asked (`editor::Edit`), which
  `editor::edit_town` does for the town and `build` for a house.
- **The four buttons stay on the thing, not on the screen**
  (`EditMenu::follow`). They ring a point on it — its middle, no more than 1 m
  up — wherever that lands, so turn the camera away and they slide off the
  screen with it; once none of them could be seen, or it is behind the camera,
  they are put away and cannot be pressed (`EditMenu::shown`). Until 2026-09-29
  they were pulled in to the screen and down clear of the time and the theme,
  so with the thing out of view they sat pinned at the edge, and Hajun asked
  for that to go. **Keep it that way:** never clamp the anchor to the screen.
  The one pull is down, clear of the time along the top — a tap on it skips the
  wait, so the bin must not be over it — and only as far down as the thing
  itself goes on the screen (`clear_of_the_top` in `follow`): a house up the
  screen is ringed lower down its front, a small or far thing keeps them on
  it, and one that has gone up off the screen takes them with it. Only with its
  middle behind the camera (in your eyes, in a house) are they round the part
  of it that is on the screen.
  The turn buttons' circular arrows are drawn in code (`editor::turn_image`),
  the way the rating star is.

- **Adding an object to the world means adding a `CATALOGUE` entry and a record
  to `default_town`,** not a `commands.spawn` in `setup_world`. The default
  town is empty for now; the catalogue's one kind, the tree, no longer gives
  off light. A save written before 2026-09-26 could still hold the two trees
  the default town used to start with, and on 2026-11-01 one of them stood
  in Hajun's new town on their phone (Hajun: "remove that tree in the map i
  dont know why it spawned cuz its not part of the map"): `SAVE_VERSION`
  is 2 since, and a save of an older version is started over from
  `default_town` and written over (`map::read_save`), on every device.
- **A save is one JSON document per player**, so it can become one row in a
  server table later without reshaping. It records a *kind*, never an asset
  path. Bump `SAVE_VERSION` if that shape changes, or to throw every
  device's save away, as above.
- It lives outside the repo, so a rebuild never wipes a town:
  `%APPDATA%\donggeurami_town\town.json` on Windows, the app's internal
  storage on Android, `Library/Application Support` in the sandbox on iOS
  (`Documents` until 1.0.2, when the Files app was given Documents for the
  log; what 1.0.1 left there is moved). Delete it to start over from
  `default_town`.
- **Editing only reaches the town, and only while you are in it.** Going home,
  or into a game, puts away whatever was being edited.
- **Editing on desktop needs the cursor free — press `Escape`.** While it is
  grabbed for looking around it sits hidden in the middle of the screen, where
  only the player's own back is ever under it. On a phone you just tap.

## The town holds 32 players; offline, 31 of them are AI

(For the day of 2026-11-07 the 32 arrived on the four islands of the PvP
map, eight to each, by seat; since that night that is Bridge Wars'
arrival, four to a game, and whoever falls off there is put back where
their seat arrives, `lobby::respawn_the_fallen`, "Games", above.)

`src/lobby.rs` holds the room: seats, each a flat `Member` record — an
id, a name, a look and an `ai` flag — the very record the server sends
(`roundtown_net::Member`). Offline you are seat 0 and every other seat is an
AI, walked by `src/ai.rs`: eight seats as the app opens (`lobby::SEATS`,
which is also a game of House Builder's) and then as many as fill the town
to `roundtown_net::TOWN_BODIES`, 32 since 2026-11-01 (Hajun: "a total of 32
ppl in the map including ai, so each new player join replaces a ai"; 8
until then), on arrival spots round the pond (`lobby::TOWN_SPOTS`, 32: the
sixteen on the path and sixteen more on a ring 2 m further out,
`roundtown_net::arrival`), under 48 names, 24 for the boys and 24 for the
girls since 2026-11-02 (`roundtown_net::BOY_NAMES`, `GIRL_NAMES`, which the
server's bots go by too; Hajun: "use the more manly ai names for the boy
ais and girly cute ai names for the girl ais from the names you already
made"; `lobby::AI_NAMES`, one list, until then, and 20 until 2026-11-01;
past a side's names an AI is "Bot"). Online, the server sends whoever else is in
your channel of the town, and this device keeps the town at 32 bodies with
AIs of its own (below, "Playing online"). The cost of 32 bodies is not
measured on a phone.

- **Players come from a roster**, the way objects come from the town save:
  the town's (`lobby::Lobby`), or a game of House Builder's (`builder::Game`).
  All eight of the town's are spawned there when the app opens; a game's AIs
  are spawned when it fills and go when it ends. Either way through
  `lobby::spawn_player`, and nothing else spawns a player, yours included.
- **To move a body to another island, `lobby::send` it.** It drops the jump it
  was in, stops an AI's walk, keeps the move from being read as a stride, and
  swings the camera behind you if it is yours.
- **An AI is a player.** Same body, same collider, moved by the same
  `move_bodies`; only its `Intent` comes from `ai::Steering` (walk this far
  that way, then stop) instead of a stick. Players on the same island collide
  with each other.
- **Only your own legs follow your stick.** Everyone else's walk is read off
  how their body moved (`walk_from_motion`), so walking never has to go over
  the network: a position is enough.
- There are five looks and eight seats: you are the boy or the girl you chose
  on signing up (the boy until then), and the AIs a boy or a girl at random
  in one of three colourings each (since 2026-11-02, "The first time, you
  sign up"; red, yellow and purple in turn until then); the boy and the
  three old looks drawn as `lobby::MODEL` and the girl as
  `lobby::GIRL_MODEL` since the night of 2026-11-01 ("Models").

## You can talk to anyone in the town

Since 2026-10-31 (Hajun: "add a proximity prompt chat bubble ... add it for
every ai character in the map for now"), in `src/talk.rs`: the town's AIs
(`talk::Speaker::Townsfolk`, this device's own online) and the House Builder
(`Speaker::HouseBuilder`). People online are not spoken to.

- **One bubble at a time, over the nearest to you** (`talk::choose`;
  Hajun: "at most one prompt active at a time (nearest in front of you). No
  choosing between prompts, so no precision needed"): a small white circle
  with a black outline, its tail down and to the left, three black dots in it
  (`bubble_image`, drawn in code as the rating star is; 10% of the short
  side of the screen wide, 8 until Hajun asked for bigger that evening, and
  what is said and the replies a tenth bigger than first drawn the same
  evening: "very slightly increase the size of both the text box and the
  font size and the button"), its point over the
  head of the nearest of them within 4.5 m along the ground, whichever way
  you face (since 2026-11-02, Hajun: "it should be irrelevant where the
  player is facing, just their xy location"; within 70° of the way you
  face as well until then),
  1.85 m over their feet, where their name's foot is (`tags::OVER_HEAD`;
  1.95 until 2026-11-01, Hajun: "move its z slightly lower so its slightly
  more closer to the nametag"),
  and in sight of your eyes, no wall between. Once it is over someone it stays
  out to 5 m (and 80° round, while facing counted), unless someone else is
  half a metre nearer, so that it does not flicker between two. Being the
  only one, it takes a tap 3
  vmin wide of it. Only in the town, and never while anything else is up or
  the camera is on its way back from a face. Online it sits over a name:
  its point a name and a half's height (`tags::TAG_TEXT`) over the name's
  foot, the same way over the name however near the camera is, since the
  two stand on one point of the world; with its point 0.1 m higher the gap
  grew as the camera came in, about 1% of the screen's height at 8 m and
  3% at 3.5 m.
- **Tap it, or on desktop press E, and you talk to them** (Hajun: "zoom in
  first person view to show that ai character's front face (ai character
  pause until dismiss)"). They stop where they stand, the two of you turn to
  face each other, and the camera comes in over 0.45 s to your eyes, turned to
  their face (`CloseUp`, `close_up_view` in `src/lib.rs`), from no further than
  1.3 m from them: further off, it comes forward between you, at the height of
  your eyes. On a phone the stick and the jump button are put away. What they
  say is along the bottom of the screen, black on white inside a black
  outline, with their name on a black tab over it; what you can say back is
  over it on the right, white pills outlined in black, as the bubble is
  ("response/action dialog on right side of screen above chat dialog"). A tap
  anywhere, or on a reply, and you are done (a press is the conversation's
  wherever it lands): the camera goes back out over 0.35 s to exactly where you
  had it, they carry on, and the House Builder turns back to face out. On
  desktop the cursor is let go meanwhile, Enter says the first reply, and
  Escape or E says goodbye.
- **An AI says one of `SMALL_TALK`**, at random, never the same twice running,
  and you answer in a word, which is goodbye. The lines are mine, about the
  town (its fountain, buses, lamps, sunset, music), in plain ASCII for the
  built-in font.
- **The House Builder asks whether you want to play** (`builder::pitch`):
  Play, with its 100 coins beside it, or Later. Play goes to House Builder as
  the old dialog's Play did (`builder::Ask::Join`), and the camera cuts back
  to you rather than easing out from a face left in the town.
- Seen in the desktop build on 2026-10-31, scripted, with real touches through
  picking: the bubble over an AI and over the House Builder, both
  conversations, a tap on the world ending one, Later, and Play paying 100 and
  going to the lobby. Not yet on a phone, nor online.

## House Builder

One of the two games, in `src/builder.rs` (the other, Bridge Wars, is in
`src/arena.rs`: "Games", above). On screen it is called "Build Your Own
House" (`builder::NAME`), and "Home Builders" on its card in the game
selector, which since the night of 2026-11-07 is the way in
(`builder::Ask::Join`, as the House Builder's Play was); in the code it is
House Builder, after the NPC who offers it. The House Builder stands at
the door of a house in the town, when the town has one: `roundtownmap.glb`
as it is (its fourth version, 2026-10-31) has none, so there is no NPC in
it, and what follows about them holds for a town that has.
Since 2026-10-30 (Hajun: "for now just put the build house batte minigame
battle in front of one house"), the house nearest where the first seat
arrives, 2.5 m out from its door, facing out (`build::town`): in the new town
the first house up the straight road behind where you arrive, on its right
going out, whose door faces the ring round the pond. They are the body
everyone has, in the black top the old town had them in
(`builder::stand_house_builder`: the model's `body` material painted black,
which `playercharacter.glb` has not, so since 2026-11-01 they look like
everyone else), given a collider and their arms hung at
their sides like everyone else's. Until then they were part of the old
town's model, the node called `House Builder NPC` in `circlemap1.glb`, with
"BUILD TO IMPRESS" in raised white letters over their door (`buildtext`); a
node of that name in a town's model still stands where it is put
(`builder::find_house_builder`).

- **Talk to them as to anyone in the town** (above, "You can talk to anyone
  in the town"), and they ask whether you want to play their game for 100
  coins (`builder::pitch`): Play pays, and takes you to the lobby
  (`lobbymap.glb`); Later, or a tap anywhere else, does not. Without 100
  coins, Play is greyed out and does nothing, and they say you need them.
  Since 2026-10-31 (Hajun: "for the build minigame thing, use this same
  style ... play and later"); until then a speech bubble over their head,
  "Want to build a house?", opened a dialog with Cancel and Play, and on
  desktop Space talked to them.
- **The lobby counts players top centre** ("1/8 Players"). There is no server
  and no matchmaking: the computer takes the empty seats one at a time, one
  every 5/7 s, so the room is full 5 s after you arrive (`QUEUE_FILL`), with
  AIs named unlike anyone in the town, each paying its own entry. Until
  2026-10-01 all seven came at once after 5 s; Hajun asked for one at a time
  "just for now", standing in for real players joining. The room shows
  8/8 for 2 s, then everyone goes to a plot of their own (`Venue::Plot(seat)`).
- **Building lasts 5 minutes** (`BUILD_SECS`; 3 until 2026-10-01), with the time top centre, a random theme
  across the screen at the start and under the time throughout. You build with
  the hammer button (below); the computer's players build nothing.
- **Then everyone visits each plot for 15 s**, in seat order, yours first. At
  anyone else's, you give it 1–5 stars along the bottom of the screen (on
  desktop, keys 1–5 as well); nobody rates their own, and the AIs rate at
  random.
- **Then the results, as a leaderboard down the right of the screen**
  (`builder::spawn_results`; Hajun, 2026-10-08, in place of a dialog that
  only named the winner). Every house is placed by its stars, most first,
  a tie settled by lot (`roundtown_net::standings`; the lot for every place
  is my reading, the winner's was before), and everyone stands at the
  winner's plot. All eight rows: the place, the first three on gold, silver
  and bronze medals in bold (the game's font has one weight, Inter's
  Regular since 2026-11-13 as Bevy's own before it: the number
  is drawn twice, a hair apart), the name, the stars, and what the place
  paid. **Every place pays** (`rules::PRIZES`, Hajun's): +500, +400, +300,
  +200, +100, +50, +25 and +0, the computer's players too offline; online
  only people, into their accounts. **It is not a dialog**: you walk and
  jump round the winner's house with it up, and only a press on it is its
  own. Play again (another 100, greyed out without it, and lit up once the
  prize brings it) or Exit; nothing pressed, you are back in the town after
  15 s (`rules::RESULTS_SECS`), counted down on it. On desktop Enter plays
  again, and Escape frees the cursor for clicking it, as anywhere else. It
  slides in from the right, 52 wide and 62 tall in shares of the short side,
  which keeps it above a phone's jump button; online, names over heads that
  would show through it are put away (`tags::HidesNames`).
- **Building and visiting are seen as the town is** (above, "The camera
  stands where you zoomed it"): from behind you, or, zoomed all the way in,
  through your own eyes. From 2026-09-29 to 2026-10-09 a round button over
  the hammer (V on desktop) swapped first person for third while you built
  and visited; Hajun had it taken out when zooming in became the way into
  your eyes.
- **Tapping the time top centre skips to the end of the wait** — the build,
  or a visit. It is for trying the game out, and meant to go.
- **While a dialog is up you stand still, and presses reach only it.** On
  desktop the cursor is let go for as long as it is up and taken back after,
  if it was taken before (`cursor_for_dialogs` in `src/lib.rs`); Enter says
  yes and Escape no.
- **Everything a game spawns carries `builder::InGame`** — plots, AIs, the
  board, the stars, the results — and is despawned when you leave or play
  again. Only your own body comes back.
- **Testing on desktop:** posted clicks are dropped unless the real cursor is
  over the game window, but picking takes a finger's taps from
  `WindowEvent::TouchInput`, so writing those from a throwaway system presses
  any button the way a phone does. (Writing plain `TouchInput` messages does
  nothing: picking does not read them.) For the world, set every field of
  `editor::EditPointer` from the same system, after `read_desktop_pointer`:
  the real mouse over the window adds to its `travel` otherwise, and a tap
  reads as a drag. Bevy's `Screenshot::primary_window()` with `save_to_disk`
  lets the game take its own screenshots on cue.

## Building your house

While the time to build runs, a hammer button sits top right, below where a
button that went home sat in the town until 2026-10-31 (`src/shop.rs`). It opens the shop: a window under
the time, 65% of the screen wide, with a tab each for Foundation, Doors &
Windows, Furniture and Decorations, and a grid three across and two down that
scrolls (a finger drags it, a mouse wheel turns it). A tab that sells nothing
is not shown (`Tab::sold`): since 2026-10-03 that is Decorations, as none of
Hajun's pieces is one yet. The cross on its corner, a tap on the dark round
it, or `Escape` shuts it.

- **Tap a piece and it comes up in front of you as a ghost** — in nobody's
  way, with the tap menu open on it (`src/build.rs`). Only a house is drawn
  see-through as a ghost (`GHOST_ALPHA`, 0.65); everything else is drawn just
  as it will be once down (Hajun, 2026-10-01, after finding every ghost too
  see-through). A house comes up 3 m out from your
  edge (`HOUSE_IN_FRONT`), so that you see it whole; before 2026-09-30 it was
  0.6 m, like furniture (`IN_FRONT`), and its front wall filled the view.
  Drag it; the tick puts it down, solid; the bin throws it away. Tap it again
  later and it is picked back up, a ghost until the tick; all but the house
  you are standing in (since 2026-09-30), which from inside nearly every tap
  landed on: a tap goes through it (`build::untappable_from_inside`, which
  reads inside off its `Floor...` objects); and, standing in a house, all
  but what stands between the camera and you, which is see-through there
  and which a tap goes through as well (since 2026-10-09, in a house only
  since 2026-10-25; "The camera stands where you zoomed it"). Only one
  thing is a ghost at a time: the hammer is put away until it is down. On
  desktop the
  cursor is let go for as long as it is (`hud::WantsPointer`).
- **One house to a plot**, and doors, windows and what hangs on walls need
  one: the shop greys out what cannot be chosen, with the reason over the grid.
- **How each piece moves** (`build::Mount`): a house anywhere its whole
  footprint is on level land; a door or window along whichever wall of the
  house is under your finger, cutting its hole in it, a door standing on the
  floor and a window 22% of the way up its storey (`WINDOW_SILL`: 32% until
  2026-09-30, when Hajun asked for lower, first 25%, then 20%; then 22% on
  2026-10-01, "very slightly higher"). A window stays at that height: since
  2026-10-01 (Hajun: "lock the height movement of the window so it can just
  go sideways") it is dragged only along the wall, and up or down only from
  one storey to another, as a door is; a hanging, like the script's painting
  or clock (none is sold since 2026-10-03; `build::my_hanging` is ready for
  one of Hajun's), on either face of a wall; everything else carried over the floor the
  way a body walks — up a small step, through a door, up the stairs, never
  into a wall with any of its footprint. What you grab stays under your
  finger, on a wall as on the floor. A door, window or hanging follows every
  move of the finger, whether or not the finger is over the level it was
  taken hold of at (`editor::Edit::Drag`'s `to` is then `None`), and with
  the finger off every wall it follows along the plane of its own
  (`Wall::across_plane`). Until 2026-10-01 a window stopped dead both ways,
  dragged up past your eyes or over the top of its wall, and Hajun found it
  sticking.
- **Pieces go right up to the edges** (Hajun, 2026-09-30: "dont put too much
  margin"). Anything in or on a wall comes as near its ends, another wall
  meeting it (`Wall::joins`, read off the other `Wall...` boxes, so a window
  reaches the inside corner whichever wall runs on into it), and its storey's
  floor and ceiling as `EDGE`, 6 cm: a little more than the made windows'
  sills stuck out past their frames. Two pieces on one wall still keep
  `APART`, 0.5 m.
  Furniture goes flush against walls and other pieces, touching: it is
  tested at exact points (`Island::blocked`) rather than on the 0.25 m
  lattice bodies use, `TOUCH`, 1 mm, in from its edge, and a drag
  (`editor::carry`) that meets something closes the last step to it before
  sliding along it, across or along the piece's own turn, so that one turned
  45° slides along what it is square to. Before 2026-10-03 it was tested 2 cm
  out from its edge, and two of Hajun's cafe desks pushed together stopped 2
  to 6 cm apart. Before that, the lattice, a 0.125 m margin and the 0.25 m
  steps together left a gap of up to about 0.4 m.
- **Furniture snaps into line** (`build::snaps`; Hajun, 2026-10-03: hard to
  line up, "no snapping at all"). Carried within `SNAP`, 0.2 m, of a wall or
  another piece it is square to (turned the same, or a right angle off), it
  goes flush against it; and flush against another piece, it lines up with
  that one's nearer end, or its middle, within `SNAP` too, so that two desks
  go end to end in one line, or a desk lines up along another's front. Only
  what it could not be carried onto counts (a rug does not), each piece as
  the box round it, and a snap that would put it in something else, or off
  the floor, is not taken. Checked in a scripted run: the Cafe Desk Corner
  dragged toward 12 cm short of the edge's end and 10 cm off its line went
  to exactly flush and in line, and along its front to flush and level with
  its end.
- **Furniture always turns** (`build::turned_clear`; Hajun, 2026-10-09: up
  against a wall or another piece, it would not turn at all). Turned into a
  wall or another piece, it moves out of it as little as it takes, tried 16
  ways round (`ROOM_WAYS`), over the same floor and never through a wall, and
  snaps into line as a drag would leave it; with nowhere clear near enough,
  it turns where it is. Checked in a scripted run: the Cafe Desk Edge, long
  side flush against a wall of the Square House, turned 45° and came straight
  out 0.55 m. A house still turns only where it fits on the land.
- **Every door swings open by itself** (`build::swing_doors`: the Classic
  Door since 2026-09-30, three more since 2026-10-03, the Classic Window Door
  since 2026-10-04). Each is Hajun's, in `assets/mybuilds/`. A leaf, the
  part called `Door` (whatever is on it, a handle, a window's glass and bars,
  a child of it or joined into it), turns about its origin, the hinge, from 0, shut as
  modelled, through 90° (Hajun's call, 2026-10-01, after trying 135°, 180°
  and 170°) out of the house, to stand square to the wall. The Classic
  Double Door and the Glass Double Door have a second leaf, `Door.001`, as
  Blender names a copy, hung on the other side; both leaves swing out
  together. `Swing` in `build::PIECES` names the leaves and the angle, since a
  glTF says neither (`ONE_LEAF`, `TWO_LEAVES`); which way is out, the game
  works out for each leaf from which side of its middle its hinge is
  (`build::outward`), and it turns the leaf in the space the leaf hangs in,
  so a leaf on either side, or one modelled as another's mirror image, swings
  out.
  A door opens for anyone on the plot on its storey, AIs too, within 3 m
  of the middle of the doorway from outside and 1.5 m from inside
  (`SWING_OUTSIDE`, `SWING_INSIDE`: Hajun's rule, nearer from inside, going
  out; outside was 3.5 m until 2026-10-03, when Hajun found it opening too
  far off and asked for a little less, inside as it was); for a double door,
  of the middle of either leaf or anywhere between
  them (`nearest_leaf`), so that it opens for someone going through either
  leaf where a single door would. It stays open 0.5 m further
  (`SWING_HOLD`), swings open in 0.35 s (players run at 7 m/s) and shut in
  0.8 s. A ghost stays shut. Like every door the leaves are not solid: the
  hole is what counts. The single doors are 2 m by 3.35, the double doors
  3.8 m by 3.35: they fit Hajun's 5 m storeys, but a wall lower than 3.35 m
  has nowhere for them, and one would be thrown away there. Checked in a
  scripted run (2026-10-03, with 3.5 m outside): all four began to open
  3.46 m out coming in and 1.45 m in going out, the double doors walked
  through 0.8 m off their middle, and every leaf swung out of the house.
- **A house with a floor over another has a storey to each**
  (`build::Wall::storeys`), read off its `Floor...` objects: a floor with a
  body's height of wall under it splits every wall there. A door, a window or
  a hanging goes in the storey your finger is on (fresh from the shop, the
  one you stand on), as if that storey were a wall of its own, but only
  where it fits: a storey too low for it takes none, and it goes in the
  nearest storey that has room. A floor level with the top of the walls, like
  the Flat House's flat top, closes the storey under it. That is how
  the Roof Terrace House has windows downstairs only: its walls stop at its
  upper floor, and the 1 m parapet round the terrace is not a wall. A window
  asked for on the terrace goes in the ground floor wall you face, and one
  dragged up there stays under the terrace floor.
- **Nothing goes in a wall where the stairs go up it** (Hajun, 2026-10-01).
  Whatever is built into a house besides its `Wall`, `Floor`, `Roof` and
  `Gable` parts (the `Stairs`, or anything else, such as a parapet) counts
  where it stands against a wall, like another wall meeting it
  (`Wall::joins`): no door, window or hanging goes over the box it fills
  there. In the two floor houses the stairs run the length of the right-hand
  wall (Blender +X) up to the upper floor, so that wall takes nothing
  downstairs, and the back wall nothing beside the top of them; over the
  stairwell, upstairs, a window can still go. Fresh from the shop, a piece
  goes on the storey you stand on of the wall you face, or failing that of
  the next nearest wall, and only then on another storey: in a scripted run,
  facing the stairs downstairs, a square window and a classic door both came
  up on the front wall.
- **What is down is part of the plot** (`island::Islands::build_on`): walked
  into and stood on. Houses (walls with their holes cut) and furniture count;
  doors, windows and hangings do not, the hole is what counts; ghosts never
  do. A window's hole, though, is filled with a box the size of it
  (`build::pane`), its glass as far as bodies go: drawn open, shut to them,
  so that a window is never a way in. A door's hole is open to bodies.
  Standing in a house, what stands between the camera and you is
  see-through, and a tap goes through it; outside it, on the plot, nothing
  is (above, "The camera stands where you zoomed it").
- **When the time runs out**, any ghost is put down where it is and nothing
  can be changed any more. Nothing is saved: every piece carries
  `builder::InGame` and goes with the game.
- **The shop sells only Hajun's own pieces, in `assets/mybuilds/`** (their
  call: houses, doors and windows since 2026-09-30, everything since
  2026-10-03, "just keep mybuilds assets"). The Foundation tab is the four
  houses, and Doors & Windows is the Classic Door, the Classic Window Door
  (`classic_door_with_window`, since 2026-10-04; the name is mine, kept as
  short as "Classic Double Door" so that it fits a card on one line), the
  Classic Double Door,
  the Glass Door and the Glass Double Door (since 2026-10-03),
  the Square Window (2 m by 2) and the Wide Window (3 m by 2). Furniture is
  their Single Bed (`singlebed`, since 2026-10-03), 2.3 m wide with a red
  blanket (pink until Hajun recoloured it, 2026-10-04), and Double Bed (`doublebed`, `build::my_floor`, since 2026-10-01),
  3 m by 4.5, 0.95 m high, a jump to get onto, made of stacked boxes whose
  touching faces the game reads correctly; and, since 2026-10-03, their Cafe
  Chair (`cafechair`), a round seat 0.75 m up in a wooden ring, with a curved
  back to 1.375 m, on four splayed legs; their Circle Table
  (`circletable`), a round top 2 m across and 1.2 m high on a pedestal; and
  their Cafe Desk Edge and Cafe Desk Corner (`cafedeskedge`,
  `cafedeskcorner`), counters 1.5 m high, 2 m by 1 and 1 m by 1, one mesh
  each with a wood front, a cream top and a dark plinth, and since
  2026-10-04 a grey strip along the top's edge (`desktopedge`); and, since
  2026-10-13, their Soft Couch (`softcouch`, blue, 3.75 m; the name on the
  card mine), coloured by a palette texture embedded in its glb, the only
  texture in the game; and since 2026-10-16 ("add streetlamp as one of the
  furnitures"), their Street Lamp (`streetlamp`, the name on the card mine),
  the lamp the town's are, 7.4 m tall, lit at night as they are (`lamps`).
  Its `.blend` was a copy of the town's with everything but the lamp taken
  out, all but the House Builder's armature and two road path empties,
  which were then taken out of it too, and it had no picture: it was given
  the thumbnail camera, sun and sky every piece has and its picture
  rendered. Nothing is in Decorations.
  On 2026-10-16 Hajun had ten pieces taken out of the game, their files
  deleted (`.blend`, `.glb`, `.png`): the Three Leg Table
  (`threelegcircletable`, of 2026-10-03) and nine of the ten added on
  2026-10-13, all but the Soft Couch: the Frame Bed (`Bed_Single`),
  Nightstand (`NightStand_2`), Wooden Chair (`wooden_chair`), Cabinet
  (`Cabinet_medium`), Fridge (`Kitchen_Fridge`), Oven (`Kitchen_Oven`),
  Large Oven (`Kitchen_Oven_Large`), Bathtub (`Bathroom_Bathtub`) and
  Bathroom Sink (`Bathroom_Sink`). Pieces go over the network by their kind's
  name, and one this app does not sell is skipped (`build::raise_shadows`),
  so an older app's on a plot does no harm.
  The script's pieces are no longer in `build::PIECES`: its five houses,
  three doors and three windows since 2026-09-30 (Hajun deleted their files
  on 2026-10-03, all but the tall window's), and its furniture and
  decorations since 2026-10-03 (bed, chair, table, sofa, kitchen, bookshelf,
  armchair, wardrobe, nightstand; flower pot, potted plant, rug, floor lamp,
  vase; and the hangings, painting, clock, mirror and wall shelf). Their
  files are still in `assets/build/`, which the APK leaves out
  (`<dir>build` in `assets-left-out.txt`), and the script still makes them
  if run again.
- **The script's pieces are Blender models, made by `blender/make_build_items.py`**
  into `assets/build/` (`.blend`, `.glb`, and the `.png` the shop shows). Run
  it again to remake them. They are drawn at real size, which suits the
  players (1.7 m to the top of the head), and its `SCALE` is 1 since
  2026-09-29. It was 2.5 for a day, to give the third-person camera room
  indoors; Hajun then asked for the pieces to make sense next to the players.
  The cottage is 6 m square with 2.8 m walls, a door is 2.3 m high, and a
  mattress is 0.55 m off the floor: it was low enough to step up onto, until
  the step limit went to 0.5 m (2026-09-30), and now takes a jump. Indoors
  the camera no longer comes in close behind you (since 2026-10-09): zoomed
  out, it stands outside the house, and the walls between fade.
- **`square_house` is Hajun's own model, and is loaded as they made it.**
  `assets/mybuilds/` holds their `.blend`, the `.glb` exported from it and the
  `.png` the shop shows; the piece is `my_house(...)` in `build::PIECES`, and
  `PieceDef::dir` sends the game there rather than to `assets/build/`. It is
  12 m square with 5 m walls (6 m until Hajun reshaped their houses on
  2026-10-01). There is no
  conversion step, and there must not be one: the file itself follows the
  rules. It has four `Wall_` boxes (plain boxes, so no bevels on walls: the
  game remakes every `Wall` as one to cut holes), two closed `Gable_` shells,
  `Floor_1`, `Roof`, the other houses' three materials, and the camera, sun and
  sky the shop picture is rendered with (in the `glTF_not_exported`
  collection, which the exporter skips, with the reference `Icosphere`).
  To change it: edit the `.blend`, export it (File > Export > glTF 2.0, `.glb`,
  defaults, over `square_house.glb`), and render it from the camera (F12) to
  `square_house.png`.
  What went wrong before it followed the rules (2026-09-30): fused walls, so
  every door was thrown away; no materials, so a part loads as glTF's default,
  fully metallic, near-black by the loader's code (not rendered to check); and,
  when the gables were split off as open shells, a doorway that could not be
  walked through (see "A solid has to be a closed shape"). Checked with a
  player walked in through a door and back out, from the file as it is.
  Blender is at `C:\Program Files\Blender Foundation\Blender 5.2`.
- **Three more of Hajun's are built from it, two of them with two floors.**
  As reshaped on 2026-10-01: `square_house_two_floor` ("Two Floor House")
  has 10 m walls and a `Floor_2` 5 m up over all but a 2 m stairwell along
  its right wall (Blender +X), where a solid `Stairs` block climbs from 1.25
  m in from the front to a landing at the back: 25 steps of 0.2 m, under the
  step limit. `square_house_roofless_two_floor` ("Roof Terrace House") is
  the same with no roof or gables: its `Wall_` boxes stop at the upper floor,
  5.1 m, and a `Parapet_` on each, tapering on the outside from 0.5 m thick
  to 0.25, stands 1 m round the terrace. `square_house_roofless` ("Flat
  House", "Roofless House" until 2026-10-01) has one floor, 5 m walls and a flat `Floor_2` on top level with
  them. On 2026-10-01 Hajun had modelled the terrace's walls with the taper
  on them; since the game remakes every `Wall` as a plain box, the tapered
  tops were split off into the `Parapet_` parts, and its stairs, called
  `Cube`, were renamed `Stairs`. The two floor house's `Stairs` has every
  face twice over, all facing the same way: harmless, as the game merges
  surfaces met twice at one height, and left as modelled. Earlier fixes
  (2026-09-30): the stairs had no material, and `Gable_front` was inside
  out, as it still is in `square_house.blend`, where it is buried in the roof
  and harmless. Their thumbnail cameras are fitted to each house.
  The glTF loader turns every model half round: Blender's (x, y, z) is
  (-x, z, y) in a piece's own space in the game, front toward -Z.
- **`classic_door` is Hajun's too** (2026-09-30): `Frame`, and `Door` with
  its origin on the hinge and `Door_handle` its child, on the outside (-Y),
  and since 2026-10-04 `Door_handle.001`, on the inside.
  Fixed in the file: it was modelled with the frame's front at the origin
  (0 to 0.5 m back), so everything moved 0.25 m back to centre the frame on
  it, as a door's origin is the bottom middle of the hole, halfway through
  the wall; `DoorFrame` was renamed `Frame`; and its thumbnail camera, copied
  from the square house with the render path still `square_house.png`, was
  fitted to the door and pointed at `classic_door.png`.
- **So are `square_window` and `wide_window`** (2026-09-30): `Frame`,
  `Glass` (alpha 0.2, blended, so it exports see-through) and bars. Fixed in
  the files the same way as the door: modelled round their middles and from
  the origin backward, they were moved up 1 m and back 0.25 m so the origin
  is the bottom middle of the hole, halfway through the wall; `GlassFrame`
  was renamed `Frame`; the frame and bars, which had no material, were given
  `window_frame`, Blender's own default look (grey 0.8, roughness 0.5), which
  is how they showed in Blender; and the thumbnail camera was fitted and
  pointed at each one's own `.png`. The wide window's left bar is 5 cm nearer
  the middle than its right one (-0.45 against 0.5 m), as modelled.
- **So are `classic_double_door`, `glass_door`, `glass_double_door` and
  `singlebed`** (2026-10-03). They came following the rules: `Frame` parts
  round the hole (split in two halves on the double doors), each leaf's
  origin on its hinge, the origin of the whole the bottom middle of the hole
  halfway through the wall, a material on every part, and the glass blended
  at alpha 0.3. Each `.blend`, copied from an earlier one, still rendered to
  that one's picture (`classic_door.png`, `doublebed.png`), so its thumbnail
  camera was fitted and its own `.png` rendered. The glass door's spare
  handle parts are in `Collection.001` and `.002`, which no scene holds, so
  they are not exported; the handle that shows is joined into the leaf.
  The two cafe desks and the two round tables came the same way
  (2026-10-03), with no picture yet and their render still pointed at
  another piece's (`square_house.png`, and for the Circle Table, copied from
  the other, the Three Leg Table, since removed, `threelegcircletable.png`);
  only the camera and the path were
  changed, and each `.glb` is Hajun's export. A round table's camera is
  fitted to its corners rather than its box, which a disc does not fill: fitted
  to the box, the table came out small in its card. The Circle Table's
  pedestal is open underneath and at its top, under the table top, as
  modelled: harmless there, since the top fills those heights over it.
- **So is `classic_door_with_window`** (2026-10-04): the Classic Door, saved
  as a copy, with a window cut through its leaf, glass (`Cube`) and two bars
  (`Cube.001`, `.002`) across it. **What hangs on a leaf must be its
  child:** the glass and bars came parented to nothing, and would have stood
  in the doorway while the leaf swung away. They were parented to `Door`
  keeping where they are (Ctrl+P > Object), which changed nothing else in
  the export, and the render path, still `classic_door.png`, was pointed at
  its own picture. The same day Hajun recoloured the beds' blankets and the
  Circle Table's top, and their pictures were rendered again.
  **Don't mirror a part with a negative scale; apply it.** The glass double
  door's right-hand leaf and frame half came as the left-hand ones with a
  scale of -1. Blender draws that fine, but Bevy draws a mirrored part of a
  double-sided material lit inside out: in the game that half's frame came
  out about a quarter darker than the other half's. The mirror was applied to
  their meshes, their faces turned back to face out (Object > Apply > Scale
  does both), and the file exported again.
- **Keep a piece's mesh light.** The plot's shape is every triangle of every
  piece down on it, and a body tests every triangle in the 2 m square under
  it several times a frame. A Cloud Sofa made from Hajun's photo
  (`assets/cloudsofaimage.png`) was in the shop for part of 2026-10-03, kept
  to 7,680 triangles for that (a first version had 31,000, and 123,000 with
  its Subdivision modifier applied), and went with the other assets Hajun
  cleared out that day. The glTF exporter's defaults leave modifiers off, so
  the game gets the mesh as it is, not as a Subdivision modifier shows it in
  Blender. That cuts both ways: the Cafe Chair's legs are one leg and an
  Array modifier, and its first export had one leg. It was exported again
  with Apply Modifiers ticked (`export_apply`), which is how to export it
  after changing it. And a pale, matte piece comes out as one flat shape in its shop
  picture: the thumbnail sun sits in `glTF_not_exported`, which is not
  rendered, so only the even sky lights it; the sofa's picture was rendered
  with the sun linked into the scene.
- **Every material has to reach an output set to All, through a Principled
  BSDF**, or the glTF exporter writes it bare (`{"name": ...}`), which glTF
  reads as white and fully metallic. Eight of the ten pieces of 2026-10-13
  (all but the Couch and the Cabinet) came from Blender with two outputs to
  each material, the Principled BSDF on one set to EEVEE and a Diffuse BSDF
  on one set to Cycles, and their exports had no colour at all; and the
  ovens' glass was a Transparent BSDF, which glTF has no way to say. Fixed in
  their `.blend`s: the EEVEE output set to All (the Cycles one left as it
  was), the glass made a Principled BSDF in its own tint at 0.3 alpha,
  blended, as Hajun's windows' glass is; then exported with the defaults,
  which gave the same nodes and vertices byte for byte, and their pictures
  rendered with the thumbnail camera, sun and sky every other piece has,
  added to each file. Those eight were all taken out of the game on
  2026-10-16, but the lesson stands: check a new model's exported materials
  (`pbrMetallicRoughness` in the glb's JSON) before selling it.
- **Nobody gets in through a window** (since 2026-09-30, when Hajun found
  they could). Their windows came up 1.28 m up (in 6 m walls), a jump reaches 1.64 m, and
  at 2 m tall a window is taller than a body, so with the hole open a jump
  onto the sill walked you straight in. Now the hole is filled for bodies
  (`build::pane`): checked in a scripted run, a player running at a square
  window and jumping for 4 s got its feet to 1.61 m and never past the
  outside of the wall. Before that, with the made windows, a jump could get
  a player in even through a small window, when `resolve_circle_aabb` pushed
  a body that landed half inside the wall out on the inside.
- **The game reads every size off the models**, so reshaping one in Blender
  is all it takes: a house's walls are its objects named `Wall...`, each an
  axis-aligned box at least 0.25 m thick, which the game remakes so it can cut
  holes in them; its `Floor...` objects tell inside from out; a door's or
  window's hole is the box round its `Frame...` objects; a hanging takes up
  its whole box. `build::PIECES` only names each kind, its tab and how it
  goes in. The script's docstring has where each model's origin must be.
