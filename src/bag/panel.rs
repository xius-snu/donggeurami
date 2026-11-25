//! The bag's window, behind the bag's button among those round the jump
//! button (`hud::BagButton`), as Hajun drew it (`bagandinventorypanel1.png`
//! and `2`, 2026-11-21: "the first tab is a inventory tab and the second tab
//! is the shop tab. and on the left, you can select the categories
//! (scrollable if long), and on the right side u see the items in rows of 3
//! (scrollable)"). Only in the town, where your plot and your car are.
//!
//! It is a window as every window is (`window`): dark and see-through, its
//! two tabs along its top, a box seen from a corner for what you own and a
//! cart for the shop, the one shown white over a white line, and a bare
//! white cross in its corner. Down its left the categories, a line between
//! each, the one shown white: All, Buildings, Vehicles, Utilities and
//! Cosmetics, and in the shop Owned before them; each tab opens on All
//! (the shop's too, Owned still its first: Hajun, 2026-11-22, "the default
//! section for the shop should be the All rather than owned (keep order the
//! same tho)"). The window is tall enough for all six to show ("i dont like
//! the fact that i need to scroll the sections to see the cosmetics
//! section"). On its right the things, three to a row, each a dark card
//! with its picture and under it its name, in the colour of its rarity
//! (`colours.bag`), and how many of it are in your bag ("x1"). They come
//! kind by kind, furniture first, then the houses, the doors and windows
//! and the cars, and of each kind by rarity: in the shop the commonest
//! first, of what you own the rarest (since 2026-11-25, [`shown`]).
//!
//! **What you own** shows what is in your bag: what you own that does not
//! stand on your plot. A tap on a building, standing on your own plot,
//! shuts the window and brings the building up in front of you to be put
//! down (`build::stand`); anywhere else, the window says why not along its
//! top. A tap on one of your cars shuts the window and puts you in that one
//! (`drive`).
//!
//! **A thing sold for gems** (since 2026-11-25: the houses of five, seven
//! and nine floors, as Hajun's list has them) has the gem where another
//! has the coin, under its name and on its two buttons, and is red, and
//! its Buy greyed, without the gems for it ("Not enough gems."). Every
//! car's picture is the pack's own of it (`drive::CARS`).
//!
//! **The shop** shows everything there is, or what of it you own, and
//! under each thing's name the coin and what it costs, white with the
//! coins for it and red without ("it should be red if u cant afford it rn,
//! and white if u can afford it"). A tap on a thing
//! puts two buttons over its picture, a little see-through, the picture
//! still there behind them (Hajun: "i dont want the item images to
//! disappear when u press sth in the shop", and the buttons "slightly
//! transparent too like my design image"): Buy, in green, with the coin and
//! its price; and Sell, in red, with half that. Both are always there
//! ("even if u dont have the item, the sell button should just be greyed
//! out if there isnt any"): Sell is greyed with none of the thing in your
//! bag, Buy without the coins for it. Each is asked of `bag` ([`Deal`]):
//! online the server's to do, and both buttons greyed until it has
//! answered.
//!
//! The sizes are measured off the two pictures, 874 by 402, as shares of
//! the short side of the screen (four of their pixels to a share), but its
//! height, which is the six categories'; and every colour and word is
//! `assets/ui.ron`'s. The window is drawn again whenever what it shows
//! changes, each list where it was scrolled to.
//!
//! **A list is scrolled as a list on the phone is** (Hajun, 2026-11-22: "do
//! it the way samsung settings page scrolls on mobile touch input"): it
//! follows a finger dragged along it, stops where a finger that has
//! stopped lets it go, and flicked, goes on by itself and slows to a stop
//! (`flick`, which has Android's own numbers for it). A finger put on a
//! list that is going stops it, and is no tap on what is under it. **A
//! swipe across the window goes to its other tab** ("i want it so u can
//! swipe left/right to move between inventory/shop tabs"): a finger going
//! left brings the shop, going right what you own, as the tab is changed
//! when the finger comes up ([`swiped`]). A press goes one way or the
//! other once it has moved further than a touch does, and keeps to it
//! (`flick::way`): a swipe scrolls nothing, and a scroll changes no tab.

use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use super::{Bag, Deal, Group, THINGS, Thing};
use roundtown_net::bag::Money;
use crate::build::stand::{Home, Stand, Why};
use crate::build::{PIECES, Stock, Tab as BuildTab};
use crate::builder::{self, DIALOG_LAYER, Fill};
use crate::drive::{CARS, CarPressed, Driving, car_def};
use crate::flick::{self, Coasting, Throw, Track, Way};
use crate::hud::{self, BagButton, DialogUp, TakesPress};
use crate::island::Venue;
use crate::lobby::Balance;
use crate::ui::{BagColours, UiChanged, fill, ui};
use crate::window;
use crate::{CloseUp, Player};

/// The window: how wide and how tall it is, and how much of the screen's
/// width it takes at the most, in parts of a hundred, on a screen as broad
/// as a tablet's, where the first is more than there is. Taller than it was
/// drawn, by six shares: tall enough for the shop's six categories to show
/// with none scrolled to (Hajun, 2026-11-22).
const WIDTH: f32 = 130.0;
const HEIGHT: f32 = 76.0;
const MOST_WIDE: f32 = 93.0;
/// The room over the tabs, and between the line under them and the
/// categories; how far its cross's middle is in from its right and down
/// from its top.
const OVER_TABS: f32 = 1.5;
const UNDER_TABS: f32 = 1.7;
const CROSS_IN: f32 = 5.0;
const CROSS_DOWN: f32 = 4.8;
/// The gap between its two tabs.
const TAB_GAP: f32 = 3.2;
/// The categories down the left: how wide their column is, how tall each
/// is, its words, and the line between two of them.
const SORTS_WIDE: f32 = 15.3;
const SORT_ROW: f32 = 10.4;
const SORT_TEXT: f32 = 3.0;
const SORT_LINE: f32 = 0.25;
/// The room between the categories and the cards, and right of the cards.
const BETWEEN: f32 = 4.5;
const RIGHT: f32 = 8.0;
/// A card: how far the first row is under the categories' top, how tall it
/// is, the gap between two of them across, the room under it for its name,
/// the name's size, and how much of the card's height its picture takes.
const OVER_CARDS: f32 = 3.5;
const CARD_HEIGHT: f32 = 20.0;
const CARD_GAP: f32 = 4.2;
const NAME_ROW: f32 = 6.1;
const NAME_TEXT: f32 = 2.7;
const PICTURE: f32 = 88.0;
/// In the shop, where what a thing costs is under its name: how tall the
/// name's own line is there, the row of the price with the room under it,
/// the number's size, the coin's, how far down the coin is put to stand
/// level with the number, and the gap between the two.
const NAME_LINE: f32 = 4.2;
const PRICE_ROW: f32 = 5.3;
const PRICE_TEXT: f32 = 2.7;
const PRICE_COIN: f32 = 2.7;
const PRICE_COIN_DOWN: f32 = 0.3;
const PRICE_GAP: f32 = 0.7;
/// The two buttons over a card's picture in the shop: how wide and how
/// tall each is, the gap between them, their words, and the coin beside
/// the price.
const DEAL_WIDE: f32 = 22.6;
const DEAL_HIGH: f32 = 6.0;
const DEAL_GAP: f32 = 1.7;
const DEAL_TEXT: f32 = 2.6;
const DEAL_COIN: f32 = 2.8;
/// How far a press may be dragged and still be a tap, in logical pixels,
/// rather than a drag to scroll a list; and how long the line along the
/// bottom stays up, in seconds.
const TAP_SLOP: f32 = 16.0;
const NOTICE_SECS: f32 = 3.0;
/// A swipe across the window, to its other tab, in logical pixels (a
/// phone's dp): how far across a flick has to have gone, and how fast it
/// has to be going as it comes up, a second, which are Android's own for
/// the pages of a `ViewPager`; and how far across is a swipe however
/// slowly it went (mine: a centimetre).
const SWIPE_FLICKED: f32 = 25.0;
const SWIPE_SPEED: f32 = 400.0;
const SWIPE_FAR: f32 = 64.0;
/// The line that says why something cannot be done, along the window's top
/// between its tabs and its cross: its words, and the room either side of
/// it.
const NOTICE_TEXT: f32 = 2.6;
const NOTICE_GAP: f32 = 2.0;

/// The window's two tabs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Tab {
    /// What you own.
    #[default]
    Inventory,
    Shop,
}

/// A category down the window's left.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sort {
    /// What of the shop's you own: the shop's alone, and its first.
    Owned,
    All,
    Of(Group),
}

impl Sort {
    /// The categories of `tab`, top to bottom.
    fn of(tab: Tab) -> Vec<Sort> {
        let owned = (tab == Tab::Shop).then_some(Sort::Owned);
        owned.into_iter().chain([Sort::All]).chain(Group::ALL.map(Sort::Of)).collect()
    }

    fn title(self) -> String {
        let ui = ui();
        let words = &ui.words.bag;
        match self {
            Sort::Owned => &words.owned,
            Sort::All => &words.all,
            Sort::Of(Group::Buildings) => &words.buildings,
            Sort::Of(Group::Vehicles) => &words.vehicles,
            Sort::Of(Group::Utilities) => &words.utilities,
            Sort::Of(Group::Cosmetics) => &words.cosmetics,
        }
        .clone()
    }
}

/// What kind of thing something is, in the order the window shows the
/// kinds in (Hajun, 2026-11-25: "grouped by category first like furnitures
/// together, foundation (house models) together, cars together ... for
/// "All" furniture should come first", and what you own "same category
/// order (furniture, then house models, then cars)"): furniture, the
/// painting and the street lamp among it; the houses, House Builder's
/// Foundation; the doors and windows that go in them, after them as in
/// House Builder's shop (mine: his list does not say); and the cars.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kind {
    Furniture,
    Houses,
    Openings,
    Cars,
    Other,
}

impl Kind {
    fn of(thing: &Thing) -> Self {
        match thing.group {
            Group::Vehicles => Kind::Cars,
            Group::Buildings => match PIECES.iter().find(|def| def.kind == thing.kind).map(|def| def.tab) {
                Some(BuildTab::Foundation) => Kind::Houses,
                Some(BuildTab::Openings) => Kind::Openings,
                _ => Kind::Furniture,
            },
            Group::Utilities | Group::Cosmetics => Kind::Other,
        }
    }
}

/// The things `tab` shows under `sort`, each with how many of it are in
/// the bag `has`. What you own shows only what is in your bag, and so does
/// the shop's Owned; the rest of the shop shows everything of its category.
/// Kind by kind ([`Kind`]), and of each kind by rarity: in the shop the
/// commonest first, and of what you own the rarest (Hajun: "order of shop
/// items should be order of rarity, with common being first", and of the
/// inventory "high rarity is first so transcendent is first"); things as
/// rare as each other in the order of his list.
fn shown(tab: Tab, sort: Sort, has: &roundtown_net::bag::Bag) -> Vec<(&'static Thing, u16)> {
    let mut shown: Vec<(&'static Thing, u16)> = THINGS
        .iter()
        .map(|thing| (thing, has.free(thing.kind)))
        .filter(|&(thing, free)| match sort {
            Sort::Owned => free > 0,
            Sort::All => true,
            Sort::Of(group) => thing.group == group,
        })
        .filter(|&(_, free)| tab == Tab::Shop || free > 0)
        .collect();
    // Stable: the list's order among those alike.
    shown.sort_by(|(a, _), (b, _)| {
        Kind::of(a).cmp(&Kind::of(b)).then_with(|| match tab {
            Tab::Shop => a.rarity.cmp(&b.rarity),
            Tab::Inventory => b.rarity.cmp(&a.rarity),
        })
    });
    shown
}

/// The window: whether it is up, which tab, which category of each tab,
/// and, in the shop, the thing whose two buttons are up.
#[derive(Resource, Debug)]
struct Panel {
    open: bool,
    tab: Tab,
    sorts: [Sort; 2],
    picked: Option<&'static str>,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            open: false,
            tab: Tab::Inventory,
            // Everything, in the shop too, where Owned comes before it
            // (Hajun, 2026-11-22; Owned until then, as drawn).
            sorts: [Sort::All, Sort::All],
            picked: None,
        }
    }
}

impl Panel {
    fn sort(&self) -> Sort {
        self.sorts[self.tab as usize]
    }
}

/// What a press on something of the window's does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Tap {
    Close,
    Show(Tab),
    Sort(Sort),
    Thing(&'static str),
    Buy(&'static str),
    Sell(&'static str),
    Nothing,
}

/// The presses on the window since they were last acted on ([`act`]); that
/// the bag button was pressed; and the press now down: how far it has been
/// dragged, so that a drag to scroll a list is not also a tap on what it
/// began on; which finger it is, or the mouse; which way it is going, once
/// it has gone far enough to say; for one going along a list, how far it
/// had gone when the list took it up, and how far the list has followed it
/// since; whether it went down on a list that was going on by itself, and
/// stopped it; and where it has been, for how fast it is going as it comes
/// up.
#[derive(Resource, Default)]
struct Pressed {
    taps: Vec<Tap>,
    open: bool,
    dragged: f32,
    pointer: Option<PointerId>,
    way: Way,
    slack: f32,
    followed: f32,
    caught: bool,
    track: Track,
}

impl Pressed {
    /// Which way the press now down is going, now that it is `gone` from
    /// where it went down: said once, and kept to.
    fn feel(&mut self, gone: Vec2) -> Way {
        if self.way == Way::Unsure {
            self.way = flick::way(gone);
            if self.way == Way::Along {
                // The list takes it up from here, as Android's does: it
                // does not jump by what the finger had gone already.
                self.slack = flick::SLOP.copysign(gone.y);
                self.followed = 0.0;
            }
        }
        self.way
    }

    /// Whether the press that has come up was a tap: it went no way, nor
    /// far, and did not go down to stop a list.
    fn tapped(&self) -> bool {
        self.way == Way::Unsure && self.dragged <= TAP_SLOP && !self.caught
    }
}

/// The line along the window's top that says why something cannot be
/// done, and how long it has been up.
#[derive(Resource, Default)]
struct Notice(Option<(String, f32)>);

/// How far each of the window's lists was scrolled, to draw it there
/// again: the categories of each tab, and the cards.
#[derive(Resource, Default)]
struct Scrolled {
    sorts: [f32; 2],
    cards: f32,
}

/// The lists that scroll.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum List {
    Sorts,
    Cards,
}

/// The pictures on the window's two tabs, drawn once (`hud::cube`,
/// `hud::cart`); and every car's, read as the app opens and kept, as every
/// building's is (`build::Stock`), so that a card is never up empty.
#[derive(Resource)]
struct Icons {
    cube: Handle<Image>,
    cart: Handle<Image>,
    cars: Vec<(&'static str, Handle<Image>)>,
}

/// The root of the window, over the whole screen.
#[derive(Component)]
struct Root;

/// The line that says why something cannot be done.
#[derive(Component)]
struct NoticeLine;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Panel>()
        .init_resource::<Pressed>()
        .init_resource::<Notice>()
        .init_resource::<Scrolled>()
        .add_observer(press)
        .add_observer(start_press)
        .add_systems(Startup, draw_icons)
        .add_systems(
            Update,
            // Where each list has got to, by a finger or by itself, is
            // remembered before the window is drawn again there.
            (remember_scroll, act, redraw, show_notice)
                .chain()
                .after(flick::coast)
                .after(super::BagSystems)
                // Before whatever a tap in it sets going is read: the car
                // (`drive::toggle`), and a building brought out
                // (`build::stand`).
                .before(crate::read_player_input)
                .before(crate::build::BuildSystems),
        );
}

fn draw_icons(mut images: ResMut<Assets<Image>>, assets: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Icons {
        cube: images.add(hud::cube()),
        cart: images.add(hud::cart()),
        cars: CARS.iter().map(|car| (car.kind, assets.load(car.picture))).collect(),
    });
}

/// A new press, wherever it lands: nothing dragged yet, and going no way
/// yet. One that goes down on a list that is going on by itself stops it
/// there, and is then no tap on what is under it, as a finger put on a
/// list on the phone is not.
fn start_press(
    press: On<Pointer<Press>>,
    time: Res<Time<Real>>,
    parents: Query<&ChildOf>,
    coasting: Query<(), With<Coasting>>,
    mut pressed: ResMut<Pressed>,
    mut commands: Commands,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    let held = std::iter::successors(Some(press.entity), |&entity| parents.get(entity).ok().map(ChildOf::parent)).find(|&entity| coasting.contains(entity));
    if let Some(list) = held {
        commands.entity(list).try_remove::<Coasting>();
    }
    pressed.dragged = 0.0;
    pressed.pointer = Some(press.pointer_id);
    pressed.way = Way::Unsure;
    pressed.caught = held.is_some();
    pressed.track.begin(time.elapsed_secs_f64());
}

/// Whatever of the window's was tapped or clicked, or the bag button, to
/// be acted on ([`act`]). Only what the press landed on, not what the click
/// passed up through on its way out; and not a press dragged to scroll a
/// list or swiped across the window, nor one that stopped a list.
fn press(click: On<Pointer<Click>>, taps: Query<&Tap>, bags: Query<(), With<BagButton>>, mut pressed: ResMut<Pressed>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if bags.contains(click.entity) {
        pressed.open = true;
    }
    if let Ok(&tap) = taps.get(click.entity)
        && pressed.tapped()
    {
        pressed.taps.push(tap);
    }
}

/// A press dragged anywhere on the window: how far it has gone, which way
/// it is going, and where it has been. (A list it began on has heard of it
/// first, the window being round the list: [`drag_list`].)
fn drag_pane(drag: On<Pointer<Drag>>, time: Res<Time<Real>>, mut pressed: ResMut<Pressed>) {
    if Some(drag.pointer_id) != pressed.pointer {
        return;
    }
    pressed.dragged = pressed.dragged.max(drag.distance.length());
    pressed.feel(drag.distance);
    pressed.track.moved(time.elapsed_secs_f64(), time.delta_secs_f64(), drag.distance);
}

/// Scrolls a list with a finger, or a mouse held down, dragged along it:
/// once the press is going that way, by as far as it goes from there.
fn drag_list(drag: On<Pointer<Drag>>, mut lists: Query<(&ComputedNode, &mut ScrollPosition), With<List>>, mut pressed: ResMut<Pressed>) {
    if Some(drag.pointer_id) != pressed.pointer || pressed.feel(drag.distance) != Way::Along {
        return;
    }
    let Ok((node, mut scroll)) = lists.get_mut(drag.entity) else {
        return;
    };
    let wanted = drag.distance.y - pressed.slack;
    let by = wanted - pressed.followed;
    pressed.followed = wanted;
    let to = (scroll.y - by).clamp(0.0, flick::room(node));
    if scroll.y != to {
        scroll.y = to;
    }
}

/// A list let go: flicked, it goes on by itself, the way the finger was
/// taking it and as fast, and slows to a stop (`flick::coast`). Let go by
/// a finger that had stopped, it stays where it is.
fn let_go_of_list(end: On<Pointer<DragEnd>>, time: Res<Time<Real>>, lists: Query<&ComputedNode, With<List>>, pressed: Res<Pressed>, mut commands: Commands) {
    if Some(end.pointer_id) != pressed.pointer || pressed.way != Way::Along {
        return;
    }
    let Ok(node) = lists.get(end.entity) else {
        return;
    };
    if flick::room(node) <= 0.0 {
        return;
    }
    // A finger going up the screen takes the list further down.
    let now = time.elapsed_secs_f64();
    let speed = -pressed.track.speed(now, time.delta_secs_f64()).y;
    if let Some(throw) = Throw::at(speed) {
        commands.entity(end.entity).insert(Coasting::new(throw, now));
    }
}

/// A press let go that went across the window: a swipe, if it went far
/// enough or fast enough ([`swiped`]), to the tab on that side, as a tap on
/// that tab is.
fn let_go_of_pane(end: On<Pointer<DragEnd>>, time: Res<Time<Real>>, panel: Res<Panel>, mut pressed: ResMut<Pressed>) {
    if Some(end.pointer_id) != pressed.pointer || pressed.way != Way::Across {
        return;
    }
    let speed = pressed.track.speed(time.elapsed_secs_f64(), time.delta_secs_f64()).x;
    if let Some(tab) = swiped(panel.tab, end.distance.x, speed) {
        pressed.taps.push(Tap::Show(tab));
    }
}

/// The tab a press that went across the window goes to, if it was a swipe
/// and that tab is not the one shown: `across` is how far to the right it
/// had gone by the time it came up, `speed` how fast it was going that way
/// then, a second. A finger going left brings the tab on the right, the
/// shop; one going right, what you own. A flick is a swipe, and so is a
/// press dragged a long way however slowly; one that was going back the
/// other way as it came up is not.
fn swiped(from: Tab, across: f32, speed: f32) -> Option<Tab> {
    let fast = speed.abs() >= SWIPE_SPEED;
    let with_it = (speed > 0.0) == (across > 0.0);
    let flicked = fast && with_it && across.abs() >= SWIPE_FLICKED;
    let dragged = across.abs() >= SWIPE_FAR && !(fast && !with_it);
    if !(flicked || dragged) {
        return None;
    }
    let to = if across < 0.0 { Tab::Shop } else { Tab::Inventory };
    (to != from).then_some(to)
}

/// Why a building cannot be brought out of the bag just now, in words.
fn said(why: Why) -> String {
    let ui = ui();
    let words = &ui.words.bag;
    match why {
        Why::OffYourLand => &words.on_your_land,
        Why::OneHouse => &words.one_house,
        Why::HouseFirst => &words.house_first,
    }
    .clone()
}

/// Does what was pressed: the bag button opens the window, in the town,
/// with nothing else up; and its own buttons go on from there. On desktop
/// `Escape` shuts it. Gone from the town with it up, it is shut.
#[allow(clippy::too_many_arguments)]
fn act(
    mut pressed: ResMut<Pressed>,
    mut panel: ResMut<Panel>,
    mut up: ResMut<DialogUp>,
    mut notice: ResMut<Notice>,
    mut scrolled: ResMut<Scrolled>,
    (close_up, keyboard, driving): (Res<CloseUp>, Res<ButtonInput<KeyCode>>, Res<Driving>),
    me: Query<(&Venue, &Balance), With<Player>>,
    (bag, home): (Res<Bag>, Res<Home>),
    (mut deals, mut stands, mut cars): (MessageWriter<Deal>, MessageWriter<Stand>, MessageWriter<CarPressed>),
) {
    let Ok((&here, balance)) = me.single() else {
        pressed.taps.clear();
        pressed.open = false;
        return;
    };
    let shut = |panel: &mut Panel, up: &mut DialogUp| {
        panel.open = false;
        panel.picked = None;
        if up.0 {
            up.0 = false;
        }
    };
    if std::mem::take(&mut pressed.open) && !panel.open && !up.0 && !close_up.busy() && here == Venue::Town && !driving.on() {
        // On what you own, whichever tab it was shut on: it is your bag
        // that the button opens (mine). Each tab keeps its category.
        panel.open = true;
        if panel.tab != Tab::Inventory {
            panel.tab = Tab::Inventory;
            scrolled.cards = 0.0;
        }
        panel.picked = None;
        notice.0 = None;
        up.0 = true;
    }
    if !panel.open {
        pressed.taps.clear();
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) || here != Venue::Town {
        pressed.taps.clear();
        shut(&mut panel, &mut up);
        return;
    }
    for tap in std::mem::take(&mut pressed.taps) {
        match tap {
            Tap::Close => {
                shut(&mut panel, &mut up);
                break;
            }
            Tap::Show(tab) if tab != panel.tab => {
                panel.tab = tab;
                panel.picked = None;
                notice.0 = None;
                scrolled.cards = 0.0;
            }
            Tap::Sort(sort) if sort != panel.sort() => {
                let tab = panel.tab as usize;
                panel.sorts[tab] = sort;
                panel.picked = None;
                scrolled.cards = 0.0;
            }
            Tap::Thing(kind) if panel.tab == Tab::Shop => {
                panel.picked = (panel.picked != Some(kind)).then_some(kind);
            }
            // Out of the bag: your car, to drive; a building, to put down
            // on your plot.
            Tap::Thing(kind) => {
                let Some(thing) = roundtown_net::bag::thing(kind).filter(|thing| bag.has.free(thing.kind) > 0) else {
                    continue;
                };
                match thing.group {
                    Group::Vehicles => {
                        shut(&mut panel, &mut up);
                        cars.write(CarPressed(Some(thing.kind)));
                        break;
                    }
                    Group::Buildings => {
                        let Some(def) = PIECES.iter().find(|def| def.kind == kind) else {
                            continue;
                        };
                        match home.may_stand(def) {
                            Ok(()) => {
                                shut(&mut panel, &mut up);
                                stands.write(Stand(def));
                                break;
                            }
                            Err(why) => notice.0 = Some((said(why), 0.0)),
                        }
                    }
                    Group::Utilities | Group::Cosmetics => {}
                }
            }
            Tap::Buy(kind) => {
                let Some(thing) = roundtown_net::bag::thing(kind) else {
                    continue;
                };
                if bag.waits() {
                    continue;
                }
                if !affords(thing, balance.0, bag.has.gems) {
                    let ui = ui();
                    let words = match thing.money {
                        Money::Coins => &ui.words.bag.no_coins,
                        Money::Gems => &ui.words.bag.no_gems,
                    };
                    notice.0 = Some((words.clone(), 0.0));
                    continue;
                }
                deals.write(Deal::Buy(thing.kind));
            }
            Tap::Sell(kind) => {
                if let Some(thing) = roundtown_net::bag::thing(kind).filter(|thing| !bag.waits() && bag.has.free(thing.kind) > 0) {
                    deals.write(Deal::Sell(thing.kind));
                }
            }
            Tap::Show(_) | Tap::Sort(_) | Tap::Nothing => {}
        }
    }
}

/// Puts the window up while it is open, and draws it again whenever what
/// it shows has changed: the tab or the category, the thing picked, what
/// is in your bag, what you have to spend, the file's colours and words.
/// Takes it down once it is shut.
#[allow(clippy::too_many_arguments)]
fn redraw(
    panel: Res<Panel>,
    bag: Res<Bag>,
    changed: Res<UiChanged>,
    me: Query<Ref<Balance>, With<Player>>,
    scrolled: Res<Scrolled>,
    roots: Query<Entity, With<Root>>,
    (assets, stock, icons): (Res<AssetServer>, Res<Stock>, Res<Icons>),
    mut commands: Commands,
) {
    if !panel.open {
        for root in &roots {
            commands.entity(root).despawn();
        }
        return;
    }
    let Ok(balance) = me.single() else {
        return;
    };
    let stale = panel.is_changed() || bag.is_changed() || balance.is_changed() || (changed.is_changed() && !changed.is_added());
    if !stale && !roots.is_empty() {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    let view = View {
        panel: &panel,
        bag: &bag,
        balance: balance.0,
        gems: bag.has.gems,
        scrolled: &scrolled,
        assets: &assets,
        stock: &stock,
        icons: &icons,
    };
    draw(&mut commands, &view);
}

/// What the window is drawn from.
struct View<'a> {
    panel: &'a Panel,
    bag: &'a Bag,
    balance: u32,
    gems: u32,
    scrolled: &'a Scrolled,
    assets: &'a AssetServer,
    stock: &'a Stock,
    icons: &'a Icons,
}

/// The picture of `thing` on its card: a building's is its piece's, the
/// one House Builder's shop shows; a car's its own (`drive::CARS`).
fn picture_of(view: &View, thing: &Thing) -> Handle<Image> {
    match PIECES.iter().find(|def| def.kind == thing.kind) {
        Some(def) => view.stock.picture(def),
        None => {
            let kind = car_def(thing.kind).kind;
            view.icons.cars.iter().find(|(car, _)| *car == kind).map(|(_, picture)| picture.clone()).unwrap_or_default()
        }
    }
}

/// The picture of what `thing` is paid for with: the coin, or the gem.
fn money_icon(view: &View, thing: &Thing) -> Handle<Image> {
    view.assets.load(match thing.money {
        Money::Coins => hud::COIN_ICON,
        Money::Gems => hud::GEM_ICON,
    })
}

/// The window as things stand: its root, over the whole screen, which a
/// tap on shuts it; the pane, its tabs and its cross; the categories down
/// its left; and the cards.
fn draw(commands: &mut Commands, view: &View) {
    let colours = ui().colours.window;
    let root = commands
        .spawn((
            Root,
            Tap::Close,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let pane = commands
        .spawn((
            ChildOf(root),
            Tap::Nothing,
            window::pane(Node {
                width: Val::VMin(WIDTH),
                max_width: Val::Vw(MOST_WIDE),
                height: Val::VMin(HEIGHT),
                padding: UiRect {
                    left: Val::VMin(window::PAD),
                    right: Val::VMin(RIGHT - CARD_GAP * 0.5),
                    top: Val::VMin(OVER_TABS),
                    bottom: Val::ZERO,
                },
                ..default()
            }),
        ))
        // A press dragged anywhere on it, and let go: a swipe across it
        // is to its other tab.
        .observe(drag_pane)
        .observe(let_go_of_pane)
        .id();
    // The tabs, from where the cards begin.
    let tabs = commands
        .spawn((
            ChildOf(pane),
            Node {
                height: Val::VMin(window::TAB_HEIGHT),
                margin: UiRect::left(Val::VMin(SORTS_WIDE + BETWEEN)),
                column_gap: Val::VMin(TAB_GAP),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for (tab, image) in [(Tab::Inventory, &view.icons.cube), (Tab::Shop, &view.icons.cart)] {
        let shown = view.panel.tab == tab;
        commands.spawn((
            ChildOf(tabs),
            Tap::Show(tab),
            window::tab(shown),
            children![window::picture(image.clone(), Val::VMin(window::TAB_ICON), window::tab_ink(shown))],
        ));
    }
    commands.spawn((
        ChildOf(pane),
        window::cross_at(view.assets.load("icons/close.png"), Tap::Close, window::CLOSE, CROSS_IN, CROSS_DOWN),
    ));
    // Under them: the categories, and the cards.
    let body = commands
        .spawn((
            ChildOf(pane),
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                margin: UiRect::top(Val::VMin(UNDER_TABS)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    sorts(commands, body, view);
    cards(commands, body, view);
    // Why something cannot be done: along the top, between the tabs and
    // the cross, where nothing else is. (Along the bottom, as the friends'
    // panel has it, it lay over the names of the second row of cards.)
    commands.spawn((
        ChildOf(pane),
        NoticeLine,
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            left: Val::VMin(window::PAD + SORTS_WIDE + BETWEEN + 2.0 * window::TAB_WIDTH + TAB_GAP + NOTICE_GAP),
            right: Val::VMin(CROSS_IN + window::CLOSE * 0.5 + NOTICE_GAP),
            top: Val::VMin(OVER_TABS),
            height: Val::VMin(window::TAB_HEIGHT),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        Pickable::IGNORE,
        children![(NoticeWords, window::words("", NOTICE_TEXT, colours.notice))],
    ));
}

/// The words of the line that says why something cannot be done.
#[derive(Component)]
struct NoticeWords;

/// The categories down the window's left, a line between each, the one
/// shown white: a list that scrolls, where it was last scrolled to.
fn sorts(commands: &mut Commands, body: Entity, view: &View) {
    let colours = ui().colours.window;
    let all = Sort::of(view.panel.tab);
    let list = commands
        .spawn((
            ChildOf(body),
            List::Sorts,
            ScrollArea,
            Node {
                width: Val::VMin(SORTS_WIDE),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition(Vec2::new(0.0, view.scrolled.sorts[view.panel.tab as usize])),
        ))
        .observe(drag_list)
        .observe(let_go_of_list)
        .id();
    for (index, &sort) in all.iter().enumerate() {
        let shown = sort == view.panel.sort();
        let last = index + 1 == all.len();
        commands.spawn((
            ChildOf(list),
            Tap::Sort(sort),
            TakesPress,
            Button,
            Node {
                height: Val::VMin(SORT_ROW),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::bottom(Val::VMin(if last { 0.0 } else { SORT_LINE })),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(colours.line),
            children![builder::label(sort.title(), SORT_TEXT, window::tab_ink(shown))],
        ));
    }
}

/// The cards of what the window shows now, three to a row: a list that
/// scrolls, where it was last scrolled to. With nothing to show, a line
/// that says so.
fn cards(commands: &mut Commands, body: Entity, view: &View) {
    let ui = ui();
    let colours = ui.colours.window;
    let list = commands
        .spawn((
            ChildOf(body),
            List::Cards,
            ScrollArea,
            Node {
                flex_grow: 1.0,
                flex_basis: Val::ZERO,
                min_width: Val::ZERO,
                margin: UiRect::left(Val::VMin(BETWEEN - CARD_GAP * 0.5)),
                padding: UiRect::top(Val::VMin(OVER_CARDS)),
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition(Vec2::new(0.0, view.scrolled.cards)),
        ))
        .observe(drag_list)
        .observe(let_go_of_list)
        .id();
    let things = shown(view.panel.tab, view.panel.sort(), &view.bag.has);
    if things.is_empty() {
        commands.spawn((
            ChildOf(list),
            Node {
                padding: UiRect::left(Val::VMin(CARD_GAP * 0.5)),
                ..default()
            },
            Pickable::IGNORE,
            children![window::words(ui.words.bag.nothing.clone(), window::HEADING, colours.status)],
        ));
        return;
    }
    for (thing, free) in things {
        let cell = commands
            .spawn((
                ChildOf(list),
                Node {
                    width: Val::Percent(100.0 / 3.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::VMin(CARD_GAP * 0.5)),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        let (fill, ..) = window::card(true);
        let card = commands
            .spawn((
                ChildOf(cell),
                Tap::Thing(thing.kind),
                TakesPress,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::VMin(CARD_HEIGHT),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
                    ..default()
                },
                BackgroundColor(fill.idle),
                fill,
            ))
            .id();
        // Its picture, always: the two buttons of the thing picked in the
        // shop go over it, not in its place.
        commands.spawn((
            ChildOf(card),
            ImageNode::new(picture_of(view, thing)),
            Node {
                height: Val::Percent(PICTURE),
                aspect_ratio: Some(1.0),
                ..default()
            },
            Pickable::IGNORE,
        ));
        let shop = view.panel.tab == Tab::Shop;
        if shop && view.panel.picked == Some(thing.kind) {
            deal_buttons(commands, card, view, thing, free);
        }
        // Its name, in its rarity's colour, and how many are in the bag.
        let name = commands
            .spawn((
                ChildOf(cell),
                Node {
                    height: Val::VMin(if shop { NAME_LINE } else { NAME_ROW }),
                    column_gap: Val::VMin(0.9),
                    padding: UiRect::top(Val::VMin(0.9)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        commands.spawn((
            ChildOf(name),
            window::words(ui.words.bag.name(thing.kind).to_string(), NAME_TEXT, ui.colours.bag.rarity(thing.rarity)),
        ));
        if free > 0 {
            let count = fill_count(free);
            commands.spawn((ChildOf(name), window::words(count, NAME_TEXT, ui.colours.bag.count)));
        }
        // In the shop, under its name: the coin, or the gem, and what it
        // costs.
        if shop {
            commands.spawn((
                ChildOf(cell),
                Node {
                    height: Val::VMin(PRICE_ROW),
                    column_gap: Val::VMin(PRICE_GAP),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        ImageNode::new(money_icon(view, thing)),
                        Node {
                            width: Val::VMin(PRICE_COIN),
                            height: Val::VMin(PRICE_COIN),
                            // Level with the number beside it, whose line
                            // is a little taller than its figures.
                            margin: UiRect::top(Val::VMin(PRICE_COIN_DOWN)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ),
                    window::words(hud::money(thing.price), PRICE_TEXT, price_ink(&ui.colours.bag, affords(thing, view.balance, view.gems))),
                ],
            ));
        }
    }
}

/// Whether you have what `thing` costs: its coins, of `balance`; or for one
/// sold for gems, its gems, of `gems`.
fn affords(thing: &Thing, balance: u32, gems: u32) -> bool {
    match thing.money {
        Money::Coins => balance >= thing.price,
        Money::Gems => gems >= thing.price,
    }
}

/// The colour a thing's price is written in under its name in the shop:
/// white with the coins for it now, or the gems for one sold for gems
/// ([`affords`]), and red without (Hajun, 2026-11-22: "the coin price
/// number text is white right now, but it should be red if u cant afford
/// it rn, and white if u can afford it"). By what you have alone: a
/// thing's price is not red while a purchase is on its way, as its Buy is
/// greyed ([`can_deal`]).
fn price_ink(colours: &BagColours, affordable: bool) -> Color {
    if affordable { colours.price } else { colours.price_cannot_afford }
}

/// Which of the two buttons on a thing in the shop can be pressed, Buy and
/// Sell: Buy with what it costs in hand ([`affords`], `balance` coins and
/// `gems`), Sell with one of it in your bag, `free` of them, and neither
/// while a purchase or a sale is on its way.
fn can_deal(thing: &Thing, free: u16, (balance, gems): (u32, u32), waits: bool) -> (bool, bool) {
    (!waits && affords(thing, balance, gems), !waits && free > 0)
}

/// How many of a thing are in the bag, as the file's words have it: "x1".
fn fill_count(count: u16) -> String {
    fill(&ui().words.bag.count, &[("count", &count.to_string())])
}

/// The two buttons on the card of the thing picked in the shop, over its
/// picture and a little see-through, the picture there behind them: Buy,
/// with the coin and its price, green; and Sell, with half that, red. Both
/// are always there, and one that cannot be pressed is greyed
/// ([`can_deal`]).
fn deal_buttons(commands: &mut Commands, card: Entity, view: &View, thing: &'static Thing, free: u16) {
    let ui = ui();
    let colours = ui.colours.bag;
    let (buys, sells) = can_deal(thing, free, (view.balance, view.gems), view.bag.waits());
    let money = money_icon(view, thing);
    let greyed = (
        Fill {
            idle: colours.deal_off,
            lit: colours.deal_off,
        },
        colours.on_deal_off,
    );
    let buy = Fill {
        idle: colours.buy,
        lit: colours.buy_lit,
    };
    let sell = Fill {
        idle: colours.sell,
        lit: colours.sell_lit,
    };
    // Over the whole card, the two in its middle; a press between them is
    // the card's own, which puts them away.
    let over = commands
        .spawn((
            ChildOf(card),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                top: Val::ZERO,
                bottom: Val::ZERO,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(DEAL_GAP),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let look = if buys { (buy, colours.on_deal) } else { greyed };
    deal_button(commands, over, Tap::Buy(thing.kind), look, &ui.words.bag.buy, (money.clone(), thing.price));
    let look = if sells { (sell, colours.on_deal) } else { greyed };
    deal_button(commands, over, Tap::Sell(thing.kind), look, &ui.words.bag.sell, (money, thing.resale()));
}

/// One of the two: its word, the coin or the gem, and how many.
fn deal_button(commands: &mut Commands, over: Entity, tap: Tap, (fill, ink): (Fill, Color), word: &str, (icon, coins): (Handle<Image>, u32)) {
    commands.spawn((
        ChildOf(over),
        tap,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(DEAL_WIDE),
            max_width: Val::Percent(92.0),
            height: Val::VMin(DEAL_HIGH),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.0),
            border_radius: BorderRadius::all(Val::VMin(0.8)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![
            builder::label(word.to_string(), DEAL_TEXT, ink),
            (
                // Dimmed with the words on a button that cannot be pressed.
                ImageNode::new(icon).with_color(Color::WHITE.with_alpha(ink.alpha())),
                Node {
                    width: Val::VMin(DEAL_COIN),
                    height: Val::VMin(DEAL_COIN),
                    margin: UiRect::left(Val::VMin(0.8)),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            builder::label(hud::money(coins), DEAL_TEXT, ink),
        ],
    ));
}

/// Shows the line that says why something cannot be done for a few
/// seconds, and then puts it away.
fn show_notice(
    time: Res<Time>,
    mut notice: ResMut<Notice>,
    mut lines: Query<&mut Visibility, With<NoticeLine>>,
    mut words: Query<&mut Text, With<NoticeWords>>,
) {
    let said = match &mut notice.0 {
        Some((text, age)) => {
            *age += time.delta_secs();
            (*age <= NOTICE_SECS).then(|| text.clone())
        }
        None => None,
    };
    if said.is_none() && notice.0.is_some() {
        notice.0 = None;
    }
    for mut visibility in &mut lines {
        visibility.set_if_neq(if said.is_some() { Visibility::Inherited } else { Visibility::Hidden });
    }
    if let Some(said) = said {
        for mut text in &mut words {
            if text.0 != said {
                text.0.clone_from(&said);
            }
        }
    }
}

/// Remembers how far each list is scrolled, by a finger or going on by
/// itself, for when the window is drawn again.
fn remember_scroll(panel: Res<Panel>, lists: Query<(&List, &ScrollPosition), Changed<ScrollPosition>>, mut scrolled: ResMut<Scrolled>) {
    for (list, scroll) in &lists {
        match list {
            List::Sorts => scrolled.sorts[panel.tab as usize] = scroll.y,
            List::Cards => scrolled.cards = scroll.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use roundtown_net::bag::{Bag as Rules, CAR, LAMP};

    use super::*;

    #[test]
    fn the_window_fits_a_phone_lying_on_its_side_and_a_tablet() {
        // The short side is 100 shares: a phone's long side is over 200, a
        // 4:3 tablet's 133, where the window is as wide as it is let be.
        assert!(HEIGHT < 100.0 - 2.0 * hud::EDGE);
        assert!(WIDTH < 200.0 - 2.0 * hud::EDGE);
        assert!(133.0 * MOST_WIDE / 100.0 < 133.0 - 2.0 * hud::EDGE);
        // Three cards and the gaps between them fill what is left of the
        // window's width beside the categories, as drawn: a card 121 of the
        // picture's dots wide, thirty shares.
        let cards = WIDTH - window::PAD - SORTS_WIDE - BETWEEN - RIGHT;
        let card = (cards - 2.0 * CARD_GAP) / 3.0;
        assert!((card - 30.1).abs() < 0.2, "{card}");
        // Two rows of cards and their names show whole under the tabs, and
        // in the shop their prices under those.
        let room = HEIGHT - OVER_TABS - window::TAB_HEIGHT - UNDER_TABS - OVER_CARDS;
        assert!(2.0 * (CARD_HEIGHT + NAME_ROW) <= room, "{room}");
        assert!(2.0 * (CARD_HEIGHT + NAME_LINE + PRICE_ROW) <= room, "{room}");
        // The two buttons fit a card, on a tablet too, where a card is
        // narrower.
        assert!(2.0 * DEAL_HIGH + DEAL_GAP < CARD_HEIGHT);
        assert!(DEAL_WIDE < card);
        // Every category shows whole with none scrolled to, the shop's six
        // too (Hajun, 2026-11-22: "try to fit it by making the panel tall
        // enough so no scrolling is necessary"), and the window is no
        // taller than that takes.
        let sorts = HEIGHT - OVER_TABS - window::TAB_HEIGHT - UNDER_TABS;
        let all = Sort::of(Tab::Shop).len() as f32 * SORT_ROW;
        assert_eq!(Sort::of(Tab::Shop).len(), 6);
        assert!(all <= sorts && sorts < all + 0.5 * SORT_ROW, "{sorts} for {all}");
        // Every line that says why something cannot be done fits between
        // the tabs and the cross: a letter of Inter is a little under half
        // as wide as its size, taken over a line of words (the longest of
        // these, 36 letters at 2.6, was 44 across on the screen).
        let room = WIDTH - (window::PAD + SORTS_WIDE + BETWEEN + 2.0 * window::TAB_WIDTH + TAB_GAP + NOTICE_GAP) - (CROSS_IN + window::CLOSE * 0.5 + NOTICE_GAP);
        let ui = ui();
        let words = &ui.words.bag;
        for line in [&words.on_your_land, &words.one_house, &words.house_first, &words.no_coins, &words.no_gems] {
            let wide = line.chars().count() as f32 * NOTICE_TEXT * 0.5;
            assert!(wide <= room, "{line:?} is about {wide} wide, and there is room for {room}");
        }
    }

    #[test]
    fn a_swipe_across_the_window_goes_to_its_other_tab() {
        // A finger flicked to the left brings the shop, which is on the
        // right; flicked to the right, what you own (Hajun, 2026-11-22: "u
        // can swipe left/right to move between inventory/shop tabs").
        assert_eq!(swiped(Tab::Inventory, -40.0, -900.0), Some(Tab::Shop));
        assert_eq!(swiped(Tab::Shop, 40.0, 900.0), Some(Tab::Inventory));
        // Nothing further that way.
        assert_eq!(swiped(Tab::Shop, -40.0, -900.0), None);
        assert_eq!(swiped(Tab::Inventory, 40.0, 900.0), None);
        // Not far enough, or a short way slowly: no swipe.
        assert_eq!(swiped(Tab::Inventory, -20.0, -900.0), None);
        assert_eq!(swiped(Tab::Inventory, -40.0, -300.0), None);
        // A long way, however slowly, and a finger that had stopped: one.
        assert_eq!(swiped(Tab::Inventory, -70.0, -100.0), Some(Tab::Shop));
        assert_eq!(swiped(Tab::Inventory, -70.0, 0.0), Some(Tab::Shop));
        assert_eq!(swiped(Tab::Shop, 64.0, 0.0), Some(Tab::Inventory));
        // Dragged across and flicked back: not one.
        assert_eq!(swiped(Tab::Inventory, -70.0, 900.0), None);
        assert_eq!(swiped(Tab::Inventory, -40.0, 900.0), None);
    }

    #[test]
    fn a_press_goes_one_way_and_is_a_tap_only_if_it_went_none() {
        // A touch that hardly moves is a tap.
        let mut pressed = Pressed::default();
        assert!(pressed.tapped());
        assert_eq!(pressed.feel(Vec2::new(3.0, -4.0)), Way::Unsure);
        pressed.dragged = 5.0;
        assert!(pressed.tapped());
        // Dragged up a list: the list's from there, less what it had gone
        // when the list took it up, and a tap no more. It stays the list's
        // though it then goes across.
        assert_eq!(pressed.feel(Vec2::new(1.0, -12.0)), Way::Along);
        assert_eq!((pressed.slack, pressed.followed), (-flick::SLOP, 0.0));
        assert_eq!(pressed.feel(Vec2::new(90.0, -12.0)), Way::Along);
        assert!(!pressed.tapped());
        // Dragged across: a swipe from there, though it then goes down.
        let mut pressed = Pressed::default();
        assert_eq!(pressed.feel(Vec2::new(-14.0, 3.0)), Way::Across);
        assert_eq!(pressed.feel(Vec2::new(-14.0, 80.0)), Way::Across);
        assert!(!pressed.tapped());
        // A finger put on a list that was going stops it, and is no tap.
        let mut pressed = Pressed { caught: true, ..default() };
        assert!(!pressed.tapped());
        pressed.caught = false;
        assert!(pressed.tapped());
    }

    #[test]
    fn the_shop_has_owned_before_the_categories_the_inventory_has() {
        let (bag, shop) = (Sort::of(Tab::Inventory), Sort::of(Tab::Shop));
        assert_eq!(bag[0], Sort::All);
        assert_eq!(bag.len(), 5);
        assert_eq!(shop[0], Sort::Owned);
        assert_eq!(shop[1..], bag[..]);
        // Each opens on everything: the shop too, though Owned is its first
        // (Hajun, 2026-11-22: "the default section for the shop should be
        // the All rather than owned (keep order the same tho)").
        let panel = Panel::default();
        assert_eq!((panel.tab, panel.sort()), (Tab::Inventory, Sort::All));
        assert_eq!(panel.sorts[Tab::Shop as usize], Sort::All);
        // And each has its words.
        for sort in shop {
            assert!(!sort.title().is_empty(), "{sort:?}");
        }
    }

    #[test]
    fn the_inventory_shows_what_is_in_the_bag_and_the_shop_everything() {
        let kinds = |things: Vec<(&'static Thing, u16)>| things.into_iter().map(|(thing, free)| (thing.kind, free)).collect::<Vec<_>>();
        let mut bag = Rules::new();
        // What everyone begins with: a lamp and a car.
        assert_eq!(kinds(shown(Tab::Inventory, Sort::All, &bag)), [(LAMP, 1), (CAR, 1)]);
        assert_eq!(kinds(shown(Tab::Inventory, Sort::Of(Group::Buildings), &bag)), [(LAMP, 1)]);
        assert_eq!(kinds(shown(Tab::Inventory, Sort::Of(Group::Vehicles), &bag)), [(CAR, 1)]);
        assert!(shown(Tab::Inventory, Sort::Of(Group::Cosmetics), &bag).is_empty());
        // The shop: everything there is, 118 things since 2026-11-25, and
        // what is owned.
        assert_eq!(shown(Tab::Shop, Sort::All, &bag).len(), THINGS.len());
        assert_eq!(shown(Tab::Shop, Sort::Of(Group::Buildings), &bag).len(), 95);
        assert_eq!(shown(Tab::Shop, Sort::Of(Group::Vehicles), &bag).len(), 23);
        assert_eq!(kinds(shown(Tab::Shop, Sort::Owned, &bag)), [(LAMP, 1), (CAR, 1)]);
        assert!(shown(Tab::Shop, Sort::Of(Group::Utilities), &bag).is_empty());
        // The lamp stood on the plot is out of the bag: gone from the
        // inventory and from what is owned, and still in the shop.
        let here = roundtown_net::At::Ground { x: 0.0, y: 0.0, z: 0.0, yaw: 0.0 };
        bag.put(1, LAMP, here).unwrap();
        assert_eq!(kinds(shown(Tab::Inventory, Sort::All, &bag)), [(CAR, 1)]);
        assert_eq!(kinds(shown(Tab::Shop, Sort::Owned, &bag)), [(CAR, 1)]);
        assert!(shown(Tab::Shop, Sort::All, &bag).iter().any(|&(thing, free)| (thing.kind, free) == (LAMP, 0)));
        // Two beds bought: two in the bag.
        bag.buy("singlebed", 1_000).unwrap();
        bag.buy("singlebed", 1_000).unwrap();
        assert_eq!(kinds(shown(Tab::Inventory, Sort::All, &bag)), [("singlebed", 2), (CAR, 1)]);
    }

    #[test]
    fn things_come_kind_by_kind_the_commonest_first_in_the_shop_and_the_rarest_first_in_the_bag() {
        // Hajun, 2026-11-25: "order of shop items should be order of
        // rarity, with common being first, obviously grouped by category
        // first like furnitures together, foundation (house models)
        // together, cars together ... and for "All" furniture should come
        // first. order of inventory items should be same category order
        // (furniture, then house models, then cars), but high rarity is
        // first so transcendent is first."
        use roundtown_net::bag::Rarity;
        let mut everything = Rules::new();
        for thing in THINGS {
            everything.owned.retain(|(kind, _)| kind != thing.kind);
            everything.owned.push((thing.kind.to_string(), 1));
        }
        let place = |kind: &str| THINGS.iter().position(|thing| thing.kind == kind).unwrap();
        for (tab, rarer_first) in [(Tab::Shop, false), (Tab::Inventory, true)] {
            for sort in Sort::of(tab) {
                let things = shown(tab, sort, &everything);
                // Kind by kind, and of each by rarity, the list's order
                // among things as rare as each other.
                for pair in things.windows(2) {
                    let (a, b) = (pair[0].0, pair[1].0);
                    let (kinds, rarities) = ((Kind::of(a), Kind::of(b)), if rarer_first { (b.rarity, a.rarity) } else { (a.rarity, b.rarity) });
                    assert!(kinds.0 <= kinds.1, "{tab:?} {sort:?}: {} before {}", a.kind, b.kind);
                    if kinds.0 == kinds.1 {
                        assert!(rarities.0 <= rarities.1, "{tab:?} {sort:?}: {} before {}", a.kind, b.kind);
                        if a.rarity == b.rarity {
                            assert!(place(a.kind) < place(b.kind), "{tab:?} {sort:?}: {} before {}", a.kind, b.kind);
                        }
                    }
                }
                // Nothing left out, nothing twice.
                let mut kinds: Vec<&str> = things.iter().map(|(thing, _)| thing.kind).collect();
                kinds.sort_unstable();
                kinds.dedup();
                assert_eq!(kinds.len(), things.len());
            }
        }
        // Everything, in the shop: furniture first, a common thing the
        // first of all; the houses after the last of the furniture, from
        // the Square House to those of ten floors; the doors and windows;
        // and the cars, ending with the Yang.
        let shop: Vec<&Thing> = shown(Tab::Shop, Sort::All, &everything).into_iter().map(|(thing, _)| thing).collect();
        let kinds: Vec<Kind> = shop.iter().map(|thing| Kind::of(thing)).collect();
        let first = |kind: Kind| kinds.iter().position(|&each| each == kind).unwrap();
        assert_eq!((shop[0].kind, shop[0].rarity), ("cafechair", Rarity::Common));
        assert_eq!(kinds[0], Kind::Furniture);
        assert!(first(Kind::Furniture) < first(Kind::Houses) && first(Kind::Houses) < first(Kind::Openings) && first(Kind::Openings) < first(Kind::Cars));
        assert_eq!(shop[first(Kind::Houses)].kind, "square_house");
        assert_eq!(shop[first(Kind::Houses) - 1].rarity, Rarity::Legendary, "the chandelier or the street lamp, the rarest furniture");
        assert_eq!(shop[first(Kind::Openings) - 1].rarity, Rarity::Transcendent, "a house of ten floors");
        assert_eq!(shop[first(Kind::Openings)].kind, "classic_door");
        assert_eq!(shop.last().map(|thing| thing.kind), Some("yang"));
        assert_eq!(shop[first(Kind::Cars)].rarity, Rarity::Rare);
        assert!(shop.iter().any(|thing| thing.kind == "painting" && Kind::of(thing) == Kind::Furniture));
        assert_eq!(Kind::of(roundtown_net::bag::thing(LAMP).unwrap()), Kind::Furniture);
        // Everything, in the bag: the same kinds in the same order, each
        // from its rarest; the Yang the first of the cars.
        let bag: Vec<&Thing> = shown(Tab::Inventory, Sort::All, &everything).into_iter().map(|(thing, _)| thing).collect();
        assert_eq!(bag.iter().map(|thing| Kind::of(thing)).collect::<Vec<_>>(), {
            let mut sorted = kinds.clone();
            sorted.sort();
            sorted
        });
        assert_eq!(bag[0].rarity, Rarity::Legendary, "the rarest furniture");
        let houses = bag.iter().position(|thing| Kind::of(thing) == Kind::Houses).unwrap();
        assert_eq!(bag[houses].rarity, Rarity::Transcendent);
        assert_eq!(bag[houses].kind, "square_house_ten_floor");
        let cars = bag.iter().position(|thing| Kind::of(thing) == Kind::Cars).unwrap();
        assert_eq!(bag[cars].kind, "yang");
        assert_eq!(bag.last().map(|thing| thing.rarity), Some(Rarity::Rare));
        // Of the vehicles alone, the same.
        let vehicles = shown(Tab::Shop, Sort::Of(Group::Vehicles), &everything);
        assert_eq!((vehicles[0].0.kind, vehicles.last().unwrap().0.kind), ("gen", "yang"));
        let vehicles = shown(Tab::Inventory, Sort::Of(Group::Vehicles), &everything);
        assert_eq!((vehicles[0].0.kind, vehicles.last().unwrap().0.kind), ("yang", "suv"));
    }

    #[test]
    fn both_buttons_are_on_a_thing_picked_in_the_shop_and_the_one_that_cannot_be_pressed_is_greyed() {
        let lamp = roundtown_net::bag::thing(LAMP).unwrap();
        let bed = roundtown_net::bag::thing("singlebed").unwrap();
        // (Buy, Sell). A lamp in the bag and no coins: it can be sold and
        // not bought.
        assert_eq!(can_deal(lamp, 1, (0, 10), false), (false, true));
        // No bed, and the coins for one to the coin: bought, and not sold
        // (Hajun: "the sell button should just be greyed out if there isnt
        // any").
        assert_eq!(can_deal(bed, 0, (bed.price, 0), false), (true, false));
        assert_eq!(can_deal(bed, 0, (bed.price - 1, 10_000), false), (false, false));
        assert_eq!(can_deal(bed, 2, (10_000, 0), false), (true, true));
        // Neither while the server has a purchase or a sale to answer.
        assert_eq!(can_deal(bed, 2, (10_000, 0), true), (false, false));
        // A house sold for gems, 50 of them: by the gems, whatever the coins.
        let house = roundtown_net::bag::thing("square_house_five_floor").unwrap();
        assert_eq!(house.price, 50);
        assert_eq!(can_deal(house, 0, (1_000_000, 10), false), (false, false));
        assert_eq!(can_deal(house, 1, (0, 50), false), (true, true));
        // The buttons are a little see-through, for the thing's picture
        // behind them to show ("slightly transparent too like my design
        // image"), and the one that cannot be pressed is another colour
        // than either.
        let colours = crate::ui::Ui::built_in().colours.bag;
        for colour in [colours.buy, colours.buy_lit, colours.sell, colours.sell_lit, colours.deal_off] {
            assert!((0.6..1.0).contains(&colour.alpha()), "{colour:?}");
        }
        assert_ne!(colours.deal_off, colours.sell);
        assert_ne!(colours.deal_off, colours.buy);
        assert!(colours.on_deal_off.alpha() < colours.on_deal.alpha());
    }

    #[test]
    fn a_price_is_white_with_the_coins_for_the_thing_and_red_without() {
        let colours = crate::ui::Ui::built_in().colours.bag;
        assert_ne!(colours.price, colours.price_cannot_afford);
        let bed = roundtown_net::bag::thing("singlebed").unwrap();
        // To the coin.
        let ink = |thing: &Thing, coins: u32, gems: u32| price_ink(&colours, affords(thing, coins, gems));
        assert_eq!(ink(bed, bed.price, 0), colours.price);
        assert_eq!(ink(bed, bed.price - 1, 10_000), colours.price_cannot_afford);
        assert_eq!(ink(bed, 0, 0), colours.price_cannot_afford);
        assert_eq!(ink(bed, 1_000_000, 0), colours.price);
        // A house sold for gems: to the gem.
        let house = roundtown_net::bag::thing("square_house_nine_floor").unwrap();
        assert_eq!((house.price, ink(house, 0, 750), ink(house, 1_000_000, 749)), (750, colours.price, colours.price_cannot_afford));
        // Of everything there is: red wherever Buy would be greyed for want
        // of what it costs, and nowhere else, whether or not you have one.
        for thing in THINGS {
            for have in [0, thing.price - 1, thing.price, thing.price + 1] {
                for (coins, gems) in [(have, 10), (1_000_000, have), (have, have)] {
                    for free in [0, 2] {
                        let (buys, _) = can_deal(thing, free, (coins, gems), false);
                        assert_eq!(ink(thing, coins, gems) == colours.price, buys, "{} with {coins} coins and {gems} gems", thing.kind);
                    }
                }
            }
        }
    }

    #[test]
    fn every_thing_has_a_name_a_picture_and_a_colour_and_the_file_s_rarities_are_hajun_s() {
        let ui = ui();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for thing in THINGS {
            let name = ui.words.bag.name(thing.kind);
            assert_ne!(name, thing.kind, "{} has no name in ui.ron", thing.kind);
            let picture = match PIECES.iter().find(|def| def.kind == thing.kind) {
                Some(def) => def.picture(),
                None => car_def(thing.kind).picture.to_string(),
            };
            assert!(root.join(&picture).is_file(), "{}: no picture, {picture}", thing.kind);
        }
        // A building's name is the one House Builder's shop has for it.
        for def in PIECES {
            assert_eq!(ui.words.bag.name(def.kind), def.name);
        }
        // The colours as Hajun gave them, 2026-11-21.
        use roundtown_net::bag::Rarity;
        let hex = |colour: Color| {
            let srgb = colour.to_srgba();
            format!("{:02X}{:02X}{:02X}", (srgb.red * 255.0).round() as u8, (srgb.green * 255.0).round() as u8, (srgb.blue * 255.0).round() as u8)
        };
        let colours = crate::ui::Ui::built_in().colours.bag;
        let given = [
            (Rarity::Transcendent, "9E0915"),
            (Rarity::Mythic, "FF4500"),
            (Rarity::Legendary, "FFA500"),
            (Rarity::Epic, "800080"),
            (Rarity::Rare, "4169E1"),
            (Rarity::Uncommon, "32CD32"),
            (Rarity::Common, "D3D3D3"),
        ];
        for (rarity, colour) in given {
            assert_eq!(hex(colours.rarity(rarity)), colour, "{rarity:?}");
        }
        assert_eq!(fill_count(3), "x3");
    }

    #[test]
    fn every_line_of_hajun_s_list_of_rarities_is_a_thing_there_is_as_the_list_has_it() {
        // `assets/itemrarities.txt`, Hajun's: a thing to a line, "Name -
        // Rarity", and " (gem)" after one sold for gems. Each is the thing
        // whose name, kind or file reads the same once only its letters and
        // figures are kept, the list's "Roof Terrace House" being the
        // game's `square_house_roofless`; and every thing there is is on it.
        use roundtown_net::bag::Rarity;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("assets/itemrarities.txt")).expect("Hajun's list of rarities");
        let plain = |name: &str| name.chars().filter(char::is_ascii_alphanumeric).collect::<String>().to_lowercase();
        let ui = ui();
        let mut seen = Vec::new();
        for line in list.lines() {
            let Some((name, rarity)) = line.trim().rsplit_once(" - ") else {
                continue;
            };
            let (rarity, gems) = match rarity.strip_suffix(" (gem)") {
                Some(rarity) => (rarity, true),
                None => (rarity, false),
            };
            let wanted = plain(name).replace("roofterracehouse", "squarehouseroofless");
            let thing = THINGS
                .iter()
                .find(|thing| {
                    let file = PIECES.iter().find(|def| def.kind == thing.kind).map(|def| def.file).unwrap_or(thing.kind);
                    [ui.words.bag.name(thing.kind), thing.kind, file].into_iter().any(|said| plain(said) == wanted)
                })
                .unwrap_or_else(|| panic!("{name:?} in the list is no thing there is"));
            let as_listed = Rarity::ALL.into_iter().find(|each| format!("{each:?}") == rarity).unwrap_or_else(|| panic!("{rarity:?}"));
            assert_eq!((thing.rarity, thing.money == Money::Gems), (as_listed, gems), "{name}");
            seen.push(thing.kind);
        }
        for thing in THINGS {
            assert!(seen.contains(&thing.kind), "{} is not in the list", thing.kind);
        }
        assert_eq!(seen.len(), THINGS.len());
    }

    #[test]
    fn the_car_s_picture_and_the_gem_s_are_in_the_apk() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt")).expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim_start_matches("<dir>").trim_start_matches("<file>"))
            .collect();
        for picture in CARS.iter().map(|car| car.picture).chain([hud::GEM_ICON]) {
            let bytes = std::fs::read(root.join("assets").join(picture)).expect(picture);
            assert_eq!(&bytes[1..4], b"PNG", "{picture}");
            let left_out = patterns.iter().find(|pattern| match (pattern.strip_prefix('*'), pattern.strip_suffix('*')) {
                (Some(end), _) => picture.ends_with(end),
                (_, Some(start)) => picture.starts_with(start),
                _ => **pattern == picture,
            });
            assert!(left_out.is_none(), "{picture} is left out of the APK by `{}`", left_out.unwrap());
        }
    }
}
