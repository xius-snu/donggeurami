# Signing up, and the box everything is typed into

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## The first time, you sign up

Since 2026-10-31 (Hajun), the first time the app is run, or whenever the name
and the character are not set, three pages come before the town
(`src/signup.rs`): the town's name over a Start button, with "Already have an
account? Log In" under it; "Choose Username", "Shown to other players.
2-16 characters." on one line (a hyphen: the font has no en dash), a box
to type a name into and Save; and "Choose Character", "You may change this
later.", a boy and a girl on two cards with their pictures and nothing
else, since 2026-11-02 with a white arrowhead either side of each card
that turns it through that side's colours (`signup::ColourArrow`,
`Action::Turn`, `turn_cards`; Hajun: "put some arrow thingy next to each
guy and girl character selection in the start so u can choose which color
guy or which color girl"; pressing an arrow chooses that side as well; the
pictures, `assets/signup/<look>.png`, one to each look, are rendered from
the models by `blender/render_person.py`, which takes a `.glb` since the
colourings have no `.blend`; until then `boy.png` and `girl.png` from
`limbperson.blend` and `limbpersongirl.blend`), and Save (Hajun,
2026-11-01; until then three lines of explanation, "Save Username", "Save
Character" and "Boy" and "Girl" under the pictures). Solid colours, no
outlines, no words but those (Hajun:
"as minimalistic as possible"): black pages, the box and the cards dark
grey, the chosen card lighter, and white buttons with black words (since
the evening of 2026-10-31, Hajun: "black for the initial pages ... white
instead of yellow for the buttons"; dark blue, blue and gold until then).

- **It is kept on the device, `profile.json` beside `town.json`**, so the
  pages never come again; and online it is the account's: sent as soon as
  the server has let you in, if the account has you otherwise, and whenever
  it changes (`Ask::Profile`, the last of the asks, so that older apps are
  not told to update), kept in Postgres in `players.name` and `players.look`,
  which were there already, and sent to everyone in your room as a new
  member record. No other database: the pages are two columns on the
  accounts table. A native sign-in later (Apple, Google) is one more way to
  find that row, by the token's subject, and is where "Log In" goes; for now
  Log In says "Log in is not available yet".
- **The look saved is any of either side's** (`roundtown_net::BOY_LOOKS`,
  `GIRL_LOOKS`, `profile::look`; since 2026-11-02, `boy` or `girl` alone
  until then): the server checks the same list, so **it has to be deployed
  with the app**, or a profile in a new colour is thrown out whole, name and
  all, and the account keeps its generated name online.
- **A name is 2 to 16 letters of English or Vietnamese, or up to 8 of
  Korean** (`roundtown_net::profile`, `roundtown_net::text`, checked on
  both sides; since 2026-11-14, Hajun: "i need korean text to be able to
  work in the global chat not just english. same for username and send
  message", "also vietnamese text as well"; plain ASCII until then, the
  game's font having no Hangul). Its length is counted in widths, a Hangul
  syllable two, so that a name is as long on the screen in either: "민"
  is a name, and "동그라미타운주민" as long a one as fits. What may be
  written, and how a letter typed in pieces is put together, is under "The
  font is Inter", below; the page says "2-16 characters, or 8 in Korean"
  (`words.signup.name_hint`, my wording). **A name with a common swear
  word in it is no name** (since later that evening; "The chat", below):
  Save stays grey, and the hint's line reads "Pick another name." in the
  colour of a warning for as long as it is in the box, here and under the
  box of the settings' Edit name (`signup::NameNotice`,
  `words.signup.name_refused`). It need not be unique: since 2026-11-13 every account has a
  tag after its name, `#1000` to `#9999`, and no two have both the same
  ("Friends, groups and messages", below). Typing: on desktop the keys and Enter;
  on a phone the soft keyboard, asked for by the window's `ime_enabled`,
  which winit shows on both phones. An iPhone's sends each letter as a key
  press and Return as a newline; Android's sends no keys at all, and
  writes into the activity's own text (`GameTextInput`), which the name is
  read off every frame (`signup::android_text`, through
  `bevy::android::ANDROID_APP`). Neither has been tried on a phone yet; the
  Android build type-checks. A phone's keyboard, with the phone on its
  side, covers most of the screen, so the name's page, and the settings'
  Edit name, slide up out from under it (`signup::Keyboard`,
  `signup::LiftsForKeyboard`, `lift_for_keyboard`; Hajun, 2026-10-31: "the
  keyboard hides the edit name box so u cant see the text anymore. make it
  go up in reaction to keyboard. as well as when u first set ur name"):
  laid out as with no keyboard and slid up by their own transform, by as
  much as puts their bottom 2% of the short side over the keyboard, and
  never so far that the box the name goes in is nearer the top than that
  (the heading may go off the top). On Android the keyboard's height is
  what the window's insets say: `MainActivity.onApplyWindowInsets` hands
  the IME inset's bottom to the game through a native method the game
  exports (`imeInset`, `Java_town_donggeurami_MainActivity_imeInset` in
  `signup`), kept in an atomic the game reads every frame, in the screen's
  pixels: times `RENDER_SCALE`, over the window's scale factor. **Not
  through `GameActivity`'s native side**: the first build of this called
  `GameActivity_getWindowInsets` on `AndroidApp::activity_as_ptr()`, which
  is a JNI reference to the Java activity and not the native `GameActivity`
  struct, and the app crashed on launch (Hajun, 2026-10-31: "app is
  crashing on launch"). An iPhone's winit says nothing of its keyboard, so
  there it is taken to cover 0.55 of the screen's height while a box is up
  (`IOS_KEYBOARD`, my guess). Seen in the desktop build with a keyboard of
  0.55 written in by a script, both pages; not yet with a real keyboard on
  a phone.
- **The character is a look** (`lobby::LOOKS`, `roundtown_net::PEOPLE_LOOKS`):
  `boy` is Hajun's `playercharacter.glb` and `girl` their
  `playercharactergirl.glb` (`lobby::MODEL`, `GIRL_MODEL`; below, "Every
  body is Hajun's `playercharacter.glb`"; since the night of 2026-11-01:
  for the day's first hours both were the boy's, and until that day
  `limbperson.glb` and `limbpersongirl.glb`, the same rig and bones, a
  pink top). The cards' pictures, `assets/signup/boy.png` and `girl.png`,
  are rendered from the two by `blender/render_person.py` (run again the
  same night). Every account from before wears `blue`,
  which is drawn as the boy, as is any look an app has not heard of. Saving
  swaps your body's model on the spot (`lobby::dressed_as`; the scene
  spawner takes the old one down) and your name in the town's room, so a
  game of House Builder offline is played under it; online, `net::redress`
  swaps anyone else's as their record changes, and their name over their
  head follows (`tags::rename`). The AIs are a boy or a girl by the toss of
  a coin, and then in any of that side's colours (`roundtown_net::ai_look`,
  `BOY_LOOKS`, `GIRL_LOOKS`: the boy and the girl people choose and Hajun's two
  recolourings of each, `playercharacterblackbrown.glb`,
  `playercharacterwhitejeans.glb`, `playercharactergirlwhiteblue.glb` and
  `playercharactergirlyellowpink.glb`, looks `boy_blackbrown` and the like
  in `lobby::LOOKS`; on this device in `lobby::ai_member` and for the
  server's bots in `server/src/builder.rs`, since 2026-11-02; Hajun: "make
  the ai's also be random girl/guy 50/50", "use variations of the colors. i
  made various colors for both guy and girl for the ais"); until then they
  took turns at red, yellow and purple, which stay known, drawn as the boy,
  for the server's bots until it is deployed again and for older apps, as
  any look an app has not heard of is.
- **While the pages are up the world waits as under a dialog**
  (`hud::DialogUp`): you stand still, every press is the page's, the cursor
  is free on desktop, and the town's name waits to fade in until the pages
  have gone. Deleting your account (`settings`) forgets the profile and
  brings the pages back.
- Seen in the desktop build on 2026-10-31, scripted with posted clicks and
  keys: all three pages, Log In's notice, Save greyed until there is a name
  or a choice, the girl chosen and walking in the town under the typed name,
  the name and look on a local server, and a relaunch going straight to the
  town.
