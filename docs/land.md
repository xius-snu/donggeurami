# Land: the town's plots, and whose they are

The town's 48 plots, who has which, and what stands on them (`src/land.rs`).
Dated entries, newest first.

## Contents

1. 2026-11-25: the made-up owners' tastes, sixteen of them, three owners each, in the new houses and with the new things, mostly one floor; server deployed, APK 56
2. 2026-11-24: where everything is, is thrown as a channel opens (the numbers only say which plot); 48 to a channel, someone walking every plot nobody has; nobody's land in two channels; APK 53
3. 2026-11-23: all 48 plots always filled, in Hajun's order: whoever is here on the first sixteen, everyone else's land and made-up land on the rest; protocol 25, APK 52
4. 2026-11-22: what you own stands on your plot, and everyone sees it
5. Later on 2026-11-21: everyone in your channel has their sign, and you see it
6. 2026-11-21: the words are a thing in the world, a picture hung on the board; a metre and a half at the corner; the sign is solid; APK 44
7. Later on 2026-11-20: you arrive on your plot; the words slant with the board, on both its faces; more room at the corner
8. 2026-11-20: a plot for your seat, and the sign on it that says whose it is

## 2026-11-25: the made-up owners' tastes, and the new houses and things on their land; server deployed, APK 56

Hajun, that night: "also make the npc plots to use the new house types and
furnitures, keep it not to rich so its not like 10 floor buildings but
pretty diverse and different from each other. floor height should be
biased towards lower heights for the npc plots."; and "oh and make each
npc's plot have a coherent theme/personality for each one so that its not
just "diverse" randomly but its like the owner's design taste is
"diverse" so the each of the entire owner's creation should be coherent
with their personality and each npc is different from each other."

- **Sixteen tastes, three owners each** (`roundtown_net::premade::TASTES`,
  mine): the cook (a stove and a fridge where the counters were, a pan on
  the stove, a barbecue on the lawn), the bookworm (an armchair, a reading
  lamp, tea, a grandfather's clock), the gamer (a computer, its screen and
  mouse, headphones), the gardener (cupboards, flowers, plants inside and
  a garden out), the stargazer (a globe, and a telescope on the roof
  terrace), the one who never grew up (a stool, a doll, a present, a box
  of toys), the tea drinker (tea and a rocking chair), the tinkerer (tools
  on the workbench, a stool, a box and a bin), the one who loves a bath
  (the bathhouse, or a bath in the room), the night owl (an armchair, a
  lamp in and a lamp out), the traveller (a globe, a coat rack, an
  umbrella, a mailbox), the athlete (a treadmill and dumbbells), the party
  host (a glass and a burger on the table, a barbecue, a big table on the
  roof terrace),
  the minimalist (next to nothing), the office worker (a computer, a
  printer, a stapler, tissues, a bin) and the collector (a globe, a clock,
  a grandfather's clock and a safe). Each owner's colours go with their
  taste (`COATS`): the cook's warm, the gardener's greens, the tea
  drinker's rose, the stargazer's night blues. Two owners numbered one
  after the other never share a taste, and whatever a taste brings is on
  the lot of at least one of its owners (a test).
- **How it is put together** (`furnished`): each plot keeps the room it
  had, whose places were already held to the rules, and its owner's taste
  puts its own things in them: swapped in at the back and middle of what
  was there (a chair at its front, at its table), small things on the
  room's tables and counters, and what it adds at the spots each room
  leaves for it; and things on the roof of a roof terrace. A room turned
  the other way round puts each thing's middle where the other's was,
  which a thing whose origin is not its middle needs.
- **The houses**: two thirds of one floor (14 Square Houses, 12 Flat
  Houses, 8 Wide Roof Houses, with three rooms of their own: a bathhouse,
  a party room, a study), 10 of two floors, and 4 of three (the Square
  House and the Roof Terrace of three floors); none higher. A test holds
  that. A window of a house of three floors goes in its storey's wall as
  the game numbers them (`wall_of`).
- **Lighter**: lighter things where the heaviest stood (a plain chair for
  the cafe chair), and fewer street lamps on the lawns: the lots are
  189,122 triangles in all, under the 190,000 the checker allows, none
  over 5,000.
- **The checker** (`build::premade_check`) holds every lot, on its own plot
  and on plots turned other ways, to the game's rules as they now are: a
  small thing on what it stands on, the bathtub as no bed but with floor
  beside it, a chair at any of the tables, things on a roof terrace in the
  house, and what is carried on a thing not in its way.
- **Old apps**: protocol 25 still. An app before APK 55 does not know the
  new houses and things, and shows the made-up land without them (a room
  of floating furniture, some plots without their house) until it is
  updated; APK 55 and 56 show it all. The server deployed with it.
- **The server** deployed at 19:44 UTC on 2026-11-24 (`ce0188aa`, after
  one at 19:41 with a colour in it left unused), the binary before kept
  as `roundtown-server.prev-2026-11-25b` (`613d35f0`): nobody connected
  since 17:41. After: the VM's binary the same as the one built here,
  and this PC's account online, hidden, its join in the VM's journal and
  the town's clock the server's. No table changed.
- **Checked**: the made-up lots' own tests and the game's checker, as
  above; in the desktop build, hidden, nine lots looked at from above
  (Flat Houses, Square Houses, the Wide Roof House's rooms, a roof
  terrace, a house of three floors): what is in each stands on its floor
  and in nothing else, and goes together. **Not checked**: a phone;
  online, by two devices.

## 2026-11-24: where everything is, is thrown as a channel opens; 48 to a channel; nobody's land in two channels; APK 53

Hajun, the day after the entry below, in three messages as the work went:

> i think if the npc is named after a real person, it should say like
> "bunnihihi's bot". but if its a fake house owner's npc then just keep
> their name like "Sagwa" like now. also i think there should be a
> character for all 48 plot not just the first 16, and channel size for
> real players should be increased from 16 to 48, so if theres just 1 real
> player online in a channel, then there should be 47 npcs, each on their
> plot

> also for the plot numbers, just keep them as a reference number rather
> than a player joined order placement or land value order. i want it so
> every time, its random location on all the plots, whether its a real
> player or an npc or a offline real player. i just need the numbers so i
> can later tell you oh plot number ## should be fixed like blah blah, not
> necessarily for fixed placement. so every time a channel is
> "created/launched" a the plot order is gona be shuffled so the player
> who joined gona be placed in a random plot location. and their plot
> location lasts until the channel is "closed" due to no real player on
> that channel after a time. and it shuffles on another launch.

> so what matters is that there is no duplicate plots across channels (Npc
> plot duplicates are ok) and thats solveable by when u first join u first
> check if your plot is present in a certain channel due to another person
> being on a channel that "summoned" ur plot, and if its present, just go
> to that channel and take that land plot.

The names and the 47 residents are `docs/town.md`'s first entry; what an
app already out makes of it is `docs/online.md`'s.

### What there is

- **A plot's number says which plot, and nothing of who gets it.**
  `HOMES` is as it was, his numbers, for him to point at a plot by. What
  the entry below calls priority is gone: no plot is the first to be
  given, and none is kept for the worthiest land.
- **An order is thrown as a channel opens** (`roundtown_net::land::Order`:
  the 48 plots shuffled, `Order::thrown(seed)`; the server throws one with
  `fastrand` in `Towns::sit` as it opens a channel's `Estate`). Everything
  an estate keeps is by **place** in that order, and everything it is
  asked or answers is a plot by its number (`Order::plot`, `place`): the
  rules of the entry below are what they were, said of places, and where
  each place is in the town is different every time. Whoever opens the
  channel has the first place, which is any plot of the 48; real people's
  land has the places after it; **each made-up lot is its place's**
  (`premade::lot(place)`, where it was the plot's own), so that the
  made-up houses are somewhere else each time too.
- **It lasts as long as the channel does**: nobody's plot moves while
  anyone is in the channel, and the channel is shut when the last of it
  has been gone five minutes (`land::GRACE`, as before:
  `Estate::deserted`, `Towns::tick`). Whoever opens it next has an order
  thrown again. A server begun again (a deploy) is every channel opened
  again.
- **48 to a channel** (`roundtown_net::TOWN_SEATS` is `land::PLOTS`; 16
  until today). `land::FRONT`, the sixteen plots people were kept to, is
  gone: whoever comes has any place. What arrives sixteen to a ring round
  the middle of an island (`roundtown_net::arrival`, which the town's own
  people have not used since they arrive on their plots, and which Blend
  In's crowd's grid is still walked from) has a number of its own,
  `RING_SPOTS`.
- **Someone walks every plot nobody of the channel has**: `Holder::Minded`
  is said of every filled plot, and `Holder::Nobody` of none (kept in the
  list: an app that knows it is told nothing new).
- **Whoever comes takes a made-up lot's place** (`Estate::join`): the one
  their own land fills, if it fills one; otherwise the first place a
  made-up lot fills, and only with none of those left one that real land
  fills (which then goes where the entry below says); and a plot nobody
  stands on before one somebody does, whatever fills it. Until today they
  took the lowest numbered plot, which was the worthiest real land's, and
  that land moved: there being no best plot now, nobody's house is moved
  for a newcomer.
- **Nobody's land is in two channels at once** (his third message):
  - **Whoever comes into the town goes to the channel their land fills a
    plot of, and takes that plot** (`Towns::sit`, `Estate::shows`): after
    the channel they have a plot in already (back from a game), the one
    their group is in, and the one that keeps their plot for them (gone a
    moment ago); before the fullest with room.
  - **A channel opens with no land that is in another already**
    (`Estate::has`: filling a plot there, or its owner's own plot).
  - **Whoever is in one channel has nothing in any other**: someone
    seated has their land taken off every other channel's plots, and any
    plot kept for them there let go (`Estate::forget`, which says whether
    anything changed; that channel is told what stands there instead, as
    it is of any plot filled with something else); and someone moved to
    another channel with their group leaves the place's own made-up lot
    behind, not their land (`Estate::leave` without `kept`; their land
    filled the plot they left until today).
  - A made-up lot may be in every channel, as he says.
- **Offline the device throws its own** (`land::thrown`, once as the app
  opens, `fastrand`): you are on the plot of its first place
  (`lobby::spawn_players`), and each made-up owner on the plot of theirs
  (`land::offline_deeds(mine, order)`; `bag::made_up` goes by owner now,
  not by plot, `premade::of`). `RT_LAND_ORDER=numbered` on the desktop
  keeps the plots by their numbers, you on his plot 1, as the tests have
  it.
- **Still protocol 25**: nothing that is sent is anything else
  (`docs/online.md`).

### What was mine

- **Whoever comes takes a made-up lot's place first**, so that no real
  person's house moves for them; and being stood on counts before what
  fills a plot.
- **The group before the plot.** Someone whose group is in one channel
  while their land fills a plot of another goes to their group, and the
  land is taken off the other. His words have them go to the plot; with
  the plot first, someone invited into a group would be parted from it by
  where their house happened to be shown.
- **Offline is thrown too**, a town with no server being the device's own
  channel, opened as the app is.
- **On connecting, the town is another town.** The app opens offline in
  its own order and the server's is another, so as the connection is made
  (a second or two after the town is in sight, if the cover has gone by
  then) you are stood on another plot and all 47 others are filled again.
  It was so before whenever your plot was not the first; it is always so
  now. Not waiting for the network is a rule of the app (`CLAUDE.md`,
  "Playing online"), so nothing is held back for it. If it reads badly on
  a phone, the cover could stay until the server has spoken or two
  seconds have gone: his to say. **He said it that evening** ("yeah hold
  black cover until the server answers"), and since APK 54 the cover
  stays (`docs/town-hud.md`'s first entry).
- Real land that fills no plot of a channel with more of it than plots
  (`Estate::spare`) is not kept out of another channel: it would take 48
  people's furnished land to matter.

### Checked

- **The tests**: the rules' 277 (an order is every plot once, no two
  seeds alike, the first place on every plot about as often over 4,800
  throws; a channel in a thrown order asked and answering by plot;
  whoever comes takes a made-up lot's place and no real land moves, the
  one after it with someone standing there, real land only with every
  made-up one stood on; 48 to a channel, the 49th turned away; nothing of
  someone gone to another channel left behind; land withdrawn, and a kept
  plot let go, told once), the server's 128 (opened 200 times over, the
  first person on more than half the town's plots; someone comes to the
  channel their land fills a plot of though another is fuller; land in
  one channel is in none that opens after; someone seated with their
  group has their land taken off the other channel, which is told; back
  after five minutes, the same plot), the game's 577.
- **The desktop build, hidden.** Offline: on his plot 40 one time, each
  made-up owner somewhere else than their number, 47 residents; with
  `RT_LAND_ORDER=numbered`, on his plot 1. Two copies against a server on
  this PC: plots 23 and 22, the same owner on every plot on both, all 46
  residents about the same thing within a metre on both.
- **The VM**, after the deploy (06:07 UTC): this PC's own account, three
  times some minutes apart, on his plot 39, then 37, then 21, with
  `bunnihoho`'s land on 5, 45, 10 and `bunnihihi`'s on 15, 25, 19: the
  channel had shut between each (more than five minutes), and was thrown
  again. The journal clean.

### Not checked

- **On a phone**, any of it: none was attached.
- Two real accounts in a channel on the VM; more than one channel there,
  which now takes 49 people or a group moved from a full one (the tests
  alone say what happens across channels).
- APK 52 against this server: it should play on, by what is sent
  (`docs/online.md`); not run.

## 2026-11-23: all 48 plots always filled, in Hajun's order; protocol 25, APK 52

Hajun, with a picture of the town from above in the repo's root
(`plotorder.png`), each plot numbered by hand, 1 to 48:

> right now there are 1~16 players online per channel, 48 total plots, so
> 47~32 will be unfilled at the moment. i want to change it so that at all
> times, there are 48 plots filled, but the exact way it is filled is
> described below: first look at the image plotorder.png, i numbered the
> plots in priority order so 1 is priority. fill remaining plots with
> total, real ppl's (including offline), unempty plots first in order of
> higher land valuation (total sum or placed asset's buy price) calculated
> at the moment of first real user in the channel and remains fixed as
> long as 1 real player is online in that channel. then, if there are
> still unfilled plots, fill with fake plots that are premade. if >= 16
> players are online in a channel, then no NPCs. if < 16 players. for
> example, 3 players -> the 13 next plots will have an AI NPC spawn in
> each plot with that plot owner's name [...] goal = always have all 48
> plots filled, and always have plots 1~16 to have a character assigned to
> it (whether its npc or real player - also if the real player joins and
> there is an npc of them in that channel, then that real person should
> just take that npc's plot location) Real online player take foremost
> priority so their plot will replace the auto filled plots/characters
> (unless there is a real player standing on that plot, then replace the
> next number plot on the priority). Also plot placement should be server
> authoritative, and a real online player's plot should not be moved as
> long as they are online, and there should be no conflict between plot
> position. also need to think about what happens when u invite someone to
> your group. [...] also when u go to minigame, your house should remain
> there and u should count as someone who is "on that channel" - with no
> NPC spawned in ur place. tell me if there is some contradiction or
> uncertainty between what should happen for some edge case that u
> genuinely are unsure of, otherwise choose the optimal that u think.

The people who walk the plots are `docs/town.md`'s first entry; a group
brought into one channel is `docs/friends.md`'s; what is sent is
`docs/online.md`'s.

### What there is

- **His order is written down** (`roundtown_net::land::HOMES`, the middle
  of each plot, his plot 1 first). His picture is Blender's top view: the
  ball of Blender's navigation gizmo in its corner, toward the map, is the
  bare ring with a line that Blender draws for -X in a view down an axis
  (read in its source, `view3d_gizmo_navigate_type.cc`), so +X is to the
  right and +Y up, and a point (x, y) of the map in Blender is (-x, y)
  across the game's ground. His plot 1, right of the pond and just over
  the street that runs right, is at (-90, 25) in the game. His numbers go
  round in fours, each plot a quarter turn on from the one before; a test
  holds the table to that, and a test of the game's holds it to the map.
  **If plot 1 is somewhere else in the game than he meant, the whole
  table is to be turned half round, and nothing else changes.**
- **A seat of the town is a plot** (`shared/src/land.rs`, a file of its
  own now). The seat the server gives you (`Tell::Arrive`) is the number
  of your plot, 0 to 47, and you arrive in the middle of it. Until today
  it was one of sixteen, and each device worked a plot out for it
  (`land::seat_plots`, gone; `land::in_order` matches each written middle
  to the map's plot).
- **One channel's plots are an `Estate`** (`roundtown_net::land`, rules
  with no world in them, tested by themselves; the server keeps one a
  channel, `rooms::Towns`):
  - **Whoever is in the channel has one of the first sixteen**
    (`FRONT`): the lowest numbered that nobody of the channel has and
    that nobody stands on (the server looks every half second at who
    stands on which plot, `players::mind_the_town`,
    `land::plot_at`: within 17 m of its middle); or, if one of the
    sixteen is filled with their own land, that one.
  - **It is theirs until they are gone**: in the town (`Holder::Here`),
    away in a game or at home (`Away`: `Towns::away` where the games
    used to stand them up, and `sit` gives them back the plot they
    had), and for five minutes after their connection has gone
    (`land::GRACE`, `Estate::leave`, `tick`), as they left it, with
    nobody on it. Their land is read off their body as it goes
    (`players::body_gone`, an observer, so that every way a body goes
    is covered).
  - **The rest are filled** (`Lot`: whose, the name, the look, what
    stands, what is painted, as it stood when taken up): real people's
    land that has anything on it, the worthiest first (`land::worth`,
    the shop's price of each thing standing), then each plot's own
    made-up lot (`roundtown_net::premade`, below). Worked out as the
    first person comes into an empty channel (`Estate::open`), from what
    the server knows of everyone's land (`Towns::record`: read from the
    database as the server starts, `accounts::Snapshot::lands`, and kept
    up as bags change, `bag::register`), and as it is until nobody is
    left.
  - **One thing moves when someone comes**: what filled the plot they
    take goes to the first plot a made-up lot fills (`Estate::settle`),
    so that real land is always shown before made-up land, and nothing
    else shifts. With no made-up lot left to give way, it takes the
    place of the least worth shown, if it is worth more.
  - **Someone whose land has anything on it and who is gone for good**
    (past the five minutes, or moved to another channel with their
    group) leaves it filling their plot, with someone of the computer's
    on it under their name; back, they have that plot. With nothing on
    it, the plot is filled with something else.
  - **Someone of the computer's walks each of the first sixteen that
    nobody of the channel has** (`Holder::Minded`), and nobody the other
    32 (`Nobody`). Sixteen real people, and there are none.
- **The server says whose every plot is** (`Tell::Plots`, a `Deed` to
  each of the 48: its number, whose, the name, the look, and who is on
  it), to everyone in the town of a channel whenever any of it changes;
  and what stands on each as it always said what stood on anyone's
  (`bag::Tell::Theirs`, `Painted`, by the deed's `who`): to whoever sits
  down, all 47 others, and to everyone as a plot is filled with something
  else (`bag::seat_news`).
- **On a device** (`src/land.rs`): `Deeds` is whose every plot but yours
  is, as the server said, and offline every plot's own made-up owner
  (`land::offline_deeds`). A sign stands on all 48 (`holders`), what
  stands on each is stood by the code that stood your neighbours'
  (`stand::keep_theirs`, by deed now), and offline the made-up land is
  the device's own to stand (`bag::made_up`): the same land the server
  fills a plot with, so that going online changes only the plots real
  people have to do with.
- **The made-up land** (`shared/src/premade.rs`): 48 furnished plots, one
  to each plot, written by hand in tables: one of the four houses,
  standing somewhere on the plot and turned some way, a door and windows,
  a bed, a couch and more on its ground floor, on some a lamp or
  somewhere to sit outside, and its own colours. Owners go by the
  computer's names, a boy on the even plots and a girl on the odd.
  183,690 triangles in all, 4,896 on the heaviest. **A test stands every
  one of them on the town's map by the game's own rules**
  (`src/build/premade_check.rs`: the house where a house fits, doors and
  windows where their walls have room, nothing in anything else, a way on
  foot from the door to the bed and every seat, every painted part a part
  of its model): change a number there and run it.
- **A sign's words are hung on it only near the camera**
  (`land::word_the_near`: within 45 m, until 60 m): with 48 signs a
  picture each was a megabyte kept twice and two draws, for words smaller
  than a dot across the town. And **the town's stood shape is read no
  oftener than every 0.35 s** (`build::RESHAPE_REST`): at the app's
  opening a piece's model loads nearly every frame for a few seconds, and
  each reading goes through some 180,000 triangles.

### Where his words did not settle it (told to him)

- **"3 players -> the 13 next plots" and "always have plots 1~16 to have
  a character"** agree only if real people are always among the first
  sixteen. They are: someone whose own land fills a plot further out
  (nobody walks those) comes to the front like anyone, and that plot is
  filled with something else.
- **"remains fixed as long as 1 real player is online"** and someone
  leaving: their land stays on the plot they had, as above, rather than
  the plot going back to what the first ranking had there. It keeps a
  phone that only put the app away on its plot (the server's journal has
  one such account gone and back four seconds later, three times in ten
  minutes).
- **"unless there is a real player standing on that plot"** is taken both
  ways: a plot someone of the channel has is never taken, and nor is one
  someone is standing on.
- **Real land with nothing on it fills nothing** ("unempty plots first"):
  after the real land that has something come the made-up ones.

### Mine

The five minutes. That a plot is kept for someone at home as for someone
in a game. One thing moving, to the first made-up plot, rather than every
filling shifting along. A leaver's land staying. The worth of land as it
was when the channel opened, or its owner left: not kept up after. Someone
real who is in another channel can have their land fill a plot of this
one, with someone walking it under their name. 48 made-up lots, one to a
plot, and what is in each. Forty-five metres for a sign's words. A new
protocol number, below.

### What it costs

- **`PROTOCOL_VERSION` 25** (`docs/online.md`): a seat means something
  else, so APK 51 and older are told to update and stay offline, and so
  is the iPhone's build, which already was.
- **More to draw**: the made-up land is 184,000 triangles and 48 signs
  are 309,000 (one is 6,434, most of it chain), where a town of 32
  wanderers and two furnished plots was about 225,000 in all; the sixteen
  bodies there are now are 61,000 where 32 were 122,000. Only what is in
  view is drawn. **Not measured on a phone**; the desktop build, hidden,
  held its 120 frames a second with all of it (below). If a phone does
  not: the signs are the first thing to make cheaper (a plain one from
  far off), then what stands in houses across the town.

### Checked

- **Tests**: the rules' 276 (the table's order and quarter turns; an
  estate opened, come into, left, kept and let go of; real land before
  made-up, the worthiest shown; `Tell::Plots` last on the wire; every
  made-up lot sound), the server's 126 (`Towns` with estates; whoever
  sits down told every plot; a body gone keeping its plot; `Tell::Plots`
  by the names accounts go by), the game's 577 (the map's plots in his
  order, signs for every deed, offline land, the 48 lots by the game's
  own building rules).
- **The database** on the VM, on scratch copies dropped after
  (`SERVER.md`'s recipe): the new reading of everyone's land ("3 accounts
  keep a bag, 2 of them with something standing on their land").
- **The desktop build, hidden**, offline: 48 signs, 365 things standing,
  15 residents; pictures from above and from the ground looked at; frame
  times over the first 8 s (mean 9.1 ms, 8 frames over 20 ms; 35 before
  the stood shape was read less often) and settled (8.5 ms, the cap).
- **Two hidden copies against a server on this PC**: each on its own
  plot (0 and 1), each with the other's deed and 46 made-up ones, 354
  things standing on each; the one who left kept as away on the other's
  screen.
- **The VM after the deploy** (13:55 UTC, `docs/online.md`): this PC's
  own account in, hidden: the two real accounts' land filling plots 2
  and 3 by Hajun's numbers, each with someone walking it under its
  owner's name, made-up land on the rest; out again, the journal clean.

### Not checked

- **Anything on a phone.** How it runs there most of all.
- Two real accounts in one channel on the VM; someone gone for five
  minutes and back; a full channel; a second channel.
- Someone going into a game online and back (the tests have the plot
  kept and given back; no run was made).
- **Whether plot 1 is where Hajun meant it**: by the reasoning above, not
  by his eye.


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
