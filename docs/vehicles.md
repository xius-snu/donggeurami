# The town's buses, and the car you drive

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

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
