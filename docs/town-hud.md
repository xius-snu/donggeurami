# The opening, the buttons round the screen, and the settings

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Clear of the camera's hole; two people for friends; the chat's own button

Since the night of 2026-11-14. Hajun, having played on the phone: "Right
now I checked, and in my mobile Samsung phone, the chat box is covered
slightly, uh, and the input field too, by the, uh, small camera hole. Uh,
so I guess I need the UI elements to use the OS safe area, so that the
left margin is just for the UI elements is shifted to the safe area so
it's not covered. So right now in the UI, right side, there's 2 chat icons.
I like the top chat icon design better, so use that chat icon for the
bottom chat icon. And instead of the top chat icon, which was for friends,
uh, use a, uh, person, 2-person icon for the friend UI button." The rest of
what they said that night is the chat's ("The chat is its words alone",
in `docs/chat-and-text.md`). `src/safe.rs`, `src/hud.rs`,
`MainActivity.java`. APK 31; no change to `shared/`.

- **The camera's hole** (`src/safe.rs`). The game is drawn under it
  (`MainActivity.applyImmersive`: the window lies in the cutout along the
  short edges), and held on its side the phone has it half way down the
  left edge, where the chat is, or the right one, turned the other way up
  (`sensorLandscape`). `MainActivity.tellSafeInsets` reads the cutout's
  safe insets, left and right, off the decor view's root insets, as the
  window takes the focus and every 300 ms with the clipboard's poll, and
  hands them to the game by a native method (`safeInsets(int, int)`, the
  sixth; numbers only, as the keyboard's height is). The game brings them
  to logical pixels (`safe::SafeArea`).
- **What moves** (`safe::KeepsClear`): what stands 4 shares of the short
  side in from the sides (`hud::EDGE`): the bar across the top, in the
  town and in Bridge Wars (your name and what is under it on the left;
  the three buttons, or the minimap, on the right), and the chat. Each is
  given a margin that brings it 1.5 shares inside the safe area's edge on
  the side the hole is on, and none on a screen with no hole. **My
  reading**: "shifted to the safe area so it's not covered" is moved just
  clear of the hole, not the whole screen's worth of margins added to the
  hole's depth. **What does not move**: the stick and the buttons round
  the jump button (7 and 6 parts in a hundred of the width in, where a
  hole ends 3 or 4 in, by my reckoning from a status bar's height, not
  measured), and whatever comes up in the middle of the screen.
- **`RT_SAFE_AREA=<left>,<right>`**, in logical pixels, makes a hole up on
  desktop.
- **An iPhone is not asked**, and its island still lies over what is
  beside it: winit 0.30 has the safe area, but nothing behind
  `target_os = "ios"` can be checked on this PC.
- **The friends' button is two people** (`icons/friends.png`, Material's
  `group`, the friends' panel's own tab picture), and **the big bubble
  over the jump button is the one with three dots** that the friends'
  button had (`icons/social.png`, Font Awesome's comment-dots). Material's
  bubble (`icons/chat.png`) is on nothing now, and still in the APK.
- **The chat's own small button**, a keyboard, over the bubble and a
  little to the right, in the town and in Bridge Wars alike, and the
  bubble's fan of things to say: in `docs/chat-and-text.md`.
- **Checked.** A unit test of how far in a thing is moved for a hole
  (none with no hole, just clear of one, never back out). The desktop
  build, scripted, with a hole made up 70 pixels deep on the left and then
  on the right, in the town and in a game of Bridge Wars: the name, the
  balance (or the health and the four counts) and the chat moved in on the
  left; the three buttons, or the minimap, on the right; the stick and the
  buttons round the jump button where they were, and nothing run into
  anything. The new pictures, enlarged.
- **Not checked: anything on a phone.** The Java that reads the cutout has
  never run: that Android hands the hole over the way it is read, that it
  is told again when the phone is turned the other way up, and how deep
  the S26's hole is, are all still to be seen on it. **If the chat is
  still under the hole on the phone, read the phone's log first**: the
  game says what it was handed to begin with and each time it changes
  (`adb logcat`, a line with "the safe area begins", in logical pixels and
  in the screen's own), and the activity says once if it could not hand
  it over at all (`adb logcat -s Donggeurami`). Nothing handed over, or
  nothing but noughts, is the Java's side; numbers that look right and a
  chat still covered is `safe::CLEAR`, or the hole being deeper than its
  safe inset says.

## The app opens under a cover, and you drop into the town

Since 2026-11-13 (Hajun: "right now, when i open the game, there is no
smooth introduction animation. i need a short 0.5~1s animation in the
beginning. also when i open the game, some part of the map is not loaded
yet so it looks bad. only show the map after it gets loaded. im thinking
maybe it can be falling from an extreme height, and when it lands, maybe a
short sit on but fall landing animation, and then get up/standup and then u
can play"; of the four ways I put to them, a drop onto your bottom, a drop
into a squat, a circle opening and a dive from above the town, they chose
the circle with the squat: "ok C+B try it"), in `src/intro.rs`.

- **The cover.** From the first frame the screen is black with the town's
  name on it in white (`colours.intro.cover` and `title`, and
  `words.welcome`, in `assets/ui.ron`), over everything on the screen but
  the sign-up's pages (its layers are 25 to 27, theirs 30), and the town
  loads under it. It takes every press (`TakesPress`) and holds
  `hud::DialogUp`, so you stand still under it as under a dialog. While it
  is up the camera clears each frame to the cover's colour too: the first
  frames may be drawn before the GPU has what draws the cover itself
  (its pipeline is compiled in the background like any other), and would
  then be the bare sky; reasoned from Bevy's source, not seen. Android's
  own window is black before that
  (`windowBackground` in `themes.xml`, since APK 24).
- **It opens once the town is there to be seen** (`Loading::there`, `Wait`):
  the town's shape read (`Islands`), every model in the world spawned (every
  `WorldAssetRoot` with its instance ready: the islands, the buses,
  everyone's bodies), the buses' way round worked out (`car::Planned`), and
  your own limbs bound; and then only once the GPU has had nothing left to
  compile for eight frames running (`Compiling`: a system in the render
  world, `count_compiling`, counts `PipelineCache::waiting_pipelines` after
  each frame; Bevy compiles each material's pipeline in the background and
  leaves out whatever is not ready, which is one way a town comes up a
  piece at a time), and not on the heels of a frame that took over 50 ms.
  With anything never coming it opens anyway after 10 s (`MOST_COVERED`),
  and the log says what was missing. It logs one line as it opens, with
  when each thing was there: on this PC, a dev build, covered for 1.5 s,
  the town and its models at 0.3 s and the buses' way at 1.4 (it takes a
  second to work out here). The first time the app is run the sign-up's
  pages stand over it, the wait is theirs, and it opens as they go, 0.13 s
  after Save here.
- **The opening is 0.85 s** (`UP_AT`; `pose`, `hole`): the cover opens as a
  circle about where you stand, 동그라미, reaching every corner in 0.55 s
  (`OPEN_SECS`; one node, all border, with a round hole, as Bridge Wars'
  blindfold is, `gear::blindfold`); you drop in from 11 m up, over the top
  of the screen, arms up in a V and knees a little bent, faster as you
  come, and land at 0.4 s (`FALL_SECS`, `FALL_FROM`); you sink into a deep
  squat in 0.07 s, arms out at your sides, leaning in, drawn 14% shorter
  and 8% wider, stay down 0.06 s, and spring up over 0.32 s, through
  a little taller than you are; a white ring goes out over the ground from
  your feet as you land, to 2 m in 0.45 s, fading (`Dust`, drawn as the
  punch's range is, `range::look`, and as brightly as the sky lights the
  ground, `Daylight::unlit`, so that it does not shine at night; it waits
  at your feet under the cover, too small and faint to see, so that the
  GPU has it ready); and the camera dips 0.12 m as you land. From 0.85 s
  you can move, and the buttons are yours.
- **Nothing of the fall is your body's.** Your body stands where it
  arrived all along; what drops is its model, the node your limbs hang
  under, hung that much higher over it and put back as it was at the end
  (`Model`, `hang`). So it is no move to the island, to anyone else, or to
  the server, and nobody online sees it. The squat is your limbs'
  (`Landing` in `src/lib.rs`, a component as `Carrying` and `Drinking`
  are, `landing_offset`, read by `animate_walk` before anything else): a
  thigh turned 1.05 rad forward at the deepest and its shin bent back by
  twice that, an arm turned up to 2.2 rad from hanging; and the model
  comes down by what its bent legs take off its height, less what its
  heels come up (`Landing::sink`, from how high its hips are, measured off
  its thigh bones as the cover opens), so its feet stay on the ground.
- **The town's name stays where it was on the cover**, over the town now,
  for 2.2 s and fades in 0.8 (`TITLE_STAYS`, `TITLE_OUT`): it is the
  welcome, in white. `hud::welcome` leaves that one to it
  (`Intro::welcomes`); back from a game, the gold welcome fades in as
  before.
- **For trying it on desktop**: `RT_INTRO_SLOW=<times>` plays the opening
  that many times slower; `RT_INTRO=off` leaves all of it out, cover and
  held input and all, with the gold welcome as before, for a scripted run
  that presses or looks from its first frame.
- **What is mine**: every number; the white name, staying on in place of
  the gold welcome; the ring of dust and the dip; that the buttons come
  into sight with the circle rather than after you stand; and **that the
  cover waits for the buses**, where I had told Hajun I would let them
  drive in: it is the last 1.1 s of the 1.5 s the cover is up here, and a
  bus coming onto the ring road a second after the town is first seen is
  the kind of thing that was asked to go. Taking `car::Planned` out of
  `Loading::there` is all it takes to let them drive in instead.
- **Checked on 2026-11-13, scripted, in the desktop build** (a throwaway
  `src/intro/scratch.rs`, removed, that had the game save its own
  screenshots through the opening): at a fifth of its speed, from the
  game's camera and from the side close up, by day and by night: the
  cover from the first frame, the circle from nothing to past the
  corners, the drop, the squat with the feet on the ground and the knees
  forward, the ring, the name fading; at its own speed, three runs, every
  frame of the opening 16.2 to 17.6 ms on a 60 Hz screen, and the stick
  let go at 0.85 s; and the first run, with an empty `APPDATA`, through
  the sign-up's pages by keys and a tap, the cover opening as they went.
  Unit-tested: the poses on the models (`limbs_of`), the fall, the
  circle, the dust, and the wait. **Not by hand, and not on a phone**: how
  long a phone is covered, whether its first frames are black rather than
  sky, and the Adreno drawing the ring are all unseen. APK 24, built that
  night by the session that made Bridge Wars' five item slots, is the
  first with it, and was not installed, no phone being attached; a phone's
  log says how long it was covered and for what ("the opening: covered
  for ... s").

## The app opens in Round Town

(Since 2026-11-13 the app opens under a cover, and you drop into the town
as it lifts: the section above. What this one says of the first frame, and
of the town's name fading in as the app opens, is how it was until then.)

There is no menu. Once you have signed up (above), the app opens straight
into the town (`roundtownmap.glb`, from 2026-10-30, `circlemap1.glb`
before; for the day of 2026-11-07 the PvP map, `pvptownmap.glb`, which is
Bridge Wars' since that night, above). Everyone stands on the path round
its pond, facing the fountain in the middle of it, and the town's name,
"Donggeurami Town", fades in and out over the first few seconds. You can
walk, jump and look round from the first frame; on desktop the cursor is
taken for looking round at startup. The games, House Builder and Bridge
Wars, are chosen from the gamepad button (above, "Games").

- **The buttons round the screen are laid out as Hajun sketched them**
  (`assets/UIbuttonlayout.jpg`, 2026-10-31, kept out of the APK;
  `hud::show_top_bar`): top left your name on a plate (the buttons' grey, a
  little more solid, corners rounded a little), beside it a person button
  and the settings' gear ("Settings", below), and under it your balance,
  the coin and the number on nothing; down the right edge a gift, a
  speech bubble (social chat) and a gamepad (a leaderboard, a city at
  first, until Hajun said which it was, stood between the gift and a group
  of people until 2026-11-01, when Hajun had the leaderboard taken out
  and the group made a bubble); and round a phone's jump button a face
  (emotes, one day), a speech bubble (chat) and a fist. Every colour of it
  is `assets/ui.ron`'s since 2026-11-04 (above, "The screen's colours and
  words are in `assets/ui.ron`"). The person and the
  gear are 8% of the short side (`hud::SMALL_BUTTON`), the three down the
  right edge 10.5 (`SIDE_BUTTON`; 9.5 for an hour on 2026-11-01, then "very
  slightly" bigger) and 1.5 further in from the top and the right than the
  name is (`SIDE_INSET`, "very slightly increase margin"), the three round
  the jump button 14
  (`ACTION_BUTTON`), the stick 25 (`STICK_SIZE_VMIN`; 22 until Hajun
  asked for "slightly" bigger, 2026-11-01), 7% of the width in from the
  left (`STICK_LEFT_VW`; 8 until "very very slightly more left", the same
  day), and the jump button itself 22,
  6% of the width and 9% of the height in from the corner
  (`JUMP_SIZE_VMIN`, `JUMP_RIGHT_VW`, `JUMP_BOTTOM_VH` in `src/lib.rs`;
  since the evening of 2026-10-31, Hajun: "keep the size of the user icon
  and the settings icon but increase the size of every button the right
  hand side both top and bottom, especially jump button, and move the
  bottom right buttons slightly closer to the bottom right corner"; 8, 8,
  12 and 18 at 8% and 11% until then, then 5% and 7% for a night, until
  Hajun asked for "very slightly more margin", and 8% of the height for an
  hour, until "move them up very very slightly"). The three are placed from the
  jump button's corner so as to sit about 2.5% clear of it, the face up
  and to the left, the bubble over it and the fist beside it, as Hajun's
  sketch has them. **Only the gear, the stick, the jump button, the fist,
  the face, the gamepad and the speech bubble under the gift do anything**
  (Hajun: "u dont need to make any button
  functional yet other than jump, joystick, settings"; the fist since
  2026-11-01, below, "The fist throws a punch"; the face since 2026-11-05,
  the car's switch, below, "The car you drive", where the fist is the
  reverse and the jump button the boost; the gift since 2026-11-06,
  which held in the car is its gun, there too; the gamepad since the
  night of 2026-11-07, the game selector, above, "Games"; and the speech
  bubble under the gift since 2026-11-13, your friends, with how many
  things wait in your inbox counted in gold on its corner, below,
  "Friends, groups and messages"); the rest take a press and drop it. The three round the jump button are placed from where
  the jump button is, on every platform (until 2026-11-06 the desktop
  build showed the three with no stick or jump button among them; since
  then it has both, below, "The stick and the jump button are on Windows
  too"). Until then a
  button top right took you home (`testhomemap.glb`) and back (Hajun:
  "remove the top right home button"); nothing takes you home now, though
  it is still loaded at startup (`island::LOADED`) and the server still
  answers `Ask::Travel`. The town's name shows again every time you arrive
  back in the town, from a game of House Builder.
- **Every button is in one style, the stick's and the jump button's**
  (Hajun, 2026-10-31): a see-through grey disc with no rim, and on it a
  see-through white picture (`hud::round_button`, `TOUCH_DISC`,
  `TOUCH_MARK`: white at 0.52; "a little greyed", 0.92 at 0.38, until Hajun
  asked for "slightly more whiter"; held, the jump button is a little more
  solid). The jump button's arrow, the pencil and the fist are drawn in
  code (`hud::jump_arrow`, `pencil`, `fist`), each one filled shape edged
  by how far every pixel is from it, as the rating star is: nodes
  overlapping would show where they do through the see-through white. The
  fist is a closed hand from the front, four fingers over the palm with
  the thumb across them (Hajun, 2026-11-01: "more a hand fist rather than
  boxing glove"; Material has no closed hand); the gear was drawn in code
  too until that day, when Hajun's screenshot of it was Material's
  `settings`. The other pictures are 96 px PNGs in `assets/icons/`, white
  on clear (`hud::ICONS`): Google's Material icons (Apache 2.0, its
  licence beside them and out of the APK), turned white by hand (Hajun:
  "search online for that icon + png/transparent, download and modify
  it"): person, settings, videogame_asset, mood and textsms; and Font
  Awesome Free's gift (solid) and comment-dots (regular), the gift and the
  social chat's bubble top right (CC BY 4.0, `LICENSE-font-awesome.txt`
  beside them, out of the APK), the icons in Hajun's FlutterFlow
  screenshots of 2026-11-01, rendered white from their SVGs with an 8 px
  margin by a throwaway `resvg` tool (the gift and the bubble were drawn
  in code for an hour first). card_giftcard, leaderboard, groups and
  sports_mma are still in the folder, unused. Until 2026-10-31 the stick
  was white with a white rim, and the jump button yellow, its arrow three
  strokes. The stick and the jump button are put away while you talk to
  someone (below), and since 2026-11-01 the three round the jump button
  with them (`hud::hide_round_the_jump`; Hajun: "the punch, quickchat, and
  emote button is not hidden. hide those too"). Seen in the desktop build; none of it has been seen on a phone yet.
- **The stick's middle is set where the finger first goes down, and no
  further than half its radius from its own place** (`stick_origin`,
  `STICK_ORIGIN_REACH` in `src/lib.rs`, since 2026-11-14; Hajun: "if I tap
  the very bottom of the circle, the joystick's origin gets set too low, so
  it's hard to move downward at full speed. Limit the area where the origin
  can be set to 50% of the current radius. If the first tap lands outside
  that radius but still inside the joystick hitbox, set the origin on the
  edge of the 50% radius at the same angle. Movement magnitude should then
  be measured from that origin"). A finger put down at the very bottom of
  the disc has the middle half a radius down, and is pushing half way down
  from the start. From the small hours of that day the middle was wherever
  the finger went down (`versionCode` 28, "Building"). The mouse's on
  desktop the same. Unit-tested
  (`the_stick_s_middle_is_set_no_further_than_half_its_radius_from_its_own`);
  not seen under a thumb, and a script cannot hold the mouse on the stick.
- **The stick and the jump button are on Windows too** (since 2026-11-06,
  Hajun: "put on screen joystick and jump button on windows version too";
  until then they compiled for phones alone, `setup_hud` and the stick's
  and the jump button's visuals gated to them): drawn as on a phone, where
  a phone has them, and driven by the mouse with the cursor free, Escape
  (`read_mouse_controls` in `src/lib.rs`, before
  `editor::read_desktop_pointer`): pressed on the stick's disc, the mouse
  holds the stick, pushed toward wherever the cursor goes until the button
  comes up, as a finger is; pressed on the jump button it jumps, or boosts
  the car while held. The mouse is one more finger to `TouchControls`
  (`MOUSE_ID`), so `read_player_input` and `drive::steer` need no desktop
  path, and the keys and Space work as before. A press on either is not a
  tap on the town (`TouchControls::has_the_mouse`, which the editor's
  desktop pointer asks), though it dismisses an open menu, as on a phone;
  and one on a button over the world is the button's. Walking with the
  mouse on the stick, the camera comes round behind you after 1.5 s, as it
  does on a phone (`assist_camera`): the mouse is off the camera while the
  cursor is free. With the cursor taken for looking round there is no
  cursor to press with. Asked for to drive the car with a stick on the PC,
  the keys giving only eight ways. Seen in the desktop build the same day.
- **The fist throws a punch** (`Punching`, `start_punch`, `punch_offset` in
  `src/lib.rs`, since 2026-11-01; Hajun: "a very basic animation for the
  fist attack that activates each time u press the punch button. dont make
  the animation too long so that user can press the attack button like 4~8
  time per second"): each press of the fist, as it goes down
  (`hud::press_buttons`), or F on desktop, out of the car (in it the fist
  is the reverse, below, "The car you drive"), swings the right arm straight out
  level in front of you and back, 0.07 s out and 0.11 s back (`PUNCH_OUT`,
  `PUNCH_BACK`, `PUNCH_REACH` 1.7 rad, level in front of
  `playercharacter.glb`, whose arms hang a little back of the shoulder; 1.5
  until 2026-11-01; the two arms in turn for an hour,
  until Hajun asked for "only use right hand"); a press mid-punch starts
  the next at once, from the side. The arm punches in place of its walk;
  the rest of the body walks on. Nothing of it goes over the network, so
  nobody online sees it; it hits nothing. Seen from the side in the
  desktop build, the arm level in front (it first swung backward: forward
  is the same turn in both arms' bone spaces, the walk only negates the
  right's to swing it against the left); not yet on a phone.
- **On desktop, press `Escape` to free the cursor before clicking a button**,
  the same as for editing. On a phone you just tap it.
- **A press on a button is the button's alone.** Every button over the world
  is a `bevy_ui` node pressed through `bevy_picking`'s click, which a mouse
  and a finger both send, and carries `hud::TakesPress`. The touch controls
  and the editor's desktop pointer ask `hud::Presses::claimed` first, which
  tests the node's laid-out shape the way picking does, so the same press is
  not also taken for a look round or a tap on the town. A new button needs
  `TakesPress` and nothing else: there are no sizes to keep in step.
- **Every player has a balance** (`lobby::Balance`), shown top left next to
  `assets/coin.png`. Offline it starts at 1,000 (`lobby::STARTING_BALANCE`)
  and is not saved, so every launch starts at 1,000 again. Online it is the
  server's: kept with your account in Postgres, which a new account starts
  with 1,000 in, and sent to the device (`Tell::Balance`). The only things
  that change it are House Builder's entry fee and its prizes (below).
  `assets/coin.glb` is still in the repo, but nothing loads it.
- The text is Inter since 2026-11-13 ("The font is Inter", below); until
  then Bevy's built-in font, which is plain ASCII: no Hangul, and no ★,
  `…` or `·` either. Inter has no Hangul either. The rating star is drawn
  in code (`builder::star_image`).

## Settings

A gear beside your name, top left (`src/settings.rs`, since 2026-10-31;
Hajun: "a setting icon ... make it show a semi transparent greyish
rectangular popup which will have all sorts of different useful
settings"; top right for an hour, until their sketch of the screen put it
by the name), Material's `settings` icon (`assets/icons/settings.png`,
since 2026-11-01, Hajun's screenshot; drawn in code as the jump button's
arrow is until then), in the buttons' own style (`hud::round_button`),
after the person button
in `hud::BarLeft`. It opens a window of
grey glass over the dimmed screen, a dialog like the shop (`DialogUp`: you
stand still, every press is its, Escape or the cross on its corner or the
dark round it shuts it), with three tabs along its top: **Graphics** and
**Controls**, with nothing in them yet, and **Accounts**: the name you go
by, with a pencil beside it (`hud::pencil`) that opens a smaller dialog,
"Edit name", with the name in the sign-up's typing box (`signup::NameBox`,
`signup::name_box`) and Cancel and Save (Hajun: "a pencil/edit icon that if
u press, you can edit your name in a smaller new popup. and save so thats
ur new name"), which on a phone slides up out from under the keyboard as
the sign-up's name page does (`signup::LiftsForKeyboard`, above, "The first
time, you sign up"): Save makes it your name everywhere the sign-up did
(`signup::wear`: the town's room, over your head, and the server, through
the profile changing), and goes back to the tab; "Sign in with Google" and
"Sign in with Apple" blocked out until there is a sign-in to be had; and
**Delete account**, which asks once more, then has the login forget the
account; this device forgets its secret and its profile, and the sign-up's
pages come back. Offline, Delete says "Not online: try again". Seen in the
desktop build on 2026-10-31, scripted; not yet on a phone. Since 2026-11-13
an account deleted takes its friends, what it asked and was asked, and
what was written to and by it with it ("Friends, groups and messages",
below).
