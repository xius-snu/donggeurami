# The bag: what you own, the shop, gems, and what stands on your plot

Your bag (`src/bag.rs`), its window (`src/bag/panel.rs`), the rules of it
that both sides play by (`roundtown_net::bag`), the server's side
(`server/src/bag.rs`), the buildings put down on a plot of the town
(`src/build/stand.rs`), how its window's lists are scrolled
(`src/flick.rs`), and changing what stands on your plot: edit mode and
paint (`src/customize.rs`, `src/build/paint.rs`). Dated entries, newest
first.

## Contents

1. 2026-11-22, late: a spot of the colour chosen in its window, a hint at edit mode for someone tapping at a thing, and a house's walls numbered the same on every device, so that its doors are in the same wall for everyone; APK 51
2. 2026-11-22, night: paint taken off with a cross, the colours used along the window's foot, the one button moved up, a tap away letting go, things dragged through each other with a grey tick, a house taking what is in it along, smaller menu buttons set by the thing as drawn, and two faults of see-through (things deaf to a tap, eyes fading); server deployed, APK 50
3. 2026-11-22, evening: edit mode and its three buttons by the left edge, the paintbrush and the colour window, paint kept by the server and seen by everyone, the car's door where the bag's button is; server deployed, APK 49
4. 2026-11-22, after that: a list flicked goes on by itself as a phone's does, and a swipe across the window changes its tab; APK 48
5. 2026-11-22, afternoon: the coins' number the gems' size and the gem the coin's; a price you cannot afford in red; the box on the bag's button; APK 47
6. 2026-11-22, later: the shop after Hajun had looked at it (both buttons always, over the picture and see-through; the price under the name; All first shown; six categories with no scrolling); APK 46
7. 2026-11-22: the bag, its window, gems, and buildings on your plot that everyone sees; APK 45

## 2026-11-22, late: the colour chosen shown in its window, a hint at edit mode, and a house's doors in the same wall on every device; APK 51

Three more of Hajun's that evening (sitting and lying down, asked with the
first of them, is `docs/town.md`'s; the hot phone, `docs/rendering-and-
frame-rate.md`'s):

> also in the paint color palette i want another circle that shows the
> current color maybe in the gap between the top right corner of the rgb
> circle, and the right bar, u could sneak a circle current indicator
> without resizing or moving any other elements in that panel - just so
> its easier for ppl to tell what is the current color without having to
> look the left edge of ur screen.

> also the house rotation is different for another person like the door
> was on the wrong wall on my friend's screen, but in my screen he jumped
> through my window. make it consistent

> also for a small userfriendly way for users, just for the first few
> times, if they repeatedly tap on an object without being on house edit
> mode, give small text like "Press the edit house mode to move objects"
> sth like that as a hint. cuz i think some ppl after placing an object
> and they try to press the object to move again, but dont know they have
> to be in house edit mode to do it.

- **The colour chosen, in its window** (`picker::Now`): a round spot 8.8
  across in the corner the wheel leaves at its top right, out over the
  gap to the bar, as far clear of the one as of the other; the colour as
  it is, as a finger moves. Nothing else of the window is moved.
- **The hint** (`stand::hint_at_edit_mode`): with edit mode off, a second
  tap on a thing of yours within four seconds of the first says "Tap the
  house edit button to move things." (`words.bag.edit_hint`) across the
  screen for three and a half. Three times ever on a device
  (`Customize::may_hint`, kept in `paint.json`), and it takes two more
  taps each time. A tap a bubble took is not one.
- **A house's walls are numbered the same everywhere** (`build::within`,
  `wall_order`). A door or a window is kept as which wall of its house it
  is in, the walls counted in order of their low corners. Those corners
  were worked out through the world, the wall's place there in the
  house's place there, and came out a few millionths of a metre off, by
  a different few wherever the house stood. Three walls of Hajun's
  houses begin at the same place across, so which counted first hung on
  those millionths: on one plot the order was front, right, back, left,
  and for a house on another, right, back, front, left (logged, with
  the old way put back for the purpose). A plot is whichever your seat
  has, so the order could change from one visit to the next, and
  between the device that put a door in and everyone else's. Now each
  part is measured in the house's own space, by how each node stands in
  the one above, the same to the last bit anywhere; and the walls are
  ordered by their corners to the millimetre: front, right, back, left.
  House Builder's houses are numbered by the same code.
- **What this does to doors and windows already in**: each is kept by a
  number that meant whichever wall was so numbered where the house stood
  when it went in. From APK 51 the number means one wall for good. A door
  put in while the order was another will stand in another wall now, on
  its owner's screen as on everyone's: it is to be moved once, and stays.
  There is no telling from what is kept which wall was meant.

**Calls that were mine.** Where the spot is and how big; its rim the
window's line. The hint's words (Hajun's "sth like that"), three times,
two taps four seconds apart. Not moving anyone's doors for them.

**Checked.** The tests (the spot clear of the wheel and the bar; two
taps, and never more than three times, kept in the file; the walls'
order with their corners a little off, and the old order that came of
it). Hidden on the desktop: the window with its spot; two taps on a sofa
and the words across the screen, `paint.json` saying once. Two copies
against a server on this PC with the two real plots: every one of the 39
things of the one and the 13 of the other, their 17 doors and windows
among them, stood on the other copy where its owner's had it.

**Not checked.** On a phone. A door put in before this, on the device
that put it in.

## 2026-11-22, night: paint taken off, the colours used, a tap away, things dragged through each other, a house with what is in it, the menu's buttons, and two faults of see-through; APK 50

Hajun, having tried APK 49, in five messages:

> i think for the painting mode, on the right side of the rgb color icon,
> it should be an x/clear icon and u can select between the color or the
> clear, and if u paint sth with the clear, it will clear that color to
> its original. also, right now the house edit button, is a bit too down,
> i think u can move it up a bit, and when it flanels out to the 3
> buttons, the middle one can be the location of the original house edit
> button, so i guess the location right now where the paint brush icon
> is, that is where the original house edit button is, and when u press
> the house edit button, that location becomes the paint brush, and above
> it is the house edit, and below is the check. also in the color
> selector, instead of "Color" have "Paint Color" and on the bottom of
> hat panel, it should be like one row of the previous used color history
> (in the beginning it can be empty) also i think for the editing mode,
> when u tap an object and brings up the 4 buttons (trash,rotates,check)
> there is no feature for "cancel" and so i think when u tap somewhere
> that is not the object, it should "cancel" so it closes the 4 button
> menu, and if the position is valid/not colliding it will apply it, but
> if the position is invalid, will just "cancel" and set it back to the
> previous applied position. also when ur moving around the furnitures,
> it seems like moving it around is a puzzle cuz u cant move it through
> other objects due to collision, but i think it should be freely able to
> move around even through other objects but it will just grey out the
> check mark if its colliding/cant be applied. then rebuild apk and push
> to git.

> also right now, when u try to move your house object that has
> furnitures inside + doors and windows, and i try to click check, it
> says it has to stand on land. i think everything inside the house obj
> including the doors and windows should move together if u decide to
> move the house.

> also i noticed sometimes, not always, some objects become unclickable
> (like a sofa) or like part of the chair (the seat part of the chair
> suddenly disappear + become an unclickale chair until i reopen the app)
> see if there is a solution to that error.

> also i think the 4 buttons during edit mode (trash,rotates,check) can
> be slightly smaller, similar size as the current top right corner icons
> like the social/minigame icons and make it more better/accurate in
> where the buttons are relative to the cam angle/zoom and object

> also right now, when the player is in the house, the eye becomes
> transparent i think thats a bug. eyes and mouth shouldnt become
> transparent

**What was built.**

- **The cross beside the colour** (`customize::Ink`, `customize::clear`):
  with the brush, two round buttons beside it, the colour and a cross
  (`icons/close.png`, the windows' own), the one the brush paints with
  ringed. With the cross a tap takes the paint off the part under it, and
  it is as its model has it (`paint::tap`, `Bag::unpaint`,
  `roundtown_net::bag::Paints::clear`). Online that is asked of the
  server (`bag::Ask::Unpaint { id, part }`, one more variant at the end,
  `docs/online.md`), which tells everyone in the channel who asked for
  colours, as it does of paint.
- **Where the buttons are.** The one button is where the brush is among
  the three, from 38.75 to 49.75 of 100 down (it was from 51 to 62, where
  the tick is); the three are as they were, the house and pencil over it
  and the tick under. That is where the chat's lines begin, so they step
  aside of the one button too (`customize::aside`).
- **"Paint Color"**, the window's title (`words.bag.paint_title`), and
  along its foot **a row of the colours painted with**, the last seven,
  newest first, each once, none to begin with (`Customize::used`,
  `note_used`, `picker::Past`): a tap on one chooses it again. They and
  the colour chosen are kept on the device for the next time the app is
  opened (`paint.json`, beside `bag.json`; `customize::keep`), written
  once the window is shut, not as a finger is dragged over the wheel.
- **A tap somewhere else lets go** (`editor::Edit::Away`, from
  `editor::drive_menu`; `build::edit_pieces`): with the four buttons up
  on a thing of yours, a tap that is on neither the thing, nor its
  buttons, nor a button of the screen, puts the thing down where it is
  if it can stand there, and puts it back where it stood if it cannot
  (`stand::PutBack`, `stand::put_back`); the menu goes either way.
- **A thing goes where the finger takes it, through whatever stands
  there** (`build::drag_freely`), and turns where it is. Whether it can
  be put down where it is, is said on it every frame it is in your hand
  (`build::mark_misfits`, `build::misfit`, `editor::CannotStand`): the
  whole of it on the plot, in nothing else that stands there
  (`build::overlapping`: the two boxes, each turned as it is, by more
  than a touch), a house where a house fits. **The tick is grey while it
  cannot** (`colours.edit_menu.confirm_off`, `confirm_off_rim`), and
  pressed all the same says why: "Something is in the way."
  (`words.bag.in_the_way`) or "It has to stand on your land.". Brought up
  to another thing it still goes flush against it and in line, as it did.
- **A house takes what is in it along** (`stand::Rides`,
  `stand::ride_along`, marked in `stand::note_edits`): whatever of yours
  stands on its floors as it is picked up is carried where it stood in
  the house, wherever the house goes and whichever way it is turned, is
  in nobody's way meanwhile, and is put down with it, the bag told of
  each. Its doors and windows always went with it, being parts of it.
  That was the "it has to stand on land": the beds in a house were in
  the way of the house.
- **The menu's four buttons** (`src/editor.rs`): 10.5 of the screen's
  short side across, the size of the two people and the gamepad down the
  right edge (13 until now). Each stands just clear of the thing as it is
  drawn (`editor::Reach`, in `EditMenu::follow`): the bin over its top,
  the tick under its foot, a turn to each side of it, by 1.5 of room; so
  they close in on a small thing or one far off, and open out round a big
  one or one zoomed in on, where they stood 15 from one point of the
  thing whatever its size. No nearer that point than 9, which keeps them
  clear of each other, and no further than 26; the bin under the time and
  the balance first. Never held to the screen, as before.
- **Things that could not be tapped, and the chair's seat that went.**
  One fault (`see_through::fade_what_hides_you`): a part in the way of
  the camera for a single frame, standing in a house, was given its
  see-through copy and, being all there still the frame after, was left
  with it for good. That copy is drawn from one side only, so a part
  modelled facing the other way, the cafe chair's seat, was gone; and
  what the part belongs to was out of a tap's reach (`InTheWay`), until
  the app was opened again. Now it is its own material again the frame
  after.
- **Eyes and mouths** (`see_through::of_a_body`): on every look but the
  first boy's, whose face is a picture, the eyes and the mouth are
  meshes of their own hung on the head, and so were in the way of the
  camera like a chair, and drawn at a quarter while you stood in a house
  facing it. Nothing of a body is in the way now, yours or anyone's.

**Calls that were mine.**

- **The round colour button** takes the colour up when the cross is in
  hand, and opens the window when the colour is in hand already.
- **A colour is used when something is painted with it**, not when it is
  chosen in the window. Seven, a row; on the device, not with the
  account.
- **Just out of the bag and let go of where it cannot stand, a thing goes
  back into the bag**, having no place to go back to.
- **A tap on another thing of yours lets go of the one in hand**; a
  second tap picks the other up. A finger dragged, to look round, lets go
  of nothing.
- **Through each other on your plot only.** A piece in House Builder is
  carried up to what is in its way and no further, and waits for its
  tick, as it always did: that game is stalled, and nothing was spent on
  it.
- **A thing dragged keeps to the floor it is on** (`build::floor_under`):
  off the land onto a house's ground floor, a step up; off the edge of an
  upper floor, down. It no longer climbs stairs under the finger: to have
  a thing upstairs, bring it out of the bag standing there.
- **What rides in a house** is what stands on its floors, not what stands
  on the land beside it, which is in its way.
- **The numbers of the menu's buttons**: 1.5 of room, 9 at the nearest,
  26 at the furthest, each side's turn by its own side of the thing.
- **Nobody's eyes fade**, not only yours.

**Checked.** The three crates' tests (the game's 543, the rules' 259, the
server's 117); new among them: paint taken off (the rules, the wire's
order, the server, the device), the colours used and their file, where
the buttons are and what the chat steps aside of, a thing dragged through
another and what is in the way, the floor under a thing dragged, a house
carrying what stands in it and putting it down, a thing put back, the
menu's buttons by the thing as drawn, a part in the way for one frame,
no part of a body. The desktop build run hidden at 874 by 402, by day,
offline: the one button in its new place and the three; the chat's lines
beside each; a bed out of the bag onto the lamp (the tick grey), let go
of there (back in the bag), dragged clear (green) and let go of (down),
dragged back on to the lamp (grey), the grey tick pressed ("Something is
in the way."), let go of (back where it stood); a house with a bed, a
chair, a door and a window in it picked up, dragged 5.5 m and turned,
all five where they should be and put down by the tick; a roof painted,
cleared with the cross, a colour chosen on the wheel, painted, the
window's row with both colours, the older one tapped and chosen,
`paint.json` written. **Both faults seen before and after**, by a switch
for the old way taken out again: the girl's eyes and mouth drawn at a
quarter in the house and solid now, at 42 places of the camera; the
chair's seat gone and the chair deaf to a tap after one frame of it in
the way, and whole and picked up now. Two copies against a server on
this PC: one painted its lamp and cleared it, the other was told of each.
**The deploy**: nobody connected by the VM's journal (the last had left
at 08:03:58 UTC); the running binary kept as
`roundtown-server.prev-2026-11-22c`; up at 09:37:47 UTC on 2026-11-22,
the VM's binary the same as the one built here (`fe58c3de`); this PC's
own account in, hidden: its lamp stood, painted, cleared and taken back,
its bag in the database as it was before, the journal clean.

**Not checked.** On a phone: APK 50 was built and not installed, none
being attached. Both faults were found and put right on the desktop
alone; whether they were all of what Hajun saw on the phone is his to
say. By night; a tablet's shape; a house of two floors; a house with
things in it moved online (each thing's place is told as a thing moved by
itself is, which was); an app from before against this server, by the
tests alone. iOS: not type-checked, nothing behind `target_os = "ios"`
touched, no tag asked for.

## 2026-11-22, evening: edit mode, the paintbrush, and the door in the bag's place; APK 49

(Changed that night, the entry above: where the one button is, the
window's title, the colour kept when the app is closed, and a thing in
the way of another. Committed with it.)

Hajun, after APK 48:

> also for when ur on the car, i think the leave door/car button instead of
> left center, just put it where the inventory ui supposed to be cuz rn
> that slot is empty. also i want the left center spot to be a ui button
> that only appears while you are in ur plot, and its a "customize icon"
> maybe an icon that looks like editinghouse/sign/customize icon, and only
> when u press it, it toggles on the edit mode, which alllows u to tap any
> object in ur plot to move it around - with edit mode off, tapping objects
> should do nothing, and also when u open invetory and tap an object to
> place it, then edit mode gets toggled on until u turn it off. and i think
> when edit mode is off, its just one ui button, but when its on, it
> switches to three buttons in a column on the left middle, the top one
> being a normal edit/customize icon, the second one being a paintbrush
> icon (which will be a button for coloring/texturing the objects/house so
> when the paintbrush icon is selected, u also get a circular icon on the
> right side of it that just shows a color, and u tap that color button and
> it opens a rgb selector panel, and u can select a color there and then u
> can tap objects and it will paint it to that color (so if u tap the
> blanket of the red bed, it will just change that red color to the color
> that u chose, or if u tap your house wall, it will just change the color
> of the walls, and if u tap the roof, it will just color the roof, if u
> tap the floor, it will just color the floor, so its like a modular
> colorer (color gets reset if u trash that item and place it back, but
> stays if u leave it and also the new color should be visible to other
> players), and the third one being a check mark button (which is the edit
> mode off button). the rgb selector should be a circular color selector
> with white in the middle, and also on the right it should have a
> brightness (whiter/darker) bar selector, no transparency selector.

They asked for neither an APK nor a push this time. The server was
deployed, as the rule for a change to `shared/` has it; APK 49 was built
(mine: every round before ended in one, and the phone shows none of this
without it); **nothing is committed**.

**What was built.**

- **The door** out of your car is where your bag's button is out of it
  (`hud::BAG_PLACE`), among the buttons round the jump button, and as big
  as they are; no longer half way down the left edge.
- **The button by the left edge** (`hud::LeftEdge`, filled by
  `customize::draw`): a house with a pencil across its corner
  (`hud::customize`, drawn in code), there while you stand on your plot
  in the town, out of your car. A press turns **edit mode** on
  (`customize::Customize`).
- **Edit mode**: three buttons, one over the other, in its place: the
  house and pencil, which is the hand; a paintbrush (`hud::brush`); and a
  tick (`hud::tick`), which is edit mode off. The tool in hand has a ring
  round it and its picture white. With the hand a tap on a thing of yours
  picks it up, to be moved, turned or binned, as before; with edit mode
  off, or with the brush, nothing of yours is picked up by a tap
  (`stand::hands_off`, `build::untappable_from_inside` for your house).
  Bringing a thing out of the bag turns edit mode on, with the hand, and
  it stays on (`stand::bring`).
- **The paintbrush**: beside it a round button the colour it paints
  (`customize::swatch`). A tap on a part of a thing of yours paints that
  part (`build::paint::tap`): the nearest of your parts under the finger,
  found on the meshes themselves (`MeshRayCast`). **A part is a material
  of the model**, by its name in Blender (`paint::Part`): a bed's
  `blanket`, `bedframe` and `mattress`; a house's `plain_wall`,
  `plain_roof` and `plain_floor`; a door's `door`, `doorframe`,
  `doorhandle` and `glass`; a lamp's `lampostbody` and `lightbulb`.
  Everything of one thing drawn with one material is painted together.
  What is painted is drawn with a copy of the part's own material in
  that colour (`paint::Looks`), as see-through as it was.
- **The colour window** (`customize::picker`), behind the round button: a
  window as every window is, "Color" along its top (`words.bag.
  paint_title`), a wheel with white in its middle and every colour round
  its rim, and to its right a bar from that colour at its top down to
  black. A finger put on either, or dragged over it, chooses; a ring on
  each marks the choice, and the round button by the brush is that colour
  as it changes. No see-through to choose.
- **Kept, and seen by everyone.** What you have painted is beside your bag
  (`bag::Bag::paints`, `roundtown_net::bag::Paints`: the thing's number,
  the part's name, three bytes of colour), in `bag.json` offline and with
  your account online, in the same document as the bag
  (`server/src/bag.rs`, `Kept::paints`). Taking a thing back into the bag
  lets its colours go; moving it keeps them. Everyone in your channel is
  told (`bag::Tell::Painted`) and draws it (`stand::keep_colours`,
  `paint::colour_parts`).
- **Over the wire** (`docs/online.md`): `bag::Ask::Colours` and
  `bag::Ask::Paint`, and `bag::Tell::Painted`, each added at the end. The
  server tells of colours only a device that asked (`Colourers`).
  **`PROTOCOL_VERSION` is 24 still**: APK 45 to 48 play on, and see
  everything as its model has it.

**Calls that were mine.**

- **Where the buttons are.** The left edge is taken above by your name,
  coins and gems (to 24 of 100 down), in the middle by the chat's lines
  (38.5 to 48.6), and below by the stick (from 64). So the one button is
  a little under half way, 51 to 62, under the chat's lowest line; and
  the three grow up from there, the tick where the one was. The three
  reach up past the chat's lines, which step aside to the right of them
  for as long as edit mode is on (`chat::Aside`). The door, when it stood
  half way down, stood on the chat's lowest line.
- **Edit mode ends by itself** when you are more than 3 m off your plot
  (`customize::NEAR`), in your car, or out of the town. Whatever was in
  your hand then goes back where it stood, or into the bag if it never
  had.
- **The tick, or the brush, with a thing in your hand** puts it down where
  it is first, as its own tick does; if it cannot stand there ("It has to
  stand on your land.") the hand stays, and the menu on it.
- **Paint is at once**, with nothing to undo it but painting again, or
  taking the thing back into the bag.
- **Glass stays glass** painted, and a thing with a pattern (the couch,
  one material with a picture on it) is tinted over its pattern. A lamp's
  bulb can be painted.
- **A house's gables and stairs are its walls**, being of that material.
- **The brush begins blue**, and the colour chosen is not kept when the
  app is closed.
- **The bar is how bright**, to black at its foot; the wheel is always as
  bright as it goes ("white in the middle").
- **No more than 400 painted parts to a plot** (`MOST_PAINTED`), and a
  part's name no longer than 40 bytes.
- **Only colour**: "coloring/texturing" was Hajun's word for what the
  brush will be; no textures were made.
- **On desktop the cursor is let go for as long as edit mode is on**, to
  tap with (`build::want_pointer`).
- The town's own objects, tapped offline to edit the map, are as they
  were.

**Checked.** The three crates' tests (the game's 533, the rules' 259,
the server's 117): the rules of paint, its order on the wire, the
server's part (kept, told to those that asked and to nobody else, a
purchase keeping it, a thing taken back losing it, a bag kept before
colours reading as it did), the device's (painted at once, the server's
word, kept in the file), the materials (every mesh of a part, nothing
else, a door hung on a house its own thing, a ghost), the edit mode's
rules, the colour's arithmetic and the window's. The desktop build run
hidden at 874 by 402, by day, offline: the one button; the three; the
brush and the round button; the window, a tap on the wheel, on the bar,
and a drag; a bed out of the bag (edit mode on by itself), put down, its
blanket painted blue, frame and mattress as they were; the bed and its
colour there again on the next run, out of `bag.json`; the bed picked up
and binned, its colour gone; a house's walls (red), roof (blue) and,
from inside through the wall drawn see-through, floor (green); the
painted house picked up, a ghost in its colours, and put down by the
tick; the brush taken with the house in hand; a step off the plot (still
on) and well off it (off); the car, its door where the bag's button was,
and out by it. Two copies against a server on this PC: one stood its lamp
and painted its post; the other, another account in the same channel,
was told and drew it blue, and red when it was painted again; and the
first, coming back, had its lamp and its colour from the server. **The
deploy**: nobody connected by the VM's journal (the last had left at
05:50 UTC); the running binary kept as `roundtown-server.prev-2026-11-22b`;
up at 07:08:18 UTC on 2026-11-22; this PC's own account in, its clock set
and its bag as the server has it (a lamp and a car, nothing standing,
nothing painted), with nothing amiss in the VM's journal.

**Not checked.** On a phone: APK 49 was built and not installed. The
buttons by night and on a tablet's shape (looked at by day at 874 by 402
alone). Paint on a door, a window's glass, the couch or a lamp's bulb by
night: the same code, not looked at. Paint against the real database:
I did not paint on Hajun's account; the document it is kept in is the
bag's, which is written there the same way. An app from before against
this server: by the tests alone. iOS: not type-checked, nothing behind
`target_os = "ios"` touched, no tag asked for.

## 2026-11-22, after that: a flicked list goes on, and a swipe changes the tab; APK 48

Hajun, after APK 47:

> also right now, scrolling in the shop/inventory is like as soon as u take
> ur finger out, it stops the scroll. i want it so if its like a strong
> flick scroll then it shouldnt stop scroll right away there should be some
> like velocity and friction slowing it down kind of like if u scroll on
> youtube scrolling or samsung settings page scrolling like if u scroll it
> like with ur finger pressed down or sth it just stops right when u take
> ur finger out but if u like finger flick upward to scroll down then it
> doesnt stop right away but like has velocity that accelerates downward so
> it goes a bit more than the amount u scrolled down so u can like scroll a
> lot with a minimal effort by flicking motion, or if u want precise scroll
> u just scroll with ur finger pressed down sth like that - i dont know how
> to explain it but just do it the way samsung settings page scrolls on
> mobile touch input.

and, while that was being done: "also i want it so u can swipe left/right
to move between inventory/shop tabs." Then "rebuild apk and push".

`src/flick.rs` (new), `src/bag/panel.rs`. Nothing in `shared/` or
`server/`, nor in `ui.ron`: no deploy, protocol 24 still.

**A list is scrolled as a list on an Android phone is** (`src/flick.rs`,
which has the arithmetic and says where each number is Android's):

- It follows the finger. A finger that has stopped lets it go where it is.
- A finger still moving as it comes up throws it: it goes on the way it
  was going, as fast, and slows to a stop. How fast the finger was going
  is read as Android's `VelocityTracker` reads it (a parabola fitted
  through where the finger was over its last tenth of a second; a finger
  still for 40 ms was going nowhere), and how far and how long the list
  then goes on is Android's `OverScroller`'s (`flick::Throw`): 1,500 a
  second goes 393 on in three quarters of a second, 3,000 goes 1,309 in a
  second and a quarter, and 200, a list barely moving as it is let go,
  drifts 12. No slower than 50 a second is a throw, and none is faster
  than 8,000. The lengths are Android's `dp`, which a logical pixel of the
  window is on a phone, so the same flick goes as far here as there: on a
  phone the shop's whole list is 520 to 600 of them, by the phone, which
  one good flick covers.
- A finger put on a list that is going stops it there, and is no tap on
  the card under it.
- At its end a list stops.

**A swipe across the window goes to its other tab** (`panel::swiped`): a
finger going left brings the shop, going right what you own, from
anywhere on the window (the cards, the categories, the tabs). It is a
swipe if it was flicked (25 across and going 400 a second as it came up,
Android's own for the pages of a `ViewPager`) or dragged 64 across however
slowly; not if it was going back the other way as it came up. The tab
changes as the finger comes up, as by a tap on that tab.

**A press goes one way** (`flick::way`, `panel::Pressed`): nothing until
it has moved 8, further than a touch does; then across, if it has gone
twice as far across as up or down, and along the list otherwise; and it
keeps to that. A swipe scrolls nothing and a scroll changes no tab. So a
list no longer moves with the first dot of a drag: it takes the finger up
after those 8, as Android's does, and a press that has gone a way is no
tap.

**Calls that were mine.**

- The numbers are stock Android's, written out from what I know of its
  `OverScroller`, `ViewConfiguration` and `VelocityTracker`, with no
  Android source or phone here to hold them against but one figure a test
  has (a list let go at 4,000 dots a second on a three-dot screen goes
  961 dots in 686 ms). Samsung's settings page is an Android list;
  whether Samsung has tuned its own differently I do not know.
- At its end a list stops dead: nothing stretches or bounces, as a
  Samsung list does a little.
- A second flick does not add to the first: each is its own.
- The pages do not slide with the finger in a swipe: the tab changes when
  the finger comes up. The 64 for a slow swipe is mine (a centimetre).
- A swipe begins the other tab's list at its top, as a tap on the tab
  does.
- Only the bag's window. The friends' lists, the chat and House Builder's
  shop stop where they are let go, as they did: `flick` is there for them
  to take up if Hajun wants it everywhere.

**Checked.** The game's 519 tests (eight new: a throw as far and as long
as Android's, its curve, how fast a finger was going from where it was,
a finger that stopped, which way a press goes, a list carried to its end
and no further, which tab a swipe goes to, what is a tap). The desktop
build run hidden and offline at 874 by 402, at about sixty frames a
second, with fingers made by a script: a flick of 1,500 a second carried
the shop's list 392.9 on in 0.750 s, where the arithmetic says 392.9 in
0.748; one of 3,000 the other way ran it to its top, where it stopped;
one of 200 drifted 11.9; a drag held and let go left it where it was (92
for a drag of 100); a finger put on it as it went stopped it there and
picked nothing, and the next tap picked the card. Swipes: a
flick left to the shop and right back; nothing further right; 80 slowly
left, to the shop; 30 slowly right, nothing; a flick across one card in
the shop, nothing picked; from the categories and from the tabs' row; a
slanting flick down the list, no tab changed. The same with the mouse
held down: a swipe, a flick that went on, a slow drag, a click on a card.

**Not checked.** On a phone: APK 48 was built and not installed, none
being attached, so a real finger has not flicked it. A real finger's
touches come unevenly, where the script's are exact. Against a server:
nothing it has a say in changed. iOS: not type-checked, nothing behind
`target_os = "ios"` touched, no tag asked for.

## 2026-11-22, afternoon: your coins and gems one size, a red price, the box on the button; APK 47

Hajun, after APK 46:

> right now, the gems icon is slightly larger than the coin icon, and also
> the coin number text is larger than the gem text. make the coin number
> text same size as the gem text size. and slightly decrease the gem icon
> size to match the coin icon size. also in the shop, the coin price number
> text is white right now, but it should be red if u cant afford it rn, and
> white if u can afford it. also i dont like the bag icon for that
> inventory panel. just reuse the box icon u used for the inventory tab as
> the ui button for opening the inventory/shop panel. then rebuild apk and
> push

`src/hud.rs`, `src/bag/panel.rs`, `assets/ui.ron`, `src/ui.rs`. Nothing in
`shared/` or `server/`: no deploy, protocol 24 still.

- **Your coins' number is the gems' size.** One number for both now,
  `hud::BALANCE_TEXT`, 3.8 (the coins' was 5.0; the gems' was 3.8, three
  quarters of it, as drawn in `bagandinventorypanel1.png`).
- **The gem is as wide as the coin.** Both pictures were drawn five shares
  wide, but the coin's (`coin.png`, 512 across) has clear round the coin,
  which is 459 of it, and the gem's (`gems.png`, 288) is nearly all gem,
  281: so the gem came out 4.88 wide to the coin's 4.48, a twelfth wider.
  The gem's picture is drawn `hud::GEM_WIDE` wide now, 4.59 (the coin's
  five by 459/512 over 281/288), in the middle of the five the coin's
  has, so that the gem is under the coin's middle and its number begins
  where the coin's does. The coin is as it was. A test reads the two
  pictures and says so if either is drawn again with other margins
  (`COIN_FILLS`, `GEM_FILLS`).
- **A price you cannot afford is red.** In the shop the number under a
  thing's name is `colours.bag.price` with the coins for it and
  `colours.bag.price_cannot_afford` without (`panel::price_ink`), to the
  coin, and is drawn again as your coins change with the window up (the
  window always was). Under a thing you own too.
- **The box is on the bag's button.** `hud::cube`, the very picture the
  window's first tab has, in place of the sack (`hud::bag`), which is
  kept: `hud::SACK` puts it back.

**Calls that were mine.**

- The gem is matched to the coin by width: they stand one over the other,
  and it is their sides that showed the difference. The gem is a wide
  shape, so it comes out a little shorter than the coin (23 dots to 26 at
  603 down).
- The red, `#ff3b30`: the one a count that has run out already has
  (Bridge Wars' metres of bridge, `arena.bridge_out`). A Mythic thing's
  name is orange-red by Hajun's colours (`#ff4500`), so on the car, which
  is the one Mythic thing, a red price is nearly its name's colour.
- A price is red by your coins alone: not while a purchase is at the
  server, when Buy is greyed for a moment. The number on the Buy button
  is as it was, greyed with its button.

**Checked.** The game's 511 tests (two new: which colour a price has for
which coins, for all twenty things; the two pictures measured). The
desktop build run hidden and offline: at 874 by 402 by day, the top left
and the buttons round the jump button looked at four times enlarged and
measured beside the run before (coin 26 by 26 dots, gem 26 by 23 where it
was 28 by 25, the same middle); the shop with 300 coins (2,500 and 400
red, 150 and 50 white), with none (all red) and with 20,000 (all white),
the list scrolled meanwhile and staying where it was; the car with 14,999
(red), with 15,000 (white, Buy lit) and picked with 100 (red, Buy
greyed); the same by night; and at 1024 by 768.

**Not checked.** On a phone: APK 47 was built and not installed, none
being attached. Against a server: nothing it has a say in changed. iOS:
not type-checked, nothing behind `target_os = "ios"` touched, no tag asked
for.

## 2026-11-22, later: the shop after Hajun had looked at it; APK 46

Hajun, the same day, having looked at the window of the night before:

> in the item shop, when u press sth, it should always show both buy and
> sell button even if u dont have the item, the sell button should just be
> greyed out if there isnt any. also for the shop, the buy price should be
> shown under the name of the item as (coin_icon + #,###) also i think the
> default section for the shop should be the All rather than owned (keep
> order the same tho) and also i dont like the fact that i need to scroll
> the sections to see the cosmetics section. try to fit it by making the
> panel tall enough so no scrolling is necessary. also i dont want the item
> images to disappear when u press sth in the shop. and the buy/sell button
> should be slightly transparent too like my design image, and should be
> able to see the item image behind the button still kind of. then rebuild
> apk

All of it is the window's (`src/bag/panel.rs`) and the file's
(`assets/ui.ron`, `src/ui.rs`). Nothing in `shared/` or `server/` changed:
no deploy, protocol 24 still, and what the entry below says of the window
stands but where this one says otherwise.

- **Both buttons, always.** A thing pressed in the shop has Buy and Sell
  on its card whether or not you have one. Sell is greyed with none of it
  in your bag (so too for one that stands on your plot, which is out of
  the bag), Buy without the coins for it ("Not enough coins." at a tap, as
  before; a tap on a greyed Sell does nothing), and both while a purchase
  or a sale is at the server (`can_deal`). One that is greyed is a grey of
  its own with faint words and a dimmed coin (`colours.bag.deal_off`,
  `on_deal_off`), not a window's `Look::Greyed`, whose dark box would be
  lost over a picture.
- **The picture stays.** The two buttons are laid over the card's picture
  (a layer the size of the card with the two in its middle; a press on
  the card beside them still puts them away), no longer in its place. I
  had read the design wrongly the night before: in
  `bagandinventorypanel2.png` the lamp's post shows between the two
  buttons and, faintly, through the red one.
- **A little see-through.** `colours.bag.buy`, `buy_lit`, `sell` and
  `sell_lit` have an alpha now, `d0` (82 parts in a hundred solid). The
  design's own red is about nine tenths solid (the post is 5 of 255
  brighter through it where it is 47 brighter in the open) and its green
  solid; `d0` is my number, for the picture to be seen "kind of" through
  both. The colours were made brighter by what the alpha takes off, so
  that over a card's own dark they come out as the design's: read off the
  run, the green (30, 199, 34) for the design's (24, 199, 30), the red
  (195, 67, 65) for (195, 65, 64). Bevy mixes in linear light, so a light
  part of a picture shows through more than a dark one: a bed's sheets
  plainly, the lamp's thin post barely.
- **The price under the name**, in the shop alone: the coin and the number
  as the balance writes it ("2,500", `hud::money`), in
  `colours.bag.price`. A cell of the shop is that much taller (`NAME_LINE`
  4.2 and `PRICE_ROW` 5.3, for the inventory's `NAME_ROW` 6.1).
- **The shop opens on All**, Owned still the first of its categories
  (`Panel::default`). Each tab still keeps the category it was left on
  while the app runs.
- **No scrolling to Cosmetics.** The window is 76 tall for 70 (`HEIGHT`):
  the six categories' 62.4 under the tabs, with 2.2 to spare, and nothing
  else moved. The categories can still be scrolled if there are ever more
  of them. Two rows of the shop's cards with their names and prices show
  whole (59 of 61.1), and a third begins.

**Calls that were mine**: `d0`, and the colours brightened to come out as
the design's; the grey of a button that is off; white for the price, at
the name's size; the price under things you own too (it is what another
costs); the window made taller, as he said, and by no more than the six
take, rather than the rows shorter; nothing said at a tap on a greyed
Sell.

**Checked.** The game's 509 tests (the window's among them: six categories
whole and the window no taller than they take, two rows with prices
whole, the shop opening on All, `can_deal`, the buttons' alpha). Hidden
runs of the desktop build, offline, scripted by the throwaway driver,
which is out again: at 874 by 402 the inventory; the shop as it opens
(All, six categories, a price under every name); the lamp pressed with
one in the bag and 5,000 coins (both live); the bed pressed (Buy live,
Sell grey); a bed bought with 200 coins (50 left, "x1", Sell live, Buy
grey) and sold again (125, both grey); Buy pressed without the coins
("Not enough coins."); Owned; the list dragged to its end (15,000 under
the car's name). At 1024 by 768, a tablet's shape, the same, six
categories whole. At night. The buttons were looked at three times their
size, and their colours read off the picture.

**Not checked**: on a phone (none attached). Online: the window asks the
server as it did, by the same `Deal`, and no run was made against one.

## 2026-11-22: the bag, its window, gems, and buildings on your plot that everyone sees; APK 45

Asked late on 2026-11-21 and built through the night. Hajun, with two
pictures in the repo's root (`bagandinventorypanel1.png`, `2`), a gem's
picture (`assets/gems.png`) and a list (`assets/itemrarities.txt`):

> im gonna change the emote button in the lobby to a inventory/shop panel.
> the design is in the root folder as bagandinventorypanel1~2.png. the first
> tab is a inventory tab and the second tab is the shop tab. and on the
> left, you can select the categories (scrollable if long), and on the right
> side u see the items in rows of 3 (scrollable). in the inventory tab, when
> u click on a building item, (if the player is standing on their own plot)
> it should close the panel and have the building item in front of the
> character in the map similar to how the building placing thing works in
> the house builder game with the rotate buttons on the left and right and
> trash in the top, and check/apply in the bottom. and when u place it in
> your plot, that item is gone from ur inventory until u click that building
> in the map and trash it, then it come back to ur inventory. also the
> placed inventory, after pressing the check button, gets placed in ur plot
> and saved so each time u reopen the app it stays there and other ppl can
> see that item in ur plot in that exact location too. for the vehicle
> section in ur inventory, u tap your car and u go in to the car riding
> mode, and when ur in the car riding mode, a new ui button around left side
> center of your screen that is like a leave door icon and u press that to
> get off ur car. for the shop tab, when u tap an item in the owned
> category, it shows a buy button and a sell button. also if u see in the
> images, u can see the names are colored, they indicate the rarity. rarity
> is tied to the color, but is not tied to the price. but for now, set the
> price for each rarity items to this (also this hex color): Mythic FF4500
> $15,000, Legendary FFA500 $2,500, Epic 800080 $1,000, Rare 4169E1 $400,
> Uncommon 32CD32 $150, Common D3D3D3 $50. also the rarities for now are
> listed in itemrarities.txt. also if u notice from the
> bagandinventorypanel1~2.png, i added gems below the coin. add it as
> another thing. the gems image is in the asset folder too. and make the
> default gem count to be 10. also for now, make every player own 1 car and
> 1 street lamp by default. sell price should be 50% of buy price. then
> build apk and push to git

### What there is to own, and what it costs

- **Twenty things** (`roundtown_net::bag::THINGS`, in the order of Hajun's
  list, which is the order the shop shows them in): the nineteen pieces
  House Builder's shop sells, each by its name there (`build::PIECES`), and
  the car you drive (`red_sports_car`, `car1.glb`). A thing has a group
  (Buildings, Vehicles; nothing is a Utility or a Cosmetic yet), a rarity,
  and a price of its own, which for now is its rarity's
  (`Rarity::price`): Common 50, Uncommon 150, Rare 400, Epic 1,000,
  Legendary 2,500, Mythic 15,000. Sold back it fetches half
  (`Thing::resale`). The rarities are the list's: the Street Lamp
  Legendary, the Flat House Epic, the Double Bed and the Soft Couch Rare,
  the Single Bed, the Two Floor House and the Roof Terrace House Uncommon,
  the car Mythic, the other twelve Common.
- **A bag** (`bag::Bag`) is one person's: their gems, how many of each
  thing they own, and which of those stand on their plot and where.
  **Everyone begins with ten gems, one car and one street lamp**
  (`Bag::new`). Nothing earns or spends a gem yet: it is a number under
  your coins.
- **A thing that stands on the plot is out of the bag** (`Bag::free`): the
  window shows what is in the bag, and only that is sold or stood. At most
  99 of a kind, and 100 things on a plot (`MOST_OF_A_KIND`,
  `MOST_STANDING`: a plot is told of in one message, a few kilobytes at
  the most).
- The rules are in `shared/` for both sides: the server plays by them with
  the account's coins, the game by itself offline with the coins the
  device has.

### The window

- **The face among the buttons round the jump button is a bag**
  (`hud::BagButton`, the sack drawn in code for Bridge Wars' first sketch
  and unused since), in the town alone. It opens the window on what you
  own each time; each tab keeps the category it was on.
- **Laid out off the two pictures** (874 by 402; a share of the short side
  is four of their dots; `bag/panel.rs`'s constants): 130 by 70, the two
  tabs along the top from where the cards begin, a packed box seen from a
  corner for what you own and a cart for the shop (`hud::cube`,
  `hud::cart`, drawn in code), the one shown white over a white line; the
  bare white cross in the corner; down the left the categories, 10.4 tall
  with a line between each, the one shown white (All, Buildings, Vehicles,
  Utilities, Cosmetics, and in the shop Owned before them, the one the
  shop opens on, as drawn); on the right the cards, three to a row, 30 by
  20, each with its picture and under it its name in its rarity's colour
  and how many are in your bag ("x1"). Both lists scroll, by a drag or a
  mouse's wheel, and are drawn again where they were. On a screen as broad
  as a tablet's the window is no wider than 93 parts in a hundred of it.
  It is a window as every window is (`window`): the panel's and the cards'
  colours are `colours.window`'s.
- **The pictures** are the pieces' own, the ones House Builder's shop
  shows (`mybuilds/<kind>.png`); the car had none, and was rendered as
  they are (`blender/render_thing.py`, from a copy of `car1.blend`, the
  file itself untouched; `assets/car1.png`).
- **What you own**: a tap on a building, standing on your own plot, shuts
  the window and brings the building up in front of you (below); anywhere
  else the window says why not, along its top between the tabs and the
  cross ("Stand on your land to put this down.", "One house to a plot.",
  "Put a house down first."). A tap on your car shuts the window and puts
  you in it.
- **The shop**: a tap on a thing puts two buttons on its card in place of
  its picture: Buy, with the coin and its price, in the green of the
  picture, greyed without the coins for it ("Not enough coins." at a tap);
  and Sell, with half that, in the picture's red, there while one is in
  your bag. So in Owned both are there, as asked, and elsewhere Sell is
  there for what you have.
- **Gems** are under your coins on the bar (`hud::gems`, `GemsText`): the
  gem as wide as the coin, the number smaller, as drawn.
- Every word and colour of it is `assets/ui.ron`'s (`words.bag`, with each
  thing's name; `colours.bag`, the six rarities' colours as given, the
  count's, and the two buttons').

### Putting a building down on your plot

- **House Builder's own way of putting a piece down** (`build.rs`), which
  until now worked on a plot of that game alone. Where you are building is
  now a `build::Site`: that game's plot, or, in the town, your own plot of
  it as a square (`build::Yard`, from `land::Plot`). The same ghost, the
  same menu (bin over it, a turn either side, the tick under it), the same
  rules: a house where the land is level, one to a plot; a door or a
  window into a wall of the house, cutting its hole; furniture carried
  over the floor, snapping into line; doors that swing open; walls that go
  see-through round you inside.
- **Nothing goes off your plot**: a piece is dragged and turned only as
  far as the whole of it stays inside the square (`Yard::holds_piece`,
  `pull_in`), comes up inside it, and is not put down outside it
  (`build::Misplaced`: its menu comes back, and "It has to stand on your
  land." across the screen). A house besides stands only where nothing
  else does: the plot's sign, or what is down already (`house_fits_in`).
- **The tick puts it down and takes it out of your bag**
  (`stand::report`, `bag::Bag::put`); tapped again it is picked back up, to
  move, or by the bin to take back into the bag (`stand::note_edits`,
  `Bag::take`). A house taken back takes what was in its walls with it.
- **Where it stands is kept in the plot's own space**
  (`bag::Standing`, `Yard::place`: the plot's middle, its front, its
  ground), so that it stands the same on whichever plot your seat in the
  town has that day: seats are given as people come in, and a plot goes
  with the seat (`docs/land.md`).
- **It is part of the town's shape**: stood on and bumped into by you, the
  town's people and a car, as House Builder's pieces are on their plots.
  The town is too big to read again for every chair, so what stands on
  its plots is laid over it, a layer beside the plots' signs
  (`Islands::lay_layer`).
- **Everyone else's in your channel stands on their plots**
  (`stand::keep_theirs`), from what the server says of each
  (`bag::Theirs`) and the seat each has (`lobby::Channel`): drawn, solid,
  their doors opening, and never to be tapped.

### Your car

- **In the bag** (Vehicles): tap it and you are in it, as the face button
  put you (`drive::CarPressed`). **Out of it by a door** half way down the
  left edge of the screen (`hud::ExitButton`, Material's `logout`, which
  the friends' panel has for leaving a group), there while you drive. C on
  desktop still gets in and out.
- Into it only with one in your bag (`Bag::has_car`): sold, there is no
  getting in, and the server tells nobody you are in one
  (`players::hear`, `Ask::Drive`).

### Whose word it is, and where it is kept

- **Online the bag is the account's.** The server holds it on your body
  (`bag::Holds`), read with the account as you are let in, and tells your
  device the whole of it as you sit down in the town and once in answer to
  everything asked of it (`bag::Tell::Yours`), numbered, so that an answer
  older than what the device has done since is passed over
  (`Bag::heard`). A purchase or a sale waits for it (the two buttons grey
  until it answers); standing a thing or taking it back is done on the
  device at once and told, and the answer puts right whatever the server
  would not have.
- **In the database**: `players.gems` (ten for every account, new or
  old) and `players.bag`, one JSON document of what is owned and what
  stands where (none until an account first changes its bag). A purchase
  or a sale changes the coins and the bag in one transaction, with its
  line in `coin_changes` ("bought a thing", "sold a thing";
  `Bank::deal`); while one is on its way, whatever else that person asks
  of their bag waits its turn (`Holds::waiting`).
- **What stands on a plot is told to its channel**
  (`bag::Tell::Theirs`): to whoever sits down, of everyone there with
  anything standing, and to everyone there of the newcomer's and of every
  change (`bag::seat_news`, `Towns::take_sat`).
- **No new protocol**: `Ask::Bag` and `Tell::Bag` are new variants at the
  end, said only by and to a device that asked (`bag::Ask::Watch`), as
  `Ask::Channel` was (`docs/online.md`). `PROTOCOL_VERSION` is 24 still:
  APK 43 and 44 play on, with the face button putting them in the car as
  before.
- **On the device** the bag as last known is kept in `bag.json`, beside
  the profile, so that what stands on your plot stands there as the app
  opens, before any server has answered. **Offline it is the device's
  own**, bought with the coins the device has (a thousand each time the
  app opens, as ever offline); whatever was done offline gives way to the
  server's word as soon as that comes. An account deleted in the settings
  starts the bag afresh.

### Calls that were mine

- **Opening on what you own**, whichever tab was shut; the shop opening on
  Owned, as its picture has it.
- **The shop's cards outside Owned**: Buy on every one, Sell beside it for
  what is in your bag; the count after a name wherever you have one.
- **The bag is put away while you drive**, and the door is the way out.
- **One house to a plot**, as in House Builder, though more than one may be
  owned: a second stays in the bag.
- **A thing that stands is not sold**: it has to be taken back first.
- **The name "Red Sports Car"** (the list has "Red sports car"), and
  "Street Lamp", as House Builder's shop and the list have it (the picture
  says "Lampost").
- **The window's own dark** is every window's (`colours.window.panel`),
  lighter on the screen than the picture's, as the friends' panel is
  lighter than its own sketch: one colour for every window is the rule.
  The Buy and Sell buttons are the picture's own colours, since its red is
  not a window's dark red for "no".
- **Where a notice goes** (the top, beside the tabs: along the bottom it
  lay over the second row's names), 99 of a kind, 100 things to a plot,
  the door's size, the gem count's size (three quarters of the coins', as
  measured).
- **A tap from your car picks nothing up** (`stand::hands_off_in_the_car`):
  driving across your plot, a stray tap would have your house in your
  hand.
- **No server-side rule for what goes where** beyond a place being on the
  plot and the thing being yours: one house, doors in walls and the whole
  of a piece inside the plot are the device's to keep, as House Builder's
  are. A device that broke them could only spoil its own plot.
- `assets/itemrarities.txt` is left out of the APK: the game reads nothing
  from it.

### Checked, and where

- **Unit tests**: the rules (`roundtown_net`, 258: prices and rarities as
  the list has them, the beginning bag, buying and selling, standing and
  taking back, a bag kept wrongly put right, both new variants last on the
  wire); the server (115: a bag kept and read back, who is told what as
  they sit down, a purchase waiting on the database with what is asked
  meanwhile done in its turn, a plot told to its channel and to nobody who
  did not ask); the game (508: the bag's asks and answers, the window's
  sizes against the pictures, what each tab shows, every thing's name,
  picture and colour, the plot's space, a piece inside the plot, the bag's
  button and the car's door).
- **The database's new statements against Postgres on the VM**, on two
  scratch databases dropped after: an empty one (everything written and
  read back, gems, a bag, a purchase, a sale, one turned down), and a copy
  of the live one taken through starting up (3 accounts, each with 10
  gems, no bag, nothing else of them changed).
- **The desktop build, hidden** (`RT_BACKGROUND=1`), driven by a throwaway
  module with touches and the mouse, taken out again, pictures looked at
  beside Hajun's two: the bar with the gems; the window on each tab; the
  lamp out of the bag, turned, put down, picked up, taken back; the car
  and its door; a bed bought twice and sold once offline; the cards
  scrolled; a house, a door and a window in its wall, a bed dragged
  against each edge of the plot (it stopped with its side on the edge, to
  the centimetre); the house taken back with its door and window; the
  notice off your plot; a start with the bag kept from the run before
  (what stood, stood); a bed stood where you arrive (you are put beside
  it); a lamp and a bed walked at (solid); the door swinging open and
  walked through; a tap on the house from the car (nothing); a window
  shaped as a tablet.
- **Two hidden copies against a server on this PC** (no database): each
  saw the other's lamp on the other's plot; a house and a door bought in
  the shop (1,000 to 900 coins, the server's count), put down, and seen by
  the other copy standing on the builder's plot; taken back, gone from the
  other's town; the copy that left, its things gone from the one that
  stayed; back again, the bag as the server had it; in the second seat,
  the lamp standing on that seat's plot in the same place of it.
- **The VM after the deploy** (15:59 UTC on 2026-11-21, 00:59 on the 22nd
  here; nobody connected since 14:20 UTC; the database dumped first to
  `/var/backups/roundtown/before-bags-2026-11-22.dump`, the binary before
  kept as `roundtown-server.prev-2026-11-22`): the VM's binary the same
  bytes as the build here, the three accounts with 10 gems each, and this
  PC's own account online on it, hidden: its bag told, the lamp stood on
  its plot and taken back, both kept in the database, its coins untouched,
  the journal clean.
- **APK 45** (`docs/building.md`).

### Not checked

- **Anything on a phone**: none was attached. The window under a thumb,
  its lists dragged; a piece dragged on a touch screen in the town (the
  menu and the drag are House Builder's, which has been played on phones);
  the door's place beside an iPhone's island (it keeps clear as the bar
  does, `safe::KeepsClear`).
- **A purchase on the live database** by a real account (the statements
  were run on the scratch copies; the live check stood a lamp and took it
  back, which spends nothing).
- **Two real accounts on the VM**; more than two people's things in one
  channel; a plot as full as it may be.
- **An app from before against the new server on a phone**: by the code and
  the server's tests it is sent nothing new and drives as it did.
- **The edit menu under the town's buttons**: at the right edge of the
  screen its tick can lie under the bag or the fist, and the town's button
  takes the press, as in House Builder; turn the camera, or drag the
  piece.
- **Epic's purple on the dark window** is hard to read (`#800080`, as
  given; the picture's "Soft Couch" is the same). One line of `ui.ron`
  changes it.
