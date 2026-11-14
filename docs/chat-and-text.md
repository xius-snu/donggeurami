# The chat, the fonts, and what may be written

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## The chat is its words alone until you type, and a bubble of things to say

Since the night of 2026-11-14, a few hours after the chat itself ("The
chat", below: how it was until then, and how it still is while a line is
typed). Hajun, having played it on the phone: "I think I don't want the
permanent dark. Chat box for both the global chat and the Bridgewaters game
chat, but rather just the text and the. Box should only appear when the
when you are typing a message. And when. It's idle, um, the older line
should be drawn fainter because it's partway through fading. And the input
text box and the send button, they should not appear until you press the
the chat icon on the right bottom right corner. And instead of being like
the first text starts at the top of the rectangle and makes its way
downward, no, it should be like every new text starts at the same line.
Even the first message stays at the same area, and it goes up and fades as
new messages come." And, of the buttons: "the bottom right chat icon is too
big of a button to be used for the global chat. So make a smaller circle
above the current chat icon, a little bit to the right, and. That button
can be in the same location for both the Bridgewaters minigame and the
lobby map. And that button is used for the chatting. Opening the chat. Box
input." And, typed after it: "the current larger chat icon button should be
used for a quick chat selector for preset chat messages that flannel out"
(dictated: I take it for "fan out"). `src/chat.rs`, `src/quick.rs`,
`src/hud.rs`. Nothing of `shared/` or the server changed: protocol 16
still, no deploy. APK 31.

- **With nothing typed the chat is its words alone.** No box behind them
  (`chat::Backdrop`, hidden), no box to type into (`InputBox`, hidden, its
  room kept, so that nothing moves when it comes). The words take no
  press: a finger put down on them looks round, and the wheel over them
  zooms. Each line has a darker shadow than other words on the screen
  (`colours.chat.shadow`, `#000000b3`, mine: it is all that sets them off
  from the world).
- **The newest line is always on the same line**, the lowest of the room
  the lines have (four in the town, three in a game, where they were): a
  spacer over the lines takes whatever room they leave, so the first thing
  said is written there too, and each line after moves the others up.
- **A line fades from the moment it is said** (`chat::faded`): by
  `1 - (age / 12 s) ^ 2.2`, in 40 steps, so it is nine parts in ten there
  after 4 s, three quarters after 6, half after 9 and gone at 12, and of
  any two lines the older is the fainter, as asked. Its words, each piece
  in its own colour, and its shadow, written only as it comes to its next
  step (a `TextColor` written lays nothing out again in Bevy 0.19: it is
  read as the frame is drawn). `LINE_SECS`, `FADE_BEND` and the steps are
  mine. When each line was said is kept by its number (`Drawn::said`), so a
  line drawn again as the file changes is as old as it was.
- **A press on the small keyboard, bottom right, begins a line**
  (`hud::ChatButton`; T or Enter on desktop): then the dark box is behind
  the lines, every line kept is in full strength and scrolls under a
  finger or the wheel, and the box to type into is under them with its
  dart. Said, or let go of (Escape, a press anywhere else, or the
  keyboard's button again, which only lets go: `Typing::shut`), it is its
  words alone again, back at the newest line.
- **The keyboard's button** (`hud::CHAT_BUTTON`): 9 shares of the short
  side across, where the big speech bubble is 14; 1.6 over the bubble and
  its middle 5.5 to the right of the bubble's; in the same place in Bridge
  Wars, over the bridge button. Its picture, a keyboard drawn in code
  (`hud::keyboard`), is mine: none was named, and a second speech bubble
  over the big one would not have said which of the two types. The three
  dots that used to stand there in Bridge Wars are off (`hud::DOTS`); put
  back, they would stand over this (`ARENA_DOTS`, moved).
- **The big speech bubble fans out things to say** (`src/quick.rs`,
  `hud::QuickChatButton`), in the town: the file's `words.chat.quick`, the
  first six that are lines anyone could type, each on a grey plate
  (`colours.chat.quick`, `quick_text`), one over another from just over
  the face, their right ends on a curve that leans in over the bubble,
  each coming out of the bubble 35 ms after the one under it. A press on
  one says it as if typed (`chat::Say`: asked of the server online, which
  tells the room; written in your own chat offline) and shuts the fan; so
  does a press on the bubble again, or anywhere else, which is whatever it
  landed on's as well. You can walk with it out. The server lets anyone
  say four lines at once and then one every second and a half, typed or
  not. **What it offers is mine** ("Hello!", "Follow me!", "Anyone for
  Bridge Wars?", "Thanks!", "Good game!", "Bye!"), for Hajun to change in
  the file; so are how it fans, how big, and how fast. **Bridge Wars has no
  such bubble** (the bridge button is about there) and nothing was asked of
  it: there the chat is typed, or not said.
- **Checked.** Unit tests: 299 of the game's (a line's fading, and that
  the older is fainter; when each was said, kept; the keyboard's picture;
  its button clear of every other, the gamepad and the minimap on screens
  from 4:3 to 19.5:9; the fan's plates clear of the buttons, on the
  screen, and coming out of the bubble; what the file offers being lines
  as written), 65 of `shared/`, 50 of the server's. The desktop build,
  scripted, offline: the town with no box; the keyboard's button putting
  the box and the dart up, and letting go at a second press; "hello
  there" written on the lowest line, and three more said three seconds
  apart each fainter than the one under it, enlarged to see; the fan half
  way out and out; "Follow me!" said by a press on it; the fan shut by the
  bubble and by a press on the world. Bridge Wars, offline: the keyboard's
  button over the bridge button, a line typed and the chat idle after it.
  Two copies against a server on this PC: "Hello!" from one's fan and a
  line it typed, and "Follow me!" from the other's, each in both chats.
- **Not checked.** A phone: APK 31 is built and not installed. So how the
  bare words read over the town and over Bridge Wars' sky on a phone's
  screen (they are 2.5 shares tall, as they were in the box), the fan
  under a thumb, and the keyboard's button beside the gamepad. Scrolling
  the lines back while typing was not run again; what it is dragged on
  changed (the dark box, where it was the lines' own). Nothing was said
  through the VM.

## The chat

Since 2026-11-14 (Hajun: "I think there should be a global chat in the
Bridge Wars game, with notifications that pop up based on game events: When
anyone dies: "Bob fell off the world." When someone kills someone: "Bob
killed Hannah with a Stone Sword," or "Bob knocked Hannah off the world."
Team-specific notifications, like "Bob purchased Heal Pool for your team."
All player names in these messages should be colored with their team's
color. The chat box can go on the left side middle of the screen, between
the joystick and the name tag, kind of like minecraft server chatbox style.
There should be a text input box right below the chat box with a send icon
on the right. Pressing it lets you type a custom message, which if you
press "ENTER" on your phone OR press the send icon sent to everyone and
shows your name followed by the text you typed. This chat box should also
be in the main lobby ... The total height of the chatbox shouldnt be too
tall but scrollable, and should be semi transparent so its not too
obstrusive"), in `src/chat.rs`, `server/src/chat.rs` and
`roundtown_net::chat`. **`PROTOCOL_VERSION` 16**, with the rest of that
day's work ("Games").

- **Where it is** (`chat::place`): down the left of the screen, 50 shares
  of the short side wide, a box of lines and under it a box a line is
  typed into with a dart at its right end. In the town it begins 35.5
  shares down and shows four lines; in a game of Bridge Wars, its waiting
  room too, it begins 39 down, under your name, your health and the four
  counts, and shows three, so as to end over the stick in both (57 down:
  the stick's disc reaches up to 58 held from as high as it may be). Not
  at home, in House Builder, under the sign-up's pages or while you talk
  to someone. See-through black (`colours.chat.back`, `field`), the words
  2.5 shares, whole lines only: the box is cut at the lines' own room
  (`OverflowClipMargin::content_box`) and is a whole number of lines tall,
  or the feet of the line scrolled off its top showed along it. The names
  over heads are under it. Bridge Wars' shop windows lie over its right
  end on a screen of sixteen by nine, not on a phone's.
- **What it shows** (`chat::Log`, the last 60 lines of the room you are
  in): what anyone in the room says, "Name: words", and, in a game, what
  the game says (below). A name is in its team's colour in a game
  (`colours.arena.team_*`) and in `colours.chat.name` in the town. A
  finger dragged on the box, or the wheel over it (which does not zoom the
  camera there), scrolls back; at its newest line it stays there as lines
  come. Coming into another room, the town, a game, another game, begins
  it afresh.
- **Saying something** (`chat::act`): a press on the box, or on the speech
  bubble among the town's buttons (`hud::ChatButton`, which did nothing
  until now), or Enter or T on desktop, takes the keys (`signup::NameBox`)
  and asks a phone for its keyboard, and the chat slides up over it
  (`LiftsForKeyboard`). You stand still while it is typed into
  (`hud::DialogUp`). Enter on either keyboard, or the dart, says it; a
  press anywhere else lets go of it as it lands, and is the stick's or a
  button's as well, what was typed kept for next time; Escape too.
- **The rules** (`roundtown_net::chat`): a line is 80 widths at the most
  of what the game writes, English, Korean and Vietnamese ("The font is
  Inter"); four at once and one more every second and a half (`BURST`,
  `EVERY`). The server tells it to everyone connected in the speaker's
  room, the speaker among them (`Ask::Chat`, `Tell::Said`, with the name
  the server has them by): a channel of the town, 16 people at the most,
  or a game from its waiting room to its end. **Nothing is kept of it**,
  on the server, in the database or on a device past the room, and what is
  said is not written to the server's log. Offline a line is written on
  your own screen and goes nowhere.
- **What a game of Bridge Wars says** (`chat::Feed`, `Tell::Feed`,
  `arena::feed_line`; the server's online, the device's own in a game
  played offline, each device writing it in the file's words,
  `words.chat`): "{name} fell off the world."; "{killer} knocked {name}
  off the world.", for a fall within five seconds of their last hit by
  another (`CREDIT_SECS`, Minecraft's); "{killer} killed {name}." with a
  bare hand, and "with a Basic Sword", "a Silver Sword", "an Aura Sword",
  "a Fireball" or "Dynamite" (Hajun's "Stone Sword" being whichever sword
  it was); "{name} purchased {ware} for your team.", to that team alone,
  for whatever the upgrade shop sells, by its card's name at the step
  bought; and, mine, "{name} destroyed {team}'s anchor.". A zombie's kill
  is said, in the zombies' colour; a zombie's own death is not.
- **Common swear words are starred out** (`roundtown_net::text::masked`,
  since the evening of 2026-11-14; Hajun: "add basic word filter for common
  swear words so they turn to ****** of that word's length"): a star to
  each letter, in a line of chat and in a message to a friend, by
  `chat::line` and `social::message`, so the server does it to whatever it
  passes on (deployed at 09:52 UTC, protocol 16 still, so APK 29, built
  before it, is filtered online; the binary before kept on the VM as
  `roundtown-server.prev-2026-11-14c`). English words whole (a stem inside
  a word, drawn-out letters squeezed; "class" and "cockpit" left alone;
  words run together with a capital to each, "BigDick", each by itself),
  Korean where it stands in a run of Hangul with its shortened letters
  ("다시발견" let be), Vietnamese words as spelt with their marks (without
  them most are everyday words: "cac ban" is "các bạn"). Basic: nothing
  spelt with digits or signs, or with spaces between its letters, and
  whoever means to get a word past it will. The lists are mine (`STEMS`,
  `WORDS`, `HANGUL`, `HANGUL_FINE` in `shared/src/lib.rs`): 보지, 자지,
  새끼 and 미친 alone are not in them, nor "cc", "cl" and "dm", being
  everyday words. Checked against a server on this PC with two copies, the
  lines said to it raw, as an app from before the filter says them: both
  were told "what the **** is this ********", "아 ** 진짜 **" and "***
  mẹ, ngu **", and "class pass cockpit, 다시발견, các bạn" as it was. No
  line has been said through the VM.
- **A name with one of them in it is refused** (`text::swears`,
  `profile::name`, `profile::rude`; an hour later, Hajun: "yes add the
  username filter too"): stars would be no name to go by, so it is no
  name, on both sides. The server takes none from a device (deployed at
  10:15 UTC, protocol 16 still; the binary before kept as
  `roundtown-server.prev-2026-11-14d`), so an app from before, which would
  send one, keeps the name it had online. In the app (APK 30) Save stays
  grey and the box's line says "Pick another name." ("The first time, you
  sign up"). **A name kept on a device from before, with one in it, is no
  longer a name there**: the sign-up's pages come up at the next launch
  with it in the box, to be changed (`Profile::set`); none of the server's
  three accounts has one. What the server makes up for a new account, and
  what the computer's players are called, pass (tests keep it so). Seen in
  the desktop build, scripted: "BigDick" and "씨발놈" refused on the name
  page, Enter doing nothing, "Dickens" taken; "shithead" refused in Edit
  name and "Hajun2" not, the dialog as tall either way. Not on a phone.
- **It is the first thing anyone writes that strangers read**, and has no
  block and no report (and until that evening no filter for words, which
  is a basic one), as the friends' messages have none: whoever is in your channel of the town, or your game, reads it.
  Hajun asked for it as it is; the plan (`MULTIPLAYER.md`, "Only once it
  is added") has those down as what a release to the shops wants, Apple's
  asking them of anything users write to each other. The game is not
  released: the server has three accounts.
- **Checked on 2026-11-14.** Unit-tested: a line and how many may be said
  (`shared`, `server/src/chat.rs`); what the game says, by seat, and to
  whom (`the_chat_is_told_who_killed_whom_with_what_who_fell_and_what_a_team_bought`);
  each line's words and colours (`arena`); the box's place and size. **In
  the desktop build, scripted**: six lines said in the town and scrolled
  to the newest, a line of eighty letters typed with its end in sight; in
  a game offline, a line said in your team's colour, four purchases for
  the team, "Hajun fell off the world." and "Mochi killed Hajun." with
  each name in its team's colour. **Against a server on this PC, two
  copies**, teammates: each other's lines in the town and in the game, the
  team's purchases on both, a fall on both. **Not on a phone**: its
  keyboard, the chat's lift over it, and how the box sits beside a thumb
  on the stick are unseen; no line has been said on the VM.
- **What is mine**: its size and place, three lines in a game; that it
  begins afresh in each room; the 80, the four and the second and a half;
  that a purchase is said to its team alone and only for upgrades; the
  five seconds; the line about an anchor; that zombies' deaths are not
  said; T and Enter on desktop; that the wheel over it scrolls it.

## The font is Inter, with Pretendard for Hangul

Since 2026-11-13 (Hajun: "I want to change the font of most things to the
font "Inter" Regular"), every word on the screen is written in Inter
Regular (`src/font.rs`), in place of the font Bevy comes with, FiraMono, a
fixed-width font cut down to plain ASCII. Since 2026-11-14 Korean is
written too, in Pretendard Regular, and names, chat and messages may be in
English, Korean or Vietnamese (Hajun: "i need korean text to be able to
work in the global chat not just english. same for username and send
message", and "also vietnamese text as well").

- **Nothing that draws words names a font.** Each takes Bevy's default
  (`TextFont::default()`), which is whatever font sits under the default
  handle; `font::plugin` puts Inter there as the app is built
  (`AssetId::default()`, after `DefaultPlugins`, whose text plugin puts
  its own there), so every word is Inter's from the first frame, the
  opening's cover included, with nothing loaded later to swap in. So
  "most things" came to everything: the bar, the dialogs, the shops, the
  names over heads, the headlines, Bridge Wars' numbers. Should one thing
  want another font, it names it in its own `TextFont`.
- **The file is compiled in** (`include_bytes!`), as `assets/ui.ron` is:
  `assets/fonts/Inter-Regular.ttf`, 411,640 bytes, the static Regular of
  Inter 4.1 (`extras/ttf/Inter-Regular.ttf` of the release at
  github.com/rsms/inter), under the SIL Open Font License, which is
  beside it (`LICENSE-inter.txt`). A phone needs nothing packaged for it:
  both files are left out of the APK (`assets-left-out.txt`; a test
  keeps it so, and that the font built in is Inter's Regular).
- **Every text asks for fixed-width digits** (`font::sized`, `tnum`,
  which every text's `TextFont` is made by: `hud::words`, `builder::label`
  and the rest). Inter's digits are of different widths, a 1 narrower
  than an 8, where FiraMono's were all one: a clock or a count would
  shift a hair as its digits changed, the whole line with it where it is
  centred. A test keeps every text asking.
- **Hangul is Pretendard's, as a fallback** (`font::fall_back_on_hangul`,
  since 2026-11-14). Inter has Latin, with every letter Vietnamese has,
  Greek and Cyrillic, and no Hangul. `assets/fonts/Pretendard-Regular.ttf`
  (1.3.9's static Regular, `packages/pretendard/dist/public/static/alternative/`
  of github.com/orioncactus/pretendard, 2,725,828 bytes, under the same
  SIL Open Font License, `LICENSE-pretendard.txt` beside it) is compiled
  in beside Inter and left out of the APK as a file: the font Korean
  screens are mostly set in, built on Inter's Latin to go with it. **It
  is not the game's font and draws no Latin**: Bevy 0.19 lays text out
  with parley, which, for a letter the font asked for has no shape for,
  asks its list of fonts (`FontCx`, the `fontique` crate) for whatever
  that letter's script falls back on; `font` puts Pretendard in the list
  and names it for the script Hangul. So "하준", over a head, in the chat
  or on a plate, is Pretendard's, and the Latin on the same line Inter's:
  every screen there was is as it was, to the pixel. `fontique` is a
  dependency of the game for this, pinned to the version Bevy's text
  builds (`=0.9.0`), with no more of it on than Bevy has (not `system`,
  which would go looking through the device's own fonts). The APK's
  library is 2.7 MB bigger. **A word in `assets/ui.ron` may be Korean
  now** (the tests that kept its words to plain ASCII keep them to what
  the fonts draw).
- **What may be written** (`roundtown_net::text`, `shared/src/lib.rs`, one
  rule for names, chat and messages, on both sides): what is on an English
  keyboard; Latin-1's accented letters and signs; the rest of Vietnamese's
  letters (ă, đ, ĩ, ũ, ơ, ư and every vowel with every tone, U+1EA0 to
  U+1EF9); curly quotes, an en and an em dash and an ellipsis, which a
  phone's keyboard writes by itself; and Hangul, all 11,172 syllables and
  the 51 letters as they are typed alone ("ㅋㅋ"). Nothing else: no emoji,
  no Hanja, no other script, and of Latin Extended-A only Vietnamese's
  (a Turkish dotted I is lowered differently by Rust and by Postgres,
  which would let two accounts have one name and tag). A test in `font`
  has every one of them drawn by Inter or by Pretendard
  (`every_letter_the_game_writes_is_drawn_by_inter_or_by_pretendard`).
  **Lengths are in widths** (`text::length`): a Hangul syllable or letter
  is two, anything else one, so that a name or a line is as long on the
  screen either way and every plate, row and board made for 16 Latin
  letters holds a Korean name: a name is 16, a line of chat and a message
  80. **A letter is the same letter however it was typed**
  (`text::clean`, Unicode's NFC, the `unicode-normalization` crate, which
  Bevy builds already): Windows' own Vietnamese keyboard sends a vowel
  and its tone mark as two, and some keyboards a Hangul syllable as its
  parts; they are put together before anything is kept, sent or looked
  for, so that "Hải" is found whichever keyboard wrote it. What the game
  does not write is left out, a new line or a tab is a space, and the
  spaces round it are trimmed; `profile::name`, `chat::line`,
  `social::message` and `social::sought` all go by it, and cleaning what
  is clean changes nothing, so the server does it again to whatever it is
  sent. **No database change**: `players.name` and `messages.body` are
  `TEXT`, and the VM's database is UTF-8 with `en_US.UTF-8` (looked at
  before the deploy: `lower('ĂĐƠƯ HẢI 하준')` there is `ăđơư hải 하준`).
- **Typing it** (`signup::NameBox`, the one box everything is typed
  into). **On desktop** the system's input method composes Korean, and
  Vietnamese by Telex: what it is composing is shown after what is typed
  (`Ime::Preedit`, `NameBox::shown`) and typed once it is done
  (`Ime::Commit`). Keys and what is composed are read from one list, in
  the order they came (`WindowEvent`, `type_name`): a syllable is done by
  the key typed after it, a space say, and read as two lists the space
  went in before its syllable. A box done with by a button while a
  syllable is still being composed takes the syllable with the rest.
  **On Android** the keyboard writes into the activity's own text, which
  is read every frame as before; a syllable being composed is in that
  text as far as it has got, and the text is written back only when
  something is taken out of it (what the game does not write, or what is
  past the box's end), with the caret counted as Java counts, in UTF-16
  units, where it was counted in bytes, which for ASCII were the same
  (`java_length`). **An iPhone's** keyboard sends each letter as a key,
  and composes by taking the last letter back and sending the next,
  which the box does as it is told. A box takes as many widths as its
  kind is long, so a ninth syllable of a name is not taken.
- **Checked on 2026-11-14.** Unit-tested: the rule, the widths, the
  putting together (63 in `shared`); a search and a message in Korean and
  in Vietnamese however typed, two of a Korean name with different tags
  (49 in `server`); the box, by units, with something being composed (292
  in the game); the fonts, the fallback and that every letter is drawn.
  **In the desktop build, scripted**, by day and by night: "안녕하세요
  hello ㅋㅋ", a Vietnamese sentence whole and one typed in pieces, a
  syllable shown while it is composed and then typed with its space in
  its place, an emoji and a Hanja left out, forty syllables and no
  forty-first, all in the chat, Hangul and Latin on one line with one
  foot; "하준" on the plate; the sign-up's name page with "동그라미<3"
  and with eight syllables. **Against a server on this PC, two copies**
  named 하준 and Nguyễn Hải: both names taken by the server, over each
  other's heads, in each other's chat in the town and in a game, and in
  what the game says ("하준 fell off the world."). **Not with a real
  keyboard's input method on any device**: Windows' Korean and Telex
  keyboards, Android's and an iPhone's are all untried, the script
  having sent what they are documented to send. Not in the friends'
  panel by hand (its search and its message go by the same box and the
  same rule).
- **What is mine**: Pretendard, and as a fallback rather than in Inter's
  place; a Hangul syllable counting two, so 8 to a name and 40 to a line;
  that the letters alone are allowed, in names as in chat (the plan,
  `MULTIPLAYER.md`, would keep them out of names once there is a filter
  for bad words, which there is not); Latin-1 and the curly quotes
  besides what was asked; that what cannot be written is left out rather
  than refusing the whole; the name page's words.
- Inter is proportional and a little narrower than FiraMono was at one
  size, so every line is shorter than it was; nothing was resized for it.
  Seen in the desktop build, scripted, on the screens that day's runs went
  through: the bar, the sign-up's three pages, the settings, the game
  selector, House Builder's lobby, Bridge Wars' board, counts, buttons
  and both its shops' windows, and the friends' panel. Not opened in it:
  House Builder's shop, stars and results, and what is said when you talk
  to someone. Not on a phone.
