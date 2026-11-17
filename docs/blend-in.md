# Blend In

> Blend In, the hide-and-seek game among the town's people, from its first
> day. Dated entries, newest first: what was built, why, in whose words,
> whose call each thing was, and what was and was not checked, and where.
> Its rules are `roundtown_net::blend_in` (`shared/src/blend_in.rs` and
> `shared/src/blend_in/`); the server plays them (`server/src/blend_in.rs`);
> each device draws them (`src/blend_in.rs` and `src/blend_in/`). How its
> table fills, starts and pays is the tables', which the four games of
> 2026-11-18 share.

## Contents

1. Later on 2026-11-18, after the game was checked: the crowd is sent by the rule a person's device sends by, nothing said from the hide to the end of the seek is passed on, a tap on the board no longer decides a round, and whoever leaves comes last
2. Since 2026-11-18 Blend In is played online: two to four people, each hiding once among twenty-eight of the town's people while the others look for them

## Later on 2026-11-18, after the game was checked: the crowd is sent by the rule a person's device sends by, nothing said from the hide to the end of the seek is passed on, a tap on the board no longer decides a round, and whoever leaves comes last

A check of the game as first built (the entry below), the same day and
before anything of it was merged, found nine things wrong; each is fixed
here, or, where the fix is not the game's to make, asked of whoever's it
is (below). Nothing of Hajun's changed: every call in this entry is mine.

- **The crowd is sent by the rule a person's device sends by**
  (`Beat`, `server/src/blend_in.rs`). A device marks its body as
  standing still once it has stood one frame, to the centimetre and to the
  turn, and says so at its next beat (`net::tell_where_i_am`); the crowd
  was marked still whenever it stood where it had last been sent, however
  it had turned since, so that one turning where it stood was sent still
  and facing somewhere new, which no device sends, and each of its stops
  came as two sendings, the place and then the stop, where a device's
  comes as one unless its beat falls on the very last step. Each of the
  crowd's bodies now goes by the device's rule, on a beat of its own as
  before, its `Spot` written only with something new. A test plays two
  minutes of the crowd through the crowd's sending, through a device's
  (its rule written out), and through the first build's: no update marked
  still otherwise than the frame before says, nothing sent of a body
  standing, and its stops in two sendings 0.22 of the time for the crowd
  and 0.23 for a device; the first build sent 992 updates still while
  turning, and 0.91 of its stops in two.
- **Nothing said is passed on from the hide to the end of the seek,
  whatever a device sends** (`Rules::hushes`, which the chat asks through
  `Tables::hushed`). The device put its chat away then (the design), but
  the server passed on any line, under the name its speaker's body goes
  by, which from the hide is the hider's disguise: a line the hider sent
  as the choosing ended, heard once they were dressed, would have told
  everyone who they are; and a device made to could have talked all
  through the seek. The server now drops such a line, and logs that it
  held one back.
- **A tap on the board hurries only the choosing and the round shown**
  (`Game::hurry`). With skipping allowed, as the VM has it while people
  play there, a tap on the board in the seek ended it with the hider's
  point, whoever tapped, and one in the hide cut the hider's time to get
  into place. The hide and the seek now take their time whoever taps.
- **Whoever leaves comes after everyone who stays** (`Game::outcome`), as
  leaving a game of Bedwars loses it. Paid by points so far, one of two
  at none each, the hider about to be found, could leave, the game over
  at once as a draw, and have their fee back like the one who stayed. Now
  the one who stayed wins, 100 coins, and the one who left has nothing;
  among those who stayed, and among those who left, by points, ties
  sharing. The end's headline goes by the same places, the table's
  ("Mochi wins!", not "A draw!").
- **The window put away by its cross is up again at the next table**
  (`Blend::shut`). Which round's window the cross had put away was kept
  apart from the table, and never forgotten: after Play again, or in any
  later game, the round of the same number never had its window, Ready
  could not be pressed, and everyone waited the choosing's 30 s out.
- **A device opened again in the middle of a seek wears its disguise on
  its own screen** (`follow`). What it wears was told before the round had
  come, and nothing dressed it after: its own body was drawn in the
  account's look, and its plate said the account's name.
- **Quick taps on the window are each kept** (`Blend::asked`). Every
  choice is answered with what is worn as it stands; an answer to a
  choice made before the last one put the window back on it, and a tap
  made then stepped on from there and was lost. The answers are counted
  in, and only the last of them is shown.
- **The hider's row among the points has a thin stroke round it**, as your
  name has on its plate in a team's colour (`stroked`, `screen.rs`): in
  the game's violet it could hardly be read on the see-through plate by
  day.
- **The end's headline is up two seconds**, gone as the table's results
  come up over it two seconds after the end; and it is taken down as they
  do (it showed through their see-through window).

### Asked of others: not the game's to change

- **The tables' `src/net.rs`, one line**, in `tell_where_i_am`:
  `sent.was = Some(Spot { still: false, ..spot });` in place of
  `sent.was = Some(Spot { still: spot.still, ..spot });`. As it is, a
  device standing still flips its body's `still` every frame, as the
  builder found, so that a person standing is sent several times a second
  and one of the crowd standing never is: the one tell left in how often
  a body is sent. The crowd is sent as a device will be with this line;
  it is not changed here because it changes what every device sends
  everywhere, Bridge Wars too (less of it: nothing but bandwidth).
- **The rooms, or the tables: the hider's body jumps, and is the body it
  was.** As the hide begins the server stands the hider's body where they
  hide (`players::place`), and every device is sent it going 25 m or more
  from the pond at once, while the crowd's bodies walk; and it is the
  very body the hider sat in, under their account, while everyone waited
  and chose. A device made to read the network knows the hider from the
  start; only the seekers' black screen keeps it from the game as it is
  played. What would end it: every body in the room kept from the
  seekers' devices from the hide to the seek (they see nothing then
  anyway), and shown to them again at the seek as bodies new to them, in
  an order thrown; a body afresh for each person alone would still jump.
  Until then this is how it stands.

### Changes outside the game's files

Both additive, for the chat's hush, and nothing else of either changed:

- `server/src/tables.rs`: `Rules::hushes`, false unless a game says
  otherwise (only Blend In does), and `Tables::hushed(room)`, which asks
  it of the table in a room, never true for any room but a table's; and a
  test of it.
- `server/src/chat.rs`: `hear` drops a line said in a room that
  `Tables::hushed` says is hushed (taking the tables as `Option`, so that
  a world without them runs as before). The town's rooms, Bridge Wars'
  and House Builder's are never hushed.

### Calls of mine besides

- A round first seen part way through, on a device begun afresh in the
  game, says nothing across the screen: the table says the game's name
  there then, and on the reopened copy below the game's "Seek!" stood
  over it. The plate top centre says what is going on.
- Whoever left is placed after everyone who stayed whatever their points,
  and the end's headline is said by those places, the table's; before
  they come (they come with the end), by points.

### What was checked, and where (this entry)

- **Tests**: the rules' (`cargo test -p roundtown_net`, 153 passed,
  3 ignored): whoever leaves placed after everyone who stays, of two at
  none each and of three and four; a tap hurrying the choosing and the
  round shown and never the hide nor the seek. The server's (`-p
  roundtown_server`, 93 passed, 4 ignored): the crowd sent what a device
  would send of the same walk (above); nothing passed on from the hide to
  the end of the seek, at each part of a round, and `Tables::hushed` for
  a real table of Blend In and nowhere else; the plate's tap; one of two
  who leaves at none each paid nothing and the other 100. The game's
  (`--lib`, 385 passed): a device opened again part way through a
  round dressed as it wears whichever word comes first, and not while
  everyone chooses; an answer to a choice made before the last never
  shown over the last; the window put away by its cross staying away at
  the same table begun afresh, and up at the next; the end's words by
  places.
- **Three hidden runs of two copies against a server on this PC**
  (`RT_BACKGROUND=1`, port 5481, `RT_ALLOW_SKIP=1`, no database), scripted
  by a throwaway taken out again (kept outside the tree,
  `RoundTown-wt/scratch-blendin/kept/scratch_fix.rs` and
  `fix_online.sh`). The first was stopped early, its script's taps going
  down and up in one frame, but showed the chat as below; the second, by
  day, and the third, at night, each played two whole games:
  - choosing: three taps on the right arrow two frames apart took the
    look three on, a boy's to a girl's; on this PC the answers to the
    first two came in the same frame, so a tap between two answers was
    seen in the test only. A line said while choosing was heard by both
    ("said a line ... to 2" in the server's log);
  - the cross: Ready, and the window put away for that round; at night,
    on the copy that pressed it (never opened again), the window up for
    the next round, and after Play again for the next game's first;
  - the hider's lines in the hide and in the seek held back by the server
    ("while it is hushed: held back"), heard by nobody;
  - a seeker's tap on the plate 1.5 s into the seek: the seek went on,
    146 s of it left, nobody's point;
  - the hider's row among the points, stroked, by day and by night,
    enlarged;
  - the seeker of the second round, its app ended and opened again 3 s
    into the seek: put back in its seat ("is back, put back in seat 0"),
    its plate the name it went by with "(you)", its body in that look, by
    day and by night; by day the table's "Blend In" and the game's "Seek!"
    one over the other on it, at night (after the change above) "Blend In"
    alone;
  - the first game's end, "A draw!", gone as the results came up two
    seconds later, 50 each (1,000 coins each after it);
  - Play again; in the second game the hider left 2 s into the seek at
    none each: "Bravo wins!", 100 and nothing (1,050 and 950).
  - What one device was sent of each body over a seek of 7 s: each of the
    crowd's 8 to 51 times, 3 to 6 of them marked still, one to a stop;
    the hider's 50 times, 14 of them moving and 19 marked still while
    standing: the flip in `net.rs` asked above.
  - Nothing panicked or logged an error; nothing was left running.
- The pictures are in `RoundTown-wt/scratch-blendin/shots/fix/`, the
  runs' logs in `scratch-blendin/out_fix_first`, `out_fix_day` and
  `out_fix_night`.
- **Not**: a phone or an iPhone; the VM; a database; three or four people
  since these changes; a slow network.

## Since 2026-11-18 Blend In is played online: two to four people, each hiding once among twenty-eight of the town's people while the others look for them

Hajun, 2026-11-18 (the whole message is in the games' design,
`specs/design-games.md` in the worktrees): "1 random hider chosen as the
person walking around pretending to be an NPC/AI. 1~3 other people trying
to find who is the person, by punching with a aura sword. This game min
player count is 2, so if there 2 ppl, it will not fill the 2 slot with 2
ai's but rather start game with just 2 real ppl. There is a time limit, and
incorrect guesses cost you time by stunning/blinding etc. Both hider and
seeker can Choose (must choose within the timer of maybe 30s, autostart
when both have chosen): Guy/girl + skin of character, Name. Map =
roundtownmap. You get point as hider for surviving until timer runs out,
And you get point as seeker for finding the hider before timer runs out.
You swap until every one has tried hiding atleast once. Winner is
determined, or given a draw if both found each other or both couldnt find
each other (for 2 players). (i think freecam is ok for blend in mode,
similar to main lobby camera)"; and of every game: "each of these games
have a coin entrance fee and payout. Generally should be above 50% to win
net positive coin, and if not, net negative coin"; "the details i didn't
mention are yours to decide to make each game polished and fun".

**Whose calls.** Everything below that is not in Hajun's words is a call
of ours, not Hajun's, made in one of three places, said where it matters:
the games' design (2026-11-18, *the design*), the first build of the
rules, which tuned some of it and said why (*the rules*), and this build
of the server's and the device's sides (*mine*).

### The game, as it is played

- **The selector's card** "Blend In", "Hide among the town's people", 50
  coins. **Online only**: offline the card is blocked out and says "Online
  only" (nobody to hide from), and a device never plays a game of it
  itself. **People only, two to four**, never the computer: the countdown
  begins 15 s after the second person sits down, at least 8 s after each
  later one, or at once with four (the design; the tables do it).
- **Where**: the town's own map on a venue of its own (`Venue::BlendIn`):
  its houses, lamps, fountain and pond, and none of its people, buses or
  House Builder. The orbit camera, as in the town ("freecam is ok"). No
  car, no game selector, no friends, no talking to the crowd, and the
  fountain throws nobody (the design).
- **The ring**: play is kept inside 70 m from the middle, the ring road's
  outer edge (the design; the rules measured the white line there at 70.3
  to 70.7 m). A faint dashed line lies on the ground at 70 m, unlit, in
  the plate's grey (the design), in dashes of 1.6 m with gaps of 1.6 m,
  0.22 m wide (mine); your body is kept a body's width inside it as you
  walk at it (mine: seen at 69.55 m in the run below), and the server puts
  back anyone seen 1.5 m past it, at most once every 2 s (the rules).
- **The crowd**: 28 of the town's people (the town's 32 bodies with four
  people), run by the server so that every device sees the same crowd in
  the same places, walking from the moment the table opens (the design).
  Each is a boy or a girl by the coin, in one of that side's three looks,
  with a name from that side's list, no two alike and none a player's
  (the design). Their mind is the town AIs': a walk of 2 to 9 m over dry
  land that never steps down what it could not climb back, then stand
  (the design); each with its own wait between walks, about 2 to 2.9 s
  and 2.4 on average, as busy as the town's own were measured to be, its
  own odds of standing a whole turn or only turning, its walks ending
  1.0 to 1.6 m from anyone, all thrown again every round (the rules, in
  place of the design's 2 to 4.5 s, which measured a third lazier than
  the town). Each is sent to the room as a person is (`Member` with `ai`
  false, an id from the one range every body's is thrown from, a name of
  its side's, a look of it, its `Spot`, since the entry above by the rule
  a person's device sends its own body by), so that each device draws
  them as it draws people: the same smoothing, the same walk read off the
  motion, a name over each head (the design).
- **Rounds**: as many as there are people, each hiding once, in a thrown
  order (the design). Each round:
  1. **Choosing**, up to 30 s, over as soon as everyone has said Ready
     (Hajun's "maybe 30s, autostart when both have chosen"). A window as
     every window is, the friends' panel's: "Round 2 of 3" and the time
     left; "You're hiding: look and sound like one of the town's own" or
     "You're seeking: find who isn't one of them"; your look between two
     arrows that step through all six looks (the sign-up's pictures and
     arrowheads); your name in a box like the sign-up's; a dice beside it
     that throws one of the town's own names for the look's side; the
     players with a tick by each who has chosen and "hides" by the hider;
     and Ready, green, grey while the name typed is no name, "Waiting for
     the others" once pressed (the design). It opens on what the game
     gives you: the hider a guise thrown as one of the crowd's is, a
     seeker what they wore the round before or their own account the first
     time (the rules). Every choice goes to the server as it is made and
     is answered with what you wear as it stands; a name that is no name,
     or a seeker's that another seeker has, is turned down ("Pick another
     name.") and what you had is kept; the hider's is checked against every
     seeker's, and should a seeker take the hider's name, the hider is
     given another of the town's at the hide (the rules). Nothing chosen in
     time keeps what it opened on. Your account is never changed: what you
     are in the game is the game's.
  2. **The hide**, 15 s: everyone dressed as they chose, every body in the
     game under a new id and the crowd under new names and looks. The
     hider is stood 1.5 to 4 m from one of the crowd at least 29 m from
     every seeker (the rules) and told "Hide!" ("Walk like them, stop like
     them"); the seekers stand on the path round the pond, their screens
     black but for the plate, "The hider is getting into place" and the
     seconds, and cannot move (the design).
  3. **The seek**, 2:30 (the design): "Seek!" ("Find who isn't one of
     them") to the seekers, "Stay hidden!" ("The seekers are coming") to
     the hider. The seekers run at 10 m/s, and only in the seek; the hider
     walks at 7, as the crowd does (the rules). The seekers hold the aura
     sword for everyone to see, and a faint glow lies on the ground round
     each, so that the hider sees them coming (the design). The sword is
     where the town has its fist, and the punch's range is drawn under the
     seeker who is you (the design). A swing that lands on the hider finds
     them. One that lands on one of the crowd is a wrong guess: the one
     struck staggers back a metre over 0.35 s, turns to face the seeker
     and stands 0.8 to 1.6 s (the rules); the seeker cannot move and sees
     only a circle 2.6 m round them (Bridge Wars' blindfold's), "Wrong! 4"
     counting down: 4 s for the first wrong guess of the round, 7 for the
     second, 10 for the third and 3 more for each after (the design), a
     stun that ends with the seek (the rules); and everyone's chat says
     "Dubu struck one of the town's own." (never which one). A seeker's
     swing passes another seeker by; the hider has no swing (the design).
     The chat and the quick chat are off from the hide to the end of the
     seek (the design: a line said would give the hider away, and nobody
     can say anything while the others cannot); since the entry above the
     server passes on nothing said then either.
  4. **The round over**, 6 s: a tall soft column of light over where the
     hider stands (the design), and "Found by Dubu in 1:12!" or "Mochi
     stayed hidden!", the same in the chat. A point to the finder, or to
     the hider if the time ran out (Hajun's).
- **A hider who leaves**, or whose device is gone 20 s, gives up their
  round with no point ("Mochi's round is given up"), and someone who
  leaves before their turn to hide loses that round; with fewer than two
  people left the game ends, paid by points so far (the design), whoever
  left placed after everyone who stayed (mine, since the entry above).
- **The end**: "Mochi wins!", "Mochi and Dubu win!", or "A draw!"; then the
  table's results, a row to each person with their points and coins.
- **Coins**: 50 to join. Paid by place: of two 100 and nothing, of three
  110, 40 and nothing, of four 120, 70, 10 and nothing; those tied share
  the places they cover (two who both found the other, or neither: 50 each,
  the fee back) (the design, from Hajun's "above 50% to win net positive
  coin").
- **On the screen**: top centre a plate, as Bridge Wars' board is, saying
  what is going on and the time by the server's clock, "Seek 2:14" or
  "Stay hidden 2:14", with "Round 2/3" under it (the design); and in the
  other parts of a round "Choosing 0:24", "Hide 0:12" to the hider and
  "Hiding 0:12" to the seekers, "Next round 0:05" and "Results 0:05"
  (mine). Top right the points, a row to each person (the design), the
  hider's in the game's violet (mine). Top left your name, and from the
  first hide the name you go by with "(you)" under it, in its place (the
  design). Names over every head, the crowd's and everyone's, as the town
  draws them. Round the jump button the speech bubble, the chat's keyboard
  and, while you seek, the sword.
- **Desktop keys**: as in the town (WASD, Space, F swings, T or Enter for
  the chat); the window's buttons are clicked, Enter takes a name typed and
  Escape puts its box away.

### What nothing a device is sent says

Who hides is no secret (everyone is told who hides, not where). What is
kept from every device until the round is over is which body is the
hider's (the design): every body's id is thrown afresh each round from one
range, by dice sealed so that no id tells of another (the rules), whose
seed the server draws from the system's own randomness and never sends
(the rules asked it; `secret` in `server/src/blend_in.rs`); nobody is sent
as the computer's (`ai` false for all); the crowd's names are never a
player's and change every round; the bodies are dressed for the hide in an
order thrown afresh, the people's among the crowd's (mine, as the rules
asked); the seekers' bodies are named in what the room is sent from the
hide to the end of the seek (`BlendRound::swords`, so that each device can
put the sword in their hands), the hider's never (mine). The crowd's
bodies are sent where they stand as a person's device sends its own body,
at most every fifteenth of a second and a little later at a throw, four or
five of the server's frames apart (mine, as the rules asked), never every
frame: in the run below, while moving, a crowd body's sendings came 0.058
to 0.076 s apart on average, the hider's 0.074. How each is marked
standing still was not as a device marks it in this first build (still
whenever it stood where it was last sent, however it had turned since);
since the entry above it is.

**One thing still tells**, found in the online run below and not in this
game's files: a person's device standing still keeps sending where it
stands, the `still` flag flipping each frame (`net::tell_where_i_am` keeps,
as the place it compares the next frame against, the flag it worked out
itself). So a person standing is sent three or four times a second and one
of the crowd standing not at all: over one seek of 15 s in the run, the
hider's body was sent 88 times, 36 of them moving; each of the 28 crowd
bodies 23 to 64 times, all but 3 to 9 of them moving. Nothing a device
draws shows it; a device made to read the network could. The fix is one
line in `src/net.rs`, the tables' file, asked of whoever has it (the
entry above): keep the place compared against as not still,
`sent.was = Some(Spot { still: false, ..spot });`.

Also left as it is: **a person's body is the same entity from one round to
the next**, and the same as while everyone waited at the table, when it
went by its account; and as the hide begins **every device is sent the
hider's body going at once from the pond to where they hide** (found by
the check in the entry above). A device made to read the network could
follow the hider's body from there. Keeping every body from the seekers'
devices from the hide to the seek, and showing them again as bodies new
to them, is the rooms' or the tables' to do (they own the people's
bodies), and is asked of them in the entry above. Nothing the game draws
shows either.

### Calls that were mine

- The window stands to the right of the screen, clear of the chat down the
  left and of the chat's keyboard and the speech bubble (on a phone on
  its side; on a narrower screen the chat's lines draw over it). It is no
  dialog, so that the chat may be used while it is up (the design said
  both "a dialog: you stand still" and "the chat may be used"), and you
  are held still instead. Its box is the keyboard's only while the name is
  being typed, after a press on it. Its cross is Ready with what is
  chosen, and puts the window away for the round.
- An arrow that takes the look over to the other side, a boy's to a
  girl's, throws a name of that side too if the name was one of the other
  side's town names: a girl by a boy's name, which none of the crowd is,
  would give a hider away.
- The sword is in the seekers' hands from the hide, as they stand blind at
  the pond, so that the hider knows who they are from the start.
- The seekers' bodies are on one team on every device (never drawn), so
  that a seeker's swing passes another seeker by and lands on whoever is
  behind them.
- The glow round a seeker is a soft disc of light 1.5 m out on the ground,
  in the game's violet, added to what is under it; the column over the
  hider three crossed sheets of warm light, added too, 24 m high and 2.4
  wide, strongest at the foot, nothing at the edges or the top.
- The stun blinds by a blindfold of the game's own, drawn as Bridge Wars'
  is, whose own would say "A trap has blinded you!" as it comes: Blend In
  takes none of Bridge Wars' fight state.
- "Wrong!" takes away "Seek!" if it is still up, and each headline of the
  game takes the place of the one before, so that two are never one over
  the other.
- The speech bubble and the chat's keyboard are put away while the chat
  is off, and come back with it.
- What everyone is told and the chat says goes by the names people sit at
  the table under (their accounts'), the names they wear being the game's.
- A device back in the game after its connection went is told again what
  it wears and whether it is stunned.
- For trying the game out, a tap on the plate (only where the server
  allows skipping, `RT_ALLOW_SKIP`) ends the choosing as if everyone were
  ready, or the round shown. (In this first build it ended the hide and
  the seek too, the seek as run out with the hider's point, whoever
  tapped: no longer, since the entry above.)
- Words and colours are all in `assets/ui.ron` (`words.blend_in`,
  `colours.blend_in`): the ring `#6666668c`, the glow `#b98cff80`, the
  column `#fff1c2bf`, the black `#000000`, the tick `#34d634`.

### A change outside the game's files

Two of the tables' own tests (`server/src/tables.rs`) played a table of
Blend In as it was before this, a stand-in that did nothing: one expected
a device back to be told nothing by the game, the other a game of two to
go on once one of them had left. Each now puts in the tables' tests' own
stand-in (`OverAsTheyGo`), as four of their tests already did, one line
each; nothing else of the tables' was changed.

### What was checked, and where

- **Tests**: the rules' (`cargo test -p roundtown_net`, 32 of Blend In's,
  two new: the seekers' bodies named only from the hide to the end of the
  seek and never the hider's; hurrying each part of a round); the server's
  (four in `server/src/blend_in.rs`: the crowd up as the table opens, sent
  as people are and as standing once stopped, never every frame; a table
  of two played whole through its rules, with the hide's dressing, a
  choice answered and one refused, a wrong strike stunning and staggering
  and said to everyone, a find, a seek run out, 100 coins to the winner,
  and the crowd gone with the table; a hider who leaves giving up their
  round and a device back told what it wears; the plate's tap hurrying the
  round); the game's (ten in `src/blend_in*`: the moments of a round, the
  end's words, who runs and when, the sword clear of the other buttons,
  the window clear of the chat and its button, the six looks and their
  pictures, the dice's names, the dice's and the tick's pictures, the
  ring's line, the light's edges). All three suites pass whole.
- **Two hidden copies against a server on this PC** (`RT_BACKGROUND=1`,
  `RT_ALLOW_SKIP=1`, scripted by a throwaway taken out again), three
  times by day: a whole game of two with real taps on the arrows, the
  dice, Ready and the sword; the crowd up while waiting; choosing; the hide
  from the seeker's black screen and from the hider's; the seek with the
  sword, the range and the glow; a wrong strike (the blind circle, the one
  struck staggering, the chat's line); a find and the column of light; a
  seeker run out at the ring and kept inside it; a seek run out; "Bravo
  wins!"; the results, 100 to the winner and nothing to the other, 1050
  and 950 coins; back in the town each device with one body, the other
  person's, in their own name and look; a connection made again welcomed
  as the account (the server without a database makes up the look and the
  coins of every connection, and the device puts its own look back).
  What every device was sent was counted from each: every body in the game
  `ai` false, every id in the one range, the timing above.
- **Three hidden copies at night**: a game of three to its end, every
  round run out, "A draw!", 50 each.
- **Offline, hidden, by day and by night**: the card blocked out ("Online
  only"), a seat asked for anyway turned down, the town as it was.
- The pictures are kept in `RoundTown-wt/scratch-blendin/shots/`.
- **Not**: a phone or an iPhone (nothing of Blend In has been seen on
  one); the VM; a database (the run's server kept the coins in memory);
  play by hand; four people; a device put away and back in the middle of a
  round (in the tests only).
