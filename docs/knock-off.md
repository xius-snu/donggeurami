# Knock Off

> The full account of Knock Off: what was built, why, in whose words, what
> was tried, and what was and was not checked. `CLAUDE.md` has the rules
> and how things stand now. Dated entries are newest first.

## Contents

1. Since 2026-11-18 Knock Off is a game people play, online and offline: four on a disc of ice in the sea, everyone aiming at once in secret and sliding at once, the ice breaking a metre a turn

## Since 2026-11-18 Knock Off is a game people play, online and offline

Hajun, 2026-11-18, among the four new games: "You spawn on a tiny icy
circular island with 2~4 other ppl (fill with ai like bridgewars and its
deathmatch so 1v1v1v1). Controls (left side is joystick for direction,
right side is a up down arrow for controlling intensity). Everyone is
frozen in place, and before every certain time interval like 10 second,
you choose the direction and intensity that you wanna slide. Then after
the interval, everyone goes toward that direction and intensity and
collide, some ppl will be knocked off. You keep going until last player
surviving is the winner. If you are out, you can spectate or exit. Map =
iceisland (not sure what camera is good for this one, you choose)". And of
all four: "each of these games have a coin entrance fee and payout.
Generally should be above 50% to win net positive coin, and if not, net
negative coin", and "the details i didn't mention are yours to decide to
make each game polished and fun". **Everything below not in his words is
mine** (the session that built it, and the ones before it that built its
rules and its table), and is said so.

`PROTOCOL_VERSION` 22 for all four games (bumped once, for all of them).
**Nothing of it has been deployed, built into an APK or seen on a phone.**

Files: `shared/src/knock_off.rs` and `shared/src/knock_off/` (`slide.rs`,
`computer.rs`, `loiter.rs`): the rules, one piece for the server and the
device; `server/src/knock_off.rs`: the game at its server table;
`src/knock_off.rs` and `src/knock_off/` (`ice.rs`, `screen.rs`): the
device's side; `assets/ui.ron` `colours.knock_off`, `words.knock_off`
(and their structs in `src/ui.rs`). The table round it (seats, fee,
countdown, results, prizes) is `server/src/tables.rs` and `src/table.rs`
(`docs/minigames.md`).

- **The table** (`knock_off::SHAPE`): four seats, every one for themselves.
  People first; 10 s after the last person joined, the computer takes the
  seats left, one at a time, as in Bridge Wars ("fill with ai like
  bridgewars"); a group is in the same game, on no team. A 5 s countdown;
  the results up 15 s. **50 coins to join; paid by place 130, 60, 10 and
  nothing** (net +80, +10, -40, -50: the first two of four come out ahead),
  those who went in at the very same moment sharing the places they cover;
  the computer is paid nothing. Leaving before the game is the fee back;
  after, your place is kept and paid.
- **The island**: Hajun's `iceisland.glb`, a disc of ice 24 m across (12 m
  out, `RADIUS`, a test reading it off the map), its top at 0, in the sea.
  Waiting, everyone walks about on it with the town's controls under the
  game's camera; **someone who walks off into the sea is stood back on
  their spot** (mine: nobody jumps in Knock Off, and the ice's rim is more
  than a step out of the water; offline here, online by the server, asked
  as Bridge Wars' fall is). **The computer's players**, as each comes out
  onto the ice, stand about on it and now and then stroll a few metres
  somewhere else, well in from the edge and clear of everyone
  (`knock_off::Loiter`, mine, the server's online and the device's
  offline: a stroll ends within 8.5 m of the middle, 1.2 to 5 m off, at
  1.4 to 2.8 m/s; it stands 1 to 4.5 s first; then strolls, looks round at
  someone or at the middle, or stands on, each thrown afresh, as Hajun's
  rule for the computer's players has it: "if some thing becomes a definite
  if-then, then its not a real human, but a program"). **At "Go!"**
  everyone is stood on four spots 7 m out on the diagonals, facing the
  middle (`START_RING`).
- **A turn** (the rules' module has every number; the G1 report of
  2026-11-18 has why):
  1. **Aim**, 12 s the first turn (mine: the game being new) and 10 after
     (Hajun: "like 10 second"). Nobody moves (`Frozen`). Your aim is yours
     alone until the reveal: the server is sent it as it changes, at most
     ten times a second and always the last of it, and nobody else is sent
     anything of it. **Ready** ends the aim for you and keeps your aim as
     it is; once every person still on the ice is ready the aim is over for
     everyone (the computer's players make up their minds each at a moment
     of its own, a tick over their heads as they do). **A turn begins with
     no aim at all** (mine): standing still is what not aiming is, so that
     someone who does nothing is not thrown off the ice by last turn's.
  2. **Reveal**, 0.8 s: everyone's arrow is shown, each in its owner's
     colour, and everyone turns to the way they will go.
  3. **Slide**, until everything on the ice has stopped, at the most 6 s:
     everyone sets off at once; bodies are discs 0.6 m out; they bounce off
     each other (0.9 of the speed kept along the line between them, equal
     weights: one sliding into another standing hands it nearly all its
     speed); the ice slows everyone evenly, 5 m/s each second. **Power is
     distance**: power p slides p x 2.2 m on its own (`reach`), so that the
     arrow is as long as the slide; 10 all but crosses the island (2.6 m a
     step was tried, and whole games were over in two or three turns). A
     body whose middle passes the edge goes over, out, and into the sea.
     **The same to the bit on every machine**: the slide goes by `+ - * /`
     and the square root alone, a fixed step of 1/120 s and the pairs in
     one order (`slide.rs`); the server plays it out and sends where
     everyone began and how each set off, and every device plays it again
     from there by the town's clock.
  4. **The ice breaks**, 1.2 s (mine: "the island's edge cracks and breaks
     off"): a metre a turn, 12, 11 ... 3, then 2.25, 1.5, 0.75 and none, so
     that every game ends, and ends with everyone close together; whoever
     stands on what breaks goes in with it.
  - **Places** go by when you went in, the last first: by the turn, and in
    a turn by the moment, to the step of the slide (of two who knock each
    other in, whoever stayed on longer comes first); those the ice broke
    under after the slide's; whoever went in at the very same moment
    sharing a place. Whoever is left alone on the ice wins; with nobody
    left, whoever went in last. The results' score is **the turns each was
    still on the ice at the end of** ("turns survived", mine: the place has
    its own column there).
  - **The computer's players**, each one of six kinds (bully, sniper,
    turtle, wildcard, counter, brawler), weigh a few dozen moves each turn
    by playing each slide out against guesses of what the others will do,
    and pick one by how it scored, thrown off by their kind's error. A
    thousand games of them: 6 turns on the median.
- **The camera** (Hajun: "you choose"; mine): **fixed over the island from
  your own spot's side, looking down at its middle at 55 degrees**, as far
  out as keeps the whole island and 2 m of sea round it on the screen,
  clear of the plate at the top (11% of the screen) and a little of the
  other edges, fitted to whatever shape the screen is (16:9, 19.5:9 and
  4:3 tested). Your spot is toward the bottom of the screen, and **the
  stick's up is away from the camera**, across the ice toward the middle,
  so that aiming is pushing yourself across it. It stays put from the
  moment you sit down to the end: waiting, aiming, sliding and watching once
  you are out (`HeldCamera`). Where two meet in a slide it shakes a few
  centimetres for a tenth of a second (1.5 to 6 cm, by how hard they met).
- **The buttons** (Hajun: "left side is joystick for direction, right side
  is a up down arrow for controlling intensity"): the stick on the left is
  the town's own, read for which way, camera-relative; let go, the arrow
  keeps the last way. On the right, where the jump button would be: an up
  arrow and a down arrow (the jump button's arrow, the down one turned),
  between them a bar of ten steps filled in `power` over the plate's grey
  and the number over it, stroked dark; a press is a step, and held, after
  0.35 s, a step every 0.08 s, however few frames that is in; under them
  **Ready**, a round button with a tick, green once pressed; and the chat's
  keyboard to its left. The arrows and Ready are faint while there is no
  aiming to do. **Desktop**: W, A, S, D or the arrow keys turn the way,
  eight ways, as the camera looks; Q and E, Page Up and Page Down, and the
  mouse's wheel set the power, E and Page Up up; Space is Ready. While a
  line is typed into the chat every key is the box's.
- **What is drawn on the ice** (mine, all of it; `ice.rs`):
  - **Your arrow**, while you aim: from 0.75 m out from your feet (just
    outside a body), as long as your slide would go on an empty island, a
    flat shaft and head lying on the ice in your colour, with a ring where
    you would stop (as wide as a body), and a dark rim round all of it so
    that it reads on the ice's white by day and its dark by night. Built
    again only as its length changes; turned and moved, not built, as its
    way does. Nobody else's is drawn until the reveal, when everyone's is.
    With no power there is no arrow.
  - **A ring at everyone's feet** in their colour, so that who is who is
    plain from over the ice.
  - **The cracks**: while everyone aims, looks and slides, a thin dark ring
    of cracks where the ice will break at the end of the turn, wandering a
    little either side of its line, with short cracks running out from it
    toward the edge; on the last turn, cracks running out from the middle.
    Unlit, they come out lighter than the darkened ice by night.
  - **The ice breaking**: the island's model narrowed across the ground to
    the ice that is left (its land's `Transform.scale`, x and z: whole again
    away from the game), while a ring of slabs in the ice's own material,
    where the edge was, lies a moment as the edge in pieces, then tips
    outward and drops into the sea, some with a splash.
  - **A splash** where anyone goes into the sea, the moment they reach the
    water (a ring spreading 2.3 m on it and a dozen drops thrown up), a
    smaller one where a slab does. Whoever goes over slides on off the
    edge, drops in, and is gone once under (`Dead`, drawn nowhere).
  - **A puff of ice dust** where two meet hard enough to count as a hit.
  - **Sliding, a body is posed for it**: knees half bent, arms out to the
    sides (the intro's landing pose, `Landing`), turned the way it set off;
    and settled onto where the server says the slide left it over 0.2 s,
    if it is anywhere else (it has not been, in every slide checked).
  - **Each seat's colour** is its own in the file (`colours.knock_off`,
    mine: Bridge Wars' red, blue, green and yellow, as bright as they read
    on white ice): drawn unlit and 1.6 times as bright, so that the
    tonemapping leaves a red red and not a coral (2.5 times, the punch's
    range's, made them pale).
- **Over the world** (`screen.rs`):
  - **The plate top centre**, the table's board's look: "Turn 3" over
    "Aim! 7" (whole seconds), "Ready · 7" once you are, "Slide!", "The ice
    breaks"; "Last turn" on the last; "Get ready" before the first; out,
    "You're out · 3rd".
  - **Names over heads**, each in its seat's colour with a dark stroke,
    **as big wherever they are**, as Bridge Wars' are, and a green tick over
    whoever is ready; never your name over you (Hajun, 2026-10-31), but
    your tick, once you are ready. **Why not the town's names** (`tags`):
    sized by how far off a head is, from a camera far enough out to see
    the whole island they were a centimetre high on a phone, white on white
    ice. `tags` leaves Knock Off's ice to these (one condition there).
    Under a window (the results, the panel), a name is put away, as the
    town's are.
  - **Headlines**: "Go!" as the first turn begins, with "Aim with the
    stick, set how hard, then Ready" over it; each turn after, "Turn 4",
    and "Last turn!", to whoever is still on the ice; at the end, "Mochi
    wins!", "Mochi & Dubu wins!" for a shared first, or "You win!".
  - **The chat**: a line for each who goes into the sea, as they do, names
    in their colours: "Mochi slid into the sea", "Dubu knocked Mochi into
    the sea" (whoever last hit them, which every device's own play of the
    slide says), "The ice broke under Mochi", "Mochi left the game", and
    "Mochi wins!".
  - **Out**: once your turn's slide is over and where you came is known, a
    panel as Bridge Wars' "You're out!" is: "You're out! 3rd", "Stay and
    watch, or go back to the town. Your place is kept.", **Leave** (to the
    town, your place kept and paid: online the server pays it at the end;
    offline, mine, the rest of the game is played out at once, a day on by
    its clock, which the rules pass through in one step, the place paid,
    and then the town) and **Watch** (it goes, the camera as it is). Not
    once the game is over: its results say it.
  - **The results**: the table's window, "turns survived" by each score.
- **Online and offline it is the same game**: online the server steps it
  by its clock (`server/src/knock_off.rs`), which every device has for the
  town's, and sends the room the round whenever any of it changes (at most
  110 bytes); offline the device steps the very same game by its own time
  of day. **Every body is placed by the device itself** while the game is
  played, people's too (`Frozen { placed: true }`: neither the stick nor the
  server's updates move one), from the round and its own play of the
  slide, so that nothing of where anyone is goes over the network between
  the reveal and the next turn. A device that comes back to the game while
  everyone aims is told its own aim (`Tell::Aimed`) and draws it where it
  left it. Whoever leaves while it is played is out in the turn they leave
  in. With `RT_ALLOW_SKIP`, a skip during play has everyone still on the
  ice ready: the aim is over at once (for trying it out; the board's own
  skips the wait and the countdown).
- **Outside this game's files**, two changes, each to be looked at at the
  merge:
  - `src/tags.rs`, `place_tags`: the town's names are not shown on Knock
    Off's ice (`here != Venue::KnockOff`), whose own are drawn as big
    wherever they are (above). Bridge Wars and the town are as they were.
  - `server/src/tables.rs`, the test
    `someone_alone_plays_knock_off_with_the_computer_from_the_join_to_the_town`:
    it waited for the stand-in's twenty-second game to end; the tables'
    test harness steps a game by the wall clock, which hardly moves in a
    test, so it now has the game say it is over (`Table::finish`, as the
    other tables tests do) where it waited. The game itself is played
    whole through the table's rules by Knock Off's own server tests.
- **Checked**: the rules' tests (155 and 3 ignored; Knock Off's own 34 and 1
  ignored, the stroll's 2 among them), the server's (90 and 4 ignored; Knock Off's own
  5, a table played from "Go!" to its end through the `Rules`, the aims
  secret and a device coming back told its own, a leaver out and placed
  last, a skip and a fall before the game, the computer's players strolling
  and staying on the ice), the game's (376; Knock Off's own 10: the camera
  for every seat on three shapes of screen, who faces which way, a slide
  drawn here the game's and settled onto the server's, the arrows' and the
  cracks' meshes, the buttons clear of each other, where each came, the
  tick, the repeat of a held arrow). **In the desktop build, hidden**
  (`RT_BACKGROUND=1`), offline, by a throwaway taken out again: by day a
  whole game from the join to the town, the arrow at powers 3, 7, 10 and 6
  (the 10 by three real taps on the up arrow, Ready by a real tap), turn
  two's aim by the keys (E held, Space), the reveal, slides with a hit and
  with people going over with a splash, breaks, going out on purpose, the
  panel, Watch by a real tap, the game watched to its end, the winner's
  headline, the results and 950 coins after (4th: the fee, nothing back);
  the computer's players strolling about the ice in the countdown; by
  night the same first turn, out on purpose in the second, the panel, and
  Leave by a real tap, the rest of the game played out at once and the
  place paid (4th: 950) before the town. **Against a server on this PC,
  two hidden copies** (port 5482, `RT_ALLOW_SKIP`): a whole game of two
  people and the computer; while both aimed, each copy's picture had its
  own arrow and nobody else's; at the reveal both had all four; **every
  slide each copy played came out where the server's had, to the bit, on
  both** (logged at every break: the same `start` and `out` on both and
  the same as each copy's own ending); both said the same who went in and
  how; the out panels said 4th and 3rd, the results were the same on both
  with each one's own row picked out, and the coins came out 950 and 960
  (4th and 3rd: 0 and 10 back); afterwards nothing of the table was left on
  either device (no body on the ice, nothing held where the game put it).
  The pictures: `RoundTown-wt/scratch-knockoff/shots/`.
- **Not checked**: anything on a phone, Android or iPhone (nothing behind
  `target_os` was touched, and the iOS type-check was not run); the frame
  rate on a phone; the VM; real accounts or a database; two people
  pressing by hand; a power arrow held down on the screen (taps and a held
  key were); a device coming back to the game in the middle of it (by unit
  tests and the server's own test only); the slide's bit-for-bit agreement
  between the server's x86
  and a phone's ARM (`slide.rs` keeps to what IEEE rounds alike; the test
  pinning it runs on this PC only).
- **Rough, for Hajun to say**: the results' word follows every count, "1
  turns survived" (the table writes one word for any number); names over
  two who stand close overlap, as in Bridge Wars; "Go!" and its line stand
  over the far side's name for their three seconds; a break throws up a
  ring of splashes, some may find it busy.
