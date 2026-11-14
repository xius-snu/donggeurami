# Friends, groups and messages

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Friends, groups and messages

Since 2026-11-13 (Hajun, with four sketches of the panel at the root of the
repo, `Friendtabdesign1.png` and `friendtabdesign2.png` to `4`: "I want to
add a friend system. First every account need a random number between
1000~9999 that is gona serve as a #1234 tag for identification when there
is duplicate name and your friend need to know which one u are, and
whenever u change ur name to sth, it will first check if there is someone
else that has your #1234 tag is also that same name, and so that there can
be same names, but never BOTH same name AND same number. The friend panel
popup will open when u tap the top right corner chat bubblt icon under the
gift icon ... you should be able to search the username, and it will give u
a list of everyone with that username and u can choose which one to add,
and when u add the other person, will get a small 1 badge next to the chat
icon, and the "message" tab inside the panel, andthey can accept or reject
the add friend request. and when youre in the group with ur friend, only
the leader can queue for a minigame, and everyone get queued at the same
time in the same team (unless group is >2 and ur playing 2v2v2v2 bridge
wars, then it will need to be 2 ppl same team 1 ppl other team i guess).
and inviting to group should also land in the same panel's messaging icon
tab with the small 1 circular badge in gold background as a subscript of
the chat icon. and they can accept or reject the group invite too"; and of
the sketches, "just the concept + the general vibe/colorscheme/design
vibe. you can adjust the actual layout and add functions that are not
shown in the design if u think its necessary. but atleast have all the
features that are shown in the designs"). What the two sides say is
`roundtown_net::social` (in `shared/src/lib.rs`: `Ask::Social(Act)`,
`Tell::Social(News)`, the last of each, and `Refusal::NotLeader`); what the
server keeps and does is `server/src/social.rs`; the panel is
`src/friends.rs`. **`PROTOCOL_VERSION` 15**: a `Tell` an app from before
cannot read, so the server went out with the app: **deployed at 14:07 UTC
on 2026-11-13** (23:07 here), with Bridge Wars' dynamite of the same day
("Games"), nobody having been on it for three and a half hours; **APK 26
is the one that plays online**, and an APK from before is told to update.
The binary before is kept on the VM as `roundtown-server.prev-2026-11-13c`
and a dump of the database as it was before tags as
`/var/backups/roundtown/before-tags-2026-11-13.dump`, out of the nightly
backups' rotation.

- **Everything about friends is the server's, and there are none offline**
  (the panel says "You are offline. Friends need a connection."). The
  server holds all of it in memory (`social::Social`): what every account
  is called and its tag, who is whose friend, every inbox, every group. It
  is read from the database once, as the server starts
  (`accounts::Snapshot`, out of `accounts::set_up`), and from then on the
  game's loop goes by its own copy, so nothing about friends waits on a
  query; whatever is to outlast the server is written down after the fact,
  in the order it happened (`accounts::Keep`, `Bank::keep`, `keep_it`,
  through the accounts' thread). Without a database (`RT_DB` unset, a
  server on this PC) it all lasts as long as the server runs. `Social`
  knows nothing of the world: it takes what is asked (`Social::act`) and
  where everyone is (`saw`, `left`, `tick`), and says what is to be told
  and to whom and what is to be kept (`told`, `kept`), which three small
  systems carry out (`hear`; `track`, four times a second; `flush`), so
  all of it is unit-tested with no server running. **A device is told
  everything of its account's as it comes** (`Social::greet`: its tag and
  level, its friends, its inbox, its group), and nothing is kept on the
  device: it forgets all of it the moment it is offline
  (`friends::forget_offline`). That has to hold for a device let in while
  the server still thinks the one before it connected, a phone's app put
  away and opened again within the few seconds a dead connection takes
  to be given up: `players::admit` says so (`Social::connected`), or the
  panel came back empty until something changed.
- **The tag** is `players.tag`, `#1000` to `#9999`
  (`social::TAG_FIRST`, `TAG_LAST`). No two accounts have both the same
  name, whatever its capitals, and the same tag: the server's copy keeps to
  it (`Social::enroll`, `rename`, `taken`), and the table has a unique
  index on `(lower(name), tag)` (`players_name_tag`, `SCHEMA_AFTER`),
  which holds the login to it as well. A new account is given a tag nobody
  of its generated name has (`login::account_of`, trying again up to 16
  times should the index refuse it); every account from before was given
  one the first time the server started with tags (`accounts::give_tags`).
  **Taking a name that someone with your tag already has gives you another
  tag**, at random, that nobody of that name has (`Social::rename`, from
  `Ask::Profile`): mine. Hajun asked that the two never both be the same;
  the other way is to refuse the name, which would make a name something
  that can be taken. The device is told its tag and level whenever either
  changes (`News::Me`), and so are its friends (`News::Friend`).
- **The tables** (`accounts::SCHEMA`, made as the server starts, each
  statement leaving what is there alone): `players.tag` and `players.xp`;
  `friends (a, b)`, the lesser number first; `friend_requests (sender,
  receiver)`; `messages (id, sender, receiver, body)`. Every one goes with
  its account (`ON DELETE CASCADE`): the login's `/delete` takes them with
  the row, and tells the game's loop which account has gone
  (`login::Setup::gone`, `social::forget_deleted`), so that its own copy
  lets go of it too. **The server from before runs on the tables as they
  are now** (its own INSERT and UPDATE of `players` tried on a migrated
  copy: a row made without a tag is allowed), so going back to a
  `.prev` binary needs nothing undone in the database; a rename that
  lands on someone's name and tag would be refused by the index there,
  and that is all.
- **A level** is what an account has earned playing (`players.xp`,
  `Social::award`, `social::level`): 20 for playing a game to its end
  (`XP_PLAYED`), 50 more on the team that wins Bridge Wars (`XP_WON`), 50,
  30 and 15 more for House Builder's first three places (`XP_PLACED`),
  and a level for every 100 (`XP_A_LEVEL`), from 1. All mine: the sketch
  has "Level 53" by each name and nothing says what a level is. A game
  skipped to its end (`RT_ALLOW_SKIP`) earns as it pays its prizes.
- **The panel** (`src/friends.rs`; every colour and word of it
  `assets/ui.ron`'s `colours.friends` and `words.friends`): the speech
  bubble under the gift (`hud::FriendsButton`, the "social" button that did
  nothing) opens it, a dialog (`hud::DialogUp`: you stand still, and a
  press on it is its own; the cross or Escape shuts it), dark and
  see-through as sketched, 112 by 66 shares of the short side, which is
  the sketch's own size on a phone and fits a 4:3 tablet (a test keeps it
  inside both). Along its top two tabs and the cross. It is drawn again
  whole whenever what it shows
  changes (`redraw`, as the settings' window is), keeping what is typed
  (`signup::NameBox`) and how far each list is scrolled (`Scrolled`); a
  list scrolls under a finger or the mouse held (`drag_list`), and a
  press dragged more than 16 px is no tap on the button it began on
  (`TAP_SLOP`).
  - **A press anywhere off the panel shuts it as it goes down, and is
    whatever it landed on's as well** (since the night of 2026-11-13,
    Hajun: "it should close just from the DOWN. and also when i for
    example touch the joystick as the way of closing it or jump button,
    it should register the joystick tap/drag or the jump button rather
    than needing to press twice"; for its first evening a tap, down and
    up, on the dark round it shut it, and that tap was the panel's
    alone). The panel's root is over the whole screen, takes no press for
    itself (`TakesPress` is on the panel and on what comes up over it)
    and lets what is under it be pressed (`Pickable` that blocks nothing):
    `press_outside` hears a press that lands on it through picking and
    puts the panel away, so a thumb on a button under it presses that
    button too, the gamepad opening the games with the one tap; and
    `shut_under_a_press`, ordered before `read_player_input`, puts it
    away in the very frame a press lands on nothing that takes one, so
    that the stick has that touch and a press on the jump button jumps
    (a jump is read only in the frame its press lands, and you stand
    still under a dialog). The speech bubble itself, pressed with the
    panel up, shuts it and does not open it again with the same press
    (`Pressed::shut_it`).
  - **Your tag is after your name on its plate, top left**
    (`hud::YourTag`, which `friends::show_tag` writes as the server says
    it; `hud::show_name`: "Hajooon#5571" online, your name alone
    offline), in the town and in Bridge Wars alike, and nothing of your
    own is in the panel's top; **your own level is shown nowhere but on
    your row of a group's list** (Hajun, the same night: "i dont need
    that text there. just put that number in the main Ui top left corner
    name tag ... Your own Level doesnt need to be shown anywhere for
    now"; for its first evening the panel's top had your name, tag and
    level beside the cross). The cross is 8 shares across (`CLOSE`; 7
    until "increase size of the close button slightly") and has no disc
    come up behind it under the pointer or a finger, as the panel's other
    bare buttons have ("it should just close it no hover/click
    background"; `bare`'s `lit`).
  - **Friends**, the first tab: your group, if you are in one ("Group
    (2/4):", Leave, each member with "(Leader)" and "(You)", and for the
    leader a cross that puts a member out); then "Friends (1/2):", how
    many are here of how many, with the person and plus that opens Add
    Friend; and a row to each friend, whoever is here first and then by
    name: a disc with a dot, green here and grey gone, the name, where
    they are ("In Main Lobby #1", the channel of the town; "At home";
    "Bridge Wars: In Queue" and "In Game"; "Offline for 15 hours", drawn
    again each minute), "Level 53", a button that asks them into your
    group (the sketch's question first, "Would you like to invite ... to
    your group?", Cancel and Yes; faint when they cannot be asked: gone,
    in your group already, or you in a group you do not lead), and a
    gear, under which Message and Remove Friend (which asks first).
  - **Add Friend**: a name typed into the sign-up's box
    (`NameBox::open_for`), with "Friend's Username" standing faint in it
    until the first letter (`Hint`, `show_hint`, `words.friends.
    search_hint`, `colours.friends.hint`; a line under the box said to
    type a name and tap the glass, until Hajun had it put in the box);
    the glass, or Enter, looks for it (`Act::Find`):
    everyone of that name, whatever its capitals, and whoever has that
    name and tag if it ends in one (`social::split_handle`), up to 20,
    whoever is here first and then whoever was here last, never yourself.
    Beside each, "Add friend" (`Act::Befriend`), or "Requested",
    "Friends", or "Accept" for one who has asked you. Under them "My ID"
    and your own name and tag, the tag fainter, with a button that copies
    it. **Until something has been looked for, "My ID" stands right under
    the search bar**: the list of who was found is as many rows tall as
    it has to show, none before a search, one for "Nobody goes by that
    name.", and up to three, past which it scrolls (`found_rows`; Hajun:
    "there should be barely any gap between the search bar and the "My
    ID" box. that gap only appears after you press enter/click the search
    icon"; three rows' room was kept from the start until then). **On a
    phone the box, like the one a message is written in, slides up over
    the keyboard only until its own top is at the top of the screen**
    (`signup::LiftsForKeyboard::whole`, `lifting_whole`; Hajun: "the Add
    Friend popup's top edge should touch the top of the screen when
    keyboard is up. right now the top area is hidden by going over the
    border"), so its title and the way back out of it stay in sight and
    whatever does not fit is under the keyboard; the sign-up's name page
    and the settings' Edit name still slide as far as keeps the box typed
    into on the screen, their headings going off the top.
  - **The envelope**, the second tab, is your inbox, newest first: who
    wants to be your friend (Accept, Reject), who invites you to their
    group (Join, Reject), and what your friends have written (Reply, OK);
    with nothing in it, "All Caught Up." (and an empty list of friends
    says "No friends."; both Hajun's words, that night).
    How many wait there is counted in gold on the envelope's corner and
    on the speech bubble's among the buttons (`hang_badge`, `count_badge`;
    "9+" past nine). A request and a message wait for someone who is not
    here, and are there when they next come.
  - What came of something asked is said in a line along the panel's
    bottom for 3.5 s ("Friend request sent.", `Outcome`, `words.friends.
    said`); what happens to your group while the panel may be shut is said
    across the lower screen (`toast`).
- **A message** (`Act::Write`) is up to 80 letters of English or
  Vietnamese, or 40 syllables of Korean (`social::message`,
  `MESSAGE_LONGEST`, counted in widths; plain ASCII until 2026-11-14, "The
  font is Inter"), to a friend, here or not; it waits in their inbox until they press OK, 30 at the most
  (`LETTERS_MOST`), and is kept in the database until then. Mine: the
  sketch's gear has "Message" under it and nothing more is said of it.
  **It is the first thing one player writes that another reads**: friends
  only, no filter, no block and no report, which a shop's review may ask
  for (Apple does of user-generated content) before a release to
  strangers. Hajun's call.
- **A group** (`Act::Invite`, `Answer`, `LeaveGroup`, `Kick`; `News::Group`)
  has a leader, whoever first asked someone into it, and up to four in it
  (`GROUP_MOST`: two teams of Bridge Wars; mine). Being asked stands for
  ten minutes and only while whoever asked is here (`INVITE_SECS`); saying
  yes to one group leaves the one you were in; the leader leaving, the
  first of the rest who is here leads; one alone is no group. Whoever's
  connection goes is kept in their group for two minutes (`GROUP_GRACE`: a
  phone closes its connection whenever the app is put in the background).
  Groups are never written down: the server starting again ends them all.
- **Only a group's leader starts a game.** Anyone else in a group finds
  the game selector's cards blocked out over "Only your group's leader can
  start a game." (`games`, `Friends::follows`), and the server refuses
  them (`Refusal::NotLeader`). The leader choosing a game takes with them
  everyone of the group who is free to go: connected, and in the town or
  in a game that is over for them, and, for House Builder, with its 100
  coins, each paying their own (`Social::party`; `Ask::Arena` in
  `server/src/arena.rs`, `Ask::Play` in `server/src/builder.rs`). Whoever
  is not is told "Your group started a game without you." and stays in
  the group. **In Bridge Wars they are seated two by two onto a team**
  (`Game::seats_for`, `seat_together`): two on one team; of three, two on
  one team and the third on the island across the map, as Hajun has it;
  four, two teams; in the game filling now if it has the seats for all of
  them, and in a new one if not (`Arenas::filling`). In House Builder they
  are all in one game (`Games::filling`). Whoever is taken along has the
  panel put away and "Your group is playing Bridge Wars" across the
  screen.
- **Copying your name and tag**: on desktop to the system's clipboard
  (Bevy's `system_clipboard` feature, on for desktop alone in
  `Cargo.toml`). On Android the game cannot reach the clipboard, so it
  keeps what is to be copied until `MainActivity` has taken it, asked
  every 300 ms by numbers alone (`clipLength`, `clipByte`, `clipTaken`,
  exported from `friends` as the keyboard's `imeInset` is from `signup`:
  nothing of Java's is touched from the game's side, which is what
  crashed the first build of the keyboard's lift). An iPhone has no copy
  button: the name and tag are there to read.
- **What is mine, Hajun having drawn the panel and said the rules above**:
  another tag rather than a refused name; what a level is; messages, their
  80 characters and 30 waiting; the 100 friends and 50 unanswered
  requests an account may have (`FRIENDS_MOST`, `REQUESTS_MOST`); four to
  a group, its ten minutes and two; that whoever cannot go is left behind
  rather than holding the group back; that a request turned down says
  nothing to whoever asked, who may ask again; "9+"; no copy button on an
  iPhone; and, of the night's changes, that a press off the panel is a
  button's under it as well as the stick's and the jump button's, that
  the list of who was found is as tall as what it shows rather than three
  rows once anything has been looked for, and that your own row of a
  group's list still has your level, as every row of it has.
- **Not built**: blocking or reporting someone; anything said to you while
  the app is shut (a request waits for when you next open it); a group's
  own chat. (Hangul in a name was not, until 2026-11-14: "The font is
  Inter", below.)
- **Enter on a phone's keyboard looks for the name typed** (since
  2026-11-14; Hajun: "in the friend popup's Add Friend popup, when i type
  the name, and i press enter nothing happens - only when i press the
  search bar"), and saves a name, and sends a message or a line of chat:
  Android's keyboard has no Enter key to press, its action key ("Done", a
  tick) tells the activity to do what the box is for, which nothing heard.
  `MainActivity.onEditorAction` hands it to the game through a native
  method the game exports (`imeAction`,
  `Java_town_donggeurami_MainActivity_imeAction` in `signup`, counted in
  an atomic as the keyboard's height is handed over), and
  `signup::android_text` takes it for Enter (`NameBox::entered`, which
  `friends`, `settings`, the sign-up and `chat` already asked), once for
  presses within a quarter of a second of each other, a plugged-in
  keyboard's Enter being handed over as it goes down and as it comes up.
  The Java side catches anything that goes wrong: a missing native method
  leaves Enter doing nothing, as before. An iPhone's Return was typed as a
  newline all along, and a desktop's Enter is a key. Compiled and
  packaged (APK 29, the five entry points read back off the library); not
  pressed on a phone.
- **Checked on 2026-11-13.** Unit-tested: the rules both sides share (56
  in `shared`), the server's `Social` and a group's seats in Bridge Wars
  (45 in `server`, nineteen of them new for this), the panel's own (276
  in the game). **The database, on the VM's own Postgres, against scratch
  databases made and dropped for it** (the server's own never written
  to, its accounts' names, looks and coins hashed before and after): two
  ignored tests that will run only on a database whose name ends in
  `_scratch` (`RT_TEST_DB`; `accounts::tests::scratch`), three times
  each: everything `social` asks to be kept written and read back, the
  login making forty accounts, each with a tag nobody of its name has,
  and **a copy of the server's own accounts taken through starting up**,
  each given its tag with nothing else of it changed. **End to end with a
  real Postgres**: the new server on this PC against a scratch database
  on the VM (reached through SSH as the VM's own `roundtown` user, with
  nothing opened there and nothing left), three scripted desktop copies:
  accounts from before given their tags as it started, two requests, one
  answered, two messages, one seen off, a group taken into a game of
  Bridge Wars and its 20 earned; the server stopped and started; and
  every tag, the friendship, the request still unanswered and the
  message not yet seen off were as they had been, in the table and on
  the three screens, the request then answered. That run found accounts
  numbered past the
  middle of what the table counts read back as one account (ids clamped
  at nought where everything else casts them: `Snapshot::read`, and
  `keep_it` ordering two friends other than the table does), which only
  a server run without the login can have, and which is mended. **In the
  desktop build with a server on this PC, scripted, six runs of two or
  three copies**, one of them by touches on the buttons alone and two
  more in part, each run again as the code changed: a search finding two
  players of one name, one gone; asked, accepted, rejected and asked
  again; the gold count on both corners; a group of three with its
  leader, a member put out, one leaving; a message written, read,
  answered and seen off, seven of them scrolled with a finger, a drag
  begun on a button pressing nothing; the leader taking three into
  Bridge Wars, two on one team and one on another; a follower refused,
  and the selector blocked for them; a group into House Builder, each
  paying; a friend gone and back, with what was written meanwhile
  waiting; and a copy killed outright and opened again at once, which
  the server took for never having left ("is back"), with its friend and
  its message there again. **Not by hand, and not on a phone**: the keyboard's lift for
  the two boxes typed into, the copy on Android, and how the panel sits
  under a thumb are all unseen. The desktop's copy was read back off the
  system's clipboard after a scripted press of the button ("Copied").
  With a phone's keyboard written in by the script, over the lower 55% of
  the window, both boxes slid up clear of it: Add Friend with its box,
  its results and "My ID" all in sight and its title off the top, and the
  message's box with Cancel and Send.
- **As it went out.** The server's own tables were given the columns, the
  three tables and the index as the new server first started, and its
  three accounts their tags ("3 accounts were given their tags"), with
  every account's name, look and coins as they were (hashed before and
  after). This PC's own account was then let in through the login, shown
  its tag and level in the panel, told "Nobody goes by that name." of a
  name nobody has, and written down as seen when it left. **Nothing
  between two accounts has been done on the VM itself**: no request, no
  group and no message, there being one account to try it with and no new
  ones made for it; that is the tests', the scratch databases' and the
  runs' on this PC alone. APK 26 is built, its 85 asset files checked
  byte for byte, its library the stripped one with the four entry points
  `MainActivity` calls (`imeInset`, `clipLength`, `clipByte`,
  `clipTaken`), and its number read back, and **not installed, no phone
  being attached** (APK 25 is kept beside it as
  `donggeurami_town-release.prev-2026-11-13c.apk`).
- **The changes Hajun asked for later that night, having tried APK 26**
  (said where each is, above: the tag on the plate and the panel's bare
  top, the bigger cross with no disc behind it, "No friends." and "All
  Caught Up.", Add Friend's hint and its "My ID" right under the search
  bar, the boxes' tops kept on the screen over a keyboard, and a press off
  the panel shutting it as it lands and passing on) **are not in APK 26**:
  the app's own, nothing of the server's or of what is sent, so they went
  out with the next build, **APK 27**, in the small hours of 2026-11-14,
  with Bridge Wars' changes of that night ("Games"; built by the session
  that made those, and not installed, no phone being attached). Seen in
  the desktop build, scripted, against a
  server on this PC and offline: the tag on the plate online and the name
  alone offline; Add Friend with its hint, one row for nobody found and
  two for two; with a phone's keyboard written in over the lower 55% of
  the window, the short box whole above it and the taller one with its
  top at the top of the screen; a finger put down off the panel shutting
  it before the finger came up, where the stick is, where the jump button
  is, on the gamepad (the games up with that one tap) and on the speech
  bubble (shut, and not opened again); and a tap on the panel leaving it
  up. **That the stick and the jump button have that touch is the code's
  and a test's of the frame's order, not a run's**: on desktop they go by
  the mouse, which a script cannot hold down there, and a phone has not
  been tried.
