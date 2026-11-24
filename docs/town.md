# The town: fountain, music, houses, objects, AIs, talking

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Residents: one on every plot, and "X's bot" on a real person's land (2026-11-24)

Hajun, the day after the entry below:

> i think if the npc is named after a real person, it should say like
> "bunnihihi's bot". but if its a fake house owner's npc then just keep
> their name like "Sagwa" like now. also i think there should be a
> character for all 48 plot not just the first 16, and channel size for
> real players should be increased from 16 to 48, so if theres just 1 real
> player online in a channel, then there should be 47 npcs, each on their
> plot

Where each plot is, and whose, is `docs/land.md`'s first entry: thrown
afresh as a channel opens.

**What there is** (`src/resident.rs`).

- **One on every plot nobody of your channel has**: 47 with you alone,
  none with 48 people (`roundtown_net::TOWN_BODIES` is 48). The server
  says `Holder::Minded` of every filled plot, and offline every plot but
  yours has one. The town has 48 people in it, where it had sixteen for a
  day and 32 before that.
- **Named for what they are** (`resident::called`): someone on a made-up
  owner's land goes by that owner's name, as before; someone on the land
  of a real person who is not here goes by "{name}'s bot"
  (`words.land.bot` in `ui.ron`, his to change), over their head and when
  talked to. The sign still says whose the land is, "{name}'s land". A
  resident whose words change (the file read again on the desktop) is
  made again under the new ones.
- **A visit is as rare a thing at your plot as it was**
  (`resident::CALLERS`, 15): with more residents to each real person than
  fifteen, each visits that much less often, so that someone alone with
  47 of them has one or so at theirs at a time, as with fifteen, and not
  three times as many. With sixteen people about and 32 residents,
  everyone visits as often as they lean to. A test holds both.

**Calls that were mine.** Thinning the visits; that the sign keeps the
owner's plain name; that the name is a word of `ui.ron`.

**Checked.** The game's tests (a day of 47 residents about one person:
between 0.3 and 1.3 guests at a time, seldom more than three; sixteen
people and 32 residents: a guest at each some of the time). The desktop
build, hidden: 47 residents offline, about twenty of them sat or lying
at any moment, one of them over at my plot; two copies against a server
on this PC with all 46 about the same thing within a metre on both. On
the VM with this PC's own account: "bunnihihi's bot" over the girl who
walks bunnihihi's land, in her house, "bunnihoho's bot" over the one on
bunnihoho's, and "Sagwa" plain over a made-up owner's, each stood beside
under the game's own camera and looked at in a picture. Frame times are
`docs/rendering-and-frame-rate.md`'s first entry.

**Not checked.** On a phone. An app from before this (APK 52) beside a
new one: it walks 47 too, under plain names, visiting three times as
often, and what its residents are about at a moment is no longer what a
newer app's are.

## Residents: someone of the computer's on each plot nobody here has (2026-11-23)

Hajun, with the 48 plots always filled (`docs/land.md`):

> if >= 16 players are online in a channel, then no NPCs. if < 16
> players. for example, 3 players -> the 13 next plots will have an AI
> NPC spawn in each plot with that plot owner's name, and they should
> just stay on their plot/house and do random things to make them seem
> like a real person, and maybe sometime go over to the plot of the real
> player and go inside and do random things again like sit/sleep and
> leave and go back to their own plot, etc. random based on random
> personality/mbti assigned to each bot, kind of like bridge wars bots.

**What there is** (`src/resident.rs`).

- **One on each of the first sixteen plots that nobody of your channel
  has** (`land::Deeds`, `Holder::Minded`: the server says which; offline
  all but yours), under the plot's owner's name and in their look, a real
  person's or a made-up one's. **They take the place of the wanderers
  round the pond**: the town has sixteen people in it now, not 32
  (`roundtown_net::TOWN_BODIES`), and `lobby::fill_the_town` is gone. One
  goes as someone real comes to that plot, and comes as they go for good.
- **Each device walks its own**, as it always has the town's people
  (Hajun's choice of 2026-10-08), and nothing of them is sent. **But
  what each is doing is worked out from the town's clock and from who
  they are** (`resident::plan`, a sum with no chance in it but dice made
  of those two, `dice`), so that every device has the resident of a plot
  about the same thing at the same place: on the sofa, or on the way to
  the same neighbour's. Two copies on one server had all fourteen about
  the same thing, each at the same point to the centimetre.
- **Who one is** (`Persona`): four leanings, each one of two ways and
  then by how much within it: out among people or at home (a visit in 7
  to 13 sessions of a hundred, or 2 to 5; how much of the time on their
  feet is out of doors), on their feet or settled, a napper or a sitter,
  steady or changeable (how often two blocks running are one thing; how
  fast they walk, 45 to 75 parts in a hundred of a body's pace). Every
  one a share the dice are thrown against, as his rule for Bridge Wars'
  bots has it.
- **The day**: blocks of 30 s, each resident's begun at its own moment,
  and sessions of eight. At home a block is one thing: out on the lawn,
  at the front of the plot looking out, about the house, sat on a chair
  or a sofa, lying on a bed (`What`); where there is none of it, the
  next best (no bed, a seat; no seat, about the house; no house, the
  lawn). **A visit** is a session at the plot of someone real in your
  channel, you among them: six blocks there, about their house and on
  their chairs and beds as at home, and two for the way back and home.
  Of fifteen residents about one person alone, one or so is at theirs at
  a time (a test holds it between 0.3 and 1.3, and to seldom more than
  three).
- **Getting there** (`Search`, `Walkable`): over the town's own shape,
  what stands on its plots with it, half a metre to a square, as a body
  could walk it, through a door and round a table; found a little each
  frame (40 squares nobody has looked at before, among all of them), and
  what is found of a square kept until the town's shape changes. Between
  plots: out by the plot's front, along the pavement 75 m out from the
  middle, clear of the street lamps and the plots, the shorter way round,
  and in by the other's front (`stages`). One that gets no nearer for 1.6
  s looks for another way, and after two of those is put where it was
  going.
- **Sitting and lying are `rest`'s**, as yours are: a resident is put on
  the place from beside it, within sight of it and 3.4 m (`beside`; a
  bed's middle is further than a step from anywhere beside a bed), in
  its pose; a place someone is on is left to them, and one who finds
  theirs taken looks for another. Up again when it is time for something
  else, or the thing is moved from under them.
- **To be talked to** as the town's people were (`talk`), but not while
  sitting or lying; **their names over them online or not**
  (`tags::Named`: it is who the plot's sign says). Standing about, one
  looks another way now and then, and the restless hop: those small
  things are each device's own.
- **While you are anywhere but the town they stand where they are.**

**Calls that were mine.** That they take the wanderers' place. Worked out
from the clock rather than sent or left to chance on each device. The
four leanings and every number in them; 30 s and eight; how often a
visit. That a visitor keeps to the house more than its owner does. No
visiting each other, only someone real. The pavement ring for the way.
Putting one that is stuck where it was going. Not to be talked to lying
down.

**Checked.** The tests (who one is, the same everywhere and no two
alike; a day mostly at home, a visit one stretch at one plot, the same
asked twice; a way round a wall and through a doorway, found a little a
frame and the same way, nothing looked at twice; beside a bed and never
through a wall; between plots by the gates and the ring). The desktop
build, hidden: fifteen of them about their plots offline, several sat
and lying in their own houses; four sent to a furnished plot of mine by a
switch since taken out, one on the chair at the table, one on the sofa,
two on the two places of the double bed, seen from in the house and from
over it; frame times with way-finding as it is now (a way took up to 157
ms in one frame before it was shared out; since, the longest frame of
thirty seconds was 28 ms, `docs/rendering-and-frame-rate.md`). Two hidden copies on a server on this PC: the same plans
and places on both.

**Not checked.** On a phone, where looking for a way costs more. A
resident through a real person's house upstairs (a place more than 2.2 m
over the plot is left alone). By night. A resident and a bus.

## Sitting and lying down, and the bubble a thing in the world (2026-11-22)

Hajun, after APK 50:

> i want players to be able to sit, and lie down on chairs, sofas, and
> beds. i think the proximitybubble thing can work here, but reduce the
> activation distance so u need to go closer to the chairs/sofas and make
> the icon instead of ... to be a chair_sit icon, and lie_down icon and i
> dont want proximity bubbles in general to scale when u zoom the camera
> out, the scale should stay the same, so instead of a screen based ui
> button, make it a in-world ui button that just rotates according to
> camera viewing angle, similar to how the text in the "player's land"
> sign is made, and make the sitting animation and the lying down
> animation&pose, so for the sofa there should be 2 places u can sit so 2
> proximity bubbles, chair just 1, and single bed just 1, and double bed
> 2. and it will do a very quick animation to get to that pose and stay
> that pose and both animation + the pose should be visible to other
> players. and jump to leave that pose and go back to normal.

**The bubble** (`src/bubble.rs`; `talk`'s was a button on the screen
since 2026-10-31, below, "You can talk to anyone").

- It is a flat sheet in the world with its picture on it, as the words on
  a plot's sign are (`docs/land.md`): its circle 0.62 m across wherever
  the camera is, the point of its tail on a point of the world
  (`Bubble::tip`), turned every frame to face the camera squarely
  (`face_the_camera`); unlit, blended, behind whatever stands in front of
  it. It pops up as it comes, as it did. So zoomed out it is small, as a
  head is, and no longer grows over the town.
- **A tap on it** (`bubble::tap`): a tap on the world, read from the
  editor's pointer, that lands on it, or within 5 of the screen's short
  side of its middle where it is smaller than that; the nearest, with
  two in reach. It is told to whoever put the bubble up (`Tapped`), and
  nothing else answers that tap (`EditMenu::swallow`, `TookTap`).
- **What is in it** (`Says`): three dots, someone sat on a chair, or
  someone lying on a bed, drawn in code as the dots always were, and
  drawn again at each smaller size the picture is kept at.
- **Talking** (`talk::show_bubble`, `tapped`) is as it was but for that:
  over the nearest within 4.5 m, one at a time, E on desktop. Over
  someone with a name it is still a name and a half's height over the
  name's foot as the screen has it, so further over their head the
  further out the camera is (`over_a_name`). Last Stand's chests have the
  same bubble.

**Sitting and lying** (`src/rest.rs`).

- **Where** (`rest::spots`, by a piece's name): the cafe chair, one place
  to sit; the soft couch, two to sit back on; the single bed, one to lie
  on; the double bed, two. Measured off Hajun's models: the chair's seat
  0.77 m up, the couch's cushion 0.62, the beds 0.95 with a pillow 0.22
  over that. In the town alone, on anyone's plot.
- **The bubbles** (`rest::offer`): within 1.5 m of the edge of such a
  thing (to 1.9 once up), on its floor, a bubble over each of its places
  that is free, in sight of your eyes, and no further than 3.7 m (a body
  is put no further in one go than the server lets one move, 4 m). Not
  in edit mode, nor in your car, nor with anything else up.
- **Down** (`rest::sit`): a tap on one, or E on desktop for the nearest,
  puts your body on the place, facing as the thing does, and in a fifth
  of a second it is in the pose (`roundtown_net::Pose`): **sat**, the
  hips on the seat and the knees bent over its front (the chair); **sat
  back**, leant a little, the legs out along the cushion (the couch,
  which is deep for a body this size); **lying**, on the back with the
  head on the pillow and the body a little up toward it. The model is
  moved under the body (`rest::settle`, as `arena::squash` lays one
  flat) and the limbs turned by `animate_walk` (`rest::limb`).
- **Up** (`rest::leave`): the jump button, or Space, and you are on your
  feet where you stood before, jumping. By itself, with no jump, as what
  you rest on is picked up, moved or taken away, and as you get into
  your car or leave the town.
- Meanwhile nothing moves you and you are in nobody's way
  (`move_bodies`), you throw no punch, nobody is talked to, and what you
  rest on does not fade from under you (`see_through`).
- **Everyone sees it**: your body is where it rests, which your moves say
  as ever, and how you rest is said to the server (`Ask::Rest`) and told
  to everyone in your channel whose app asked (`Ask::Resting`,
  `Tell::Resting`; `docs/online.md`). Who rests is drawn so
  (`rest::their_poses`), getting into the pose as you do, their name
  lower over them (`rest::lower`).

**Calls that were mine.** The bubble's size (a little more than a head
across) and the tap's least reach. A sofa is sat back on with the legs
out, a chair with the knees bent: the couch's cushion is 1.2 m deep, and
sat on its front edge a body is a metre from its back. Up where you
stood before, not on the seat: got up on the seat and gone forward, a
body sat at a table landed on the table. A second place of a bed too far
to be put on in one go has no bubble until you come round to it. No sit
or lie pictures were in `assets/icons`: both are drawn in code. In first
person the camera is still where your eyes would be standing. The town's
people do not sit.

**Checked.** The three crates' tests (555, 260, 118). The desktop build
run hidden at 874 by 402 by day, on a copy of a real plot (39 things, its
owner's): the two bubbles over a sofa from a metre off and none from
1.7; a tap, sat back, seen from the front and the side, the sofa solid
under you; Space, back on the floor where you stood; the one bubble of
the double bed's nearer place, a tap, lying with the head on the pillow,
seen from the side and from over it; a chair at a table, sat, from the
front and from behind, and up again to the floor and not the table; the
bubble over one of the town's people near the pond, small from far off,
and a tap on it starting the talk. Two copies against a server on this
PC, each with one of two real plots: one sat on its sofa and got up, and
the other had it sat back there and on its feet again.

**Not checked.** On a phone: APK 51 was built and not installed, none
being attached. How someone else looks sat, from their side (the same
code draws them; only that they were so was read off the second copy).
By night. The single bed. First person.

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

(Until 2026-11-23: sixteen since, a resident to each plot nobody here
has, the first entry of this file.)

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
