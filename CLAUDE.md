# 동그라미타운 (RoundTown)

A third-person 3D world in Bevy 0.19, zoomed into first person all the way
in: a lobby town everyone walks about in, and minigames entered from it. One
codebase, three targets: Windows (`src/main.rs`), Android (`mobile/android`,
a cdylib built with cargo-ndk), iOS (`mobile/ios`, a Rust binary Xcode
bundles). It plays online against a server in Seoul (`server/`; `shared/` is
`roundtown_net`, what the two sides say and the rules both go by).

**What matters now** (Hajun, 2026-11-14): the main lobby, Round Town, and
Bridge Wars. House Builder, the build-battle game, is stalled: keep it
working, and spend nothing on it unasked. Since 2026-11-18 the selector has
the four games Hajun asked for that day, each played at a table: Blend In,
Knock Off, Bridge Race, Last Stand (`docs/minigames.md`). Having played
them (2026-11-19), Hajun likes Bridge Wars and Knock Off and had Bridge
Race's and Last Stand's cards turned off (`games::OFF`, since APK 44):
spend nothing on those two unasked.

As of 2026-11-25: `PROTOCOL_VERSION` 25 (since 2026-11-23: a seat of the
town is one of its 48 plots, so **APK 51 and older are told to update**;
APK 52 to 55 play on, those before 55 without the new things on the
made-up land), **APK 56** (`versionCode`, built 2026-11-25, not put on a
phone from here, `docs/building.md`), the server deployed at 19:44 UTC on
2026-11-24 with everything in `server/` and `shared/`. **All of it
committed and pushed, up to APK 56**: the land (`docs/land.md`: all 48
plots always filled, where each is thrown afresh as a channel opens,
someone walking every one, each made-up owner of a taste), everyone's
signs and cars, the town's slower buses, the bag (`docs/bag.md`) with its
window, its button and how it is scrolled and its 118 things, small
things on tops, the 23 cars (`docs/vehicles.md`), edit mode on your plot
and paint, sitting and lying down, the plots' residents and the things
that work (`docs/town.md`). The iPhone's last tag,
`ios-1.0.2-18`, pushed in the first hour of 2026-11-16 (whether Codemagic
built it is not known here), is protocol 18: its build is told to update
and stays offline until `ios-1.0.2-19` is built, whose tag is Hajun's to
ask for. Apple warns that from April 2027 an upload needs Xcode 27
(`docs/building.md`, APK 42).

## This file, and where the rest is

This file is the rules and how things stand. The full account of everything
(what was built, why, in Hajun's words, what was tried and turned down, what
was and was not checked) was moved out of it, word for word, into `docs/` on
2026-11-14, because it had grown to 6,700 lines read by every session.

**Read the file for an area before changing that area. Do not read them
otherwise.** A section named in quotes anywhere ("Games", "The chat") is a
heading in one of these:

| Before touching | Read | Old sections in it |
|---|---|---|
| rendering, sky, lights, see-through, frame rate | `RENDERING.md`, `docs/rendering-and-frame-rate.md` | "Before touching rendering", "It aims for 120 frames a second" |
| colours and words on the screen, how a window looks | `docs/ui-style.md` | "Every window is drawn as the friends' panel is", "The screen's colours and words are in `assets/ui.ron`" |
| building, packaging, APK numbers | `docs/building.md` | "Building" |
| the players' models, walk and punch | `docs/models.md` | "Models" |
| the sign-up's pages, anything typed into | `docs/signup.md` | "The first time, you sign up" |
| **Bridge Wars** | `docs/bridge-wars.md`, `bridgewarsgameevent.md` | "Games" |
| Bridge Wars' map | `docs/bridge-wars-map.md` | "The PvP map itself" |
| the four games' tables (joining, waiting, fees, prizes, results), the fight kit they share with Bridge Wars | `docs/minigames.md` | |
| **Blend In** | `docs/blend-in.md`, `blendingameevent.md` | |
| **Knock Off** | `docs/knock-off.md`, `knockoffgameevent.md` | |
| **Bridge Race** | `docs/bridge-race.md`, `bridgeracegameevent.md` | |
| **Last Stand** | `docs/last-stand.md`, `laststandgameevent.md` | |
| the opening, the buttons round the screen, the camera's hole, settings | `docs/town-hud.md` | "The app opens under a cover", "The app opens in Round Town", "Settings" |
| friends, groups, messages | `docs/friends.md` | "Friends, groups and messages" |
| the chat, the quick chat, fonts, Korean and Vietnamese, the word filter | `docs/chat-and-text.md` | "The chat", "The font is Inter" |
| netcode, the login, the town's clock | `docs/online.md`, `SERVER.md`, `MULTIPLAYER.md` | "Playing online" |
| a map's model, collision | `docs/maps-and-collision.md` | "The islands are Blender models" |
| the town's 48 plots, whose each is and what fills the rest (`land`, `roundtown_net::land`, `premade`, `server/src/rooms.rs`), the signs | `docs/land.md` | |
| the bag: what you own, the shop, gems, buildings put down on your plot, a list flicked, edit mode and paint (`bag`, `build::stand`, `build::paint`, `customize`, `flick`, `server/src/bag.rs`) | `docs/bag.md` | |
| fountain, music, town houses, town objects and their editor, the plots' residents (`resident`), talking, the bubble in the world (`bubble`), sitting and lying down (`rest`), what works (`working`) | `docs/town.md` | "The fountain runs", "The town has music", "The town's houses", "The town is data", "The town holds 32 players", "You can talk to anyone" |
| the buses, the car you drive | `docs/vehicles.md` | "The town's buses", "The car you drive" |
| the camera | `docs/camera.md` | "The camera stands where you zoomed it" |
| House Builder, the shop's pieces, `assets/mybuilds/` | `docs/house-builder.md`, `homebuildersgameevent.md` | "House Builder", "Building your house" |
| anything under `mobile/ios` | `IOS.md` | |

`docs/bridge-wars.md` is long, a log with the newest work first: read its
contents list, then only the entries to do with what you are changing.

**Keep this file short.** When work is done, write it up as a dated entry at
the top of the right `docs/` file: what, why, Hajun's words, what was checked
and where, what was not, which calls were yours. Change this file only when a
rule or how things stand has changed, and then in a line or two. No logs here.

## Working with Hajun

- Their words decide. Where they have not said, decide as the genre would
  (Bridge Wars is Minecraft Bedwars seen from above), and say which calls
  were yours.
- They edit `assets/ui.ron` (colours, words, the car's numbers) and
  `bridgewarsgameevent.md` (Bridge Wars' items, prices and timetable) by
  hand, sometimes while you work: read them again before changing a number.
- Every other game has a short sheet like it at the root
  (`<game>gameevent.md`, since 2026-11-19: its events in order, items,
  prices, coins), written from the code: change a number in a game and its
  line there with it.
- Say what was checked where: unit tests, the desktop build (scripted), two
  copies against a server on this PC, the VM, a phone. A phone is rarely
  attached; never let "built" read as "seen on the phone".
- "Remove" usually means turn off: keep the mechanism behind a switch
  (`sound::PLAYING`, `hud::DOTS`, `hud::GIFT`, `drive::GUN`,
  `utility::SECOND_ALWAYS`, `customize::HINTS`, `see_through::FURNITURE`).
- **Every window looks as the friends' panel does** (Hajun, 2026-11-14: "I
  want to use this same design style throughout the project"): friends,
  settings, the game selector, Bridge Wars' shops and its "You're out!" are
  made of `src/window.rs`'s pieces in `colours.window`, and a new one is
  too: dark and see-through, nothing dimmed round it, tabs or a title
  along the top, a bare white cross inside the corner, green for yes and
  dark red for no. "Too dark" has meant more see-through, not greyer.
- What stands over the world and is no window (your name's plate, the
  time, the minimap) is the plate's grey, `#666666`; no blue tint anywhere.
  Sizes and layout stay in code; colours and words in `ui.ron`.
- In Bridge Wars the camera is straight overhead: judge every visual from
  there, and look at small things enlarged before calling them right.

## Rendering: hard rules

Read `RENDERING.md` first. It has what was tested on the phone and what not
to try again.

- **Windows and Android run the same rendering code.** `src/sky.rs` has no
  `#[cfg(target_os = ...)]`. Materials, lights and geometry are shared, with
  real `AlphaMode::Blend` and `AlphaMode::Add` on both.
- **`NoIndirectDrawing` on the camera (`setup_world`, `src/lib.rs`) is
  load-bearing.** Without it the Adreno 840 silently drops the sorted
  transparent phase: water, clouds, sun glow gone, no error. Never remove it.
- **Do not fake transparency with opaque geometry** (it made the sky grey).
- **Test on the device, do not infer from the engine's source** (the adb
  build-install-screenshot loop is in `RENDERING.md`).
- Unlit things are tonemapped too (TonyMcMapface, kept): drawn brighter than
  their colours (`sky::UNLIT_GAIN` 5, `fountain::WATER_GAIN` 2.5,
  `range::GAIN` 2.5). `ClearColor` is not tonemapped. Bevy 0.19 does not
  apply exposure to `emissive`: a glow is so many times its colour.
- Lights are clustered on the CPU on every platform
  (`cluster_lights_on_the_cpu`), into one cell while no lamp is lit and
  Bevy's own grid while any is (`lamps::cells`). Shadows are off
  (`SUN_SHADOWS`). No real mirrors: every camera gets its own shadow maps.
- **The world is drawn in one pass** (`src/pass.rs`, `ONE_PASS`; since
  2026-11-15, not yet seen on a phone): the solid things, then the
  see-through, and neither the four samples nor the depth is kept after
  it. Anything that reads either once the world is drawn (a second camera
  on the window, depth of field, a prepass read back) has to be looked at
  there first.
- A day and a night pass every 20 minutes by the town's clock (`sky.rs`;
  `RT_TIME_OF_DAY=day|sunset|night|sunrise|<seconds>` stops it). One
  directional light is sun and moon. The sky hangs from a `SkyDome` that
  follows the camera.
- What gives off light at night is a list by material (`lamps::SOURCES`):
  street lamps' `lightbulb`, Bridge Wars' anchors' `spawn`; and two of the
  shop's things by kind, the Chandelier and the Lamp With Shade
  (`lamps::BY_KIND`, since 2026-11-25). At most 24 point
  lights (`MOST_LIT`), none by day.
- See-through walls (`see_through.rs`) happen only inside a house, with
  alpha to coverage; never seen on a phone from here. Nothing of a body
  fades (`of_a_body`: five looks have eyes and a mouth that are meshes of
  their own), nor furniture (Hajun's, 2026-11-24: a bed, a chair or the
  couch in the way stays solid and hides you; `see_through::FURNITURE`,
  off), and a part no longer in the way
  has its own material back the frame after. What is in the way is looked
  for again only as you or the camera move, and five times a second
  (`Looked`): every frame, it was a tenth of the world's thread.
- The fountain's water is the one custom shader (`fountain.wgsl`, built in).
- A mesh with no vertices must not go into `Assets<Mesh>` (Bevy logs a
  use-after-free for each).

## Frame rate: hard rules

- The target is 120 frames a second on Windows (`src/pace.rs`) and Android
  (`MainActivity` asks the surface for 120). iOS is held to 60.
- **Anything that runs every frame writes only what has changed**
  (`set_if_neq`, or compare first). Writing a `Transform` re-poses a
  skeleton; writing a UI `Node` lays the whole screen out again.
- Every schedule runs on one thread (`one_thread_per_world`), except the
  render world's `Render` schedule on iPhone and Mac; and Bevy's own
  systems have one worker to split their loops over (`worker_threads`).
  The gamepad plugin is off, and Bevy's audio while the music is
  (`sound::PLAYING`): a new sound needs that turned on.
- The phone draws at three quarters size: `RENDER_SCALE` 0.75 is in both
  `MainActivity` and `src/lib.rs`; keep the two the same.

## The UI's colours and words: `assets/ui.ron`

- Every colour and every word drawn over the world is in `assets/ui.ron`,
  for Hajun to edit by hand; so are the numbers of the car (`car`), of the
  punch (`melee`) and of Bridge Wars offline (`arena`). `src/ui.rs` reads
  it; `ui()` is a global handle; `ui::fill` fills a word's `{something}`.
- It is compiled in (`include_str!`): a phone shows what it said when its
  app was built. On Windows it is read again as it changes on disk, and
  what is on the screen is drawn again (`ui::UiChanged`).
- A colour is `"#rrggbb"` or `"#rrggbbaa"`. Unknown or missing fields are
  errors (`deny_unknown_fields`); a number the game would break on is
  refused (`Car::check`, `Arena::check`, `Melee::check`).
- **Adding one**: a field on the struct in `src/ui.rs` and the line in the
  file; read it with `ui()` where the node is spawned. A screen that stays
  up needs a system that redraws on `Res<UiChanged>`. A bundle returned
  from a function must own its strings (`.clone()`, edition 2024).
- A press on a button is the button's alone: every button over the world
  carries `hud::TakesPress` and is pressed through `bevy_picking`. While a
  dialog is up it holds `hud::DialogUp`: you stand still and every press is
  its own.

## Build, run, ship

- Desktop: `cargo run`. Tests: `cargo test --lib`, `cargo test -p
  roundtown_net`, `cargo test -p roundtown_server`.
- Server: `cargo build -p roundtown_server`; `server/deploy.sh` builds it
  for Linux here and puts it on the VM (`SERVER.md`). **Never
  `--workspace`**: it would build the game with the server's half of
  replicon.
- **Deploy the server with every change to `shared/`.** Before: read the
  VM's journal for anyone connected, and keep the running binary as
  `roundtown-server.prev-<date>`. After: connect with this PC's own account.
- Android: the `release-android` profile, a manual strip, then Gradle; the
  commands are in `mobile/android/app/build.gradle`'s closing comment and
  `RENDERING.md` ("Build gotchas"). `versionCode` there is the APK's
  number. What `mobile/android/app/assets-left-out.txt` names stays out of
  the APK; tests fail if that leaves out something the game loads. One APK
  is kept, the newest, `donggeurami_town-release.apk`: no `prev-` copies.
- iOS cannot be built here: Codemagic builds it on a pushed tag
  `ios-<version>`. It can be type-checked here, and is before every tag:
  `bash mobile/ios/check_on_windows.sh` (`IOS.md`; nothing is linked or
  run). Keep code behind `target_os = "ios"` to call sites all the same.
  An iPhone writes `log.txt` to its Documents (`src/log_file.rs`): ask
  for it before guessing at a failure.
- Desktop testing knobs: `RT_SERVER=<addr>|off`, `RT_UNSECURE=1` (debug
  build, a server with no key), `RT_FREE_CURSOR=1`, `RT_INTRO=off`,
  `RT_SIGNUP=1`, `RT_TIME_OF_DAY`, `RT_CAR_AT`, `RT_CLOCK_OFF`,
  `RT_ARENA_FROM=<seconds>`, `RT_ARENA_PURSE=<count>`,
  `RT_ARENA_TEAM=red|blue|green|yellow` (offline),
  `RT_ARENA_WATCH=1` (offline: a game of Bridge Wars begun by itself and
  played by the computer alone, your seat too, to look on at; Tab is the
  next player's view; `src/arena/watch.rs`), `RT_UNFOCUSED=1` (the
  window shown but not made the active one),
  `RT_SAFE_AREA=<left>,<right>` (a camera's hole made up),
  `RT_STAND_WAVE=<n>`, `RT_STAND_PURSE=<count>` (Last Stand from a later
  wave, with coins to spend; offline, and the server's own),
  `RT_LAND_SIGNS=all` (a sign on every plot of the town, to look at),
  `RT_LAND_ORDER=numbered` (offline: you on Hajun's plot 1 and every
  made-up owner on the plot of their number, not thrown),
  `RT_BACKGROUND=1` (below). Two copies need
  their own `APPDATA`. A server on this PC: `RT_PORT=5443
  RT_ALLOW_SKIP=1 target/debug/roundtown-server.exe`, and the copies with
  `RT_SERVER=127.0.0.1:5443 RT_UNSECURE=1`.
- To see a change in the running game, script the desktop build with a
  throwaway module (taps as `WindowEvent::TouchInput`, which picking takes;
  `Screenshot::primary_window()` for pictures) and take it out again before
  finishing. Posted mouse clicks do not reach the world.
- **Every such run, and the check after a deploy, is hidden:
  `RT_BACKGROUND=1`** (Hajun, 2026-11-16, at work on this PC while the game
  was being tried: "dont open it as active tab but rather background
  tab"). The window is made hidden and without the keyboard, takes no
  mouse, and the game runs, draws and takes its pictures all the same
  (`in_the_background`, `src/lib.rs`); the run has to end itself.

## Playing online

- **The app never waits for the network.** It opens offline and connects
  in the background (`src/net.rs`, `src/login.rs`); with no server it is
  the offline game exactly. **But the opening's cover** (Hajun's,
  2026-11-24, since APK 54): it stays until the server has said what the
  town is, while there is one that may answer (`Link::hopeful`) and no
  longer than two seconds once all else is there (`intro::SERVER_WAIT`).
- Three crates: the game (root), `shared/` (`roundtown_net`: every message,
  registered in one order by `protocol`, and every rule both sides go by),
  `server/` (headless Bevy, replicon and renet over UDP 443).
- **replicon's protocol hash is only the names and order of the types.** A
  field added, or one meaning something new, needs `PROTOCOL_VERSION`
  bumped, or an old app connects and breaks. An app whose version differs
  is told to update and stays offline; a change that leaves what is sent as
  it was keeps the apps already out playing. Something new that either
  side can do without may be a variant added at the END of `Ask` or
  `Tell`, with no new number, if the server says it only to a device that
  asked (`Ask::Channel`, `players::Watching`; `docs/online.md`).
- Your body is yours: moved here at once, told to the server 15 times a
  second while it moves, which checks the move could have been made (no
  faster than 52 m/s, the car's boost with slack; a rise no faster than a
  jump with 4 m in hand, which the fountain's throw and a leap potion
  pass). Everyone else is drawn two updates behind. Players walk through
  each other.
- Everything that goes by the clock goes by the town's clock, the server's
  online (`src/clock.rs`): buses, the fountain's jet, the day.
- The login: the device's secret (`device.id`) over HTTPS to the VM, the
  server's certificate pinned (`login::PINNED`); the VM's addresses are
  compiled in (`net::SERVERS`). A new VM means a new app.
- A game ends over two ticks on the server (`close`, then `end`): replicon
  writes a despawn only to a device the entity is still shown to.
- A phone closes its connection whenever its app leaves the foreground
  (`net::lifecycle`). **A seat in a game of Bridge Wars waits for it**
  (`Arenas::keeps`, `returns`): the body stands in the game, to be hit; an
  app only put away carries on; one closed and opened again is put back
  (`Tell::Back`) while someone else of the people in the game is
  connected; with nobody connected a game goes on a minute.
- The VM allows the tap that skips waits (`RT_ALLOW_SKIP=1`): take it out
  before real players come.
- **The website** is a repository of its own, `xius-snu/donggeurami-town`
  (GitHub Pages, since 2026-11-24; a folder beside this one): the support
  page and the privacy policy, which lists what the server keeps. **Change
  `privacy.html`, and its date, with anything newly kept, written down or
  handed to anyone** (a sign-in, anything sold, a report): `docs/online.md`,
  "The website".

## Maps and models

- Maps are Blender models (`assets/*.glb`, `island::Venue` names them):
  `roundtownmap.glb` the town, `pvptownmap.glb` Bridge Wars, `lobbymap.glb`
  and `homemap.glb` House Builder's, `testhomemap.glb` home, `iceisland.glb`
  Knock Off, `bridgeracemap.glb` Bridge Race, `laststandmap.glb` Last Stand
  (these two stood 0.25 m up as they load, to Bridge Wars' heights:
  `island::lift`); Blend In is played on the town's (`island::Scenery`).
  `src/island.rs`
  reads every triangle of each; nothing is kept by hand. All are loaded at
  startup and stand at the origin: **`Venue` keeps them apart**, and
  anything spawned without one is drawn everywhere and collides with
  nothing.
- A metre in Blender is a metre in the world: resize in Blender, never at
  spawn. Models face Blender's -Y. The loader turns every model half round:
  a point (x, y, z) read out of a `.glb` is (-x, y, -z) in the game.
- **A solid has to be a closed shape**, with normals pointing out; a walkable
  top must face up. Faces steeper than 50 degrees are walls; anything
  thinner than 0.25 m can be walked through. A body steps up anything
  under half a metre (`LAND_STEP_UP` 0.495); half a metre takes a jump.
- Water is a part whose material is called `water`; a material named with
  `.001` after it is the same material. A part with no material loads near
  black. Skinned meshes are not land. Where a map has nothing underfoot
  there is sea (`WATER_Y`), but under Bridge Wars' and Bridge Race's,
  which have the void (`island::bottomless`).
- Export with Apply Modifiers (arrays and mirrors come out as one piece
  otherwise). A texture in a `.glb` needs Bevy's feature for its format
  (`jpeg` is on).
- After a new export of Bridge Wars' map: the constants in `shared/` have
  to agree with it (tests read the map), the bots' map has to be baked
  again (`RT_BAKE_NAV=1 cargo test --lib nav_bake`), and the server
  deployed. So too Bridge Race's (`race_bake`) and the town's, whose grid
  is Blend In's crowd's (`blend_in_bake`), and the middles of whose plots
  are written for the server (`roundtown_net::land::HOMES`; a test of
  `land`'s says when they no longer agree).
- Every body is Hajun's `playercharacter.glb` or `playercharactergirl.glb`
  and their two recolourings each (`lobby::LOOKS`; the same list is
  `roundtown_net::BOY_LOOKS`, `GIRL_LOOKS`, which the server checks: deploy
  it with any new look). Limbs are bound by bone name (`walk_bone_kind`)
  and turned about the body's own axes.
- A body walks at 7 m/s (runs at 10 in Bridge Wars), jumps 1.64 m, and
  bumps as the old model's shape: 0.42 m across the legs, 0.9 the body
  (`LEG_RADIUS`, `PLAYER_RADIUS`). Every body is moved by `move_bodies`
  from its `Intent` (`src/lib.rs`): yours from the stick
  (`read_player_input`), an AI's from its steering, a bot's from the
  brain; everyone else's walk is read off how they moved. To put a body on
  another island, `lobby::send` it.

## Names, typing, chat, friends

- **What may be written** is one rule for names, chat and messages, on both
  sides (`roundtown_net::text`): English, Korean, Vietnamese, and nothing
  the fonts cannot draw. Lengths are in widths, a Hangul syllable two: a
  name is 2 to 16 (so up to 8 syllables), a line of chat and a message 80.
  A letter typed in pieces is put together (NFC, `text::clean`).
- The font is Inter (`src/font.rs`, compiled in), and Pretendard for
  Hangul as a per-script fallback (Bevy 0.19's text is parley and
  fontique). Every text asks for fixed-width digits (`font::sized`).
- Common swear words are starred out of chat and messages by the server
  (`text::masked`), and a name with one in it is refused ("Pick another
  name.", `profile::name`, `profile::rude`). A basic filter, by its lists.
- One box is typed into everywhere (`signup::NameBox`): desktop keys and
  the input method through one ordered stream (`WindowEvent`); Android
  reads the activity's own text each frame; iOS gets keys. **Android is
  reached only through native methods `MainActivity` calls** (`imeInset`,
  `imeAction`, `safeInsets`, `clipLength`...): calling GameActivity's C API on
  `AndroidApp::activity_as_ptr()` crashed the app at launch.
- The first run shows three pages: Start, a name, a character
  (`src/signup.rs`); kept in `profile.json` beside `town.json`, and online
  it is the account's (`Ask::Profile`, Postgres `players`).
- The chat (`src/chat.rs`, `server/src/chat.rs`), down the left in the
  town and in Bridge Wars: its words alone with a thin stroke, three
  lines in sight, none faded (`colours.chat.strengths`, all 1; other
  numbers fade them by place), scrolled by a drag; the dark box and the box
  to type into come only while a line is typed (the keyboard button). A
  line goes to everyone in the room; nothing is kept or logged. No block
  and no report yet, nor for friends' messages: both are wanted before a
  release to the shops (`MULTIPLAYER.md`).
- Friends (`src/friends.rs`, `server/src/social.rs`) are the server's
  alone, none offline: a `#1000`-`#9999` tag after every name, requests,
  groups of up to four whose leader alone starts a game, messages that wait
  in an inbox (a reply sent sees the one it answers off). Whoever comes
  into a group is moved to its leader's channel of the town, or the whole
  group to one with room (`players::gather`). The server reads the
  database once and goes by its own copy.

## The lobby, Round Town, as it stands

- The app opens under a plain black cover that fades once the town is
  loaded (`src/intro.rs`, since 2026-11-17) and the server has said whose
  the plots are and what stands on them (since APK 54: you are on your
  plot and the town is the server's before any of it is seen; "Playing
  online" has the limits); the "Loading..." bar and the
  drop through a circle are turned off (`SAYS_LOADING`, `DROPS_IN`). No
  menu. You arrive in the middle of your plot, facing the fountain
  (`roundtown_net::land`, since 2026-11-20, the server too).
- **The town has 48 people in it** (since 2026-11-24, Hajun's: "there
  should be a character for all 48 plot ... if theres just 1 real player
  online in a channel, then there should be 47 npcs"; sixteen for the day
  before, 32 until then, wanderers round the pond): whoever is in your
  channel (48 people to one), each on their plot, and **a resident** of
  the computer's on every plot nobody of the channel has, in its owner's
  look: under a made-up owner's own name, and on a real person's land
  "{name}'s bot" (`words.land.bot`; `src/resident.rs`, `docs/town.md`).
  Each device walks its own, and what each is about is worked out from
  the town's clock and who they are (`resident::plan`), the same on
  every device: about its plot, sat or lying in its house (`rest`), and
  now and then over at the plot of someone real. Its way is found over
  the town's shape a little each frame (`resident::Search`). Bodies are
  spawned only by `lobby`'s own functions (`spawn_player` and its kin,
  `spawn_resident`). Names over heads (`src/tags.rs`) are everyone's but
  your own.
- **The buttons** (`src/hud.rs`, as Hajun sketched them): top left your
  name on a plate (with its tag online), the gear (settings), and your
  balance under them; down the right edge two people (friends, with the
  inbox's count in gold) and a gamepad (the game selector). The person
  button that stood by the gear and the gift over the two people (the
  car's gun, itself off since, `drive::GUN`) are turned off since
  2026-11-14, "too many lobby ui buttons" (`hud::PERSON`, `hud::GIFT`).
  Round the jump button a
  box (your bag's button, since 2026-11-22, with the box of its window's
  first tab on it since APK 47, Hajun not liking the sack, `hud::SACK`;
  a face until then, which put you in
  the car), a speech bubble (a fan of things to say
  at a press, `src/quick.rs`, `words.chat.quick`), a small keyboard over
  it (the chat's box) and a fist (a punch; the car's reverse); the stick,
  whose middle is set where the finger first goes down, within half its
  radius. In your car the bag's button is put away and a door stands in
  its place, which gets you out (`hud::ExitButton`; half way down the
  left edge until APK 49). By the left edge, just above half way
  down, where the chat's lines begin, which step aside of it: a house and
  pencil while you stand on your plot, which begins changing it, and in
  edit mode three, the house and pencil over that place, the brush in it
  and the tick under it (`hud::LeftEdge`, `src/customize.rs`, below). Under
  your balance, your gems, the two numbers one size and the gem as wide
  as the coin (Hajun's, APK 47). On Windows the stick and the jump button
  take the mouse.
- **The bag** (`src/bag.rs`, `src/bag/panel.rs`, since 2026-11-22, APK
  45; not seen on a phone; drawn from Hajun's `bagandinventorypanel1.png`
  and `2`): a window of two tabs, what you own and the shop, categories
  down the left, all of them in sight with none scrolled to, cards three
  to a row, each thing's name in its rarity's colour (`colours.bag`). The
  shop opens on All, has each thing's price under its name, red when you
  cannot afford it (APK 47), and a thing
  pressed there has Buy and Sell over its picture, a little see-through,
  both always there and the one that cannot be pressed greyed (Hajun's,
  later that day, APK 46). Its lists are scrolled as a phone's are: a
  flick goes on and slows to a stop, by Android's own arithmetic
  (`src/flick.rs`; the bag's lists alone so far), and a swipe across the
  window changes its tab (Hajun's, APK 48). **118 things** (since
  2026-11-25, APK 55): every line of Hajun's `assets/itemrarities.txt` in
  its order, 95 buildings (House Builder's sell only their nineteen,
  `PieceDef::house_builder`) and 23 cars, each at its rarity's price, of
  seven rarities, Transcendent the highest, sold back for half; the six
  houses his list marks "(gem)" are bought with gems and sold back for half
  of them (`roundtown_net::bag`: **a test holds the file and `THINGS`
  together: change both, and deploy the server**). Everyone begins with ten gems, a car
  and a street lamp. Online it is the account's (Postgres, `players.gems`
  and `.bag`): buying and selling are the server's; offline the device's
  own, kept in `bag.json`. A tap on a car in it drives that one; a tap on a
  building, standing on your plot, puts it down. The Chandelier and the
  Ceiling Fan hang only from a ceiling (`build::Mount::Ceiling`), the
  Painting on a wall (since APK 56). **Small things** (`PieceDef::small`,
  29: what goes on a desk or a table) dragged over a top go up onto it
  (`build::top_under`), and ride what they stand on (`stand::Rides`).
- The top bar and the chat keep clear of the camera's hole and an
  iPhone's island (`src/safe.rs`: Android's cutout, by `MainActivity`,
  seen working on Hajun's phone; an iPhone's safe area, read off its
  window since 2026-11-15, on both sides, not yet seen on one). The
  right end of the bar is moved no further than the fan of things to say
  leaves room, and the chat's small button steps in from an island.
- The gamepad opens the selector (`src/games.rs`): four cards, two to a
  row, Bridge Wars, Blend In, Knock Off (50 coins each) and Home Builders
  (100), each price the coin's picture and the number. Bridge Race's and
  Last Stand's cards are turned off since 2026-11-19 (`games::OFF`: the
  cards alone, the games and their tables are as they were), and so is
  the card for one to come (`games::COMING_SOON`).
- **All 48 plots are always filled** (`docs/land.md`, since 2026-11-23,
  Hajun's; not seen on a phone). The plots are the map's `plot` planes,
  read as it spawns, **numbered as Hajun numbered them** (`plotorder.png`,
  `roundtown_net::land::HOMES`: his plot 1 is at (-90, 25) in the game),
  and **a seat of the town is a plot**. **A number only says which plot**
  (his, 2026-11-24, to point at one by): **where anything is, is thrown
  as a channel opens** (`land::Order`, since APK 53), you, real people's
  land and each made-up lot on another plot every time, and as it is
  until the channel has been empty five minutes. The server says whose
  each is (`land::Estate`, `rooms::Towns`, `Tell::Plots`, `land::Deeds`
  on a device): whoever is in your channel has a plot, the one their own
  land fills or else a made-up lot's, one nobody stands on first, and
  keeps it in a game and for five minutes after their connection goes
  (`land::GRACE`); the rest are filled with real people's land that has
  anything on it, the worthiest first by the shop's prices, and then
  with made-up land (`roundtown_net::premade`, 48 furnished plots a test
  holds to the game's building rules, `build::premade_check`; since APK
  56 each owner of one of sixteen tastes, two thirds of the houses one
  floor and none over three). **Nobody's
  land is in two channels at once**: whoever comes goes to the channel
  their land fills a plot of (after their group's), and has nothing left
  in any other. Offline the device throws its own order (`land::thrown`).
  **Change `HOMES` or a made-up lot and deploy the server.**
- **Your plot** (`src/land.rs`, since 2026-11-20): a sign
  (`plotsign.glb`) stands on every plot, a metre and a half in
  from the right hand end of its front, the side with a street along it
  or toward the pond, saying "{name}'s land" (`words.land.sign`) on both
  faces of its board, the words hung on it only within 45 m of the
  camera (`land::word_the_near`). **The words are a thing in the world**, a picture
  of them drawn once with the game's fonts and hung on the board (since
  2026-11-21): never text on the screen laid over it, which Hajun turned
  down. The sign is solid, a box laid over the town's shape
  (`Islands::lay_over`). **What you own
  stands on it** (`src/build/stand.rs`, since 2026-11-22): a building out
  of your bag is put down as a piece is in House Builder, by the same
  code (`build::Site`, `Yard`: the same ghost and menu, nothing of it off
  the plot, one house to a plot), is out of your bag until it is tapped
  and binned, is kept by the server in the plot's own space, so that it
  stands the same on whichever plot your seat has, and is seen and bumped
  into by everyone in your channel (`bag::Tell::Theirs`). A house binned
  takes what is in it into the bag, and what is in the way is the
  shapes of things, not the boxes round them (`build::in_anything`;
  both since APK 56, Hajun's). **Nothing of
  yours answers a tap but in edit mode** (`src/customize.rs`, since APK
  49, Hajun's; not seen on a phone): on by the button by the left edge,
  or by bringing a thing out of your bag, and off by its tick, or by
  going more than 3 m off your plot, into your car or out of the town.
  **While it is on** (since APK 52, Hajun's) "Editing Mode Enabled"
  stands on a plate top centre and the edges of your plot are lit; with
  it off, a tap on a thing of yours makes the edit button beat
  (`customize::Pointed`), and the words that hinted at it are turned off
  (`customize::HINTS`).
  Its three buttons are the hand (a tap picks a thing up, as before), the
  paintbrush, and the tick. **With the hand** (since APK 50, Hajun's) a
  thing goes where the finger takes it, through whatever stands there,
  and the tick of its menu is grey where it cannot stand
  (`build::drag_freely`, `mark_misfits`; House Builder's pieces are
  carried as they were); a tap anywhere else lets go of it, put down if
  it can stand there and put back where it stood if not
  (`editor::Edit::Away`, `stand::PutBack`); and a house picked up takes
  what stands in it along (`stand::Rides`). **With the brush a tap paints
  the part under it** (`src/build/paint.rs`): a part is a material of
  Hajun's model, by its name in Blender (a bed's `blanket`, a house's
  `plain_wall`), the colour chosen in a window behind the round button
  beside the brush ("Paint Color", `customize::picker`: a wheel, white in
  its middle, a bar for how bright, and a row of the last seven colours
  painted with, kept on the device in `paint.json`); with the cross
  beside that button in hand, a tap takes the paint off. What is painted
  is kept beside the bag (`roundtown_net::bag::Paints`), by the server in
  the bag's own document, is let go of when the thing is taken back into
  the bag, and is told to everyone in your channel whose app asked for
  colours (`bag::Ask::Colours`, `Paint`, `Unpaint`, `Tell::Painted`:
  variants at the end, protocol 24 still, so an older app plays on and
  sees none). **A new
  material name in a model is a new part; one renamed loses its paint.**
  **A door or a window is kept as its wall's number**, the walls of its
  house ordered by their corners to the millimetre, measured in the
  house's own space (`build::wall_order`, `within`, since APK 51): never
  work a model's parts out through the world, where the last bits of a
  number differ by where it stands, which had a door in one wall on one
  phone and in another on the next.
  Where the land is going is Hajun's to say (`docs/land.md`).
- **You can sit and lie down** on a chair, a sofa or a bed on anyone's
  plot, and lie in the bath (`src/rest.rs`, since APK 51, Hajun's; the
  things of 2026-11-25 since APK 56; not seen on a phone): a
  bubble over each place of one you stand within 1.5 m of, a tap, and you
  are there in its pose in a fifth of a second (`roundtown_net::Pose`:
  sat on a chair, sat back on a sofa, lying on a bed); the jump button
  gets you up, where you stood before. Everyone in your channel sees it
  (`Ask::Resting`, `Ask::Rest`, `Tell::Resting`, at the end, protocol 24
  still). **Every such bubble is a thing in the world** (`src/bubble.rs`:
  the one over someone to talk to as well): a sheet 0.62 m across turned
  to the camera, not a button on the screen, which Hajun turned down.
- **Some things work** (`src/working.rs`, since APK 56, Hajun's ask, my
  numbers): a treadmill's belt carries whoever stands on it toward its
  open end at 5 m/s, and the ceiling fan turns.
- You can talk to any AI (`src/talk.rs`), ride the four buses that go
  round the roads by the clock (`car.rs`, `roads.rs`: the way is read off
  the road's paint; 5.25 m/s, 10.5 on the open road, three quarters of
  what they were, since 2026-11-21), drive a car of your own (`drive.rs`,
  out of your bag, its numbers in `ui.ron`; everyone else in your channel
  sees the car since 2026-11-21, `Ask::Drive`, and which since 2026-11-25,
  `Ask::Car`, `Tell::Cars`), and be thrown by the fountain's jet. **There
  are 23 cars** (`drive::CARS`, since 2026-11-25): `car1.glb`, the
  `italia` of a pack made 1.25 times as big, and the pack's other 22 in
  `assets/cars/` at that size, each as fast as its rarity (`car.speed`,
  `car.pickup`). The pack as downloaded and its previews stay out of git
  (`.gitignore`: the repository is public and its licence forbids handing
  it on; `docs/vehicles.md`).
- The camera orbits you 1.5 to 18 m out, and zoomed all the way in is your
  eyes. **Nothing in the world moves it**: walls never bring it nearer. On
  a phone the stick brings it round behind you after 1.5 s, and a double
  tap swings it there (not in edit mode, Hajun's, 2026-11-24). In the car
  and in Bridge Wars it is fixed straight overhead.
- Town objects are data (`src/map.rs`, a catalogue and `town.json`), edited
  by tapping (`src/editor.rs`), offline only. The edit menu's buttons stay
  on the object, each just clear of it as it is drawn (`editor::Reach`):
  never clamp them to the screen.
- The balance is the server's online (Postgres); the games' fees and
  prizes change it, and the bag's shop. The music is turned off
  (`sound::PLAYING`), and Bevy's audio with it.
- Settings (`src/settings.rs`): Graphics and Controls are empty; Accounts
  has your name on a row with a pencil (Edit name, a box over the window),
  sign-in buttons that do nothing yet, and Delete account, which asks once
  more in a box over the window.

## Bridge Wars, as it stands

Hajun's `bridgewarsgameevent.md` has the items, prices, rates and timetable,
and the game is as that file has it. The rules both sides play by are
`roundtown_net::bridge_wars` (`shared/src/lib.rs`).

- **A game**: 50 coins (`bridge_wars::FEE`, back to whoever leaves before
  the fight), each person paid by their team's place, 130, 60, 10, 0
  (`PRIZES`, `places`); the selector, a black loading screen, a glass
  waiting room 150 m over the map; ten seconds after the last person joins the computer
  takes the seats left, its players coming in one at a time and going
  about the room each as who it is (`bots::mingle`, `BOT_GAP`); a
  countdown; 20 minutes, with no clock on the screen until the last ten
  seconds (`show_board`). **Which team anyone is on is drawn nowhere until
  "Fight!"** (`Fighting::teams_out`: names in the colour of words, no bar
  over a head, no teammate on the minimap, and where anyone stands in the
  room thrown, not their seat's). **Always 2v2v2v2**: eight
  seats, four teams, a team to an island (seat modulo four); people are
  teamed with each other first, and a group is seated together; which team
  they begin on is thrown for each game (`seat_order` on the server,
  `thrown_seat` offline). Online the
  server runs it (`server/src/arena.rs`); offline the device does, with
  seven bots (`src/arena.rs`).
- **The map** (`pvptownmap.glb`): the town's middle, four team islands 130
  m out on the axes, red, blue, green, yellow, four smaller islands on the
  diagonals, and the void: fall 30 m and you are dead. Bridges are not in
  the map, they are built.
- **Winning**: each island has its team's anchor. With it standing, the dead
  come back in 4 s; with it broken, they are out. The last team with anyone
  in play wins. At 14:00 every anchor breaks; from 15:00 zombies come, two
  and then twice as many every 36 s, to fifty, all over the map
  (`zombies_at`, `zombie_spot`); at 19:30 everyone is told the end is
  coming, and at 20:00 it is a draw; the generators quicken on the way
  (`TIMETABLE`).
- **Zombies** (`roundtown_net::horde`, one mind for the server and for
  offline): each goes for whoever of the players is nearest that it can
  see, at 6 m/s, over anything that can be walked, a bridge too, and hits
  for 20. **Nothing hurts one**: a blow or a blast only knocks it a
  little, a bus lays it flat, and the void alone takes it, for 12 s. No
  bar over its head.
- **Dying** takes all you hold (the four resources, your sword, what is on
  your buttons, your metres of bridge) and leaves your shield and team's
  upgrades. Killed on land, your sapphire and amethyst are thrown out
  round you, onto ground only, for whoever walks into them once they
  have lain a moment (`bridge_wars::Mint::spill`, `SPILL_WAIT`; a bot
  that is by takes them before it goes on, `Squad::mind_spoils`);
  knocked off the world, or killed with no land under you or on a
  bridge that is coming down, they go to whoever did it
  (`bridge_wars::spoils`).
- **Generators** (`bridge_wars::Mint`) make silver and gold on each team's
  pad, sapphire on the smaller islands, amethyst in the middle; you pick up
  what you walk into (`src/loot.rs`).
- **Two shops**, each opened by standing in its outline (`src/market.rs`):
  the red stand sells bridge by the metre, a Bridge Builder, three swords,
  three shields, a tracker, and what goes on a button (fireball, dynamite,
  speed, leap, invisibility and healing potions); the blue stand sells the
  team's upgrades (generator, swords, shields, heal pool, blindness trap,
  builder's speed). What everyone has is `Outfits` (`Kit`, `Upgrades`).
- **Buttons** (`hud::Layout::Arena`): the sword where the jump button was,
  jump beside it, the plank (building on and off), the chat's keyboard, and
  up to five for things to use, shown only when filled (`src/utility.rs`).
  What is aimed is held and dragged; a potion is a press, drunk for 1.6 s
  at half pace, put down by a press anywhere else.
- **Bridges** (`src/bridge.rs`): with building on you carry a plank at the
  builder's pace and planks are laid under your walk, each costing its
  length in metres; mitred, cut flush where they leave and meet land or
  another bridge. **A bridge that is built stays as it was built.** Its
  geometry was hard won: read its entries in the log before touching it.
  Planks go to the server as they are fixed (`Ask::Rung`). A stick of
  dynamite takes the planks its blast reaches at once and the rest of the
  connected bridge plank by plank from the hole, at 6 m a second and
  quickening to 12, a little over a run (`Nav::cascade`,
  `bridge_wars::cascade_after`, `bridge::crumble`, `CASCADE_*`), each drawn in
  `colours.arena.plank_doomed` until it falls.
- **The computer's players** (`shared/src/bots.rs`, one brain for the
  server and for offline, over a map baked from the model,
  `shared/src/nav.rs`, `arena.nav`): each is one of sixteen kinds
  (`PERSONAS`) and plays the whole game: visits its pad and does not stand
  on it, stands at its shops or goes out to look about with nothing to do
  (`Goal::Scout`), throws what a raid is for as it lands (`Aim`: the
  anchor, the people, the pad), may build a way its team could do without
  (`Squad::scheme`), runs off a bridge that is coming down
  (`Squad::mind_falling`), and never fights a zombie but runs from one
  that comes near, hitting whoever of the players it passes
  (`Squad::mind_zombies`). **Hajun's rule for them: nothing
  is a definite if-then; every threshold is a spread, thrown again.** How
  hard they play is `Skill::MODERATE`. Tune by whole games: `cargo test -p
  roundtown_server many -- --ignored --nocapture` (`RT_GAMES`, `RT_SEED`,
  `RT_STALLS=1|2` for where one got nowhere). Online, tuning them is a
  server deploy alone; `RT_ARENA_BOTS=old` in the VM's env goes back to
  bots that keep to their islands.
- **A punch** lands on whoever is in the range drawn in front of you
  (`src/range.rs`) and knocks them back, which is how people go into the
  void (`start_punch`, `Shove::knock`; the numbers are `ui.ron`'s `melee`,
  Minecraft 1.8.9's). Never a teammate.
- **Health, what is held and what is bought are the server's online**:
  a punch is asked (`Ask::Punch`), a purchase is asked within 5 m of a shop
  (`Ask::Buy`), a fall is a death (`Ask::Respawn`). What happens is said in
  the chat by the game (`chat::Feed`), names in their teams' colours.
- The camera is fixed straight overhead, 15.5 m up, and nothing zooms it
  (`arena.camera_height`). Top right a minimap (`src/minimap.rs`), each
  team's island lined in its colour; top left your name in your team's.
- **Four buses**, the town's bus, one to each lane of the ring road round
  the pond, the outer half anticlockwise and the inner clockwise, at 14.2
  m/s (twice the 7 the town's went at then), each thrown round its lane
  every game
  (`bridge_wars::LANES`, `Buses`). **Each stops short of a crossing while
  anyone of the players is on it**, and goes on as nobody is (the four
  `CROSSINGS`, `Buses::drive`); the server runs them and sends how they go
  as that changes (`Leg`), offline the device does (`car::ArenaBuses`,
  `squash::steer_buses`). **A bus's front runs over whoever it comes to**
  off a crossing, never anyone at its side or riding it: 20 off their
  health and flat on the road for 10 s (`arena::squash`, `crate::Squashed`, `Game::run_over`),
  unable to do anything, hit as anyone is but knocked nowhere, and not run
  over again until they are up. Your own device says it of you
  (`Ask::RunOver`), the server of the computer's players, zombies and
  whoever's device has gone; who lies flat is sent (`Flattened`). The
  computer's players mind the buses, or do not, and some go for whoever
  lies flat (`Squad::mind_buses`, `spot_prey`).
- Also: `src/gear.rs` (swords, shields, the blindfold), `src/fire.rs`,
  `src/dynamite.rs`, `src/compass.rs`, `src/range.rs` (the punch's range),
  `src/trail.rs` (sparks behind a potion), `src/bridge/builder_bot.rs`.
- Nearly all of it since 2026-11-07 was checked in the desktop build and
  against servers on this PC, and built into APKs that were not installed:
  the log says of each thing whether it has been seen on a phone.

## The four games of 2026-11-18, as they stand

Each is played at a **table** (`server/src/tables.rs`, a `Rules` per game;
`src/table.rs` on the device, which hosts it offline; `roundtown_net::table`):
the fee as you join, a board top centre with Leave, a 5 s countdown, the
game, then the results in a window (places, scores, coins paid; Leave, Play
again) and back to the town. Leaving before play gives the fee back; a
device that goes keeps its seat, as in Bridge Wars. Fees and prizes are in
`docs/minigames.md`: by Hajun's rule, more than half beaten pays more than
the fee. Each game's rules are one `Game` in `shared/`, stepped by the
server online and by the device offline.

- **Blend In** (online only; 2 to 4 people, no computer): on the town's map
  inside a 70 m ring, among 28 of the town's people. Everyone picks a name
  and a look (30 s, sooner once all have); each round one person hides (15
  s, the seekers' screens black) and the rest seek for 150 s with an aura
  sword, the speed potion's effect and the town's free camera; a wrong
  strike blinds, 4 s and 3 more
  each time. A point for finding, one for staying hidden; everyone hides
  once.
- **Knock Off** (four seats, the computer filling them): a 12 m disc of ice
  in the sea. Everyone aims at once in secret (12 s, then 10: the stick
  for the way, taken anywhere on the screen but round the buttons
  (`stick_anywhere`), up and down for power 1 to 10, from 1 each turn),
  then all slide at once, the same to the bit on every device and the
  server; the ice's edge breaks a
  metre a turn, gone after turn 13. The last on it wins; the rest watch or
  leave.
- **Bridge Race** (the computer filling; one person is one against one,
  two or more two against two, people on one team first): two islands and
  the golden one. 10 m of bridge each to begin, 8 to 16 m to each team
  every 12 s shared between its players, and a stick of dynamite three
  grants in ten, each grant thrown ahead and said on the plate ("Next
  Item: 14m", `RaceRound::next`); a stick brings the whole connected
  bridge down after its blast, as in Bridge Wars; a punch only knocks, and the fallen are back on their island in 2 s. A team wins
  standing all together on the golden island for 3 s, a bridge to it or
  not; at 6 minutes, the team with more on it or nearer, or a draw. Two
  people are on one team, on each, or one against one, a third of the time
  each (`bridge_race::pairing`). Always noon (`sky::ALWAYS_DAY`). Bridge
  Wars' camera and buttons.
- **Last Stand** (1 to 4 people together, no computer): endless waves of
  zombies at the anchor crystal (1,500); two generators raised from silver
  and gold to sapphire and amethyst; loot off zombies, luckier as you
  upgrade; an item shop and an upgrade shop, endless levels. The sea round
  the sand, nobody past a ring 41.5 m out (`last_stand::SHORE`). The dead
  drop all four things and their sword for anyone to take, a sword only by
  one holding a lesser, five minutes (`drop_all`, `LIES_FOR`); now and then
  an unannounced treasure chest, broken open by one blow of the aura
  sword, two of another, three of a fist (`Game::hit_chest`, `CHEST_HP`),
  its contents thrown out round it to lie as the dead's do; a zombie a
  blow does not kill shows red. Over when the
  crystal is broken and nobody stands; everyone's score and the team's,
  10 coins a wave cleared, to 400. Bridge Wars' camera and buttons.
- Tuned by whole games, as Bridge Wars' bots are (`cargo test -p
  roundtown_net <name> -- --ignored --nocapture`): `many_games_of_blend_in`,
  `many_games_of_the_computer` (Knock Off), `many_races`, `stand_many`.

## House Builder (stalled)

"Build Your Own House", "Home Builders" on its card: 100 coins, a lobby,
eight builders each on a plot of their own, five minutes to build with
pieces from a shop, 15 s visits giving 1 to 5 stars, and a leaderboard that
pays every place. Online it is the server's (`server/src/builder.rs`). The
shop sells only Hajun's own models (`assets/mybuilds/`, `build::PIECES`),
loaded as they made them: fix a model in its `.blend`, never convert it.
`src/builder.rs`, `src/build.rs`, `src/shop.rs`. All of it is in
`docs/house-builder.md`.
