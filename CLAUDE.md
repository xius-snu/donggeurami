# 동그라미타운 (RoundTown)

A third-person 3D world in Bevy 0.19, zoomed into first person all the way
in: a lobby town everyone walks about in, and minigames entered from it. One
codebase, three targets: Windows (`src/main.rs`), Android (`mobile/android`,
a cdylib built with cargo-ndk), iOS (`mobile/ios`, a Rust binary Xcode
bundles). It plays online against a server in Seoul (`server/`; `shared/` is
`roundtown_net`, what the two sides say and the rules both go by).

**What matters now** (Hajun, 2026-11-14): the main lobby, Round Town, and
Bridge Wars. House Builder, the build-battle game, is stalled: keep it
working, and spend nothing on it unasked.

As of 2026-11-14: `PROTOCOL_VERSION` 16, APK 31 (`versionCode`), the server
deployed that day.

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
| colours and words on the screen | `docs/ui-style.md` | "The screen's colours and words are in `assets/ui.ron`" |
| building, packaging, APK numbers | `docs/building.md` | "Building" |
| the players' models, walk and punch | `docs/models.md` | "Models" |
| the sign-up's pages, anything typed into | `docs/signup.md` | "The first time, you sign up" |
| **Bridge Wars** | `docs/bridge-wars.md`, `bridgewarsgameevent.md` | "Games" |
| Bridge Wars' map | `docs/bridge-wars-map.md` | "The PvP map itself" |
| the opening, the buttons round the screen, the camera's hole, settings | `docs/town-hud.md` | "The app opens under a cover", "The app opens in Round Town", "Settings" |
| friends, groups, messages | `docs/friends.md` | "Friends, groups and messages" |
| the chat, the quick chat, fonts, Korean and Vietnamese, the word filter | `docs/chat-and-text.md` | "The chat", "The font is Inter" |
| netcode, the login, the town's clock | `docs/online.md`, `SERVER.md`, `MULTIPLAYER.md` | "Playing online" |
| a map's model, collision | `docs/maps-and-collision.md` | "The islands are Blender models" |
| fountain, music, town houses, town objects and their editor, AIs, talking | `docs/town.md` | "The fountain runs", "The town has music", "The town's houses", "The town is data", "The town holds 32 players", "You can talk to anyone" |
| the buses, the car you drive | `docs/vehicles.md` | "The town's buses", "The car you drive" |
| the camera | `docs/camera.md` | "The camera stands where you zoomed it" |
| House Builder, the shop's pieces, `assets/mybuilds/` | `docs/house-builder.md` | "House Builder", "Building your house" |
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
- Say what was checked where: unit tests, the desktop build (scripted), two
  copies against a server on this PC, the VM, a phone. A phone is rarely
  attached; never let "built" read as "seen on the phone".
- "Remove" usually means turn off: keep the mechanism behind a switch
  (`sound::PLAYING`, `hud::DOTS`, `utility::SECOND_ALWAYS`).
- Panels and backgrounds are neutral greys, the plate's `#666666`; no blue
  tint. Sizes and layout stay in code; colours and words in `ui.ron`.
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
  (`cluster_lights_on_the_cpu`). Shadows are off (`SUN_SHADOWS`). No real
  mirrors: every camera gets its own shadow maps.
- A day and a night pass every 20 minutes by the town's clock (`sky.rs`;
  `RT_TIME_OF_DAY=day|sunset|night|sunrise|<seconds>` stops it). One
  directional light is sun and moon. The sky hangs from a `SkyDome` that
  follows the camera.
- What gives off light at night is a list by material (`lamps::SOURCES`):
  street lamps' `lightbulb`, Bridge Wars' anchors' `spawn`. At most 24 point
  lights (`MOST_LIT`), none by day.
- See-through walls (`see_through.rs`) happen only inside a house, with
  alpha to coverage; never seen on the phone.
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
  render world on iPhone and Mac. The gamepad plugin is off.
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
  the APK; tests fail if that leaves out something the game loads.
- iOS cannot be built here: Codemagic builds it on a pushed tag
  `ios-<version>`. Code behind `target_os = "ios"` cannot be type-checked
  on this PC: keep it to call sites. An iPhone writes `log.txt` to its
  Documents (`src/log_file.rs`): ask for it before guessing at a failure.
- Desktop testing knobs: `RT_SERVER=<addr>|off`, `RT_UNSECURE=1` (debug
  build, a server with no key), `RT_FREE_CURSOR=1`, `RT_INTRO=off`,
  `RT_SIGNUP=1`, `RT_TIME_OF_DAY`, `RT_CAR_AT`, `RT_CLOCK_OFF`,
  `RT_ARENA_FROM=<seconds>`, `RT_ARENA_PURSE=<count>`,
  `RT_SAFE_AREA=<left>,<right>` (a camera's hole made up). Two copies need
  their own `APPDATA`. A server on this PC: `RT_PORT=5443
  RT_ALLOW_SKIP=1 target/debug/roundtown-server.exe`, and the copies with
  `RT_SERVER=127.0.0.1:5443 RT_UNSECURE=1`.
- To see a change in the running game, script the desktop build with a
  throwaway module (taps as `WindowEvent::TouchInput`, which picking takes;
  `Screenshot::primary_window()` for pictures) and take it out again before
  finishing. Posted mouse clicks do not reach the world.

## Playing online

- **The app never waits for the network.** It opens offline and connects
  in the background (`src/net.rs`, `src/login.rs`); with no server it is
  the offline game exactly.
- Three crates: the game (root), `shared/` (`roundtown_net`: every message,
  registered in one order by `protocol`, and every rule both sides go by),
  `server/` (headless Bevy, replicon and renet over UDP 443).
- **replicon's protocol hash is only the names and order of the types.** A
  field added, or one meaning something new, needs `PROTOCOL_VERSION`
  bumped, or an old app connects and breaks. An app whose version differs
  is told to update and stays offline; a change that leaves what is sent as
  it was keeps the apps already out playing.
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
- The VM allows the tap that skips waits (`RT_ALLOW_SKIP=1`): take it out
  before real players come.

## Maps and models

- Maps are Blender models (`assets/*.glb`, `island::Venue` names them):
  `roundtownmap.glb` the town, `pvptownmap.glb` Bridge Wars, `lobbymap.glb`
  and `homemap.glb` House Builder's, `testhomemap.glb` home. `src/island.rs`
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
  there is sea (`WATER_Y`), but under Bridge Wars' map, which has the void
  (`island::bottomless`).
- Export with Apply Modifiers (arrays and mirrors come out as one piece
  otherwise). A texture in a `.glb` needs Bevy's feature for its format
  (`jpeg` is on).
- After a new export of Bridge Wars' map: the constants in `shared/` have
  to agree with it (tests read the map), the bots' map has to be baked
  again (`RT_BAKE_NAV=1 cargo test --lib nav_bake`), and the server
  deployed.
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
  town and in Bridge Wars: its words alone, the newest always on the
  lowest line, each fading over 12 s; the dark box, every line in full and
  the box to type into come only while a line is typed (the small
  keyboard button). A line goes to everyone in the room; nothing is kept
  or logged. No block and no report yet, nor for friends' messages: both
  are wanted before a release to the shops (`MULTIPLAYER.md`).
- Friends (`src/friends.rs`, `server/src/social.rs`) are the server's
  alone, none offline: a `#1000`-`#9999` tag after every name, requests,
  groups of up to four whose leader alone starts a game, messages that wait
  in an inbox. The server reads the database once and goes by its own copy.

## The lobby, Round Town, as it stands

- The app opens under a black cover that lifts once the town is loaded,
  and you drop into it (`src/intro.rs`). No menu. Everyone arrives on the
  path round the pond (`roundtown_net::arrival`), facing the fountain.
- The town holds 32 bodies: whoever is in your channel (16 people to one)
  and this device's own AIs for the rest (`lobby.rs`, `ai.rs`). An AI is a
  player whose `Intent` comes from `ai::Steering`; it never steps down what
  it could not climb back. Bodies are spawned only by `lobby`'s own
  functions (`spawn_player` and its kin). Names over heads (`src/tags.rs`)
  are everyone's but your own.
- **The buttons** (`src/hud.rs`, as Hajun sketched them): top left your
  name on a plate (with its tag online), a person button (nothing yet), the
  gear (settings), and your balance under them; down the right edge a gift
  (the car's gun, in the car), two people (friends, with the inbox's
  count in gold), a gamepad (the game selector); round the jump button a
  face (in and out of the car), a speech bubble (a fan of things to say
  at a press, `src/quick.rs`, `words.chat.quick`), a small keyboard over
  it (the chat's box) and a fist (a punch; the car's reverse); the stick,
  whose middle is set where the finger first goes down, within half its
  radius. On Windows the stick and the jump button take the mouse.
- The top bar and the chat keep clear of the camera's hole (`src/safe.rs`:
  Android's cutout, by `MainActivity`; unseen on a phone; not on iOS).
- The gamepad opens the selector (`src/games.rs`): Bridge Wars, Home
  Builders (100 coins), and one to come.
- You can talk to any AI (`src/talk.rs`), ride the four buses that go
  round the roads by the clock (`car.rs`, `roads.rs`: the way is read off
  the road's paint), drive a car of your own (`drive.rs`, the face button,
  its numbers in `ui.ron`), and be thrown by the fountain's jet.
- The camera orbits you 1.5 to 18 m out, and zoomed all the way in is your
  eyes. **Nothing in the world moves it**: walls never bring it nearer. On
  a phone the stick brings it round behind you after 1.5 s, and a double
  tap swings it there. In the car and in Bridge Wars it is fixed straight
  overhead.
- Town objects are data (`src/map.rs`, a catalogue and `town.json`), edited
  by tapping (`src/editor.rs`), offline only. The edit menu's buttons stay
  on the object: never clamp them to the screen.
- The balance is the server's online (Postgres); only House Builder's fee
  and prizes change it. The music is turned off (`sound::PLAYING`).
- Settings (`src/settings.rs`): Graphics and Controls are empty; Accounts
  has the name, sign-in buttons that do nothing yet, and Delete account.

## Bridge Wars, as it stands

Hajun's `bridgewarsgameevent.md` has the items, prices, rates and timetable,
and the game is as that file has it. The rules both sides play by are
`roundtown_net::bridge_wars` (`shared/src/lib.rs`).

- **A game**: the selector, a black loading screen, a glass waiting room
  150 m over the map; ten seconds after the last person joins the computer
  takes every seat left; a countdown; 25 minutes. **Always 2v2v2v2**: eight
  seats, four teams, a team to an island (seat modulo four); people are
  teamed with each other first, and a group is seated together. Online the
  server runs it (`server/src/arena.rs`); offline the device does, with
  seven bots (`src/arena.rs`).
- **The map** (`pvptownmap.glb`): the town's middle, four team islands 130
  m out on the axes, red, blue, green, yellow, four smaller islands on the
  diagonals, and the void: fall 30 m and you are dead. Bridges are not in
  the map, they are built.
- **Winning**: each island has its team's anchor. With it standing, the dead
  come back in 4 s; with it broken, they are out. The last team with anyone
  in play wins. At 10:00 every anchor breaks, at 15:00 zombies rise, at
  25:00 it is a draw; the generators quicken on the way (`TIMETABLE`).
- **Dying** takes everything you hold (the four resources, your sword, what
  is on your buttons, your metres of bridge) and leaves your shield and
  your team's upgrades.
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
  Planks go to the server as they are fixed (`Ask::Rung`).
- **The computer's players** (`shared/src/bots.rs`, one brain for the
  server and for offline, over a map baked from the model,
  `shared/src/nav.rs`, `arena.nav`): each is one of sixteen kinds
  (`PERSONAS`) and plays the whole game. **Hajun's rule for them: nothing
  is a definite if-then; every threshold is a spread, thrown again.** How
  hard they play is `Skill::MODERATE`. Tune by whole games: `cargo test -p
  roundtown_server many -- --ignored --nocapture` (`RT_GAMES`, `RT_SEED`).
  Online, tuning them is a server deploy alone; `RT_ARENA_BOTS=old` in the
  VM's env goes back to bots that keep to their islands.
- **A punch** lands on whoever is in the range drawn in front of you
  (`src/range.rs`) and knocks them back, which is how people go into the
  void (`start_punch`, `Shove::knock`; the numbers are `ui.ron`'s `melee`,
  Minecraft 1.8.9's). Never a teammate.
- **Health, what is held and what is bought are the server's online**:
  a punch is asked (`Ask::Punch`), a purchase is asked within 5 m of a shop
  (`Ask::Buy`), a fall is a death (`Ask::Respawn`). What happens is said in
  the chat by the game (`chat::Feed`), names in their teams' colours.
- The camera is fixed straight overhead, 15.5 m up, and nothing zooms it
  (`arena.camera_height`). Top right a minimap (`src/minimap.rs`).
- Also: `src/gear.rs` (swords, shields, the blindfold), `src/fire.rs`,
  `src/dynamite.rs`, `src/compass.rs`, `src/range.rs` (the punch's range),
  `src/trail.rs` (sparks behind a potion), `src/bridge/builder_bot.rs`.
- Nearly all of it since 2026-11-07 was checked in the desktop build and
  against servers on this PC, and built into APKs that were not installed:
  the log says of each thing whether it has been seen on a phone.

## House Builder (stalled)

"Build Your Own House", "Home Builders" on its card: 100 coins, a lobby,
eight builders each on a plot of their own, five minutes to build with
pieces from a shop, 15 s visits giving 1 to 5 stars, and a leaderboard that
pays every place. Online it is the server's (`server/src/builder.rs`). The
shop sells only Hajun's own models (`assets/mybuilds/`, `build::PIECES`),
loaded as they made them: fix a model in its `.blend`, never convert it.
`src/builder.rs`, `src/build.rs`, `src/shop.rs`. All of it is in
`docs/house-builder.md`.
