# Bridge Race

> The full account of Bridge Race: what was built, why, in whose words,
> every rule and number and whose call it was, and what was and was not
> checked. Dated entries are newest first. The rules are
> `roundtown_net::bridge_race` (`shared/src/bridge_race.rs`, the computer's
> players in `shared/src/bridge_race/brain.rs`); the server's side
> `server/src/bridge_race.rs`; this device's `src/bridge_race.rs` with
> `src/bridge_race/offline.rs` (the game played here offline) and
> `src/bridge_race/show.rs` (what is drawn). How its table fills, starts
> and pays is the tables' (`server/src/tables.rs`, `src/table.rs`,
> `roundtown_net::table`).

## Contents

1. 2026-11-18, the same day: nine things a checker found, put right
2. Since 2026-11-18 Bridge Race is played at its tables, online and offline, to the golden island

## 2026-11-18, the same day: nine things a checker found, put right

A checker went over the first entry's work and found nine things wrong;
all nine are fixed. What each was, what now happens, and whose call:

1. **Taken on into another game from the race's results** (your group's
   leader starting another of the four games' tables from them), the race
   went on here under the next game. The table says so with that game's
   `table::Entered` alone, no `Gone` between (its way), and only a `Gone`
   gave the race back: it went on writing the fight as its own every
   frame, putting its team on you, taking the next game's death words down
   or counting your death twice; and in Bridge Wars after it, its fight and
   Bridge Wars' were written in turn every frame, which tears the bridges
   down (`bridge::rebuild` clears them whenever the fight's number
   changes). Now **the race lasts exactly as long as you sit at a table of
   Bridge Race** (`table::AtTable`): at none, or at another game's, it is
   given back by one function, `give_back`, whichever way you went. If the
   next game has taken the fight already in that same frame (which of the
   two goes first is nobody's say), its fight is left as it is (mine).
   What the race put on everyone else's body, their team and no bar, is
   marked (`Dressed`) and taken off wherever the body is now: one of your
   group comes into the next game with you as the same body, and its bar
   over its head in Last Stand would have stayed hidden (mine). Your own
   body is the table's to let go of, or the next game's to dress.
   - Found with it (mine): Last Stand gives its fight back on any
     `Entered` with no metres of bridge (`Resources::default()`), and run
     after the race's taking of the fight in that frame it would have left
     you 0 m at the start. The race now keeps your metres at ten until it
     begins, unless the server has told you yours. Last Stand's side of it
     is not this game's file, and is reported to the orchestrator.
2. **A player whose device had gone could not be knocked off the golden
   island, and still counted on it.** Their body stands where it stood,
   and a knock is their own device's to carry out (`Tell::Knocked`), which
   is not there to: put the app away on the island and you held it and
   won. Now someone whose device has gone (the table's `Seat::away`, told
   to the game every frame, `Game::away`) **counts for nothing on the
   golden island**: their team cannot hold it, nor have them counted at
   the time limit, until they are back, or gone for good (the rest of the
   team then holds it alone). The checker offered this or the server
   knocking the body itself; mine, this: the server has no knock of a
   person's own (its numbers are `ui.ron`'s `melee`, which only the device
   reads), and a body it moved would not be where the device has it as
   the app is resumed.
3. **The server took any rung within 15 m of its layer's feet**, however
   much bridge they had been given: the first entry's own online run laid
   63 m on 28 m granted (its throwaway gave the device 90 m more) and was
   paid. Now a rung is taken only while what its layer has been held to
   (`charged`: each plank as its rung comes; a rung that comes before
   those ahead of it at the longest a plank is, 6 m, its own plank not
   yet known) stays within what they have been given and what their own
   device lays past it (`most_laid`): the 10 m they began with, every
   grant to them as it came, 1.5 m at the start and at each grant, and 6 m
   once. The 1.5 m is the device's bridge itself: as it runs out, the
   plank in your hands is fixed where it reaches, never shorter than the
   1.05 m it keeps ahead of you whatever is left, so that you do not walk
   off its end, and that leaves you none, so once at the most for each
   grant (read in `bridge::grow` and `fix_tip`); the 6 m is the plank
   that lands a bridge on a shore, as long as the gap it crosses (up to
   5 m) whatever is left. Mine, all of it: every rung of a real bridge
   is taken (a rung turned away would leave its layer on a plank nobody
   else has), and a device saying it has more than it was given lays no
   more than one that does not. And **a rung from someone fallen into
   the void is not taken**, but in the first second after the fall: the
   device fixes the plank in hand as it finds you dead and sends it just
   after its word that you fell (`bridge::lay`), which turned away would
   be yours alone (mine).
   - In numbers: an over-saying device at the start is taken to about
     17.5 m; a device laying exactly what it is given, running out at
     every grant, lays 99 m on 22 grants two against two and 109 m on 14
     one against one, and every rung of it is taken (a test).
4. **Someone whose device had gone fell only where the map had nothing
   under them**, so that one gone in mid-fall under a bridge or an island
   hung in the void to the end. Now, as Bridge Wars has it, they fall too
   when under the ground there by more than 1.5 m (`AWAY_UNDER`, Bridge
   Wars' own). The rule is the game's now (`Game::away`), not the
   server's wrapper's.
5. **A device put back in the race after its app was closed played the
   start again**: "Go!" and your team, "Dynamite!" for the sticks you
   held, whose bridge had reached the golden island and who was holding
   it, across the screen and in the chat. Now a race this device sees
   begin more than 3 s into its play (`LATE`) is taken as it stands, and
   nothing of it is said again: which bridges are across, who holds the
   island, your sticks, the grants (your metres the server tells you).
   Mine: the 3 s, which a device seeing a race from its start never
   reaches, and reopening an app and logging in again always does.
6. **The words at the end were wrong for some endings**: a win by the
   other team all leaving said "Your team got there first!", a draw by
   both teams holding the island in the same moment "Time's up!", and a
   skipped game "got there first". The round now says how it ended
   (`RaceRound::finish`, the protocol still 22, not yet out), and the end
   goes by it: held by your team ("Your team got there first!"), by the
   other ("The other team got there first"), by both at once ("Both teams
   got there at once", new), the time up or skipped ("Time's up!"), or a
   team all gone ("Blue left the race", new, over "Red wins!"). The two
   new lines are mine, in `words.bridge_race` (`together`, `team_left`).
7. **Three colours were set in the code**: each team's bar on the plate
   and "Hold! 2" drawn solid, and the ring at 0.92, whatever
   `arena.team_red` and `team_blue` say (they are kept faint, `d9`, for
   the names over heads). Now `colours.bridge_race` has `bar_strength`
   (1.0), `ring_strength` (0.92) and `hold_strength` (1.0), how solid each
   draws the teams' own colours, for Hajun to change: what is drawn is as
   it was.
8. **The first entry claimed more than was seen**: its night run, its
   day pictures and its online run are put right in it, below.
9. **"1/2 on the island" stayed up for the one left of two**: the round
   says how many of each team are still in the game (`RaceRound::players`),
   and the plate counts of those.

The round is 15 bytes as a game begins (13 before), 17 at the most in a
game, 20 at the most it could be. Protocol 22 is unchanged: none of this
has been out.

### Checked, and where

- **Unit tests**, all three suites through `rtcargo.sh race`: the rules'
  145 (three new: a gone device on the golden island, in mid-fall and
  over the void; rungs past the metres given and from someone fallen a
  while; and a device laying exactly on its grants, running out at each,
  every rung of it taken over a whole game, both sizes of table; and in
  the old tests, how each game ended as the round says it and its size);
  the server's 88 (one new: a person whose device has gone holds nothing
  on the golden island, and a device laying straight across at the start
  is taken only as far as its metres); the game's 378 (four new: taken on
  into Last Stand's table the race gives the fight back, the mate's body
  undressed and nothing written again after, and the next game's fight
  left as it is when that game took it first; a race come back into
  under way, nothing of it said; the end's words for every ending; the
  teams' colours as solid as the file says).
- **In the desktop build, hidden**, by a throwaway taken out again, every
  picture looked at (kept as `RoundTown-wt/scratch-race/shots/fix_*`):
  - Offline, by day, one against one: four grants built on what was
    granted alone, and every rung this device laid taken by the game (8,
    13, 18 and 23 of as many); then told 90 m more, as a device gone wrong
    would be, its bridge went on across while the game took its rungs to
    46 m (37 of 47; at four grants it allows 47.5); on the golden island
    the ring filled in red with "Hold! 2" and "Red is holding the golden
    island!", then "Red wins!" over "Your team got there first!", the
    results ("46 metres built", +110, 1,060), and back in the town
    nothing of the race. The bars, the ring and "Hold!" looked at
    enlarged: as they were before their strengths were in the file.
  - Against a server on this PC (port 5483), three hidden copies in turn:
    two people on red against the computer's two, 3 m a grant. Every rung
    the one building laid on its grants was in the room (20 of 20 after
    five grants). The other's app was closed eight seconds into the race
    and opened again: put back 40 s in, told its 16 m, and nothing of the
    start said again (no "Go!", no "You are on Red"), the next grant's
    "+3m" and the stick that came with it ("Dynamite!") said as they came.
    The first skipped the rest: "Blue wins!" over "Time's up!" on both,
    and back in the town nothing of the race left on either.
- **Not checked**: by night (not run again: this PC's GPU driver reset
  twice in the first tries by day, `DeviceLost`, each within two seconds
  of another worktree's build starting); a group taken on into another
  game from the race's results in a real run (its unit test alone); a
  table of three; a draw by holding together, a team all gone, a gone
  device on the golden island, in a run (unit tests alone); the results
  window online (the copies ran so slowly that the table's 15 s of
  results were over before the throwaway's picture of them); any phone.
- **Seen, and not this game's**: put back after its app was closed, a
  device has the game's name said across the screen again ("Bridge
  Race"), the table's own word for a table it was not at (`table`).

## Since 2026-11-18 Bridge Race is played at its tables, online and offline, to the golden island

Hajun, 2026-11-18, among the four new games: "Like King of the Ladder but
with bridge building. You start on two islands next to each other, and
there is a golden island at equidistance from each island like the tip of
a 80deg 80deg 20deg isosceles triangle. Every interval you are given more
bridge meters, and sometimes u get dynamite as well, randomly. Your goal is
to reach the golden island first compared to the other team - both of u
need to make it across cuz its 2v2. You can punch each other off the map,
and ur respawn is the center of ur own player island. You can use dynamite
to destroy bridge. This is not a 2 player game, this is a 4 player game,
and for now, i want 2 ai bots to be on the same team, and the other team is
real players either 1~2 player. If there is only 1 real player, then its a
1v1 against bots, and if its 2 players then its a 2v2 against bots (later
this will just be real player only but for the situation where there is
not enough real players and only 1~2 ppl are in queue, then 10second later
it will fill with bots, similar to how bridge wars is working now)
(similar controls and cam with bridge wars). Map = bridgeracemap". And of
all four: "the details i didn't mention are yours to decide to make each
game polished and fun", and "each of these games have a coin entrance fee
and payout. Generally should be above 50% to win net positive coin".

### The game

- **The map** is Hajun's `bridgeracemap.glb` (`Venue::BridgeRace`), stood
  0.25 m up as the game loads it (`island::lift`, so that every rule
  written for Bridge Wars' heights holds): red's island at (-80, -21) and
  blue's at (-80, 21), each 20 m across, 42 m apart; the golden island in
  the middle of the map, 20 m across, 83 m from each, 62.7 m of void from
  shore to shore; the void under all of it (fall 30 m and you are dead).
  The stands and pads on the teams' islands are scenery: there is no shop.
  The golden island is purple in the map, with a gold band 5.5 to 6 m out
  from its middle, a step up (the band is the map's own; the ring drawn
  over it is ours, below). Everyone waits for the race in a glass room 120
  m over the map between the two islands.
- **The table** (the tables' own, Bridge Wars' way): people are seated two
  by two onto a team, red first (seats 0 and 2 red, 1 and 3 blue; people
  take seats in the order 0, 2, 1, 3, so that two people are one team).
  10 s after the last person joined the computer comes for the seats left,
  one at a time: one person plays one against one, two people two against
  two (Hajun: "If there is only 1 real player, then its a 1v1 against
  bots, and if its 2 players then its a 2v2 against bots"); three, the
  first two (or a group of two) against the third and the computer; four,
  people alone. A countdown of 5 s, the race, and the results up for 15 s.
  Offline a table is always one against one.
- **Coins**: 50 to join, given back to whoever leaves before the start;
  each of the winning team's people is paid 110 (net +60), the losers
  nothing (net -50), and a draw gives everyone their 50 back.
- **Metres of bridge**: everyone begins with 10, and every 16 s each team
  is given 6 more, shared between those of it still in the game: 3 m each
  two against two, 6 m one against one or for one left of two (G1's change
  from the first spec's 5 m to everyone every 10 s, at which two against
  two was over in under a minute and one against one in under two; a game
  is about two and a half minutes at the median either way). A person's
  metres are their own device's to count, as in Bridge Wars: it adds each
  grant the round says has come (`RaceRound::share_of` your team), lays
  your bridge under your walk with the plank button (B on desktop) and
  pays for it; the server keeps only what the device last said
  (`bridge_race::Ask::Metres`, once a second at the most), to give back to
  a device that comes back to the game afresh, with every grant since
  (`bridge_race::Tell::Metres`).
- **Dynamite** ("sometimes u get dynamite as well, randomly"): with each
  grant each player has one chance in five of a stick, two at the most, on
  a button for things to use (Q or 1 to 5 on desktop). It is thrown as
  Bridge Wars' is, held and dragged; it takes the planks its blast reaches
  and no more (G1's call: no falling of the rest of the bridge after them,
  which in a race would decide it at the first stick), from under anyone;
  and it knocks only the other team.
- **Punches** are Bridge Wars' (Minecraft 1.8.9's knock, more on the run)
  and do nothing but knock: nobody has health. The void is the only way to
  die, and you are back on your own island's middle 2 s after you are dead
  ("ur respawn is the center of ur own player island"), with your metres
  and your sticks (G1's call). A fall counts to whoever of the other team
  last hit you within 5 s; a blast that takes the plank from under you
  counts to its thrower.
- **Winning**: every one of a team standing on the golden island at once
  ("both of u need to make it across cuz its 2v2"), together for 3 s (ours:
  "Like King of the Ladder"); the other team can still knock one of them
  off, which empties the ring. At 6 minutes the team with more of its
  players on the golden island wins, or else the one whose bridge has got
  further across (as the plate shows it), or it is a draw. A team with
  nobody left in the game loses at once.
- **The computer's players** (G1, `brain.rs`): each of six kinds (builder,
  bruiser, saboteur, steady, rusher, keeper), every threshold a spread
  thrown again (Hajun: "if some thing becomes a definite if-then, then its
  not a real human, but a program"). They lay their team's way across, one
  at a time, the one with metres in front; wait at the end for more; punch
  whoever of the other team comes near, toward the void; throw dynamite
  at the other team's bridge where it is longest or someone stands on it;
  jump onto the golden island only for a reason (cut off, or the other
  team about to win); and hold it. Over thousands of whole games of the
  computer alone (`shared/src/bridge_race/tests.rs`: `many_races` with
  `RT_GAMES`, `RT_FROM` and `RT_STALLS=1`, `watch_a_race` with `RT_SEED`,
  `RT_SEATS` and `RT_EVERY`, `timing_of_a_race`, all ignored until asked
  for): a game is about 151 s two against two and 154 s one against one at
  the median; red and blue each win about half; the first team onto the
  island went over by its bridge in about nine games in ten, and wins
  about four in five; a stick is thrown in 98% of two against two games
  and 86% of one against one; about 19 knocks and 2.4 knocked off a game
  two against two, 4.5 and 0.5 one against one; and a step of a game
  takes about 9 microseconds.

### What is on the screen (`src/bridge_race/show.rs`), all judged from the camera over you

- **Bridge Wars' buttons** (Hajun: "similar controls and cam with bridge
  wars"): the camera fixed straight down 15.5 m over you; your name top
  left, in your team's colour once the race is on; the fist in the jump
  button's place, the jump beside it, the plank with your metres in it, a
  button for each kind of thing to use (a stick of dynamite, with its
  count), and the chat's keyboard. No health bar, no generators' counts,
  no minimap.
- **The plate top centre**, on the plate's grey, from "Go!" to the end: the
  time left ("4:12"), and under it each team's way across side by side,
  red's and blue's, a bar in the team's colour as long as its bridge has
  got toward the golden island (its plank nearest it, as a share of the
  62.7 m), with under it how far it still has to go ("34m to go"), or
  "Across!", or how many of the team stand on the island ("2/2 on the
  island"). Mine: the words are white, the bar saying whose it is (in the
  teams' colours, as first written, they went all but unread on the
  plate's see-through grey over the islands' grey by day).
- **The ring round the golden island**: over the map's own gold band, an
  arc in the colour of a team that has all of its players on the island,
  filling as its hold goes on, red's the way the clock goes from the top
  and blue's the other way; rebuilt only as it comes to another tenth of
  the 3 s; unlit, so that it reads by night as by day; full in the
  winner's colour once a team has won by holding. And "Hold! 2" in the
  same colour over the island, a little up the screen from its middle,
  the seconds still to hold. Mine: all of it, the radii (5.25 to 6.25 m,
  covering the band either side) and where "Hold!" stands (at the middle
  it sat on your own head; 3.4 m up the screen it met the line across the
  upper half).
- **The way to the golden island**: while its middle is off the screen, an
  arrow at the edge of the screen pointing to it, in its gold, with how far
  it is from you to its shore ("73m"; mine: to the shore, the void being
  what you cross). Mine: the words go up the screen from the arrow on a
  side's edge, where the buttons are not (toward the middle they ran into
  the fist).
- **A grant**: "+3m" rising off you, up the screen, in white (mine: the
  planks' colour, as the brief had it, was lost on the planks you stand on
  as you build; white is what the metres left in front of you are in;
  `colours.bridge_race.granted`), and the count in the plank button going
  up.
- **Lines across the upper half**, one at a time (Bridge Wars' look): a
  stick come to your button, "Dynamite!" with its picture; a team's
  bridge come across ("Red's bridge has reached the golden island!"); a
  team holding the island ("Red is holding the golden island!"); the last
  two in the chat too. Mine: two said in one frame drew over each other,
  and are now one; and at the end every line is taken down, so that the
  end is read on its own.
- **The chat** (names in their teams' colours): "Mochi fell into the
  void.", "Latte knocked Mochi into the void.", "Mochi blew a hole in
  Red's bridge." (whose bridge it took most planks of; a stick that took
  none says nothing). Online the server tells each connected device
  (`bridge_race::Tell::Feed`), offline the device's own game says it, both
  by the rules' `feed`.
- **Falling**: "Back on your island in 2", counting down in halves, across
  the screen, then you stand on your island's middle (online as the server
  stands you there, offline as this device's game does).
- **The start and the end**: "Go!" over "You are on Red"; at the end "Red
  wins!" or "Blue wins!" or "Draw", over "Your team got there first!", "The
  other team got there first" or "Time's up!"; then the table's results,
  the score "metres built" (whole metres each laid), and the coins.
- **Names over heads** in the teams' colours once the race is on, with no
  bar under them (the fight kit's, `arena::NoBar`); plain in the waiting
  room, as in Bridge Wars.

Every word and colour is in `assets/ui.ron` (`words.bridge_race`,
`colours.bridge_race`, and the teams' own colours in `colours.arena`).

### How it is played (whose code is which)

- **Online the server plays the rules' `Game`** (`server/src/bridge_race.rs`,
  the tables' `Rules`): it is fed everyone's feet each frame (one whose
  device has gone falls if what was under them goes), each person's asks
  (`Ask::Rung`, `Ask::Punch`, `Ask::Use(Use::Dynamite)`, `Ask::Respawn`,
  `Ask::BridgeRace(Metres)`), and carries out what comes of it: every rung
  as a `Plank` and every stick as a `Dynamite` sent to the room, a stick
  that blew as a `Blasted` with nothing to fall after it, a knock to its
  person's device (`Tell::Knocked`), someone dead stood on their island
  again (`Tell::Arrive`), the computer's players where it has them, the
  round (`RaceRound`) and everyone's sticks (`Kits`) on the table's entity
  each as it changes, the chat's lines, and once it is over the places and
  scores, which the table pays by. A device back in the game afresh is
  told its metres. `Ask::Skip` (only with `RT_ALLOW_SKIP`) calls the time
  now, for trying it out (`Game::call_time`; mine).
- **Offline this device plays the same `Game`** (`src/bridge_race/offline.rs`)
  once its table is played: your feet, your new rungs, your punches that
  land on the computer's players (`crate::Landed`), your sticks and your
  fall go into it; the computer's bodies are put where it has them, with a
  plank in their arms while they lay and hidden while dead, their rungs
  laid into the bridges as the server's come, sticks drawn and blown where
  it blows them, a knock on you, and you stood up again; and at the end
  how everyone came is written for the table (`table::Finish`).
- **The fight's state is this game's while its table is up** (the fight
  kit's way): `Fighting` with Bridge Race's venue and number, each seat's
  team and members, playing from "Go!"; `OfflineArena` offline;
  `Resources` with your metres; `Outfit` with your sticks; `FightTime`;
  and `Team` and `NoBar` on every body. All of it is written only as it
  changes, and given back once as the table goes ("no game" with the
  number kept), with everything the fight put up taken down. The kit's
  bridges, dynamite, buttons, names over heads and punches then serve the
  race unchanged.

### Checked, and where

- **Unit tests** (all three suites green, through `rtcargo.sh race`): the
  rules' 142 (two new: the chat's lines of a fall and a blast, and time
  called; and the round's test of what is sent, with the chat's lines);
  the server's 87 (two new: a whole table of one person against
  the computer played through the `Rules` with no network, from the
  begin to the computer holding the golden island, the table paying,
  every rung and the round sent to the room and everything taken away at
  the close; and a person's rung, punch, metres, fall and skip each
  reaching the game, with what comes back to their device); the game's
  374 (five new, and the buttons kept clear of each other: the fight
  state taken at a table offline, the race begun with the game played
  here, a grant counted once onto your metres, and all of it given back;
  the chat's lines; the ring's arc; the arrow at the edge; the plate's
  words).
- **In the desktop build, hidden** (`RT_BACKGROUND=1`), offline, by a
  throwaway taken out again, every picture looked at (the best kept in
  `RoundTown-wt/scratch-race/shots/`): by day, a whole game in which you
  are given 90 m more and build straight across and win (the waiting
  room, the countdown, the start on your island with 10 m in the plank
  button, a bridge out over the void, both teams' bridges from over the
  map, a grant "+6m", the ring filling with "Hold! 2", "Red wins!", the
  results with +110). Those day pictures (`01` to `09`) were taken before
  the last fixes of that entry (the plate's words white over the bars,
  "+3m" white and rising up the screen, "Hold!" and the arrow's words
  moved, one line at a time across the upper half); the online pictures
  show those. **By night the same game got only as far as the ring
  filling** (`night_01` to `night_07`): there this PC's GPU driver reset
  (`DeviceLost` in its log), and neither "Red wins!" nor the results were
  seen by night. And a game of the fight, by day (a punch on the run
  knocking the computer's player off its bridge, "Mochi knocked Bori into
  the void.", your own fall with "Back on your island in 0.5" and back on
  your island, "Dynamite!" with its picture and the button, the stick
  aimed by a scripted finger on its button, in the air, lying in its ring,
  and the hole it blew cut clean, looked at enlarged; the computer's
  bridge grown, from over the map).
- **Against a server on this PC** (port 5483, no database, waits
  skippable), two hidden copies: both people on red against the
  computer's two, 3 m a grant each, one of them falling into the void and
  stood up again, the other building across and the first following over
  the bridge, both on the golden island holding it, red winning: paid 110
  each (1,060 from 1,000), the same plate, ring, words, chat lines and
  results on both, and back in the town nothing of the game left on either
  (no plank, stick, blast, round or kit). **The one who built across was
  given 90 m more by the throwaway**, so nobody bridged across online on
  the metres granted (the server took all of it then; since the next
  entry it takes no more than was given). An earlier run had the two at
  two tables of one against one each: one won (paid 110), one lost to the
  computer (paid nothing), both cleared away.

### Not checked

- **Nothing of it on a phone or an iPhone**, nor on the VM, nor with the
  database (`RT_DB`) taking and paying the coins.
- A table of three people, a group going in together, a device put back in
  the race in the middle of it (its metres given back are a unit test
  alone), a draw by both teams holding at once, and the six-minute time
  limit in a run (the rule is a unit test, and a skip calls it).
- Online the computer's players are not seen carrying a plank as they lay
  (offline they are): nothing sent says so, as in Bridge Wars online.
- **Seen in hidden runs, not ours to fix**: at a few frames a second (a
  hidden window on this loaded PC) a step can outrun the plank being
  laid, and you fall through the end of your own bridge as you build: the
  bridges are laid over the island after `bridge::lay` but not before
  `move_bodies`, so at such a rate the ground under your next step comes a
  frame late. The scripted runs held a step to a tenth of a second, as a
  slow phone's. And this PC's GPU driver reset under two or more hidden
  copies at once (lost devices, in other agents' runs too); every run here
  that finished is one that did not meet it.
