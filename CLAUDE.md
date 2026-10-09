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
- **There is no day–night cycle.** The sun is fixed at mid-morning so that
  everything casts a visible shadow; `src/sky.rs` sets it once at startup and
  nothing moves it.
- **Don't fake transparency with opaque geometry.** An earlier attempt added an
  opaque haze dome and giant "sun wash" spheres; that is what made the sky look
  grey and washed out. `RENDERING.md` has the details and the anti-patterns.
- **Test on the device, don't infer from the engine source.** The adb
  build-install-screenshot loop is in `RENDERING.md` and takes about ten seconds
  per cycle.
- **Lights are sorted into clusters on the CPU, on every platform**
  (`cluster_lights_on_the_cpu` in `src/lib.rs`, since 2026-10-08). Bevy 0.19
  does it on the GPU everywhere but Android and the iOS simulator. The world
  has no light to cluster, so on the CPU it is nothing at all, and every
  platform takes Android's path. It was turned off as the suspect for the
  iPhone's black screen, which it was not (`RENDERING.md`).
- **The town's fountain has a material and a shader of its own**
  (`src/fountain.rs`, `src/fountain.wgsl`): see-through running water whose
  streaks the GPU moves by the frame's time. It is the one custom shader;
  Bevy compiles it on each platform like its own, built into the binary
  rather than loaded from `assets/`.
- **What stands between the camera and you is drawn see-through**
  (`src/see_through.rs`, since 2026-10-09): a blended copy of its own
  material, in the transparent phase like the sea and the clouds, so on the
  phone it needs `NoIndirectDrawing` as they do. Not yet seen on a phone.

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
- **The audio and gamepad plugins are off** (`src/lib.rs`). Nothing uses them,
  and left in, one kept an audio stream open playing silence and the other
  polled for controllers every 8 ms. Turn them back on when there is a sound
  to play or a controller to read.
- **Anything that runs every frame writes only what has changed**
  (`set_if_neq`, or compare first). Writing a `Transform` with the value it
  already had still marks it changed, and for a player that means placing
  their whole skeleton again; writing a UI `Node` has the whole screen laid
  out again.

## Building

Desktop: `cargo run`.

The game server: `server/deploy.sh` builds it for Linux on this PC and puts it
on the Vultr VM (`SERVER.md`). `cargo build` at the root builds the game
alone; the server is `cargo build -p roundtown_server`. Never `--workspace`,
which would build the game with the server's half of replicon in it.

Android needs the `release-android` profile, a manual strip step, and Gradle —
see the "Build gotchas" section of `RENDERING.md`, or the commands in
`mobile/android/app/build.gradle`'s closing comment. The APK carries only
what the game loads: whatever `mobile/android/app/assets-left-out.txt` names
stays out (Blender files, reference pictures, retired models), and
`cargo test` fails if that would leave out a piece the shop sells.

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

## The app opens in Round Town

There is no menu. The app opens straight into the town (`circlemap1.glb`)
with all eight players already standing round the fountain, and the town's
name, "Donggeurami Town", fades in and out over the first few seconds. You can
walk, jump and look round from the first frame; on desktop the cursor is taken
for looking round at startup. The one game there is, House Builder, is played
from the town (below).

- **The button top right goes home and back** (`src/hud.rs`). In the town it
  shows a house and takes you to your home (`testhomemap.glb`); at home it
  shows a town hall and takes you back. Either way you land where your seat arrives
  (`island::arrival`), facing in, with the camera behind you. Only you go home:
  the AIs stay in the town. The town's name shows again every time you arrive
  back in the town. The button is put away for as long as you are in a game of
  House Builder, which has its own way back.
- **On desktop, press `Escape` to free the cursor before clicking the button**,
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

## Playing online

Since 2026-10-08 the game plays online, against a server in Seoul (a $10
Vultr VM). `MULTIPLAYER.md` is the plan and the why, `SERVER.md` how the
server is run. The rest of this file holds online too: being online changes
where things come from, not what they are.

- **The app never waits for the network.** It opens on the offline town as
  it always has; `src/net.rs` connects in the background, IPv6 first, then
  IPv4. Once the server has let you in, whoever else is in your channel of
  the town (16 to a channel) walks into it, and one of this device's AIs
  leaves for each, keeping the town at eight bodies
  (`lobby::fill_the_town`; Hajun's choice, 2026-10-08). Each device runs its
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
  remote body has no collider; AIs and walls still bump.
- **Getting in.** The device makes up a secret the first time it runs
  (`device.id`, beside `town.json`), sends it over HTTPS to the login on the
  VM (`src/login.rs`, TCP 443, the server's own certificate pinned by its
  SHA-256, `login::PINNED`: no domain), and gets a signed connect token. A new
  device is a new guest account in Postgres with a generated name ("Mochi
  482"), 1,000 coins, and the blue look, which every real player wears for
  now. **The VM's addresses and certificate are compiled into the app**
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
- **Names over heads** (`src/tags.rs`), online only, AIs included. **Town
  editing is off** while online (`map::hands_off_online`): nobody else would
  see it. **Your account** is the round button beside the balance, online
  only (`src/account.rs`): its name, and Delete account, which Apple asks for
  even for accounts nobody signed up for.
- **Testing on desktop.** Two copies side by side need their own `APPDATA`
  (their own `device.id`, so their own accounts) and `RT_FREE_CURSOR=1`, so
  that neither takes the mouse. `RT_SERVER=<address>[,<address>]` points a
  desktop build at another server, `RT_SERVER=off` keeps it offline, and in a
  debug build `RT_UNSECURE=1` skips the login, for a server run without
  `RT_KEY`. The VM allows the tap on the time that skips House Builder's
  waits (`RT_ALLOW_SKIP=1`): take that out of `/etc/roundtown/env` before
  real players come.

## The islands are Blender models

`assets/circlemap1.glb` is the town, `assets/testhomemap.glb` your home, and
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
- **Everyone arrives on a ring 6 m out from the middle**, seat by seat, facing
  in (`island::arrival`): round the town's fountain, and on the grass of the
  home, the lobby and the plots.
- **Someone modelled into a map is not land.** Skinned meshes are left out of
  the shape: they are drawn where their skeleton puts them, not where their
  transform does, so read as land they stood somewhere else entirely. (The
  House Builder read that way was an invisible wall 23 m out from the
  fountain, on the road to them.) Give such a person a collider in code, as
  `builder::settle_house_builder` does.
- **The sea is everywhere the model has nothing underfoot,** at `WATER_Y` — z = -0.5
  in Blender. The land's flat top is z = 0 (`LAND_TOP`); the lowest ground on
  either island, the town's roads and the home's sand, is z = -0.2, so keep
  anything walkable above about -0.25 or it counts as wading. Each island has
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
  nothing else in the file changed.
- **`circlemap1.glb` had a stray `Icosphere` in it** (found 2026-09-30): 2 m
  across, with no material, at the middle of the fountain, read as land, so
  that it bulged out round the foot of the tower. It is not in Hajun's exports
  of the town since 2026-10-03 (no part without a material is).
- **The town has eleven trees and a second house** (Hajun's export of
  2026-10-04): the trees round the fountain and out toward the edge, the
  house across the road from the House Builder's, with a sign of its own,
  `Text.002`, as high as theirs. Being part of the model, they are land like
  the rest: bodies bump into the trunks, and a trunk or leaves between the
  camera and you fade (`see_through`), as a town object such as the
  catalogue's tree does. One trunk is
  in twice, `Cylinder.009` and `Cylinder.011` in one place: harmless, as the
  game merges surfaces met twice at one height. Each trunk is closed but for
  a disc left inside it, which nothing can reach. **Keep the signs' font
  simple:** a text object is exported as a mesh of every letter, front, back
  and sides, and in Calibri Bold the two signs were 47,000 of the town's
  62,000 triangles, about 1,500 a letter. In Blender's built-in font, as
  Hajun set them the same night, they are 5,600 of 22,000.
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
- **A body is the shape of the model** (since 2026-10-08): its legs, 0.42 m
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
  still counted whole.
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
- **The jet goes by the clock** (`jet_phase`, the time since 1970 modulo 4 s),
  not by the game's time, so every device's goes up together and everyone
  online sees the same jet throw the same player. A throw is the thrower's
  own device's, like a jump, and the server takes it as one: it rises over
  twice as fast as a jump, inside the server's slack for one, which allows
  a throw up to about 13 m (`MOST_RISE` and `BURST` in
  `server/src/players.rs`, which a test there keeps in step).
- **Everything is read off the model** once the town has spawned
  (`Fountain::read`): the tower, by its name `Water_fountain_tower`, for its
  axis and its top; every piece of `water` on its axis for the pools; and for
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

## The camera stands where you zoomed it

There is one view (Hajun, 2026-10-09): the camera orbits you as you turn and
zoom it (`OrbitCamera`, `follow_camera` in `src/lib.rs`), 2.5 to 18 m out
behind you, and zoomed all the way in it is your own eyes. Nothing in the
world moves it: walls, ceilings and trees never bring it in nearer you.
Whatever stands between it and you is drawn see-through instead, and a tap
goes through it (`src/see_through.rs`). That is Hajun's call (2026-10-09):
"at all times u should be able to zoom out to the current max zoom out even
when ur inside a room or covered by some wall and the camera should not
automatically go near the player". In a house in House Builder, zoomed out,
the camera is outside it looking in, and you tap what is in there with you
through its wall.

- **It always points exactly the way you turned it, and stands as far out as
  you zoomed it**: 2.5 to 18 m (`CAMERA_DISTANCE_MIN`, `_MAX`; 15 m at most
  until Hajun asked for "a slight bit more", 2026-10-09). It tips down as far
  as straight down (`PITCH_MAX`, a right angle; about 69° until Hajun asked,
  the same day), for a view of the tops of things, a roof fading to show the
  rooms under it. Until 2026-10-09 the island brought it in: a
  ball was swept out from you to where you zoomed it and the camera stood
  where the ball stopped, eased in and out (`CameraFit`), with a screen
  across every doorway to keep it in a house (`build::screen`). All of that
  is gone. Before that, a version slid it along walls and climbed it up them,
  and Hajun found it glitchy (2026-09-27): every bit of a swipe has to turn
  the camera by as much, wherever you are. **Keep both**: nothing moves it off
  where you turned and zoomed it.
- **What stands between it and you fades to a quarter of itself** (`SEEN`,
  0.25, as much as Roblox's Invisicam leaves of what it fades) over 0.15 s,
  and back once it does not (`FADE_SECS`). Every frame the straight lines
  from the camera to three points on you, the bottom of your body, your chest
  and your head (1.15, 1.4 and 1.65 m up, `SEEN_AT`), are tried against
  everything drawn on your island: the box round each part first, then its
  triangles, read once per mesh into boxes inside boxes (`island::Faces`) and
  again whenever the mesh changes, like a wall a door has just cut. What
  hides only your legs, like a bed you stand behind, is not in the way. A
  part fades with a blended copy of its own material that shows only its near
  side, so that a wall is one pane rather than two, and has its own back once
  it is solid again. It costs 5 to 13 µs a frame in a debug build (measured
  in the town and on a plot).
- **What fades:** a piece of furniture, a door, a window or a town object
  fades whole; a house and the island a part at a time, each as modelled in
  Blender: a wall, the roof, a floor, a tree's trunk or its leaves. Bodies
  never fade (skinned meshes are left out), nor water, glass, ghosts or the
  sky, which are blended already, nor a piece being put down, nor whatever
  the edit menu is open on. Seen through a doorway, the door fades and the
  wall round it does not.
- **A tap goes through whatever is see-through** (`see_through::InTheWay`,
  which `editor::pick` skips as it does `Untappable`, for as long as any of
  it is see-through): only what is not in the way of you can be tapped.
  Checked in a scripted run (2026-10-09): standing in the Square House with
  the camera 15 m out, its front wall faded, and a tap on the bed through it
  opened the menu on the bed. A house you are standing in still cannot be
  tapped (`build::untappable_from_inside`), whatever the camera sees.
- **Zoomed all the way in, it is your eyes, and that is as far in as it
  zooms** (Hajun, 2026-10-09: "the max zoom point is the point at which it
  becomes first person view ... as soon as you zoom out u are back to third
  person view"). Zoomed in past 2.5 m it goes into them, and zoomed out of
  them it is straight back behind you at 2.5 m; either way it takes a tenth
  more zoom than the camera can follow (`SWAP_ZOOM`), a notch of the wheel,
  so that fingers that waver as a pinch ends do not swap it back and forth.
  There is no view button any more (from 2026-09-29 it swapped first and
  third person while you built and rated), and no zooming the view in your
  eyes (it narrowed to 23°). It takes in 66° top to bottom (`CAMERA_FOV`),
  near and far: first person's width; behind you it was 45° until then.
- **Your eyes are inside your head, as high as the model's eyes, 1.57 m**
  (`EYE_HEIGHT`, under the top of the head at 1.71 m; Hajun, 2026-10-09).
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
  as it should, that it looks straight down with ahead still ahead, and which
  of a house's walls, its roof and a bed are in the way from inside the house
  and from outside it.
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
  off light. A save written before 2026-09-26 may still hold the two trees the
  default town used to start with.
- **A save is one JSON document per player**, so it can become one row in a
  server table later without reshaping. It records a *kind*, never an asset
  path. Bump `SAVE_VERSION` if that shape changes.
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

## The town holds eight players; offline, seven of them are AI

`src/lobby.rs` holds the room: eight seats, each a flat `Member` record — an
id, a name, a look and an `ai` flag — the very record the server sends
(`roundtown_net::Member`). Offline you are seat 0 and the other seven are AIs,
walked by `src/ai.rs`. Online, the server sends whoever else is in your
channel of the town, and this device keeps the town at eight bodies with AIs
of its own (below, "Playing online").

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
- There are four looks and eight seats: you are always blue, and the AIs take
  turns at red, yellow and purple.

## House Builder

The one game, in `src/builder.rs`. On screen it is called "Build Your Own
House" (`builder::NAME`); in the code it is House Builder, after the NPC who
offers it. The House Builder stands at the door of
their house in the town — the node called `House Builder NPC` in
`circlemap1.glb`, found by that name once the town has spawned, so moving them
in Blender moves where the game is offered. Nothing about them is placed in
code: the game only gives them a collider and hangs their arms at their sides
like everyone else's (the model has them out). Over their door, in raised
white letters, is "BUILD TO IMPRESS" ("Build Your Own House" until
2026-10-04), the text object `Text.001` in `circlemap1.blend` (it was added
as `HouseBuilderSign`): part of the model, and so of the town's shape, but
from 7.5 to 10.8 m up (since Hajun's export of 2026-10-04; about 6 m the day
before, and 5.5 m before that), over any jump.

- **Walk within 4.5 m and a speech bubble comes up over their head.** Tap it,
  or on desktop press Space (which then does not jump), and a dialog asks
  whether to play for 100 coins. Play pays, and takes you to the lobby
  (`lobbymap.glb`); Cancel, or a tap on the dark round the dialog, does not.
  Without 100 coins, Play is greyed out.
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

While the time to build runs, a hammer button sits top right, below where the
home button is in the town (`src/shop.rs`). It opens the shop: a window under
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
  reads inside off its `Floor...` objects); and all but what stands between
  the camera and you, which is see-through and which a tap goes through as
  well (since 2026-10-09; "The camera stands where you zoomed it"). Only one
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
  so that a window is never a way in. A door's hole is open to bodies. What
  stands between the camera and you is see-through, and a tap goes through
  it (above, "The camera stands where you zoomed it").
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
  back to 1.375 m, on four splayed legs; their Three Leg Table
  (`threelegcircletable`; "Three Leg Circle Table" wrapped onto two lines of
  a 16:9 shop card) and Circle Table (`circletable`), round tops 2 m across
  and 1.2 m high on three curved legs and on a pedestal; and their Cafe Desk
  Edge and Cafe Desk Corner (`cafedeskedge`, `cafedeskcorner`), counters 1.5
  m high, 2 m by 1 and 1 m by 1, one mesh each with a wood front, a cream top
  and a dark plinth, and since 2026-10-04 a grey strip along the top's edge
  (`desktopedge`). Nothing is in Decorations.
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
  the other, `threelegcircletable.png`); only the camera and the path were
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
