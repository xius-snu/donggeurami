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

1. Since 2026-11-18 Bridge Race is played at its tables, online and offline, to the golden island

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
  called); the server's 86 (two new: a whole table of one person against
  the computer played through the `Rules` with no network, from the
  begin to the computer holding the golden island, the table paying,
  every rung and the round sent to the room and everything taken away at
  the close; and a person's rung, punch, metres, fall and skip each
  reaching the game, with what comes back to their device); the game's
  373 (six new: the buttons clear of each other; the fight state taken at
  a table offline, the race begun with the game played here, a grant
  counted once onto your metres, and all of it given back; the chat's
  lines; the ring's arc; the arrow at the edge; the plate's words).
- **In the desktop build, hidden** (`RT_BACKGROUND=1`), offline, by a
  throwaway taken out again, every picture looked at (the best kept in
  `RoundTown-wt/scratch-race/shots/`): a whole game in which you are given
  90 m more and build straight across and win (the waiting room, the
  countdown, the start on your island with 10 m in the plank button, a
  bridge out over the void, both teams' bridges from over the map, a grant
  "+6m", the ring filling with "Hold! 2", "Red wins!", the results with
  +110), by day and by night; and a game of the fight (a punch on the run
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
  (no plank, stick, blast, round or kit). An earlier run had the two at two
  tables of one against one each: one won (paid 110), one lost to the
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
