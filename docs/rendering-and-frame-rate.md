# Rendering and frame rate

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Forty-eight furnished plots: what was done about their cost (2026-11-23)

With every plot of the town filled (`docs/land.md`), and someone walking
sixteen of them (`docs/town.md`), there is three times as much standing
in the town as the day before, the day a phone ran hot with two plots.
**Nothing here was measured on a phone**; these are what the desktop
showed and what was done about it.

- **What there is to draw**, all of it, were it all in view: the made-up
  land 184,000 triangles (a test holds it under 190,000, and each plot
  under 5,000), 48 signs 309,000 (one is 6,434, nearly all of it chain),
  sixteen bodies 61,000 (32 were 122,000), the map 42,000. About 600,000,
  where the town of the day before was about 225,000. Things that are
  the same model are drawn together; what is not in view is not drawn.
- **A sign's words are hung on it only within 45 m of the camera**
  (`land::word_the_near`): 48 pictures were a megabyte each, kept on both
  sides, and two see-through draws each, for words that are less than a
  dot from across the town. Drawing one takes a third of a millisecond.
- **The town's stood shape is read no oftener than every 0.35 s**
  (`build::RESHAPE_REST`): some 180,000 triangles are gone through each
  time, and as the app opens a piece's model loads nearly every frame.
  The first eight seconds, hidden on the desktop: 35 frames over 20 ms
  before, 8 after, the mean 13.4 ms and then 9.1.
- **A resident's way is found a little each frame** (`resident::Search`,
  `Walkable`): one way over a quarter-metre lattice took up to 157 ms in
  a single frame (timed, with fifteen of them about). It is looked for
  over half-metre squares now, what is found of each square kept until
  the town's shape changes, and no more than 40 new squares looked at a
  frame by all of them together.
- **Settled**, hidden on the desktop at its cap of 120, with that: 8.5 ms
  a frame over thirty seconds standing on a plot, the longest 28 ms, 14
  frames of 3,529 over 20 ms. What the 14 were was not looked for.
- **If a phone does not keep up**, in the order I would try: a plain
  sign from far off (four boxes for 6,434 triangles); nothing drawn of
  what stands inside a house across the town; fewer than sixteen bodies.
  Each changes what is seen, and is Hajun's to say.

## Two furnished plots and a hot phone (2026-11-22)

Hajun, having played twelve minutes with a friend on APK 50: "now the
game is so hot on my phone, my phone becomes so hot - not sure what
changed - maybe its because of the house models? or some optimization
required? it just loads 2 real lands with house designs atm player
"bunnihoho" and "bunnihihi" and it heats up the phone so much. think of a
way to optimize."

**What was looked at**, on the desktop, with the two plots as their
owners have them (39 things and 13, read out of the database into
scratch saves; a temporary driver, taken out again):

- **What a frame costs**, in the processor's cycles (the dev build, 60
  frames a second): an empty plot, 18.3 million, 8.2 of them on the
  world's thread; stood in the house of the 39, 20.4 and 9.5 to 9.7;
  outside it looking at it, 18.6 and 8.4. Two copies online against a
  server on this PC, each with one of the plots, 21.5 and 10.9.
  Run to run these differ by half a million or so.
- **What is drawn**, the phone's way (no indirect draws): 173 kinds of
  thing for 364 meshes on the empty plot; 206 for 514 with the 39 things,
  and 31 see-through things for 24. A sixth more. The two plots are
  31,000 triangles in all, where the town is 42,000 and its 32 people
  120,000.
- **What is written every frame**, counted over 240 of them standing
  still, alone and with the second copy walking about: no part of the
  screen laid out again, no words, no material, mesh or picture sent
  again; 126 limbs and wheels turned, as on the empty plot. Nothing of
  edit mode, paint or the plots runs away.
- **One thing found: the see-through search** (`find_what_hides_you`).
  Standing in a house it went through every drawn part of the town every
  frame: 0.75 to 1.0 million cycles, a tenth of the world's thread. With
  you and the camera where they were it finds what it found, so now it
  looks again only as either moves, as the edit menu opens on something,
  as a mesh changes, and five times a second besides for what moves by
  itself, a door's leaf (`Looked::still_good`, `LOOK_AGAIN`). Standing
  still: 0.08 to 0.12, with the same parts see-through. Walking, as
  before.

**What was not found.** Anything that explains a phone "so hot" by
itself. What a phone's graphics chip does with these scenes cannot be
measured on this PC, nor its heat. Two things that are heavier there
than here and were not touched: walls drawn see-through with alpha to
coverage while you stand in a house, and by night each lamp on a plot,
which lights 40 m round it like the town's own, three more of them on
these two plots.

**Left for Hajun to say**, being things one can see: a choice of 60
frames a second in Settings, Graphics, which is empty, or 60 by itself
while the phone says it is hot (Android's thermal status, which needs
the phone to try); lamps on plots lighting less far. And with a phone
attached, the measuring in `RENDERING.md`, which says where the heat is.

## Two faults of see-through put right (2026-11-22)

Hajun, of his plot's house on the phone: "some objects become unclickable
(like a sofa) or like part of the chair (the seat part of the chair
suddenly disappear + become an unclickale chair until i reopen the app)",
and "when the player is in the house, the eye becomes transparent i think
thats a bug. eyes and mouth shouldnt become transparent". Both are
`src/see_through.rs`; `docs/bag.md`, the entry of that day marked
"night", has the rest of that round.

- **A part in the way for one frame was left see-through for good**
  (`fade_what_hides_you`). It is given its see-through copy the frame it
  is found in the way, as solid as it was; the frame after, in the way no
  more, it was "as much of it as should be drawn" already, and the code
  went on to the next part before it came to giving its own material
  back. So it kept the copy, which is drawn from its front alone
  (`cull_mode` back, not two-sided): a part modelled facing the other
  way, the cafe chair's seat, was not drawn at all, and whatever the part
  belongs to stayed out of a tap's reach (`InTheWay`). Only being in the
  way again, long enough to fade, put it right. Now a part that is all
  there and not in the way has its own material back whether or not it
  had begun to fade.
- **No part of a body is in the way** (`of_a_body`: anything under what
  `lobby` spawns, by its `Seat`). A body's skinned meshes never were; but
  five of the six looks have eyes and a mouth that are meshes of their
  own on the head bone (`Sphere`, `Sphere.001`, `Cylinder`), solid, and
  those were found in the way of the lines from the camera to your head
  and drawn at a quarter whenever you stood in a house facing it. The
  first boy's face is one blended picture (`facemat`), which never was.
- **Seen on the desktop before and after**, by a switch for the old way
  that was taken out again: the girl's eyes and mouth at a quarter in a
  house, and solid, from 42 places of the camera; the chair's three
  parts stuck after you were put beside it, its seat gone from above and
  the chair deaf to a tap, and whole and picked up with the fix.
- **What this says of the phone**: Hajun saw eyes go see-through on his,
  so alpha to coverage is drawn there, which nothing here had shown
  (below, "never drawn alpha to coverage for this game"). Not seen from
  here on a phone, before or after.

## Less work for the same picture: a cooler phone (2026-11-15)

Hajun: "When i play the game for a long time on my phone, my phone becomes
somewhat hot. i dont rly want to downgrade the visual or graphic or
functionality or ntework, but i want to make it so my phone isnt as hot when
i play the game. try to see if there is a way to make it more optimized
overall, without needing to sacrifice as much noticeable graphic or sth. if
u cant find any that wouldnt sacrifice any visual or noticeable changes, then
dont make any and just say so." And, while it was being looked into: "it
should be optimized for both android and iphone."

Nothing that is seen, played or sent was changed. Five things that cost work
and gave nothing were found and taken out, all in code every platform runs.
**No phone was attached: every number here is the desktop build's, and what
a phone's GPU saves is not measured at all.** APK 34, built and not
installed.

**What a frame cost, and how it was measured.** A throwaway module counted
the processor's cycles the whole game used a frame (Windows'
`QueryProcessCycleTime`, and each thread's), over 12 to 20 seconds of a
scripted run offline, in an optimized build (the phone's own profile,
`release-android`, built for Windows), on the laptop, whose screen was at 60
Hz that day. A build with Bevy's `trace_chrome` gave what each system took.
What it showed: no one thing is slow. The game's own 265 systems a frame
are about a fifth of the game world's time, `move_bodies` the largest at a
tenth of a millisecond; the rest is Bevy's, some 500 systems a frame across
the two worlds, each small. So what could be taken out was what did no work:

- **Bevy's own systems handed their loops out to a pool of worker threads**
  (`worker_threads` in `src/lib.rs`). `one_thread_per_world` (2026-09-27)
  stopped the schedules handing out their systems, and said that systems
  which split their own work (`par_iter`) still did: a dozen and more a
  frame do (placing transforms, finding what is in sight, handing meshes to
  the renderer), each waking every worker for a few entities. On the laptop
  its eight workers took 2.5 of the 19.5 million cycles a frame the game
  used, 13%. Bevy's "compute" pool now has one worker (it would start one
  for every core left over: four on the S26, two on an iPhone), and with one
  Bevy runs such a loop on the thread that asks: the one worker takes 0.55,
  the loops costing less where they are asked for than the waking did. Not
  none: Bevy's transform system spawns one task fewer than there are
  workers, and would spawn none fewer than none. The pools for reading
  files and for background work are as Bevy has them, idle once the game
  has loaded.
- **The camera's cells for sorting lights into were made up by day, for no
  light** (`lamps::cells`, `light_the_lamps`). Since the lamps
  (2026-10-16) the camera has Bevy's own few thousand cells, so that a spot
  is lit by the few lamps near it; by day, and wherever no lamp is lit,
  Bevy still made them up, sent them to the render world and to the GPU
  every frame: 0.65 to 0.9 million cycles a frame, a twentieth. The camera
  now has one cell (`ClusterConfig::Single`, what it had before the lamps)
  until a lamp is lit, and Bevy's own from the frame the first is. The
  lamps are the only lights there are to sort. With one cell or many the
  picture is the same; a live dusk was run through to see the change
  happen as the lamps come on (at 0.0001 of their light), with nothing to
  see of it.
- **Sound was sent to the device, of silence** (`plugins` in `src/lib.rs`,
  `sound::plugin`). The music has been off since 2026-11-02
  (`sound::PLAYING`) and it is the game's only sound, but Bevy's audio was
  still in: it opens the device's output as it starts and plays silence
  into it for as long as the app is up, which on a phone keeps a thread of
  the app's and the phone's own sound hardware awake every few
  milliseconds. Bevy's audio is now left out while `PLAYING` is off, and
  comes back with it, as it was left out until the music came
  (2026-10-30: "keeping a stream open playing silence, until there was
  something to hear"). On the laptop its thread was 1% of the cycles; what
  the phone's hardware saves is not measured. **One thing that may be
  noticed on an iPhone, and is not tested:** an app that opens sound output
  can stop music playing from another app; with none opened, someone's own
  music should now play on under the game.
- **The world was drawn in two passes on the GPU** (`src/pass.rs`, new;
  `ONE_PASS` turns it off). Bevy draws everything solid in one render
  pass and everything see-through in a second (the sea, the clouds, the
  fountain's water, the punch's range), both into the same picture at four
  samples a pixel with the same depth. A phone's GPU draws a pass a tile
  at a time in a small memory of its own, so between two passes it writes
  every tile out to the phone's memory and reads it back: at the phone's
  1080x2340 the four samples and the depth come to some 80 MB to write
  and 80 MB to read back each frame, as they lie, and the one-sample
  picture is worked out twice, for nothing that shows. They are now one pass (`world_pass`): the same
  draws in the same order with the same pipelines, and at its end the
  samples and the depth are let go rather than kept, since nothing reads
  them again (the UI is drawn on the one-sample picture, and that goes to
  the screen). Bevy 0.19's camera passes are systems in a schedule
  (`Core3d`), and its own two are taken out of it and this put in where
  they were, on the first frame (`swap`); if they are not both there to
  take out, Bevy's stay. **What needs the two apart gets them back by
  itself** (`apart`): a material light passes through (Blender's
  Transmission, which no model has: every `.glb` was read for it) and
  order-independent transparency. What else would (a second camera on the
  window, depth of field, Bevy's atmosphere, screen-space reflections) is
  not in the game and has to be looked at in `pass.rs` before it is. On
  the desktop it saves next to nothing, a desktop's GPU drawing straight
  into memory: **the saving is a phone's, and unmeasured.** It is what
  every guide to a phone's GPU says to do first, and the commoner way to
  drive one, but `RENDERING.md`'s rule stands: it has to be seen on the
  phone. What to look for there is in `RENDERING.md`, "The world in one
  pass".
- **On an iPhone the whole render world ran on Bevy's own executor**
  (`one_thread_per_world`). Since the black screen (2026-10-08) the render
  world's schedules were left alone on Apple's platforms, because one
  system of one of them has to run on the main thread (`create_surfaces`,
  in `Render`; it is the only system of the render world's that asks for
  it: every `NonSend` in Bevy's render crates was looked for). So an
  iPhone was still paying, for the render world, what the phone paid
  before 2026-09-27. Now only `Render` keeps Bevy's executor there; what
  is taken over from the game world each frame (which runs on the main
  thread) and the camera's passes run on one thread as everywhere else.
  Written so that Windows compiles and tests every line of it
  (`cfg!(target_vendor = "apple")`). **Not run on an iPhone**: the next
  iOS build is the first, and if it opens on a black screen, this is
  where to look, with the phone's `log.txt`.

**What it comes to, on the laptop** (million cycles a frame the whole game
used, before and after, at 60 frames a second; two runs of each):

| | before | after | |
|---|---|---|---|
| the town, by day | 19.5, 20.1 | 16.6, 17.2 | 15% less |
| the town, at night | 19.9, 19.6 | 18.4, 18.3 | 7% less |
| a game of Bridge Wars, by day | 20.6, 19.9 | 17.1, 17.5 | 15% less |
| a game of Bridge Wars, at night | 20.3, 20.2 | 18.7, 18.4 | 8% less |
| as fast as it will go, drawn the phone's way | 12.4 (718 frames a second) | 8.7 (817) | 30% less |
| the town by day, threads as on an iPhone | 28.4, 29.3 | 17.7, 18.3 | 37% less |

"Before" is the same program with the four changes switched off, which
measured as the build before them did (19.5). The night saves less because
the lamps' cells are there for the lamps. "Drawn the phone's way" is with
`NoIndirectDrawing` on the camera and no waiting for the screen; "threads
as on an iPhone" is with `Render` on Bevy's executor, as an iPhone has it,
against every render schedule on it and eight workers, as it had: a
desktop's guess at an iPhone, whose own pool was two. Frames came as evenly
after as before in every one (much the same middle, 95th and 99th in a
hundred frame times). 116,000 frames of a game ran with the memory level and
nothing in the log.

**That the picture is the same**, checked on the desktop build (Vulkan, the
phone's API), by the game saving its own screenshots:

- Nothing moving (Bridge Wars' map from straight overhead, three views, by
  day and by night, in separate runs): the build as it is now against
  everything as it was, and against each change switched off by itself:
  the same to the last pixel, every one.
- The town frozen mid-frame (the clock and the game's time both stopped)
  and drawn in one pass and then in Bevy's two, five views (the pond's
  water, the sea, the clouds, the fountain, people, the HUD), by day and by
  night, the desktop's way and the phone's (`NoIndirectDrawing`): the same
  to the last pixel, all 35 pairs, over three rounds.
- A game of Bridge Wars frozen at five moments (a bridge, a sword mid-cut
  with the range drawn, a potion's sparks, a shop, the middle), by day and
  by night, some 170 pairs in all: the same to the last pixel in all but
  ten. In seven, one or two pixels on the edge of a see-through shape
  differ by 1 to 5 parts in 255 (never between two shots drawn the same
  way, and not at all in a run that went back and forth four times on one
  frame: the GPU's rounding where see-through edges overlap, it seems, and
  nothing an eye could find). In two the computer's players had begun a
  swing between the two shots, until their deciding was stopped for the
  shot as well. And in one, once, the player had moved a metre between the
  two shots, which no pass can do, and which 22 more runs did not show
  again (16 of them with the player's place written down at every shot):
  the freeze, not the drawing; said here because it was seen.
- Unit tests: 308 pass (four new: the pool's one worker, no audio while
  the music is off, the camera's cells, the passes apart only for what
  reads between them). `shared/` and `server/` are untouched: no protocol
  change, no deploy.

**Not checked: anything on a phone.** Not the heat, not the frame rate, not
that the one pass draws as it should on the Adreno or on an iPhone's GPU.
The APK is built and not installed.

**Looked at and left alone**, each because it would be seen, or could not
be tried here:

- 120 frames a second, four samples a pixel, the phone's three-quarter
  size, the lamps' reach: all seen. Half the frames is half the heat, and
  is Hajun's to ask for.
- Leaving out the parts of Bevy the game never uses (sprites, gizmos,
  states, animation: some 160 small systems a frame): a few hundredths of
  the frame between them, and the UI leans on some of them.
- Drawing the UI straight onto the screen's own picture, saving the copy
  Bevy makes at the end of a frame (a third of a millisecond of the
  phone's GPU, measured 2026-09-27): three of Bevy's passes to take over,
  and a format that differs by phone.
- Telling Android what a frame costs so that it clocks the processor to
  suit (its performance hints), and turning the picture the way the screen
  lies so that the phone need not (Vulkan's pre-rotation): both are a
  phone's alone, and want the phone in hand.
- On an iPhone, the render world on the main thread with no thread of its
  own (which would take `Render` off Bevy's executor too): a different way
  of running, for a phone that cannot be tried here.

**Calls that were mine**: all five changes (Hajun asked only that nothing
seen should change); keeping each behind what turns it back
(`pass::ONE_PASS`, `sound::PLAYING`; the pool and the cells are a line
each); shipping the one pass on, unseen on a phone, rather than off;
touching the iPhone's threads at all without one to run it on.

**The harness** (throwaway, taken out again): `scratch_heat.rs` (the
cycles a frame by thread, what is in sight, switches that turn each change
off, `RT_X=pool,clusters,pass,audio`, and try the phone's drawing and the
iPhone's threads, `RT_ON=direct,apple,novsync`), and the scripted-run
harness of 2026-11-14 with `pause` (the game's time, the town's clock and
the computer's players all stopped) and `apart` (Bevy's two passes for the
frames that follow). Kept in the session's scratchpad as `kept_heat/`, with
`scratch_in.py` and `scratch_out.sh` to put it in and take it out, the
step files, `same.py` (two screenshots, pixel for pixel) and
`trace_stats.py`.

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
