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
(`versionCode` in `mobile/android/app/build.gradle` is the build number: 9
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
  no Hangul. It need not be unique. Typing: on desktop the keys and Enter;
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
  jump, open to the sky, in `glass` at a tenth), found by name once the
  map has spawned (`arena::find_waiting_room`, `WaitingRoom`: the box
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
  over you, is there with you). **A bot goes for whoever is nearest
  within `hunt_radius`** (`arena::hunt`, after `ai::steer`, over its
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
  to the car's, `Driving::take_view`): the file's `car.overhead` up with
  the top of the screen north, zooming between `overhead_nearest` and
  `overhead_furthest`, nothing turning it. Everywhere else it is behind
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
  `assets/ui.ron`: `hp`, `hp_back`, `hp_rim`); top right a bag (the
  inventory, which does nothing yet) and under it the four resources,
  wood, gold, diamond and emerald, each a picture with its count beside
  it, 0; bottom right, in the jump button's very place (the jump button
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
  where a bow and dynamite go once bought; and over those a smaller
  button of three dots (`hud::ARENA_DOTS`), which does nothing yet; the
  stick stays, and nothing else of the town's bar is there (no balance,
  person, gear, gift, social, face or chat). The pictures are drawn in code
  (`hud::sword`, `bag`, `log`, `ingot`, `gem`, `emerald`, `dots`, through
  `hud::drawn` and a polygon distance, `polygon_distance`; tests sample
  each). Space still jumps on desktop. The punch's range band is drawn
  only in the arena now (`range::follow`).
- **Falling off the map** is the arena's alone (`island::bottomless`):
  the town has its sea back. The bots are kept on by `strands` as the
  town's AIs are; a punch knocks them off all the same, and they are dead
  `fall_to_die` down, as you are ("The fight", above).

### The PvP map itself

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
gone). The desk across each island's middle begins 0.1 m past the
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
  `island::glb::read`, has it right. The
  pond has no floor under its water, as the old town's had none: under
  the pond is the void.
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

## The app opens in Round Town

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
  the face and the gamepad do anything** (Hajun: "u dont need to make any button
  functional yet other than jump, joystick, settings"; the fist since
  2026-11-01, below, "The fist throws a punch"; the face since 2026-11-05,
  the car's switch, below, "The car you drive", where the fist is the
  reverse and the jump button the boost; the gift since 2026-11-06,
  which held in the car is its gun, there too; and the gamepad since the
  night of 2026-11-07, the game selector, above, "Games"); the rest take a press and drop it. The three round the jump button are placed from where
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
- The text is Bevy's built-in font, which is plain ASCII: no Hangul, and no ★,
  `…` or `·` either. The rating star is drawn in code (`builder::star_image`).

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
desktop build on 2026-10-31, scripted; not yet on a phone.

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
  482"), 1,000 coins, and the blue look; the name and the character chosen
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
  `RT_CLOCK_OFF=<seconds>` puts a copy's own clock that far out, for
  trying the town's clock out on one machine, whose clock both copies
  share. A
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
  and bronze medals in bold (the built-in font has one weight: the number
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
