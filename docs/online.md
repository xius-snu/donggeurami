# Playing online

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Still protocol 25: 48 to a channel, an order thrown as it opens, nobody's land in two

2026-11-24 (`docs/land.md`'s first entry has the rules). **`PROTOCOL_VERSION`
is 25 still**: nothing that is sent is anything else, and APK 52 plays on.

- **What changed is the server's alone, and which numbers it says.** A
  seat in `Tell::Arrive` and `Seated` was already any of the 48 plots to
  an app; it is any of them now for whoever comes, where it was one of
  the first sixteen. A `Deed`'s `Holder` was already `Minded` or
  `Nobody`; it is `Minded` of every filled plot now. A channel holds 48
  (`TOWN_SEATS`, which nothing sent is sized by: `Tell::Channel` is a
  list).
- **What APK 52 makes of it**: it is given a plot anywhere in the town and
  stands on it; it walks someone on every plot nobody has (it walks
  whatever is `Minded`), 47 with one person in the channel, under the
  owners' plain names, not "X's bot" (the name is the device's to make,
  `words.land.bot`), and visiting as often as fifteen did; what its
  residents are about is no longer what a newer app's are at the same
  moment, each device walking its own. Nothing else differs. Reasoned
  from what is sent; not run.
- **Where a channel's people sit** (`rooms::Towns::sit`): the channel they
  have a plot in already; their group's, with room; the one keeping
  their plot for them; **the one their land fills a plot of**; the
  fullest with room; a new one. And whoever sits has nothing of theirs
  left in any other channel: that channel is stirred, and told what
  stands on the plot instead.
- **A second channel** now opens only with 48 in the first, or for a
  group moved from a full one.
- **The opening's cover waits for the server's word on the town**, since
  APK 54 later that day (Hajun: "yeah hold black cover until the server
  answers"; `docs/town-hud.md`'s first entry): while the server is being
  tried for the first time or is connected (`Link::hopeful`), and no
  longer than two seconds once everything else is there. The one place
  the app waits for the network; with none it opens as it did.
- **Deployed** at 06:07:22 UTC on 2026-11-24, nobody having been
  connected since 04:23:56 (`bunnihoho`, on APK 52); the binary before it
  kept as `roundtown-server.prev-2026-11-24` (`1b7e5524`: plots in his
  order, sixteen to a channel). After: the VM's binary the one built here
  (`d6f499a8`), "2 of 3 accounts with a bag have something standing on
  their land", this PC's own account in, hidden, three times some minutes
  apart, on another plot each time with the two real accounts' land on
  other plots each time and their bots under "bunnihihi's bot" and
  "bunnihoho's bot", and out; the journal clean.

## Protocol 25: a seat of the town is a plot, and whose every plot is

2026-11-23 (`docs/land.md` has what the plots are). **`PROTOCOL_VERSION`
is 25**: APK 51 and older, and the iPhone's build, are told to update and
stay offline; APK 52 is the first that plays online.

- **Why a new number, where the last four changes had none.** A seat of
  the town meant one of sixteen, for which each device worked a plot out;
  it is the plot's own number now, 0 to 47 in Hajun's order
  (`land::HOMES`), in `Tell::Arrive` and in `Seated`. An app from before
  would stand everyone's land on another plot than the server has them
  on. Something that means something else is a new number, as the rule
  has it.
- **`Tell::Plots(Vec<land::Deed>)`**, the last of `Tell` (number 21; a
  test holds it): whose each of the 48 plots of your channel is, its
  number, the owner's id (an account's, or a made-up owner's,
  `land::PREMADE` and the plot's number), the name on its sign, the look
  of whoever walks it, and who is on it (`Holder`: here, away, minded by
  the computer, nobody). All 48 each time, about a kilobyte and a half,
  to everyone in the town of a channel that asked of its channel
  (`Ask::Channel`), with `Tell::Channel` and on the same stir: someone
  comes or goes, into a game or back, takes another name, or a plot is
  filled with something else. `Tell::Channel` is as it was, of whoever is
  in the town now, each seat a plot.
- **What stands on each plot** comes as it did for anyone's
  (`bag::Tell::Theirs`, `Painted`, by `who`), after the deeds: to whoever
  sits down, of all 47 others, land that fills a plot and the land of
  someone away in a game among them, each whether or not anything stands
  on it (a device may have been told otherwise of the same owner); and
  to everyone in the town as a plot is filled with something else.
- **`social::Outcome::Gathered`**, the last of its kind (17): you have
  been moved to your group's channel (`docs/friends.md`).
- **The server keeps everyone's land** with anything standing on it in
  memory (`rooms::Towns::lots`), read as it starts
  (`SELECT id, name, look, bag FROM players WHERE bag IS NOT NULL`) and
  kept up from the bags of whoever is here. No table changed.
- **Who is in a channel** is whoever has a plot of it: in the town, in a
  game, at home, or gone within five minutes. The line of numbers the
  server logs each minute counts them so.
- **Deployed** at 13:55:16 UTC on 2026-11-23, nobody having been
  connected since 09:29:51; the binary before it kept as
  `roundtown-server.prev-2026-11-23` (`b4f39103`: who rests, protocol
  24). Before: the database's new reading run on scratch copies on the
  VM, dropped after. After: the VM's binary the one built here
  (`1b7e5524`), "2 of 3 accounts with a bag have something standing on
  their land", this PC's own account in, hidden, on plot 1 with the two
  real accounts' land on plots 2 and 3 and made-up land on the rest, and
  out, the journal clean.

## Who sits and who lies: three more at the end

2026-11-22, late (`docs/town.md` has what sitting and lying are).
`PROTOCOL_VERSION` is 24 still, and APK 45 to 50 play on, and see
everyone on their feet.

- **`Ask::Resting`** (number 29) and **`Ask::Rest(Option<Pose>)`** (30)
  at the end of `Ask`, after `Bag`; **`Tell::Resting(Vec<Posed>)`** at
  the end of `Tell`; and the two things they carry, `Pose` (`Sit`,
  `Lounge`, `Lie`) and `Posed { who, pose }`. A test of the rules holds
  the numbers.
- **The server** (`players`): a body has its pose (`Body::pose`), set by
  `Rest` in the town alone and let go of as it changes room. A device
  that says `Resting` is told from then on (`Posing`), with the channel
  as that is told (`tell_channels`, on the same stir): everyone in it
  who rests, all of them each time, usually nobody.
- **Where someone rests is where their body is**, said by their moves as
  ever; nothing of the chair is said. Sitting down puts a body up to 3.7
  m across and a metre up in one move, inside what the server lets one
  be (`BURST`, 4 m).
- A device says `Resting` once as it comes online, and `Rest` again then
  if it is resting (`rest::say_so_on_coming_online`), a phone's
  connection going whenever its app is put away.
- **Deployed** at 13:09:04 UTC on 2026-11-22, nobody having been
  connected since 10:39; the binary before it kept as
  `roundtown-server.prev-2026-11-22d` (paint taken off, nobody resting).
  After: the VM's binary the one built here (`b4f39103`), this PC's own
  account in, hidden, and out, the journal clean. It owns no chair: who
  rests was seen told between two copies on this PC.

## Paint taken off: one more variant at the end

2026-11-22, night (`docs/bag.md`, the entry marked "night").
`PROTOCOL_VERSION` is 24 still, and APK 45 to 49 play on.

- **`bag::Ask::Unpaint { id, part }`** at the end of `bag::Ask`, after
  `Paint` (the eighth, number 7 on the wire; a test of the rules holds
  the order). The server takes that part's paint off
  (`bag::Paints::clear`), keeps the document, and tells `Painted` to the
  device and to everyone in its channel who asked for colours, as it does
  of paint; of a part that had none it says nothing.
- An app from before never says it. APK 50 against the server from
  before would have had it passed over: the server was deployed first.
- **Deployed** at 09:37:47 UTC on 2026-11-22, nobody being connected (the
  last had left at 08:03:58); the binary before it kept as
  `roundtown-server.prev-2026-11-22c`, which is the one with paint and
  without this. Checked after with this PC's own account, hidden: a lamp
  stood, painted, cleared and taken back, and its bag in the database as
  it was.

## Paint: three more variants at the end, told only to who asked

2026-11-22, evening (`docs/bag.md` has what paint is). `PROTOCOL_VERSION`
is 24 still, and APK 45 to 48 play on.

- **`bag::Ask::Colours`** and **`bag::Ask::Paint { id, part, colour }`**
  at the end of `bag::Ask`, and **`bag::Tell::Painted { who, asked,
  paints }`** at the end of `bag::Tell`. `bag::Bag` and `bag::Standing`
  are as they were on the wire: what is painted is kept beside the bag
  (`bag::Paints`) and said in a message of its own, because a field
  added to either would be read wrongly by an app from before.
- **Only to a device that asked.** A device that knows of colours says
  `Colours` once, after `Watch`, as it comes online (`server/src/bag.rs`,
  `Colourers`). It is then told `Painted` after each `Yours` (its own
  plot, with the same `asked`: it passes over one that answers fewer of
  its asks than it has sent, as it does a bag) and after each `Theirs`,
  and whenever someone in its channel paints. An app from before never
  asks and is told what it always was.
- **Kept** in `players.bag`, the bag's own document, as one more list in
  it (`Kept::paints`, read as empty where there is none): no change to
  the table. The binary before this one reads such a document and keeps
  no colours: rolled back, it would write them away.
- **Deployed** at 07:08:18 UTC on 2026-11-22, nobody being connected; the
  binary before it is `roundtown-server.prev-2026-11-22b`. This PC's own
  account connected after, and its bag was read.

## Bags: asked and told with no new protocol, and kept with the account

2026-11-22 (`docs/bag.md` has what a bag is). What goes over the wire for
it, and why `PROTOCOL_VERSION` is 24 still.

- **`Ask::Bag(bag::Ask)` and `Tell::Bag(bag::Tell)`**, one new variant at
  the END of each, as `Ask::Channel` and `Tell::Channel` were the day
  before (the entry below says why that keeps the apps out playing: a
  variant is written as its number, and a side sent one it does not know
  drops that one message). A device says `bag::Ask::Watch` as it comes
  online, and the server speaks of bags only to a device that has
  (`bag::Watchers`). A test of the rules holds both to being the last
  variant on the wire.
- **What a device asks**: `Buy(kind)`, `Sell(kind)`, `Put { id, kind, at }`
  (stand this on my plot, or move what stands under that number),
  `Take { id }`. A kind is its name (`bag::Thing::kind`), never a number
  that a thing added later could shift.
- **Every answer is the whole bag** (`bag::Tell::Yours { asked, bag }`):
  once for each thing asked, done or not, in order, with how many of the
  connection's asks have been answered by then. The device counts what it
  has sent, and passes over a bag that answers fewer (`bag::Bag::heard`):
  it has done something since, and the answer to that is coming. Standing
  and taking back are done on the device at once; a purchase or a sale
  waits for its answer.
- **What stands on a plot** goes to the channel as
  `bag::Tell::Theirs { who, stands }`, the whole plot each time, 100
  things at the most, a few kilobytes: to whoever sits down in the channel
  of everyone there with anything standing, and to everyone there of the
  newcomer's and of every change (`server/src/bag.rs`, `seat_news`;
  `Towns::take_sat` says who has just sat down). Whoever has nothing
  standing is not spoken of, and a device lets go of what it was told of
  anyone no longer in its channel.
- **Kept with the account**: `players.gems` and `players.bag`
  (`SERVER.md`, "The database"); a purchase or a sale is one transaction
  with the coins (`Bank::deal`, answered as `accounts::Dealt`), and a
  person's asks wait behind one that is at the database
  (`bag::Holds::waiting`).
- **Deployed** at 15:59 UTC on 2026-11-21 (00:59 on the 22nd here), from a
  tree not yet committed, after the new statements had been run on scratch
  databases on the VM. Before: nobody connected since 14:20 UTC; the
  database dumped (`before-bags-2026-11-22.dump`); the binary running kept
  as `roundtown-server.prev-2026-11-22` (the one of 17:15 UTC on the
  20th). After: the VM's binary the same bytes as the build here, and this
  PC's own account online on it, hidden, its bag told, a lamp stood on its
  plot and taken back, both kept in the database, the journal clean.

## A channel of the town is told who is in it, with no new protocol

Since 2026-11-21, for the signs on everyone's plots (`docs/land.md`) and the
car everyone else now sees (`docs/vehicles.md`): until then the server told
a device its own seat in the town and nobody else's, and nothing of anyone's
car.

- **What is said.** `Ask::Channel`: "tell me of my channel", sent once with
  the hello. `Ask::Drive(bool)`: into your own car, or out. `Tell::Channel`:
  everyone seated in your channel of the town, each `Seated { who, seat,
  driving }`, the whole of it each time (sixteen at the most), sent to
  every device in a channel that has asked, at the end of any tick in
  which someone sat down in it, stood up, or got into or out of their car
  (`Towns::stir`, `players::tell_channels`, in `PostUpdate`, after the
  games' own seating). On a device it is `lobby::Channel`, emptied
  offline.
- **Why `PROTOCOL_VERSION` is still 24**, and APK 43 and 44 play on. The
  three are new variants at the END of `Ask` and of `Tell`. replicon's hash
  is the names and order of the types registered, which are as they were;
  postcard writes a variant as its number, so every variant from before
  reads as it did; and a side sent a variant it does not know fails to read
  that one message and drops it (read in bevy_replicon 0.44.3's source that
  day: `client_message.rs`, "ignoring message ... from client ... that
  failed to deserialize", at debug, on the server; `server_message.rs`,
  the same as an error in the log, on a device). So a server from before
  ignores the two asks, and **the server speaks of the channel only to a
  device that asked** (`players::Watching`), so an app from before is
  never sent what it cannot read.
- **When that may be done again**: for something new that either side can
  do without, added at the end, the server saying it only to those who
  asked. Never for a field added to something already sent, nor for
  anything that now means something else: those are a new protocol
  number, as the rule has it.
- **A leap is by how fast, not only how far** (`net::leaps`; the account is
  in `docs/vehicles.md`): a body is taken to have been put somewhere,
  rather than to have gone there, when it is shown further off than 3 m
  and than the fastest the server lets a body go would have taken it
  since its last update. That bound is one number now on both sides,
  `roundtown_net::cars::MOST_SPEED` (52 m/s; the server's own until then).
- **Deployed** at 17:15 UTC on 2026-11-20 (02:15 on the 21st here), from a
  tree not yet committed, with the town's buses' new speed in `shared/`
  too. Before: nobody connected since 16:20 UTC; the binary running kept
  as `roundtown-server.prev-2026-11-21` (the one of 13:31 UTC that day).
  After: the VM's binary the same bytes as the build here, and this PC's
  own account online on it, hidden, on its plot with its sign, into its
  car and out, the journal clean, three accounts as before.
- **Checked** with two hidden copies of the desktop build against a server
  on this PC, and by the three suites (490, 252, 108). **Not checked**: an
  app from before against this server on a phone (APK 44 sends neither
  ask, and what the server does for it is what it did); two real accounts
  on the VM.

## Playing online

Since 2026-10-08 the game plays online, against a server in Seoul (a $10
Vultr VM). `MULTIPLAYER.md` is the plan and the why, `SERVER.md` how the
server is run. The rest of this file holds online too: being online changes
where things come from, not what they are.

- **The app never waits for the network.** It opens on the offline town as
  it always has; `src/net.rs` connects in the background, IPv6 first, then
  IPv4. Once the server has let you in, whoever else is in your channel of
  the town (16 to a channel) walks into it, and one of this device's AIs
  leaves for each, keeping the town at 32 bodies
  (`lobby::fill_the_town`; Hajun's choice, 2026-10-08; eight until
  2026-11-01). Each device runs its
  own AIs, so two players see them in different places. With no server to be
  had it is the offline game exactly, and `net` keeps trying, less and less
  often. Never mid-game offline: a game of House Builder started offline is
  finished offline.
- **Three crates.** The game is the root; `shared/` (`roundtown_net`) is
  everything the two sides say, registered in one order by
  `roundtown_net::protocol`; `server/` is the server, a headless Bevy app
  with replicon and renet over UDP 443. **Deploy the server with every change
  to `shared/`**: an app whose protocol differs from the server's is told "A
  new version is out" and stays offline. replicon's protocol hash is only the
  names of the types registered and their order (read in its source,
  2026-10-08), so it misses a field added to one or a field meaning
  something new: bump `PROTOCOL_VERSION` for those, or an old app connects
  and breaks. A change that leaves what is sent as it was keeps the apps
  already on phones playing online, as the prizes did.
- **Your body is yours.** It is moved here, at once, as offline; where it is
  goes to the server 15 times a second while it moves and not at all while
  it stands (`net::tell_where_i_am`). The server checks a move could have been
  made (no faster than a run, no higher than a jump) and passes it on.
  Everyone else is drawn two updates behind, smoothed between them
  (`net::smooth`). **Players walk through each other** (Hajun's choice): a
  remote body has no collider; AIs and walls still bump. Riding one of the
  town's buses, where you are goes as a place on the bus, in its own space
  (`Spot::aboard`, since `PROTOCOL_VERSION` 2), and everyone else draws you
  on the bus as they see it (below, "The town's buses").
- **One clock for the whole town** (`src/clock.rs`, since 2026-10-24,
  `PROTOCOL_VERSION` 3). Everything that goes by the clock goes by the
  town's clock: the buses round the roads, the fountain's jet and the day.
  Offline it is the device's own time of day; online it is the server's.
  Until then each device went by its own, and phones keep theirs to the
  network's only roughly: on Hajun's two phones the cars were in different
  places, and a rider, drawn on the car as each phone had it, jumped onto it
  on the other phone and back off it (Hajun: "i want the car location to be
  synced too as well as the fountain ... in a very network cost efficient
  way"). As a device comes online it asks the server the time eight times,
  0.1 s apart, and then once a minute (`net::ask_the_time`, `WhatTime`); the
  server answers at once (`server/src/clock.rs`, `TheTime`, sent
  independent of replication, which would hold it to the next tick); and
  the answer that came back quickest sets the clock, put forward by half
  the time it took (`TownClock::heard`). Between answers it counts on the
  device's steady clock (`Instant`), which nothing puts forward or back; the
  app going to the background goes back to the time of day, put out by as
  much as the server's was ahead of it, until it is heard again. That is
  one number, and 8 bytes there and 16 back an answer. The first answer
  logs how far the device's own clock was out. Checked with two
  copies on this PC and a server on it, one copy's clock put 5 s out
  (`RT_CLOCK_OFF=5`): it logged "-5.000 s on from this device's time of day,
  to within 8 ms", and both copies' town clocks were within 3.2 ms of the
  machine's, their buses within a centimetre of each other. Not yet tried on
  phones.
- **Getting in.** The device makes up a secret the first time it runs
  (`device.id`, beside `town.json`), sends it over HTTPS to the login on the
  VM (`src/login.rs`, TCP 443, the server's own certificate pinned by its
  SHA-256, `login::PINNED`: no domain), and gets a signed connect token. A new
  device is a new guest account in Postgres with a generated name ("Mochi
  482"), since 2026-11-13 a tag after it that nobody of that name has
  ("Friends, groups and messages", above), 1,000 coins, and the blue look;
  the name and the character chosen
  on signing up replace those as soon as it is let in ("The first time, you
  sign up"). **The VM's addresses and certificate are compiled into the app**
  (`net::SERVERS`, `login::PINNED`): a new VM means a new app.
- **House Builder online is the server's** (`server/src/builder.rs`): the
  queue (10 s for people, then the computer one seat at a time; Hajun's
  choice), the stages and their clock, the fee and every prize (each in a
  Postgres transaction, with its line in `coin_changes`), the stars, the
  places, and the results' 15 s, after which it sends whoever is left back
  to the town. The device follows its `Round` and `Seats`
  (`builder::follow_round`) and is moved by it. The places are not sent:
  both sides work them out from the stars with `roundtown_net::standings`,
  the lot being the game's `Round::theme`, so that the apps already out
  could keep playing online when the prizes came in. Pieces put down go to the
  server as they are (`build::report_pieces`), and everyone else's are built
  from what it says (`build::raise_shadows`, `build::Shadow`), on their plots,
  for the visits. The computer's players stand where they arrive: the server
  has no islands to walk them on. A dropped connection mid-game waits 15 s to
  come back (`builder::RECONNECT_WAIT`) before the game is given up.
- **Names over heads** (`src/tags.rs`): everyone else's online, AIs
  included, and never your own (since the evening of 2026-10-31, Hajun:
  "remove the in world player name for the player's own character"; until
  then yours was always over you, online or not: it is on its plate top
  left). Each is a line of UI text laid out once (3.6% of the short side
  of the screen; 6% for an hour, which Hajun found "too big, especially
  your own name": "reduce by more than 50%", so 2.8, then, with their own
  gone, "slightly bigger", 3.2, and on 2026-11-01 "slightly increase"
  again), and scaled down by
  how far its head is from the camera, as words standing in the world would
  be: full size 8 m off and nearer, half at 16, a tenth at 80 (Hajun,
  2026-10-31: "like a in the world text rather than a on screen text so
  further away the text look small naturally and closer look larger"; until
  then one size, 3.2%, at every distance). Read
  out to 75 m (25 until then; Hajun: "like 3x"), which is further than the
  scaling leaves one readable. **Town
  editing is off** while online (`map::hands_off_online`): nobody else would
  see it. **Your account** is under Settings (below, "Settings"), where
  Delete account is, which Apple asks for even for accounts nobody signed
  up for. Until 2026-10-31 it was a round button beside the balance, online
  only (`src/account.rs`, now `src/settings.rs`).
- **Testing on desktop.** Two copies side by side need their own `APPDATA`
  (their own `device.id`, so their own accounts) and `RT_FREE_CURSOR=1`, so
  that neither takes the mouse. `RT_SERVER=<address>[,<address>]` points a
  desktop build at another server, `RT_SERVER=off` keeps it offline, and in a
  debug build `RT_UNSECURE=1` skips the login, for a server run without
  `RT_KEY`. `RT_TIME_OF_DAY=night` (or `day`, `sunset`, `sunrise`, or
  seconds into the twenty minutes) stops the day at one time, and
  `RT_CAR_AT=<seconds>` the town's buses at one point of their loop.
  `RT_ARENA_FROM=<seconds>` begins a game of Bridge Wars' fight that far
  into its timetable (the server for its games, a desktop build for one
  played offline), and `RT_ARENA_PURSE=<count>` with that many of each
  thing in every person's purse, for trying the shops out.
  `RT_CLOCK_OFF=<seconds>` puts a copy's own clock that far out, for
  trying the town's clock out on one machine, whose clock both copies
  share. `RT_INTRO=off` leaves out the opening's cover, which otherwise
  holds every press for the first second or two of a run (above, "The app
  opens under a cover"). A
  server on this PC for two copies to meet on: `cargo build -p
  roundtown_server`, then `target/debug/roundtown-server.exe` with
  `RT_PORT=5443` and no `RT_KEY`, and the copies with
  `RT_SERVER=127.0.0.1:5443 RT_UNSECURE=1`. The VM allows
  the tap on the time that skips House Builder's waits
  (`RT_ALLOW_SKIP=1`): take that out of `/etc/roundtown/env` before real
  players come.
