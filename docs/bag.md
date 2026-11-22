# The bag: what you own, the shop, gems, and what stands on your plot

Your bag (`src/bag.rs`), its window (`src/bag/panel.rs`), the rules of it
that both sides play by (`roundtown_net::bag`), the server's side
(`server/src/bag.rs`), and the buildings put down on a plot of the town
(`src/build/stand.rs`). Dated entries, newest first.

## Contents

1. 2026-11-22, afternoon: the coins' number the gems' size and the gem the coin's; a price you cannot afford in red; the box on the bag's button; APK 47
2. 2026-11-22, later: the shop after Hajun had looked at it (both buttons always, over the picture and see-through; the price under the name; All first shown; six categories with no scrolling); APK 46
3. 2026-11-22: the bag, its window, gems, and buildings on your plot that everyone sees; APK 45

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
