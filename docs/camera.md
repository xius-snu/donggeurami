# The camera

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

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
