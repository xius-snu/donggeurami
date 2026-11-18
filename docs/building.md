# Building, and every APK there has been

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## APK 42, and the server with protocol 23

The evening of 2026-11-18, for the dozen asks Hajun sent after playing the
four games, the last "then build apk and push to git"
(`docs/minigames.md`, the evening's entry, lists them and where each is
written up). **`PROTOCOL_VERSION` 23**: APK 41, and the iPhone's
`ios-1.0.2-18`, are told to update and stay offline.

- **Before the deploy**: the three crates' tests (the game's 473, the
  rules' 252, the server's 106), after the throwaway drivers of the hidden
  runs were taken out; the whole-games tunings of Bridge Race
  (`many_races`, 200 of each size, after Hajun's export moved its
  islands) and Blend In (`many_games_of_blend_in`); the bakes of Bridge
  Race's map (new) and the town's (the same bytes); hidden runs of the
  desktop build, offline, of every ask (`docs/minigames.md`, "Checked, and
  where"). Last Stand's (`stand_many`, 20 games of each way of playing for
  one to four people) only after the deploy, before the commit: the
  stand-ins dying and fetching what they dropped, 6 to 13 waves cleared
  in the middle by how many play and how well.
- **The server**: `server/deploy.sh` at 08:53:10 UTC (17:53 here), with
  nobody connected since 06:17 UTC (the VM's journal: one person's two
  games of Last Stand that morning); the binary before (deployed 04:16
  UTC, protocol 22, sha1 `cd9ce3a8`) kept on the VM as
  `roundtown-server.prev-2026-11-18b`. After: the VM's binary the same
  bytes as the one built here (sha1 `0ceec955`), the service active, the
  database kept ("3 accounts"). This PC's own account online on it,
  hidden, from a scratch folder whose `device.id` was compared byte for
  byte with this PC's ("connected to 141.164.62.193:443", "online as xd,
  a boy"), through Bridge Wars' new fee and back, by a throwaway driver
  taken out again: 1,000 coins; into Bridge Wars, 950 in the waiting room
  (seat 2); the app sent to the background as a phone's is (the
  connection closed, the server told) and quit; the server, "seat 2's
  device is not back as the fight begins, and the seat is free again",
  "26 left before the fight, and has the fee back", and the game closed
  with only the computer in it; the database's `coin_changes` rows
  "Bridge Wars entry" -50 and "Bridge Wars refund" +50 (game 0), the
  balance 1,000; and the device, opened again, told 1,000. The prizes at a
  game's end were not met on the real database (a whole game of twenty
  minutes): the server's tests pay them.
- **The APK**: 42 (`versionCode`), at 18:01 here (09:01 UTC), from the
  main checkout. cargo ndk 2m19s, stripped from 196.5 MB to 105.7 MB with
  `GameActivity_onCreate` and the six `MainActivity` methods in it, Gradle
  20 s; 123.3 MB, put over APK 41 with no copy kept. Checked: `versionCode`
  read back with aapt; the library in it the same bytes as the stripped
  one (sha1 `23c18384`); all 90 files under `assets/` the same as the
  tree's, the 152 of the tree hashed before cargo ndk and after Gradle and
  unchanged (nothing exported meanwhile), among them the town's map with
  the plots, Bridge Race's with its islands moved, and the chest; no
  `.blend`, no `ui.ron`; four of the evening's words (the chest's, the
  red, the selector's) found in the library. Hajun's `coinfountain.glb`,
  in the tree, not committed and loaded by nothing, is in it too (160 KB):
  the APK takes every asset not left out. **Not installed: no phone was
  attached.** Nothing of the evening's work has been seen on a phone.
- **The iPhone**: no tag pushed (Hajun's to ask for; the next is
  `ios-1.0.2-19`), and not type-checked this time: nothing behind
  `target_os = "ios"` was touched.
- **Apple's mail of that day** (ITMS-90725, build 54 of 1.0.2: built with
  the iOS 26.5 SDK; from April 2027 an upload must be built with the iOS
  27 SDK, Xcode 27): a warning, nothing refused. `codemagic.yaml` asks for
  `xcode: latest`, which Codemagic moves to a new major only some while
  after its release; Xcode 27 is on its machines (`edge`, and by version).
  Pinning `xcode: 27.x` before April 2027 is the fix, after an iPhone
  build to check it on; not changed here.

## APK 41, and the server with protocol 22

2026-11-18, for the four games Hajun asked for that morning, Blend In,
Knock Off, Bridge Race and Last Stand (`docs/minigames.md` and each game's
own doc), in one message that ended "then build apk and push to git", and
then "keep going, tell me when the apk is built". **`PROTOCOL_VERSION`
22**: APK 40 and an iPhone's build of `ios-1.0.2-18` are told to update
and stay offline.

- **Before the deploy**: the three crates' tests (the rules' 246, the
  server's 105, the game's 465); the Android library built once beforehand
  to catch anything only Android compiles (none), and the iPhone's
  type-check on the merged code (`check_on_windows.sh`, clean); two
  play-throughs of the merged build, hidden, one offline and one with two
  copies against a server on this PC (`docs/minigames.md`, "Checked, and
  not"), and what they found mended and looked at again.
- **The server**: `server/deploy.sh` at 04:16 UTC (13:16 here), from the
  main checkout fast-forwarded to the merged branch, with nobody connected
  since 15:53 UTC the day before (the VM's journal: one person's game of
  Bridge Wars closing then); the binary before (deployed 2026-11-17 07:05
  UTC, protocol 21, sha1 `8e3c05c6`) kept on the VM as
  `roundtown-server.prev-2026-11-18`. After: the VM's binary the same bytes
  as the one built here (sha1 `cd9ce3a8`), the service active, the database
  kept. This PC's own account online on it, hidden, from a scratch folder
  whose `device.id` was compared byte for byte with this PC's ("connected
  to 141.164.62.193:443", "online as xd, a boy", `xd#3690 joined`), and
  through a table and back: Knock Off joined, its fee of 50 taken by the
  real database, left while it waited, the fee given back (the device told
  1000, 950, 1000; the server "26's fee of 50 is given back"; the ledger's
  two rows "Knock Off entry" -50 and "Knock Off refund" +50). Nothing of the
  four games had met a database before that.
- **The APK**: 41 (`versionCode`), at 13:26 here, **built in the merged
  worktree, not the main checkout**, so as to carry the committed town
  (`roundtownmap.glb` sha1 `fc2af54b`, as APK 40): the main checkout has
  Hajun's export of 2026-11-17 23:54 (48 flat planes of a new `plot`
  material, for something to come), uncommitted, which nothing here was
  made or checked against, Blend In's crowd's grid (`town.nav`) among it.
  cargo ndk 2m35s, stripped to 105.5 MB with `GameActivity_onCreate` and
  the six `MainActivity` methods in it, Gradle 37 s; 123.0 MB, put over APK
  40 with no copy kept. Checked: `versionCode` read back with aapt; the
  library in it the same bytes as the stripped one; all 88 files under
  `assets/` the same as the tree's; the three new maps and the sword in it,
  no `.blend`, no `ui.ron`; four of the games' words found in the library.
  **Not installed: no phone was attached.** Nothing of the four games has
  been seen on a phone.
- **The iPhone**: type-checked, **no tag pushed** (Hajun's to ask for; the
  next is `ios-1.0.2-19`).

## APK 40, and the server with protocol 21

2026-11-17, for Bridge Wars' four buses that stop at the crossings
(`docs/bridge-wars.md`, its first entry) and the app's plain black opening
that fades (`docs/town-hud.md`, its first entry). Hajun asked for those,
not for an APK or a push: the server went up because `shared/` had
changed, and the APK was built because that tells every app before it to
update. **`PROTOCOL_VERSION` 21**: APK 39, which was online that morning
(the VM's journal: `poopoo#2435` from 04:27 to 04:31 UTC), and an
iPhone's build of `ios-1.0.2-18` are told to update and stay offline.
**Committed and pushed** later that day, at Hajun's "push to git", in one
commit with the night before's work.

- **Before the deploy**: the three crates' tests (the rules' 109, the
  server's 61, the game's 320); 96 whole games of the computer's players;
  the desktop build, hidden, offline (on a crossing, and in front of a bus
  off one); and a server on this PC with one hidden copy of the game in a
  game on it, standing on a crossing.
- **The server**: `server/deploy.sh` at 07:05 UTC (16:05 here), with nobody
  connected since 04:31; the binary before (deployed 2026-11-16 14:07 UTC,
  protocol 20, sha1 `74a7717a`) kept on the VM as
  `roundtown-server.prev-2026-11-17`. After: the VM's binary the same bytes
  as the one built here (sha1 `8e3c05c6`), the service active; this PC's
  own account online on it, hidden, from a scratch folder whose `device.id`
  was checked against this PC's first ("connected to 141.164.62.193:443",
  "online as xd, a boy", `xd#3690 joined, in Town(0)`).
- **The APK**: 40 (`versionCode`), at 16:12 here: cargo ndk five minutes
  for the library, stripped to 101.6 MB with `GameActivity_onCreate` in it,
  Gradle 48 s; 118.7 MB, put over APK 39 with no copy kept. Checked:
  `versionCode` read back with aapt; the library in it the same bytes as
  the stripped one; no source newer than it; every file under `assets/`
  the same before and after the build. **Not installed: no phone was
  attached.** Nothing of the four buses or the black opening has been seen
  on a phone.
- **The iPhone**: nothing behind `target_os = "ios"` was touched; not
  type-checked for it this time, and **no tag pushed**.

## APK 39, and the server with protocol 20

On the night of 2026-11-16, for the game of twenty minutes, the anchors at
14:00 and the zombies that come a few and then more (`docs/bridge-wars.md`,
its first entry; with it in the tree, the game the computer plays alone to
be looked on at, the entry under that one). Hajun asked for the rules, not
for an APK or a push in so many words: the server went up with them
because `shared/` had changed, as it does every time, and the APK was built
because the server's going up tells every app before it to update.
**`PROTOCOL_VERSION` 20**: APK 38, which was on a phone and online that
evening (the VM's journal: two accounts in Bridge Wars from 12:00 to 12:14
UTC), and an iPhone's build of `ios-1.0.2-18` are told to update and stay
offline. **Nothing is committed**: the tree has all of it (committed with
APK 40's work on 2026-11-17).

- **Before the deploy**: the three crates' tests (the rules' 107, the
  server's 60, the game's 318); 144 whole games of the computer's players
  played out by the server's tests, from the start, from 14:40 and from
  16:50; the desktop build, hidden, a watched game from 14:40, from 17:40
  and from 19:20; and a server on this PC begun at 16:40 with one hidden
  copy of the game in a game on it, killed by a zombie the server moved.
- **The server**: `server/deploy.sh` at 14:07 UTC (23:07 here), protocol
  20, with nobody connected since 12:14; the binary before (deployed 09:32
  UTC, protocol 19, sha1 `bb11ee76`) kept on the VM as
  `roundtown-server.prev-2026-11-16e`. After: the VM's binary the same
  bytes as the one built here (sha1 `74a7717a`), the service active, UDP
  443 listening; this PC's account online on it, hidden ("connected to
  141.164.62.193:443", "online as xd, a boy", `xd#3690 joined, in
  Town(0)`).
- **A mistake of mine in that check, put right.** The first copy of the
  game I ran against the VM was given the scratch folder kept for servers
  on this PC, whose `device.id` is one it made for itself, not the folder
  with the copy of this PC's own: the VM took it for someone new and made
  an account for it, `xd#8900`, at 14:07:22. I ran the check again with
  the right folder (above), and took the account out: the service stopped
  at 14:12 UTC for a few seconds with nobody connected, the one row
  deleted from `players` by its id, tag, name and the minute it was made
  (its device and its starting coins go with it, by the tables' own
  rule), three accounts left as there were before, the service up again.
  **Which folder holds the copy is to be looked at before any copy of the
  game is pointed at the VM**: `sha1sum` of its `device.id` against
  `%APPDATA%\donggeurami_town\device.id`.
- **The APK**: 39 (`versionCode`), at 23:13 that night, after the deploy:
  cargo ndk some two and a half minutes for the library, stripped to
  101.7 MB with its seven entry points, Gradle 20 s; 118.7 MB, put over
  APK 38 with no copy kept. Checked: `versionCode` read back with aapt;
  the 85 files of its `assets/` each the same bytes as the file on disk,
  those the same before and after Gradle, and the same as went into APK
  38; the library in it the stripped one, with the words of
  `assets/ui.ron` as it stood ("ends in a draw in", "Tab: next player");
  no source newer than the library. **Not installed: no phone was
  attached.** Nothing of the new timetable, the hidden clock or the
  zombies has been seen on a phone.
- **The iPhone**: its code type-checked on this PC with all of it in
  (`bash mobile/ios/check_on_windows.sh`: no errors); **no tag pushed**.

## APK 38, and the server with protocol 19

On the evening of 2026-11-16 (Hajun: "when ur done build apk, and push to
git, but i may need to turn my laptop off midway. also when ur testing the
game on my computer dont open it as active tab but rather background tab cuz
im in class and looking at my course material", of the two buses that drive
round Bridge Wars' map and run over whoever is in front of one;
`docs/bridge-wars.md`, its first entry; and, as those were being finished,
"also use loadingscreendesign.png for when u open the game - a minimalistic
quick loading indicator", `docs/town-hud.md`, its first entry, which is the
app's own). **`PROTOCOL_VERSION` 19**: an app from before, APK 37 and an
iPhone's build of `ios-1.0.2-18`, is told to update and stays offline.

- **No game window came up for any of it**: every run of the desktop build
  was hidden, `RT_BACKGROUND=1` (new that day: `in_the_background`,
  `window_settings` and `lock_cursor` in `src/lib.rs`; `CLAUDE.md`, "Build,
  run, ship"). The window is made hidden and without the keyboard (winit
  never activates one made so, and nothing shows in the taskbar), the mouse
  is not taken, Bevy goes on updating and drawing with no window in sight,
  and `Screenshot::primary_window()` gives the frame as drawn, 1920 by
  1080 at this PC's scaling. Seen so the first time it was used: a watcher
  asking Windows which window was in front ten times a second through a
  run had Chrome there from start to end, and the game's own pictures had
  the whole town and its buttons. A hidden run has to end itself (the
  scripted module's `AppExit`, or `timeout`); posted Win32 clicks do not
  reach it.
- **Before the deploy**: the three crates' tests (the rules' 102, the
  server's 60, the game's 319); 96 whole games of the computer's players
  played out by the server's tests; the desktop build scripted offline
  (run over, flat, up; beside a bus; one of the computer's run over); and
  a server on this PC with two scripted copies of the game in one game on
  it, one run over and the other watching it lie.
- **The server**: `server/deploy.sh` at 09:32 UTC (18:32 here), protocol
  19, with nobody connected since 07:35 (that had been this PC's own
  account, checking the deploy before; the journal had nothing in the two
  hours between); the binary before (deployed 07:23 UTC, protocol 18)
  kept on the VM as `roundtown-server.prev-2026-11-16d`. After: the VM's
  binary the same bytes as the one built here (sha1 `bb11ee76`), the
  service active, UDP 443 listening; this PC's account online on it,
  hidden ("connected to 141.164.62.193:443", "online as xd, a boy",
  `xd#3690 joined, in Town(0)`); and one scripted, hidden game on the VM
  as that account with seven of the computer's, in which a bus ran it over
  ("a bus ran seat 2 over, left with 80" in the VM's journal), it lay ten
  seconds and was up. The game was left with nobody connected, and given
  up by the server a minute later.
- **The APK**: 38 (`versionCode`), at 19:48 that evening, after the
  deploy and with what says the town is loading in it: cargo ndk 1 m 28 s
  for the library, stripped to 101.6 MB with its seven entry points
  (`GameActivity_onCreate` and `MainActivity`'s six), Gradle 17 s; 118.7
  MB, put over APK 37 with no copy kept. Before it the game's tests again
  (320, one more for the bar). Checked: `versionCode` read back with aapt;
  the 85 files of its `assets/` each the same bytes as the file on disk,
  and those the same before and after Gradle; the library in it the
  stripped one, with the words of `assets/ui.ron` as it stood ("was run
  over by a bus", "Loading..."); no source newer than the library.
  **Not installed: no phone was attached** (`adb devices`). Nothing of the
  buses, of lying flat or of the bar on the cover has been seen on a
  phone.
- **The iPhone**: its code type-checked on this PC with all of it in
  (`bash mobile/ios/check_on_windows.sh`: no errors); **no tag pushed**:
  the build of `ios-1.0.2-18` is protocol 18 and is told to update until
  `ios-1.0.2-19` is built, whose tag is Hajun's to ask for.

## APK 37, and the server with the waiting room's bots

On the afternoon of 2026-11-16 (Hajun: "rebuild apk and push to git after
ur done", of the team that is drawn nowhere until the fight and the
computer's players that come into the waiting room one at a time and go
about it; `docs/bridge-wars.md`, its first entry). `PROTOCOL_VERSION` 18
still: nothing sent has changed its shape, and an app from before plays
on.

- **The server**: `server/deploy.sh` at 07:23 UTC (16:23 here), with
  nobody connected since 07:12 (Hajun's phone had been in the town for a
  minute; nobody else that day); the binary before (deployed 15:35 UTC on
  the 15th) kept on the VM as `roundtown-server.prev-2026-11-16c`. After:
  the VM's binary the same bytes as the one built here (sha1), the service
  active, UDP 443 listening, the login answering over HTTPS. **This PC's
  account was brought online on it twelve minutes later, hidden**
  (07:35:36 UTC: "connected to 141.164.62.193:443", "online as xd, a
  boy", the town's clock set by the server's to within 16 ms), by the
  other session in the tree once it had a way to run the game with no
  window (`RT_BACKGROUND=1`, below): Hajun, in class, had asked that no
  game window come up in front.
  Before the deploy the same server on this PC with one scripted copy of
  the game through the waiting room into the fight.
- **APK 37** (`versionCode` 37, 16:27 here): `cargo ndk` 2 m 25 s, the
  library stripped from 189 MB to 101 with its seven entry points, Gradle
  23 s; put over APK 36. 85 asset files, each the bytes of the one on
  disk, and the library in it the stripped one; `assets/ui.ron`, the two
  maps and `bridgewarsgameevent.md` hashed before and after. **Not
  installed**: no phone was attached.
- **The iPhone**: no tag pushed, and the type-check not run (nothing
  behind `target_os = "ios"` was touched): the build of `ios-1.0.2-18`
  plays online as APK 36 does, with the server's half of this and not the
  app's. `ios-1.0.2-19` is Hajun's to ask for.

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
  (`mobile/ios/check_on_windows.sh`), and the tag `ios-1.0.2-18` pushed on
  its own at `9de4318`, the commit of all the above, a little after it,
  when Hajun said "push the ios tag" (it was held until he asked).
  Whether it built, and how it runs on an iPhone, was not known when this
  was written: nothing on this PC can see Codemagic. The build before, of
  `ios-1.0.2-17`, is protocol 17 and is told to update.

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
