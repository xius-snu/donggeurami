# Building, and every APK there has been

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## APK 36, and the server with protocol 18

In the first minutes of 2026-11-16 here, 15:07 UTC on the 15th (Hajun:
"build apk and push to git", of the dynamite at 1 amethyst with its
quickening fall, what the dead drop, and the chat's three lines, which he
had asked for an hour before with "dont rebuild apk or push yet. just
change"; `docs/bridge-wars.md` and `docs/chat-and-text.md`, their first
entries).

- **Before the deploy**, the one thing no test had: what the dead drop,
  over the wire. A server on this PC (`RT_ARENA_PURSE=40`) and one scripted
  copy of the game in a game of eight on it for five and a half minutes:
  a stick's bridge fell at the new pace (26 planks over 4.4 s), the server
  dropped a thing as one of its players was killed in the middle, the
  copy was sent it, drew it flying from the dead to where it lay (seen
  from over it), and no longer had it once someone had taken it.
- **The server**: `server/deploy.sh` at 15:07 UTC, protocol 18, with nobody
  connected since 14:16 (a person had been in two games of Bridge Wars on
  APK 35 from 13:52 to then); the binary before (deployed 13:26 UTC,
  protocol 17) kept on the VM as `roundtown-server.prev-2026-11-16`.
  After: the VM's binary the same bytes as the one built here (sha1), the
  service active, this PC's account online ("online as xd", `xd#3690
  joined`), and a scripted game on the VM with seven of the computer's
  for four minutes, in which the server dropped a thing and then four as
  two of them were killed on land, and the copy had each.
- **And again at 15:35 UTC**, with what Hajun asked for while the first
  was being built: a thing dropped lies 0.3 s before anyone can have it,
  and one of the computer's players that is by as someone dies takes
  what they dropped before it goes on. Protocol 18 still: nothing sent
  changed. Nobody had been connected but the scripted copy; the binary
  of 15:07 kept as `roundtown-server.prev-2026-11-16b`. After: the same
  bytes as the one built here, the service active, and a scripted game
  on the VM as this PC's account.
- **APK 36** (`versionCode` 36, 00:34 here, built a second time with
  those two; the first, of 00:03, left this PC no way): `cargo ndk` 1 m
  23 s, the library stripped from 189 MB to 101 with its seven entry
  points, Gradle 3 s; put over APK 35. 85 asset files, each the bytes of
  the one on disk; `assets/ui.ron`, the two maps and
  `bridgewarsgameevent.md` hashed before and after. **Not installed**: no
  phone was attached. An APK from before is told to update and stays
  offline.
- **The iPhone**: type-checked on this PC
  (`mobile/ios/check_on_windows.sh`); **no tag pushed**: the build it has,
  of `ios-1.0.2-17`, is protocol 17 and is told to update until
  `ios-1.0.2-18` is built, whose tag is Hajun's to ask for.

## APK 35, and the server with protocol 17

On 2026-11-15, late (Hajun: "build apk and push to git", of the seat that
waits in Bridge Wars, the dynamite that takes a whole bridge and the
computer's players that go out; `docs/bridge-wars.md`, its first entry).

- **The server**: `server/deploy.sh` at 13:26 UTC, protocol 17, with
  nobody connected since 09:50; the binary before (deployed 16:42 UTC on
  2026-11-14, protocol 16) kept on the VM as
  `roundtown-server.prev-2026-11-15b`. After: the VM's binary the same
  bytes as the one built here (sha1), the service active, this PC's
  account online ("online as xd", `xd#3690 joined`), and a scripted game
  on the VM (`docs/bridge-wars.md`).
- **APK 35** (`versionCode` 35, 22:30 here): `cargo ndk` 1 m 21 s, the
  library stripped from 189 MB to 101 with its seven entry points
  (`GameActivity_onCreate` and the six `Java_town_donggeurami_MainActivity_*`),
  Gradle 15 s; put over APK 34. 85 asset files, each the bytes of the one
  on disk; `assets/ui.ron` and the two maps hashed before and after.
  **Not installed**: no phone was attached.
- **The iPhone**: type-checked on this PC (`mobile/ios/check_on_windows.sh`),
  and the tag `ios-1.0.2-17` pushed on its own at `cf9edff`, the commit of
  all the above, a little after it, when Hajun said "push the ios tag"
  (the tag is what starts Codemagic's build; it was held until he asked).
  Whether it built, and how it runs on an iPhone, was not known when this
  was written: nothing on this PC can see Codemagic. The build before, of
  `ios-1.0.2-16`, is protocol 16 and is told to update.

## Only the newest APK is kept, and what the folder's size was

On 2026-11-15 (Hajun: "also this RoundTown foldersize is HUGE now. check
why. i dont rly need multiple apk files. just the latstet"). The folder was
76 GB, 72 of it `target/`, cargo's builds, of which cargo never deletes
one: every change of a dependency's version or features since September
had left what it built before behind, for each kind of build. (Sizes here
are as `du` gives them, which counts a file under two names twice, as the
finished programs in `target/` are.)

- **One APK is kept**, `donggeurami_town-release.apk`, the newest (APK 33,
  built later that night). The 29 kept beside it as
  `donggeurami_town-release.prev-<date>.apk`, 3.1 GB, are deleted, and no
  more are made: a new APK goes over the old one. Every "kept beside it
  as" below is how it was; none of those files is there. (A file of the
  APK that shipped, to compare a new one with, has to be taken out of it
  before the next is built.)
- **`target/` is 32 GB, and the folder 33** (23 and 24 on disk), and
  everything in it is of a build still in use, so nothing builds from
  nothing: the
  desktop game and its tests (`target/debug`, 27 GB: 4 of it one file,
  `donggeurami_town.lib`, the library an iPhone links, which every desktop
  build makes too, the crate being one for all three), the server and its
  tests, the server for Linux (`target/release` and
  `x86_64-unknown-linux-musl`), and the phone's library
  (`target/release-android` and `aarch64-linux-android`). Gone: what
  cargo keeps to rebuild the desktop game quickly (`debug/incremental`,
  22 GB; it is back at 4 after a few builds, and will grow again), an
  iPhone's build and a debug one for Android that nothing here makes, a
  desktop build of each release profile from September and October, and
  in every `deps` folder each file of a build no longer made (28 GB, told
  by the sixteen digits in its name, against what cargo says each build
  in use is made of, `--message-format=json`).
- **It will grow again**, by a few GB with every change to `Cargo.lock`.
  `cargo clean` takes all of it at once, and the next build of each kind
  is then from nothing, about a quarter of an hour for the desktop game.
- **A trap in listing the phone's build**: cargo-ndk keeps cargo's
  listing to itself, and `cargo ndk-env` does not print what cargo-ndk
  really sets (it leaves out `CC_aarch64-linux-android` and the rest
  whose names have hyphens, `ANDROID_ABI`, and its own linker's). Plain
  cargo run under what it prints builds `ring` and `android-activity`
  again and everything over them. What cargo-ndk sets was read off a
  `cargo-printenv` of two lines put on the `PATH` (`cargo ndk -t
  arm64-v8a -P 26 printenv`), and under exactly that cargo lists 463
  files and builds none. The phone's library was built twice into
  `target/` for this and never into `jniLibs`: the library APK 32 was made
  of, and that APK, stayed the bytes they were (checked by their hashes
  before and after) until APK 33 was built an hour on.

## Building

Desktop: `cargo run`.

The game server: `server/deploy.sh` builds it for Linux on this PC and puts it
on the Vultr VM (`SERVER.md`). `cargo build` at the root builds the game
alone; the server is `cargo build -p roundtown_server`. Never `--workspace`,
which would build the game with the server's half of replicon in it.

Android needs the `release-android` profile, a manual strip step, and Gradle
(`versionCode` in `mobile/android/app/build.gradle` is the build number: 34
in the afternoon of 2026-11-15, nothing to see or play in it: less work for
the same picture, for a cooler phone (Bevy's own systems with one worker,
the lamps' cells only while a lamp is lit, no sound output while there is
no sound, the world drawn in one pass, and on an iPhone only one of the
render world's schedules on Bevy's executor; the first entry of
`docs/rendering-and-frame-rate.md`). Protocol 16 still, nothing of
`shared/` or `server/` changed, so no deploy. Built and not installed, and
put over APK 33; 33
in the small hours of 2026-11-15: a team thrown for each game of Bridge
Wars, each team's island lined in its colour on the minimap, your name top
left in your team's colour, the chat two lines and a half with none faded
and a little lower in a game, and the car's gun off (the first entries of
`docs/bridge-wars.md`, `docs/chat-and-text.md` and `docs/vehicles.md`).
Protocol 16 still; **the server deployed with it at 16:42 UTC on
2026-11-14** (01:42 on the 15th here), `server/` alone having changed and
nothing of `shared/` (nobody had been connected since 15:39; the binary
before kept on the VM as `roundtown-server.prev-2026-11-15`; this PC's own
account came online after). Built and not installed, no phone being
attached, and put over APK 32, of which no copy is kept ("Only the newest
APK is kept", above); 32
late on the night of 2026-11-14, the work of two sessions in the one tree:
the chat's three lines that fade upward and scroll, with a stroke round
their words; a plank a tenth wider than the map's; a kill's sapphire and
amethyst; the computer's players no longer held fast, and taking a moment
at a stand ("The last three lines" in `docs/chat-and-text.md`, the first
two entries of `docs/bridge-wars.md`); and every window drawn as the
friends' panel is, with the gift and the person button turned off (the
first entries of `docs/town-hud.md`, `docs/friends.md` and
`docs/ui-style.md`). Protocol 16 still; **the server deployed with it at
14:47 UTC**, `shared/` having changed (nobody had been connected
since 13:37; the binary before kept on the VM as
`roundtown-server.prev-2026-11-14e`; this PC's own account came online
after). Built and not installed, APK 31 kept beside it as
`donggeurami_town-release.prev-2026-11-14f.apk`); 31
on the night of 2026-11-14, the chat its words alone until a line is typed,
a small keyboard button that begins one, things to say fanned out of the
big speech bubble, two people on the friends' button, and the top bar and
the chat kept clear of the camera's hole ("The chat is its words alone" in
`docs/chat-and-text.md`, "Clear of the camera's hole" in
`docs/town-hud.md`; protocol 16 still and nothing of the server's changed,
so no deploy; a sixth native method, `safeInsets`, on the stripped
library; built and not installed, APK 30 kept beside it as
`donggeurami_town-release.prev-2026-11-14e.apk`); 30
an hour after 29, swear words starred out of the chat and of messages, and
a name with one in it refused ("The chat"; protocol 16 still, the server
deployed for the one at 09:52 UTC and for the other at 10:15, the binaries
before kept on the VM as `roundtown-server.prev-2026-11-14c` and `d`;
built and not installed, APK 29 kept beside it as
`donggeurami_town-release.prev-2026-11-14d.apk`); 29
on the evening of 2026-11-14, the chat, Korean and Vietnamese in it and in
names and messages, the Leap Potion and the builder's upgrade, a drink that
shows, a wider bridge, the Bridge Builder seen by everyone, and the stick's
middle kept near its own place ("Games", "The chat", "The font is Inter";
protocol 16, the server deployed with it at 09:35 UTC that day, the binary
before kept on the VM as `roundtown-server.prev-2026-11-14b`; built and not
installed, APK 28 kept beside it as
`donggeurami_town-release.prev-2026-11-14c.apk`); 28
in the small hours of 2026-11-14, after 27, the stick's middle wherever the
finger holding it first goes down, its disc going there and back to its own place as it is
let go (Hajun: "wherever the user tap first should be the middle";
`TouchControls::stick_origin`, `JoystickBase`, `stick_corner` in
`src/lib.rs`; protocol 15 still, no server change; built and not
installed, APK 27 kept beside it as
`donggeurami_town-release.prev-2026-11-14b.apk`); 27
in the small hours of 2026-11-14, the dynamite's arch drawn up the screen
with an X where it will land, its brown fuse, no three dots and no empty
buttons, the computer's players standing their ground against whoever
keeps coming at their base ("Games"), and the friends' panel's changes of
the night before ("Friends, groups and messages"; protocol 15 still, the
server deployed for the bots at 15:45 UTC on 2026-11-13, the binary before
kept on the VM as `roundtown-server.prev-2026-11-14`); 26
on the night of 2026-11-13, every word in Inter, a tag after every name,
and the friends' panel with its requests, groups and messages ("Friends,
groups and messages", "The font is Inter"), and with it what another
session had in the tree, Bridge Wars' dynamite and the computer's players
that no longer fall for three easy tricks ("Games"; protocol 15, the
server deployed with both at 14:07 UTC that day, its tables given the
friends' columns and tables as it started; the binary before kept on the
VM as `roundtown-server.prev-2026-11-13c`, and the database as it was
before tags as `/var/backups/roundtown/before-tags-2026-11-13.dump`); 25
an hour after 24, the compass's arrow pointing out with its words clear of
it, the anchors in their teams' colours, and the computer's players
carrying up to five kinds of thing and sending Bridge Builders (protocol
14 still; the server deployed with the bots); 24
in the small hours of 2026-11-13, five buttons for things to use and
several kinds held at once (protocol 14, the server deployed with it), and
with it what two other sessions had in the tree: the computer's players'
second round, the teams' colours, the compass, and the app's opening; 23
on the night of 2026-11-12, the computer's players that play the game in
a game played offline, the line about your anchor, and what a bot takes
flying to it (protocol 13 still; the server deployed with the bots
online that night); 22
in the afternoon of 2026-11-12, the sword held straighter at the ready
(protocol 13 still, no server change); 21
at midday on 2026-11-12, dying taking everything you hold, the fireball's band
seen over the void, and a sword held on its edge and cut downward (protocol
13 still; the server deployed for the first); 20
in the small hours of 2026-11-12, Bridge Wars' two shops and everything they sell, the swords,
the second button, and games of two to four people with no bots (protocol
13, the server deployed with it); 19
late on the night of 2026-11-11, what the generators make dropping onto
its pad, the level and the count written by the sapphire and amethyst
generators, grey panels, 25 m of bridge for 4 silver, and the camera at
one height over you in Bridge Wars (protocol 12, the server deployed with
it); 18
earlier that night, Bridge Wars' generators, its shop, its
timetable and its zombies, on Hajun's map of that day, with the anchors
lit by night (protocol 11, the server deployed with it); 17
on the afternoon of 2026-11-10, the metres of bridge left written in front of you
and running out of them stopping you where you stand (protocol 10 still,
no server change); 16
in the afternoon of 2026-11-10, a fork's root and a landing's end following the
bent edge they meet, no sliver off a curve of bridge (protocol 10 still,
no server change); 15 before it that day, a bridge that is built staying
as it was built
whatever is built past it, and unfinished ends square (protocol 10 still,
no server change); 14 earlier that day, sharp turns as L corners, short
forks square, 500 m
of bridge to start and text shadows in proportion to the letters; 13
earlier that day, the bridges cut flush along the edges they leave and
meet, at any slant, and a fork off your own bridge's side (protocol 10
still, no server change); 12
on 2026-11-09, the bridges a mitred ribbon, the minimap, the metres of
bridge, the resources under the health bar, half a second past hurting on
the server, the anchors at 120 (protocol 10 still; the server deployed the
same day with its two-tick game end, the binary before kept on the VM as
`roundtown-server.prev-2026-11-09`); 11
on the night of 2026-11-08, Bridge Wars online with eight seats in teams,
respawn anchors and bridges built by walking (protocol 10); 10 that
afternoon, the fight's minor things (dead bots hidden, fleeing, regen); 9
in the small hours of 2026-11-08, Bridge Wars' fight, its bots that go for
you, running, arrivals in the islands' middles and the jump beside the
sword; 8
late on the night of 2026-11-07, the town `roundtownmap.glb` again with the
game selector, Bridge Wars on the PvP map with Hajun's waiting room, and
its own buttons (protocol 9); 7
at the end of 2026-11-07, the map's fourth version of the day, every
island furnished and every arrival at its own marker (protocol 8); 6 half
an hour before: you arrive on the furnished island at its spawn marker
(protocol 7), the punch's reach 2 m and its range fainter, and out of the
car you walk again; 5 an hour before, the map's third version of
the day, one island furnished; 4 a quarter of an hour before that, the
punch's range band, the islands at 130 m and the planked bridges; 3 an
hour before that, with the range as a ring sector; 2 that morning, the PvP
town's; 1 for every APK before) —
see the "Build gotchas" section of `RENDERING.md`, or the commands in
`mobile/android/app/build.gradle`'s closing comment. The APK carries only
what the game loads: whatever `mobile/android/app/assets-left-out.txt` names
stays out (Blender files, reference pictures, retired models), and
`cargo test` fails if that would leave out a piece the shop sells, or
either of the town's buses.

The app icon, on both phones, is Hajun's picture `mobile/appiconroundtown.png`
(since 2026-10-08). `mobile/make_icons.ps1` makes the iOS icon and Android's
adaptive icon from it; run it again after changing the picture.

iOS cannot be built on Windows at all — it needs macOS. `codemagic.yaml` rents
one per build and ships the result to TestFlight. **Read `IOS.md` before
touching anything under `mobile/ios`**; it covers the Apple-side setup, why the
`.xcodeproj` is generated rather than committed, and what breaks if the shell
scripts are checked out with CRLF line endings. A TestFlight build is made by
pushing a tag, `ios-<version>`, after bumping `CFBundleShortVersionString`
in `mobile/ios/Info.plist`; the build number is Codemagic's own. Code behind
`target_os = "ios"` cannot even be type-checked here (`ring` wants Apple's
SDK), so keep it to call sites and put the logic in code every platform
compiles and tests, as `log_file` and `map::kept_in_library` do.

**An iPhone writes its own log** (`src/log_file.rs`, since 1.0.2): `log.txt`
in the app's Documents, which the Files app shows under On My iPhone >
Donggeurami Town. Everything Bevy logged in the last launch, and any panic
with its thread. iOS keeps the app's real log where only a Mac can read it,
so this is how a black screen says why: the first `log.txt` Hajun sent
(2026-10-08) named the black screen's cause on its own, after two builds of
guessing. Ask for it before guessing.
