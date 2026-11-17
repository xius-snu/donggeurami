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

1. Since 2026-11-18 Blend In is played online: two to four people, each hiding once among twenty-eight of the town's people while the others look for them

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
  the town). Each is sent to the room exactly as a person is (`Member`
  with `ai` false, an id from the one range every body's is thrown from, a
  name of its side's, a look of it, its `Spot`), so that each device draws
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
     can say anything while the others cannot).
  4. **The round over**, 6 s: a tall soft column of light over where the
     hider stands (the design), and "Found by Dubu in 1:12!" or "Mochi
     stayed hidden!", the same in the chat. A point to the finder, or to
     the hider if the time ran out (Hajun's).
- **A hider who leaves**, or whose device is gone 20 s, gives up their
  round with no point ("Mochi's round is given up"), and someone who
  leaves before their turn to hide loses that round; with fewer than two
  people left the game ends, paid by points so far (the design).
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
five of the server's frames apart, and as standing still once they stop
(mine, as the rules asked), never every frame: in the run below, while
moving, a crowd body's sendings came 0.058 to 0.076 s apart on average,
the hider's 0.074.

**One thing still tells**, found in the online run below and not in this
game's files: a person's device standing still keeps sending where it
stands, the `still` flag flipping each frame (`net::tell_where_i_am` keeps,
as the place it compares the next frame against, the flag it worked out
itself). So a person standing is sent three or four times a second and one
of the crowd standing not at all: over one seek of 15 s in the run, the
hider's body was sent 88 times, 36 of them moving; each of the 28 crowd
bodies 23 to 64 times, all but 3 to 9 of them moving. Nothing a device
draws shows it; a device made to read the network could. The fix is one
line in `src/net.rs`, the tables' file, asked of whoever has it: keep the
place compared against as not still, `sent.was = Some(Spot { still: false,
..spot });`.

Also left as it is: **a person's body is the same entity from one round to
the next**, and the same as while everyone waited at the table, when it
went by its account. A device made to read the network could follow the
hider's body from there. A body afresh for each person at each hide is the
tables' to do (they own the people's bodies), and is asked of them. Nothing
the game draws shows it.

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
  allows skipping, `RT_ALLOW_SKIP`) ends the part of the round under way:
  the choosing as if everyone were ready, the hide, the seek as run out
  (the hider's point), the round shown.
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
