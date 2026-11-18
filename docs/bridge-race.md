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

1. Later on the evening of 2026-11-18: bigger grants, oftener, said ahead on the plate; dynamite twice as often, bringing the whole bridge down
2. The evening of 2026-11-18: always by day, two people seated three ways, the islands where Hajun moved them
3. 2026-11-18, later that day: what a play-test found, the race made easier to follow
4. 2026-11-18, the same day: nine things a checker found, put right
5. Since 2026-11-18 Bridge Race is played at its tables, online and offline, to the golden island

## Later on the evening of 2026-11-18: bigger grants, oftener, said ahead on the plate; dynamite twice as often, bringing the whole bridge down

Hajun: "for the bridge race game, shorten the interval at which u get sth,
increase the amount of bridge you get by 2x and make it a random range
(8~16m), increase the rate at which u get a dynamite by 2x, and make the
dynamite to destroy the entire bridge connected similar to bridge wars.
and remove the unnecessary stats below the timer (red: 62m to go, blue:
55m to go) and replace that with a Next Item: 14m (bridge icon) 1
(dynamite icon)." **`PROTOCOL_VERSION` 24**, with Last Stand's chests
(`docs/last-stand.md`); the server and APK 43 have it
(`docs/building.md`).

- **A grant every 12 s, 16 until now** (mine, the number: "shorten"
  named none), **of 8 to 16 m to each team**, a whole number of metres
  thrown for each team and each grant, as often each as the others
  (`TEAM_GRANT`, `GRANT_EVERY`; 6 m every time until now). Shared between
  those of a team still in the game as before (`team_share`): 8 to 16 m
  for one against one, 4 to 8 m each two against two. "2x" by the grant:
  12 m at the middle where it was 6.
- **Thrown a grant ahead** (`Game::roll_next`, as a game begins and at
  every grant), so that it can be said: the round sends each team's next
  metres and whose next grant brings a stick (`RaceRound::next`,
  `next_sticks`, `next_of`, `stick_next`; three bytes more).
- **Dynamite twice as often by the clock** (mine, how to read "the
  rate"): a chance of 0.3 a grant every 12 s, one stick every 40 s each,
  where it was 0.2 every 16 s, one every 80 s (`DYNAMITE_ODDS`); thrown a
  grant ahead with the metres, given if there is room on the button then,
  two at the most as before.
- **A stick brings the whole bridge down** (`Game::tick_sticks`,
  `Game::crumble`): the planks its blast reaches go at once, as before,
  and the rest of every bridge they were part of after them, plank by
  plank from the hole outward, at Bridge Wars' own pace (`Nav::cascade`,
  `doom`, `crumble`, `cascade_after`: a beat, then 6 m a second quickening
  to 12); drawn on every device in the colour of a plank about to go
  until it falls. Online the server sends which and when with the blast
  (`Blasted::later`, empty until now) and every device takes each at its
  time, as in Bridge Wars; offline the device dooms them itself
  (`offline::play`). Whoever of the other team stood on a plank that fell
  fell by the thrower's doing. Until now the blast's planks alone went
  (mine then: "a hole, not the bridge").
- **The plate**: the clock, and under it "Next Item: 14m", the plank's
  picture, and, while a stick comes with your next grant and there is
  room for it, "1" and the stick's picture (`show::NextItem`,
  `NextStick`, `next_words`; the words `words.bridge_race.next_item`).
  Your share of your team's next grant, so "Next Item: 7m" two against two
  for a team's 14, or "6.5m" for its 13. Each team's bar and "62m to go"
  (or "1/1 on the island") are put away behind a switch
  (`show::SIDES_SHOWN`), not deleted.
- **What it did to a race**, two hundred whole races of each size by the
  computer (`the_computer_plays_whole_races_close_varied_and_not_too_long`):
  over at **73 s** at the median two against two and **65 s** one
  against one, where both were about two and a half minutes. Still won by
  holding the golden island 98% and 100%, each side as often as the
  other, a stick thrown in 93% and 74% of races and taking planks in 92%
  and 69% (75 planks came down in one); a knock in 66% and 40% of races
  (the test asked 80% and 50% of the slower game, and asks 60% and 30%
  now), someone knocked off the world in 91% and 66%. One race in two
  hundred, one against one, had the computer's player stand 2.5 s at the
  edge of a blast's hole mid-bridge before it thought again, and win: the
  test allows that in one race of a hundred, never more than once in it.
  If a race is over too soon now, `GRANT_EVERY` is the number to raise.
- **Tests brought along**: the grants (each the size the round said
  before, every size from 8 to 16 as often as the others over 1,200
  throws, a person's metres given back by what each grant gave), the
  sticks (three in ten, as the round said), the blast (the rest of the
  bridge doomed, the further from the hole the later, all of it gone by
  the last), the round's size, a straight bridge laid on every grant (as
  many grants as fit on the map now), the device's grant and the server's
  table, with what each told.
- **Seen hidden** (offline, enlarged, a throwaway driver taken out
  again): "Next Item: 13m", the plank's picture, and "1" and the stick's
  picture, a stick coming with the next grant; at each grant after, the
  next: 14m with a stick, then 13, 11, 9 and 11 without. The plank's
  picture is drawn bigger since the first look, where it was a row of
  dots (`NEXT_PICTURE`, reaching over and under the line without making
  it taller). **Not seen**: a bridge coming down in Bridge Race running
  (offline a person races the computer's one player, which has nothing of
  the person's to blow up when the person lays none), nor anything on a
  phone.

## The evening of 2026-11-18: always by day, two people seated three ways, the islands where Hajun moved them

Three of Hajun's messages of that evening, with the other games'
(`docs/minigames.md`, the evening's entry). **`PROTOCOL_VERSION` 23** with
the rest; the server and APK 42 have it (`docs/building.md`).

- **Always by day** (Hajun: "or bridge race game, i think it should always
  be day time cuz theres no light source"). The sky, the sun and the light
  stand at noon while you are on Bridge Race's islands, whatever the town's
  clock says, and go by the clock again as you leave (`sky::ALWAYS_DAY`,
  `sky_time`, a test). Only the sky: the clock itself, the buses' and the
  fountain's, is the town's as ever. Mine: noon, the light Hajun approved
  for the town (2026-10-13), rather than some other hour of day.
- **Holding the golden island is what wins, however you came onto it**
  (Hajun: "in bridge race, the bridge physically reaching the target
  island shouldnt matter cuz u could jump across, so u should be able to
  win and finish the game as long as u reached it and hold it"). It was so
  already: the rules count whoever stands on its land with their device
  there (`Game::on_gold`: over the golden island, their feet at its top),
  and never ask how they came, and the test of the hold puts people
  straight onto it with no bridge built
  (`a_team_wins_with_every_one_of_it_on_the_golden_island_together_for_three_seconds`).
  Nothing changed but this account. Seen hidden: offline, the town's clock
  at night, put onto the golden island's shore with no bridge near it,
  "Red is holding the golden island!", "Hold! 2", and "Red wins!" four and
  a half seconds after.
- **Two people at a table are seated three ways, a third of the time each**
  (Hajun: "for the bridge race game make it so if there is 2 real players,
  then 33/33/33 chance of 1. being on same team against 2 bots 2. being on
  different team, each with a bot teammate, 3. being a 1v1 with no bot
  player"; always the first until then). `bridge_race::Pairing`
  (`Together`, `Apart`, `Alone`), `pairing(roll)`; thrown on the table's
  dice as the computer comes, when two people are coming
  (`server/src/tables.rs`, `wait`): together, red's two seats against two
  of the computer's, as before; apart, the one in red's far seat moved to
  blue's first, `close_ranks`, the computer in the two seats left; alone,
  the table cut to two seats, one against one, as one person alone plays.
  The server logs which. Mine: a group of two is seated the same way as two
  who came apart, his words saying nothing of groups; and it is the
  server's alone, two people never being at a table offline. Three or four
  people are as they were. Tests: the third of the way each
  (`roundtown_net`), and a table of two of either arrangement
  (`roundtown_server`). Not seen with two devices.
- **Hajun's export of the afternoon moved each team's island 5 m nearer
  the other**, with everything on it: red's at (-80, -16) and blue's at
  (-80, 16), 32 m apart, 42 until then (`bridge_race::ISLANDS`, which a test
  reads off the map, found it). The way across is 61.6 m of void, 62.7
  before, and 81.6 m from an island's middle to the golden island's, 83
  before. The bots' map baked again (`RT_BAKE_NAV=1 cargo test --lib
  race_bake`, `shared/src/race.nav`), its test reading the way across off
  the rules now and not 63 m. Two hundred whole races of each size after it
  (`many_races`): one against one, the winners had gone over on their own
  bridge in 145 of 200 and jumped the last of the void in 55, so the test's
  three in four is seven in ten, with a note (and Hajun's words of the same
  evening); one race of two hundred, seed 141, had a player stop three
  times on its own bridge, 3.1 s in all, never held fast, so "never more
  than once" in every race is "more than once in one race of a hundred at
  the most, never more than three times". The map's other change, the
  golden island's colour a little redder, is his.

## 2026-11-18, later that day: what a play-test found, the race made easier to follow

A play-tester played three whole games offline in hidden runs (1920 by
1080 by day, a phone's 852 by 393 by day, and by night with an iPhone's
safe area) and found the game working and the coins right, but not yet
fun: about 85 seconds in every hundred stood at the end of your own
bridge with nothing left to lay and the other team out of sight; the
golden island's scramble hard to follow, its words wrong; blue's colours
all but gone on the island's lilac; the screen dark as you fell; the
arrow under a phone's thumb. Hajun's word for all four games stands over
what was done: "the details i didn't mention are yours to decide to make
each game polished and fun". Every call below is mine unless it says
otherwise; what was turned down says why.

### What a player sees now

- **The whole race, top right** (`src/bridge_race/map.rs`, new): a
  minimap where Bridge Wars has its own, in its look: the three islands
  from above, red's and blue's lined in their colours and the golden
  island in its gold, both teams' bridges drawn again as they change
  (holes and all), and a dot for everyone in the race in their team's
  colour from "Go!" on, yours bigger with a white rim. Until now the
  other team's bridge first came into sight about 140 seconds into a
  race of about 150, at the screen's edge. Bridge Wars' minimap shows you
  and your teammate alone (Hajun, for Bridge Wars: "not show the other
  players"); this one shows the other team too, a race being run in the
  open and where they stand being how far they have got.
- **When the next metres come**: a ring round the rim of the plank
  button that fills, the way the clock goes, over the sixteen seconds to
  the next grant, drawn again in 32 steps (`show::next_grant`,
  `count_to_grant`). Offline it goes by the game's own clock; online by
  this device's count from the round that brought the last grant.
- **"Jump!"** in gold over the gap between you and the golden island,
  while you stand within 6.5 m of its shore with too little bridge to lay
  across (`show::jump_hint`). A body runs 10 m a second in the race and a
  jump keeps it in the air 0.77 s (`JUMP_SPEED`, `GRAVITY`), 7.7 m at a
  run, which clears 6.5 m with a metre in hand; and a grant of one
  against one leaves you at most 6 m short. The computer's players jumped
  such gaps and nothing told a person they could. It stands up the screen
  off the line of your bridge, and only while you stand on something.
  (At first 5.5 m, over the gap's middle, and in the air too: the first
  scripted run waited out a last gap of 5.7 m it could have jumped, and
  the next wrote "Jump!" over the "0m" in front of you and kept it up as
  it jumped.)
- **"Build one bridge together!"** across the upper half three seconds
  after "Go!", to a team of two (people, or one with the computer): two
  each laying their own bridge go half as fast as two laying one.
- **The golden island's scramble**:
  - Each holding team's "Hold! 2" in a place of its own when both hold
    (one "Hold!" went from red to blue and back as either got further).
  - "Hold broken!" over the island in the team's colour as its hold goes,
    whether it was knocked off or jumped (`ON_GOLD`: a jump takes you off
    the island's ground, as the rules have had it since G1; now it is
    said), rising from a little down the screen from the island's middle
    (begun where "Hold!" stands, red's was written over blue's "Hold! 3"
    as blue held on), and taken down with the lines as the race ends (one
    ran into "Blue wins!").
  - "X is holding the golden island!" is in the chat the first time each
    team holds it, and across the upper half then, and again only while
    you are more than 24 m from the island; the line comes down as that
    hold is broken. One scramble put up seventeen of them, and one stayed
    up after its hold had gone.
  - A dark band under the ring's arcs, a little wider than them
    (`colours.bridge_race.ring_edge`), so that each team's colour has an
    edge on the lilac.
  - The end's caption says who held it: "Your team held the golden
    island!" or "Blue held the golden island" (it said "The other team got
    there first" when red had got there first and blue won the
    scramble).
- **Stroked words** (`show::stroked`): "Hold!", "Hold broken!", the
  arrow's metres, "Jump!", "+3m" and the words across the middle of the
  screen have a thin dark stroke, four copies a hair off toward each
  corner in `colours.chat.stroke`, as your name top left has in its
  team's colour. Blue's "Hold! 2" on the lilac measured 1.04 to 1, red's
  1.19, gold on the day's sky 1.17; Hajun's way, his colours kept, is to
  stroke them.
- **"Go!" and the end are the race's own** (`show::banner`): where and
  as big as `hud::announce` puts Bridge Wars', fading the same, with the
  stroke, and a team's name in its colour ("You are on Red", "Red
  wins!"). The end is up two seconds and goes as the table's results come
  up (at three, under the see-through window, it showed through).
- **The camera does not follow you into the void** (`watch_over`, by
  `crate::Lookout`, which until now only Bridge Wars' watching used):
  falling, it stops 0.3 m under the land's top over where you are, so
  that you see the bridge you went off and whoever knocked you, and stays
  there while you are dead; and as the race is won on the golden island
  it comes over the island in 1.2 s, to the winners and the full ring. The
  play-tester was knocked off at the end and saw only the dark void, then
  "Blue wins!" over their own island.
- **The arrow to the golden island** keeps inside the safe area (an
  iPhone's island stood over it 15 to 37 points from the edge), stands on
  the right edge over the buttons round the jump button rather than among
  them under the thumb (or along the bottom between the stick and them,
  the island being more down than to the right), never higher than 32
  shares of the short side, under the plate and the minimap, goes as soon
  as any of the island is on the screen (its gold words sat on the
  island's gold band), and writes its metres inward of it.
- **The plate**: each team's bar has a sliver of its colour from the
  start (both were the same grey at 0 m), its end rounded as its track
  is, and its words at 2.8 shares of the short side (2.3 was 9 points on a
  phone).
- **"+3m"** begins 0.4 m up the screen from you and rises 1.4 m (it rose
  3.4 m, into the line across the upper half that the same grant's bridge
  across had put up).
- **A stick lost in the void is said in the chat**: "Mochi's dynamite fell
  into the void." (`Feed::Lost`, `Event::Lost` with its thrower; online
  the server tells every connected device, as it does the rest of the
  feed; the round's size is unchanged). It fell out of sight as a red
  speck, and a miss looked like nothing at all.

### Turned down, and why

- **Smaller grants more often** (the play-tester's 1.5 m every 4 s, "still
  every interval"): the kit's bridge, as it runs out, fixes the plank in
  your hands at least 1.05 m ahead of you whatever is left
  (`bridge::lay`, its `AHEAD`), once at every run-out; at 1.5 m a grant
  (0.75 m each two against two, under a plank's length) a person would lay
  up to 70 to 140 per cent more than granted and the computer none, and
  the server's cap (`SPARE_EACH`, 1.5 m a grant) would come to more than
  the grants themselves. G1 tuned whole games of the computer on the
  sixteen seconds. The ring round the plank button answers the "when" of
  it instead.
- **Things to pick up for metres**, as the genre's Bridge Race has bricks:
  a new rule, where Hajun's is that the metres come by the clock ("Every
  interval you are given more bridge meters"). His to decide.
- **Dynamite thrown further**, or given later: its reach is the kit's
  throw (`utility`'s drag) and Bridge Wars' numbers, not this game's file;
  when it comes is G1's odds, which the computer's whole games were tuned
  on. A stick held early keeps for later.
- **A score that separates players**: the spec names it ("score word:
  metres built"), and in a team of two it does say who laid the bridge.
  The end's words no longer show through the results.
- **Something to find yourself by at night** (the bridge is 1.18 to 1 on
  the night's void, your head 1.26): what gives light is `lamps::SOURCES`,
  the planks are `bridge`'s, and a race only by day is Hajun's call. The
  minimap shows where you are by night as by day.
- **Jumping on the golden island keeps your hold**: the rules have said
  "not in the air" since G1 (`ON_GOLD`); it is said now instead.

### Not this game's files (for the orchestrator)

- Names over heads are `arena::spawn_tag`'s: blue's on the lilac is all
  but unread; a stroke there, as the race's own words have now.
- The plank button at 0 m still looks pressable, with a white "0m", while
  the metres in front of you are red (`hud`).
- The game selector's card shows the fee, not what a win pays (`games`).
- The chat's lines run under the results' window (`chat`, `table`).
- With the waits skipped (`RT_ALLOW_SKIP`, the VM's), the table's "Bridge
  Race" across the screen is still up as "Go!" comes (`table`).
- The map is Hajun's: each team's island has Bridge Wars' red and blue
  shop stands and pad, which do nothing here (a blue stand on red's
  island), and the "golden" island is violet (`goldenisland`, 0.49, 0.15,
  0.76) under the words and the arrow's gold.

### Checked, and where

- **Unit tests**, all three suites through `rtcargo.sh race`: the rules'
  145 (5 ignored; two of them extended: a stick lost said in the chat,
  whose it was, and what is sent of it, its size); the server's 88 (4
  ignored); the game's 389, eleven of them new: the arrow's room on a
  phone and a desktop,
  over the buttons, between the stick and them along the bottom, under
  the plate, inside an iPhone's safe area; the arrow gone once any of the
  island is in sight; "Jump!" only within a jump with too little bridge,
  on your feet, and a running jump clearing what it is said from; the
  ring round the plank button filled the way the clock goes on its rim;
  stroked words as four copies under the words, clear to begin with; the
  minimap's picture (north up, the three islands lined in their colours,
  the void clear) and a plank and a dot where they stand; the end's
  words as pieces, a team's name in its colour, held by the other team
  named; a stick lost said in the chat; a hold said across the upper half
  the first time, its line taken down as it is broken, "Hold broken!"
  said, a hold again said only from afar; a team of two told to build one
  bridge after "Go!" and a team of one not; the camera falling, dead,
  over the winners, and given back with the race.
- **In the desktop build, hidden** (`RT_BACKGROUND=1`), by a throwaway
  taken out again (kept at `RoundTown-wt/scratch-race/polish/kept/`),
  every picture looked at, those of small things enlarged (in
  `RoundTown-wt/scratch-race/polish/`):
  - Offline, 1920 by 1080, by day (`desk_day/`), a whole game from the
    town's selector: the waiting room with the minimap and nobody on it;
    "Go!" over "You are on Red", red in red, stroked over the blue stand;
    each team's sliver on the plate; off the back of the island and the
    camera held at the land as you fell and lay dead ("Back on your island
    in 1.5" over the island's edge, not the dark); the ring round the
    plank button empty at a grant and half full eight seconds on; "+6m"
    clear of the upper half; the arrow over the buttons with "73m"
    inward; "Hana's dynamite fell into the void." in the chat after a
    stick thrown wide; both bridges growing on the minimap; a scramble on
    the golden island: "Hold! 2" in red and "Hold! 3" in blue side by side
    over the ring's dark band, "Hold broken!" again and again, every
    first hold once in the chat; the camera coming over the island to
    "Blue held the golden island" over "Blue wins!", blue in blue; the
    results with no end words through the window; back in the town at 950.
    Seen there and put right after it: "Hold broken!" began where
    "Hold!" stands, over the other team's (now from down the screen), and
    one ran into "Blue wins!" (now taken down for the end); and "Jump!"
    from 5.5 m let a gap of 5.7 go by (now 6.5).
  - Offline, a phone's 852 by 393, by night, an iPhone's safe area of 59
    points either side (`phone_night/`): the arrow inside the safe area
    and over the buttons; the minimap moved in with the bar; by night,
    your dot on it where your bridge ends, the bridge itself all but
    unseen; "Jump!" 6.4 m from the shore with 3 m left, the jump, and the
    hold won before either bridge was across ("Red wins!" over "Your team
    held the golden island!", the full ring, by night); the results; back
    in the town at 1,060. Seen there and put right after it: "Jump!" stood
    on the "0m" in front of you, and stayed up in the air.
  - Offline, the phone's size by day (`phone_day/`), the last of it as
    committed: "Jump!" up the screen clear of "0m", the jump, a jump on the
    golden island while holding it and "Hold broken!" in red rising from
    under the middle, the hold won after, "Your team held the golden
    island!" over "Red wins!" with nothing left rising, 1,060 after.
  - Against a server on this PC (port 5483), two hidden copies at 1280 by
    720 (`online_1v1/`, `online/`), each its own `APPDATA`. The first time
    they came to two tables, one against one each: on one the race was
    skipped ("Red wins!" over "Time's up!", paid 110); on the other the
    copy built, was told "Jump!" (now up the screen, clear of "0m", gone in
    the air), jumped, held the golden island and won (paid 110), its two
    sticks that missed said in the chat through the server and the one
    that hit holed the computer's bridge; the ring round the plank button
    filled by the device's own count; the minimap showed the server's
    computer player. The second time both sat at one table, red together
    against the computer's two: "Build one bridge together!" on both
    after "Go!", the two building one bridge, the minimap with both teams
    on it; skipped 50 s in, "Red wins!" over "Time's up!" on both, the
    camera left over you for a time-up, paid 110 each. Every time both
    back in the town at 1,060 with nothing of the race left, the logs
    clean.
- **Not checked**: anything on a phone or an iPhone; a table of three;
  the time limit run out (a skip calls it); a draw; a team gone; the
  scripted player knocked off the golden island by a punch (its breaks
  were the computer's and its own jumps and walks); the online copies by
  night.

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
