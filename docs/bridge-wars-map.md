# Bridge Wars' map

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

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
