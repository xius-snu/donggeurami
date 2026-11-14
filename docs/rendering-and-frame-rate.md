# Rendering and frame rate

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

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
