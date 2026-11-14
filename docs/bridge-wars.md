# Bridge Wars, and the game selector

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Contents

The entries of the log below, in its order (the newest dated work first,
then the game as it was first made). Search for an entry by its first words.

1. Since the evening of 2026-11-14 there is a Leap Potion and a faster builder to buy, a drink shows, a potion that is on leaves sparks, the bridge is...
2. Since the small hours of 2026-11-14 the dynamite's arch rises and falls and leans to neither side, with an X where it will land; its fuse is seen t...
3. Since the night of 2026-11-13 there is dynamite, which cuts bridges; the computer's players no longer fall for three easy tricks, and are not all a...
4. Later on 2026-11-13: the compass's arrow points out, the anchors are their teams' colours, and the computer's players fill their buttons, each as i...
5. Since 2026-11-13 there are five buttons for things to use, and you hold several kinds at once
6. Since late on the night of 2026-11-12 each of the computer's players is someone, a knock no longer stuns them, and no two fights with them go the s...
7. Since the night of 2026-11-12 the computer's players play the game
8. Since the evening of 2026-11-12 every game online is 2v2v2v2, people teamed up with each other first and the computer in every seat left
9. Later on 2026-11-12: dying loses everything you hold, what is aimed is seen over the void, and a sword is held on its edge and cut downward
10. Since 2026-11-12 Bridge Wars has two shops, and everything in Hajun's file is in the game
11. Since the night of 2026-11-11 Bridge Wars has generators, a shop and a timetable
12. Since the night of 2026-11-08 Bridge Wars is the server's online, eight to a game in four teams of two, with respawn anchors and bridges you build
13. The gamepad down the right edge opens the game selector
14. Bridge Wars
15. The fight
16. Online, a game of Bridge Wars is a room of your own
17. The camera is fixed straight down over you only in the arena
18. In the arena the buttons are Hajun's `pvpmapbuttonui.jpg`
19. Falling off the map

## Games: the gamepad opens a selector, and Bridge Wars is the PvP map

Since the night of 2026-11-07 (Hajun: "i want the default map to open when
u open the app to be roundtownmap.glb, and when u press the game icon ui
button, it will open a popup game selector menu with 3 games (the first one
is called 'Bridge Wars' the second is called 'Home Builders' and third is a
??? coming soon. you select one and you get a quick black loading screen
and then when the new map is loaded, the player is teleported to either
the pvp map's (waitingroom box above the map until 4 ppl is queued, which
at that point it will count down from 5 and then teleport each player to
each of the 4 island), or the build battle island for queueing - same as
before ... if there is no player joining for 10sec, fill it with bots ...
just make it so the pvpmap game (small) timer of 10 minute (tap to skip)
and after it, the players go back to the main roundtownmap. pvpmap doesnt
need buses and doesnt need npcs. also when in the roundtownmap or the
build battle, it will be freecam, but in the pvpmap game it will switch to
top down cam and the button ui should change to pvpmapbuttonui.jpg
layout"), the town is `roundtownmap.glb` again, with the sea round it and
everyone on the path round its pond, the camera behind you as it was, and
the PvP map, `pvptownmap.glb`, is a game of its own, Bridge Wars, on an
island of its own, `Venue::Arena` (`roundtown_net::Venue`, `island::model`,
loaded as the app opens with the rest; `PROTOCOL_VERSION` 9, the server
deployed with it, the binary before kept on the VM as
`roundtown-server.prev-2026-11-07d`). For the day of 2026-11-07 the PvP map
was the town itself; everything below about the map, its islands, the
void, the punch's knockback and its range held for the town then, and
holds for the arena now.

- **Since the evening of 2026-11-14 there is a Leap Potion and a faster
  builder to buy, a drink shows, a potion that is on leaves sparks, the
  bridge is a quarter wider, a Bridge Builder is seen by everyone, and what
  happens is said in the chat** (Hajun, in one message: "First implement
  the new items in the shop listed in the bridgewarsgameevent.md (leap
  potion and bridge speed upgrade) and the small edits i made like pressing
  the potion button again shouldnt cancel it, only other part of screen
  like joystick or jump or world. Also, for potions: when you drink a
  potion, it's not very noticeable that you are drinking. There should be
  an indicator, like a shrinking circle around you when you use the potion
  ... and its also not noticeable when ur effect runs out - so maybe add a
  "Invisible (##s)" text above the player while you are invisible, and for
  speed potion maybe add a blue particle effect trail and for jump potion
  maybe green particle effect trail. The bridge could also be a little
  wider, maybe 1.25 times its current width"; and, while it was being
  built, "make your bridgebot visible to other players too."). The chat,
  asked for in the same message, has a section of its own ("The chat",
  below), and so has Korean and Vietnamese, asked for as it was being
  finished ("The font is Inter"). **`PROTOCOL_VERSION` 16**: a kit, a
  team's upgrades and what is asked and told have all changed, so the
  server and the app went out together: **the server deployed at 09:35 UTC
  on 2026-11-14** (18:35 here; nobody had been on it for two hours; the
  binary before kept on the VM as `roundtown-server.prev-2026-11-14b`),
  **and APK 29 is the one that plays online**: 28 and before are told to
  update.
  - **The Leap Potion** (`Utility::Leap`, the seventh thing for a button;
    `bridge_wars::LEAP`, `LEAP_SECS`, `leap_speed`; Hajun's file: "Leap
    Potion: 1 amethyst (after drinking time 1.6s, you get 5x jump height
    jump boost for 45 sec, similar ui as speed potions)"): 1 amethyst at
    the item shop, drunk as the others are, and for 45 s a jump goes five
    times as high, off the ground at the square root of five times a
    jump's speed (`Intent::leap`): 8.2 m where a jump is 1.64 (the top
    frame of each in the desktop build: 8.05 and 1.57 m over the ground).
    Its seconds stand over its button as the speed potion's do
    (`Kit::leap_until`, `Kit::lasts_until`). The camera, fixed over you,
    goes up with you, so the island shrinks under you as you rise. At a
    run a leap carries about 20 m across the ground, and 28 on a speed
    potion too: not from an island to the middle, 44 m, but over any hole
    and onto a bridge half built. The server lets it pass as it lets the
    fountain's throw (`a_jump_on_a_leap_potion_is_a_rise_a_body_can_make`
    in `server/src/players.rs`).
  - **The builder's upgrade** (`Ware::BuildSpeed`,
    `Upgrades::build_speed`, `bridge_wars::BUILD_PACE`, `build_pace`,
    `builder_speed`; Hajun's file: half a run by default, "Advanced
    Builder" 75%, "Master Builder" 100%, "Grand Builder" 125%, for 4, 8
    and 16, "also applies to bridge bot"): at the upgrade shop, your whole
    team's, three steps: how fast you walk with a plank in your arms, as a
    share of a run, 5, 7.5, 10 and 12.5 m/s (measured so in the desktop
    build), and a Bridge Builder's pace with it, as it is sent off. The
    file prices it in "diamond", which the game has none of: sapphire is
    what the upgrade shop takes, so sapphire it is.
  - **A drink is put down by a press anywhere else, and not by its own
    button** (`utility::pressed_elsewhere`): the stick, the jump button,
    the world, any other button; the same potion's button pressed again
    does nothing; another potion's button begins that one in its place.
    Space and F on desktop put it down as those buttons do. A hit still
    does, and dying.
  - **A drink shows** (`utility::show_ring`, `DrinkRing`): a ring on the
    ground round you in the potion's colour, 2.3 m out as you begin and
    closing to 0.55 m as the 1.6 s run out, drawn as the punch's range is
    (`range::look`), with the time left written over you as before. Yours
    alone.
  - **"Invisible (27s)" is written over you while you are unseen**
    (`utility::show_unseen`, `words.arena.invisible`,
    `colours.arena.unseen_text`), counting down, yours alone.
  - **A potion that is on leaves sparks** (`src/trail.rs`): blue ones
    behind whoever runs fast on a speed potion, green ones behind whoever
    is on a leap potion, both from someone on both; twenty-two a second on
    the move and a few a second standing, each 0.22 to 0.42 m across,
    left round the legs (or under the feet, in the air), rising and
    shrinking away in half a second. When they stop, the potion has run
    out. Everyone's are drawn (`arena::Wears::fast`, `leaping`, from the
    kits every device has), never those of an enemy who is unseen, which
    would say where they are. Unlit, in `colours.arena.trail_speed` and
    `trail_leap`. For their first hour they were 0.16 to 0.3 m and sixteen
    a second: a thin line of dots from the camera over you. **The speed
    potion's flask is blue** now, on its button and its card
    (`colours.arena.potion_speed`), to go with its sparks, and the leap
    potion's green.
  - **A plank is a quarter wider, and as much longer**
    (`nav::PLANK_HALF_WIDTH`, 1.70 m, so 3.41 m across where it was 2.73;
    `PLANK_SPACING` with it, 1.70 m where it was 1.36): the bridge is the
    shape it was, a quarter bigger, so that everything in `bridge` that
    sets one length of a plank against another, how far a mitre or a slant
    cuts into it, holds as it did; a bridge turns round in about 2.8 m
    where it took 2.3. Metres are metres still: 25 m is fewer planks. A
    span to the middle is 27 planks where it was 33. A stick of dynamite
    takes three or four planks, 5.1 to 6.8 m, where it took four or five
    of the narrower; a bot jumps a hole of three and mends one of four
    (`bots`' tests, retuned). `arena.nav` is the map without bridges, and
    is not baked again. Found on the way: the plank left at an L corner
    ran 5 cm past the fork's far edge, 7 cm along the fork's own way,
    which a step of the feeling for it landed on with the narrower planks
    and stepped over with these, leaving the fork's far corner hanging:
    `CORNER_ROOM` is 0.08. A hundred and twenty whole games of bots after
    it: over at 12:36 on average, none drawn, twelve deaths by punches and
    ten and a half by falls before 10:00, much as before.
  - **A Bridge Builder is seen by everyone** (`Ask::Builder`, `BuilderOf`,
    `server/src/arena.rs`; `bridge::builder_bot`, `dress_remote`): the
    device that sent it tells the server where it walks, fifteen times a
    second, and the server shows everyone else in the game a body there in
    its owner's look, carrying a plank: for show, in nobody's way, no
    punch lands on it. Gone when its bridge is done, or two seconds after
    its device falls silent (`BUILDER_SILENT`); four to a player at the
    most (`MOST_BUILDERS`). A bot's builder is shown the same way, by the
    server online (`Squad::builder_at`) and by the device in a game played
    offline (`arena::show_bot_builders`), where until now a bot's bridge
    grew by itself.
  - **The shops** (`market`): the item shop's rows are the bridge, the
    Bridge Builder and the three swords; the three shields and the
    tracker; and the six things for a button, fireball, dynamite, speed,
    leap, invisibility and healing. The upgrade shop is one row of six
    narrower cards, the builder's last, with a pip to each of its three
    steps and "75% speed" and the like for its line
    (`words.market.build_speed`, `build_speed_stat`; `leap`, `leap_stat`).
    The tracker's line reads "Nearest foe", where "Nearest enemy" ran onto
    a second line and pushed its price off the card's foot. **The time
    over the second button stands a quarter of the button to the right**
    (`hud::UTILITY_TIMER_LEAN`): straight over it, it ran into the count
    in the corner of the button beside it, a "1" and "43s" reading as
    "143s", which two potions at once make common.
  - **A fireball or a stick of dynamite that hurts someone shows whoever
    threw it**, if they were unseen, as a punch that lands does (Hajun's
    file: "being hit or attacking another person or anchor reveals u").
  - **The computer's players** lay their bridges at their team's pace, and
    buy neither the leap potion nor the builder's upgrade.
  - **Checked on 2026-11-14, scripted, in the desktop build** (a
    throwaway `src/arena/scratch_run.rs` again, removed; kept in the
    session's scratchpad). Offline: both windows; all four potions bought
    onto four buttons; the ring, blue and green, at three moments of a
    drink; a drink not put down by its own button and put down by a tap
    on the world, by a thumb on the stick and by a jump; 14 m/s on the
    speed potion with its sparks behind, a leap to 8 m with the island
    small under it, both kinds of sparks from one body; "Invisible (30s)"
    counting down, a bot leaving someone unseen alone; "+50" from the
    healing potion; the bridge 3.4 m wide at 5, 7.5, 10 and 12.5 m/s as
    each step was bought, and the card reading "Max" after the third; a
    bot's builder walking the end of its growing bridge with a plank.
    Against a server on this PC, two copies seated as teammates: one's
    Bridge Builder seen from the other's screen at the head of its
    bridge, one's sparks seen from the other's, the purchases the server's.
    Unit-tested: the rules (63 in `shared`), the server's game and its
    check on a leap (49), the game (292). **Not by hand, and not on a
    phone**: no phone was attached, so APK 29 is built, its 85 asset files
    checked byte for byte, its library the stripped one with its six entry
    points, its number read back, and not installed (APK 28 is kept beside
    it as `donggeurami_town-release.prev-2026-11-14c.apk`). The Adreno has
    not drawn the ring or the sparks (blended and unlit, as the punch's
    range is). On the VM after the deploy this PC's own account was let
    in ("online as xd", the journal clean); no game was played there.
  - **What is mine, the file and the message not saying**: sapphire for
    "diamond"; that the upgrade is the whole team's, as everything at that
    shop is; that a leap is five times the height by the speed it leaves
    the ground at, with nothing changed in the air; the ring's size; the
    sparks' size, number and life, that everyone's are seen and an unseen
    enemy's not; the blue flask; that the plank is longer as well as
    wider; four builders to a player and the two seconds; that only a
    blast that hurts someone shows its thrower; the shop's rows; "Nearest
    foe"; the time's place over the second button; and that the bots buy
    neither new thing.
- **Since the small hours of 2026-11-14 the dynamite's arch rises and falls
  and leans to neither side, with an X where it will land; its fuse is seen
  to be brown; the three dots are gone, and a button for a thing to use is
  shown only with something on it; and whoever keeps coming at a team's
  base is stood against, sooner or later, by who each of them is** (Hajun,
  having played APK 26, in three messages late on the night of 2026-11-13:
  "i want the dynamites hit indicator's curved part to be curved only in
  the Y axis (up and down) not the sideways axis. its a arch projectile not
  a boomarang. also i want there to be some indicator of where the dynamite
  will land, maybe a x in the middle of the circle, and the arched hit
  indicator rectangular part should be connected to the middle of the
  circle. also read BridgewarsAIbottleneck. i just tested the new version
  and found another ai interaction that could solved. fix and
  rebuild+repush"; "also i think the dynamite's string part should be
  brown, like a string."; and "also i dont think the UI ... button is
  necessary. just remove it and keep the default as the only option - only
  showing the button if the slot is filled and not showing empty
  buttons"). **`PROTOCOL_VERSION` 15 still**: nothing that is sent has
  changed (a stick is sent as where it was thrown from and to, as before,
  and how it is drawn between the two is each device's), so APK 26 plays
  on against the server. The bots' part is the server's online, **deployed
  at 15:45 UTC on 2026-11-13** (00:45 on the 14th here; the binary before
  kept on the VM as `roundtown-server.prev-2026-11-14`); the rest is the
  app's, **APK 27**, which has the friends' panel's changes of the same
  night as well ("Friends, groups and messages", below).
  - **The arch** (`bridge_wars::lob_track`, `Lob::track`, `Lob::rise`,
    `DYNAMITE_LEAN`, `UP_THE_MAP`). A stick is thrown straight at where it
    comes down, and rises and falls on the way (`Lob::apex`: 0.8 m over
    the straight line of the nearest throw, 2 m over that of the
    furthest). The camera is straight over you, and from there a throw
    that only rose and fell would be seen going along a straight line,
    however high; so it is drawn as an arch is on a map, **up the screen
    as it rises and back down it as it falls**, by as much as it is high
    (`DYNAMITE_LEAN`, 1; north, the game's -Z, whichever way it is
    thrown). Thrown to the left or the right, the whole arch is seen;
    thrown up the screen or down it, it goes along a straight line. For
    its first day it swung out to the thrower's right by 0.18 of the
    throw's length (`DYNAMITE_BULGE`, gone), which is what Hajun saw as a
    boomerang. Where it comes down, when, and what it blows are as they
    were, and the server's online; the leaning is only where each device
    draws it on the way.
  - **The band** (`utility::lob_arch`, `range::arch`): the same arch, 0.6
    m wide as before, built in the air: each of its 24 stretches as high
    as the stick will be there, and as far up the screen, from the hand to
    **the middle of the ring** (it stopped at the ring's edge, and in the
    nearest of throws was not drawn at all), with its fill, its outline
    and its dark rim (`range::arch` takes a line through the air;
    `range::ribbon`, along the ground, is gone). Built again as the throw
    changes by 4 cm or turns by half a degree, which way the top of the
    screen is from it changing as it turns; its sheets are never left out
    for where they were a moment ago (`NoFrustumCulling`). **And an X in
    the middle of the ring** (`range::cross`, `LANDING_CROSS`): two
    strokes a metre long and 17 cm thick, in the band's white with its
    dark rim, aslant on the screen whichever way you aim (the ring is
    turned back by as much as the band is turned with the aim).
  - **The fuse** (`dynamite::FUSE_RADIUS`, `FUSE_LENGTH`): 0.42 m long and
    0.14 across where it was 0.26 and 0.09. It was brown all along
    (`colours.arena.dynamite_fuse`), and from the camera over you a dark
    hair at the sticks' end, of no colour at all. **Its picture is in two
    colours** (`hud::painted`, `Picture::in_its_own_colours`,
    `hud::repaint`): the sticks red and the fuse brown, the file's own
    two, on the button and on the shop's card, where the whole picture
    was tinted the sticks' red, with a dot for a spark at the fuse's end,
    which is gone. It is the one picture not tinted where it is shown
    (`market::look_of` gives it white), so it is painted again as the
    file changes.
  - **The buttons** (`hud::DOTS`, `utility::SECOND_ALWAYS`, both `false`,
    the code behind each kept): the three dots are not put up, and
    nothing shows an empty button. **A button is shown only with
    something on it**, the second one too, which until then stood empty
    up and to the left of the sword from the start: with nothing to use,
    the sword has the jump and the bridge buttons round it and no more.
    That the second one goes with the rest is my reading of "not showing
    empty buttons"; `SECOND_ALWAYS` is the one word to have it back.
  - **What Hajun found** (`BridgewarsAIbottlenecks.txt`, its one item
    now: "When i go to invade another base, i go near their island, and
    attack the spawner, then the 2 enemies of that island come toward me
    to defend, and i retreat back a slight bit to the bridge, and then
    they both go back to what they were doing (going to their generator)
    so i go back to attack the spawner a little bit, then i retreat back,
    and i keep repeating this until their spawner destroyed"; it asks for
    a patience that differs from one to the next, standing ground after
    the first, second or third "agitation", "until he rly rly leaves",
    and says "if some thing becomes a definite if-then, then its not a
    real human, but a program"). A bot had no memory of anyone: it went
    home for whoever stood on its island, and back to what it was doing
    the moment they stepped off it, as often as they liked.
  - **Standing their ground** (`shared/src/bots.rs`: `Vigil`, `Stand`,
    `Squad::keep_vigil`, `guard_spot`, `Goal::Guard`, `GUARD_SCORE`
    0.79, over anything a bot does about its island and under seeing
    someone off it and backing off hurt). Whoever of another team is on
    the land of a bot's island, or within 5 m of its anchor as that loses
    health, it has in mind from then on, and counts each time they come
    again after having stepped back (`Vigil::comings`). **Each bot has
    its patience, 1 to 3 comings** (`Persona::patience`, `patience_of`:
    its kind's number and a bell's throw, rounded: ISFJ 1.0, INFJ 1.1,
    ISTJ 1.2, ESTJ 1.3, ESTP 1.4, ENTJ and ESFJ 1.5, ENFJ 1.6, ISTP 1.7,
    INTJ 1.8, ENTP 2.2, ESFP 2.4, INTP 2.5, ISFP 2.6, ENFP 2.7, INFP
    2.8; thrown again each time it has forgotten someone). When they step
    back and it has not had enough of them, it looks after them for 0.8
    to 4 s (`VIGIL_LOOK`) and goes back to what it was doing; once they
    have come as many times as it has the patience for, **it stands its
    ground**: on its own land between them and its anchor, on the line
    from them to it, 3 to 5 m from them and 0.9 to 2.2 m in from the
    shore (`VIGIL_OFF`, `VIGIL_INSIDE`, the more for one that minds being
    thrown off), or, with too little land for that, right in front of the
    anchor; the two of a team 0.9 m to either side of the line; turned to
    them, and moving as they move. **At them if they come near**: within
    5 to 12 m of where it stands, times how bold it is (`VIGIL_WATCH`:
    measured from where it stands, not from where it is, so that it is
    not led out a step at a time), it goes out onto the bridge after
    them, and back to its place when they are further. **There until
    they are really gone**: 24 to 38 m from the middle of its island
    (`VIGIL_GONE`; the bridge's far end is 44 m off) for 2.5 to 8 s
    (`VIGIL_CALM`), when it goes back to what it was doing. **And not for
    ever**: after 12 to 36 s of it (`VIGIL_STAND`) with them still about,
    it either goes out at them, as far as they are and 6 m more, for 6 to
    14 s (the likelier the bolder and the better at a fight: 0.25 and 0.3
    of its `pvp` and 0.35 for each 1 of `bold` over 1, between 0.1 and
    0.85), or lets them be, and has not forgotten them either way: the
    next time they come it stands at once. Left alone 50 to 110 s
    (`VIGIL_FORGET`), it has forgotten them. Every number is thrown from
    its spread each time it is needed, so no two stands are alike.
  - **And nothing else it does about a fight begins or ends on the same
    line twice** (`KEEPS_ON`, `KEEN`, `KEEN_LASTS`, `STING`, `NERVE`):
    how near it lets people come before it goes for them is 0.72 to 1.35
    times its usual, thrown again every 4 to 11 s; someone it is on
    already it keeps after 1.3 times as far; a blow smarts for 2.2 to
    6.5 s, in which it goes for whoever is within 9 m (3 s, always,
    until then); and where it backs off hurt is up to 0.08 of its health
    either side of its usual, thrown again after each fright.
  - **Two bots after each other across a hole fell into it, and do not
    now** (`Squad::act`, `Squad::fight`, `Bot::awaits`, `Nav::leap`,
    `Nav::nearest_walkable_within`). Found by the server's whole games:
    with the dice falling otherwise for the throws above, in one of its
    eight a bot walked off an edge. Two bots of different teams, either
    side of a hole a stick had left in a bridge, each jumped it at the
    other at the same moment, passing in the air, three times running,
    and the third time one fell. Four things, each mended: **someone on
    their way over a gap to it is waited for** (in the air over the
    void, or running at it with no straight way between them: it jumps
    none itself until they are down or have stopped); **a jump comes
    down a step in from the edge** (0.9 m, wherever whoever it is after
    stands: with them on the very edge, the place to come down was the
    edge itself); **in the air it keeps on for where it means to come
    down and stops over that very spot** (the jump was over as soon as it
    was within 0.4 m of there, in the air or not); and **what someone in
    the air steers back for has ground under it** (the squares walked
    along a plank reach round its end, out over the hole: it steered for
    the middle of the one it was in, to and fro about it, down into the
    void).
  - **Whole games, again** (a hundred and twenty of bots alone, as
    deployed): over at 12:32 on average, none drawn; won by island 33,
    26, 26 and 35; before 10:00 twelve and a half deaths by punches and
    eleven by falls a game, and 0.5 anchors broken by bots: much as it
    was (12:29, twelve and a half and ten and a half, 0.4).
  - **Checked on 2026-11-14.** Unit-tested: the arch (to neither side of
    its throw, up the map by its height, straight thrown up or down it),
    the band and the X, the two-coloured picture, that no empty button
    is shown; someone who keeps coming stood against at the coming its
    kind is like (of 24 each: ISFJ at the first 21 times, INTJ at the
    second 21, ENFP at the second 8 and at the third 16), a bot keeping
    between them and its anchor until they are gone, and what it does
    when it has stood long enough (of thirty stand-offs the lone wolf
    went out at them 19 times and let them be 11, the dreamer 4 and 26);
    the jump past someone on the edge of a hole, 24 of 24 down on the
    planks, and the wait for someone coming over, which fails without it.
    **Scripted in the desktop build**, offline (a throwaway
    `src/arena/scratch_run.rs`, removed): the band aimed nine ways, to
    the right at three lengths, to the left, up the screen, down it and
    across the corners, by day and by night, the X an X in each and the
    band meeting the ring's middle; a near throw and a far one watched
    along their arch; the fuse close by and on the button; the buttons
    with nothing held and with one kind; and a team baited as Hajun did
    it: both at the head of their bridge between the baiter and their
    anchor, one out at the baiter standing 4 m off, both back to their
    places, and both back to what they were doing 7 to 16 s after the
    baiter had gone 31 m off. **On the VM after the deploy**, nobody
    having been on it for an hour, this PC's own account was let in and
    seated with seven of the computer's, three of their bridges across
    by half a minute; the script then stood on another team's island,
    whose two bots had it dead in six seconds; the journal clean. APK 27
    is built, its 85 asset files checked byte for byte, its
    library the stripped one with its entry points, and its number read
    back, and **not installed, no phone being attached** (APK 26 is kept
    beside it as `donggeurami_town-release.prev-2026-11-14.apk`). **Not
    by hand, and not on a phone**: the arch and its X under a thumb, and
    a real person baiting the server's bots, are unseen.
  - **What is mine**: that the arch is drawn up the screen, and by as
    much as it is high; that the band is built in the air; the X's size;
    the longer, thicker fuse; hiding the empty second button; every
    kind's patience and every number of a stand; that a bot which has
    stood long enough goes out at them or lets them be by chance; that it
    forgets; the spreads of `KEEN`, `STING` and `NERVE`; and that a bot
    waits for someone already on their way over a gap.

- **Since the night of 2026-11-13 there is dynamite, which cuts bridges;
  the computer's players no longer fall for three easy tricks, and are not
  all as good as each other at a fight; and one whose teammate is a person
  is out and about** (Hajun, in three messages: "read the
  bridgewarsgameevent.md, i made some update like adding the dynamite
  item. add the dynamite item and functionality to the minigame, test it
  thoroughly. dont need to build apk or push to git when ur done"; "also
  read BridgewarsAIbottlenecks.txt. those are some of the things i noticed
  while playtesting against the current ai bots. try to fix them as well";
  and "the ai teammate right now when ur in a team with a bot, the bot
  stays on the iron/gold generator for toooo long. reduce the time it
  spends on the generator and make it go out more or use shop longer or
  bridging or raiding an island more and just come back to pick up
  resource and leave again quickly not stay"). **Hajun's rules file is
  `bridgewarsgameevent.md` since that day** (`bridgewarsgameevent.txt`
  wherever this file says so of an earlier day); besides the dynamite it
  has the healing potion giving back half of your health where it gave a
  quarter (`bridge_wars::HEALING`; a green "+50"), and so does the game.
  **`PROTOCOL_VERSION` 15, the number it shares with the friends system of
  the same day** ("Friends, groups and messages", below): a `Kit` with a
  stick of dynamite on a button is nothing an app from before can read,
  so the server and the app go out together, and the two pieces of work
  with each other. **The session that made this neither deployed, built an
  APK nor committed**, as asked: it goes out with the friends system, by
  the session that made that ("Building", above, has the build).
  - **The rules** (`roundtown_net::bridge_wars`; Hajun's file: "Dynamite: 1
    amethyst (similar to default dynamite attack for dynamike in
    brawlstars, hold&drag&letgo mechanic like fireball, but instead of one
    straight rectangular hit range indicator its like an long arched
    curved rectangular, with ability to change the total distance with the
    magnitude of the hold&drag&letgo, and when fired, it will throw an
    arched dynamite (2 minimalistic mini red cylinder shape with smaller
    brown cylinder sticking out of one face which is the string), and
    after landing on the ground/bridge, it will show a circular blast
    radius range around it and it will shine bright red 3 time before it
    explode, and when it explode, it will do 20 damage to player in that
    blast radius and destroy that blast radius region of the bridge - can
    stack up to 3 times max at once)"). `Utility::Dynamite` is the sixth
    thing for the buttons: 1 amethyst at the item shop, three to a button
    (`most_held`), aimed as the fireball is (`Utility::aimed`). A stick
    thrown is a `Lob`: from the hand, 1.2 m up and half a metre in front
    of the thrower (`DYNAMITE_HAND`, `DYNAMITE_AHEAD`), to the ground 2.5
    to 10 m from them (`DYNAMITE_NEAREST`, `DYNAMITE_FURTHEST`; whatever
    is asked for is brought within those), in 0.45 s and 0.04 s more for
    each metre (`Lob::secs`: 0.55 s the nearest, 0.85 s the furthest),
    along an arch (for its first day one that also swung out to the
    thrower's right by 0.18 of its length, `DYNAMITE_BULGE`, so that the
    camera straight over you saw an arch at all; since 2026-11-14 it is
    drawn up the screen as it rises, above). Down, it lies 1.2 s
    (`DYNAMITE_FUSE`), shining three times (`DYNAMITE_FLASHES`,
    `Lob::shine`), and blows: 20 (`DYNAMITE_DAMAGE`), less what a shield
    keeps off, to everyone of another team than the thrower's whose feet
    are within 3 m of it and half a body's width more (`DYNAMITE_RADIUS`,
    `in_dynamite`), zombies among them, with no knock; whoever it hurts is
    seen again if they were unseen, as by any blow. And **every plank
    whose middle is within 3 m of it is gone** (`nav::plank_in_blast`),
    whoever laid it, from under whoever stands on it: mostly four planks,
    5.45 m of bridge, sometimes five. One that comes down with nothing to
    lie on within half a metre (`DYNAMITE_CATCH`, `caught`) falls on into
    the void and never blows. The server's online (`Game::lob`,
    `Game::dynamites` in `server/src/arena.rs`), this device's own in a
    game played offline (`arena::dynamites_offline`).
  - **Aiming it** (`utility::aim`, `Hold::reach`, `Hold::far`): held and
    dragged as the fireball is, and **the further the finger is dragged
    the further the throw**: from 4 shares of the short side from where it
    went down (`AIM_DEAD`, as ever), the nearest of throws, to 22
    (`AIM_FULL`, about a button and a half), the furthest. What is drawn
    is a band 0.6 m wide that bends as the stick will fly (for its first
    day out to your right and back along the ground, `range::ribbon`
    along `lob_line`, ending at the ring's edge; since 2026-11-14 an arch
    in the air to the ring's middle, with an X there, above; `LobBand`),
    and a ring as wide as the blast, round where it will come down
    (`LandingRing`, `range::disc`: mine, as Brawl Stars shows where a
    thrower's shot lands), in the fireball band's own colours with its
    dark rim (`colours.arena.aim_*`). In the nearest of throws the ring
    reaches back past your hand (and for that first day the ring alone
    was drawn). Dragged back to where the finger went down
    it is cancelled, a quick tap that never dragged throws 4.5 m the way
    you face (`DYNAMITE_TAP`), and so does its key. Half a second between
    two, as with the fireball.
  - **What you see** (`src/dynamite.rs`): two red sticks side by side, 0.8
    m long and 0.64 across the pair, with a brown fuse out of one end
    (`colours.arena.dynamite`, `dynamite_fuse`), turning end over end one
    and a half times on their way; lying, a red ring on the ground as far
    as the blast will reach (`blast_outline`, `blast_fill`, with the
    band's dark rim, so that it is seen over the void), and the sticks
    shining three times: five times their own red given off (`SHINE`) and
    a red glow round them out to 1.1 m; then the fireball's own ball of
    fire (`fire::burst`). One that found nothing to lie on tumbles on
    down and is gone 35 m below. A plank blown leaves its bridge with a
    hole: the ends left are capped, what is left of the bridge stays as
    it was built (`Chain::gone`, `holed`; `bridge::Bridges::blast`,
    `break_planks`), the plank you are laying goes too if the blast
    reaches its middle (`blast_tip`), the minimap shows the gap, and
    planks laid across a hole afterwards meet the ends left flush, as a
    bridge meets a shore. The button's picture and the shop's card are
    the two sticks and the fuse (`hud::Picture::Dynamite`).
  - **Online** each stick is an entity the server sends as it is thrown
    (`Dynamite`: who threw it, from where and to where), which every
    device lobs on by itself (`dynamite::follow_server`), and takes away
    as it blows, sending in its place where it blew and which planks went
    (`Blasted`, kept in the game's room with its planks, so that someone
    who comes late has the holes; `dynamite::hear_blasts`); one taken
    away with no blast fell into the void there, and falls on here.
    `Ask::Use(Use::Dynamite { heading, far })` throws one: the server
    takes it off the button, throws it from where it has you standing,
    and refuses numbers that are none.
  - **The shop**: the item shop's third row is six cards now, the
    dynamite's last ("Dynamite", "Cuts bridges", `words.market.dynamite`
    and `dynamite_stat`: the line is mine), and a card in a row of six is
    narrower, 14 shares of the short side where the rest are 15.5
    (`market::NARROW_CARD`, `card_width`), so that the row ends short of
    the second button on a screen of sixteen by nine: six of the other
    width reached under it. **"Slots full" comes up now**: with five other
    kinds on the five buttons, the sixth cannot be bought.
  - **The computer's players and dynamite** (`shared/src/bots.rs`). Each
    kind has its leaning to it as to the other five things
    (`Persona::carries`, six numbers now: the trickster and the tinkerer
    most, 0.4, the captain and the sentinel least, 0.05), and one a fifth
    of whose leanings are to it keeps two, the rest one
    (`Persona::loadout`). **It is thrown at whoever of another team is out on a
    bridge** a throw away, at where they will be when it blows if that is
    still on a bridge, to take the bridge from under them or from in
    front of them: from the land, or from a bridge itself if what would
    go is well clear of where it stands; never where a teammate stands on
    the planks that would go; 6 to 12 s between two (`Order::blast`).
    **A stick thrown at one it gets out from under** (`Squad::mind_sticks`,
    `Stick`, `View::sticks`): once it has woken up to it, 0.75 to 2 s
    after the throw for the worst of them at a fight and 0.35 to 1 s for
    the best, a quarter of a second later for anyone in a fight
    (`STICK_WAKES`, `STICK_WAKES_SOONER`, `STICK_BUSY`), it runs for the
    nearest ground clear of the blast and keeps out until it has blown,
    whoever it was fighting in there; one lying on a bridge ahead it
    waits short of; its own team's is nothing to it on the land. Of
    eighty sticks thrown from close by to come down right where one
    stood: the four best kinds were never caught, the middling ones
    hardly ever in the open and up to 15 times in a fight, the worst
    (INFP) 34 times in the open and 55 in a fight; a test has every
    kind's
    (`a_bot_gets_out_from_under_a_stick_of_dynamite_and_waits_short_of_one_on_its_way`). For its first hours every kind was out in half a second, and
    then walked back in after whoever it was fighting. **A hole in a
    bridge it jumps or mends** (`nav::Nav::blast`, `gaps`, `crossing`,
    `leap`; `Body::leap`; `Squad::reach_of_a_jump`): one of up to 5.8 m,
    the four planks a stick mostly takes, a bot at a run jumps, on its
    way anywhere and after whoever it fights; a longer one, in its team's
    way to the middle, either of the team lays planks across, from
    whichever side it stands on, half a metre inside what stands to half
    a metre inside what stands on the far side (`mend_for`, `MEND_INSET`),
    with the metres it has or buys.
  - **The three easy ways to beat them** (Hajun's
    `BridgewarsAIbottlenecks.txt`: "These are the list of my personally
    observed "easy" optimal ways to defeat AI's"). **Running round in a
    circle** ("the AI always is one step behind so he just keeps
    following in a circle behind me in an endless loop"): a bot goes for
    where whoever it is after will be, not where they are (`Squad::meet`,
    `foresee`, `Track`: their pace and how fast they turn, watched from
    frame to frame; no further ahead than 1.6 s, and no further than there
    is ground under them), so someone going round and round is cut off
    across the round. **Jumping from one bridge to another beside it**
    ("instead of the ai also jumping across, it goes all the way back to
    the island"): with a gap it could jump between it and whoever it
    fights, no walking straight to them and the way round far or none, it
    stands a moment and jumps, the likelier the bolder and the better it
    is (about one time in two for the worst of them, nearly always for
    the best), or thinks better of it, goes round, and thinks of it
    again in a few seconds. **All of them
    equally moderate** ("when i spam click the attack button while
    fighting, i always seem to hit them first, and they seem to get
    knocked back more than they knock me back, so i can always win even
    in 1v2 ... some bots are like pro-er, and some bots are moderate like
    now"): each kind has how good it is at a fight, from 0, as they all
    were, to 1 (`Persona::pvp`, each bot's own a little off its kind's,
    `Temper::pvp`): the lone wolf 1, the daredevil 0.9, the commander 0.8,
    the enforcer 0.7, the trickster 0.65, the mastermind 0.6, the troll
    0.55, the free spirit 0.5, the captain and the sentinel 0.4, the
    watchman 0.35, the drifter 0.25, the tinkerer and the host 0.2, the
    defender 0.1, the dreamer 0. The better it is, the sooner its first
    punch comes once it has someone in reach and the nearer its punches
    come to as fast as they count (down to one every 0.53 s, a body being
    past hurting for half a second); the oftener it steps back out of
    reach between two and comes in again on the run timed to the next,
    since a punch thrown running in knocks further than one thrown
    standing, which is why they were knocked back more than they knocked;
    the sooner it is on its feet after a knock and the harder it steers
    against one; and the more of where you are going it reads.
  - **A bot whose teammate is a person is no keeper of the pad** any
    more. It comes to the pad for what lies there once that is worth the
    walk (as many things as it wants, six at the most) and you are not
    there for it yourself, is on it no longer than 1.4 s, and is not back
    for 9 to 16 s; between visits it shops, wanders its island away from
    the pad, and goes out: from 45 s, with nobody of another team within
    25 m of its island, for the amethyst on your side of the middle and
    the sapphire of the smaller islands beside yours; from 30 s it builds
    the bridge to the middle if nothing joins your island to it, unless
    you are out over the void building one yourself (40 s until then),
    and from 75 s one to a smaller island (two minutes until then); and
    from a little after the raids begin, 2:50 at the soonest, it raids by
    itself, which it never did. In a test of seven minutes beside a
    person who stays in the middle of their island, three times over, it
    is by the pad 14 to 18 times, 6 to 10 s of it in all and never longer
    than 2.8 s at a time, off the island for 249 to 299 s, and has raided
    and fetched every time
    (`a_person_s_bot_teammate_comes_to_the_pad_for_what_lies_there_and_is_off_again`).
  - **Three faults found on the way.** A bot's body that came down to
    within a hair of the ground without quite touching it stood there
    still falling, as far as it knew, for as long as it stayed on ground
    of that height: neither in the air nor its own to move, so it never
    jumped nor drank (`Body::step`). **A mesh with no corner in it is
    not to be put in `Assets<Mesh>`**: Bevy's mesh allocator logs "Use-
    after-free: attempted to copy element data for an unallocated key"
    for each, as the nearest of throws did for the band's three sheets,
    none of it being outside the ring; they are put away as they are
    instead (`LobSheet`). And a bot that had got out of a blast walked
    straight back into it, above.
  - **Whole games, again** (two hundred of bots alone, as it stands):
    over at 12:29 on average, none drawn; won by island 43, 40, 63 and
    54; before 10:00 twelve and a half deaths by punches and ten and a
    half by falls a game, where it was nine and five (the ones that are
    good at a fight punch on the run, which throws whoever they hit
    further), and 0.4 anchors broken by bots; each kind's team winning 7%
    (the sentinel) to 55% (the lone wolf) of its games, where every
    kind's won 20% to 31%: a lone wolf dies 2.0 times a game and a
    captain 4.4. The eight of a game together bought 1.9 sticks of
    dynamite and threw 0.6, which took 2.5 planks, the most in one game
    four sticks and eighteen planks; and 12.9 fireballs, 2.3 speed
    potions, 1.0 invisibility potions, 25.0 healing potions and 1.4
    Bridge Builders, of which they used 11.1, 2.0, 0.6, 8.8 and 1.2.
    `many` prints all of it (`Chronicle::blasts`, `planks_blown`).
  - **Checked on 2026-11-13.** Unit-tested: the lob, its arch, its fuse
    and its three flashes, whom a blast reaches, and what a kit and a
    shop make of a sixth kind; planks taken out of a map and out of a
    drawn bridge, the ends left, the plank being laid, and a hole mended
    flush; the server's stick from throw to blast; the band and the ring;
    a bot out from under a stick kind by kind, short of one on its way,
    over a hole of four planks and mending one of five, throwing at
    whoever comes over a bridge and at nobody else, cutting off someone
    who runs in circles, and jumping after someone on the next bridge;
    and the teammate's visits to the pad. **Scripted in the desktop
    build** (a throwaway `src/arena/scratch_run.rs`, removed), offline,
    by day, at sunset and at night: the card bought three times and
    refused a fourth, "Slots full" with five other kinds; the band at
    three lengths and four ways, over the land and over the void, and
    dragged back; a throw watched through its flight, its ring, its three
    flashes and its blast from the game's camera and from close by; one
    thrown off the island, falling; a bridge of fifteen rungs cut from the
    shore, four planks gone, the gap on the minimap, standing on what was
    left past it, the planks under my own feet blown by my own stick and
    the fall, and walking off the cut end into the hole; sticks thrown at
    me: 20 off, 18.8 through a Basic Shield, nothing from a teammate's,
    and dead of one, with everything held gone; sticks thrown at two of
    the computer's players on their island, one of which got out every
    time and the other was caught twice, 100 to 80 to 60; and their
    bridge to the middle cut, on which, within six seconds, the one
    coming home jumped the hole while its teammate came out from the
    island and laid six rungs across it. **Against a server on this PC,
    with two copies** seated as teammates: the sticks bought from the
    server, the fourth refused; the second copy seeing the first's stick
    fly, lie, flash and blow, and four planks go from the bridge on its
    own screen; one thrown over the void, which the server had fall; the
    near end of the bridge blown from under the second copy, which took
    no harm from its teammate's stick and fell with the planks, the
    server logging the fall; and one of the server's bots taking 20 from
    a stick, the number floating off it. **Not by hand, and not on a
    phone**; no stick thrown by one of the computer's players has been
    seen on a screen, they throw so few, and that is the tests' alone.
    Known and left: thrown the full 10 m straight up or down the screen,
    half of the ring is off it, and a far throw up the screen lands under
    the time; and on a window of sixteen by nine the four more buttons,
    when they are shown, lie over the right end of the shop's bottom row,
    as they have since there were five.
  - **What is mine, the file not saying**: that a blast harms neither its
    thrower's team nor an anchor, and knocks nobody; that a plank goes
    whole, by its middle; the 3 m of the blast (the fireball's), the 2.5
    to 10 m of a throw and the 4.5 m of a tap; how long it flies and the
    1.2 s it lies; that the arch leant to the thrower's right (for its
    first day); the ring
    where it will land; that one over the void falls and never blows;
    the sticks' size and their glow; "Cuts bridges"; the narrower cards;
    everything about the computer's players and it; every kind's number
    for a fight and what it changes; and every number of the teammate.

- **Later on 2026-11-13: the compass's arrow points out, the anchors are
  their teams' colours, and the computer's players fill their buttons,
  each as it is like** (Hajun, having played APK 24, in three messages:
  "the player compass tracker, i think the arrow need to be flipped? the
  arrow icon is pointing inward at the player rather than outward. fix and
  push to git and build apk again"; "also now, there is 4 more item slot,
  so the ai should be adjusted so it can buy up to 5 even. match it with
  the personality so more diverse interaction"; and "also make the color
  of your spawn anchor (right now its purple) but make it so the color is
  your team's color. so red blue green yellow glow"). **The bots' part is
  the server's online, deployed at 16:43 UTC on 2026-11-12** (01:43 on the
  13th here; the binary before kept on the VM as
  `roundtown-server.prev-2026-11-13b`; `PROTOCOL_VERSION` 14 still, what a
  bot is ordered to do being the brain's own and nothing sent, so APK 24
  plays on against it). The arrow, the anchors and the bots of a game
  played offline are the app's: **APK 25**.
  - **The compass's arrow was drawn upside down** (`hud::Picture::Compass`).
    `compass::point` turns the picture from pointing up the screen to
    pointing at whoever it tracks, and the picture had its tip at the
    bottom: `hud::drawn` has y up, as every picture's comment says, and I
    wrote its tip at y -0.92. So round you the arrow sat on the side its
    target was on and pointed back at you. I had looked at it the day
    before in a screenshot at full size and taken it for right; enlarged,
    it plainly points in. **Look at a small thing enlarged.** It is drawn
    tip up now (its card in the shop too), and a test keeps the tip at the
    top (`the_compass_s_needle_points_up_its_picture`). **And its words
    keep clear of it** (`compass::past`, `WORDS_GAP`): their middle was a
    fixed 15.5 shares of the short side out, which up and down the screen
    cleared the arrow and across it did not, a name and a distance being
    eight times as long as they are tall ("Dori 76m" written over the
    arrow, seen in the run that checked the flip); they begin one share
    past the arrow's tip now, whichever way it points. Seen from four
    places: up and to the left at "Ddol 179m" from home, left, right and
    straight up from the middle of the map, each time pointing out at the
    island whoever it named was on, in that team's colour.
  - **Each team's anchor is its team's colour** (`arena::paint_anchors`,
    `pale`; `colours.arena.anchor_red`, `anchor_blue`, `anchor_green` and
    `anchor_yellow` in `assets/ui.ron`, `ArenaColours::anchor`): each
    team's pillar wears a material of its own, the map's `spawn` material
    in its team's colour (`#e0352b`, `#2f6fe8`, `#2fbf4a`, `#f2c21e`, more
    solid than the names' `team_*`, which are tints for words), where all
    four shared the one purple. By night each glows in its colour, `lamps`
    glowing a bulb in whatever colour its material is (`ANCHOR_GLOW`, one
    and a half times it), and lights its island in a pale shade of it:
    three parts of white to two of the colour, its strongest channel 1
    (`pale`; the purple's light, chosen by eye, was much what this makes
    of purple). Painted once the map has been read and the pillars' lights
    are there, and again as the file changes. The bar over an anchor is
    as it was, green for yours and red for the rest. Seen from over each
    island by day and by night: red at the bottom of the map, blue on its
    left, green at its top, yellow on its right, and four tinted pools of
    light.
  - **What each bot carries** (`bots::Persona::carries`, `pockets`,
    `loadout`, `KINDS`; `Bot::loadout`). A kind has its leaning to each of
    the five things there are for a button, a fireball, a speed potion, an
    invisibility potion, a healing potion and a Bridge Builder, and how
    many kinds it carries at the least and the most: the daredevil, the
    trickster, the mastermind and the tinkerer three to five; the troll,
    the commander, the free spirit and the drifter two to four; the
    captain and the enforcer two or three; the host, the watchman, the
    defender and the lone wolf one to three; the dreamer and the sentinel
    one or two. Each life it throws which, in what order it buys them (the
    likeliest mostly first), and how many of each it keeps: three
    fireballs for one that leans hard to them and two for one that leans
    some, two healing potions for one that leans on those, one of anything
    else. `shopping` gives each kind its turn among everything else it
    buys (0.76 of its kind's leaning to its buttons for the first, 0.05
    less for each after), a second or third of a kind well after the first
    of the next, and a keeper with nobody to fetch it amethyst a healing
    potion in place of what amethyst buys. One kind a life until then.
  - **Each is used in its own time** (`Squad::intend`; `Bot::drinking`
    knows which potion is at its mouth): a healing potion whenever it has
    less than 55% of its health and nobody of another team is within 9 m,
    or backing off with nobody within 6; an invisibility potion within
    62 m of the middle of the island it raids; a speed potion setting out
    on a raid or a hunt, for home from over 30 m off when someone is on
    its island, or backing off with whoever is after it 8 to 22 m behind;
    and its fireballs as before, one every 6 s at whoever is a throw away
    with land under them.
  - **A bot sends a Bridge Builder** (`Order::build`, `Sent`, `build_on`):
    with one on a button and a bridge to lay, it walks to where the bridge
    is to begin and sends the builder off from there, and the bridge goes
    up straight, a plank or two ahead of where the builder would be, at
    the pace anyone builds at, while the bot goes about something else: no
    metres of its own, no plank in its arms. Nobody sees the builder, as
    nobody sees a person's online: the bridge grows by itself. Whoever
    runs the game takes the builder off its buttons (`Outfits::spend`;
    `Squad::unsend` if it was not there after all); its owner dying, it
    stops where it is. **And there are bridges worth one**
    (`Squad::spare_span`), which a bot lays only by builder, never with
    metres, and buys a builder for when it carries them and has the
    amethyst: from the middle to **the other of the two smaller islands**
    beside its own, for twice the sapphire; and, once the raids are on,
    **a side door**, from a smaller island its team has reached straight
    onto the island of the other team that island lies beside, about 75 m
    of water, to come at them from where no bridge of theirs lands. A
    kind that likes a raid opens the side door first, the rest the
    sapphire; not beside a person. Without these no bot would ever have
    bought one: its own bridges are built before the first amethyst
    comes, a minute in. Every way a bot finds goes over a side door as
    over any bridge, so a raid comes by whichever way is shorter.
  - **Two faults found on the way.** A bot put the same potion to its
    mouth twice: in the frame a drink was done, the potion was still on
    its button as that frame saw its kit, and it began again, 1.6 s at
    half its pace for nothing, since the bots first drank. And **a
    person's bot teammate could be given the wrong bridge** since the
    kinds came: `lay` asked for its bridge again as if it were the one of
    two bots that goes out, so one whose kind opens to a smaller island
    was sent to build that, with the metres it had bought for the middle,
    and one whose kind opens late was given none for up to two minutes. It
    lays the bridge its last thought settled on now (`Bot::planned`).
  - **Whole games, again** (two hundred, as deployed): over at 13:03 on
    average, none drawn; won by island 50, 53, 58 and 39; before 10:00
    nine deaths by punches and five by falls a game; each kind's team
    winning 20% to 31% of its games. The eight of a game together bought
    16 fireballs and threw 14, drank 2.5 speed potions, 0.7 invisibility
    potions and 16 healing potions of the 29 they bought, and sent two
    Bridge Builders; the most kinds on one of them at once was two in 87
    games, three in 88, four in 23 and five in 2: they can fill five, and
    amethyst, one a minute from each pad at first, is what mostly keeps
    them from it. `many` prints all of it (`Chronicle::carried`, `used`,
    `fullest`).
  - **Checked on 2026-11-13.** Unit-tested: the picture's tip, the words'
    place, each anchor's colour and its light, what each kind carries,
    what a bot buys of several kinds and when a builder, a builder's
    bridge laid rung by rung across a gap while the bot stays ashore, the
    potion drunk being the one the moment calls for, and, on Hajun's map
    itself, the second island and the side door, which laid rung by rung
    joins the two lands. Scripted in the desktop build, offline, by day
    and by night (the throwaway `src/arena/scratch_run.rs` again,
    removed): the arrow and the anchors, as above. On the VM, deployed
    with nobody on it for twenty minutes (Hajun's own game had ended at
    16:23), this PC's account was seated and played half a minute with
    seven of the computer's, two of their bridges across with their bends
    by then, the journal clean. APK 25 is built, its 77 asset files
    checked byte for byte, its library the stripped one and its number
    read back, and **not installed, no phone being attached** (APK 24 is
    kept beside it as `donggeurami_town-release.prev-2026-11-13b.apk`).
    **Not by hand, not on a phone, and no bot's builder or side door has
    been seen on a screen**: those are the tests' alone.
  - **What is mine**: the four colours and how pale their light is; every
    number of what a kind carries and how many it keeps; when each potion
    is drunk; that bots send builders at all, the two bridges worth one,
    and who opens a side door first; and where the compass's words stand.
- **Since 2026-11-13 there are five buttons for things to use, and you
  hold several kinds at once** (Hajun, with `new4morebuttonlayout.jpg`, a
  screenshot with four circles drawn on it: "i want to update the ui
  buttons on the minigame so that it has 4 more buttons that are hidden by
  default until either u press the ... toggle button or u buy an item that
  fills up another slot", and: "update the item shop purchase mechanic
  correspondingly so that it fills up the slots according to buy order so
  u can buy more than 1 item and hold it at once and adjust
  bridgewarsgameevent.txt file accordingly to remove any of the 'only can
  fill 1' mentions"). It is what makes room for more items to come.
  **`PROTOCOL_VERSION` 14: what a kit is on the wire has changed, so the
  server was deployed with it** (2026-11-12 15:49 UTC, the binary before
  kept on the VM as `roundtown-server.prev-2026-11-13`), **and APK 23 is
  told to update: APK 24 is the one that plays online** (below).
  - **The rules** (`shared/src/lib.rs`): a `Kit` has `slots:
    [Option<Stack>; UTILITY_SLOTS]`, five, a `Stack` being a kind and how
    many, where it had one `utility` and a `count`. Buying a kind you
    hold adds to its button (three fireballs, nine of anything else, as
    before, `most_held`); a new kind goes on the first empty button
    (`Kit::free_slot`), so they fill in the order things are bought; a
    button keeps its place until it is empty, and nothing moves up to
    fill a gap, so that what is under your thumb stays there. A potion
    whose effect still lasts keeps its own button with none left on it,
    the speed's while you run fast and the invisibility's while you are
    unseen, each by itself (`Outfits::tick`). `Outfits::drink` and
    `Use::Potion` say which potion. Dying empties every button, as it
    emptied the one. With five kinds sold and five buttons, "Slots full"
    (`words.market.taken`, "Slot full" until then) cannot come up yet: it
    is there for the sixth kind (which the dynamite is, since that
    night: above). `Kit::held`, `slot_of`, `potion` and
    `holding` are what everything else reads a kit by.
  - **Where they are** (`hud::slot_place`, `MORE_RING`, `MORE_ANGLES`):
    the second button where it was, and the four more on a second ring
    round the sword's middle, 38 shares of the short side out where the
    jump, second and bridge buttons are 20.5 out, at 0, 26.7, 53.3 and 80
    degrees up from straight left of the sword, each the others' size,
    3.5 clear of each other and of the first ring, the last ending beside
    the dots. Measured off the sketch, whose circles are 37 to 40 out at
    0, 25, 52 and 80 degrees, and evened out as the first three were. A
    test keeps every button there clear of every other.
  - **When they are seen** (`utility::fill_slots`, `slot_shown`,
    `MoreSlots`): the second button always; each of the four more while
    something is on it; and the empty ones too while the three dots are
    on (`hud::MoreButton`, which did nothing until now), which are lit
    while they are, as the bridge button is, and off again at each new
    game (`fold_away`). A button put away takes no press (`Visibility`;
    Bevy's picking and `hud::Presses` both pass over what is hidden).
    (Since 2026-11-14 there are no dots, and a button is shown only with
    something on it, the second one too: above.)
  - **Using them** (`utility::buttons`, `aim`, `drink`): each as the one
    was, by what is on it: a hold and a drag to aim a fireball or a
    builder, a press to drink, the count in its corner, a potion's
    seconds over its own button. One thing at a time: while a finger
    aims, no other button aims or drinks; with a potion at your mouth, a
    press on its button puts it down and one on another potion's does
    nothing. On desktop Q is the second button, as it was, and 1 to 5
    the five.
  - **The bots keep to one kind a life** (`shopping` in
    `shared/src/bots.rs`: nothing on any button but what it fancies, and
    none of that left), so the games they were tuned by are the games
    they play: the session that made them ran the same sixty seeded
    games before and after, number for number. (For an hour: later that
    day each fills its buttons as it is like, up to all five, above.)
  - **What is mine**: that the dots never put away a button with
    something on it (it could not be used); which of the four fills
    first, the one beside the second button and then outward (53, 27, 80
    and 0 degrees), so that two or three things lie together under the
    thumb, where a plain sweep along the arc is `MORE_ANGLES` in another
    order; that an empty button looks as the empty second button did;
    the keys; and the bots' one kind.
  - **Hajun's file** says so now: the Utilities heading has the five
    buttons, the order and the stacking, and the fireball's and the speed
    potion's lines say "its button" where they said the 2nd.
  - **Checked on 2026-11-13, scripted, in the desktop build** (a throwaway
    `src/arena/scratch_run.rs`, removed, pressing the buttons with
    touches through picking). Offline: the dots showing four empty
    outlines and putting them away; healing, two fireballs, speed,
    invisibility and a builder bought in turn, a button appearing for
    each in that order; both potions drunk off outer buttons, "43s" and
    "31s" each over its own; a fireball thrown by a quick tap and one by
    a hold and a drag from an outer button, its band on the ground; that
    button gone once empty, the others where they were, and the next
    fireball bought back on it; a fall into the void emptying all five.
    Against a server on this PC: the same purchases sold by the server,
    the buttons as its kits have them, both drinks and both fireballs in
    its log. Unit-tested: the order, the stacking, a freed button filled
    first, each potion's button outlasting its count, death, the bytes,
    the layout. **Deployed and built at Hajun's word that night** ("Deploy
    and build now"), from the working tree as it stood, which held two
    other sessions' uncommitted work as well (the bots' second round with
    the compass and the teams' colours, and the app's opening, both said
    by their sessions to be finished): nobody had been on the VM for half
    an hour; after it this PC's own account was let in and seated in a
    game there, which ran half a minute with seven of the computer's and
    a clean journal. APK 24 is built, its 77 asset files checked byte for
    byte and its library the stripped one, and **not installed, no phone
    being attached** (APK 23 is kept beside it as
    `donggeurami_town-release.prev-2026-11-13.apk`). **Not by hand and not
    on a phone**: whether the outer ring lies well under a thumb, and how
    much of the island the buttons cover on the phone's screen, is unseen.
    Committed and pushed with the other two sessions' work as one commit
    (`ad147fd`) when Hajun said "push to git".

- **Since late on the night of 2026-11-12 each of the computer's players
  is someone, a knock no longer stuns them, and no two fights with them
  go the same way; the teams are red, blue, green and yellow; and the item
  shop sells a compass** (Hajun, playing the bots of that evening, in four
  messages: "when im fighting the bot, i attack the bot, and the bot is
  like 'stunned' or stop moving so they get knocked back, and while they
  are being knocked back they are not running toward me so its so easy to
  knock them off the map. make the bot run toward u even while being
  knocked back. also add some random bell curve variation to the
  'straightness' of their bridging. straight line bridge look too ai. also
  add variation to strategy ... your ai teammate going to idle when u step
  on the generator ... seem a bit weird. just let it stay on the generator
  even while ur collecting but the item collection goes to the player ...
  there is a very easy way to kill the bots which is make them chase you
  on the bridge to the center island, and then right when u reach the
  center island, turn around and knock them off ... think of some way to
  randomize this interaction and also reverse this interaction ... so that
  there is no 'optimal' way to kill bots"; "team should be red, blue,
  green, yellow. and the name color should indicate the team. and add the
  Player compass tracker item in the bridgewarsgameevent.txt as well. also
  i want ai to jump while running/ and also randomly while attacking. more
  nonlinear/curve/slightly random not efficient movements to make it seem
  like a real player"; "it should be like a personality/playstyle, and at
  game start the bots each take a certain personality/playstyle that is a
  predefined set of probability distribution for the if ___ then do ___
  thing ... there should be more personalty than 8"; and "check if the
  enemy/team character's health bar is displaying correctly after they
  take damage"). The bots are `shared/src/bots.rs` still, the server's
  online and the device's own offline, so **online all of the bots' part
  is the server's, deployed at 15:17 UTC** (the binary before kept on the
  VM as `roundtown-server.prev-2026-11-12h`; `PROTOCOL_VERSION` 13 still
  then: the compass is a ware added after the rest and a kit's word of it
  is not sent, so APK 23 played on against it as it was). The colours, the
  compass and the health bars are the app's, **and APK 24 has them**: half
  an hour later, at 15:49 UTC, the five item buttons went out from the
  same tree as protocol 14 (their own account is beside this one; the
  15:17 binary kept on the VM as `roundtown-server.prev-2026-11-13`), and
  the APK built with them carries this round and the app's opening as
  well. From that deploy on APK 23 is told to update, so on a phone all of
  this comes with APK 24, built and not installed.
  - **Who each is** (`bots::PERSONAS`, `Persona`): sixteen ways of
    playing, called by the four letters people sort each other by. Each
    bot takes one by chance as it is seated (`Squad::seat`), two of a game
    the same as likely as not, and plays the whole game by it. Every number
    of one is odds or a leaning about 1, not a rule, and what a bot does at
    each turn is still thrown for: how much it wants to be the one that
    goes out (`drive`), its odds of each opening (`opening`), what it
    spends on first (`spends`: its sword, its shield, its second button,
    its team's generator, its team's defences, its team's swords), what it
    puts on its second button (`button`: a fireball, a speed potion, an
    invisibility potion, a healing potion, thrown again each life,
    `Bot::fancy`), how readily it goes in, how much it jumps and how widely
    it weaves (`bold`, `bouncy`, `weave`, each bot's own a little off its
    kind's, `Temper`), its leaning to each way of fighting (`fights`), how
    much it goes for throwing people off an edge and minds being thrown
    off (`shoves`, `minds`), and how it raids (`eager`, seconds sooner or
    later than 2:30; `raids`, added to a raid's score; `lingers`, times
    the 45 s; `anchors`, how likely it is there for the anchor rather than
    for whoever is home and their pad). ESTP the daredevil, the action
    items and into every fight; ESFP the troll, an invisibility potion and
    over to someone's island to lurk by its edge; ENTJ the commander,
    straight for the anchors; ENTP the trickster, odd openings, a fireball,
    in and out; ENFJ the captain, its team's swords and shields; ENFP the
    free spirit, a speed potion, all over the map, jumping; ESTJ the
    enforcer, head on and on to the anchor; ESFJ the host, traps and a pool
    for whoever comes; INTJ the mastermind, everything bought first and
    then to battle; INTP the tinkerer, its generator and no hurry to fight;
    INFJ the watchman, waiting beside where your bridge comes ashore; INFP
    the dreamer, out of everyone's way; ISTJ the sentinel, by the book;
    ISFJ the defender, the smaller islands and not the middle, its
    generator and its team's defences; ISTP the lone wolf, a sword, round
    you and off the edge with you; ISFP the drifter, a potion in hand and
    never mind the anchors. The server logs who is who as a fight begins
    ("Bridge Wars game 0: 1 Kimbap is INTP, 2 Nana is ESTP, ...; the teams
    open Middle, Late, Middle, Middle"). Nothing on the screen says who is
    which: it shows in how they play.
  - **Who keeps the base, and how a team opens.** Of a team's two bots
    the one that wants to go out more goes (`Persona::drive`; the higher
    seat until then), the other keeps the base; beside a person the bot
    keeps, as before. The one that goes out settles how its team opens
    (`Plan`, `Persona::plan`, `Squad::settle`; `Opening`): **to the
    middle** at once, as every team did until then; **straight to one of
    the two smaller islands beside its own** (75 m, four buys, for its
    sapphire, the middle left until its kind's `middle_from`, a minute for
    some and five for others); or **later**, arms first, both of the team
    buying as the one that keeps does, and no bridge until 45 to 120 s in.
    In two hundred games of bots 62% of teams opened to the middle, 17% to
    a smaller island and 21% late, and won 26%, 23% and 22% of the time.
    From 4:00 a team that has built no bridge still has one built to it.
  - **What they buy** (`shopping`): everything a bot might buy next keeps
    its place in the order they all bought in, and its kind's leaning to
    that sort of thing moves it up or down: one puts a fireball before its
    sword, another its team's defences before its generator. **The speed
    and the invisibility potions are used now**: a speed potion drunk as
    it sets out on a raid or a hunt, an invisibility potion within 62 m of
    the middle of the island it raids, to walk onto it unseen, each with
    nobody of another team within 9 m. The
    Bridge Builder is still not for bots (it is since 2026-11-13, above).
  - **A knock carries a bot for a moment and no more** (`Body::step`,
    `Squad::act`, `Skill::reel`, `Skill::grip`). Until then a bot knocked
    was carried and nothing else until it had come to rest, which is what
    Hajun saw as stunned. Its legs are its own in the air and with a knock
    on it now, as anyone's are: 0.06 to 0.36 s after a knock lands
    (`reel`, mostly about 0.2) it is going about what it was doing again,
    which in a fight is at you; in the air over the void it steers for the
    nearest ground within 8 m (`REACH_BACK`); and carried toward an edge it
    walks against what carries it. How hard it steers is from half its
    pace to all of it (`grip`), and both are thrown again at every knock:
    now it is back on its feet at once, now it goes over. On the ground it
    still walks only where a body can walk, and never off an edge by
    itself.
  - **How it fights is thrown for every few seconds** (`Stance`,
    `Squad::fight`, `take_stance`, `spot`). Until then every bot ran
    straight at whoever it fought and stood in front of them, wherever
    that was: on the last plank of a bridge, with them to one side of its
    end, where one punch put it in the void. Six ways now: **press**, up
    to them and at them; **circle**, round them as it punches; **poke**,
    in for a punch and back out of their reach to 3.6 m (`POKE_BACK`);
    **wait**, standing off 4.2 m (`HOLD_OFF`) for them to come; **ambush**,
    on the land beside where their bridge comes ashore, to one side
    (`lair`), so that whoever steps off it is thrown back over its side,
    which is Hajun's own trick turned on people and on other bots; and
    **rush**, off the bridge it is on and past them onto the land at a
    run. Which, and for how long (1.4 to 4.2 s, an ambush 5 and a rush 3),
    goes by
    where the two of them stand, on the land or out on a bridge, by how
    bold it is and by its kind's leanings: on the land it presses, circles
    or pokes; with them coming over a bridge it mostly lies in wait or
    waits; out on a bridge with them on the land ahead it stops short of
    their reach or runs for the land past them; and it is thought of again
    whenever either steps onto a bridge or off one. Having stood off
    someone on the land from out on a bridge, three times in ten it gives
    them up for 3 to 8 s (`Bot::shun`) and goes about something else.
    **Where it stands to punch** is the best of twelve places 1.45 m round
    them (`RING`): better for being near, much better if a punch from
    there would carry them off the land (`flung`, by `shoves`), worse if
    one of theirs would carry it off (by `minds`), and a little by chance.
    Its punches are not to the beat either (every 0.6 to 1.04 s, the first
    0.15 to 0.48 s after it has someone in reach), now and then thrown
    with a jump, and it faces whoever it fights whichever way it walks.
  - **It jumps and it does not walk a ruler's line.** Running somewhere,
    it jumps now and then, the more the bouncier it is (`Body::jump`, the
    game's own jump; not with a plank in its arms or a potion at its
    mouth); in a fight it jumps to no purpose now and then, and with one
    punch in ten to four in ten. Walking a way, it weaves to one side and
    the other of it (`Squad::walk_to`: up to about 14 degrees either side
    for a plain one, half that for the straightest kind and nearly twice
    for the widest weavers; a third of that on a bridge, and none close to
    where it is going). With nothing to do it wanders its island from one
    thing on it to another, the keeper mostly by its anchor, where until
    then it stood on one spot.
  - **Their bridges bend** (`nav::Span::bend`, `Squad::bends`): a sweep to
    one side, a wave and a ripple along its length (so much of the sine of
    one, two and three half turns, each nothing at both ends, so that it
    leaves and meets the land where the straight one would), each as a
    bell rings it, and all of them by how straight this bridge is to be,
    thrown as a bell rings too: most a little wavy, a few all but
    straight, a few all over the place; on a line a little to one side or
    the other of the one between the two lands' middles. Never slanting
    more than 0.45 to the side for each metre on where it leaves and
    meets the land (`MOST_SLANT`, 24 degrees), nor more than a twentieth
    longer than the straight one (`MOST_BENT`), which is the metres a bot
    buys for it. Its planks are laid rung by rung along the curve, each
    headed as the curve runs there (`Span::rung`, `heading_at`), and cut
    and drawn by `bridge` as anyone's.
  - **Your bot teammate stays on the pad with you** (`Bot::yields`,
    `Squad::wants`): while you are within 3.5 m of your team's pad it
    stands on it and takes nothing, so what the pad makes is yours; when
    you leave, it takes what its own purchases need again. Until then it
    stepped off to the anchor as you came and back on as you left.
  - **The teams are red, blue, green and yellow** (`colours.arena.
    team_red`, `team_blue`, `team_green`, `team_yellow` in
    `assets/ui.ron`, `ArenaColours::team`): red the island at the bottom
    of the map, blue the one on its left, green the one at its top and
    yellow the one on its right. Everyone else's name over their head is
    in their team's colour (`arena::spawn_tag`; a teammate's green or an
    enemy's red until then, `ally` and `enemy`, gone from the file); a
    zombie's in its own (`zombie`). The bar under a name is still green
    for your team and red for any other (`hp_ally`, `hp_enemy`), which is
    what says friend or foe at a glance: mine. A team is written by its
    colour, "Team Green" where it was "Team 3" (`arena::team_name`;
    `words.arena.team_red` and the rest), and "You are on Team Red" is
    said over "Fight!" as the game begins (`your_team`), your own name not
    being over your own head. The minimap's dots are as they were, you red
    and your teammate green, whatever your team.
  - **The Player Compass Tracker** (`src/compass.rs`; Hajun's file: "2
    amethyst (one time per life purchase - lost on death, which is a non
    slot filling item that basically shows a small white arrow around
    your character that is pointing at the direction of the nearest
    nonteammate player + name (color of team) + '123m'"): the fourth card
    of the item shop's second row, "Tracker", "Nearest enemy"
    (`Ware::Compass`, `Kit::compass`, `words.market.compass`,
    `hud::Picture::Compass`), "Owned" once you have it. With it, a white
    arrow 10 shares of the short side out from you, about two metres of
    the map, points the way whoever of another team is nearest stands,
    with their name by it in their team's colour and how far off they are
    in whole metres. Yours alone, nothing of it sent. It points at nobody
    unseen, dead, or a zombie. (In APK 24 the arrow pointed back at you,
    its picture drawn upside down, and across the screen its words lay
    over it: both put right the next day, above.) **Close by it gets out
    of the way**: the
    words are put away with them nearer than 6 m, where their own name is
    on the screen (seen in a run: "Yeppi 2m" written across two names),
    and the arrow nearer than 3 m, where it would lie on them; each comes
    back three quarters of a metre further off than it went. Online the
    server sells it as it sells anything, to someone within 5 m of the
    item shop, and says so (`Tell::Bought`); **a kit's word of it is not
    sent** (`#[serde(skip)]`), so what the room is sent is what it was,
    and this device keeps its own word of yours (`arena::Game::compass`,
    put back on your kit as the room's kits come, `follow_round`) until
    you die. The computer's players do not buy it.
  - **The health bars over heads never shrank, and do now**
    (`arena::place_tags`, `fills_of`). Hajun was right. Since 2026-11-08,
    when the name went over the bar and the two were put in a column, the
    bar's fill was looked for among the tag's own children; it is the
    bar's child, so none was ever found, and every bar stayed full
    whatever its body's health, online and offline. The numbers floating
    off a body, the health itself and the deaths were right all along,
    and so was your own bar top left.
  - **`Skill::MODERATE` has two numbers more**, `reel` (0.06, 0.36) and
    `grip` (0.5, 1.0); the constants of the fights and the bridges are at
    the top of `bots.rs` (`RING`, `POKE_BACK`, `HOLD_OFF`, `CARRY`,
    `REACH_BACK`, `MOST_SLANT`, `MOST_BENT`), and everything of a kind is
    its line of `PERSONAS`. `Squad::cast(seat, name)` makes a bot someone
    for trying one out, and `Squad::set_plan` a team's opening.
  - **Whole games, again** (`server/src/arena.rs`): the eight-game test
    asks that someone be across to the middle within a minute, a smaller
    island reached within four, a sword within two, an upgrade within
    five, and no anchor broken before 4:00; no longer that every team be
    across within two minutes, nor that no runner stand 20 s, teams
    opening late or to a smaller island as they do. `many` prints each
    kind's games, wins and deaths and each opening's, and `RT_WHO=1` who
    was who. **Over two hundred games** (as deployed): over at 13:00 on
    average, the soonest at 10:07 and the latest at 13:42, none drawn; won
    by island 46, 64, 43 and 47; before 10:00 ten deaths by punches and
    six by falls a game, where it was six and fourteen: they are thrown
    off the map less than half as often, and kill each other more; 0.3
    anchors broken by bots before 10:00; each kind's team winning 19% to
    34% of its games, an ISFJ dying 1.6 times a game and an ESTP 3.9. And
    someone standing still beside the far end of a bridge, punching
    whatever comes off it twice a second, throws 217 of 960 bots into the
    void in half a minute, where 916 of them get onto the land
    (`crossings_to_someone_waiting_at_a_bridge_s_end`, an ignored test,
    by kind).
  - **Checked on the night of 2026-11-12, scripted, in the desktop
    build** (the throwaway `src/arena/scratch_run.rs` again, removed).
    Offline: "You are on Team Red" over "Fight!"; a teammate's name red
    over a green bar, the bots of the island on the left blue over red
    bars; the compass bought for 2 amethyst, its arrow up and to the right
    at "Ssal 181m", in yellow, from the red island, turning as I flew
    toward the blue one, and gone, with everything held, at a death; two
    bots' bars shrinking under my punches while theirs took me from 100 to
    16. Against a server on this PC: the compass refused 5.002 m from the
    shop, where a seat arrives, and sold standing at it; kept through the
    room's kits coming, and gone at the death the server logged; the
    server's bots' bars shrinking as it logged each hit; my bot teammate
    standing on the pad. Unit-tested: the compass's rules and whom it
    points at, a bent span's ends, length and planks, a knocked body
    brought back by its legs, and what each kind buys first. The VM,
    deployed with nobody on it for eleven minutes, let this PC's build in.
    **Not by hand, not on a phone, and no game has been played on the VM
    with it by me** (after the 15:49 deploy this PC's account was seated
    there and played half a minute with seven of the computer's, the
    journal clean). The cost of the brain on a phone is no more measured
    than it was.
  - **What is mine, Hajun having asked for the thing and not the
    numbers**: the sixteen kinds, their names and every number of them;
    that the one that wants to go out more goes; the three openings' odds
    and that a late team arms first; the six ways of fighting and their
    odds; the reel and the grip; how bridges bend and how far; that the
    bars stay green and red; the caption over "Fight!"; where the compass
    sits in the shop, its size and where it is drawn, that it leaves out
    the unseen and the zombies, that it gets out of the way close by, and
    that bots do not buy it.
- **Since the night of 2026-11-12 the computer's players play the
  game** (Hajun: "right now the other bots dont do anything other than
  stay in their base. i need the bots to be intelligent enough. think of a
  way to make this game enjoyable. the difficulty need to moderate - not
  too hard, not too easy ... maybe one way is every 1~2 seconds the bot
  scores a short list of goals and does the highest one ... and a bonus
  for the current goal so the bot doesn't flip between two every tick ...
  maybe treat islands as graph nodes and every bridge as an edge and bots
  pathfind over the graph and when no path exist, they buy meters for the
  straight line distance and bridge it"; the plan Hajun approved is
  `~/.claude/plans/right-now-when-2-lexical-goblet.md`). They collect,
  shop, bridge to the middle and to the smaller islands, buy their team's
  upgrades, raid, defend, back off, break anchors and, at the end, hunt.
  **Online all of it is the server's** (deployed three times that night:
  at 12:02 UTC, at 12:44 with a fix, and at 13:19 as it stands, the
  binaries before kept on the VM as `roundtown-server.prev-2026-11-12e`,
  `f` and `g`; `PROTOCOL_VERSION` 13 still, nothing that is sent having
  changed, so APK 22 plays against them as it is, and **tuning them
  online is a server deploy, not an app update**). `RT_ARENA_BOTS=old`
  in the VM's `/etc/roundtown/env`, and a restart, goes back to the bots
  that kept to their islands, without a deploy (`Config::arena_old_bots`,
  `Game::old_bots`). **A game played offline has the same bots, by the
  same brain, since APK 23** ("Offline", below); an app from before has
  the old ones offline, and these online.
  - **The map the server was missing** (`shared/src/nav.rs`,
    `roundtown_net::nav`). The server has no map, which is why its bots
    kept to the disc of their islands. `Grid` is Hajun's
    `pvptownmap.glb` as a body walks it, half a metre to a square
    (`CELL`): each square's floor, whether a body fits there and can walk
    to it (`Grid::clear`: its whole footprint on ground, nothing reaching
    into it, no step higher than a step), and which piece of land it is
    (`Region`: a team's island, one of the four smaller islands, the
    middle, or the pond, `Pocket`, which is standing room nobody walks
    into). **It is baked out of the map by the game's own collision**
    (`src/arena/nav_bake.rs`, a test: flooded from the arrivals and the
    pads with `Island::floor` and `Island::blocked`, as a body is moved)
    into `shared/src/arena.nav`, 39,832 bytes, built into both binaries
    (`Grid::arena`). A team's island comes to 294 m² a body can walk, a
    smaller one 422, the middle 15,883. **After any re-export of the map:
    `RT_BAKE_NAV=1 cargo test --lib nav_bake`, which writes the file
    again, and deploy the server**; without it
    `the_grid_built_in_is_the_map_s_as_it_is` fails, saying so. **The map
    is a star**: every island reaches every other through the middle (a
    team's island to the middle 43.6 m, a smaller island to the middle
    37 m; a team's island straight to a smaller one is 75 m, never worth
    it), so "no way" always means one of eight straight spans is missing.
    `Nav` is the grid with every bridge laid on it as its rungs come
    (`Nav::hear`, from people's `Ask::Rung` and the bots' own, in index
    order as `bridge` takes them): the squares within a plank's half
    width less a body's of a plank's middle line are walked (`deck`),
    which pieces can be walked between is worked out again at each plank
    (`relabel`, `joined`, `connected`), a way is the shortest over the
    squares, straightened (`Nav::path`, A*), and `on_plank` and `ground`
    say what holds a body knocked there. `Span` is the straight bridge
    across one of the star's gaps (`Grid::span`): from half a metre
    inside the one shore to half a metre inside the other, 33 planks and
    44.6 m on an axis, which `bridge` cuts along both shores as it does
    anyone's (a test lays one on the real map, rung by rung,
    `a_span_the_computer_lays_is_a_bridge_like_any_other_on_hajuns_map`;
    the plank's measures are `nav`'s now, `PLANK_HALF_WIDTH`,
    `PLANK_SPACING`, `PLANK_TOP`, which `bridge` reads). **A plank laid
    costs next to nothing** (`Nav::lay_deck`, `joined_to`, `piece`): a
    plank only ever adds squares to walk, so it joins pieces and never
    parts them, and its own squares take the number of whatever is beside
    them (the four beside: a step cornerwise needs both squares at the
    corner) and join what they lie between. For its first hours every
    plank had the whole map gone over again, 318,096 squares, 3.3 ms on
    this laptop and a dozen times a second as a game begins; a 33-plank
    span is 26 millionths of a second now where it was 113 thousandths.
    **A way is searched for greedily** (`GREED`, 2: what is left weighed
    twice what is done, each square gone on from once): from a team's
    island round the pond to the far side of the middle in 0.09 ms where
    the shortest took 2.1, and to the island across the map in 0.35 ms
    where it took 4.8, the ways found, once straightened, a metre longer
    in 274 and 3.6 m shorter in 161 (`timing_of_the_arena_s_map`, an
    ignored test, prints them). And **a bot keeps the way it has while bridges are built**: a
    plank only adds ground, so only one with no way looks again as one is
    laid (every bot did, at every plank anyone laid). All three were
    found measuring for a phone, where the bots' frame is the game's.
  - **The brain** (`shared/src/bots.rs`, `roundtown_net::bots`, plain
    code with dice of its own, so the same seed is the same game).
    `Squad` is every bot of a game. Each has **a slow layer** (`think`,
    every 1 to 1.5 s, not all at once): what it might be doing, each with
    a score, the best taken, the one it is on getting 0.15 more
    (`Goal`: `Retreat` 0.95, `Defend` 0.88, `Hunt` 0.8, `Lay` 0.78,
    `Shop` 0.72 at home, `Raid` 0.6 to 0.79, `Collect` about 0.5,
    `Fetch` about 0.5 to 0.8, `Idle`), a goal it gets nowhere with for
    3 s, or asks the shop for three times and is not given, left out for
    a while (`barred`); and **a fast layer** every frame (`act`), which
    follows its way there, fights whoever comes into reach, lays its
    planks and keeps to the ground. `Body` is its body: walked only
    where a body can walk, so **it never walks off an edge**; carried by
    a knock as anyone is, landing on whatever ground is under it, land
    or plank, and falling where there is none, which is how one is
    beaten on a bridge, and how it beats you; knocked into the pond, it
    hops out over the rim. What a frame comes to that the game has to
    carry out is an `Order` (a punch, a strike at an anchor, a purchase,
    rungs laid, a throw, a drink), which the server does as it would for
    a person who asked (`Game::play_bot`): **nothing is free, and
    everything is lost on death**, the metres of bridge with it.
  - **How they play.** (Since later that night who keeps and who goes
    out, how a team opens, what each buys first, how each fights and how
    each raids go by who the bot is, above; what follows is how they all
    played for those hours, and is still what the kinds lean away from.)
    Each team's lower seat keeps the base (`Role::
    Keeper`) and the other goes out (`Runner`); one alone does both.
    **The opening**: the one that goes out has the pad's first 8 silver
    (the keeper wants none until the bridge is across), buys 50 m and
    lays the span to the middle at the pace anyone builds at, 5 m/s, each
    plank a rung or two ahead of its feet: across by 25 to 29 s. The
    keeper buys a Basic Sword and a shield. **Then** the one that goes
    out takes the amethyst from the pad on its side of the middle,
    bridges from the middle to one of the two smaller islands beside its
    own (by 65 to 69 s) and brings sapphire home. **What they buy, in
    order** (`shopping`): the metres its bridge is short of; with
    sapphire the Heal Pool, the generator, Stronger Swords, Better
    Shields, a trap (the first upgrade at 83 to 89 s); the Aura Sword
    with 4 amethyst and the Aura Shield with 6; the Silver Sword once
    the gold is near, and until then the basic one; a shield; a healing
    potion for the second button, or, the one that goes out, from 5:00
    and with 40 silver, a fireball; the Advanced Shield. **At a pad a
    bot takes only what its next purchases need** (`Squad::wants`,
    `Mint::pick_up_where`) and leaves the rest lying; people are served
    first, whatever their seats, and of a team's two bots whichever it
    matters most to. **Fights** (`foe`): it goes for whoever of another
    team comes within 6 m at first, growing to its full sight, 24 m,
    between 2:30 and 10:00 (`Skill::aggro_at`), or within 9 m for 3 s
    after it is hit; anyone on its island, from anywhere (`Defend`: it meets
    them on the land, it does not hold the end of a bridge); and on an
    island it raids, whoever is home. Its first punch comes 0.3 s after
    it has someone in reach, since it is drawn a moment behind on your
    screen, then every 0.8 s; it knocks as a punch on the run only when
    it really was running in (every bot punch did, until then). Whom a
    team's runner raids first is thrown differently in every game
    (`Squad::salt`). **Under
    35% of its health with someone on it, it backs off** to its pad
    until it has 60%, and drinks its healing potion once nobody is
    within 6 m; 50% once its anchor is gone and dying is for good.
    **Raids** (`Goal::Raid`): none before 2:30. Then the one that goes
    out, with a sword and 70% of its health, goes to another team's
    island for no more than 45 s, fights whoever is home and takes from
    their pad until it holds 20 silver and 7 gold; home 40 s after, or
    after dying on one. One team raids a team at a time, a team raided is left alone
    for 90 s, and **a team with a person on it gets no more than its
    share**, one raid in three while there are two other teams to go to
    (`Squad::raids`, `spared_until`, `raided`). **Anchors** are left
    alone before 4:00, struck by one bot at a time every 2 s (a person
    can strike twice a second), and the bots break no more once two are
    broken, before 8:00. From 5:30 the keeper goes along with its
    teammate. From 4:00 a team that has built no bridge has one built to
    it from the middle: nobody sits a game out behind the void. **From
    13:00 everyone hunts** whoever of another team is nearest that can
    be walked to, three minutes after the timetable breaks the anchors,
    so that the end of a game is minutes and not moments. **A
    fireball** only at someone 4 to 10.5 m off with land under them,
    never at someone out on a bridge. Not for bots yet: invisibility,
    the speed potion, the Bridge Builder (the two potions since later
    that night, above; the builder still not).
  - **A bot whose teammate is a person keeps the base** whichever its
    seat: it leaves the pad's first silver to you for 40 s, then, if
    nothing joins your island to the middle, buys the 50 m and lays the
    bridge itself (seen on the VM: begun at 40 s, across at 51 s); then
    it stands at the pad taking what its own purchases need, the rest
    piling up for you, steps off to the anchor when you come within
    3.5 m of the pad (until later that night: it stays on the pad with
    you since, taking nothing while you are there, above), defends, and
    from 2:00, with nobody of another
    team within 40 m, bridges to a smaller island and fetches its
    sapphire, and the amethyst on your side of the middle, for the
    team's upgrades. It never raids by itself.
  - **Every number of how hard they play is `Skill::MODERATE`**, in one
    place: 1 to 1.5 s between thoughts, the 0.15 for the goal it is on,
    0.3 s and 0.8 s for a punch, 2 s for an anchor, 6 m growing to 24
    between 2:30 and 10:00, 35% and 60%, raids from 2:30, anchors from
    4:00, the keeper along from 5:30, two anchors spared until 8:00, 45 s
    a raid, 40 s after, 90 s for the raided, a third for people, the
    hunt from 13:00, 50% once the anchor is gone, 6 s between fireballs.
    All mine, from the plan, but 13:00: with the hunt at 10:00, as
    planned, every game ended within half a minute of the anchors
    breaking.
  - **How the server shows them**: a bot is moved by the game and sent
    as it moves (`flush`), and once more as standing still when it stops
    (`Spot::still`), so that nobody watching carries it on the way it
    was going; its rungs go to the room as a person's do (`Event::Rung`,
    a `Plank` each), so its bridges are drawn, stood on and on the
    minimap as anyone's; what it takes from a pad is taken from the
    room. **In APK 22 a thing a bot takes does not fly to it** (it
    looked only for people, the computer's picking up nothing until
    now): it goes where it lay, or to a person standing within reach.
    `loot::follow_server` takes anyone but a zombie for the taker since,
    which APK 23 has. Nobody sees a bot carry a plank or
    drink, as nobody sees anyone else do either.
  - **Whole games are played in tests** (`server/src/arena.rs`,
    `bots_alone(seed, old)`, `play`, `chronicle`):
    `the_computer_s_players_play_whole_games_sensibly`, eight games of
    bots alone in about four seconds, keeps every team across to the
    middle within two minutes, a smaller island reached within four, a
    sword within two and an upgrade within five, nobody ever walking off
    an edge, no runner idle for 20 s, no anchor broken and nobody out
    before 4:00, every game over before 20:00, three of the four islands
    winning at least once, and the same seed the same game; two more put
    stand-in people in (`with_people`): a team of people raided no more
    than its share with its anchor left alone at first, and a person's
    bot teammate keeping the base, bridging for them and never raiding.
    `cargo test -p roundtown_server many -- --ignored --nocapture` prints
    sixty games as a table to tune by, or as many as `RT_GAMES` says,
    with who won by island, and `watch` one game's raids and deaths
    (`RT_SEED`, `RT_BRIEF=1`). **Over three hundred** (2026-11-12, as
    deployed at 13:19): over at 11:04 on average, the soonest at 8:31
    and the latest at 13:19, none drawn; won by island 90, 69, 68 and
    73; before 10:00 six deaths by punches and fourteen by falls a game
    (a punch knocks 4.5 m, and an island is 20.8 across) and 1.5 anchors
    broken by bots; every team across to the middle by 30 s, a smaller
    island reached by 69 s, a sword by 55 s and an upgrade by 88 s, in
    every one of them.
  - **Two things the first deploy had wrong, found that night and
    deployed at 12:44.** **A body knocked off an edge and carried in
    under an island or a plank as it fell hung there**, neither on the
    ground nor in the air, for the rest of the game: unseen under the
    land, never dying, its team never out. Three of forty games of bots
    went to 25:00 as a draw for it, which I had first put down to two
    teams never finishing each other; with it mended, none of three
    hundred did (`Body::step`: under the ground is in the air, and goes
    on falling; a test drops one under an island). And **a body stood a
    little under the ground it is on is put onto it**: a device's
    arrivals are at the land's own height, a quarter of a metre under an
    island's top, and offline every bot stood there for good, neither
    free nor falling. Found by playing the bots offline, where they are
    seen.
  - **Checked on 2026-11-12**: with a server on this PC and a scripted
    desktop copy watching from above (a throwaway `scratch_run`, role
    `w`, removed since), a runner's bridge growing from its island and landing flush on
    the middle's rim, on the minimap too; my bot teammate on its bridge;
    a smaller island bridged, with sapphire on its pad; swords, shields,
    potions, the Heal Pool and a trap bought; at 3:12 a raider on my
    island, blinded by the trap, knocking the watcher off the edge. And
    **on the VM after the deploy**, with this PC's own account: seated
    with seven of the computer's, three bridges across at 28 s, my bot
    teammate's begun at 40 s and across at 51 s, bridges to two of the
    smaller islands by 96 s, the journal clean, and 1.1 to 1.6 KB/s out
    to a player in a game; and again after each of the two deploys
    after, a minute of a game each. Not by hand, and not on a phone.
  - **Offline, the same bots** (APK 23; `arena::drive_bots`, `Brains`,
    `Brained`). A game played offline has a map and a brain of its own
    (`Nav::arena`, `Squad`, with `Skill::MODERATE`, the server's own),
    and each of the computer's seven is **a body put where the brain has
    it each frame, in nobody's way** (`lobby::spawn_bot_at`: no `Intent`
    and no `Collider`, so `move_bodies` neither walks it nor bumps you
    into it): you walk through them, as you do online, where until now
    offline they bumped. It keeps a `Shove` and a `PlayerJump`, which is
    what your punch, a zombie's and a blast knock (`Shove::knock`): the
    knock is handed to the brain (`Squad::shoved`), which carries the
    body by it, off an edge if it must, and `reap` finds it fallen as it
    does anyone. What a frame of a bot comes to is carried out as the
    server carries it out: its punch knocks whoever it lands on and
    lands (`Landed`, so the damage, the number and the death are
    `deal_damage`'s, as yours are), an anchor is struck (the file's
    `arena.damage` and a sword's, `anchor_hurt`), a purchase made at the
    stand out of what it holds (`Brains::purses`, as yours is
    `Resources`), its rungs laid as someone else's
    (`bridge::Bridges::lay_remote`, so its bridges are drawn, stood on
    and on the minimap), a fireball thrown, a potion drunk. Every bridge
    is told to the brains' map as it is fixed, yours and a Bridge
    Builder's among them (`Bridges::laid`, by `Bridges::version`). It
    takes what its purchases need of what lies in its reach after you
    have had yours that frame, each thing flying to it; dead, it loses
    what it held, and stands again as the brain's too. **Laying a
    bridge, it carries a plank** (`Carrying`, `bridge::carry_for_bots`,
    `BotPlank`), which online nobody is seen to. `hunt` is the zombies'
    mind alone now, and they go for the computer's players as for you
    (its `Intent` is optional for that). The bots walk by the brain's
    numbers (a run of 10 m/s, a builder's 5), and are hurt and knocked
    by the file's (`assets/ui.ron`'s `arena` and `melee`).
  - **"Your anchor is under attack!"** (`arena::warn_of_the_anchor`,
    `words.arena.anchor_attacked` in `assets/ui.ron`; APK 23, online and
    offline): said in the line the timetable's are said in whenever your
    team's anchor has lost health and still stands, no oftener than
    every 8 s (`ANCHOR_WARN_EVERY`). Nothing said it until now, and
    until now only a person ever hit one. Broken, it has its headline,
    as before. Mine, with the plan.
  - **`loot::follow_server` flies a thing to whoever took it, a bot
    too** (APK 23), anyone but a zombie being looked for.
  - **Checked offline on 2026-11-12, scripted, in the desktop build**
    (`RT_SERVER=off`, the throwaway again, with a role that watches from
    above and punches whoever comes into reach): three bridges across in
    half a minute, the planks in their arms; my bot teammate's at 40 to
    51 s; smaller islands reached within 75 s; swords, shields, potions
    and the team's upgrades bought; at 3:00 a raider on my island, its
    name and bar in red, taking 11 through its shield from a punch of
    mine and thrown up by it,
    my teammate coming home over our bridge to see it off, and the
    raider killed and standing again on its own island with nothing,
    back at its pad for a sword; a raider's three strikes of 22 on
    another team's anchor; "Your anchor is under attack!" across the screen as
    mine lost health; from `RT_ARENA_FROM=890`, the twelve zombies and
    the bots at each other, and seven bodies knocked into the void in
    twenty seconds, which with the anchors gone ended the game. No
    error in any log. **Not by hand, and not on a phone**: a phone has
    never run the brain, and the cost of it there is worked out, above,
    not measured. APK 23 is built, its 77 asset files checked byte for
    byte, and not installed, none being attached; APK 22 is kept beside
    it as `donggeurami_town-release.prev-2026-11-12d.apk`.
  - **What is mine, Hajun having said only "moderate"**: every number
    above; that the lower seat keeps and the other goes out; the order
    they buy in; that they take only what they need at a pad; that the
    raids are steered (one at a time on a team, a rest, people no more
    than their share, anchors slow and capped early); that a person's
    bot teammate bridges for them after 40 s and never raids; that a bot
    throws no fireball at someone on a bridge; that bot teams fight each
    other as well as you; that offline you walk through them; and the
    line about your anchor, its words and its 8 s.
- **Since the evening of 2026-11-12 every game online is 2v2v2v2, people
  teamed up with each other first and the computer in every seat left**
  (Hajun: "right now, when 2 real players queue for the bridge wars game,
  it becomes a 1v1. i want to change it so that this game is always
  2v2v2v2, with bots filling the remaining slots, and the 2 player become
  same team. and if its 4 players, then those 2 players and 2 players are
  same team too, and so on, so that real players will be teamed up with
  each other first, and bots on remaining teams"; server deployed, the
  binary before kept on the VM as `roundtown-server.prev-2026-11-12d`;
  `PROTOCOL_VERSION` 13 still, nothing that is sent having changed, so APK
  22 plays it as it is).
  - **Seats are taken a team at a time** (`SEAT_ORDER` in
    `server/src/arena.rs`, 0, 4, 2, 6, 1, 5, 3, 7): both seats of the first
    island, both of the one across the map from it, then both of each of
    the two at its sides. Two people are a team; four are two teams on
    opposite islands; the odd one out has the computer for a teammate.
    Pairs go by the order people join in: mine, there being no parties.
    Ten seconds after the last to join, the computer takes **every** seat
    left, however many people there are (for half a day two to four
    people began with no bots, an island each).
  - **A seat left before the fight begins is anyone's again**
    (`Game::leave`, `vacate`): in the waiting room, the next person to
    come or the computer takes it; left during the countdown, the
    computer has it as the fight begins, on its island from the start
    (`seat_bot(.., landed)`). Until then it stayed its leaver's, empty,
    and a team began short. Once the fight is on, a seat left stays
    empty: mine.
  - **The server's fight is a plain function of the game now**
    (`Game::step`; nothing a player sees has changed by it): where
    everyone is is kept in the `Game` itself (`at`, `yaw`: a person's
    copied in each frame from what their device last said, `sync`; a
    bot's the game's own, to the last of a metre where a `Spot` is whole
    centimetres), and whatever a step does that takes the world, an
    entity for the room or a word to a device, it only says (`Event`:
    `Made`, `Taken`, `Thrown`, `Burst`, `Knocked`, `Land`, `Zombie`),
    for `flush` to do after. So a whole 25-minute game of the
    computer's players is played in a test, in a few hundredths of a
    second (`a_whole_game_of_the_computer_s_players_plays_itself_out`),
    which is what the bots to come are tuned by. Each game throws its
    own seeded dice.
  - **Checked on 2026-11-12 with a server on this PC and scripted desktop
    copies** (a throwaway `src/arena/scratch_run.rs`): two people in
    seats 0 and 4, on one island 4 m apart, with six of the computer's
    players; one of two leaving the waiting room, and the computer in
    their seat; what the generators make picked up, a fireball and a
    sword bought, a bot's punches knocking a person off its island, the
    fall, standing again with nothing held; a fireball thrown at a zombie
    bursting on it for 20; the twelve zombies risen, on their own team,
    at `RT_ARENA_FROM=895`; the game skipped to its end, and a second
    with nobody left over from the first. The VM, deployed with nobody
    on it, let this PC's build in. Not on a phone.
  - **This was the first part of a larger job, the computer's players
    learning to play** (collect, shop, bridge, raid, defend), which came
    that night (above). For those hours the computer's players kept to
    their islands, as before.
- **Later on 2026-11-12: dying loses everything you hold, what is aimed
  is seen over the void, and a sword is held on its edge and cut
  downward** (Hajun: "for the bridge pvp minigame, make it so death make u
  lose all ur silver, gold, sapphire, amethyst, bridge meters. also make
  the hit indicator for fireball appear even when ur on a bridge (the hit
  indicator should appear even on void area) and shootable on bridge. also
  instead of removing the code for the sword animation now (cuz i might
  wanna change it back after trying this new way), make a new sword
  animation, this time instead of holding the sword flat, hold it narrow
  and slash downward(?) and use that animation instead. try to choose the
  holding angle/slashing animation look good enough even when its like
  this way and look nice enough in a top down view. then rebuild apk and
  push to git"; `PROTOCOL_VERSION` 13 still, nothing that is sent having
  changed; the server deployed for the first of the three, the binary
  before kept on the VM as `roundtown-server.prev-2026-11-12c`; APK 21).
  - **Dying loses everything you hold**: your silver, gold, sapphire and
    amethyst with the metres of bridge, the sword and what is on the
    second button; the shield and what your team has bought stay, as
    before. Online it is the server's (`Game::die` in
    `server/src/arena.rs` empties the seat's purse, and the room is sent
    it with the rest, `Purses`), so an app from before, APK 20, loses
    them online too; offline it is this device's (`arena::lose_on_death`,
    `lost_on_death`), as the metres are online as well. Until then what
    you held was kept through a death, which was my call, Hajun's file
    saying only "all items are lost upon death" of the item shop. They
    are lost, not dropped: nobody else gets them. Checked in the desktop
    build, scripted, with a sword, a shield, a fireball and 25 m bought
    and then walked off the island: offline, [172, 80, 60, 60] to nothing;
    against a server on this PC, [122, 200, 200, 200] to nothing as the
    server logged the fall, the shield kept both times.
  - **The band of what is aimed has a dark rim round it** (`range::rim`,
    `RIM`, 0.07 m just outside its edge, in `colours.arena.aim_rim` of
    `assets/ui.ron`, black at a half; `utility::spawn_band`): the
    fireball's and the builder's. The band was drawn over the void all
    along, at your feet's height wherever you stand, but it is white,
    and by day so is the sky, nearly: measured in a screenshot of the
    desktop build, the sky (140, 209, 247), the band's fill over it (163,
    213, 243) and its outline (212, 224, 233). Off the edge of an island
    it was all but gone, and from a bridge nearly all of it lies over the
    void. With the rim it is seen over anything; over land it is a thin
    shadow round the white. **Throwing from a bridge was never refused,
    and I could not make it fail**: in the same scripted run, standing on
    a bridge 13 m out over the void, three fireballs were thrown, by a
    drag with building on, by a drag with it off and by Q, and each flew
    and burst; and the VM's log has Hajun's three of that morning bought
    and gone from the button within half a minute. If it still will not
    throw from a bridge on the phone, it is something this did not find:
    ask what was pressed, and for the log. The punch's own range has no
    rim, nobody having asked: over the void it is as faint as the band
    was.
  - **A sword is held on its edge and cut downward** (`arena.sword_swing:
    Chop` in `assets/ui.ron`, `ui::SwordSwing`; `chop_offset`,
    `chop_blade`, `chop_angle` in `src/lib.rs`; `gear::edge_on`, `Cut`,
    `Cutting`). **The first way is all still there, and `sword_swing:
    Sweep` is it** (`sword_offset`, `blade_way`, `gear::pointing`,
    `Trail`): the one word in the file goes back, on desktop as the file
    is saved, on a phone at its next build. The blade moves in one plane
    with its edge leading, as a sword is cut with: an upright plane
    through the way you face, leant over to the right by `CHOP_LEAN`,
    0.62 rad, 36 degrees. Upright altogether, the camera over you would
    see the blade and its cut edge on, a hair; flat is what Hajun asked
    to leave; leant so, it sees a slim blade, about half its breadth, and
    the cut as a crescent. At the ready the blade was out in front and up,
    0.5 rad over level in that plane, which from above points a little out
    to the right (straighter since that afternoon, below); a press raises the arm (`CHOP_ARM_FROM`, 2.35 rad
    forward of hanging, where level is 1.7) with the blade up and back
    over the right shoulder (`CHOP_FROM`, 2.0 rad over level), and brings
    both down through straight ahead to low on the left (`CHOP_TO`, -0.5;
    `CHOP_ARM_TO`, 1.15, and across, `CHOP_ARM_OUT` to `CHOP_ARM_IN`) in
    the time a punch goes out, eased, and back to the ready in the time
    it comes back. From over you: the sword flicks out to your right and
    back, whips over to the front, and ends a little left of straight
    ahead, inside the range drawn there. It is drawn thicker through its
    flat than the other way (`gear::CHOP_SCALE`, 3.0 where the flat
    sword's is 1.7), that being what the camera sees most of. **Its trail
    is the ribbon the blade itself has cut** (`Cutting::follow`,
    `gear::ribbon`): from a third of the way up the blade to its tip
    (`BLADE`, each model's own length, which a test reads off the
    models), a sample to every 5 degrees the blade has turned whatever
    the frame rate (`CUT_STEP`), the mesh built again each frame of the
    cut and faded after as the other trail is; fuller than that one along
    its length (`CUT_RAMP`), the camera seeing it from nearer its edge. A
    press mid-cut begins the next, as with a punch. What a punch lands
    on, and when, is as it was, and so is that nobody else sees your
    swing: someone else's sword is at the ready, the new way. All the
    angles are mine. Seen in the desktop build, scripted, each of the
    three swords: from the game's camera in bursts of twelve shots
    through a cut, and close from the front, the side and behind, at the
    ready, cutting, and walking and cutting; the first numbers (a lean of
    0.55, a thickness of 2.6 and the other trail's ramp) left the trail
    too faint from above, and were raised.
  - **At the ready the sword is held straighter than it cuts** (that
    afternoon, Hajun, of the hold above, playing APK 21: "i think the
    sword holding is too tilted now (both forward tilt and side tilt).
    make it more straight"; APK 22, no server change): 30 degrees forward
    of straight up and 10 over to the right, where it was 61 and 36 (the
    blade 0.5 rad over level in the plane of the cut). My reading of
    "straight" is upright, as "hold it narrow and slash downward" was of
    a sword held up and brought down; the 30 and the 10 are mine, and
    **are the file's to change**: `arena.sword_tilt_forward` and
    `sword_tilt_side` in `assets/ui.ron`, degrees from straight up, read
    as the file is saved on desktop (`crate::Guard`, `Guard::of`;
    `Arena::check` refuses anything outside 0 to 90). Straight up
    altogether the camera over you sees the sword end on, a dot: at 30
    and 10 it sees a blade about half its length, pointing nearly
    straight ahead, 0.7 to 0.9 m of it. The cut is as it was, in its own
    plane (`CHOP_LEAN`, `cut_angle`, `cut_blade_at`): the sword goes from
    the hold to the cut as it is raised, and on the way back both its
    angle and its plane's lean come back to the hold's (`chop_angle`).
    The arm at the ready is where it was. Seen in the desktop build,
    scripted, each sword at the ready from above, from the side, the
    front and behind, and through a cut and back; unit-tested with a hold
    of its own, not the file's. Not by hand or on a phone: APK 22 is
    built, its assets checked byte for byte, and not installed (APK 21 is
    kept as `donggeurami_town-release.prev-2026-11-12c.apk`).
  - **Unit-tested**: the server's purse at a death, and the device's;
    the rim round the band; the cut's poses on the model, its blade's
    path, that the flat stays square to the cut, the ribbon at 3 and at
    30 frames to a cut, and the blades' lengths against the models. The
    VM, deployed while nobody was on it, let this PC's build in. **Not
    by hand, and not on a phone: none was attached, so APK 21 is built,
    its assets checked byte for byte, and not installed** (APK 20 is kept
    beside it as `donggeurami_town-release.prev-2026-11-12b.apk`). The Adreno has
    not drawn the rim or the ribbon (both blended, as the band and the
    other trail are).
- **Since 2026-11-12 Bridge Wars has two shops, and everything in Hajun's
  file is in the game** (Hajun: "i updated the bridgewarsgameevent.txt.
  update the generator rates, then add all the itemshop/upgrade shop items,
  make them functional, and test each one to see if they work functioanlly
  properly and visually pleasingly. you may organize shoplayout in your own
  way, but keep it easy for people to be able to understand right away
  without too much text ... you may also need to make the sword animation
  for the swords rather than using punch swing animation for it";
  `PROTOCOL_VERSION` 13, the server deployed with it, the binary before
  kept on the VM as `roundtown-server.prev-2026-11-12b`; APK 20). The file
  was saved twice more while this was being built, last at 02:58 that
  night, and the game is as that one has it: **read the file again before
  changing a number**, Hajun edits it as they go. The rules are all in
  `roundtown_net::bridge_wars`, as the generators' are: `Outfits`, what
  everyone has and what each team has bought, beside the `Mint`, the
  server's online and this device's own in a game played offline. What
  there is, and where:
  - **The rates**: an amethyst every 60 s, 40 from 4:00 and 25 from 8:00
    (`interval`; 40, 30 and 20 until then), so at 60, 120, 180 and 240 s,
    every 40 s to 480, and every 25 after.
  - **What everyone has** (`roundtown_net::Kit`: the sword and the shield
    by tier, what is on the second button and how many, until when a
    potion lasts, and whether a trap has them blinded; `Upgrades`, a
    team's: its generator's step, stronger swords, better shields' step,
    the heal pool, the traps waiting). Online both are on the game's own
    entity beside the `Purses` (`Kits`, `Upgraded`), sent to the room as
    they change, so every device knows what everyone carries; a purchase
    is `Ask::Buy(Ware)`, answered `Tell::Bought(Ware)`, and using what is
    on the button `Ask::Use(Use)`.
  - **The item shop, at the red stand** (`Shop::Items`; the prices and the
    numbers are the file's): Bridge, 25 m for 4 silver; the Basic, the
    Silver and the Aura Sword, 10 silver, 7 gold and 4 amethyst, which add
    5, 10 and 20 to a punch's 12 (`SWORD_DAMAGE`); the Basic, the Advanced
    and the Aura Shield, 24 silver, 12 gold and 6 amethyst, which keep 6%,
    14% and 22% of every blow off (`SHIELD_SHARE`); and five things for
    the second button, one kind at a time ("you can only buy if the 2nd ui
    button is either empty or its filled by the same item u already
    have"): Fireball, 40 silver, three on the button at the most ("cap at
    3 stack"); Bridge Builder, 1 amethyst; Speed Potion, 1 amethyst;
    Invisibility Potion, 2 amethyst; Healing Potion, 3 gold. Only a better
    sword or shield than the one you have is sold. **Dying loses the
    sword, what is on the second button, a potion still lasting and the
    metres of bridge; the shield stays** ("sword is lost on death, shield
    carries on", `Outfits::die`, `arena::lose_on_death`); and, since
    later that day, everything you hold of the four things as well
    (above).
  - **The upgrade shop, at the blue stand** (`Shop::Upgrades`, your whole
    team's, in sapphire): the generator, 2, 4, 6 and 8, half as fast again,
    twice, amethyst too, and three times as fast (`FORGE_SPEED`, "150%",
    "200%", "starts spawning amethyst too", "300%": the team's own silver
    and gold; the amethyst comes on the team's own pad); Stronger Swords,
    4, once, 5 more on every sword of the team; Better Shields, 2, 3, 4
    and 6, 6% more kept off each time; Heal Pool, 1, 5 health every 2.5 s
    to whoever of the team stands on its island; Blindness Trap, 1, 2 and
    4 for up to three waiting.
  - **The windows** (`src/market.rs`): standing in the outline at a stand
    opens that stand's shop, the red-roofed one's (`RoofTiles_Red`) the
    item shop and the blue one's (the material `bluemarket`) the upgrade
    shop, as the map has them (`market::Shops`, `shop_zones`); the server
    sells within 5 m of where they stood when it was built (`ARENA_SHOPS`,
    `ARENA_UPGRADE_SHOPS`, `at_a_shop`; a test keeps the two together). A
    window is a card to each thing: its picture, drawn in code
    (`hud::Picture`) and tinted as the thing is, a name of a word or two,
    one short line ("+5 dmg", "-14% dmg", "+40% 45s", "Blinds 8s", each
    number filled in from the rules), a pip for each step of an upgrade
    that goes by steps, and its price, the picture of what it is paid in
    and how many; or, in the price's place, "Owned", "Max" or "Slot full".
    What cannot be had is blocked out; a tap on a card buys it at once. The
    item shop is three rows, the bridge and the swords, the shields, and
    the five things for the button; the upgrade shop one row of five taller
    cards. The window is put up again as what you hold or have changes. It
    is still not a dialog, and covers the middle of the screen, you with
    it.
  - **The second button** (`src/utility.rs`, `hud::utility_slot`, the empty
    slot up and to the left of the sword until then): the picture of what
    is on it, how many in its corner, and the seconds a potion still lasts
    over it ("45s"). **What is aimed**, the fireball and the builder, is
    held and dragged ("rather than a push button, its a hold & drag / letgo
    to shoot or cancel, and shows a long narrow rectangular hit range
    indicator similar to Brock in brawlstars"): once the finger is 4% of
    the short side from where it went down a band lies on the ground from
    you that way (`range::band`, in `colours.arena.aim_outline` and
    `aim_fill`, and since later that day with a dark rim round it,
    `aim_rim`, which is what shows it over the void, above), as long as a
    fireball flies and a metre wide, or as wide
    as a bridge; let go and it goes that way, and you turn to face it;
    dragged back to where it went down the band is gone, and letting go
    there cancels. A quick tap that never dragged, under 0.3 s, uses it
    the way you face (mine, as Brawl Stars fires at a tap), and so does Q
    on desktop. Up on the screen is north, the camera being fixed over you.
    Half a second has to pass between two. **What is drunk**, all three
    potions (the healing one too in the file's last version: "after
    drinking time 1.6s, speed reduct, same as speed potion"), is a press,
    or Q: 1.6 s (`DRINK_SECS`) with the time left written in front of you
    in tenths (`utility::show_drink`), at half your pace, with no punch,
    your right hand at your mouth and your sword put away
    (`crate::Drinking`); a press again, any hit that takes health off you,
    or dying stops it with nothing drunk; run out, the potion is drunk
    (`arena::Used::Potion`). The punch's range is put away meanwhile, as
    while you build.
  - **The fireball** (`src/fire.rs`, `bridge_wars::Flight`): a red glowing
    ball thrown from in front of your chest, flying level and straight at
    22 m/s for 12 m (`FIREBALL_SPEED`, `FIREBALL_RANGE`), through whatever
    stands on the map; where it meets anyone of another team, or at the end
    of its range, it bursts, and everyone of another team within 3 m takes
    20, less what their shield keeps off, and is knocked away from where
    it burst as a punch knocks. The blast is a ball of fire that swells to
    as far as it reaches and fades in a third of a second. Online each is
    an entity the server sends as it is thrown (`Fireball`), which every
    device flies on by itself, and takes away as it bursts. The range was
    20 m for its first hours, which bursts off the screen: the camera over
    you shows 11 m up and down.
  - **The Bridge Builder** (`src/bridge/builder_bot.rs`; "it spawns a bot
    character that walks in bridge mode toward that direction, creating a
    bridge in a straight line for 100m or until it reaches a island/land
    and then the bot pops/disappears"): a body in your own look carrying a
    plank, sent off from where you stand at the pace anyone builds at, 5
    m/s. It walks straight over the ground, through whatever stands there,
    to where the ground ends; there its bridge begins, a chain like any of
    yours (`Chain`, its root on the very edge and cut along it, heading the
    way it walked or as near that as a first plank may slant, 45 degrees),
    laid a plank or two ahead of its feet; ground within a plank's reach
    ahead, land or anyone's bridge, the bridge lands on it as yours do
    (`land_on`); out of its own 100 m (`BUILDER_METRES`, not yours) it
    ends where it is; and at the end of its bridge the builder shrinks to
    nothing. Off the side of a bridge it forks. Walking 170 m without the
    ground ending it gives up, and it is sent only from the ground. It is
    only there to be seen (`lobby::puppet`, `crate::ForShow`: in nobody's
    way, no punch lands on it) and is this device's alone, as the plank in
    your arms is; online the server takes the builder off your button and
    its bridge goes up a rung at a time (`Ask::Rung`), so everyone else
    sees the bridge grow and not the builder.
  - **The potions**: speed, 45 s at 1.4 times your pace (measured in the
    desktop build: 10.0 m/s before, 5.0 drinking, 14.0 after); the healing
    one, a quarter of your health back as it is drunk, with a green "+25"
    floating off you (half of it, and "+50", since 2026-11-13, as the
    file has it since); and invisibility, 30 s in which nobody of another
    team draws you, your name or your bar (`island::show_where_i_am`,
    `arena::place_tags`), and no bot goes for you (`arena::hunt`, the
    server's `think`), while you and your own team see you see-through
    (`gear::ghost`). **Being hit, or hitting, ends it** ("being hit or
    attacking reveals u", `Outfits::reveal`): a blow that takes health off
    you, or a punch of yours that lands on a body or an anchor, which is
    all of a punch the server hears of. While a potion lasts with none
    left, the button keeps its picture, faint, and its time, and takes
    nothing else; then it is empty.
  - **Swords and shields are seen on everyone** (`src/gear.rs`, by
    `arena::Wears`, which every body in a game carries from the game's
    kits). The sword is Hajun's model for its tier, `basicsword.glb`,
    `silver_sword.glb` and `aura_sword.glb` (theirs of 2026-11-12, each a
    blade up its own +y from a grip at its origin, 0.9 to 1.1 m; a test
    keeps them in the APK), a child of the body put in its right hand every
    frame once the limbs are posed (the rig's `hand.R`, followed up to the
    body), its flat side up. **It is drawn bigger than modelled**, 1.7
    times and twice across (`gear::SWORD_SCALE`): from the camera 15 m over
    you a metre of thin blade was a hair. (Since later that day a sword
    is held on its edge and cut downward instead, above, and what follows
    is the first way, `arena.sword_swing: Sweep`, kept to go back to.)
    **With a sword the punch is a
    slash** (`sword_offset`, `blade_way` in `src/lib.rs`): the right arm
    holds it at the ready, a little forward of hanging, the blade out in
    front, and at a press it is raised to level and swept about the body's
    upright from the right of straight ahead across to the left in the
    time a punch goes out, and back to the ready in the time it comes
    back; and the slash leaves a trail (`gear::Trail`), the arc the blade
    has swept, in the sword's colour, growing behind it and gone a sixth of
    a second after. What a punch lands on, and when, is as it was: the
    range drawn round you, at the press. A slash, like a punch, is this
    device's alone: someone else's sword is seen at the ready and their
    swing not at all. The shield is a ball of force round the body ("a
    very transparent (slightly semitransparent) spherical forcefield"),
    1.05 m out from its middle, in its tier's colour and as see-through as
    that colour's alpha (`colours.arena.shield_*`).
  - **The team's upgrades, seen**: the heal pool's 5 floats off whoever it
    heals as a green "+5" (`arena::spawn_healed`; online any jump up in
    health of 4 or more is taken for a heal). **A trap** springs on an
    enemy stepping onto your island: it is used up, they are blinded for 8
    s wherever they go (`BLIND_SECS`; the file's first version had it until
    they had been off the island 5 s), your team is told "An enemy walked
    into your trap!" and they "A trap has blinded you!", in the line the
    timetable's are said in. **Blinded** (`gear::blindfold`; "they can only
    see a very small circle around their character and the rest of their
    screen is black except their ui menu/buttons/minimap"), the screen is
    black all but a circle 2.6 m round you (`BLIND_SIGHT`): one UI node
    with a round hole kept over you, under the buttons, the minimap and
    the rest of the screen's own, over the world and the names and numbers
    written over it. A blinded bot sees no further than that.
  - **Who is in a game online, until the evening of 2026-11-12** (every
    game is 2v2v2v2 since, above) (Hajun, the same night: "when ur the only
    real person in the queue for the bridge wars, it can be same as now
    where 7 bots join u. but when there is atleast one other real person in
    that same queue lobby, no bots will join, and after the wait time of no
    more new joins, it will start the game as a 1v1 (2 side island being
    empty), or 1v1v1 (with one island being empty) or 1v1v1v1, (or 2v2v2v2
    with each bot teammate for missing slots if >=5 players)"): people are
    seated in the order 0, 2, 1, 3, 4, 6, 5, 7 (`SEAT_ORDER` in
    `server/src/arena.rs`), so that two are on opposite islands and the
    first four each on their own; ten seconds after the last to join, one
    person alone, or five or more, have the seats left filled with bots as
    before, and two to four begin as they are, the seats nobody took kept
    empty (a `Member` with no id, out from the start), so that a zombie's
    seat is still past the players' eight. Offline it is you and seven
    bots, as it was.
  - **`assets/ui.ron`** has the words (`words.market`: the two titles,
    "Owned", "Max", "Slot full", and each thing's name and line, in which
    a `{number}` comes out of the rules; `words.arena.blinded` and
    `trap_sprung`) and the colours (`colours.arena.sword_*`, `shield_*`,
    `fireball`, `potion_*`, `heal_pool`, `trap`, `pip`, `pip_off`,
    `aim_outline`, `aim_fill`, `aim_rim`, `drink`, `blind`). No number of
    the rules is in it; `arena.sword_swing` says which way a sword is
    swung.
  - **For trying it out**: `RT_ARENA_PURSE=<count>` begins a fight with
    that many of each thing in every person's purse, on the server for its
    games and on a desktop build for one played offline, as
    `RT_ARENA_FROM` begins it late.
  - **What is mine, the file not saying**: (what you hold of the four
    things was kept through a death, until Hajun had it lost, later that
    day;) Stronger Swords and Better Shields do nothing for a bare hand or
    someone with no shield; the generator's third step keeps the second's
    pace, and its amethyst is one a minute, two on the pad at the most;
    nine of anything but a fireball on the button; the fireball's speed,
    range, size and blast, that it harms neither its thrower's team nor an
    anchor, and that it flies through walls (the server has no map); the
    quick tap; that a trap springs only on stepping onto the island, and
    not again on the same enemy until they have stepped off and on, and
    never on a zombie; that your own team sees you while you are unseen;
    the builder's pace, and that only its owner sees it; and how big the
    swords are drawn, how they are held and swung, and the trail.
  - **Checked on 2026-11-12, scripted, in the desktop build** (a throwaway
    `src/arena/scratch_run.rs`, removed, with touches written in for the
    cards and the second button). Offline, each thing: both windows, every
    card bought by a tap, the prices off the counts, a fourth fireball and
    a second kind refused, the cards blocked out with nothing to pay; each
    sword from above and close, at the ready, slashing and walking, and
    each shield; a punch with the Aura Sword and Stronger Swords taking 37
    off a bot, and a bot's 12 taking 9.4 through the Aura Shield; the
    fireball aimed, dragged back and cancelled, thrown, flying, bursting,
    20 off a bot, and thrown by a quick tap; a builder bridging the 44 m
    from the island to the town and landing, a second forking off that
    bridge and reaching a sapphire island 83 m on, and standing on its
    planks; a drink stopped by a second press and by a hit, the paces
    above, a bot six metres from someone unseen leaving them alone, a punch
    ending it, the healing taking 28 to 53; the pool's 5 every 2.5 s; a bot
    stepping into a trap, and being blinded for 8 s on an enemy's island.
    Online, a server on this PC and two copies: two people begun as a 1v1
    with no bots; everything bought from the server at both stands; the
    second copy seeing the first's sword and shield, walking into its trap
    and being blinded, not seeing it while it was unseen and seeing it
    again as it punched (22 off), a fireball flying in and taking 20, and
    the builder's bridge growing on its minimap. Unit-tested: the prices,
    the tiers, the caps, the potions, the traps and the pools; the
    builder's bridge to a far shore, its hundred metres with none, a slant
    and a fork; the poses on the model; the band, the trail, the drag's
    aim. The VM, deployed, let this PC's build in. **Not by hand, and not
    on a phone: none was attached, so APK 20 is built, its assets checked
    byte for byte, and not installed.** The Adreno has not drawn the
    swords, the blended bubble and ghost, the trail, the bands or the
    blindfold.
- **Since the night of 2026-11-11 Bridge Wars has generators, a shop and a
  timetable** (Hajun: "i want to now make the generators functional in the
  pvpmap, using the 4 item models in the asset folder as the items being
  generated - they should drop in semi random location on top of the stone
  pad on each island ... the items should bob up and down and rotate
  slowly, and when the user can collect it by walking in it (short magnet)
  ... make it so that the red shop if the user goes inside the activation,
  it opens a closable popup shop menu, and right now just have 'bridge' as
  the option, which should cost 4 silver for 50m of bridge ... read the
  bridgewarsgameevent.txt for events and generator rates. it should show a
  small alert in around middle of the upper half of screen that tells the
  event ... these generators/items in map need to be server synced so
  other players see the same number of items on map at the same times and
  other player taking the item from the generators make it so you cant
  take it too ... if there is any other info ur not sure try to think for
  yourself what would be ideal, cuz this is a game similar to minecraft
  bedwars"; `PROTOCOL_VERSION` 11, the server deployed with it, the binary
  before kept on the VM as `roundtown-server.prev-2026-11-11b`; APK 18).
  Hajun's file, `bridgewarsgameevent.txt` at the root of the repo, is the
  timetable and the rates. Everything it says is in
  `roundtown_net::bridge_wars` (`shared/src/lib.rs`), which the server
  plays by online and the device by itself offline, and none of it in
  `assets/ui.ron`, which has the colours and the words. What there is, and
  where:
  - **The generators** (`bridge_wars::Mint`, `GENERATORS`, `generator`,
    `interval`, `cap`): sixteen, a silver one and a gold one on the pad at
    the back of each team's island (`ARENA_PADS`, the map's `gen1` to
    `gen4`), a sapphire one on each of the four smaller islands on the
    diagonals (`SAPPHIRE_PADS`, `sapphiregen` to `sapphiregen4`) and four
    amethyst ones in the middle of the map, 35.8 m out on the axes
    (`AMETHYST_PADS`, `amethystgen` to `.003`), as the file has them.
    Silver one every 2 s, half a second more or less, 48 to a pad at the
    most; gold every 7.5 s, a second more or less, 6; sapphire every 30 s,
    23 from 2:00 and 12 from 6:00, 4; amethyst every 40 s, 30 from 4:00 and
    20 from 8:00, 2 (every 60, 40 and 25 since 2026-11-12, above). A
    generator's time comes round whether or not it is
    full: holding its most, it makes nothing then, and its count starts
    over (mine: the file says only "max held on generator"; until late
    that night a full one's count stood still, and its next thing was a
    whole wait on from when one was taken, which a count written over it
    could not have shown). The sapphire and the amethyst ones keep time
    to the second, by the time alone (`bridge_wars::steady`, `next_at`: a
    sapphire at 30, 60, 90 and 120 s, then every 23 s to 350, then at 372,
    the third tier having come at 360 and cut the wait in hand to its own
    12 s, and every 12 s after; an amethyst every 40 s to 240, every 30 to
    480, every 20 after), which is what lets every device write the same
    count over them with nothing sent ("The sapphire and the amethyst
    generators say", below). Each thing comes up at the best of six
    places tried at random within 0.7 m of the pad's middle each way, the
    one furthest from everything lying there (`PAD_SPREAD`: my reading of
    "semi random location"). The pads' places are constants, the server
    having no map; a test reads them off the map, by what Hajun calls the
    objects (`the_generators_pads_are_where_the_map_has_them`).
  - **Picking up** (`MAGNET`, 1.5 m across the ground from your middle to
    the thing's: Hajun's "short magnet"; `Mint::pick_up`), **wherever the
    thing is, dropping or lying, once it is within your reach** (since
    2026-11-12; Hajun: "the delay is a bit too much for the delay between
    the item dropping from generator and the user collecting - user
    should be able to collect in the air too maybe when they jump"): no
    further over your feet than `REACH_OVER`, 2.14 m, half a metre over
    your head, as Minecraft picks a thing up half a block over a
    player's, nor under them than `REACH_UNDER`, 1.5 m, so that a jump
    across a pad still takes what lies on it and the waiting room and a
    fall past the island do not. A thing is where its drop has it
    (`Piece::height`, `drop_height`: the rules' now, with `HOVER`,
    `DROP_FROM` and `DROP_SECS`, as whoever runs the generators goes by
    them): standing on its pad it is yours 0.45 s after it is made, still
    dropping, and jumping as it appears, 0.33 s after, 2 m up; the
    numbers but Minecraft's half metre are mine. For the evening before
    it could not be had until it had landed and lain 0.3 s, 0.85 s after
    it was made (`PICK_UP_AFTER`, gone), which Hajun found too long.
    Measured in the desktop build, scripted, standing pinned on a pad
    (0.449 to 0.451 s) and jumping as each thing was made (0.324 to
    0.329 s, the feet 2.05 m up); not online, where it is the server's
    and unit-tested, nor by hand. Online the server gives a thing to
    whoever of the people in the game comes within reach of it, where
    they last said they stood, the lower seat first within a tick, and it
    is gone for everyone that tick (`server/src/arena.rs`, `mint`,
    `Game::gather`); offline the game does the same for you
    (`arena::gather`). The computer's players picked up nothing (mine: a
    teammate that emptied the pad would be no help, and it bought
    nothing) until 2026-11-12, since when they take what their purchases
    need, people first ("the computer's players play the game", above).
    Online each thing is an entity the server sends to the game's room
    (`Loot`: what it is and where it lies), and what everyone holds is one
    component on the game's own entity (`Purses`, four counts a seat).
  - **What you see** (`src/loot.rs`): Hajun's four models,
    `silver_item.glb`, `gold_item.glb`, `sapphire_item.glb` and
    `amethyst_item.glb`, cut stones of three, four, five and six sides,
    0.7 m across, each hung 0.42 m over its pad's top, bobbing 0.1 m
    either way every 2.4 s and turning once in seven, out of step with
    the one beside it, and taken, flying to whoever took it in 0.16 s,
    shrinking (online, to the nearest person to it, the server not saying
    who). **Made, it drops onto its pad** (since late that night; Hajun:
    "the minerals spawning animation is like a growing big animation. i
    want it to be a drop from a certain height animation. so that even
    when player is standing on top of the generator, u can see the mineral
    for a bit before the player pick it up"): it appears 5 m over where it
    will hang (`loot::DROP_FROM`: over the head of anyone standing on the
    pad, and well under the camera, so drawn nearly half as big again
    as it lies), and falls there in 0.55 s, faster as it comes
    (`loot::height`), so that someone standing on a pad sees each thing
    come down before it flies to them from where it is, as it comes
    within their reach ("Picking up", above; for its first evening it
    had to land and lie 0.3 s first); until then
    it came up from nothing to its size in a fifth of a second where it
    lay, and was theirs the frame after. Online the drop begins as the
    server's entity for it arrives, and who has it is the server's. Seen
    in a burst of shots a tenth of a second apart, offline and against a
    server on this PC, standing on the pad. Only those within
    48 m of the camera are moved: a full map has 240. At night they glow
    in their own colours, 0.9 times, as far as the lamps are on: a
    sapphire is nearly black by moonlight, and the smaller islands have
    no lamps. All my numbers.
  - **What you hold is counted top left**, under your health bar, where
    the log, the ingot, the gem and the emerald were: silver, gold,
    sapphire and amethyst, each pictured as its model is seen from above,
    a stone of three, four, five or six sides with its corners cut off as
    the model's are and the outline of its table cut through
    (`hud::item_picture`; Hajun: "change the icon to match the model
    shape"), tinted its own colour (`colours.arena.item_silver`,
    `item_gold`, `item_sapphire`, `item_amethyst` in `assets/ui.ron`).
    The metres of bridge are no longer there ("remove the log icon - the
    bridge meter only shows as in map text in front of the bridge like
    now"): they are written in front of you as you build, as before, in
    the shop, and, since 2026-11-12, inside the bridge button, under its
    plank, small and white, building or not (`hud::bridge_button`,
    `BridgeCount`, written by `show_resources`; 2.8 shares of the short
    side, in `colours.arena.bridge_count`; Hajun: "above the ui bridge
    button add a small white text "123m"", and a few minutes later,
    "instead of above the button, put it inside the button, below the
    icon": the plank is moved up 13% of the button to make room). Seen in
    the desktop build with 123 m written in.
  - **The shop** (`src/market.rs`): standing in the white outline on the
    ground in front of the red-roofed stand on an island, anyone's (the
    map's `activation` material, at a stand roofed in `RoofTiles_Red`;
    read off the map once it has spawned, `market::shop_zones`), opens a
    small window over the middle of the screen: "Shop", a card with a
    plank, "Bridge +25m" (50 until late that night; Hajun: "make it so 4
    silver gets you 25m of bridge", `bridge_wars::BRIDGE_A_BUY`) and its
    price, a silver and "4", the line "You have 0m of bridge", and a cross
    on its corner; all of it white on grey since late that night ("The
    panels are grey", below: its title was gold, its cross red and a
    price you could not pay red). It is not a dialog:
    you can walk, and the fight goes on round it. A tap on the card buys
    one at once, as often as you can pay: offline here, online asked of
    the server (`Ask::Buy`), which takes the 4 silver if you hold it, are
    standing, and are within 5 m of a shop where you last said you stood
    (`ARENA_SHOPS`, `SHOP_SLACK`; a test keeps those where the map's red
    stands are), and says so (`Tell::Bought`), on which the 25 m are
    yours. Without 4 silver the card is blocked out, its price with it.
    The cross shuts it, and so does a press anywhere else on the screen,
    which is not taken, so that touching the stick walks you away, and
    Escape; shut by hand it stays shut until you have stepped out of the
    outline and in again; stepping out of it, dying and the game's end
    shut it too. The blue stands' outlines were nothing until 2026-11-12,
    when they became the upgrade shop and this one the item shop, with a
    card for everything it sells ("two shops", above); its words were
    `words.arena.shop`, `shop_bridge` and `shop_have`, and are
    `words.market` since.
  - **Everyone starts with no bridge** (`arena.bridge_metres` in
    `assets/ui.ron`, 0 where it was 500 since 2026-11-10): my call, Hajun
    not having said, as a game of Bedwars begins with no blocks. Four
    silver, eight seconds of a pad, is 25 m, and an island is 44 m from
    the middle of the map: two buys. It is that one number to change
    back. Online
    the metres are still the device's own; it is the silver the server
    keeps.
  - **The timetable** (`bridge_wars::TIMETABLE`, `PLAY_SECS`): a game
    lasts 25 minutes where it lasted 10. At 2:00 and 6:00 the sapphire
    generators quicken, at 4:00 and 8:00 the amethyst ones; at 10:00
    every anchor left breaks, so that whoever dies after is out; at
    15:00, sudden death, the zombies rise; and at 25:00 it is a draw
    ("Time's up!" over "The game is a draw."). Each is said as it happens
    in one line across the screen a quarter of the way down, small (3.6
    shares of the short side), white, up five seconds (`arena::alert`;
    `words.arena.tier_up`, `anchors_broken` and `sudden_death`, and
    `colours.arena.alert`: "Sapphire generators have been upgraded to Lvl
    II."), over the shop's window if that is up; and since late that
    night a line about an item has that item's picture right after its
    name, the one its count has top left, in its colour (`arena::Line`,
    which cuts the file's line after `{item}`, and `AlertPicture`; Hajun:
    "add the item icon after that item name in that game event text"),
    coming and going with the words. The four headlines four
    anchors breaking at once would put on top of each other are left out
    for the one line.
  - **The sapphire and the amethyst generators say their level and count
    down to their next thing** (`arena::write_over_generators`, since
    late that night; Hajun: "for the sapphire and the amethyst
    generators, put a white text above it that shows the generator level
    like Amethyst Generator Lvl 2, and below it, put 'Next Amethyst in
    24sec...' that counts down. text should be small enough so it doesnt
    cover the amethysts or teh generator but large enough to be
    readable"): two lines of white words, 3.0 shares of the short side
    (`GENERATOR_TEXT`, mine: between the names over heads, 2.4, and the
    alert's line, 3.6; about 2 mm letters on the phone), one ending 0.55
    m north of each of the eight pads and one beginning 0.55 m south of
    it (`GENERATOR_GAP`), over and under it on the screen under the
    camera fixed over you, centred on it, shown while the pad is on the
    screen. The words are `words.arena.generator` ("{item} Generator Lvl
    {level}", the level as 1, 2 or 3 where the alert writes I, II, III,
    as Hajun wrote each) and `generator_next` ("Next {item} in
    {seconds}sec..."), the colour `colours.arena.generator`. **Nothing
    is sent for them**: the level is the timetable's and the count is
    `bridge_wars::next_at`, both by the time, which online is the
    server's to the second (`ArenaRound::left`, rounded up, so the count
    reads 1 through the last second and the whole wait the moment a
    thing comes, never 0). It counts on while the generator is full, at
    the end of which nothing comes and it starts over: mine. Seen in the
    desktop build, scripted, at a sapphire pad and an
    amethyst pad, and across 2:00, "Lvl 1" and "2sec" to "Lvl 2" and
    "23sec" as the sapphire dropped and the line came up with its
    picture.
  - **The camera stands at one height in a game and nothing zooms it**
    (`OrbitCamera::top_down`, `locked`; since late that night, Hajun:
    "make the camera in that mode's default zoom to be zoom in a bit more
    so the borders of each players island just fits around perfectly on
    the camera on my mobile phone, and lock that zoom so u cannot zoom in
    or out"): `arena.camera_height` in `assets/ui.ron`, 15.5 m over you
    (as `car.overhead` is, over your chest: 16.7 m over your feet), where
    an island, 20.8 m across, takes up 96% of the screen's height with
    you in its middle, its edge just inside the top and the bottom. The
    camera sees 66 degrees top to bottom on every screen (`CAMERA_FOV`),
    so that is so on any screen wider than it is tall, the phone on its
    side among them, with the sides to spare; worked out, measured in a
    1600 by 900 window (the island from 18 to 882 of 900 pixels), and
    not seen on the phone. A pinch and the wheel do nothing there
    (`zoom_by_ratio`); the file saved again moves the camera to its new
    number (`drive::retune`, `OrbitCamera::relock`); out of the game the
    camera is as you had it. Until then it stood the car's 24 m up,
    where the island was 64% of the height, and zoomed between 7 and 60.
  - **The panels are grey** (since late that night; Hajun: "the shop
    panel is too colorful. make it all shades of white/black/gray - the
    text, the red close button, the background, etc - no blue tint. same
    for the timer background and minimap background, and also the
    roundtownmap setting panel. i dont want any blue tint. and try to
    reuse the exact grey color if possible"). The grey is your name's
    plate's and the buttons', `#666666`: the glass behind the time
    (`colours.panel`, `panel_lit`, House Builder's too) and the minimap's
    background (`arena.map_back`) are the plate's very colour, alpha and
    all (`#66666673`), so the three across the top of the screen are one;
    the shop's window and the settings' are that grey as solid as each
    was (`shop.window` `#666666db`, House Builder's shop with it, and
    `settings.window` `#666666d1`); the cross on a window's corner is
    that grey, solid, where it was red (`close`, every window's); and
    the shadow under every word is black where it was a dark blue
    (`text_shadow`). They were a navy (`#0a1a2e`, `#102033`) and a
    bluish grey (`#5c5e66`). In the shop the title is white where it was
    gold, a price you cannot pay is greyed where it was red, and the
    silver's picture is a plain light grey (`item_silver`, which had a
    little violet in it). **The minimap's picture is clear round the
    land** now, its background the node's alone: it was filled with
    `map_back` as well, so a see-through one was laid on twice and came
    out more solid than the file said. A grey as see-through as the
    plate's takes the colour of what is behind it, the sky's blue by day
    over the void: the plate always has, and the time and the minimap do
    now as it does; each is one value in the file, its alpha how solid.
    **Left as they were, being neither blue nor asked about**: the
    settings' chosen tab, gold, and its Delete account, red; the
    headlines ("Fight!"), gold; the items' own colours beside their
    counts. Seen in the desktop build, scripted: the board and the
    minimap over the sky and over the town's paving, the shop's window,
    and the settings' window in the town.
  - **`PROTOCOL_VERSION` 12, the server deployed with it** (the binary
    before kept on the VM as `roundtown-server.prev-2026-11-11c`): a
    bridge bought is 25 m where an app from before would count 50. The
    drop's wait and the generators' counts are the server's online. This
    PC's build was let in by the VM after. APK 19 is built, its assets
    checked byte for byte, and **not installed: no phone was attached**;
    none of this has been seen on the phone, the camera's fit least of
    all.
  - **On 2026-11-12, after APK 19** (Hajun: "dont rebuild apk yet"): a
    thing is picked up as it drops, the metres of bridge are written
    inside the bridge button, and the minimap marks the sapphire and
    amethyst generators (each under its own heading above). The server
    was deployed with the new reach (the binary before kept on the VM as
    `roundtown-server.prev-2026-11-12`); `PROTOCOL_VERSION` is still 12,
    nothing sent having changed, so APK 19 plays online and has the new
    pick-up there, the server deciding it, and keeps the old 0.85 s wait
    in a game played offline, and has neither the number in the button
    nor the marks, until an APK is built again. Not committed either.
    Hajun's `bridgewarsgameevent.txt` had by then been changed by hand
    (amethyst every 60, 50 and 40 s where the game has 40, 30 and 20; an
    item shop and an upgrade shop begun): nothing of that was in the game,
    nobody having asked, until Hajun did, later that night ("two shops",
    above).
  - **The zombies are all mine but that they come**: the file says
    "Sudden death: zombies spawn all over the map", and no more. Twelve
    (`ZOMBIES`): one on each team's island, 4 m to the blue stand's side
    of its middle, one on each smaller island, 4 m from its pad, and four
    on the path round the pond (`zombie_spot`). Each is the body everyone
    has with its skin and its hair green (`arena::paint_zombies`; the look
    `zombie`), called "Zombie", on a team of its own (`ZOMBIE_TEAM`), so
    that every player and every bot is its to go for, and it theirs: with
    everyone's 100 of health and 12 a punch, a punch every 0.8 s. It
    comes on at 6 m/s where everyone runs at 10, never runs away, takes
    three tenths of a punch's knock (`ZOMBIE_KNOCK`: knocked as anyone
    is, the server's bots had every island's zombie over the edge within
    three seconds of its rising), and 12 s after it is put down another
    stands where it rose. Online they are the server's, seated past the
    players' eight seats (`arena_team` gives a seat from 8 up the
    zombies' team; `Game::zombie`, `seat_zombie`), each kept to a disc as
    the server's bots are (`zombie_home`: its island, or the middle of
    the map out to 74 m, where it walks through the lamps, the planters
    and the pond's rim, the server having no map); offline this device's
    (`arena::raise_zombie`, `Zombie`), bodies like the bots', which walk
    the map as it is. A zombie's punch knocks a player as any punch
    thrown standing does: four of them had me off my island in a scripted
    run. With every anchor gone by then, that is out.
  - **For trying it out**: `RT_ARENA_FROM=<seconds>` begins a fight that
    far into its timetable, with everything before then already done: on
    the server for its games, and on a desktop build for a game played
    offline. 595 is five seconds before the anchors break, 895 before the
    zombies.
  - **Checked on 2026-11-11, scripted, in the desktop build.** Offline:
    7 silver and a gold picked up off the pad 13 s in, the counts top
    left; the shop opened in the outline, two taps on its card taking 8
    silver for 100 m, a tap elsewhere shutting it, stepping out and in
    opening it again, "100m" in front of you building; the line at 2:00;
    the anchors gone at 10:00 under one line; twelve zombies at 15:00,
    one of them punched to death and back up. Online with a server on
    this PC: every thing the server had was on the screen, to the count,
    at every look; a purchase taken by the server; and with two copies in
    one game, both stepping onto one pad with ten things on it at the
    same moment, the server gave all ten to one and none to the other,
    both screens showed the pad empty at once, and each copy's counts
    were the server's; a punch of yours takes 12 off a server's zombie,
    and its punches knock you. The VM, deployed, let the new build in.
    **Not by hand, and not on a phone: none was attached, so APK 18 is
    built and not installed.** The item models, their glow, the shop's
    window and the alert's line have not been seen on the Adreno.
- **Since the night of 2026-11-08 Bridge Wars is the server's online, eight
  to a game in four teams of two, with respawn anchors and bridges you
  build** (Hajun: "the bridge battle minigame doesnt allow queing with
  other players i think even tho we join the game at the same time. fix
  that. also make the purple rotated cube thingy on each island the respawn
  anchor (similar to bed in bedwars) ... make it 8 player rather than 4
  player, so its 2v2v2v2 ... make it so u cannot hurt your own teammate ...
  display the ai bot/other players name above the healthbar, and make the
  healthbar and the nametag not distracting ... some indication wether they
  are teammate or enemy ... add a 'autobridge' feature"; `PROTOCOL_VERSION`
  10, the server deployed with it, the binary before kept on the VM as
  `roundtown-server.prev-2026-11-08b`). What changed, and where:
  - **Online a game is a room of the server's** (`server/src/arena.rs`,
    `Room::Arena`, as House Builder's games are): choosing Bridge Wars asks
    `Ask::Arena`, the server seats you in the game filling now
    (`Tell::Seated`), puts you in the waiting room at your seat's spot
    (`roundtown_net::bridge_wars::waiting_spot`, the room's floor, 150.5 m
    up since the map of 2026-11-11 and 100.5 until then, as a shared
    constant, which a test checks the map agrees with), fills
    the seats left with its own bots after 10 s (since 2026-11-12 only for
    one person alone or five or more, "Who is in a game online", above),
    counts down, sends everyone
    to their islands, and runs the clock; the device follows its
    `ArenaRound` and `Seats` (`arena::follow_round`): the stage and the
    time, everyone's health in whole points, each team's anchor and who is
    out. Your moves go to the server in the arena now (`net::tell_where_i_am`
    no longer leaves it out), and its arrivals move you (`net::hear`).
    **Health is the server's online**: a punch that lands sends `Ask::Punch`
    as before, and the server takes 12 off whoever it landed on, if they are
    not on your team and within 6.5 m of where you each last said you stood,
    knocks a person through `Tell::Knocked` and a bot itself; a fall into
    the void is `Ask::Respawn`, which the server answers with a death. **A
    body punched is past hurting for half a second on the server too**
    (`HURT_TIME`, `Game::wound`, and an anchor the same, `anchor_hurt`;
    since 2026-11-09, Hajun: the bots "seem to get knocked back too much
    by me maybe cuz i can hit them multiple times before they process it
    networkly": for a night the server took every punch that landed, as
    many as the presses came, where the device had kept to two knocks a
    second since 2026-11-07, "The punch knocks whoever it lands on back"). The
    rules both sides play by are `roundtown_net::bridge_wars` (health 100,
    damage 12, anchor 120, hurt 0.5 s, respawn 4 s, regen 4 a second after
    5 s, run 10, sight 24, a bot's punch every 0.8 s, flee under 20%, fall
    30 m, the end's 6 s); `assets/ui.ron`'s `arena` has the same numbers
    for offline, with `anchor_health` and `bridge_metres` added (and, for
    the number written in front of you as you build, `bridge_low_at` and
    `bridge_red_at`), and rules
    offline alone. **A game ends over two ticks** (`close`, then `end`, in
    `server/src/arena.rs` and `server/src/builder.rs` alike, since
    2026-11-09): its bots, planks or pieces and board are despawned first,
    while everyone in it is still in its room to be told so, and everyone
    is moved back to the town the tick after. Done in one tick, as for a
    night, the devices were never told: replicon 0.44 writes a despawn
    only to a device the entity is not hidden from by then, and a loss of
    sight only for an entity still there (`collect_despawns`), and the
    seven bots of the game before stood on the islands through the next
    game, bodies and health bars, with a teammate's green dot on the
    minimap; House Builder's bots had stood on its plots the same way
    since 2026-10-08, out of sight in the town. Checked with a scripted
    second game: no body left. **The server's bots**
    (`server/src/arena.rs`, `Mind`, `think`) have no map: each keeps to the
    disc of its island (`ARENA_ISLAND_RADIUS` 10.4 m), runs at whoever of
    another team comes within 24 m on or beside it, punches in reach,
    runs away under 20%, strolls when alone, is knocked up and across as a
    body is (`KNOCK_BACK`, `KNOCK_UP`, the drags) and falls off the edge if
    knocked over it; it walks through the island's desk and slab (its
    market stands, since the map of 2026-11-11), and never crosses a
    bridge. (Those were the server's bots until 2026-11-12; `think` moves
    only its zombies since, and the bots it would be gone back to with
    `RT_ARENA_BOTS=old`.) Offline the game is this device's own as
    before, with seven bots. Seen on this PC with a local server (`RT_PORT=5443`,
    `RT_ALLOW_SKIP=1`): seated, the bots in, the fight, a bridge to the west
    island, its two bots knocking you to 16 and off the edge, your punches
    taking 12 off them and their health coming back. **And with two copies
    on this PC and a local server, scripted (2026-11-10, Hajun: "check
    again if the enemy / enemy ai bots health bar decreases on attack and
    they can actually die from damage not just falling, and also check if
    other players can stand on top of your bridge / see u bridging")**: the
    second player, seated and sent to the island at -X, bridged to the
    town, walked across it and onto the first's bridge, and stood on it at
    the planks' height; the first punched them from 100 to 4 by twelves,
    the same on both devices, the server logged the kill, and they stood
    again on their island; the punches knocked them 24 m along the bridge
    and never off it; the server's bots at the island a bridge led to
    killed the player arriving on it ("seat 2 killed seat 0"), by punches,
    not a fall; and offline a bot put beside you went from 100 to 4 and
    died of the punches while taking you to 76. Others see your planks as
    they are fixed, not the plank in your arms (`Carrying` is this
    device's alone) nor the tip. Not yet on a phone with two people.
  - **Eight seats, two to a team, a team to an island** (`ARENA_SEATS`,
    `ARENA_TEAMS`, `roundtown_net::arena_team`, seat modulo four: seats 0 and
    4 share island 0; `arrival` puts the second of each team 4 m from the
    first). Every body in the game carries `Team` (`src/lib.rs`): a punch
    never lands on a teammate (`start_punch` passes them by for whoever is
    behind), a bot never goes for one (`arena::hunt`, the server's `think`),
    and the server refuses a hit on one.
  - **The purple pillar on each island is its team's respawn anchor**
    (`roundtown_net::ARENA_ANCHORS`, read by hand off the `spawn` material's
    pillars, which a test checks, `ANCHOR_RADIUS` 0.85): a bar over it
    (`arena::AnchorBar`, your team's in `colours.arena.anchor_mine`, the
    rest in `anchor_enemy`), 120 of health, ten punches (240 and twenty for
    a night, until Hajun asked for "1/2 of now", 2026-11-09). A punch of
    yours lands on an enemy anchor in its range as on a body that wide
    (`arena::hit_anchors`, off a new `Swung` message `start_punch` writes
    for every punch), never your own; broken, the pillar is hidden and
    taken out of what is stood on (`arena::break_anchors`, `Anchors`,
    `Islands::reshape` without its triangles, `is_pillar`; stood up again
    as the game ends), with a headline. **Dying with your anchor standing,
    you come back in 4 s as before; dying with it broken, you are out for
    good** (`Game::die`, checked at the moment of death, as Hajun asked):
    the body stays hidden and a panel comes up in the middle of the screen
    (`arena::Panel::Out`: "You're out!", "Your respawn anchor is gone.",
    Play again and Exit). The last team with anyone in play wins, or nobody
    when the time runs out: a headline across the middle of the screen
    ("Victory!", "Defeat" or "Time's up!", with who won under it,
    `arena::announce_result`, `RESULT_SECS` 4 s), and 6 s after the end
    (`bridge_wars::RESULTS_SECS`) everyone is back in the town. For a night
    the end was a panel (`Panel::End`, up 15 s, "Back to the town in" and
    the two buttons), which Hajun took for a stray popup (2026-11-09: "idk
    why there is a times up popup that appears near the end of the bridge
    game and it appears on the left side of the screen. remove that"). Play
    again is another game (offline a new one at once; online `Ask::Arena`
    again), Exit the town (`Ask::Leave` online), both on the Out panel
    alone. The words are `words.arena`'s.
  - **Everyone else's name is over their health bar** (`arena::TagOf`,
    `spawn_tag`; `tags` shows no names in the arena): small (2.4 shares of
    the short side) and faint, a teammate's in `colours.arena.ally` over a
    bar in `hp_ally` (green), an enemy's in `enemy` and `hp_enemy` (red),
    2.2 m over the feet, not scaled by distance under the fixed camera.
    (Since late on 2026-11-12 the name is in its team's colour, red, blue,
    green or yellow, and the bar alone says friend or foe; and the bar,
    which until then never shrank with its body's health, does: above.)
    Its shadow, like every text's, falls under the letters by 0.12 of
    their size since 2026-11-10 (`hud::shade`, `SHADOW_DROP`; 3 logical
    pixels whatever the size until then, which under these small names on
    a phone, whose logical pixels are large, was about a third of the
    letters' height: Hajun, "the shadow of the nametag is too far below
    the text").
  - **The bridges are built as you walk** (`src/bridge.rs`): the plank over
    the sword (`hud::BridgeButton`, the slot there until then; B on desktop)
    toggles building (`bridge::Bridging`, the button lit while on). On, you
    carry a plank (`Carrying` in `src/lib.rs`: both arms held out a little
    under level, `CARRY_REACH` 1.45 rad, with a board 1.1 by 0.5 m lying
    across them, `HeldPlank`, a child of your body), walk at half your pace
    (`bridge::CARRY_SLOW`, in `read_player_input`), throw no punch
    (`start_punch`) and show no hit range (`range::follow`; the three
    since 2026-11-09, Hajun: "when bride mode is toggled, decrease player
    speed to 1/2 and also remove the hit indicator and also dont allow
    punching"), and every frame before you move (`bridge::lay`, between
    `start_punch` and `move_bodies`) the ground `AHEAD` of your step is
    looked at: over the void, a plank is laid under it, as Hajun's own
    notes had it. A bridge is a chain of rungs (a point and a heading
    each), a plank between every two: 2.726 m wide and 0.25 m thick,
    Hajun's `bridge` object's measure from their export of 2026-11-08, its
    top at 0.24 m (the islands' 0.25 less a seam, so that the overlap at
    the root hides under the island and the join is the edge itself; and
    up to 7 mm under that by chain, `top_of`, the same on every device, so
    that two chains crossing never flicker through each other), each rung
    half a plank's width on from the last (`SPACING`, 1.363 m: the boards
    of a dock, which follow a curve closely; a whole width for a night,
    Hajun's `s >= 2w`). **The bridge is a ribbon laid under where you
    walk** (since 2026-11-09, Hajun, of the night's version: "when i am
    like on the edge of the bridge or when i turn too much it looks weird
    and choppy. think of a smart way to make briding feel good and look
    good no matter what"): the chain you are building has a tip plank not
    fixed yet, from the last fixed rung to `AHEAD` (1.05 m) past you,
    re-aimed at you every frame and drawn again every frame it moves, so
    that whichever way you step over the void there is a plank under you;
    each time you are a plank's length past the last fixed rung, a rung is
    fixed on the line toward you, so the fixed planks trace the path you
    walked; each joint is mitred (`corners_of`: the corners across the
    bisector of the two planks meeting there, `1 / cos(turn / 2)` wider,
    so the planks share their edges and neither gap nor overlap), turned
    no more than 35 degrees a rung (`MAX_TURN`: a plank with a mitre at
    both ends folds, its inner edge gone, once `2 * HALF_WIDTH * tan(turn /
    2)` passes `SPACING`; 35 degrees leaves a tenth; the bridge turns
    round in about 2.3 m, and walking a tighter circle you walk off the
    tip's side), and a mitre that would reach past a short plank's end is
    drawn in instead. Leaving the ground, the first rung lies on its very
    edge headed straight out from it (the edge found by bisection, the way
    out by sampling the ground round it); walking off a plank's side,
    anyone's, a fork begins flush against that side (a chain of nothing
    but its root is forgotten once you are `FORGET_BEHIND`, 1.55 m, back
    behind it, further in than a chain is ever begun, so brushing an edge
    builds nothing; at half a metre, for a day, a chain was begun and
    forgotten every other frame as you came up to an edge, its tip
    flickering and a root sent to the server each time); walking the far
    end of your own unfinished chain grows it;
    and where you step onto land, or onto another bridge, past where the
    chain began, it ends on that edge, headed at where you stepped onto
    it, with a rung there if the edge is half a plank's length or more
    from the last (`MIN_LAND`: nearer, the chain ends on its last rung, as
    a plank shorter than its own mitre would fold). A corner over the void
    at the root or at the end is pushed out onto the shore (`onto_edge`,
    `shore_ahead`, within `ONTO_EDGE` 2.7 m), so that the bridge meets the
    edge exactly however it curves; one over land stays, hidden under the
    land, which is higher: for a night the corners were pulled onto the
    edge both ways, and pulled in past a short last plank's start they
    folded it (a test caught a top face turned down). Turning building
    off, or dying, fixes the tip where it is if it is a plank's worth.
    **Every plank costs its length in metres of bridge** (Hajun: "players
    will buy the bridge by meter count ... for now make each person start
    with 100m of briding so show '100m' as the number next to the log"):
    `arena.bridge_metres` in `assets/ui.ron` to start, 500 (since
    2026-11-10, Hajun: "increase the default bridge meter to 500m instead
    of 100"; 100 until then; and none since 2026-11-11, when the shop
    began to sell it, above),
    `arena::Resources::bridge_metres`, shown by the log top left until
    2026-11-11 and taken
    as each rung is fixed; with less than a plank's length left no chain
    begins and no rung is fixed, and the tip, never shorter than `AHEAD`,
    runs out under you. The planks are ground: their shape is laid over
    the island's (`Island::lay_over`, `Islands::lay_over`, `overlay`: a
    second shape every question of the island is asked of too, so that
    50,000 triangles are never read again for a plank; laid again once
    the tip has moved 0.2 m or turned 0.05 rad, `OVERLAY_MOVE`,
    `OVERLAY_TURN`, as the minimap is drawn again, `Bridges::version`),
    and you, the bots (offline) and everyone stand on them and fall off
    their sides. Online every rung goes to the server as it is fixed
    (`Ask::Rung`), and comes back to everyone as a `Plank`
    (`bridge::hear_planks`, out of order allowed), yours left out; the tip
    is yours alone. Each chain is one mesh coloured by vertex
    (`colours.arena.plank` on top, every other plank a shade lighter,
    `plank_side` under and at the sides), the ends open but for an
    unfinished tip's cap. Nothing is taken up again; the bridges go with
    the game. Hajun's `bridge` object was taken out of `pvptownmap.blend`
    (saved without a `.blend1` of mine) and the map exported again with
    Apply Modifiers (62 objects, 50,130 triangles, no image: the plank
    texture went with the bridges). Seen in the desktop build, scripted
    (2026-11-09): a bridge off the island's edge wobbling and turning
    sharply with the walk, smooth all the way, the feet on it at 0.236 the
    whole way, and one toward the west island running out with the metres
    at 100 m and dropping you; unit-tested that a chain turning the most it
    may every rung never folds; not yet by hand or on a phone.
    **Since 2026-11-10 a chain's root and its end are cut along the edge
    they meet, at whatever slant** (Hajun, with two screenshots: "unwanted
    gaps and bridge formation when the tilt angle is a bit too large,
    especially when branching off of an existing bridge, and also sometime
    when approaching the island at a weird angle (not tangent) then it
    seemed to not connect properly to the island too"): the first plank
    heads where you walk, up to 45 degrees off straight out of the edge
    (`MAX_SLANT`; until then it stood square to the edge and bent a plank
    later, a stub and a bend off the side of a bridge), its root corners on
    the edge of the ground it leaves, the island's or a plank's
    (`corners_of`, `to_edge`: a corner over the void pushed back onto it,
    as before; one over another bridge's plank pulled in to that plank's
    edge, which a fork's first plank is made long enough for,
    `first_length`, nearly three metres at 45 degrees; one over land left,
    hidden under it); a fork's root is square to the parent plank's own
    nearest edge (`Chain::edge_normal`; the 16 points round it were out by
    up to 11 degrees, and diagonal in the corner between two bridges), an
    island's to the chord of a ring round it (`outward`). Landing, a chain
    comes in no more than 50 degrees off straight into the shore
    (`MAX_END_SLANT`, `land_on`: turning toward the shore a rung at a time
    first, up to three), its end corners pushed up to 3 m to meet it
    (`END_PUSH`; 2.7 before, and past 60 degrees a corner missed a round
    island altogether and the end hung off it) or pulled back onto another
    bridge's plank, and the plank that lands runs at least half a plank
    into the ground, under the land, so that its end edge always crosses
    the shore (it ended on its last rung, with the corners pushed, which
    missed when the last plank pointed past the shore); heading at ground
    within a plank past the next rung, along the plank or along your way,
    no rung is fixed (`STRETCH`): the tip stretches to the ground, so that
    the plank that lands is a whole one, with room for its cut; the rungs
    aim two planks ahead of you along your way (`LEAD`), so that after a
    sharp turn they settle onto your line within a few degrees rather than
    swinging 25 degrees past it; turning back over your own planks and off
    their side ends the chain where it is and begins a fork there, and off
    them onto land or another bridge ends it with no landing
    (`Bridges::abandon`, `Chain::behind`; the tip stayed frozen where you
    had turned back, and off your own bridge's side there was nothing under
    you: a scripted zigzag fell into the void); you have left the ground a
    root is on 0.2 m past its edge line (`GONE_OUT`; half a plank, which
    held back a landing on a bridge half a metre from a fork's root until a
    rung had been fixed on the way); and edges are felt to 5 mm (`EDGE`;
    2 cm left a hairline of void along a cut now and then). **A turn too
    sharp for the tip to follow makes an L corner** (later that day,
    Hajun's third and fourth screenshots, "it happens when i do a close to
    90 degree turn": a rung was fixed a plank on, past where you had
    turned, you walked off that plank's side, and the fork began there,
    leaving the rest of the plank sticking out past the fork and the
    fork's root cut, a long diagonal for a slanted fork, hanging past the
    plank's end; landing within a metre the same fork came out a
    triangle): stepping off the tip through its side
    (`Chain::tip_side_exit`), the tip is fixed as the chain's last plank
    cut off flush with the far edge of the fork that begins off its side,
    `HALF_WIDTH / cos(slant)` past where you stepped off and a hair more
    (`Bridges::corner`, `CORNER_ROOM`), so that nothing sticks out and
    nothing of the fork hangs; and a fork landing within a plank or two
    slants no more than that length can cut cleanly along the plank it
    leaves (`land_on`, from `CUT_SHARE`), so a short one is a square
    gusset rather than a triangle. What stays: turning back over your own
    fixed planks and off their side still leaves the plank past the turn
    as a dead end (it was sent as it was fixed), and crossing a junction
    of bridges at a slant lays a plank across each wedge of void between
    them. Seen in the desktop build, scripted (a throwaway
    `src/scratch_bridge.rs`, removed; 17 legs, no fall): off the island at
    40 degrees, onto the town at 60, a 35-degree fork off a bridge, a
    crossing onto the town, back along a chain and off its side, turns of
    113, 107 and 63 degrees; then right angles over the void, a right
    angle a metre off the town's shore and onto it, and a zigzag of right
    angles, no fall; and unit-tested: a fork cut flush at 0 to 45
    degrees, landings at 0 to 85 degrees meeting the shore whole, drifting
    off a bridge nearly along its edge, a 63-degree turn settling within
    10 degrees, a landing on another bridge cut flush, an L corner at 0,
    30 and 60 degrees with nothing sticking out, and a short fork landing
    with a slant its length can cut. Hajun played the first of these on
    the phone (APK 13) and found the right-angle turns; the rest not yet
    by hand.
    **A bridge that is built stays as it was built** (still later that
    day, Hajun, playing APK 14, with four screenshots, `errorframe1.jpg`
    to `4`: "when there is a unfinised end, and then i build another
    bridge that passes near that unfinished end, suddenly that unfinished
    end's vertices try to connect to my current bridge. i think bridge
    that is already made should not change"). A chain's corners are
    worked out only when the chain itself changes, a rung fixed (or come
    from the server) or its tip moved, against the ground as it stands
    then, and never because another chain has (`Chain::stale`, by how
    many rungs it has and its tip; `Chain::cut`; `Bridges::recut`, the
    chains in the order they were begun); and its root is cut for good
    once its third rung is fixed (`ROOT_KEPT`: its first plank and the
    joint after it both settled; `Chain::root_cut`). Until then `rebuild`
    worked every chain out again, against the land and every other
    chain's planks as they then stood, each time anything was fixed or a
    tip had moved 0.2 m: whatever was built near an old root or end
    afterwards recut it. **And only an end that lands is cut along what
    it lands on** (`corners_of`: its last rung on ground, or ground within
    0.15 m ahead): an end left over the void, let go of, turned back from
    or ended at an L corner, is square where it was left. Its corners had
    been pushed on like a landing's, up to `END_PUSH`, 3 m, so an end
    left within that of a shore or a bridge grew a spike out to it, as
    one in Hajun's first screenshot had to the town's edge before the new
    bridge was begun. So someone else's chain, online, shows a square end
    at each rung as it comes, and is cut along the shore by the rung that
    lands. The meshes follow: a chain is drawn again only when it has
    changed (or the file's colours have), where every chain's mesh was
    made and uploaded again each time; the shape laid over the island is
    still made of all of them each time, from triangles each chain keeps.
    **And a chain forgotten takes its tip's plank down with it**
    (`Bridges::forget`, `orphans`): a chain of nothing but its root,
    walked back from, was dropped and what its tip had drawn left where
    it was, out over the edge with nothing to stand on (found reading
    the code for this; not something Hajun reported). Seen in the desktop
    build, scripted (a throwaway `src/bridge/scratch_bridge.rs`, removed):
    two bridges left unfinished off the island, a third built a metre or
    two past the end of each, every chain's corners logged before, during
    and after. On the code before, the first end's corners moved up to
    0.77 m onto the new bridge and the second's grew a 2 m spike to it,
    as in the screenshots; on this, both chains' corners were the same
    to the millimetre in all five logs, and no fall. Unit-tested: an old
    bridge unchanged as others are built past its end, over its end and
    beside its root, where worked out afresh it would change; an end left
    2 m short of a shore square; a root cut afresh until its third rung
    and kept from then on; a forgotten chain's mesh handed over to be
    taken down. Not yet on a phone.
    **A root and an end follow the edge they meet all the way across**
    (the same day, Hajun, of a sliver in that first screenshot that I
    had pointed out: "fix the sliver gap on curved forks too"). Each was
    one straight line between its two corners, and only the corners were
    cut onto the edge: a chord across an edge that bends. Off the inside
    of a curve of bridge that left void between the fork and the bridge
    it left: a sliver on a gentle curve, and on one turning 20 degrees a
    plank 0.4 m deep in the middle, right where you step off, with
    nothing under the step; off the outside the fork lay 0.18 m over the
    bridge instead. Now the line is tried all the way across
    (`cut_along`): at 9 places evenly (`STATIONS`), each pushed or
    pulled onto the edge as a corner is (`to_edge`), and halfway between
    any two again, up to three times over (`REFINEMENTS`, so as fine as
    4.3 cm), wherever the edge there is 3 mm off the straight line
    between them (`BENT`); of those, the ones it takes to follow the
    edge within 4 mm are kept (`bends`, `STRAIGHT`), none along a
    straight edge. They are the root's edge and the end's (`Edge`,
    `Shape::root`, `Shape::end`, `Chain::root_edge`, `end_edge`, the
    root's kept with its corners at the third rung), and the first
    plank's near edge, or the last one's far edge, is built along them,
    its top and underside in strips side by side (`strips`, in `planks`
    and in `Chain::ground_quads`, which `plank_under` and the ground
    other chains are cut against go by, so what is stood on, what is
    drawn and what is cut against are one shape); a plank between
    straight edges is the one quad it always was. Every place is set
    1.5 cm in under the ground it meets (`TUCK`; the corners too, which
    lay on the very edge, to 2.5 mm, until then), so that no hairline is
    left along the cut: under the land, or under or over another
    bridge's plank, whose top is a millimetre or more off. A plank cut
    at both its ends, a fork of one plank, has its end pulled back no
    further than its root was pulled in leaves it. `corners_of` is
    `shape_of` now, and what it did the tests' alone. Off the inside of
    a bend as tight as a bridge turns (35 degrees a plank, 0.8 m round
    inside) a root is wider than the bend: it follows the edge where it
    can and lies over the bridge either side of that, no void. Cutting
    asks only after other chains' planks within 6 m of the chain's root
    or its last rung (`NEAR`), where it scanned every plank in the game
    for each question. The cost, measured in the desktop build while a
    chain is being begun, its root worked out afresh every frame: 22 µs
    a frame on average and 153 µs at the most, of a frame's 8,333; not
    measured on a phone. Seen in the desktop build, scripted (the
    throwaway again, removed), on the code before and on this: a bridge
    off the island round a bend, a fork off its inside, one off its
    outside and one off its inside at a slant; before, a dark sliver
    along each inside fork's root, and on this none, and no fall.
    Unit-tested: forks off the inside and the outside of a 20 degree
    bend lie along its bent edge all the way across, neither void nor
    overlap, where the chord stood 0.4 m out or 0.18 m in; stepping off
    bends of 10 to 35 degrees a plank, near a joint and between two,
    inside and outside, square and slanted 0.6 rad either way, 72 ways,
    nothing but bridge within half a metre of the step and no strip
    folded; landing on a curve the same; a straight edge still one quad.
    Not yet by hand or on a phone.
  - **The metres left are written in front of you as you build, and
    running out stops you** (since the afternoon of 2026-11-10, Hajun: "i
    think i need another number indicator in front of where ur bridging
    that is the same '### m' number of bridge wood left, and it should be
    white font but orange -> red when its 10m->5m left. and when it become
    0m, it should exit bridge mode (not able to go back to bridge mode
    when u dont have any bridging m left) and pause (even if the user is
    holding the joystick, so the user need to let go of the screen and do
    the joystick again to move) so that they dont fall off when their
    bridge ends"). **The number** (`bridge::show_left`, `LeftAhead`): the
    one that was by the log top left until 2026-11-11, "500m" (`Resources::metres`), as a line of UI
    text 3.6 shares of the short side (`LEFT_TEXT`) placed 2.4 m ahead of
    you the way you face (`LEFT_AHEAD`: past the end of the plank being
    laid, which reaches 1.05 m), after `follow_camera`, while building is
    on. Its colours are `assets/ui.ron`'s `colours.arena.bridge_left`
    (white), `bridge_low` (orange) and `bridge_out` (red), and where it
    turns the file's `arena.bridge_low_at`, 10, and `bridge_red_at`, 5,
    both Hajun's (`Arena::check` refuses them inside out): white with
    more than 10 m, orange at 10, shading evenly to red at 5 and under
    (`bridge::left_colour`), by the number as it is written, to the
    nearest whole metre, so "10m" is orange from 10.4. The shading
    between is my reading of "orange -> red"; the size and the 2.4 m are
    mine. Yours alone, nothing of it online. **Running out**
    (`bridge::lay`, `RanOut`): the frame the plank being laid can no
    longer keep 1.05 m ahead of you for want of metres (`Grown::spent`,
    set by `grow`: the tip is as long as the metres go and never shorter
    than `AHEAD`), or what is left is under half a metre, which is
    written "0m" (`OUT_BELOW`), what is being built is fixed where it is
    (`let_go`), the metres are set to none, building is turned off, and
    you are stopped: your walk is zeroed that frame, and
    `read_player_input` gives none while `RanOut::halted`, whatever the
    stick or the keys say, until a frame comes with neither pushed, which
    lets you go; pushed again, you walk. So you stand on the bridge with
    a plank's worth still ahead of you, about a metre (0.87 m at the
    least, at 60 frames a second: the tip is aimed at you only once you
    are a hand's width past its rung, and a frame's step goes by), and
    the last plank takes all that was left, up to a plank and that
    reach. Dying lets you go too. The "0m", red, stays in front of
    you for as long as you stand stopped and a second and a half
    (`OUT_SHOWN`), my addition, so that the stop says why. **And building
    stays off**: `bridge::toggle` turns it on only with half a metre or
    more, and a press with none shows the "0m" for that second and a
    half instead. Until then the tip ran out under you and you walked
    off the end of it; and with less than a plank's worth left (1.363 m)
    no chain began at all, building on or not, so that off an edge you
    fell with a plank in your arms: a chain begins now with anything not
    written "0m", a stub of 1.05 m that stops you at the edge. Seen in
    the desktop build, scripted through the keys (the throwaway again,
    removed), with 13 m: white "13m" on the island, orange at 10.3 m,
    red at 4.8, stopped on the bridge at none with 1.03 m of plank ahead
    and the key still held, not moved in 2 s, the plank button refused,
    and, the key let go and another pressed, walking again. Unit-tested:
    the bridge running out with 4.5, 12 and 0.6 m, a plank under you
    every frame and one ahead as it ends; the colours; standing until
    the stick, or the keys, are let go. Not yet by hand or on a phone,
    where the stick is a finger's.
  - **A minimap top right** (`src/minimap.rs`, since 2026-11-09, Hajun:
    "put a minimap that only shows the current player's location as a red
    dot and your teammate as a green dot, and the full map in top down view
    including the live bridges but not show the other players"): a picture
    256 pixels square of the map from above, 300 m across with north up,
    the land in `colours.arena.map_land` over `map_back` (the node's
    background, the picture clear round the land since late on
    2026-11-11), drawn once from
    the map's own triangles that face up and lie under 2 m (`draw_land`:
    the islands, the town's middle, the desks, and since the map of
    2026-11-11 the barrels; not the stands' roofs, the trees' leaves, the
    lamps' heads or the waiting room), the bridges drawn over it again in
    the planks' colour
    whenever they change (`draw_bridges`, `Bridges::version`), and over the
    picture a dot for you (`map_me`, red) and one for each teammate
    (`map_mate`, green), placed as you move and hidden when dead
    (`place_dots`). Nobody else is on it. **Each sapphire and each
    amethyst generator is marked on it** (`minimap::mark_generators`,
    since 2026-11-12; Hajun: "in the minimap, add a smaller sapphire and
    amethyst logo onto the minimap where the generators are. so the icon
    need to be smaller so it fits inside the smaller sapphire island in
    the minimap"): the item's stone in its colour, 5.5% of the map across
    (`MARK`) where the smaller islands are 8%, so each sits inside its
    island with the island showing round it, under the dots. It is the
    stone alone, filled, drawn 32 pixels across for the size
    (`hud::item_mark`): the picture the counts have, 128 pixels with the
    line round its table, came out ragged with a ring of flecks at 13
    pixels. Seen in the desktop build: 11 pixels across on islands 18
    across, in a 900-pixel-high window. `hud::spawn_arena_bar` puts it
    up (`hud::MinimapNode`, 26 shares of the short side, `hud::MINIMAP`);
    the picture is made as the app opens (`minimap::plugin`), clear, so
    that the bar can hold it before the map is read.

- **The gamepad down the right edge opens the game selector**
  (`src/games.rs`, `hud::GamesButton`): a dialog like the settings'
  window, "Games" over three cards, Bridge Wars ("Free"), Home Builders
  (its 100 coins, or "Need 100 coins" and blocked out without them) and
  "???" ("Coming soon", blocked out), the cross on its corner, the dark
  round it or Escape to shut it; only in the town, with nothing else up.
  Choosing a game, a black screen comes down over everything
  (`games::Loading`, you standing still under it as under a dialog):
  half a second later the game is gone into under it, Bridge Wars'
  waiting room (`arena::Ask::Enter`) or House Builder's lobby
  (`builder::Ask::Join`, paying its fee as the House Builder's Play did),
  and once the island you are on has loaded and a moment more has passed
  it lifts over 0.3 s; should nothing come of the choice (the server
  turning it down), it lifts after 3 s. The words are `assets/ui.ron`'s
  (`words.games`). House Builder's own flow from the lobby on is as it
  was ("House Builder", below); the House Builder NPC stands at a house
  only in a town that has one, and `roundtownmap.glb` has none.
- **Bridge Wars** (`src/arena.rs`): you are stood in the waiting room,
  Hajun's glass box floating over the middle of the map, the object
  called `waiting room` in `pvptownmap.glb` (their export of 23:21 that
  night: 30 m square, its floor a metre thick with its top at 100.5 m, its
  walls 1.5 m thick, thicker than the lattice, and 5 m high, more than a
  jump, open to the sky, in `glass` at a tenth; 50 m higher, its floor's
  top at 150.5 m, since their map of 2026-11-11, "The PvP map itself",
  below), found by name once the
  map has spawned (`arena::read_map`, `WaitingRoom`: the box
  round its meshes; its floor is read off the map's shape under its
  middle, so it is stood on as any of the map is) and on a ring 3 m round
  its middle, facing in; with no such object in a map, everyone waits on
  their island. The board top centre (small, as asked) counts "1/4
  Players", "Waiting for players..."; nobody else can join yet, the game
  being this device's own, so after `BOT_WAIT`, 10 s with nobody joining
  (`roundtown_net::bridge_wars`, with `COUNTDOWN` 5 and `PLAY_SECS` 600,
  so that a server playing it one day plays by the same), the computer
  takes the seats left, all at once (`ARENA_SEATS`, 4; the town's AI
  names and looks, `lobby::seat_ai`, `spawn_player_at`), the board counts
  down 5 to 1 over "Match found!", and everyone is sent to the island
  their seat says (`roundtown_net::arrival`, as the town's arrivals were
  that day), "Fight!" across the screen; then the board shows 10:00
  (25:00 since 2026-11-11, with a timetable on the way, above)
  counting down over "Bridge Wars", a tap on it skips (any stage: the
  bots at once, the countdown over, the game over), and at 0:00 everyone
  is back in the town where their seat arrives, the bots gone
  (`arena::InArena` on everything the game puts up), the camera as it
  was. Nothing is scored; `arena::Health` stays full and
  `arena::Resources` at nothing. A game of House Builder is entered the
  same way and runs as before. Seen in the desktop build, scripted (taps
  through `WindowEvent::TouchInput` at the buttons' laid-out places,
  `arena::Ask::Skip` written to hurry it): the dialog, the black screen,
  the waiting room from above, four bodies at the countdown, the island
  with "Fight!", and the town again with the camera and the buttons
  restored; and Home Builders to its lobby with the fee taken. Not yet by
  hand or on a phone.
- **The fight** (later that night, Hajun: "make the ai, if there is
  another ai/player within its visible radius, it will go towards them
  and try to fight/punch them ... make punching deal damage, and show
  healthbar for enemies. make default hp = 100 ... make damage numbers
  appear on each hit. default damage = 12. so when health <0 u die too
  ... 'You died. Respawning in 4...' -> 'You died. Respawning in 3.5...'
  until it goes to 0 and then you respawn and the camera goes to where u
  respawned ... for falling make it so u need to drop a bit more z to
  actually die. increase knockback slightly ... make it running rather
  than walking"); every number is `assets/ui.ron`'s `arena` (`ui::Arena`,
  read as the car's are, hot-reloaded): `run_speed` 10 m/s (a walk is 7),
  `health` 100, `damage` 12, `respawn_secs` 4, `regen_after` 5 and
  `regen_per_sec` 4, `hunt_radius` 24, `bot_punch_every` 0.8 (0.5 for an
  hour, until Hajun found the bots "so strong": "decrease ai hit
  frequency slightly"), `flee_below` 0.2 and `fall_to_die` 30 (`void_y`
  in `src/lib.rs`, the land less that; 15 m until then). Everyone runs in the arena
  (`Intent::sprint`, the run over a walk; the legs swing that much faster
  and up to half as far again, `animate_walk`; set by `read_player_input`
  for you and `ai::steer` for a bot, by the island they are on). Every
  body there has `arena::Health`, you included (given on entering, taken
  away on leaving); a punch that lands and takes (`Landed`, written by
  `start_punch` and by a bot's punch) takes `damage` off the victim while
  the game is on, never in the waiting room (`arena::deal_damage`), a
  number floating up off them for 0.9 s (`DamageNumber`, in
  `colours.arena.damage`), and everyone but you has a bar over their head
  (`BarOf`, placed as names are, `tags`), yours being the bar top left;
  both drawn in steps of half a percent, so that health coming back a
  little every frame does not lay the screen out again every frame.
  **Health comes back** once nobody has hit a body for `regen_after`
  seconds, `regen_per_sec` a second up to full (`arena::regen`, `Health::
  mend`; the while since the last hit counted without marking the health
  changed; later that night, Hajun: "make players health regen slowly
  after they have not been attacked for some period"); bots' too. At none,
  or fallen `fall_to_die` below the map (`arena::reap`), a body is `Dead`
  (`src/lib.rs`): hidden where it died (`show_where_i_am` for everyone
  else, `follow_camera` for you; for an hour a dead bot stood where it
  died in plain sight, Hajun: "the ai body doesnt disappear"), neither
  moving nor moved nor in anyone's way nor hit (`move_bodies`,
  `start_punch`, `ai`), the punch's range band put away with you
  (`range::follow`; it stayed over the void for an hour, Hajun: "the hit
  indicator is not gone"), for `respawn_secs`, you with "You died.
  Respawning in 4..." across the screen counting down in halves
  (`words.arena.died`, `arena::show_death`), and then put back where its
  seat arrives with its health full (`arena::respawn`; the camera, fixed
  over you, is there with you). (What follows was every bot's mind
  offline until 2026-11-12, and is the zombies' alone since; the
  computer's players have the brain the server's have, above.) **A bot
  goes for whoever is nearest within `hunt_radius`** (`arena::hunt`, after `ai::steer`, over its
  wandering, only while the game is on): at a run up to them, pressing in
  at `PRESS_IN` of a push once within the punch's reach, and a punch
  whenever they are in the punch's own range (`range::Sector`) and its
  `bot_punch_every` is up, which knocks and lands as yours does; a hit on
  the move is a sprint hit, as ever. **With less than `flee_below` of its
  health it runs away instead** (Hajun: "make the ai run away when its
  low health (below 20%)"; `arena::flight`): straight away from them, or,
  with no dry land that way for `FLIGHT` 3 m, the nearest way round that
  has it, to either side and then across (`ai::dry_all_the_way`), and
  stands with none; so beside the island's middle it runs to the edge and
  then along it, and seen on the map it ran to the bridge's end, where it
  stood a while. Once its health is back over the line it fights again.
  It sees the dead not at all, and is kept from walking off its island
  as ever (`StaysAshore`), so a target across the void it stands at the
  edge for; bots on four islands 130 m apart never see each other until
  someone crosses a bridge. The knockback is `melee.knock_back` 10 (8,
  Minecraft's, until then). Seen in the desktop build, scripted
  (`arena::Ask::Enter` and `Skip` written, a bot put beside you with
  `lobby::put`, its health and yours written to): the bot coming, "12"
  off each hit on either, its bar over its head, you knocked off the
  island into the void, the words counting down over the dark with the
  band gone, and you standing again at the island's middle with the bar
  full; a bot's hit on you right after; a bot at 15% running off to the
  edge and round; a dead bot gone, bar and all, and back on its island
  4 s later; health climbing 4 a second from 5 s after the last hit. Not
  yet by hand or on a phone.
- **Online, a game of Bridge Wars is a room of your own**, as home is:
  entering it asks the server `Ask::Travel(Venue::Arena)`, which seats
  you alone there (`server/src/players.rs`, `room_for`, as `Venue::Home`),
  so that you vanish from the town's channel rather than stand frozen in
  it, and leaving asks `Travel(Venue::Town)`. Nothing of where you stand
  in the arena goes to the server (`net::tell_where_i_am`), and its
  `Tell::Arrive` for the arena never moves you (`net::hear`): the game is
  the device's, bots and all. The server's `Ask::Respawn` is not asked
  there (`lobby::respawn_the_fallen`).
- **The camera is fixed straight down over you only in the arena**
  (`OrbitCamera::top_down`, which keeps how it was to go back to,
  `arena::Game::view`; the car's `overhead` keeps its own, and going into
  the arena gets you out of the car, `drive::toggle` on the change of
  island, the camera going back to how it was before the car rather than
  to the car's, `Driving::take_view`): the file's `arena.camera_height`
  up, 15.5 m, with the top of the screen north, nothing turning it and,
  since late on 2026-11-11, nothing zooming it ("The camera stands at one
  height in a game", above; the car's `overhead`, 24 m, zooming between
  `overhead_nearest` and `overhead_furthest`, until then). Everywhere else it is behind
  you as it was before 2026-11-07, "The camera stands where you zoomed
  it" below, and on desktop the cursor is taken for looking round as the
  app opens (`lock_cursor`, unless `RT_FREE_CURSOR`), and let go for as
  long as the camera is fixed over you, in the car or the arena
  (`cursor_for_dialogs`), where the stick and the buttons want the mouse.
- **In the arena the buttons are Hajun's `pvpmapbuttonui.jpg`**
  (`hud::Layout::Arena`, `hud::spawn_arena_bar`, put up as you arrive on
  the arena and the town's again as you leave, `hud::relayout`): top left
  your name on its plate and under it your health bar, 30 by 3.6 shares of
  the short side with a rim, filled red (`colours.arena` in
  `assets/ui.ron`: `hp`, `hp_back`, `hp_rim`), and under that what you
  hold of the four things the generators make, silver, gold, sapphire and
  amethyst, each a picture of it, tinted its colour, with its count
  beside it (`hud::RESOURCE_ICON`, `RESOURCE_TEXT`, `hud::item_picture`,
  since 2026-11-11, above; until then wood, gold, diamond and emerald,
  white, the wood's count your metres of bridge and the rest 0) (there since 2026-11-09,
  Hajun: "move the 4 item icon + number thing on the top right side of
  that screen to the left side of the screen below the hp bar ... remove
  the inventory icon"; for a night they were top right under a bag, the
  inventory, which did nothing); top right the minimap in the bag's place
  ("A minimap top right", above); bottom right, in the jump button's very place (the jump button
  is put away there, `hide_touch_hud`), the sword, which punches as the
  fist does (`hud::PunchButton`, so in the car it is the reverse), and
  round it on an arc 20.5 shares out from its middle three buttons the
  size of the town's action buttons, alike and evenly spaced (later that
  night, Hajun: "make the 3 circles surrounding the sword icon in that
  map the same size ... and even the distances out"; for an hour two were
  that size and one over the sword small): beside it the jump button
  (`hud::ARENA_JUMP`, which `hud_layout` in `src/lib.rs` reads presses
  on in the arena, lit while held as the one in its own place is; Hajun:
  "i want the bottom left one to be the jump icon and a jump button"),
  and up and to the left of it and over it two empty slots
  (`hud::ARENA_SLOTS`, `hud::empty_slot`: the buttons' disc with a faint
  rim, `colours.arena.slot_rim`, which takes a press and does nothing),
  where a bow and dynamite go once bought (the one over the sword is the
  bridge button since 2026-11-08, and the other the second button, for
  what the item shop sells to use, since 2026-11-12: `hud::utility_slot`,
  `utility`); and over those a smaller
  button of three dots (`hud::ARENA_DOTS`), which does nothing yet (it
  showed the empty item buttons from 2026-11-13, and is not put up at all
  since 2026-11-14, `hud::DOTS`: "Games", above); the
  stick stays, and nothing else of the town's bar is there (no balance,
  person, gear, gift, social, face or chat). The pictures are drawn in code
  (`hud::sword`, `item_picture`, `plank`, `dots`, and the `bag`
  nothing shows now, through `hud::drawn` and a polygon distance,
  `polygon_distance`; tests sample each; the `log`, the `ingot`, the `gem`
  and the `emerald` went on 2026-11-11). Space still jumps on desktop. The punch's range band is drawn
  only in the arena now (`range::follow`).
- **Falling off the map** is the arena's alone (`island::bottomless`):
  the town has its sea back. The bots are kept on by `strands` as the
  town's AIs are; a punch knocks them off all the same, and they are dead
  `fall_to_die` down, as you are ("The fight", above).
