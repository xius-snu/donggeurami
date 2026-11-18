# The four games of 2026-11-18, and the tables they are played at

Blend In, Knock Off, Bridge Race and Last Stand, which Hajun asked for on 2026-11-18, are
each played at a **table**: one way in, one way of waiting and starting, one way of paying
and going back to the town, the same on the server and on a device offline. Each game's own
rules and computer players are in its own file here (`docs/blend-in.md`,
`docs/knock-off.md`, `docs/bridge-race.md`, `docs/last-stand.md`); this file is what they
share. Dated entries, newest first.

## Contents

1. Since 2026-11-18 four new games are on the selector, each played at a table

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
- **Not checked**: a database (fees and prizes there), the "not enough coins" refusal, the
  15 s "Reconnecting" give-up, any phone.
