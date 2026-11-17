# Last Stand

> The full account of Last Stand, the co-op game of endless waves round an
> anchor: what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. `CLAUDE.md` has the
> rules and how things stand now. The tables every one of the four games of
> 2026-11-18 is played at (the join, the fee, the board, the results) are
> not written up here: they are `server/src/tables.rs`, `server/src/fees.rs`
> and `src/table.rs`, and have an account of their own.

## Contents

1. Since 2026-11-18 Last Stand is a game people play, online and offline

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
(`cargo test -p roundtown_net stand_many -- --ignored --nocapture`; its own
report has every table): a plain player alone, landing a blow a second,
clears six or seven waves in the middle of its games, in twelve minutes,
and has the fee back (five waves) in 85 games of 100 (Hajun's "above
50%"); four together clear eleven, in nineteen.

- **A table**: people only, one to four (alone too, mine, so that nobody
  waits for a second person who is not coming; Hajun said "2~4"); 50 coins
  to join; it begins 15 s after the last person joined, or at once with
  four; a 5 s countdown; everyone is paid 10 coins for every wave the team
  cleared, up to 400 (five waves give the fee back).
- **The map** (`laststandmap.glb`, stood 0.25 m up as Bridge Wars' is): a
  stone disc 50 m across in a ring of sand 75 m across, over the void; the
  anchor's crystal on its plinth in the middle, the generators' pads either
  side, the item shop's red stand and the upgrade shop's blue one in front
  and behind. A test holds the rules' numbers to the map
  (`the_rules_numbers_for_the_map_are_hajun_s_map_s`).
- **The anchor**: 1500 health (mine; the design's 1000 fell to the first
  brutes); while it stands whoever dies is back beside it 6 s later; broken,
  the dead stay dead, and the game is over once nobody stands.
- **Waves**: the first 20 s after "Go!", then 25 s between waves to shop,
  three seconds once everyone standing has said they are ready; walkers,
  runners from the third wave, brutes from the fourth (mine: the design's
  fifth), a giant every fifth wave, spitters from the eighth, leapers from
  the twelfth; each a little stronger every wave (health x1.22, blow
  x1.09, pace +1% to +30%); at most 40 standing at once; each climbing up
  over the rim from a few places round it, going for the anchor or for
  someone, turning on whoever hits it.
- **What you have**: 100 health, back slowly; your fists, then a sword;
  your shield; things to use on your buttons, as in Bridge Wars; what you
  hold of the four minerals. Dying, you keep all of it (mine: it is played
  together, and a death already costs the wait); what you had drunk is
  over.
- **The two shops**: the item shop's three swords, three shields and six
  things to use (healing, speed, fireball, dynamite, invisibility, leap);
  the upgrade shop's six lines, the team's, each step dearer than the last:
  the generators (gold, then sapphire, then amethyst, then faster without
  end), Sharpness, Protection (five steps), Fortify Anchor, Anchor Beam,
  Lucky Loot (five steps).
- **Score**: each kill by its kind and its wave, and one for every ten
  damage; the team's is everyone's and 100 for each wave cleared.

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
`StandRound.began` is new too (mine, eight bytes, sent once): when the
fight began by the room's clock, so that a device counts the fight's own
time (what a potion's time left is by) right from the first frame of a
game it comes back into.

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
