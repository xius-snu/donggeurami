# House Builder, and building your house

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

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
