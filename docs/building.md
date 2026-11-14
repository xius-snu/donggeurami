# Building, and every APK there has been

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Building

Desktop: `cargo run`.

The game server: `server/deploy.sh` builds it for Linux on this PC and puts it
on the Vultr VM (`SERVER.md`). `cargo build` at the root builds the game
alone; the server is `cargo build -p roundtown_server`. Never `--workspace`,
which would build the game with the server's half of replicon in it.

Android needs the `release-android` profile, a manual strip step, and Gradle
(`versionCode` in `mobile/android/app/build.gradle` is the build number: 30
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
