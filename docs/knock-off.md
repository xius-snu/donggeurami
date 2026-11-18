# Knock Off

> The full account of Knock Off: what was built, why, in whose words, what
> was tried, and what was and was not checked. `CLAUDE.md` has the rules
> and how things stand now. Dated entries are newest first.

## Contents

1. Later still on 2026-11-18, after a play-test: the camera comes in as the ice breaks, a hit is felt, nothing stands over the play, the ice breaks into pieces, and whoever wins has a moment
2. Later on 2026-11-18, after a check of it: the last person to leave is paid for their place online too, where you came is the game's own word for it, a device that comes back is held ready if it was, and every slide begins on every device the moment it begins
3. Since 2026-11-18 Knock Off is a game people play, online and offline: four on a disc of ice in the sea, everyone aiming at once in secret and sliding at once, the ice breaking a metre a turn

## Later still on 2026-11-18, after a play-test: the camera comes in as the ice breaks, a hit is felt, nothing stands over the play, the ice breaks into pieces, and whoever wins has a moment

A play-tester played the game as the entry below left it, hidden, at a
desktop's size and at a phone's (852 by 393 points, a 59-point camera's
hole each side) by day and by night, and listed sixteen things a player
would find wrong, most of them how it looks: the late fight too small to
see, hits with no impact, headlines and panels over the play. All of them
are mended here, or said below why not. **Every call here is mine** (the
session that mended them; Hajun asked for the game, not for how it looks:
"the details i didn't mention are yours to decide to make each game
polished and fun"). The entries below are corrected where they said
otherwise. Nothing sent changed: no new message, no field, so no
protocol bump (22, not out yet).

- **The camera comes in as the ice breaks** (`camera_target`,
  `view_of`, `Eased`): it had stayed as far out as the whole island all
  game, and by the eighth turn the last two were figures of five pixels
  on a phone. Now, while the edge breaks off, it is brought in over 0.9 s
  (`CAMERA_EASE`, inside the break's 1.2) to fit the ice that will be
  left and 2 m of sea, never less than 3.5 m of ice (`FEWEST`), so that
  the ice keeps its size on the screen and everyone on it grows: 18.7 m up
  at the first turn, 8.0 m by the ninth, the two left there 2.3 times as
  big. It is still while anyone aims or slides, from your own side and as
  steep (55 degrees) as before, the stick's up as before. The island is
  lifted between the plate (`CLEAR_TOP`, 16.5% of the screen) and a band
  kept along the bottom (`CLEAR_BOTTOM`, 10%) for the headlines and the
  panel, the camera looking at a point a little nearer than the middle (it
  had looked at the middle, the island low, with room over it nothing
  used): at the first turn the island is as big as it was. The camera's
  ease and its shake go by the time that passes (`Time<Real>`), not the
  engine's time of the frames, which it holds to a quarter of a second
  each. **Declined**: less sea round the ice on wide phones; the fit is by
  the screen's height on any landscape screen, and the 2 m keep whoever
  goes over seen into the sea.
- **A hit is felt** (`ice::burst`, `ice::shake_for`, `ice::trails`): a
  white flash spreading on the ice, 0.7 to 2.4 m as hard as they met, and
  a spray of motes in the dust's white and the two players' own colours,
  6 to 20 of them (a few grey motes had read as dirt); the camera shakes
  by a share of how far it is from the ice, a fifth of a per cent to a
  little over one (a third of a metre from 30 m off, a dozen pixels on a
  desktop's screen, for the hardest; the few centimetres before were not
  a pixel on a phone), for 0.28 s, and is pushed in toward the ice a
  little with it. Behind whoever slides, a streak in their colour, 0.4 s
  long, narrowing and fading, so that who went where can be followed.
  **Declined**: a pause as two meet. Every device plays the slide by the
  town's clock, to the bit, as the server did; holding the drawing still
  would put it behind and then jump.
- **Nothing stands over the play.** The headlines (`screen::low`) are low
  over the near sea, between the stick and the buttons: "Go!" over its
  how-to, each turn's, "Last turn!", who won (centred, as the town's are,
  they had stood on the far side's heads and names every turn). The panel
  for whoever is out is one row along the bottom ("You're out! 3rd", its
  line, now "Watch the rest, or go back. Your place is kept.", Leave and
  Watch): across the top of the island, where Bridge Wars has it, it had
  stood over the very ice it asks you to watch. The names over the end of
  your arrow go faint while you aim (`NAME_FAINT`, a fifth), the name of
  whoever you aim at had stood on your arrow's head and ring. Names that
  would run into each other are moved apart, the higher one up (they had
  run together, "MaruBbori"); and over your own head a small mark in your
  colour (`chevron`), so that you find yourself after a slide (never your
  name, Hajun, 2026-10-31). The tick and the mark are out of the layout
  while not shown: hidden only, they had stood a name higher, under the
  plate, which put it away.
- **No panel once one or nobody is left**: out second as the ice took the
  third, the panel had come up for a second and gone as who won was said.
- **The buttons on the right look off when they are**: their pictures
  faint too (35%: only the discs had changed, by ten in 255), and the
  whole column (the arrows, the bar, its number, Ready) put away once you
  are out and under the results.
- **The ice breaks into pieces** (`piece_mesh`, `pieces_round`): the band
  that breaks, in pieces of every size, 1.1 to 3.4 m round the rim, their
  broken insides wandering, with a crack between each two; the crack runs
  round the rim both ways from a point thrown each turn, in 0.42 s, and
  each piece tips out and drops as it reaches it, shrinking away under the
  sea (which is see-through). Boxes all alike, all at once, had read as
  the teeth of a cog. A fifth of them splash, small and quick (0.4 s): a
  ring of splashes had stood round the island into the next aim, the
  brightest thing on the screen by night. The pieces go by the town's
  clock, as the break does, and the splashes by the time that passes: in
  a build of a few frames a second they had still been sinking as the
  next aim began.
- **The ice that will break is marked**: the band from the cracks to the
  edge darker, as thin ice is (`colours.knock_off.thin_ice`), while
  everyone aims, looks and slides; a line of cracks alone had not said at
  a glance who stood on it.
- **By night**: the moon on the ice that will stay (`sheen`, as strong as
  the street lamps are on, `Daylight::lamps`), so that everyone on it
  reads as a figure on lit ice and the band that will break stands out
  dark; the splashes and the dust 55% dimmer all night; the second seat's
  blue lighter, `#58a2ff` (it was `#3f8cff`, dark on dark ice).
- **An aim that ends in the sea says so**: past the edge the shaft is in
  dashes, and a cross stands where the ring would (`leaves_the_ice`), on
  your own arrow and on everyone's at the reveal. A slide straight across
  into the sea had been drawn as a safe one, and was taken.
- **Whoever wins has a moment**: the camera comes in to them (5.5 m round,
  `WINNER_VIEW`); they turn to the camera and hop with their arms up (the
  intro's landing pose); a gold ring beats at their feet and every seat's
  colours and gold are thrown over them. Their headline is up 2 s, gone as
  the results come up (it had shown through the results' window).
- **Out, the plate keeps the clock**: "Aiming · 7", "Set...", "Slide!",
  "The ice breaks" (`watch_aim`); where you came is under your name top
  left ("You're out · 3rd", `out_tag`). It had said "You're out · 4th" for
  the rest of the game.
- **The chat**: two who knock each other in are one line, "Gamja and Maru
  knocked each other into the sea" (`knocked_each_other`, said as the
  second goes; two lines blaming each other had read as a quarrel).
- **The reveal says "Set..."** (`set`; it had said "Slide!" with nobody
  moving).
- **The computer's players mind going in more the fewer are left**
  (`computer::STAKES`: going in costs 0.6 to 1.2 times more for each one
  fewer on the ice than four, thrown each turn, as Hajun's rule for them
  has it: "if some thing becomes a definite if-then, then its not a real
  human, but a program"). A final decided by one of them sliding off on
  its own, its target having moved, felt cheap, and a quarter of finals
  were. Over a thousand games of them (seeds 1000 to 1999), with two left
  a turn ends in the sea by one's own move 4.0% of the time (6.4% before),
  222 finals so decided (277); 6.7 turns a game on average (6.0), the
  median still 6; every kind wins 12 to 36%. The whole-games test prints
  the count with two left (`two_left`).
- **Not mended here, not this game's files** (said to the orchestrator):
  the results' "1 turns survived" (the table writes one word for any
  number, `table::score_word`); offline, the board's "Waiting for players
  2/4" while only the computer comes; the selector's card says the fee
  but not the prizes; and offline the computer's players are sometimes
  "Bot", two at a table: the offline table counts the whole town's 32 as
  names taken (`table.rs`, `lobby::ai_member(.., &others)`), and a side's
  names run out. **Left as it is**: the ice's flat look and the facets of
  its outline are Hajun's model's.
- **Checked**: the game's tests, 388 (new: the camera comes in turn by
  turn and to whoever won, who cheers; an arrow into the sea is dashed and
  crossed, and where a slide leaves the ice; the pieces go all the way
  round, of every size, each solid; a streak fades behind; a shake is
  felt; two names are moved apart; your mark; two who knock each other in
  are one line said as the second goes); the rules', 156 and 3 ignored;
  the server's, 92 and 4 ignored. **In the desktop build, hidden**
  (`RT_BACKGROUND=1`), by a throwaway taken out again, from the town's
  gamepad to the town: at a desktop's size by day twice (out in the first
  turn, the panel low, Watch, the camera in turn by turn to the ninth,
  three names apart, the winner, the results with no headline through
  them, 950 after: 4th), at a phone's size with a 59-point hole each side
  by night (the sheen and the band, the burst and the streaks, out third,
  Leave, the rest played at once, 960 after) and by day (watched to the
  end, the ninth turn's two big on a phone, the dashes past the edge at a
  reveal, the winner). **Against a server on this PC, two hidden copies**
  (port 5482): each copy's own arrow only while aiming, all four at the
  reveal, one of them crossed past the edge (and that one slid in alone),
  the same slide on both, both people under the ice as it broke, no panel
  (one left), the same winner's moment and results on both, 35 each, 985
  after. Pictures: `RoundTown-wt/scratch-knockoff/shots/polish_*.png`; the
  runs, `scratch-knockoff/polish/`.
- **Not checked**: anything on a phone (Android or iPhone; nothing behind
  `target_os` was touched); the frame rate on one (each streak's mesh is
  built again each frame while anyone slides, and a break makes 20 to 40
  meshes, gone within it); the shake, which a still picture does not show
  (by its unit test and its numbers); two who knock each other in, in a
  run (by a unit test); night at a desktop's size; the VM; a database.

## Later on 2026-11-18, after a check of it: the last person to leave is paid for their place online too, and eight smaller things

A check of the game as first built (the entry below) found nine things;
all are mended, the first of them a loss of coins. Every call here is mine
(the session that mended them) but where Hajun's words are quoted. The
entry below is corrected where it said otherwise.

- **The last person to leave a table is paid for their place, online
  too.** Out third with two of the computer's players still on the ice,
  Leave (or another game, or a device given up) left the table with
  nobody at it, and the tables close such a table as it is, paying nobody:
  the panel's "Your place is kept" was untrue online (offline, Leave
  played the rest out first and paid). Now, as the last person goes, the
  computer's players play the rest at once (`KnockOff::play_out`: the
  rules step a day on by the clock, which they pass through in one step
  to the end of any game, as the device does offline), and the table is
  told how everyone came before it closes, and pays. **Mine, found while
  mending it**: the same loss came another way, an app put away once you
  are in the sea (a phone closes its connection as its app leaves the
  foreground): the tables give up a table nobody has been connected to for
  a minute, unpaid. With every person at the table in the sea and nobody
  connected for 10 s (`UNWATCHED_PLAY_OUT`), the rest is played at once
  too. With a person still on the ice it plays on as before (they may be
  back, and the minute is theirs).
- **Where you came is the game's own word for it.** "You're out! 3rd" and
  the plate's place were worked out on each device by a rule of its own,
  which could differ from the results' (someone leaving in the slide you
  went in by; a device back in the middle of a game, which has none of the
  slide that put anyone in). The round now carries each seat's place
  (`KnockRound::places`, from `Game::places`, the results' own rule) from
  the moment they go in, and the device shows it once the slide they went
  in by is over (until then, whoever goes in at the very same moment of it
  would share it). The round is 120 bytes at the most now, 80 while
  everyone aims (110 and 76 before); protocol 22 is not out yet, so no bump.
- **A device that comes back while everyone aims is held ready if you had
  pressed Ready** (the round says so; the game hears no more of your aim
  that turn): it had shown "Aim!", a live arrow and a live Ready under a
  tick, and the aims it sent were never played. Begun afresh at the same
  table once out, Watch stays pressed; and a turn's headline is said only
  while it is news, within its own seconds of the turn's beginning, not to
  a device that comes back in the middle of an aim.
- **Every slide begins on every device the moment it begins.** The device
  began it only when the slide's own round came, a server's tick and the
  network after the reveal was over, and everyone jumped forward as it
  began (as far as a metre at the hardest). The reveal's round already
  says all a slide begins from (where everyone stands, how each sets off)
  and when (its end): the slide is begun from it as the town's clock comes
  to that, and the slide's round, once here, is only checked against it
  (begun again only if it says otherwise: someone who left in the
  reveal's last moment).
- **The computer's bodies are not sent while everyone slides.** The server
  wrote them where the game had them every frame of every slide, sent to
  every device, which places every body itself then (`Frozen`) and threw it
  away. Now they are sent where each stands as a turn begins, turns to the
  way it will go, and comes to rest as the ice breaks.
- **A skip in play does nothing** (`RT_ALLOW_SKIP`, the VM's): it had
  every person ready, so that a tap on the board meant to skip the
  countdown that landed as play began ended everyone's first aim. The
  table's skips (the wait, the countdown) are as they were; in play,
  Ready is the game's own skip.
- **A sliding body turns to face the way it slides** (the spec's "turned
  to face the way it slides"): it kept the way it set off, so that one
  knocked sideways or back slid on facing elsewhere. It swings round to
  its way at 12 radians a second (half round in a quarter of a second),
  and keeps facing so as it slows.
- **Words and a colour into the file**: "Mochi & Dubu wins!" for a shared
  first is "Mochi & Dubu win!" (`win_together`, and `won_together` for the
  chat's one line, every name in its colour; `and`, " & ", what joins the
  names), and the slabs' colour until the island's own ice is found
  (`colours.knock_off.slab`, `#d9edff`, as it was in code).
- **"Get ready" is gone**: it was never shown (the plate comes up with the
  first turn's round, which comes with the play's first moment), and over
  the countdown the table's board says "Starting in 3" in the same place.
  **The computer's strolls while a table waits** went by two lines drawn
  on the ice (how far in a stroll may end, 8.5 m; how clear of everyone,
  1.6 m and three quarters of it); each is now thrown for every stroll (7
  to 9.5 m in; 1.2 to 2.2 m clear, settling for 55 to 95% of that), as
  Hajun's rule for the computer's players has it: "if some thing becomes a
  definite if-then, then its not a real human, but a program".
- **Checked**: the rules' tests, 156 and 3 ignored (new: whole games of
  two people and the computer, the people aiming anywhere and leaving at
  any moment, in which a seat's place in the round, once the slide it went
  in by is over, never changes and is the one the game ends with); the
  server's, 92 and 4 ignored (new: out third with two of the computer's
  players on the ice, the last person to leave has the rest played at once
  and is paid 10; every person in the sea and nobody connected 10 s, the
  same, but not with a person still on the ice; a skip in play ends
  nobody's aim; the computer's bodies stay put while everyone slides); the
  game's, 381 (new: the slide begun by the reveal's round and kept by its
  own, a body facing the way it slides and swung round to it the short
  way, Ready held from the round, Watch kept at the same table, the place
  from the round, who won as one or several, every Knock Off word one the
  fonts draw with what it is filled with). **In the desktop build,
  hidden** (`RT_BACKGROUND=1`, by day), by a throwaway taken out again:
  offline, a whole game from the join to the town, out on purpose in the
  second turn: the panel and the plate said 4th, and so did the results; a
  body knocked by the person's slide faced the way it slid (0 degrees off,
  measured 0.3 s after the hit; the picture enlarged). **Against a server
  on this PC, two hidden copies** (port 5482, `RT_ALLOW_SKIP`): a skip
  sent as the first aim began left it on, nobody ready; one copy pressed
  Ready in the first turn and quit, and, opened again, was put back in its
  seat half a second after it started, held ready ("Ready · 6", Ready
  green, the arrows on the right faint, its arrow where it left it, no
  "Go!" said again); on both copies every slide had begun from the
  reveal's round, one to three steps in, before its own round came (42 to
  66 ms into the slide on this PC's network), and was found the same
  slide; after a head-on hit at 18.6 m/s each body sliding faced its own
  way, one knocked back toward its edge (0 degrees off); both went into
  the sea on purpose in the second turn, the panels said 3rd and 4th as
  the server's results did; both pressed Leave, and as the second went,
  two of the computer's players still on the ice, the server played the
  rest at once ("nobody is left to play it for", six turns in a step) and
  paid 10 for third: the first copy's coins went from 950 to 960 in the
  town. Pictures: `RoundTown-wt/scratch-knockoff/shots/fix_*.png`.
- **Not checked**: anything on a phone; the VM; a database (the coins
  were the server's own, in memory); a device begun afresh in the same app
  (rather than opened again), where Watch is kept (unit test only); a
  first shared at the end on the screen (the words by a unit test); the
  play-out of a table nobody is connected to, in a run (unit tests only);
  by night (no colour drawn changed but the slabs' before the island's own
  ice is found, which is at once).

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
  offline: a stroll ends 7 to 9.5 m of the middle at the furthest, 1.2 to
  5 m off, at 1.4 to 2.8 m/s; it stands 1 to 4.5 s first; then strolls,
  looks round at someone or at the middle, or stands on, each thrown
  afresh, as Hajun's rule for the computer's players has it: "if some
  thing becomes a definite if-then, then its not a real human, but a
  program"; how far in and how clear were fixed until the entry above).
  **At "Go!"**
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
  (Since the entry at the top it comes in as the ice breaks, the island
  lifted between the plate and a band along the bottom, and shakes by a
  share of how far off it is.)
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
    outward and drops into the sea, some with a splash. (Since the entry at
    the top, pieces of every size, going in a run round the rim.)
  - **A splash** where anyone goes into the sea, the moment they reach the
    water (a ring spreading 2.3 m on it and a dozen drops thrown up), a
    smaller one where a slab does. Whoever goes over slides on off the
    edge, drops in, and is gone once under (`Dead`, drawn nowhere).
  - **A puff of ice dust** where two meet hard enough to count as a hit
    (since the entry at the top, a burst, and a streak behind whoever
    slides).
  - **Sliding, a body is posed for it**: knees half bent, arms out to the
    sides (the intro's landing pose, `Landing`), turned to face the way it
    slides (since the entry above; it kept the way it set off before);
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
    breaks"; "Last turn" on the last; out, "You're out · 3rd" (since the
    entry at the top, out, how it is going, "Aiming · 7", and where you
    came under your name). (Over the countdown the table's board is there,
    "Starting in 3".)
  - **Names over heads**, each in its seat's colour with a dark stroke,
    **as big wherever they are**, as Bridge Wars' are, and a green tick over
    whoever is ready; never your name over you (Hajun, 2026-10-31), but
    your tick, once you are ready. **Why not the town's names** (`tags`):
    sized by how far off a head is, from a camera far enough out to see
    the whole island they were a centimetre high on a phone, white on white
    ice. `tags` leaves Knock Off's ice to these (one condition there).
    Under a window (the results, the panel), a name is put away, as the
    town's are.
  - **Headlines** (low over the near sea since the entry at the top):
    "Go!" as the first turn begins, with "Aim with the stick, set how
    hard, then Ready" over it; each turn after, "Turn 4",
    and "Last turn!", to whoever is still on the ice; at the end, "Mochi
    wins!", "Mochi & Dubu win!" for a shared first, or "You win!".
  - **The chat**: a line for each who goes into the sea, as they do, names
    in their colours: "Mochi slid into the sea", "Dubu knocked Mochi into
    the sea" (whoever last hit them, which every device's own play of the
    slide says), "The ice broke under Mochi", "Mochi left the game", and
    "Mochi wins!" (or "Mochi & Dubu win!").
  - **Out** (along the bottom of the screen since the entry at the top,
    and none once one or nobody is left): once your turn's slide is over,
    a panel as Bridge Wars' "You're out!" is: "You're out! 3rd" (the place the server sends,
    `KnockRound::places`, the results' own), "Stay and watch, or go back
    to the town. Your place is kept.", **Leave** (to the town, your place
    kept and paid: offline, mine, the rest of the game is played out at
    once, a day on by its clock, which the rules pass through in one step,
    the place paid, and then the town; online the server plays on, and
    pays at the end, or, as the last person goes, plays the rest at once
    and pays) and **Watch** (it goes, the camera as it is). Not once the
    game is over: its results say it.
  - **The results**: the table's window, "turns survived" by each score.
- **Online and offline it is the same game**: online the server steps it
  by its clock (`server/src/knock_off.rs`), which every device has for the
  town's, and sends the room the round whenever any of it changes (at most
  120 bytes); offline the device steps the very same game by its own time
  of day. **Every body is placed by the device itself** while the game is
  played, people's too (`Frozen { placed: true }`: neither the stick nor the
  server's updates move one), from the round and its own play of the
  slide, begun the moment the reveal is over, so that nothing of where
  anyone is goes over the network while everyone slides (the computer's
  bodies are sent where each stands between slides). A device that comes
  back to the game while everyone aims is told its own aim (`Tell::Aimed`)
  and draws it where it left it, held ready if you had said so. Whoever
  leaves while it is played is out in the turn they leave in. With
  `RT_ALLOW_SKIP` the board's skip skips the wait and the countdown; in
  play a skip does nothing (Ready is the game's own).
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
  ring of splashes, some may find it busy. (All but the first mended in
  the entry at the top.)
