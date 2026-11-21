# Land: the town's plots, and whose they are

The town's 48 plots, who has which, and what stands on them (`src/land.rs`).
Dated entries, newest first.

## Contents

1. 2026-11-22: what you own stands on your plot, and everyone sees it
2. Later on 2026-11-21: everyone in your channel has their sign, and you see it
3. 2026-11-21: the words are a thing in the world, a picture hung on the board; a metre and a half at the corner; the sign is solid; APK 44
4. Later on 2026-11-20: you arrive on your plot; the words slant with the board, on both its faces; more room at the corner
5. 2026-11-20: a plot for your seat, and the sign on it that says whose it is

## 2026-11-22: what you own stands on your plot, and everyone sees it

With the bag (`docs/bag.md` has the whole of it, and Hajun's words): a
building out of your bag is put down on your plot as a piece is in House
Builder, stays there from one run of the app to the next, and is seen
there by everyone in your channel.

- **Your plot is a square to build on** (`build::Yard`, made of the plot
  `land` reads off the map: `stand::yard_of`), and the space what stands
  on it is kept in: its middle, its ground, and its front, the side the
  sign stands on. A place in it (`roundtown_net::bag::Standing`) is x to
  the right of someone in the middle looking out over the front, y up,
  and -z out over the front. **A seat's plot changes from one day to the
  next**, seats being given as people come in; what stands is stood on
  whichever plot the seat has, in the same place of it
  (`stand::keep_mine`, `keep_theirs`). Every plot being 24 m square, what
  fits one fits all; and the sign is in the same corner of each, so what
  is clear of it on one is clear of it on all.
- **Only on your own plot, standing on it** (`stand::Home`), and nothing
  of a piece off it (`Yard::holds_piece`).
- **The town's shape has two layers laid over it now**
  (`Islands::lay_layer`): the signs' boxes, as before, and what stands on
  the plots (`build::reshape_plot`), each laid again as its own changes
  and both felt for by every body. On Blend In's venue, which is the
  town's map, they are there unseen as the signs are, past the ring.
- **You still arrive in the middle of your plot**: with something
  standing there you are put beside it, as a body in a wall is put out of
  it; with your house there, you arrive in your house.
- **The server keeps it** with the account, knows nothing of the map, and
  holds a place only to being on a plot (`bag::sound`: within 12.5 m of
  its middle).

**Checked** and **not checked**: `docs/bag.md`. Nothing on a phone.

## Later on 2026-11-21: everyone in your channel has their sign, and you see it

Hajun, having tried the town with someone else in it (the VM's journal has
this PC's account and one other in the town together at 16:16 UTC on the
20th): "it seems like only your sign shows, and you dont see other player's
signs. also the car only shows to yourself, not to other players. fix both,
but no need to rebuild apk or cargo run again". It was so, and said so in
the entries below: the server
gave every seat a plot to arrive on, but told nobody which seat anyone else
had. The car is in `docs/vehicles.md`; how the server tells a channel of
itself, and why it needed no new protocol, in `docs/online.md`.

- **The server says who sits where** (`roundtown_net::Tell::Channel`, a
  list of `Seated { who, seat, driving }`, the whole channel each time,
  sixteen at the most): to every device in a channel of the town, as anyone
  sits down in it or stands up, or gets into their car or out
  (`Towns::stir`, `players::tell_channels`). Only to a device that has
  asked (`Ask::Channel`, as it says hello), so an app from before is sent
  nothing it cannot read.
- **A sign stands on the plot of everyone it names** (`land::holders`,
  `land::keep_signs`): the plot their seat has (`Plots::of_seat`, as yours
  always was), saying "{name}'s land" with the name their body has here,
  bare, as yours is. So a sign comes once the person has been sent to this
  device, a moment after they arrive, and is made anew if they change
  their name. It is solid, as yours is. The town's AIs have none: they have
  no seats. Offline, and with a server that says nothing, yours alone, as
  before.
- **Someone coming or going costs the one sign** (`land::staying`): those
  that stand and are still wanted are left as they are. The first version
  of that day took every sign away and drew every picture again at each
  change, which with a full channel was sixteen pictures drawn in a frame
  whenever anyone came in.
- **My calls.** Asking to be told, in place of a new protocol number, so
  that APK 43 and 44 play on. The bare name on the board, no tag. Signs for
  your own channel's sixteen, not for all 48 plots: the other channels'
  people are not in your town. Nothing about what makes a plot whose: it is
  still the seat, given as you come in.
- **What it costs.** A picture of the words for each sign, at most about
  1.6 MB with its smaller copies, kept both in memory and on the graphics
  card (`RenderAssetUsages::default()`): some 50 MB with a channel full.
  Not measured on a phone; keeping it on the card alone would halve it.
- **Checked.** Unit tests: who has a sign (`a_sign_for_everyone_in_the_
  channel_and_for_you`), which stay (`a_sign_still_wanted_stays_as_it_is`),
  the server's channel stirred and told (`rooms`, `players`); all three
  suites (490, 252, 108). Two hidden copies of the desktop build against a
  server on this PC: each stood two signs, its own and the other's on the
  other's plot with the other's name ("mina's land", a picture looked at);
  the one who stayed kept its own sign, the same one, as the other came and
  went, and the other's was gone when they left. The VM after the deploy
  (17:15 UTC on 2026-11-20, 02:15 on the 21st here; protocol 24 still;
  nobody had been connected for 55 minutes; the binary before kept as
  `roundtown-server.prev-2026-11-21`): the VM's binary the same bytes as
  the build here, this PC's own account online on its plot with its sign,
  the journal clean.
- **Not checked.** On a phone: **APK 44 has none of this** (Hajun asked for
  no APK), so a phone shows its own sign alone until the next one. Two real
  accounts on the VM. A channel with more than two people in it. A Korean
  or Vietnamese name on someone else's sign online (the words are drawn by
  the same code as your own, which a test draws in both).

## 2026-11-21: the words are a thing in the world, a picture hung on the board; a metre and a half at the corner; the sign is solid; APK 44

Hajun, of the slanted words: "i dont understand tho. why dont u make like a
in-world text obj rather than trying to rotate/warp the text? even now, when
the camera zooms in very close to the text, the text warps weird." And: "also
the sign can have slightly less margin from the corner". Then, having looked
at that, "build apk", and as it was begun: "oh sign need to be solid tho".

They were right. The words had been text on the screen, moved and slanted
each frame to lie over the board: that can narrow a line and slope it, but
not make its near end bigger than its far one, and they were held to twice
their laid-out size, so that up close they stopped fitting the board at all.
It had been done that way because names over heads are, which is known to
work on the phone, and a thing drawn in the world had not been seen there.

- **The words are drawn once into a picture** (`land::draw_words`), in the
  game's own two fonts, as their files are (`font::files`), by swash, the
  font reader and rasterizer Bevy's text is itself drawn with (a
  dependency of the game's own now, `Cargo.toml`; no new crate is built).
  Each line in the middle across, one under another; each letter from
  Inter, or from Pretendard where Inter has none (Hangul), set as far
  apart as its font says, with no kerning or shaping, which the letters
  the game lets be written (precomposed, NFC) do without. As big as lets
  the picture fill 94% of the board's room across or 92% down, whichever
  comes first (`WORDS_FILL`; the room is 1.3 by 0.65 m inside the frame):
  a short name on two lines is 0.22 m to the em, the longest name 0.07.
- **The picture is no bigger than the words** with 0.12 em clear round
  them, at 640 pixels to a metre of board (`PIXELS_A_METRE`): 526 by 385
  for "hajun's land", 783 by 125 for the longest name. White everywhere,
  the letters in its alpha, and **with every smaller copy of itself down
  to a pixel** (`words_picture`, `halved`), so that from far off they are
  drawn from one about their size on the screen and do not sparkle. About
  a megabyte for a short name on two lines, half that for the longest.
- **It hangs on two flat sheets**, children of the sign (`keep_signs`):
  one 4 mm clear of the board's front face and one of its back, inside the
  frame's depth (`FRONT_OUT`, `BACK_OUT`; a test holds both to the
  model), each looking out from its face and reading left to right from
  there. So the words are seen as the board is, from any side and any
  distance, and anything in front of them hides them, as nothing did
  before. One picture, one sheet and one paint serve every sign that says
  the same.
- **Unlit, and 2.5 times as bright as their colour** (`WORDS_GAIN`, as
  `range::GAIN`): lit as the board is they came out grey on whichever
  face was in shade, and Hajun's colour for them is white. They are white
  on both faces by day and by night, as they were on the screen.
- Nothing of it runs each frame: the system that laid the words on the
  screen (`place_words`) is gone, with `lie` and its two nodes.
- **The sign stands a metre and a half in** from the corner and from the
  edge (`FROM_CORNER`, `FROM_EDGE`; two metres the hour before, half a
  metre before that). The number is mine.
- **The sign is solid** (`land::solid`): a closed box as wide as it post
  to post (3.06 m), as tall (2.73) and 0.3 m through, standing where it
  stands, is laid over the town's own shape (`Islands::lay_over`, which
  Bridge Wars' bridges are laid over their map with; the town had nothing
  laid over it). So it is a wall like any of the map's: you, the town's
  people and the car you drive are kept out of it by what keeps them out
  of the map (`resolve_walls`), with nothing new in how a body moves.
  - A box, not the model's own triangles: the board is 0.1 m thick, and a
    body is tested against the town a quarter of a metre at a time
    (`island::LATTICE`); and nobody passes under the board, 1.04 m up, or
    over it. The box is as deep as the posts, which is as thin as that
    test lets a wall be.
  - A body stops with its middle 0.70 m from the sign's middle, measured
    at the first seat's (its own half width, 0.45, and the quarter-metre
    squares the box fills; less by up to an eighth of a metre where a
    sign falls otherwise on those squares).
  - The town's shape is its model's, which a game of Blend In is played on
    too: the boxes stand there unseen, 79 m out and more, past the 70 m
    ring nobody in that game crosses.
  - Only on your own device, as the sign is: someone else's walks through
    yours, on your screen, until the server says whose plot is whose.

**Mine**: unlit rather than lit; 640 pixels to the metre; that the words keep
their two lines and their size; a box for the whole sign rather than its
posts and its board apart.

**Checked**: unit tests (486 of the game's: the words drawn as big as the
board has room for and no bigger, a long name, Hangul and Vietnamese, the
picture's smaller copies, the sheets clear of the board's faces and inside
its frame by the model; the box closed and looking outward, filling where
each of the twelve signs round the pond stands, the turned ones too, with
no way through along it, and nothing a stride off it or where you arrive;
laid over the town's shape and taken off). The desktop build, hidden,
pictures looked at: from 8 m; from 1.3 m, sharp; from 2 m to the side and
above, the near end of each line bigger than the far; the back face, reading
the right way; from 22 m, steady; from the middle of the plot; by night, both
faces. And a body walked straight at the sign for three seconds by a
throwaway module, from 4 m in front and from 4 m behind: it stood 0.70 m
short of the sign's middle each time.

**APK 44** was built with all of it (`docs/building.md`). **Not checked**: a
phone. A picture made at run time with its own smaller copies, and not a
power of two across, has not been drawn on one: if the words come out wrong
there, that is the first thing to look at (`words_picture`, one level and
no more). The car against the sign was not tried, only what stops it read
(`drive::settle`).

## Later on 2026-11-20: you arrive on your plot; the words slant with the board, on both its faces; more room at the corner

Hajun, having looked at the first of it in the desktop build: "it seems like
the text is rotating the wrong way when i look from side. also put more
margin from the sign to the edges of the corner. and u need to spawn on your
plot in the center, facing toward the fountain." And as that was being done:
"and the text should be on both sides of the sign".

- **The words slant, and no longer lean.** They had been turned as one
  block to the slope the board's width has on the screen, so that from the
  side and above every letter leaned over with it, where on a board its
  sides, and the letters' uprights, stay upright and only its lines slope.
  Now a pixel across the words goes the way the board's width runs on the
  screen and a pixel down them the way its height does (`land::lie`):
  narrowed and slanted with the board, the letters standing. A screen's
  node can only be turned and scaled, so the words are two nodes, a box
  and the letters inside it, and the slant is the letters' turn, then the
  box's scale, then the box's turn (`SignWords`, `SignLetters`; bevy_ui
  0.19 gives a node its parent's turn and scale about its own middle,
  which is the box's, the box being as big as its letters).
- **On both faces of the board**: whichever the camera is in front of,
  reading left to right from there (`place_words`).
- **The sign stands two metres in** from the corner and from the edge
  (`FROM_CORNER`, `FROM_EDGE`; half a metre and 0.6 before). How much more
  was mine.
- **You arrive in the middle of your plot, facing the fountain**, wherever
  you used to arrive on the path round the pond: as the app opens, and
  coming back to the town from anywhere (`island::arrival_of_a_person`, in
  `lobby::spawn_player` and `lobby::send` for your own body). The town's
  AIs arrive round the pond as ever.
  - **The server has it too**, or a body it had put on the path would be
    put back there at its first step. It has no map, so the middles of the
    sixteen seats' plots are written in `shared/` (`roundtown_net::land::
    HOMES`, `land::arrival`, `arrival_of_a_person`), and a test of the
    game's holds them to the map (`land::tests::a_person_arrives_in_the_
    middle_of_the_plot_their_seat_has`): **after an export of the town's
    map that moves a plot, write them again and deploy the server.**
  - **A device that has only just opened** stands on the first seat's
    plot, the seat it has offline, until the server gives it one: let in
    standing within two metres of the middle of any seat's plot, it is
    stood on its own (`land::just_arrived`, in the server's `admit`);
    let in anywhere else, it stays where it stands, as before.
  - Nothing sent has changed: `PROTOCOL_VERSION` is 24 still. An app from
    before arrives on the path, and at its first step the server puts it
    on its plot.

**Checked**: unit tests (the game's 484, with the slant from each side, a
board seen from behind, and the written middles against the map; `shared`'s
252; the server's 106). The desktop build, hidden: the game's own camera as
you arrive (the middle of the plot at (25, 90), the fountain ahead, the sign
at the plot's front left as you stand); the board from the right and the
left, steeply and from above, and from behind, enlarged: the lines sloping
with the board, the letters upright, on both faces; the corner from above.
**Two hidden copies against a server on this PC**: the first stood on (25,
90) and stayed there, the second, come in eight seconds later, on its own
plot at (66, 66), both facing the fountain, neither put back.

**The server was deployed** with it (2026-11-20 13:31 UTC; CLAUDE.md's
rule, a change to `shared/`; not asked for in so many words, and told to
Hajun): nobody was connected (the only one that day had been this PC's own
account, gone at 13:21 as the game was shut to be built again); the binary
it replaced is kept on the VM as `roundtown-server.prev-2026-11-20` (the
one of 2026-11-18 13:33); after, the service was active, its binary's sha1
the local build's (`bc8eb58c...`), its journal clean. From an uncommitted
tree. No APK: 43 plays on, protocol 24.

**Not checked**: a phone. Coming back from a game online.

## 2026-11-20: a plot for your seat, and the sign on it that says whose it is

Hajun, after two days of thinking a land economy over (below): "ok lets
first get started with the sign with the text. the sign asset is
plotsign.glb. make it so each player, when they join the lobby, they get
assigned a plot and their plot has the plotsign in the right side corner
(from when looking from pond, bottom right corner so its near the pond and
facing the pond. or if the house is next to a street, the fence should be
bottom right corner from looking from that street and facing the street. and
the plotsign should just say "hajun's land". then do cargo run".

### What there is

- **The plots are read off the town's map** as it spawns (`land::read_map`,
  `read_plots`): every separate piece of the material `plot` with four
  corners is one. Nothing of them is written down in code: move a plot in
  Blender and everything here follows. Today's map has 48, each 24 m
  square: 12 about 93 m from the pond, 16 about 126 m, 20 from 156 to 166 m
  (`Plot::ring`, a new ring wherever the next plot is more than 15 m further
  out). They are kept ring by ring and round each ring from the game's +Z
  by way of +X (`land::Plots`), so that a plot's number is the same on
  every device with the same map.
- **A plot's front** is the side its sign stands on. A side has a street
  along it if the street's surface is on top anywhere within 13 m out from
  the side's middle (`STREET_REACH`; the materials `road` and `roadwhite`,
  with nothing lying on them: the map's road runs on under its pavements,
  which lie a quarter of a metre over it). With a street on one side, that
  side; on more than one, the side more toward the pond; on none, the side
  toward the pond. What that comes to on today's map (a test holds it):

  | Plots | Front |
  |---|---|
  | the 12 round the pond | the pond, over the ring road (10.5 to 12 m of pavement) |
  | 8 of the middle ring, beside a street that runs out from the pond | that street (7.5 m of pavement) |
  | the other 8 of the middle ring | the pond: no street near |
  | 8 of the outer ring, beside a street that runs out from the pond | that street |
  | 8 of the outer ring | the road round the town's edge, behind them from the pond (10 m) |
  | the 4 at the outer ring's corners | the pond: the edge road is 20 m off |

- **The sign** (`plotsign.glb`, Hajun's: two posts and a beam 3.06 m wide,
  a board hung from it by two chains) stands inside the plot along its
  front, at the right hand end of it as someone standing in front of the
  plot has it, its end half a metre in from the corner and its middle 0.6 m
  in from the edge (`FROM_CORNER`, `FROM_EDGE`), facing them.
- **Whose plot is whose** goes by the seat you have in the town, which the
  server already gives each of a channel's sixteen and tells you
  (`Tell::Arrive`; offline you are seat 0): seat by seat, the plot whose
  sign is nearest where that seat arrives, of those left in the ring
  nearest the pond that has any left (`land::seat_plots`). The first eight
  seats, which arrive spread round the pond, each have their sign about 50
  m behind them across the ring road; the twelve plots round the pond go
  to seats 0 to 11, and four of the middle ring to the rest. Seat 0's is
  the plot at (25, 90), its sign at the corner where the ring road meets
  the street that runs south from the pond.
- **The words**, "{name}'s land" on two lines (`words.land.sign`, a `\n`
  in it; white, `colours.land.sign`), are drawn as names over heads are
  (`tags`): text laid out once on the screen, and every frame moved,
  scaled and turned to lie on the board as the camera sees it
  (`land::place_words`, `lie`): as wide and as tall as the board's room for
  words is from there (1.3 by 0.62 m inside its frame), so that seen from
  the side they narrow with it, and tilted as its width runs across the
  screen. Put away from behind the board, nearly edge on to it, further off
  than 60 m, smaller than 8 pixels, off the screen, and anywhere but the
  town. They are laid out at 9 shares of the short side and shown at no
  more than twice that.
- **Only your own sign stands**, on any device: a device knows its own
  seat and nobody else's. `RT_LAND_SIGNS=all` on desktop stands one on
  every plot, with your name, to look at them.

### Mine

- With streets on two sides, the side more toward the pond: the twelve
  plots round the pond all look at it, the eight of them beside a street
  that runs out from the pond too. The other reading of Hajun's words is
  that those eight look at that street: `most_toward_the_pond` in
  `read_plots` is where it would change.
- 13 m as how near a street has to be. It leaves out the edge road behind
  the outer ring's corner plots (20 m) and takes in the one behind its
  other eight (10 m), which so look away from the pond.
- A seat's plot by the nearest sign, the inner ring first. Past the
  eighth seat it is the nearest of what is left, which for two of them is
  across the pond (110 m).
- Two lines on the board, which a short name fills better than one; that
  the words are sized to the board, and so can only be read from near, as
  a real sign's are (on a phone's screen from about 10 m).
- White words with the house's soft shadow; no stroke.
- The town's AIs have no plot.

### Not done, and why

- **Everyone else's sign.** The server has to say who has which plot (a
  component on each body, or their seat in `Member`): a new protocol, so a
  deploy and a new APK and iPhone build. Left until Hajun asks for those;
  nothing in `shared/` or `server/` was touched, and `PROTOCOL_VERSION` is
  24 still.
- **The sign is not solid**: a body walks through it.
- **Nothing hides the words**: a house in front of a sign would not.
- No APK, no deploy, no commit. Not seen on a phone.

### Checked, and where

- Unit tests (`land::tests`, which read the town's map and the sign's
  model out of their files): 48 plots in rings of 12, 16 and 20, each 24 m
  square; each ring's fronts as in the table; every sign inside its plot at
  the right hand end of its front; sixteen seats each with a plot of its
  own, the inner ring first, seat 0's as above; the model's width, its
  board's height and the room inside its frame as the code takes them; that
  the model is not left out of the APK; how the words are laid on the
  board.
- The desktop build, hidden and offline (`RT_BACKGROUND=1`), the camera
  held where a throwaway module put it (`HeldCamera`), pictures looked at:
  the sign at seat 0's plot from 9 m, 4 m, from the side and above, from
  behind (no words), by night, and at a phone's 852 by 393; with
  `RT_LAND_SIGNS=all`, plots of the middle and outer rings and a view
  straight down over the street that runs south, the signs where the table
  has them.
- Not checked: online against the live server (the seat it gives is used
  as it is); a phone.

### Where this is going (Hajun's words; nothing of it built)

- 2026-11-19, asking for thoughts: "first make some kind of private land
  ownership economy on the main lobby - which i still not sure exactly
  what way is good."
- 2026-11-20, of a plan in which every plot has a price its owner sets and
  5% of it a day in tax: "so if i set my house to 10000$, i pay 500$ every
  day? is that like real life real estate in korea?" (It is not.)
- 2026-11-20: "maybe it should be like everyone start with their own land,
  and maybe there can be some kind of order so that better lands are placed
  somewhere and worse lands are placed somewhere? im not sure if i should
  do it so that the close island to the fountain is the more expensive or
  it should be like sector based. so its not fixed global land plot
  location but rather like based on the current players in the lobby
  online". What makes a land "better" was asked of them and is open.
- Found on the way: every prize is made new by the server and every fee is
  gone, nothing pooled; a new account has 1,000; House Builder saves
  nothing after a game and its 19 pieces have no prices; the town has no
  houses. `assets/coinfountain.glb` is Hajun's, loaded by nothing yet.
