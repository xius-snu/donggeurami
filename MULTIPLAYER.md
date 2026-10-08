# Multiplayer — taking 동그라미타운 online

Written 2026-10-07, before any of it was built, from a day's research into
servers, hosting, libraries, store rules and what comparable games do. Read
this before adding a server, or anything that talks to one.

**Approved by Hajun on 2026-10-08, with Vultr, and built the same day**:
phases 1 to 3 run on a VM in Seoul. What was built, how it differs from what
is written below, and what is left are at the end (*Status*); how the server
is run is in `SERVER.md`. Engine: Bevy 0.19.1. Prices are list prices read on
2026-10-07, in US dollars before tax. They go stale: check them again before
signing up for anything.

---

## The one rule

**Pay for a server by the month, never per message, and send only what has
changed.**

Every player in a room receives every other player, so a room's traffic grows
with the square of how many are in it. Whether multiplayer is cheap or
ruinous comes down to how much each update weighs and who you pay to carry
it. The design below keeps an update to about ten bytes. It puts everyone who
moved into one packet per update and runs on a machine whose traffic comes
with its price. Done that way, the whole game costs about $10–20 a month up
to roughly ten thousand players a day (estimated, not measured).

---

## What makes multiplayer expensive

### Bandwidth is the bill

This is per player in a full 30-player town, at 15 updates a second.
It is arithmetic, not a measurement. It assumes 10 bytes per player per
update and 60 bytes of headers per packet. In each range, the lower figure
has 40% of players moving at any moment and the higher has all of them.

| Design | Each player receives | Per hour | 10k players a day, 30 min each, per month |
|---|---|---|---|
| **Compact (this plan):** one packet per update holding everyone who moved, positions as whole centimetres | 2.6–5.3 KB/s | 10–19 MB | 1.4–2.8 TB |
| **The common mistake:** one JSON message per player per update over WebSocket, 30 a second | ~140 KB/s | ~500 MB | ~75 TB |

The compact design, by room size, with everyone moving:

| Players in the room | Each player receives | One full room, all day, all month |
|---|---|---|
| 8 | 2.0 KB/s (7 MB an hour) | 41 GB |
| 16 | 3.2 KB/s (11 MB an hour) | 132 GB |
| 30 | 5.3 KB/s (19 MB an hour) | 414 GB |
| 50 | 8.3 KB/s (30 MB an hour) | 1,084 GB |

It matters on the phone too. 19 MB an hour is nothing on mobile data; 500 MB
an hour is something players notice.

### Where the money goes

- **Services that charge per message:** Ably, PubNub, Pusher, Supabase
  Realtime and Firebase. One 30-player room, every player sending 15 updates
  a second, all day, is 1.17 billion sends and 33.8 billion deliveries a month.
  That is about $87,000 a month on Ably or Supabase Realtime and $143,500 on
  PubNub, and Ably's and Pusher's rate limits would not allow it anyway.
  **Never send movement through these.**
- **The big clouds' outbound traffic.** Sending 4 TB a month from Seoul costs:
  - $512 on AWS EC2
  - $781 on Google Cloud (Premium tier)
  - about $493 on Azure
  - $10 on Vultr, where it comes with the plan
- **A server per game.** One program holds every room, House Builder's
  included. Renting a machine or a container per match (Edgegap, GameLift)
  is for games that need a machine each.
- **Positions in a database.** Positions live in the server's memory. The
  database is written only when coins or saves change.
- **Physics on the server.** Each device moves its own body; the server only
  checks that the result is plausible. That keeps the server's work per
  player tiny.

---

## The shape

```
  phone / PC (the game)                        one VM in Seoul (Vultr, $10/mo)
  ---------------------                        -------------------------------
  your body moves here, at once, as now

  login with a random device id   --HTTPS-->   login: device id -> account,
                                  <--------      hands back a connect token

  your position and facing,       --UDP--->    game server (headless Bevy, this repo)
    15 a second while they change                rooms: Town channel 1..n,
  everyone else in your room,     <--------        House Builder queues and games,
    drawn ~130 ms behind, smoothed                 homes
                                                 owns: coins, the fee, the clock,
                                                   the theme, the stars, the winner

                                               Postgres: accounts, coin history,
                                                 later homes and saved houses
```

### Who decides what

| What | Decided by | How |
|---|---|---|
| Where your body is | Your device | Moved by `move_bodies` exactly as now, so with no lag. The server checks it is plausible and passes it on |
| Where everyone else is, on your screen | Their devices, through the server | Drawn about two updates behind, smoothed between them |
| How they walk and jump | Nobody: it is read off their motion | `walk_from_motion`, as for the AIs today. A jump (about 0.7 s) arrives as about ten points |
| Who is in your room, their names and looks | The server | Sent once when someone arrives, not with every update |
| Which room you are in | The server | `lobby::send` becomes "the server moved you" |
| Coins | The server | Postgres, with a history of every change |
| House Builder: queue, fee, stages, timer, theme, stars, winner | The server | `builder.rs`'s `Stage` machine moves to the server, on its clock |
| The pieces on your plot during a game | Your device places them; the server keeps the list | A few KB a house, sent to everyone for the visits |
| Homes and saved houses (later) | The server, saved | One JSON document per player, shaped as `map::TownSave` is |

What is kept where:

- **Permanent, in Postgres:** account, name, look, coins and their history;
  later, homes and saved houses.
- **Temporary, in the server's memory:** positions, rooms, House Builder
  games and the houses being built in them. Gone when the room is.

### Rooms

A room is a `Venue` plus an instance number: Town channel 1, 2, and so on;
one House Builder game's lobby and plots; your home, with only you in it
for now. Bodies already collide with and draw only what shares their venue
(`island::show_where_i_am`). The server does the same with what it sends:
you receive only the players in your room. In a House Builder game the eight
builders share a room, but while they build each is on their own plot, and
the server sends you only those on yours.

Going home leaves your town channel. Coming back puts you in the same one if
it still has room, otherwise in the fullest one that does.

---

## What the game already has, and what has to change

Already shaped for it:

- **`lobby::Member` is flat**, the record a server would send: a look is a
  name, never an asset path.
- **Only your legs follow your stick.** Everyone else's walk comes from
  `walk_from_motion`, because "a position is all the network should have to
  carry for them" (`lobby.rs`).
- **A body without an `Intent` is a fixed collider** in `move_bodies`, which
  is what a player moved by the network has to be.
- **`map::TownSave` is one JSON document per player**, ready to be a
  database row.
- **Everything a game spawns carries `builder::InGame`**, so it goes when the
  game does.
- **The speech bubble moves over the House Builder's head with a
  `UiTransform`**, without the UI being laid out again
  (`builder::place_bubble`). Nametags can work the same way.

Has to change:

- **`android.permission.INTERNET`**. The manifest does not ask for it, and
  without it Android refuses every connection.
- **Arrival spots.** `island::arrival(seat)` has eight spots on a 6 m ring;
  a town room needs more.
- **Names.** Bevy's built-in font is plain ASCII, and no other font is
  bundled. Korean names need one.
- **The player model.** It has 3,180 triangles in 11 to 13 separate meshes
  (`limbperson.glb` 13, the AI looks 11), and in a crowd every mesh is drawn
  on its own. One or two meshes each would make crowds cheaper; that is a
  Blender change.
- **`ITSAppUsesNonExemptEncryption`** in `mobile/ios/Info.plist` is `false`,
  which its own comment says is true only while the game does no networking.
  renet encrypts its packets, so this answer has to be revisited.
- **Your seat.** `builder::YOU` is 0: offline you are always the first seat.
  Online your seat is wherever the server put you.
- **Town editing.** The town save is per player, so trees you placed would
  be seen only by you (open question).
- **The AIs.** Offline this device runs all of them; online, open question.
- **`allowBackup="false"`** in the manifest means a guest account, kept as a
  device id in app storage, is lost on reinstall. That is normal for a guest;
  linking Apple or Google sign-in later is what keeps an account.

---

## The protocol

- **Your device sends** your position, in whole centimetres, and your
  facing, as a 16-bit angle: about ten bytes. It sends them 15 times a second
  while they change and nothing while you stand still. Unreliable: a lost
  update is replaced by the next.
- **The server sends** each player one packet per update, holding everyone
  in their room who has moved since. replicon only replicates components
  that changed, the same idea as `set_if_neq` everywhere else in this game.
- **Smoothing.** Each player moved by the network keeps their last few
  updates:
  - They are drawn about 133 ms behind (two updates), between the two
    updates either side of that moment.
  - A gap of more than a few metres (a room change, a correction) is jumped
    rather than smoothed.
  - If updates are late, they carry on briefly the way they were going, then
    stop.
  - This is written into their `Transform` with `set_if_neq`, before
    `move_bodies`. Your body then bumps into them where they are drawn, and
    `walk_from_motion` reads the step.
- **The server checks plausibility.** It rejects:
  - moving faster than `MOVE_SPEED` (7 m/s), allowing for updates that
    arrive bunched together
  - rising faster than a jump (`JUMP_SPEED`, 8.5 m/s)
  - leaving `OCEAN_LIMIT`
  - being on any venue but your room's

  An implausible update is dropped and you are put back where you were.
  Walking through walls is not checked, since the server has no island
  shapes. For a town that is fine.
- **Reliable messages:**
  - arrivals and departures, with names and looks
  - room changes
  - House Builder's asks and answers: pieces placed and taken away, stars,
    coins
- **Versions.** The client's protocol version goes in with the connection,
  and an older client is told to update rather than dropped without a word.
- **Identity.** renet's `ConnectToken` carries 256 bytes of user data: the
  account id, signed with a key the login and the game server share.

---

## How many to a room

**Cost barely depends on it.** Each player's traffic grows only in step with
the room's size (16 players: 11 MB an hour; 30: 19 MB, with everyone
moving), so the difference is cents a month. What decides it is how the
town feels and what the phone can draw.

Other games, read 2026-10-07:

| Game | Players in one shared space |
|---|---|
| ZEPETO, official worlds | 16 a room (25 in a few, such as Hangang Park) |
| ZEPETO, creators' worlds | 24 a room |
| Sky: Children of the Light | 8 |
| Animal Crossing: New Horizons | 8 on one island |
| Roblox hangouts (each game picks; Roblox allows up to 200) | Bloxburg 12, Royale High 12, Dress To Impress 13, Livetopia 17, Catalog Avatar Creator 24, Adopt Me! 35, MeepCity 40 |
| Rec Room | 40 in custom rooms, which Rec Room itself lists as a performance trouble spot |
| VRChat | Up to 80, set per world |
| Club Penguin (2005–2017) | 300 a server, 80 a room |

ZEPETO is the nearest thing to this game: a cute 3D social app from Korea,
played on phones.
- **Server:** its worlds run on Colyseus over WebSocket, judging by Naver Z's
  own sample.
- **Send rate:** that sample sends a moving player's state 25 times a second.
- **Frame rate:** ZEPETO holds its frame rate to 30.
- **Memory:** it puts eight characters at about 280 MB.

On the S26 the GPU is already what limits 120 frames a second
(`RENDERING.md`). Every extra player adds 3,180 triangles in 11 to 13 meshes,
and a nametag.

**Proposal: 16 to start.** A new arrival goes to the fullest channel that
still has room, so the town feels busy rather than spread thin. A channel
picker, or joining a friend, can come later.

**Measure before settling it** (Step 0, below). If crowds turn out costly,
try these in order:
1. Merge the player model's meshes.
2. Draw only the nearest so many players.
3. Lower the cap.

---

## The library: bevy_replicon with renet

**bevy_replicon 0.44 + bevy_replicon_renet 0.20** (renet 2.0, netcode.io
over UDP). Checked on crates.io on 2026-10-07: replicon 0.44.3 (2026-10-06)
and bevy_replicon_renet 0.20.0 (2026-09-01) both require `bevy ^0.19`.

Why this one:

- **It has the shape this plan needs.** The server is in charge, and devices
  send it messages. A device sends its state as a message, the server writes
  it into that player's component, and replicon carries the change to the
  room.
- **Rooms** are replicon's visibility filters, keyed by room.
- **Only what has changed is sent.** replicon works off Bevy's change
  detection.
- **Pure Rust on the phone.**
  - netcode's encryption (ChaCha20-Poly1305) needs no C compiler.
  - The UDP path needs no TLS and no async runtime, so no extra threads. It
    fits `one_thread_per_world`.
- **Login tokens.** renet's `ConnectToken` carries the account id, so a
  separate login service can say who may join.
- **It keeps up with Bevy.**
  - replicon had releases for Bevy 0.17, 0.18 and 0.19 within 3, 0 and 5
    days of each.
  - bevy_replicon_renet took 9, 7 and 5 days.
- **The transport can be swapped** (renet2, quinnet, aeronet) without
  touching game code.

The smoothing has to be written here (bevy_replicon_snap is unmaintained):
about a hundred lines.

Risks:

- **API churn.** replicon breaks its API often (0.41 to 0.44 in ten weeks).
  Pin exact versions.
- **renet itself changes little** (three commits in six months). Even so,
  bevy_renet had Bevy 0.19 support within a day.
- **Few maintainers.** Each is essentially one person's project.
- **Bevy 0.20 is due.** Its second release candidate was out on 2026-09-28,
  and it will likely ship in October. replicon already has a `bevy-0.20`
  branch. Don't upgrade Bevy in the middle of a phase.

Runner-up: **lightyear 0.30.1** (2026-09-16, Bevy 0.19, now built on
replicon). It has smoothing, time sync and bandwidth caps built in. But
0.27, 0.29 and 0.30 each broke its API within twelve weeks, and every Bevy
upgrade waits on three other projects.

| Crate | Latest, 2026-10-07 | Bevy | Transports | Rooms | Smoothing |
|---|---|---|---|---|---|
| **bevy_replicon + bevy_replicon_renet** | 0.44.3 + 0.20.0 | 0.19 | UDP (netcode), Steam | Visibility filters | Written here |
| lightyear | 0.30.1 | 0.19 | UDP, WebTransport, WebSocket, Steam | Built in | Built in |
| renet2 + bevy_replicon_renet2 | 0.17.1 + 0.19.1 | 0.19, but replicon 0.41 | UDP, WebTransport, WebSocket, Steam | Through replicon | Written here |
| bevy_quinnet + bevy_replicon_quinnet | 0.21.0 + 0.20.0 | 0.19, but replicon 0.41 | QUIC; no login tokens; needs tokio | Through replicon | Written here |
| aeronet + aeronet_replicon | 0.21.0 | 0.19, but replicon 0.41 | WebSocket, WebTransport, Steam; no plain UDP | Through replicon | Written here |
| naia | 0.25.0 | 0.18 only | UDP, WebRTC | Built in | Not found |
| bevy_matchbox | 0.14.0 | 0.18 only | WebRTC, player to player | None | None |

Player to player (matchbox) does not scale to a town anyway: 30 phones each
sending to 29 others.

---

## Hosting

**Vultr, Seoul (`icn`), plan `vc2-1c-2gb`: 1 vCPU, 2 GB, 55 GB SSD, $10 a
month.** Confirmed in Vultr's plan API on 2026-10-07.
- **Traffic:** 2 TB comes with the plan and another 2 TB free per account,
  pooled; after that, $0.01 a GB.
- **Network:** it is a plain VM in Korea, so UDP works. Vultr's images turn
  on ufw, so open the port. IPv6 is free.

| Option | Nearest to Korea | $ a month | Traffic included, then | 400 GB a month | 4 TB a month | Verdict |
|---|---|---|---|---|---|---|
| **Vultr** | Seoul | 10 | 2 + 2 TB, $0.01/GB | $10 | $10 | **Pick** |
| Oracle Always Free | Seoul (as home region) | 0 | 10 TB | $0 | $0 | Testing only (below) |
| Linode (Akamai) | Tokyo, Osaka | 12 | 2 TB, $0.005/GB | $12 | $22 | A second region, later |
| AWS Lightsail | Seoul | 12 | 3 TB, incoming counts too; $0.13/GB | $12 | $142 | Fine under ~3 TB |
| DigitalOcean | Singapore | 12 | 2 TB, $0.01/GB | $12 | $32 | ~70 ms away |
| Hetzner | Singapore | 18.59 | 0.5 TB, ~$0.008/GB | $19 | $48 | ~70 ms away |
| AWS EC2 | Seoul | 20.65 | 100 GB, $0.126/GB | $58 | $512 | Traffic dominates |
| Google Cloud | Seoul | 20.64 | none, $0.19/GiB (Premium) | $97 | $781 | Traffic dominates |
| Azure | Korea Central | 25.03 | 100 GB, $0.12/GB | $61 | $493 | Traffic dominates |
| KT, NHN, Naver Cloud | Korea | 28–56 | ₩90–100/GB | $29–85 | $261–351 | Two to three times the price |
| Fly.io | Tokyo | ~20 | none, $0.04/GB | $36 | $180 | UDP is awkward, none over IPv6 |
| Cloudflare Durable Objects | Tokyo, Osaka or Seoul, best effort | 5 + use | outbound free | — | — | WebSocket only |

**Oracle's free tier is for testing, not for players:**
- It halved its free Arm allowance to 2 OCPU / 12 GB on 2026-06-15, without
  an announcement.
- It can reclaim a free VM whose CPU, network and memory all stay under 20%
  for seven days, which a quiet game server might do.
- "Out of capacity" errors can last days.

Round trips from Seoul:
- Tokyo: 31–36 ms
- Osaka: 21–40 ms, depending on the route
- Singapore: about 70 ms
- US West: about 125–140 ms

Your own movement never waits on the network; these only delay how other
players appear.

Also needed:
- **A domain**, about $12 a year, for the login's HTTPS certificate (phase 2).
- **Backups:** a copy of Postgres taken off the VM every night. Postgres runs
  on the same VM to start. A managed Postgres (from about $15 a month, such
  as Lightsail's in Seoul) is worth it once real players' coins and houses
  are at stake.

Hosting in Seoul also keeps players' data in Korea, which spares the privacy
policy its overseas-transfer section (below).

### What it costs

| Stage | A month |
|---|---|
| Testing, phases 1–3 | $10, plus about $1 for the domain |
| Public, up to ~10k players a day | $10–20 for the VM, plus $15–25 if the database is managed |
| ~50k players a day (7–14 TB) | Two or three VMs and $30–100 of extra traffic: about $70–160 |

These are estimates. How much CPU the server needs per player is small but
unmeasured; measure it in phase 1.

---

## Ruled out

| Option | Why not, as of 2026-10-07 |
|---|---|
| Ably, PubNub, Pusher, Supabase Realtime, Firebase Realtime Database | Charged per message. One 30-player room all day: $3,400–143,500 a month, and over their rate limits |
| SpacetimeDB (hosted, "Maincloud") | Rust end to end, with an official SDK and a Bevy 0.19 plugin. But it bills per call and per GB: about $700 a month for movement at small scale and $7,000 at moderate (estimated). WebSocket only; one hosted region. Self-hosting is allowed for one instance (BSL licence) |
| Nakama (Heroic Labs) | Everything built in: device, Apple and Google login, a coin wallet, storage, a matchmaker. But its Rust client was archived in 2024, its server code is Go, TypeScript or Lua, and its hosting is $400 a core a month |
| Photon | No Rust SDK. Its FAQ advises under 500 messages a second per room |
| Cloudflare Durable Objects | About $8–15 a busy room a month, with no server to run. But WebSocket only, and it may place a room in Tokyo or Osaka |
| AWS GameLift | Its bandwidth has been free since 2026-06-15. But about $73 a month for one c6g.large in Seoul, and no Rust server SDK |
| Edgegap | Rents a server per match: $0.00115 per vCPU-minute plus $0.10/GB, and on-demand servers stop after 24 h. Always-on starts at $280–350 a month |
| Colyseus Cloud | What ZEPETO's worlds use; from $12.50 a VM, Seoul included. But no Rust client, and the server is TypeScript |
| PlayFab | Its free tier was cut in 2026-03, to 1,000 accounts ever |
| Epic Online Services | Free, but player to player: nothing would own the coins or the clock |
| Hathora | Shut down; servers off around 2026-05 |
| Unity Multiplay | Deprecated 2026-04-01 |
| Rivet | No longer hosts game servers |

---

## The plan

Each phase ends with something to try on the phone.

### Step 0 — How many the phone can draw

No server yet.
- **Set-up:** put 16, then 24, then 32 AIs in the offline town. That means
  more seats in `lobby::offline_room` and more spots in `island::arrival`.
- **Measure:** take the GPU's work per frame with the method in
  `RENDERING.md`, cool and hot.
- **This decides** the cap, and whether the player model's meshes need
  merging.

The code is thrown away afterwards.

### Phase 1 — The town together

- **Workspace.** The game stays the root crate. Two crates join it:
  - `shared/`, the protocol: replicated components, messages, rooms and
    quantizing. Client and server must register these in the same order.
  - `server/`, a headless Bevy app (no window, no renderer) with replicon
    and renet. It is built for Linux x86-64, on the VM or under WSL; nothing
    in it needs a GPU.
- **Server:**
  - town channels of the chosen size, filling the fullest first
  - passes updates on 15 times a second, with the plausibility checks
  - sends names and looks
  - logs players, rooms and bytes sent
- **Client (`src/net.rs`):**
  - It connects in the background once the app opens, so the town still
    appears at once. Others in your channel arrive as the connection
    comes up.
  - Players come through `lobby::spawn_player`, smoothed.
  - Nametags are UI moved the way the bubble is. Names are generated, such as
    "Mochi 482", in the ASCII font. Tags are hidden off screen and past about
    25 m.
  - It reconnects after the app comes back from the background
    (`AppLifecycle`).
  - **With no server it is today's offline town, with AIs.**
- **Networking:**
  - The `INTERNET` permission.
  - IPv6: the server gets an IPv6 address with an AAAA record, and the
    client tries it, with an IPv6 socket, before the IPv4 address.
- **Security:** test builds can use renet's unsecure mode. No build with it
  goes to either store.
- **Deploy:** on the Vultr VM, as a systemd service, with its UDP port open.

**Done when:** two PCs and the phone see each other walk and jump round the
fountain through the Seoul server; the phone gets back in after going to the
background; and bytes per player have been measured against the tables above.

### Phase 2 — Accounts and permanent data

- **Login over HTTPS.**
  - On first launch the device makes a random id and keeps it in app storage.
    The login turns it into an account and hands back a connect token.
  - The name is generated.
  - renet's secure mode from here on.
- **Postgres tables:**
  - `players`: id, name, look, coins
  - `devices`
  - `coin_changes`: who, how much, why, which game, when
- **Coins change only in one transaction** with their history row, as in
  `UPDATE players SET coins = coins - 100 WHERE id = $1 AND coins >= 100`.
- **The balance moves to the server and is saved.** A new account gets
  `STARTING_BALANCE`, 1,000, once.
- **Account deletion** inside the app (Apple requires it even for guest
  accounts), and a web page for asking, for Google Play.
- **Nightly backups**, restored once to prove they work.

**Done when:** coins survive restarting both the app and the server, and
deleting an account removes it from the database.

### Phase 3 — House Builder online

- **The server runs `builder.rs`'s `Stage` machine:** queue, full, build,
  visits, results. It runs on the server's clock; devices show the time left
  from it.
- **The fee** is a coin change in the database before you join the queue.
  Play again is another.
- **The queue fills with real players.** After a wait, bots take the empty
  seats one at a time, as now. Their seats and names come from the server;
  how they are run is an open question.
- **Pieces sync as they happen.** Each piece placed or taken away goes to the
  server at once. It carries:
  - its kind, position and turn
  - for a wall piece, which wall of the house and where along it

  An `Entity` would mean nothing on another device.

  At each visit everyone gets that plot's list and builds it with `build.rs`,
  as if it had been placed there.
- **Stars are counted on the server.** One rating per player per house, never
  their own; bots rate at random.
- **Tapping the timer to skip** becomes development-only.

**Done when:** two real players and six bots play a whole game, and the coins
in the database are right afterwards.

### Phase 4 — Later

- **Homes:** one JSON document per player, `TownSave` as it is. Visiting
  someone's home is joining its room. Town editing moves there.
- **Saved houses** from House Builder.
- **Outfits and emotes.** A look is sent once when you arrive and an emote
  is one message, so neither costs anything per frame.
- **Typed names, Korean included.** These need:
  - a font with Hangul in it, which adds to the APK
  - a text box that takes Korean on phones; likely the platform's own,
    through JNI and UIKit (not looked into yet)
  - a Korean profanity filter
  - report and block
- **Apple and Google sign-in**, to keep an account across devices. Offering
  Google means offering Sign in with Apple, or an equivalent, too.
- **Joining a friend's channel**, and a channel picker.
- **A second region** (Linode Tokyo or Osaka) if players outside Korea come.

---

## Before a public online release

Researched 2026-10-07; check again before launch. This is a checklist, not
legal advice.

**Apple and Google**

- **Accounts.**
  - Guest accounts with no sign-in are fine. Apple's 4.8 (Sign in with Apple)
    applies only to third-party logins.
  - The account id is a random UUID, never a hardware identifier.
  - **Account deletion inside the app, even for guest accounts made
    automatically** (Apple). Google Play also wants a web link for asking.
- **Names and houses are user-generated content.**
  - Required (Apple 1.2): filtering, a way to report and to block, and a
    published contact.
  - App Review's rejection text also asks for action on reports within 24 h.
  - Google also wants terms accepted before anyone creates content.
  - The fixed catalogue already filters houses; a report needs a way to hide
    one.
- **Networking.**
  - **IPv6-only networks must work** (Apple 2.5.5; App Review tests on one).
    That means an AAAA record and both addresses in connect tokens; test on a
    NAT64 network.
  - **iOS may take back a suspended app's sockets.** Close on
    `AppLifecycle::WillSuspend` and reconnect on resume with a fresh token.
    The server keeps the player's place for a short while.
- **Store forms.**
  - A privacy policy, linked in both stores and in the app.
  - Google's Data safety form and Apple's App Privacy, redone.
  - Both age-rating questionnaires, redone: they ask about user content.
- **`ITSAppUsesNonExemptEncryption`** revisited, as renet encrypts.
  Standard encryption that iOS does not provide may also need a declaration
  to sell in France (unverified for this setup).

**Korea**

- **Rating.** Google and Apple are self-rating operators, so their
  questionnaires are the rating. A Steam release would need a rating from
  GRAC.
  - The rating mark must show at start-up for at least 3 s. It has to be an
    image, since the font has no Hangul.
- **The Game Industry Promotion Act** puts duties on online games: real-name
  and age checks, guardian consent for minors, a play-time display.
  - Small businesses are exempt. Whether an unregistered individual counts is
    unverified. Registering as a sole proprietor and as a game producer likely
    settles it. **Check before launch.**
- **PIPA, the privacy law.**
  - Account id, name and play data are personal data. What the game needs to
    run needs no consent.
  - It needs a privacy policy in Korean, naming a privacy officer.
  - Servers abroad need a disclosure section; Seoul hosting avoids it.
  - Breaches are reported within 72 h.
  - For under-14s, ask for age.

**Only once it is added**

- **Typed names.** The usual Rust filter, rustrict, has no Hangul at all.
  Korean lists exist (korcen; LDNOOBW's `ko`). Allow only Hangul syllables,
  ASCII letters and digits, which shuts out ㅅㅂ-style jamo.
- **Chat.** Apple's messaging questions, and report and block on every
  message.
- **Kakao, Naver or Google login.** Sign in with Apple, or an equivalent,
  as well.
- **Real money.** In-app purchase, published odds for anything random, and
  coins never cashed out.

---

## Open questions — answered 2026-10-08

1. **The cap:** 16 to a town channel, as proposed. Step 0, which would settle
   it, has not been measured: the phone was not connected that day.
2. **AIs online:** (b), Hajun's choice. Each device fills the town up to
   eight bodies with AIs of its own, one leaving for each real player who
   arrives. In House Builder the queue waits **10 seconds** for people, then
   the computer takes the empty seats one at a time, as offline (Hajun's
   choice).
3. **Bumping:** players **walk through each other** (Hajun's choice). AIs and
   walls still bump.
4. **Names:** generated, as proposed: "Mochi 482".
5. **Town editing:** off in the town while online, as proposed.
6. **Vultr:** signed up by Hajun on 2026-10-08: `vc2-1c-2gb` in Seoul, IPv4
   `141.164.62.193`, IPv6 `2401:c080:1c02:dd6:5400:6ff:fed3:eff6`.

---

## Status

As of 2026-10-08.

**Built and tried**, with two copies of the game on the Windows PC against the
server in Seoul: walking about the town together, with names over heads and
the AIs making way; a whole game of House Builder (lobby, bots after the
wait, building, every visit with the other player's house standing on their
plot, stars, results, back to the town); the login, accounts and the fee in
Postgres; deleting an account in the app, and starting over as someone new;
a nightly backup, restored.

- **Phase 1:** `shared/` (`roundtown_net`), `server/` and `src/net.rs` as
  planned, with nametags (`src/tags.rs`), the background connection, IPv6
  first, reconnecting after the background, and `INTERNET` in the manifest.
  Measured: 0.4–0.8 KB/s out per player with two connected, most of it
  renet keeping the connection alive, 0.6% of the VM's CPU and 8 MB of
  memory.
- **Phase 2:** the login over HTTPS, netcode's secure mode, the three tables,
  coins changed only in a transaction with their history, the balance saved,
  account deletion in the app, nightly backups (`SERVER.md`).
- **Phase 3:** the server runs House Builder; devices follow it. Pieces go to
  the server as they are put down, and everyone builds everyone else's for
  the visits. The tap on the time skips only where the server allows it
  (`RT_ALLOW_SKIP`, on on the VM for testing). Since 2026-10-09 it also
  pays every place its prize (Hajun's: +500 for first down to +0 for last),
  each in a transaction like the fee, and ends a game 15 s after its
  results come up.

**Where it differs from the plan:**

- **No domain.** The login's HTTPS uses the server's own certificate, pinned
  in the app by its SHA-256, on the VM's addresses. Safer than trusting any
  authority, but a new VM, or a domain, means a new app.
- **Port 443** for the game (UDP) as well as the login (TCP): networks that
  stop other UDP usually let QUIC's port through.
- **The computer's House Builder players stand where they arrive**, in the
  lobby and on each plot: the server has no islands to walk them on.
- **Every real player is blue**, the look you have offline; the computer's
  players wear the others. Outfits are for phase 4.
- **The town's arrival ring** holds sixteen: the first eight as offline, the
  next eight between them.

**Not done yet:**

- **On the phone:** Step 0 (how many the S26 can draw), and phase 1's
  "done when" — the phone among the players, and back in after the
  background. Nothing has been tried on a phone yet; the Android build
  compiles with all of it.
- **Before real players:** take `RT_ALLOW_SKIP` off the VM; a privacy policy
  and a web page for deletion requests (Google Play), which want a URL, and
  so likely the domain; the store forms redone (App Privacy, Data safety, age
  ratings); `ITSAppUsesNonExemptEncryption` revisited (`IOS.md`); a test on
  an IPv6-only network (NAT64); everything else under *Before a public online
  release*.
- **Phase 4**, all of it.

---

## Sources

Read 2026-10-07.

- **Vultr:** https://api.vultr.com/v2/plans ·
  https://docs.vultr.com/support/platform/billing/how-are-bandwidth-caps-calculated
- **AWS:** https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AWSDataTransfer/current/ap-northeast-2/index.json ·
  https://docs.aws.amazon.com/lightsail/latest/userguide/amazon-lightsail-faq-data-transfer-allowance.html ·
  https://aws.amazon.com/about-aws/whats-new/2026/06/amazon-gamelift-servers-free-network-bandwidth/
- **Google Cloud:** https://cloud.google.com/vpc/network-pricing
- **Oracle:** https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm ·
  https://www.infoq.com/news/2026/07/oracle-cloud-free-tier-limits/
- **Linode:** https://www.linode.com/pricing/
- **Round trips:** https://wondernetwork.com/pings/Seoul/Tokyo ·
  https://www.cloudping.co/
- **Crates:** https://crates.io/api/v1/crates/bevy_replicon ·
  https://crates.io/api/v1/crates/bevy_replicon_renet ·
  https://docs.rs/renetcode/2.0.0/renetcode/struct.ConnectToken.html ·
  https://github.com/cBournhonesque/lightyear/releases
- **ZEPETO:** https://support.zepeto.me/hc/en-us/articles/4403401372313--ZEPETO-World-What-is-the-maximum-capacity-for-each-room ·
  https://support.zepeto.me/hc/en-us/articles/4406369023257--Studio-World-What-is-the-maxmimum-number-of-visitors-in-a-room ·
  https://docs.zepeto.me/world-sdk-guide/optimization-guide ·
  https://github.com/naverz/zepeto-multiplay-example
- **Roblox:** https://devforum.roblox.com/t/experience-join-improvements-server-size-join-queues-and-social-slots-reservations/2294621
- **Rec Room:** https://rec.net/creator/p/player-performance
- **Per-message pricing:** https://ably.com/pricing ·
  https://www.pubnub.com/pricing/ ·
  https://supabase.com/docs/guides/platform/manage-your-usage/realtime-messages
- **SpacetimeDB:** https://spacetimedb.com/pricing ·
  https://github.com/clockworklabs/SpacetimeDB/blob/master/LICENSE.txt
- **Nakama:** https://heroiclabs.com/pricing · https://github.com/heroiclabs/nakama-rs
- **Edgegap:** https://edgegap.com/pricing
- **Hathora:** https://www.mcvuk.com/?p=229291
- **Unity Multiplay:** https://status.unity.com/info_notices/362941
- **Apple:** https://developer.apple.com/app-store/review/guidelines/ ·
  https://developer.apple.com/support/offering-account-deletion-in-your-app/ ·
  https://developer.apple.com/support/ipv6/
- **Google Play:** https://support.google.com/googleplay/android-developer/answer/13327111 ·
  https://support.google.com/googleplay/android-developer/answer/9876937
- **Korea:** https://www.law.go.kr/법령/게임산업진흥에관한법률 ·
  https://www.law.go.kr/법령/개인정보보호법
- **Profanity filtering:** https://github.com/finnbear/rustrict ·
  https://github.com/Tanat05/korcen
