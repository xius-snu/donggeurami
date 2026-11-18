# Last Stand

> The full account of Last Stand, the co-op game of endless waves round an
> anchor: what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. `CLAUDE.md` has the
> rules and how things stand now. The tables every one of the four games of
> 2026-11-18 is played at (the join, the fee, the board, the results) are
> not written up here: they are `server/src/tables.rs`, `server/src/fees.rs`
> and `src/table.rs`, and have an account of their own.

## Contents

1. At the merge, 2026-11-18: the headlines stroked
2. 2026-11-18, later still: what a play-tester found, made clearer
3. 2026-11-18, later: sixteen things a check of it found, mended
4. Since 2026-11-18 Last Stand is a game people play, online and offline

## At the merge, 2026-11-18: the headlines stroked

A play-through of the merged build (`docs/minigames.md`, "At the merge")
found "The anchor is under attack!", the line this game most needs read,
gold on the sand by day, and "Wave 3 cleared" the same. Every headline of
this game is now said with a thin dark stroke round its words
(`hud::announce_stroked`: four copies under them in `colours.chat.stroke`,
faded in and out with them, a test of it in `hud`), the same lines in the
same places (mine; the town's and Bridge Wars' headlines are as they were).
Seen hidden afterwards, enlarged: "Go!" at the start of a game, by day.

## 2026-11-18, later still: what a play-tester found, made clearer

A play-tester played it through four times, hidden, from the town's
gamepad to the results and back (by day and by night, at a desktop's size
and a phone's), and listed what a player would find wrong, the most
important first. Their three biggest: nothing said the anchor was being
hit, or where it was; a shop's window came down over the fight; by night
the zombies were black blobs. Every call here is mine unless it is
quoted; Hajun has seen none of it, nor has any phone.

**The anchor, the game's whole point:**

1. **Its bar on the plate is thick, with its number in it, and flashes
   red as it is hit** ("517/1500"; 30 by 2.8 of the screen's short side
   where it was 24 by 1.1, a hairline nobody saw go down; the red,
   `colours.last_stand.anchor_hit`, comes and goes with the crystal's own
   white flash, no oftener than every 0.35 s).
2. **An arrow round you points the way to the crystal while it is off the
   screen**, in its purple, flashing red with it as it is hit: the
   Tracker's arrow in Bridge Wars (`compass`, Hajun's "a small white arrow
   around your character"), drawn as it is, eleven hundredths of the
   short side out from you (`looks::toward_the_anchor`). Not while
   something is said across the middle of the screen, into whose words it
   stood pointing up ("Brutes!"), nor while a shop's window is up. Off the
   screen means its foot is not on it, the part of it drawn nearest the
   middle of the screen from straight over it (by its middle, and then by
   its middle 4 hundredths in from the edge, the arrow came up with the
   crystal and its bar still at the foot of the screen).
3. **"The anchor is under attack!"** high on the screen, in the gold of
   what is said across it, as the anchor loses health while the crystal
   is off the screen, no oftener than every 8 s of the fight (Bridge
   Wars' own while for its own anchor), and never over anything else said
   across the screen (`screen::warn`).
4. **Breaking off the screen, the camera comes to look**: it eases over
   to the crystal as it shatters and stays 2.2 s (`looks::SHATTER_LOOK`),
   then comes back to you, so that the shatter is seen; the play-tester's
   broke out of sight, and "The last stand fell" was said over empty
   stone. Not while a shop's window is up over you.
5. **"The anchor is broken!" is said across the screen in the gold of
   everything else said there**, with "No more coming back" under it
   (the file's words split in two); it was Bridge Wars' line for its
   timetable (`arena::alert`), white, which on the pale stone by day
   hardly read, and it is the game's most important line.

**The shops:**

6. **In the middle of a wave a shop's window comes down only once you
   have stood in its outline half a second without swinging or being
   hit** (`ShopDoor`, `SHOP_SETTLE` 0.5 s; between waves, at once, as
   before; once down it stays while you stay). Fighting round the crystal
   in the eighth wave, the play-tester stepped into the item shop's
   outline 4.8 m from it and had the window come down over the middle of
   the screen, every zombie round them under it. Done with the kit's own
   rule, that a shop on another game's map opens only where its game hands
   it cards (`market::Stalls`): until the door is open the game hands it
   none (`play::publish`); `market` is as it was.
7. **Nothing this game says across the screen shows through a shop's
   window**: it is put away while the window is up, and goes on fading
   meanwhile ("Go! Hold the anchor" stood between the sword cards).
8. **An endless upgrade's card says "Next: Lvl 3"** (a bare "Lvl 3" read
   as the level the team had; the capped two keep their pips), and so
   does the generators' Overclock card.

**By night:**

9. **Every zombie's head glows faintly at night in its kind's tint**,
   brought up to one brightness for every kind (`looks::NIGHT_GLOW` 0.16
   of its brightest part, about a tenth of how it is drawn by day, as the
   street lamps come on, `sky::Daylight::lamps`; none by day). Away from
   the crystal's light they were black discs, the giant too, told apart by
   their size alone. Not a light over the sand: `lamps` is not this
   game's, and every light costs the phone.
10. **A brute's and a giant's bar is as much bigger as they are, and over
    their head** (a third and twice as wide, a sixth and half again as
    thick), by a hook added to the kit's tags (`arena::Bigger`, carried
    by no body of Bridge Wars'): a giant's bar was a body's, across its
    head.

**What piled up, and what said too much:**

11. **The crystal's bar and the pads' words are put away while anything
    is said across the middle of the screen** ("——Go!", the bar run into
    it; "anchoSilver"), as they are under the plate and a shop's window.
12. **"Back in 6" stands low on the screen** (70% down, `BACK_DOWN`), the
    kit's own words moved there in this game alone: at its 38% it stood
    on the crystal and its numbers, the camera now looking at the
    crystal while you are dead (below); **in whole seconds** ("Back in
    5.5" read fussy).
13. **Numbers off a body are a little way off it, thrown, and its blows
    no oftener than every 0.3 s are said together** (`Tally`, `spread`;
    from straight overhead a number rises toward the camera and hardly
    moves on the screen, and "6" stood over "7", "134 135 51 28 38 135"
    in one heap); **the crystal's beside it**, to its left or right, never
    over its purple, where its red read badly. The kit's numbers have no
    stroke, as Bridge Wars' have none; left as they are (below).
14. **The results come up with nothing said across the screen over them**:
    "The last stand fell at wave 4" (up 4 s, the results 2 s after the
    end) showed through the window's title.
15. **The chat no longer says your own deaths** ("Mochi is down" three
    times pushed what the others did out of its three lines; the screen
    says yours), and **no longer says a wave cleared**, which the
    headline and the plate say at the same moment (`CLEARED_IN_CHAT`, off;
    the line is kept in the file).
16. **"You're out" is a slim window along the foot of the screen**,
    between the stick and the buttons round the jump, its words beside
    its Leave; half way down the screen it stood on the crystal's site and
    the horde round it, which it tells you to watch.

**The camera, and teaching a first game:**

17. **Dead, the camera eases to the anchor**, where you will stand again
    (it stayed six seconds on the stone where you fell, and then jumped);
    out, to whoever still stands, as before, now eased; at the end, to the
    anchor's place, where the last stand fell (`screen::watch_view`, half
    the way in about a ninth of a second, nearly all of it in half a
    second).
18. **The line under the plate teaches and warns**: in the break before
    the first wave "Red stand: items · Blue stand: team upgrades"; in a
    break before a wave with something new, "Next: A giant!" (to buy for
    it); in the last five seconds of any break, "Get ready"; otherwise the
    wave cleared, and, once anyone is ready, how many are.
19. **Over the pads, everything they make**: "Lvl 1: silver, gold", "Lvl
    2: silver, gold, sapphire", "Lvl 3: all four" ("Lvl 1: gold" read as
    the gold alone).

**Left as they were, and why:**

- **The palms' crowns hiding whoever is under them**: Hajun's rule is that
  nothing fades out of the way anywhere but inside a house (2026-10-25,
  `see_through`: "only happen when the player is inside the home ... not
  anywhere else"), and the palms are his map's. To ask him: move or shrink
  the palms nearest the middle of `laststandmap`.
- **A stroke round the damage numbers**: the numbers are the kit's,
  Bridge Wars' too, and drawn as they are there; spread and kept off the
  crystal and the blast, they read.
- **A light over the sand by night**: what gives off light is `lamps`'
  list, not this game's, and the phone pays for every light (`MOST_LIT`).
- **Not this game's to mend**: on a phone the shop's window reaches over
  the fireball's button on the right, and its cards' lines are about 7 pt
  (`market`'s sizes); offline alone the board says "Waiting for players
  1/4", and the results call a lone player "1st" (the tables').

**Outside this game's files**: one hook in the kit's `src/arena.rs`,
`Bigger` (a component, read where a tag is made and placed; a body without
it, every one of Bridge Wars', is tagged exactly as before).

### Checked, and where

- **Tests**, through the wrapper, on the tree as committed: the game's
  (`--lib`) 392 (384 before, eight new), the rules' (`-p roundtown_net`)
  155 and 5 ignored, the server's (`-p roundtown_server`) 87 and 4
  ignored, the rules and the server untouched; the desktop game builds
  with the five warnings it had (`lib.rs` twice, `island.rs`, `lobby.rs`,
  `signup.rs`), the server with none. New here: the shop's
  door (at once between waves, in a wave only after half a second
  unstirred, never walking through), the tally of blows and the spread,
  the line under the plate (where to buy what, "Next: A giant!", "Get
  ready", how many are ready), the anchor's number, the chat never saying
  your own deaths, "Back in" in whole seconds, what the camera looks on at,
  the end's words gone under the results, the pads' words, the cards'
  "Next: Lvl", the bigger bars and the night's glow, the arrow's place.
- **Hidden, offline, by day, at a desktop's size, from the town's gamepad
  through four waves to the results and back** (the play-tester's own
  throwaway, with a deliberate visit to the item shop in the middle of the
  second wave, and left at the rim from the fourth): the plate's bar and
  its number, flashing; "Red stand: items · Blue stand: team upgrades"
  under the plate; "Go!" with no bar or "Silver" in it; the arrow round
  you, and "The anchor is under attack!" off the screen; the camera
  easing to the anchor while dead, "Back in 6" low; the cards' "Next: Lvl
  1"; the mid-wave shop: no window 0.28 s in, the window at 0.51 s; the
  crystal breaking off the screen at the rim: the camera over it, the
  shards, "The anchor is broken!" over "No more coming back"; the end
  over the anchor's place; the results with nothing over them; 1,000
  coins to 980 (three waves cleared).
- **Hidden, offline, by night, at a desktop's size, from wave 5 with 40
  of each** (`RT_STAND_WAVE=5 RT_STAND_PURSE=40`): every kind's head
  seen on the dark sand away from the crystal's light, a giant's pale
  green and twice as broad, its bar twice as wide (enlarged); a brute's
  bar a third wider; numbers spread; "Next: A giant!"; fighting in the
  upgrade shop's outline in the sixth wave, no window; the spit; the
  anchor broken; 1,000 to 1,020 (seven cleared).
- **Hidden, offline, by night, at a phone's size (852 by 393) with a 59
  pt camera's hole each side, from wave 15** ("2 giants!"): the arrow,
  "Back in 6" low, fighting in the upgrade shop's outline with no window,
  the end over the anchor's place, the results; 1,000 to 1,090.
- **Two hidden copies against a server on this PC** (port 5484, from wave
  8 with 200 of each), twice: Dubu fallen into the void, on Dubu's screen
  the camera over the anchor and "Back in 5" low, and in Dubu's chat no
  "Dubu is down", in Mochi's "Dubu is down"; numbers off each other and a
  potion's "+36"; the connection lost in the middle of the wave (one
  number floated in the 2.5 s it was gone: a blow the round had said just
  before, its number held its 0.3 s); the anchor broken; once with Dubu
  invisible at the edge and Mochi gone into the void: Mochi out, the slim
  window along the foot with Leave beside its words, the camera on Dubu;
  "Mochi is out" in Dubu's chat and not in Mochi's; both paid 70; back in
  the town, nothing of the game on either body.
- **Not**: a phone or an iPhone (nothing here is behind `target_os`); the
  VM; Play again by day; a group led from the results into another game;
  the shatter's look and the line under attack online (the crystal was on
  Dubu's screen at the edge as it broke); the frame rate (the hidden runs
  drew 57 to 60 frames a second, a debug build on this PC).
- The runs' pictures are in `RoundTown-wt/scratch-stand/polish/<run>/shots/`
  (`desk_day`, `desk_night`, `phone_night`, `online_day`, `online_out`),
  crops beside them; the throwaways in `scratch-stand/polish/kept/`.

## 2026-11-18, later: sixteen things a check of it found, mended

A check of the game as first built (the entry below) listed sixteen
things; every one is mended or answered here, the same day. Nothing of it
has been seen on a phone.

1. **Bridge Wars' "You died. Respawning in 4..." showed every other
   frame.** Last Stand's own words while you are dead ("Back in 6") ran in
   every game and took down whatever death words were up whenever it had
   none of its own to say: in Bridge Wars, Bridge Wars' (`arena::show_death`
   put them back the next frame). Now they run only in a game of Last
   Stand, and take its words down once as it goes, as Bridge Wars' own do
   (`screen::death_words`).
2. **The crystal stayed shattered in every game after the first** when one
   table followed another (Play again, or a group's leader taking it on):
   it was stood up again only on a frame with no game of it, which a table
   followed by the next never has, and every game ends with it broken. The
   next game had no crystal to see, none to stand on or bump into, no
   light from it at night, no flash and no shatter, its bar over nothing.
   It is made whole as every game begins now (`looks::anchor`, by the
   game's own number).
3. **Online, a teammate dead stood where they fell**, their name over an
   empty bar, for the six seconds and, once out, to the end. Their body is
   dead while the round says so (`crate::Dead`), which hides it and its
   name, as Bridge Wars hides its dead (`play::publish`).
4. **A game of it ends before Bridge Wars' or House Builder's systems
   run** (`play::enter`, before `arena::ArenaSystems` and
   `builder::GameSystems`): someone led by their group from its results
   straight into one of those is told the table is gone and seated in the
   next game in one frame, and its end, run after the next game's
   beginning, would have taken that game's team, health, board, bars and
   opening bridge off them. Bridge Race's systems run in Bridge Wars' set,
   and so come after it too. Checked by the order the schedule is given
   alone, and by the game starting with it (a cycle would stop it); the
   case itself was not played.
5. **"Mochi brought down a giant" is said offline too.** Which kills the
   chat says is one rule in the shared game now (`Kind::announced`: a
   giant alone), which the server tells the room by and a device playing
   offline says its own game's kills by (`play::happened_of`).
6. **Teammates' bodies keep nothing of the game after it.** Its end takes
   the team, the health and the death off every body it put them on
   (`play::Teammate`), not only yours: a teammate still in sight in the
   town kept a bar over their head there, and one dead as the game ended
   stood there unseen.
7. **R is not Ready while a line is typed** in the chat, or any dialog is
   up (`hud::DialogUp`): typing "ready" in a break said you were.
8. **Numbers off people and off the crystal.** A blow or a spit on you or
   a teammate floats what it took, and a healing potion what it gave, in
   the heal's green, as Bridge Wars floats them (mine: a rise of three
   points or more is a heal, so that what comes back by itself, a point
   at a time, and all of it as someone stands again float nothing). The
   crystal floats what the blows on it took, each number with its flash,
   no oftener than every 0.35 s and saying all it took since the last
   (mine: a heap of zombies clawing at it would float a number a frame),
   the blow that breaks it at once; and a Fortify floats what it gave back.
9. **The connection lost no longer floats a number off every zombie.**
   `net` takes away everything the server sent as the connection goes, and
   every zombie's body going was taken for a kill: online, one is taken
   for a kill only while connected now (`looks::last_blow`).
10. **Teammates' names are over them in the waiting room**: they are given
    their health from the table's start (full until there is a round),
    the kit tagging only a body with health (mine: a full bar under each
    name there, rather than a name alone, which the kit would not give a
    bar once the play began).
11. **Someone who leaves is said to have left** ("Dubu has left", a new
    line in the file, `words.last_stand.left_line`), never "is down" or
    "is out": the round says who has gone, a bit to a seat
    (`StandRound.gone`, mine, one byte; within protocol 22, which no
    release has yet). Nothing is said of it once the game is over, as
    everyone goes then.
12. **On desktop, the spit's ball, the crystal's bar and the words over the
    pads take the file's colours as it changes** (`looks::recolour`), as
    the plate always did; and the shops' cards are made again then too.
13. **`StandRound.began` is not sent once**, as this account said: it goes
    with every send of the round, eight bytes a time, a few times a second
    in a wave. Said so now, here and in the code. Kept where it is, mine:
    a component of its own, sent once, would be one more registration for
    eight bytes a send.
14. **Offline, a stick of dynamite the game has gone into the void no
    longer lies to the end** where this device's map has ground and the
    rules' has none (`play::sink`); not seen happen, the two maps agreeing
    everywhere the hidden runs threw one.
15. **This account holds every rule and number, and whose call each was**
    (the entry below, "The game, as the rules have it"): until now some
    were only in the rules' tuning report, which is not in the repository.
    The Leap potion is said there to be a departure from the design's list.
16. **What was not checked before** is, below: the night's waiting room,
    item shop, brutes and spitters; the online runs on the final build;
    Play again; dying in Bridge Wars with this build; a connection lost
    mid-wave.

**Two more, found in the hidden runs below** (both mine):

- With a shop's window open, the pads' words showed through its
  see-through dark among the cards' words ("Silver" by "Heal" and by
  "Leap", by night): they and the crystal's bar are put away while you
  stand in a shop's outline, as the line under the plate always was
  (`looks::in_sight`).
- Where the screen put a pad, or the crystal, by the column top left, its
  words or its bar were drawn into the column's rows ("Silver" over the
  count of gold): put away where they would stand among them, the column
  read where the screen lays it out (`looks::Column`), as they are under
  the plate. Bridge Wars draws its names and its generators' words on
  under its own column; that is left as it is.

### Checked, and where

- **Tests**, through the wrapper: the game's (`--lib`) 384, eight new (the
  crystal whole in the game after one that broke it; Bridge Wars' death
  words left alone with no game of this here; R not Ready while a line is
  typed; who has left said so, never down or out; a giant brought down
  said offline by the shared rule; a potion's heal told from what comes
  back by itself; a game's end taking its marks off a teammate's body;
  the file changed drawing the spit, the bar and the pads' words again),
  the first two seen to fail on the code before; the rules' 155 and 5
  ignored (two new: who has gone is in the round, and a giant alone is
  said); the server's 87 and 4 ignored. The desktop game builds with the
  five warnings it had before (`lib.rs` twice, `island.rs`, `lobby.rs`,
  `signup.rs`), the server with none.
- **Hidden, offline, by night** (`RT_STAND_WAVE=8 RT_STAND_PURSE=200`, a
  throwaway taken out again): the waiting room; "Go!"; the item shop open,
  every card read and nothing through it; a healing potion bought; numbers
  off you ("10") and the potion's "+50"; numbers off the crystal ("4", "6"
  over its purple); spitters and their balls, a brute; the end; the
  results; and **Play again**: the next game's crystal standing, under its
  bar, the map underfoot whole again (10,284 triangles, where 10,276 were
  left once it shattered), and walked at, you stopped 1.12 m from its
  middle; then leaving.
- **Hidden, offline, in Bridge Wars**: fallen into the void, "You died.
  Respawning in 3..." up in every frame looked at, 12 of 12.
- **Two hidden copies against a server on this PC** (port 5484, Mochi and
  Dubu, from wave 8 with full purses), twice:
  - each other's names and bars over them in the waiting room;
  - Dubu fallen into the void: on Mochi's screen, gone with their name,
    "Dubu is down", and back six seconds later; numbers off each other on
    both screens, and off the crystal;
  - Mochi's connection lost in the middle of the wave, fourteen zombies on
    the sand: "Reconnecting...", no number floated off any of them (none
    in 2.5 s), and back in the game, played to the end;
  - the end: both paid 70, "Waves cleared: 7 · Team score: 700"; back in
    the town, the other's body seen, with no team, health or death of the
    game left on it;
  - the second run: Mochi's potion, "+36" over Mochi on both screens; and
    Dubu leaving in the middle of the wave at full health: on Mochi's
    screen "Dubu has left" (it said "Dubu is down" before).
  The first run was on the build before the column's check above; the
  second on the final one.
- **Not checked**: a phone or an iPhone (nothing is behind `target_os =
  "ios"`; the iOS type-check was not run); the VM; someone led by their
  group from its results into Bridge Wars or House Builder (the order is
  the schedule's alone); a stick of dynamite going into the void where the
  two maps disagree (no such place was met); the file changed under a
  running game (a test only); Play again by day.
- Two of these hidden runs lost the GPU (`DeviceLost`), with other
  copies of the game drawing on the PC at the time: the first night run,
  after its first game, and the first Bridge Wars run, at once; both were
  run again and went through. The pictures are in
  `RoundTown-wt/scratch-stand/shots/fix/`, the runs' logs in
  `RoundTown-wt/scratch-stand/fix2/out/`, and the throwaway that drove
  them in `RoundTown-wt/scratch-stand/fix2/kept/`.

## Since 2026-11-18 Last Stand is a game people play, online and offline

Hajun, 2026-11-18, of four new games: "Coop (2~4 players) Last Stand
(Endless Zombie survival game). You are in a circular map, with a spawn
anchor crystal in the center, and 2 generators and item shops/upgrade
shops. Zombies with increasing difficulty/skill/creativity/health/
attackdmg come in waves from the outer edges toward the anchor crystal in
the middle and/or player. You can upgrade generators progressively, at
first u just start with silver and u gradually also get the other 3
minerals (gold, sapphire, amethyst). And you get the loot from both
generators and killing zombies (random chance droprate between silver gold
sapphire amethyst that increases its luck as progress goes). You can buy
items from item shop and also buy upgrades from upgrade shop. I haven't
rly thought of what the items should be or what the upgrades should be,
but they should make sense and allow fun for long game so it makes sense.
I.e. Endless upgrades, endless waves of endlessly stronger and stronger
enemies. You get a numerical score at the end (individual scores of each
person + total combined team score). Map = laststandmap (similar cam and
controls with bridgewars)"; and of all four: "the details i didn't mention
are yours to decide to make each game polished and fun", "each of these
games have a coin entrance fee and payout. Generally should be above 50%
to win net positive coin". Every number below not quoted from Hajun is
ours, and said so.

**`PROTOCOL_VERSION` 22**, with the other three games (bumped once for all
four). Not deployed, no APK, no iOS tag: that is the orchestrator's, with
the four games merged.

Files: `shared/src/last_stand.rs` and `shared/src/last_stand/` (the rules,
`Game`, a plain struct both sides run); `server/src/last_stand.rs` (the
server's wrapper, `tables::Rules`); `src/last_stand.rs` and
`src/last_stand/{play,looks,screen}.rs` (the device's); its words and
colours in `assets/ui.ron` (`words.last_stand`, `colours.last_stand`, with
`src/ui.rs`); three pictures for the upgrade shop's cards in `src/hud.rs`
(`Picture::Crystal`, `Beam`, `Clover`).

### The game, as the rules have it

The rules were written and tuned first, by whole games of stand-in players
(`cargo test -p roundtown_net stand_many -- --ignored --nocapture` prints
the table, `stand_waves` the pressure wave by wave; the module's tests hold
the targets): a plain player, landing a blow a second on what is in front
of it, alone clears six or seven waves in the middle of its games, in
twelve minutes, and has the fee back (five waves) in 85 games of 100
(Hajun's "above 50%"); two and three together clear ten, four eleven, in
seventeen to nineteen minutes. A good player (three blows a second,
potions, dodging) clears nine alone and thirteen with four. A lone
player's results have two humps, the first giant (wave 5) and the walkers
needing a third blow (wave 10) being the walls; every team meets the
second about the tenth wave. A team of three or four good players plays
past twenty minutes; the plain one, which the game is tuned by, within
them.

**Hajun's**: co-op; endless waves of zombies stronger every wave coming
from the edges for the anchor or the players; the anchor crystal in the
middle; two generators, silver at first and then gold, sapphire and
amethyst as the team upgrades them; loot from the zombies, by a chance
that grows luckier as the game goes; an item shop and an upgrade shop,
upgrades without end; each person's score and the team's at the end;
`laststandmap`; Bridge Wars' camera and controls; "2~4" players; a fee
and a payout, "above 50%" to come out ahead. **The design's** is the brief
written for the four games the same day (ours, `specs/design-games.md`);
**mine** is this game's own builders' and tuners'. Every number:

- **The table**: people only, never the computer; one to four (mine: alone
  too, so that nobody waits for a second person who is not coming; Hajun
  said "2~4"); it begins 15 s after the last person joined, or at once
  with four; a 5 s countdown; the results 20 s (the design's); 50 coins to
  join; every person paid 10 coins a wave the team cleared, up to 400
  (the design's: five waves give the fee back).
- **The map** (`laststandmap.glb`, stood 0.25 m up as Bridge Wars' is,
  `LIFT`): a stone disc 50 m across in a ring of sand 75 m across, over the
  void (the design's, so that a zombie or you can be knocked off); the
  anchor's crystal on its plinth in the middle, the generators' pads
  either side, the item shop's red stand and the upgrade shop's blue one
  in front and behind. A test holds the rules' numbers to the map
  (`the_rules_numbers_for_the_map_are_hajun_s_map_s`). Everyone begins
  3.2 m from the anchor on the diagonals, facing out (mine).
- **The anchor**: 1500 health (mine; the design's 1000 fell to the first
  brutes); while it stands, whoever dies is back beside it 6 s later (the
  design's), with 2 s in which no zombie turns on them (mine); broken, the
  dead stay dead; the game is over once it is broken and nobody stands,
  or everyone has gone (the design's).
- **Waves**: the first 20 s after "Go!", then 25 s between waves to shop
  (the design's); once everyone standing has said they are ready, the next
  in 3 s (mine, `Ask::Ready`). Wave n sends (6 + 3n + n^2/4) x (0.75 +
  0.25 a person), rounded (the formula the design's; the share by people
  mine, where the design's 0.6 + 0.4 a person had four at one anchor no
  further on than one): 9 in the first for one and 16 for four, 61 in the
  tenth for one and 107 for four. They come over the rim (0.22 + 0.1 a
  person) a second in the first wave and 12% more each wave after (mine),
  from two places round the rim, up to four as the waves go on (2 + n/4 at
  most), in bursts of two to four, each place well apart from the rest
  (mine); at most 40 standing at once (the design's).
- **The zombies**, the wave each first comes in, then their health, blow
  and pace in the first wave of all (the paces the design's; the healths
  and blows mine, half the design's or less at first and growing faster):
  - walker, wave 1: 20, 6, 3.2 m/s (design 40 and 8);
  - runner, wave 3: 12, 5, 6 (design 25 and 6);
  - brute, wave 4 (mine; the design's 5th, so that each early wave brings
    one thing new and the fifth is the giant's alone): 48, 14, 2.4
    (design 160 and 22); a third bigger, takes a quarter of a knock,
    knocks you as a punch does, and its blow on the crystal counts three
    quarters (the design's "hits the anchor hard"), every other kind's a
    half (mine);
  - giant, every fifth wave: 450, 28, 2 (design 1200 and 40); twice as
    big, knocked by nothing, knocks you as a running punch does, always
    for the anchor (mine), drops a hoard of 12; one in the 5th and 10th
    waves, two in the 15th and 20th, one more every ten waves (mine; the
    design's one more every five ended half of a lone player's games at
    the tenth); its health 0.5 + 0.125 a person times its own (mine: the
    rest of a wave grows with the people in its numbers);
  - spitter, wave 8: 18, 10 (its ball's), 3 (design 35 and 12); keeps 7
    to 9 m off (mine, thrown for each) and spits a slow green ball that
    splashes whoever is within 1.3 m of where it comes down, and the
    crystal if by it;
  - leaper, wave 12: 25, 12, 4 (design 50 and 14); leaps at whoever is
    from 3.5-5 to 7.5-9.5 m off (mine, thrown for each), knocking them on
    the way it came.
  - Every wave past the first, health x1.22 and blows x1.09 (mine; the
    design's 1.12 and 1.07), pace +1% to at most +30% (the design's). A
    spitter's health comes to the design's in the 9th wave, a walker's, a
    runner's and a leaper's in the 10th, a giant's in the 13th, a brute's
    in the 16th; a leaper's blow in the 10th, a runner's and a spitter's
    in the 11th, a walker's in the 17th, a giant's in the 21st, a brute's
    in the 26th (a test holds these).
  - What each goes for, thrown as it rises (mine): the anchor, or whoever
    is nearest; a giant always the anchor, a brute 85% of the time, a
    runner 35%, the rest 80% in the first wave and 3.5% less each wave to
    35%. Each turns on whoever hits it. Some come round on an arc of their
    own (0.35 to 0.9 of a radian off), 15% in the first wave and 2% more a
    wave, to half (mine). Each turns on someone within 2 to 4.5 m (times
    the root of its size) and gives them up 11 to 22 m off (mine, thrown
    for each). At the crystal, each holds to it within 2 to 4 m past its
    reach and turns on someone near only by a chance of none to 6% each
    time it looks round (mine, thrown for each: every one turning would
    draw the whole heap off the anchor, none would make standing by the
    crystal safe). A wave's last 1.5 to 5 come on 1.4 times as fast
    (mine). No zombie walks at or hits anyone unseen, or in their two
    seconds back. The small kinds' blows knock nobody (mine: a crowd of
    them would bat whoever holds the anchor about the stone).
- **You**: 100 health, coming back 4 a second from 5 s after the last hit
  (mine; the design's "back slowly 5 s after"); fists 6, the swords 14, 22
  and 34, the shields 10%, 20% and 30% off every blow (the design's); a
  blow counts on a zombie within 6.5 m of where your device last said you
  were (and its own size), once in 0.5 s from each of you (mine, Bridge
  Wars' `HURT_TIME`). Dying, you keep your purse and everything you bought
  (mine: it is played together, and a death costs the wait); what you had
  drunk is over.
- **The generators**: silver from the start, one about every 2 s on each
  pad (the design's); gold, sapphire and amethyst about every 8, 30 and 60
  s, each give or take 1.5, 4 and 8 (silver 0.5), at most 24, 6, 3 and 2
  lying on a pad (mine); all of it faster, and as many more lying, with
  more people: 0.6 + 0.4 a person times (mine: four living off what one
  would have were each a quarter as well armed); each Overclock step
  x1.25 (the design's).
- **What zombies drop**: one thing a third of the time, a tenth more for
  each step of Lucky Loot, at most 95% (the design's third and tenth); a
  brute two or three, a giant twelve; each thing silver, gold, sapphire or
  amethyst by weights 100, 10 + 4n, 2n from the third wave, n - 4 from the
  fifth (the design's), three waves further on for each step of luck
  (mine). A zombie killed climbing drops on the sand over it; one fallen
  more than a step below the sand, nothing (mine).
- **The item shop** (each person's own, kept when they die): Basic Sword
  12 silver, Silver Sword 8 gold, Aura Sword 6 sapphire; Basic Shield 20
  silver, Advanced 10 gold, Aura 5 sapphire; Healing Potion 4 gold (half
  your health), Speed Potion 1 sapphire (30 s, +40%), Fireball 30 silver
  (40 in its blast), Dynamite 1 amethyst (120 in its blast, and a big
  knock), Invisibility 2 sapphire (20 s, the zombies lose you), all the
  design's; and **the Leap potion, 1 sapphire** (Bridge Wars' own, 45 s,
  mine: a departure from the design's list, which had none, so that every
  button Bridge Wars has does something here too). The Bridge Builder is
  not sold.
- **The upgrade shop** (the team's; each step dearer than the last by the
  times shown, worked out in whole numbers, rounded half up, so that the
  server and every device ask the same):
  - Generator: Gold Forge 24 silver, Sapphire Forge 12 gold, Amethyst
    Forge 8 sapphire, then Overclock from 4 amethyst, x1.5 a step,
    endless (the design's);
  - Sharpness: 6 gold, x1.45 a step, endless; +12% on every blow of the
    team's (the design's), the fireball and dynamite too (mine; the
    design's swords and fists alone);
  - Protection: 20 silver, x1.6 a step; 8% off every blow, five at most
    (the design's); with the best shield, at most 42% of a blow gets
    through (they multiply);
  - Fortify Anchor: 24 silver, x1.6 a step (mine; at x1.35 a lone player
    bought one after about every wave), endless; +250 health and all of
    it back (the design's); not to be had once the anchor is broken;
  - Anchor Beam: 2 sapphire, x1.5 a step, endless; every 2 s the zombie
    nearest the crystal within 12 m is zapped (the design's) for 12 a
    step (mine; with the design's 20 a lone player who got past the
    first giant was carried by the beam as far as a team of four);
  - Lucky Loot: 5 gold, x1.6 a step, five at most (the design's five).
  The prices of each line are mine; the design named the first of the
  generators' and the Overclock's.
- **The score** (the design's): each kill by its kind, walker 10, runner
  15, spitter 25, leaper 30, brute 40, giant 250, a tenth more for each
  wave, to whoever dealt the last blow, and 1 for every 10 damage dealt;
  the team's is everyone's and 100 for each wave cleared.

### The server (`server/src/last_stand.rs`)

A thin wrapper round the shared `Game`: each frame it is told the room's
clock and where everyone stands (as their devices last said), stepped, and
what it says happened carried out. A zombie is a body in the table's room
with a `Shambler` on it (its kind and health, sent only as they change),
moved only as it moves; a spitter's ball, a fireball and a stick of
dynamite are entities sent once as they set out (`Spit`, Bridge Wars'
`Fireball` and `Dynamite`) and taken away as they land, burst or blow; a
blast goes as Bridge Wars' do (`Blasted`, with no planks, kept 3 s, mine);
what lies about goes as Bridge Wars' things do (`Loot`). Whoever a brute,
a giant or a leaper lands on is knocked by their own device
(`Tell::Knocked`); whoever stands again is stood there (`Tell::Arrive`).
How the game stands goes on the table's entity, each part only as it
changes: `StandRound`, `StandUpgrades`, and Bridge Wars' `Purses` and
`Kits`. What each person asks: a punch on a zombie's body (`Ask::Punch`, by
its member's id, which the server gives each zombie from a range of its
own, a sixteenth of the way down from the top, clear of every other id),
`Ask::LastStand(Buy | Ready)`, `Ask::Use`, `Ask::Respawn`; the game
believes none of it that could not be. Whoever is dead with the anchor
broken is out of the game (`Seat::out`), free to watch or to go.
`Tell::LastStand(Slain)` is new (mine): a giant brought down, and by whom,
for the chat, which a device cannot tell from the body going.
`StandRound.began` is new too (mine): when the fight began by the room's
clock, so that a device counts the fight's own time (what a potion's time
left is by) right from the first frame of a game it comes back into. It
never changes, but goes with the round every time that is sent, a
component being sent whole: eight bytes more a time, the round being sent
a few times a second in a wave (this said "sent once" until the check
below).

### The device (`src/last_stand.rs`, `src/last_stand/`)

**The same game either way.** Offline the device runs the shared `Game`
itself (`play::run_offline`), carrying out its events as the server does,
its zombies' bodies `lobby::spawn_bot_at`'s with a `Shambler` on each;
online it follows what the server sends (`play::follow_online`) and asks
for what you do (`play::ask_online`). Everything drawn is drawn from the
same few things either way (`Stand`): the round, the upgrades, everyone's
purse and kit, and the zombies' bodies. While a table of it is played the
game owns the fight's state, Bridge Wars' kit's (`arena::Fighting` on its
venue, `Resources`, `Outfit`, `FightTime`, `market::Stalls`, `loot::PadTop`),
and gives it all back once as the table goes.

- **Camera and buttons**: Bridge Wars' (the table fixes the camera
  overhead): top left your name, your health, what you hold; round the
  jump button the sword in its place, the jump beside it, the five buttons
  for things to use, the chat's keyboard. No plank. Desktop: WASD, Space,
  F, Q and 1-5 as in Bridge Wars, and **R for Ready** (mine).
- **Everyone is on one team, the blue** (`TEAM`, mine; the brief said
  team 0, the red): a punch passes a teammate by and lands on the zombie
  behind them; names over teammates' heads, yours top left, and everyone's
  in the chat, in the one colour (red would read as hurt beside the bars,
  green as one of the zombies).
- **The plate top centre** (`screen`): in a wave, "Wave 7" over "12 left";
  between waves, "Next wave in 0:18" over "Wave 7 cleared", "Get ready"
  before the first, or "1/2 ready"; inside it, under its big line, the
  anchor's health, a bar; and beside it, not under it, between waves,
  **Ready** (mine: under it, the item shop's window came down over it).
  Standing in a shop's outline, the line under the plate is put away (the
  shop's window comes down over it).
- **Across the screen**, once each as it comes: "Go!" over "Hold the
  anchor" as the play begins; "Wave 7" as one comes, with what is new in it
  under it ("Runners!", "Brutes!", "A giant!", "2 giants!", "Spitters!",
  "Leapers!"); "Wave 7 cleared"; "The anchor is broken: no more coming
  back" (Bridge Wars' line for what its timetable does, `arena::alert`),
  but for when the game ends in the same step, the last of you down as it
  broke; "The last stand fell at wave 9", which takes that line down if
  it is still up.
- **In the chat**: "Mochi bought Sharpness Lvl 3" (Hajun's brief's
  words), "Mochi is down", "Mochi is out", "Mochi brought down a giant",
  "The anchor is broken", "Wave 7 cleared"; names in the team's blue, as
  in what anyone types here: the game gives every seat the one team
  (`Fighting::teams`), where the tables had none for Last Stand, so as
  not to write a team of everyone all in red (mine: blue is no such red;
  in the chat's own pale gold for a name, tried, the names were in
  another colour than over the heads and barely read on the sand by
  day).
- **Dead**: "Back in 6", counting down (the kit's `DeathWords`); never
  "Back in 0" (the game stands you again a frame after the clock says).
  **Out** (dead with the anchor broken): a small window low on the screen,
  "You're out" over "Watching the rest hold on", with Leave; and the camera
  on whoever of your team still stands, the nearest to the anchor, or on
  the anchor's place (`crate::Lookout`), to the end.
- **The zombies** (`looks`): Bridge Wars' zombie, each kind in a tint of
  its own (`colours.last_stand`, mine: a walker Bridge Wars' own green, a
  runner a paler yellower green, a brute a dark olive, a spitter an acid
  green, a leaper a blue green, the giant a deep grey green); a brute a
  third bigger and a giant twice as big, their legs swung the slower the
  bigger they are (a giant's stride half as often for the ground it
  covers); a health bar over each (the kit's tags); a number floating off
  each for every blow, the one that kills it too (as what it had left: its
  body goes in the frame of the blow). **No name over any** by default
  (mine: "Runner", "Brute" in the zombies' pale green barely read on the
  stone by day; the file's `kinds` puts them back).
- **The anchor's crystal** (the map's `spawn` pillar): a bar over it,
  twice a body's, stood on the screen over the crystal as the screen has
  it, over the higher of its foot and its tip (seen from straight over
  it, the tip is drawn the further out from the middle of the screen the
  further out the crystal is, and a bar a set way up the screen from a
  point over the tip lay on the crystal once it was low on the screen),
  and put away where the plate top centre is, which has the same bar
  (just under it, the two read as one bar twice); a white flash over it as
  it is hit (no oftener than every 0.35 s, mine, or a heap of zombies
  would keep it white); broken, it shatters, sixteen pieces flying out and
  falling, and it is gone from underfoot as well as from sight, stood up
  again as the game goes. **The beam**: a short bright line from just
  over the crystal's tip to just over the head of what it zaps, with a
  spark there, bright a moment and fading in half a second (mine: from
  the crystal's middle it ran inside the crystal and under the head of
  the zombie clawing at it, and was not seen).
- **The generators**: what they make comes up on the pads as Bridge Wars'
  things do; over each pad, what it makes: "Silver", "Lvl 2: gold,
  sapphire", "Lvl 5: all four, x1.56". **What a zombie drops** flies out
  from where it fell and lies for whoever walks into it.
- **A spitter's ball**: a small bright green ball arcing to where it comes
  down, and a green splash there that spreads and fades.
- **The shops' windows** (the kit's market, with this game's cards): the
  item shop's swords, shields and things to use with Bridge Wars' pictures
  and names at Last Stand's prices; the upgrade shop's six, with the
  generators' forge, a sword and a shield with a plus, and three new
  pictures drawn in code: the crystal with a plus (Fortify Anchor), the
  crystal and a bolt (Anchor Beam), a four-leaf clover (Lucky Loot). An
  endless line's card says the level it takes the team to ("Lvl 7"),
  the capped two their pips; the generators' line goes by the line's own
  level everywhere (the card, the pads, the chat).
- **The results**: the table's window, each person's score ("score"), and
  under them "Waves cleared: 8 · Team score: 4,210" (`total_words`, from
  everyone's scores as the table has them).
- **For trying it out**: `RT_STAND_WAVE` (the first wave to come) and
  `RT_STAND_PURSE` (everyone holding that many of each), on the desktop
  game offline and on a server, each said in the log as a warning, never
  on a server real players use.

### Checked, and where

- **Tests**: the rules' (`cargo test -p roundtown_net`), the server's
  (`-p roundtown_server`: two new, a table of Last Stand played through the
  `Rules` with no network from the first wave to the anchor's fall, two
  people, one swinging and buying; and each ask carried out and told:
  buying at the right stand only, the team's upgrade told to everyone, a
  fireball out and burst, Ready bringing the wave in three seconds, a fall
  into the void and the stand again six seconds later), the game's
  (`--lib`: the buttons, the words, the cards and their prices, what each
  wave says is new, the anchor's line never left under the end, the pads'
  words, the crystal's triangles, a giant's slower legs, the map against
  the rules).
- **In the desktop build, hidden** (`RT_BACKGROUND=1`), offline, by a
  throwaway taken out again, every picture looked at and small things
  enlarged: by day, games from the waiting room to wave 5 (going round
  the anchor at "Go!", wave 1 climbing over the rim, fights with numbers
  and bars, a sword, the Gold Forge and the gold on its pad, Fortify,
  Sharpness, the Ready, wave 5), and games from wave 8
  (`RT_STAND_WAVE=8 RT_STAND_PURSE=200`) to their end (the shops' every
  card, brutes and spitters, the spit, the beam, the anchor shattering,
  the end, the results); by night, games from wave 8, 10 and 15 to their
  end (the crystal's glow and the light it throws, the shops, the beam,
  the end as the anchor broke with nobody standing, the results) and one
  from the first wave to the gold on its pad. What looked wrong was
  mended and shot again (the plate under the shop's window, the
  crystal's bar on the crystal, just under the plate's own, and on the
  crystal again low on the screen, no numbers off killing blows, the beam
  unseen, the end's words over the anchor's, and again when both came in
  one step, "Back in 0", the window that says you are out over whoever
  you watch, the game's names in the chat in another colour from what
  people type there, and then, in that, barely read on the sand).
- **Against a server on this PC**, two hidden copies, each its own
  `APPDATA` (Mochi and Dubu), from wave 8 with full purses, three times
  through to the end: both seated at one table and the fee taken; both
  see the same zombies in the same places, each other's names and bars,
  the same chat lines, the same beam; the team's beam bought by both, to
  level 2; one out while the other stood (the window, the camera on the
  other); the end at wave 8, places by score, "Waves cleared: 7 · Team
  score: 997" (and 737 the last time), each paid 70 (1,020 after, from
  1,000 less the fee); and back in the town, on its arrival ring 31.5 m
  out, nothing of the game left on either device (no zombie, nothing
  lying, nothing of the fight).
- **Not**: a phone (nothing of it has been seen on one), an iPhone (the
  iOS type-check was not run; nothing of it is behind `target_os =
  "ios"`), the VM, two real accounts; a giant or a leaper near enough to
  be seen on the screen (the rules' tests have both; the hidden runs died
  or fell before one came near); people typing in the chat in a game
  (the game's own lines were seen, in the team's blue); the frame rate
  (the hidden runs drew a few frames a second, the PC busy with other
  runs; four of seventeen lost the GPU to the driver, `DeviceLost`,
  each while other copies of the game were drawing, once both copies of
  an online run at the same instant, and the same runs went through
  again unchanged).
