# Playing online

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

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
