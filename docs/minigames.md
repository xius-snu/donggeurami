# The four games of 2026-11-18, and the tables they are played at

Blend In, Knock Off, Bridge Race and Last Stand, which Hajun asked for on 2026-11-18, are
each played at a **table**: one way in, one way of waiting and starting, one way of paying
and going back to the town, the same on the server and on a device offline. Each game's own
rules and computer players are in its own file here (`docs/blend-in.md`,
`docs/knock-off.md`, `docs/bridge-race.md`, `docs/last-stand.md`); this file is what they
share. Dated entries, newest first.

## Contents

1. 2026-11-19: Bridge Race's and Last Stand's cards are turned off
2. 2026-11-19: a sheet for each game, as Bridge Wars has
3. Later on the evening of 2026-11-18: Knock Off's stick anywhere, Bridge Race's grants and dynamite, Last Stand's chests struck open
4. The evening of 2026-11-18: Bridge Wars costs 50 too, prices drawn with the coin, and twelve asks after a play
5. Since 2026-11-18 four new games are on the selector, each played at a table

## 2026-11-19: Bridge Race's and Last Stand's cards are turned off

Hajun, having played all six: "i likethe bridge wars minigame, i also like
knockoffgame as a quickie, but laststand is a bit too long and unexciting,
and bridgerace not sure but its not that fun rn which maybe is because its
ai bots and not much diverse choices, and blendin game is bit too hard cuz
its way too hard to tell who is the real person if they just walk in a
similar way as the ai so its not fun to be the seeker. also its not much
difference in the "education" that the game gives compared to the bridgewars
game." They asked what to do; of what was put to them (turn those two off,
rework Blend In once later, land in the town before another game, quick
games on Knock Off's dial) they said: "ok first turn off last stand and
bridge race. dont rebuild yet".

- **`games::OFF`** (`src/games.rs`) names the games whose cards are not on
  the selector: Bridge Race and Last Stand. Turned off, not removed:
  `CARDS` still holds the six, and with `OFF` emptied they stand as they
  did, three to a row.
- **Four cards, two to a row** (`games::shown`, `laid`): Bridge Wars and
  Blend In over Knock Off and Home Builders, in the order they were in,
  each card half the window's width, the window the size it was. A row
  short of the first (five cards: three over two) keeps the room its
  missing card would have, so that its cards are no wider than the rest.
- **Only the cards.** The two games' rules, tables, maps, words in
  `ui.ron`, sheets and tuning tests are untouched; nothing in `shared/` or
  `server/` changed, so there is no new protocol and nothing to deploy.
  The server still seats anyone at those two tables: an app built before
  this (APK 43, and whatever is on a phone) still shows six cards and
  plays all of them.
- **Mine**: that four are laid two and two (not three and one, nor four in
  a line); that Blend In's card stays, since they did not name it; that
  the server is left taking those two games' joins, so that an app already
  out is not turned down at a card it still shows.
- **Checked**: the selector's unit tests (`games::tests`: the four as they
  are shown, the six with nothing off, three over two with one off) with
  the rest of `cargo test --lib`; the desktop build, hidden
  (`RT_BACKGROUND=1`, offline, a phone's 852 by 393), the gamepad tapped by
  a throwaway module since taken out, and the picture looked at: four
  cards, level, Blend In saying "Online only" as it does offline.
- **Not done, at their word**: no APK (43 is still the newest, with six
  cards), no deploy, no commit. Not seen on a phone.

## 2026-11-19: a sheet for each game, as Bridge Wars has

Hajun: "similar to my bridgewarsgameevent.md file that has the game event by
time, the shop itemes, make the same for all the minigames. list the major
game event per time in order, the shop/item in short concise way ... need to
be concise and not cognitively dense to read so i can read it fast and quick
and understand."

- **Five files at the root**, beside `bridgewarsgameevent.md` and laid out as
  it is (the `@... , Event` line, the events in order, then a `######` heading
  to each short list): `bridgeracegameevent.md`, `knockoffgameevent.md`,
  `laststandgameevent.md`, `blendingameevent.md`, `homebuildersgameevent.md`.
  Each ends with its coins.
- **Written from the code, not from these logs**: every number was read off
  the rules as they stand (`shared/src/bridge_race.rs`, `knock_off.rs`,
  `blend_in.rs`, `last_stand/` with its `shop.rs`, `waves.rs`, `forges.rs`
  and `game.rs`, each game's `SHAPE` and `prizes`; House Builder's `rules`
  in `shared/src/lib.rs`, `build::PIECES`, `builder::THEMES`). No code
  changed; nothing was built or run.
- **They follow the code**, where `bridgewarsgameevent.md` is Hajun's and
  the game follows it: a number changed in a game is changed in its sheet
  with it. A number Hajun changes in a sheet is an ask.
- **Mine**: Knock Off's is by turn (it has no clock, a turn is as long as
  its slide), Blend In's by the time of one round, Last Stand's by wave
  after the first; ranges are written "8-16", not Hajun's "8~16" (two `~`
  on a line strike the words between out in some viewers); and what is left
  out of each: the camera, the buttons, the netcode, how the computer's
  players think.

## Later on the evening of 2026-11-18: Knock Off's stick anywhere, Bridge Race's grants and dynamite, Last Stand's chests struck open

Hajun's next message, after APK 42, ending "fix and rebuild apk and push".
**`PROTOCOL_VERSION` 24**: APK 42 and before are told to update and stay
offline. The server has it and so does APK 43 (`docs/building.md`). No iOS
tag.

| Ask | Where it is written up |
|---|---|
| Knock Off: the stick taken anywhere on the screen, its middle where the finger goes down, but not round the buttons | `docs/knock-off.md` |
| Bridge Race: grants oftener (12 s), twice the bridge, 8 to 16 m at random | `docs/bridge-race.md` |
| Bridge Race: dynamite twice as often, and bringing the whole connected bridge down as in Bridge Wars | `docs/bridge-race.md` |
| Bridge Race: "Next Item: 14m (plank) 1 (stick)" under the clock in place of each team's "62m to go" | `docs/bridge-race.md` |
| Last Stand: chests broken open by one, two or three blows, bursting like someone dying, no bubble | `docs/last-stand.md` |
| Last Stand: no words over the generators, nothing under the plate before the first wave | `docs/last-stand.md` |

**A trap the tests did not catch, and a hidden run did**: a query added
to Last Stand's doings read every body's `Transform` where the same
system moves the zombies' (`play`): Bevy refuses such a system as the app
starts (B0001), and every test passed, none of them building the whole
app. It reads `GlobalTransform` now. A hidden run of the desktop build is
what starts every system.

## The evening of 2026-11-18: Bridge Wars costs 50 too, prices drawn with the coin, and twelve asks after a play

Hajun played the four games of the morning and sent a dozen messages in a row, the last
"then build apk and push to git". **`PROTOCOL_VERSION` 23** (Last Stand's new asks, tells
and things, `docs/last-stand.md`; Bridge Wars' fee changes nothing sent): an app from
before is told to update and stays offline. The server has it and so does APK 42
(`docs/building.md`). No iOS tag. Each ask is written up in its game's file; this is the
list, and what they share.

| Ask | Where it is written up |
|---|---|
| Bridge Race always by day | `docs/bridge-race.md` |
| Knock Off's power 1, not 0, as each turn begins | `docs/knock-off.md` |
| Last Stand: everything dropped on a death, for yourself or a teammate; swords only if better; gone after a long while | `docs/last-stand.md` |
| Last Stand's shops open sooner, as Bridge Wars' do | `docs/last-stand.md` |
| The app's "Donggeurami Town" goes at once when a window comes up | `docs/town-hud.md` |
| Blend In's seekers have the speed potion's effect | `docs/blend-in.md` |
| Bridge Wars costs 50 coins | `docs/bridge-wars.md`, and below |
| The coin's picture in place of the word "coins" | below |
| Bridge Race: win by holding the golden island, jumped onto or not | `docs/bridge-race.md` (it was so already) |
| Last Stand's sea, with a ring nobody passes, and no void | `docs/last-stand.md` |
| Last Stand's zombies red a moment when hit | `docs/last-stand.md` |
| Last Stand's treasure chests, opened by the bubble | `docs/last-stand.md` |
| The town's map with Hajun's plot planes | `docs/maps-and-collision.md` |
| Bridge Race: two people seated three ways, a third of the time each | `docs/bridge-race.md` |

### The selector's prices

Hajun: "instead of writing the text "50 coins" with the word "coins" use the actual coin
icon instead so its "(coin_icon) 50"". A card's price is the coin's picture and the number,
"(coin) 50", and one you cannot pay "Needs (coin) 50", in the card's middle, the picture as
tall as the words' capitals and a little more (`games::Note`, `spawn_note`, `COIN_SIZE`,
`COIN_GAP`, mine). The words are the file's: `words.games.price` is `"{coins}"` and
`needs_coins` `"Needs {coins}"`, the picture standing where `{coins}` is, so Hajun can put
words either side of it. Bridge Wars' card says "(coin) 50" and is blocked without it, as
every card is; a game that cost nothing would still say `free`. Tests: the parts either side
of the picture, and every card's price and whether it is to be had.

### Coins: Bridge Wars

| Game | Fee | Paid by place | Net |
|---|---|---|---|
| Bridge Wars (8 seats, 4 teams of 2) | 50 | each person by their team's place, 130, 60, 10, 0 | +80, +10, -40, -50 |

Mine, by the same rule: a team's place is when it had nobody left in play, and teams still
in at 20:00 share the places they take up (all four, 50 each, the fee back). The rest is in
`docs/bridge-wars.md`.

### Checked, and where

- **Unit tests**, all three suites, after everything: `cargo test --lib`, `cargo test -p
  roundtown_net`, `cargo test -p roundtown_server` (numbers in `docs/building.md`).
- **Hidden desktop runs** (`RT_BACKGROUND=1`, offline, a throwaway driver taken out
  again), pictures looked at enlarged where small: the selector with "(coin) 50" on every
  card and the town's name gone under it; Knock Off's first turn at 1; Bridge Race by day
  with the clock at night, and won by a jump onto the golden island; Bridge Wars offline,
  1,000 coins, 950 in the waiting room, the fight skipped to a draw and 50 paid ("+50" with
  the coin under "Time's up!"), 1,000 again in the town; Last Stand's sea, its ring, a drop
  of everything on a death, a zombie red, a chest opened by E, the shop's outline; the
  town's plot planes by day. What each showed is in its game's file.
- **Not checked**: anything on a phone or an iPhone; two devices at one table (the Bridge
  Race seating, a teammate taking what you dropped online, Bridge Wars' fee from a group);
  the server's fees and prizes against the real database but for the check after the
  deploy (`docs/building.md`).

## Since 2026-11-18 four new games are on the selector, each played at a table

Hajun, 2026-11-18, in one message: "This is a big one, cuz u will need to carefully craft
each minigame to the quality level of bridge wars minigame. add these minigames: Blend In,
Knock Off, Bridge Race, Last Stand (Zombie Apocalypse). I have got the rough ideas, and some
details are not entirely planned of yet, so the details i didn't mention are yours to decide
to make each game polished and fun. Make these minigames, add them to the game selector
panel, then build apk and push to git." And of the coins: "Also make each of these games
have a coin entrance fee and payout. Generally should be above 50% to win net positive coin,
and if not, net negative coin, but the exact amount can be different based on games."
**`PROTOCOL_VERSION` 22.**

### The selector

Six cards on two rows (`src/games.rs`): Bridge Wars, Blend In, Knock Off / Bridge Race,
Last Stand, Home Builders, each with its name, a line under it ("Hide among the town's
people", "Slide them off the ice", "Build to the golden island first", "Hold out against the
waves"; mine, in `words.games`) and what it costs. A card you cannot pay for is blocked and
says so; Blend In, which needs people, is blocked offline ("Online only"); every card is
blocked while you follow someone else's group, as before. The "???" card that was to come is
turned off behind `games::COMING_SOON`, not deleted. A test holds the window to 92 vmin tall
with the leader's line on 16:9, 19.5:9 and 4:3 screens.

### A table, from the join to the town

(`server/src/tables.rs` online, `src/table.rs` on the device; the numbers are each game's
`SHAPE` in `shared/src/<game>.rs`, `roundtown_net::table::shape`.)

- **Joining** (`Ask::Join(game)`, `Tell::Entered { game, seat }`): you pay the fee and take a
  seat; a group's leader takes everyone of it who is free and can pay, into the same table
  (Bridge Race: onto one team). A game without the computer (Blend In, Last Stand) seats you
  at its **oldest** table with room, so that nobody is left waiting at one a group passed by.
- **Waiting**: the board top centre ("Waiting for players 2/4", "Bots join in 0:07",
  "Waiting for 1 more person") with a **Leave** button under it. With the computer (Knock
  Off, Bridge Race): 10 s after the last person joined it takes the seats left one at a time,
  as in Bridge Wars ("fill with ai like bridgewars"). Without: once there are as many people
  as the game begins with (Blend In two, "This game min player count is 2"; Last Stand one,
  mine) and nobody has joined for 15 s, or at once when it is full (four). Blend In waits the
  full 15 s from its second person and at least 8 s after each later one.
- **Counting down** (5 s), then **play**, by the game's own rules.
- **The results** (`table::Results`): a window in the friends' panel's style, each seat in
  place order with its score in the game's own word and the coins paid to people, your own
  row picked out (`colours.window.row_mine`, new, mine), "Back to the town in 0:13", and
  **Leave** (dark red) and **Play again** (green). 15 s (Last Stand 20), then everyone still
  there is back in the town.
- **Leaving** while waiting or counting down gives the fee back; leaving during play keeps
  your seat for the results (and your prize, if your place earns one). Someone **out** before
  the end (in the sea in Knock Off; dead with the anchor broken in Last Stand) is free to go
  into any other game from there.
- **A device that goes** (a phone put away, or its app closed) keeps its seat while the game
  is played, as Bridge Wars' do (2026-11-15): put back in its seat when it comes back, while
  anyone of the people is connected; a table nobody is connected to for 60 s is given up.
  Before play a gone device is walked out and its fee given back.
- **Offline** the device hosts a table itself with the same stages, the computer coming one
  at a time after 3 s, the fee off your balance and the prize on it (as House Builder does
  offline); `net::link` does not connect while one runs.

### Coins

| Game | Fee | Paid by place | Net |
|---|---|---|---|
| Blend In, 2 people | 50 | 100, 0 | +50, -50 (a draw: 50 each, the fee back) |
| Blend In, 3 | 50 | 110, 40, 0 | +60, -10, -50 |
| Blend In, 4 | 50 | 120, 70, 10, 0 | +70, +20, -40, -50 |
| Knock Off (4 seats) | 50 | 130, 60, 10, 0 | +80, +10, -40, -50 |
| Bridge Race | 50 | the winning team's people 110 each, the rest 0 | +60 / -50; a draw, the fee back |
| Last Stand | 50 | 10 a wave cleared, each, to 400 | wave 5 breaks even |

All mine, by Hajun's rule: whoever beats more than half of the others comes away with more
than they paid, everyone else with less. Seats that tie share what the places they take up
pay, evenly, rounded down (`table::split`); the computer is paid nothing. The **winner's
XP** (`XP_WON`) goes only to whoever came first where someone came lower: nobody on a draw,
nor anyone playing alone.

**Fixed with it, on the server** (a trap that was waiting for a second game with a fee):
every database answer to a fee was seated into House Builder (`builder::paid`). Every charge
now says which game it is the fee of (`fees::Paying { game, into }`), each game seats only its
own answers, and a body with a fee in flight is refused by every game, Bridge Wars included;
a fee taken for someone no longer free is given back as "<Game> refund". House Builder plays
as it did.

### Disguises (for Blend In)

A game may put another `Member` on a body (another name, look and id) while the account's own
is kept beside it (`players::Account`, `players::Disguised`): a device that reconnects is
welcomed as its account, never as its disguise (so it never sends the disguise back as its
profile), `Ask::Profile` changes the body only while it is not disguised, and every way out
of a table puts the account's own back.

### The fight kit serves any fight

Bridge Race and Last Stand are played with Bridge Wars' own pieces: bridges, dynamite,
fireballs, the item buttons, the shops, things lying about, swords and shields, the punch's
range, names and bars over heads. Until now those worked only on `Venue::Arena` while Bridge
Wars' own game ran, and some of them undid any other game's state every frame. Now:

- `arena::Fighting` says the **venue of the fight** being played (`Fighting.venue`) and each
  seat's **team** (`Fighting.teams`); whichever game is played owns it, writing only what
  changed, and the "no game" state is written once, as a game goes (Bridge Wars' `publish`,
  `dress`, `lose_on_death`, `feed_chat`, `show_death` and `squash::lie` act only while its
  own game exists). `island::fights` (Arena, Bridge Race, Last Stand) and `island::bridged`
  (Arena, Bridge Race) say where.
- Shops are read off every map that has them, each at its own height; another game hands
  the window its own cards (`market::Stalls`, `Pick::Own`, `market::Picked`). What lies about
  hangs at its game's pad height (`loot::PadTop`). Dynamite and fireballs land on their own
  venue (`dynamite::blow` with no cascade is Bridge Race's hole). The death line's words are
  each game's own (`arena::DeathWords`). `arena::NoBar` names a body with no bar.
- **Bridge Wars plays exactly as before**: every one of its tests asserts what it did, and
  hidden runs of it, before and after, side by side, showed the same waiting room, shop,
  dynamite and fights.
- The chat writes a name in its seat's team colour as each fight's game says (Last Stand's
  one team in the plain name colour, not all in red).

### The maps

`iceisland.glb`, `bridgeracemap.glb` and `laststandmap.glb` load as the app opens
(`island::LOADED`). Bridge Race's and Last Stand's are **stood a quarter of a metre up as
they load** (`island::lift`; mine): Hajun made their land at 0 where Bridge Wars' map has it
at 0.25, and every rule written for Bridge Wars' heights (a plank's top, a pad's, a shop's
floor, how far a body falls before it is dead) goes by 0.25. If Hajun raises them in Blender,
set their `LIFT` to 0. Blend In is played on the town's own map in a venue of its own
(`Venue::BlendIn`): the town's scenery (land, sea, fountain, houses) is drawn there
(`island::Scenery`), none of its people, buses or House Builder.

### Calls that were mine

The fees and prizes above; Last Stand alone (Hajun wrote "Coop (2~4 players)": one person
can play it rather than wait for a second who is not coming); Bridge Race's people seated on
one team first (Hajun: "if its 2 players then its a 2v2 against bots", and "If there is only
1 real player, then its a 1v1 against bots", read literally: one person plays one of the
computer's); the lift; the results' own-row colour; the cards' lines; the car only in the
town (until now `C` on desktop could drive it in any game).

### At the merge

Each game was built, checked, played and polished on a branch of its own, and the four were
merged onto the tables without a conflict. Then, of what the games' builders found in files
that were not theirs to change:

- **The results say what the scores count once, over their column** ("turns survived",
  "points"...), where each number had had the word after it and one row read "1 turns
  survived" (mine; `table::spawn_results`).
- **The computer's players at a table offline take names nobody at the table has**: they
  had been drawn against the town's thirty-two seats too, which take up every name of one
  side between them, and two at one table were both called "Bot".
- **Someone standing still is no longer sent on and on** (`net::mark_still`, with a test):
  where you were the frame before was kept marked still, and `Spot::same` compares the mark,
  so a standing body was marked still and then not, frame by frame, and sent some seven times
  a second, since the netcode was written. In Blend In it told a person from the town's own.
- The placeholders the four games' messages were laid with are gone (`Ask::Nothing`,
  `Tell::Nothing`, Bridge Race's first variant among them) before protocol 22 went out; the
  base's test of what is sent names each game's real ones.
- Knock Off's test of how long the computer thinks goes by the middle of twenty turns, not
  the slowest, which a test run beside a build held up once.

Then the merged build was played through twice, hidden, each in a worktree of its own: once
offline (Knock Off, Bridge Race and Last Stand to their results by day, again by night at a
phone's 852 by 393 with a camera's hole, then Bridge Wars, its watch mode, House Builder and
the town), and once with two copies against a server on this PC, each copy one unbroken
session through all five games (Knock Off twice with Play again, Bridge Race with one app put
away mid-race, Last Stand, Blend In, a group into Bridge Wars), by day and by night. Neither
found a crash, a panic or an error; the coins were the table's in every game, on the server
and both devices; both copies saw the same games; nothing of one game was left in the town or
in the next. Of what they found, mended (all mine, 2026-11-18):

- **The chat is put away while a game's results are up**, unless a line is being typed
  (`chat::show`): its lines ran under the see-through window into its rows, "Alpha stayed
  hidden as Moc" into "1st". They are back with the town.
- **Knock Off**: while the table waits and counts down the camera keeps the top quarter of
  the screen clear (`CLEAR_TOP_WAITING`, easing to the play's `CLEAR_TOP` as play begins),
  where Leave had stood on the far player's head; the game's name, said across the screen as
  you sit down, is said low over the near sea, as the game's own headlines are, not over the
  players at the sides; and the "You're out!" panel's line goes on two lines, the panel no
  longer reaching the chat's keyboard button at 16:9.
- **Last Stand's headlines have a thin dark stroke** (`hud::announce_stroked`, four dark
  copies under the words, faded with them), "The anchor is under attack!" having been gold on
  the sand by day; and **Blend In's "Round 1/2"** under its plate, lost in the hider's column
  by day and in a lamp by night.
- **The selector's cards stand level**: each card's line has two lines' room, where a card
  with a one-line line had sat lower than its neighbours.
- **The town's buttons' pictures are never let go of** (`hud::KeptIcons`): a picture read off
  a file and shown by the town's buttons alone was let go of during a game and read again on
  the way back, the gear, the coin and the friends' and gamepad's buttons empty discs for the
  first frames.
- The server's one unused method went, its build without a warning again.

Each was looked at in the running game afterwards, hidden (the selector, Knock Off waiting
and counting down, the town three frames after a Leave, Last Stand's "Go!" enlarged), but the
out panel, the round line and the chat under the results, which are covered by tests and code
alone. **Left, and why**: the results window's corners over the stick and the jump button
(the game is over, and the window is in front); Last Stand's damage numbers now and then on
each other, and a teammate's fall into the void floating "100" (the device cannot tell a fall
from a blow without a new message); Knock Off's own mark on its plate's line after a slide to
the far edge; "X wins!" over empty ice when the last two went in together and the one who
stayed on longer is in the sea (places by who stayed on longer, the builder's call, kept: the
design had said the same slide shares a place); Bridge Race's bridges dark on the void by
night and names in a team's blue hard to read on the violet golden island (a stroke on names
over heads would change Bridge Wars' too: offered, not done); two fresh copies coming online
at once standing inside each other in the town (the server keeps each device's own spot from
offline, as ever, and two new ones have the same; not one of the four games').

### Checked, and not

- **Tests**: the rules', the server's and the game's suites, with tests of every stage of a
  table, ties, refunds, groups, reseating before play, disguises, a fee in flight as the
  countdown ends.
- **Hidden desktop runs, offline**: the selector, each game's table from the join to the
  town and the coins tallied exactly (1000, -50, +130, -50, +50 refunded, ...).
- **Online, two hidden copies against a server on this PC** (no database): every game's
  table from the join to the town on both, the coins on both, reseating, Play again, leaving
  and refunds, an app put away and one closed and reopened mid-game, a group led into a
  table, and Bridge Wars online. That run found three bugs where the tables meet Bridge Wars
  and groups, all fixed: Bridge Wars closed in the frame its last person left (on main
  already), leaving its bots on their device; a follower got two group lines on top of each
  other; a follower taken from Bridge Wars' results into a table kept Bridge Wars running.
  **House Builder has the first and the third too: stalled, left.**
- **The netcode's mend, online** (the second play-through, a throwaway count on the server):
  three moves marked still once a body stops, then none while it stands, where it had gone
  on at about seven a second; 12 to 14 a second walking; the other copy drew the stopped body
  where it stood, its walk eased out.
- **The deployed server** (protocol 22, 2026-11-18 04:16 UTC): this PC's own account through
  a table and back against the real database, the fee taken and given back
  (`docs/building.md`, "APK 41").
- **Not checked**: prizes paid by the real database (only a fee and its refund were), the
  "not enough coins" refusal, the 15 s "Reconnecting" give-up, Blend In with three or four
  people, Bridge Race won by holding online, and **any phone or iPhone**: APK 41 was built,
  not installed.
