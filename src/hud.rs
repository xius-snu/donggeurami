//! What is drawn over the world, laid out as Hajun sketched it
//! (`assets/UIbuttonlayout.jpg`, 2026-10-31): top left your name on a plate,
//! with a person button and the settings' gear beside it (`settings`) and
//! your balance under; down the right edge a gift, two people (your
//! friends, `friends`; a speech bubble until 2026-11-14) and a gamepad
//! (**the person and the gift are away since 2026-11-14**, when Hajun found
//! the town had "too many lobby ui buttons now", and the others stand in
//! their places: [`PERSON`], [`GIFT`]);
//! round a phone's jump button a face, a speech bubble (things to say at a
//! press, `quick`), a small keyboard over that (the chat, `chat`) and a
//! fist; and headlines across the middle — the town's
//! name, as a welcome, whenever you arrive in the town, and whatever else
//! is [`announce`]d, like House Builder's theme. And how the stick and the
//! jump button look (`setup_hud`; a phone's, and since 2026-11-06 the
//! desktop's too, driven by the mouse). Of the buttons only the gear, the stick,
//! the jump button, the fist, the face, the gamepad (the game selector,
//! `games`), the two people under the gift (the friends' panel, since
//! 2026-11-13; [`FriendsButton`]), the speech bubble and the keyboard over
//! the jump button ([`QuickChatButton`], [`ChatButton`]) and, in the car,
//! the gift do anything yet (Hajun: "i dont need you to make them
//! functional yet. just make them in the screen").
//!
//! Every word on the screen is in Inter (`font`, since 2026-11-13; Bevy's
//! built-in font until then).
//!
//! Every button is in one style, the stick's and the jump button's: a
//! see-through grey disc with a whitish picture on it and no rim
//! ([`round_button`], [`TOUCH_DISC`], [`TOUCH_MARK`]). The pictures are drawn
//! in code where they are simple (the jump button's arrow, the gear, the
//! pencil) and Google's Material icons otherwise ([`ICONS`], in
//! `assets/icons/`, Apache 2.0, turned white there for the buttons to tint;
//! Hajun: "search online for that icon + png/transparent, download and
//! modify it").
//!
//! Every button over the world is a `bevy_ui` node pressed through
//! `bevy_picking`'s click, which a mouse and a finger both send, so there is
//! no platform code in them. Each carries [`TakesPress`]; the code that reads
//! fingers and the mouse for everything else asks [`Presses`] first, so that
//! a press on one is not also taken for a look round or a tap on the town.
//!
//! Until 2026-10-31 a button top right took you home and back again; Hajun
//! had it taken out. Your home (`testhomemap.glb`) is still loaded, with
//! nothing that takes you there.
//!
//! **Since 2026-11-21 the face is the button for your bag** ([`BagButton`];
//! Hajun: "im gonna change the emote button in the lobby to a
//! inventory/shop panel"), which opens your bag (`bag::panel`): what you
//! own, and the shop. On it is the packed box of that window's first tab
//! ([`cube`]; a sack for its first day, [`SACK`]: "i dont like the bag icon
//! for that inventory panel. just reuse the box icon u used for the
//! inventory tab"). The car the face used to put you in is in the bag now,
//! to be tapped there; and in the car a door gets you out of it
//! ([`ExitButton`]; "a new ui button ... that is like a leave door icon and
//! u press that to get off ur car"): half way down the left edge of the
//! screen for a day, and since 2026-11-22 where your bag's button is out
//! of the car, which is put away in it ("the leave door/car button instead
//! of left center, just put it where the inventory ui supposed to be cuz rn
//! that slot is empty"). By the left edge now is what changes your plot
//! ([`LeftEdge`], `customize`). And
//! under your coins, your gems ([`GemsText`]; "i added gems below the
//! coin"), the gem itself as wide as the coin itself and the two numbers
//! one size ([`GEM_WIDE`], [`BALANCE_TEXT`]).
//!
//! In a game of Bridge Wars (`arena`, since the night of 2026-11-07) the buttons are
//! laid out as Hajun sketched them for it instead
//! (`assets/pvpmapbuttonui.jpg`; [`Layout::Arena`]): top left your name
//! and your health bar, and under it what you hold of the four things the
//! generators make, silver, gold, sapphire and amethyst, each a picture of
//! it with its count beside it (since 2026-11-11; wood, gold, diamond and
//! emerald until then, the first of them the metres of bridge left, which
//! are written in front of you as you build and nowhere else now); top
//! right the minimap; and bottom right the sword in the jump button's
//! place, which punches, the jump button, an empty slot and the plank
//! about it, and a button of three dots over them, which does nothing
//! yet. The stick stays. The pictures are drawn in code ([`sword`],
//! [`bag`], [`item_picture`], [`plank`], [`dots`]).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use roundtown_net::{Item, UTILITY_SLOTS};

use crate::arena::{Fighting, Health, Resources};
use crate::{Player, Team};
use crate::bridge::BridgePressed;
use crate::bag::Bag;
use crate::drive::{CarPressed, Driving};
use crate::island::{Islands, Venue};
use crate::lobby::{Balance, STARTING_BALANCE};
use crate::minimap::{Minimap, MinimapNode};
use crate::safe::{KeepsClear, RightRoom, StepsIn};
use crate::signup::Signup;
use crate::ui::{UiChanged, ui};

pub(crate) const COIN_ICON: &str = "coin.png";
/// How much of its picture's width the coin itself takes: Hajun's
/// `assets/coin.png` is 512 across, and the coin in it 459, with clear
/// round it.
const COIN_FILLS: f32 = 459.0 / 512.0;
/// The gem's picture, Hajun's (`assets/gems.png`, 288 by 269): beside how
/// many you have, under your coins.
pub(crate) const GEM_ICON: &str = "gems.png";
/// How tall the gem's picture is for its width; and how much of its width
/// the gem itself takes, 281 of the 288, which is nearly all of it where
/// the coin has clear round it.
const GEM_SHAPE: f32 = 269.0 / 288.0;
const GEM_FILLS: f32 = 281.0 / 288.0;

/// Sizes, as shares of the short side of the screen, so that a phone and a
/// monitor show the same thing: your name on its plate and your balance
/// under it; the round buttons beside your name, the bigger ones down the
/// right edge, and the bigger still round a phone's jump button (Hajun,
/// 2026-10-31: "keep the size of the user icon and the settings icon but
/// increase the size of every button the right hand side both top and
/// bottom"; 8 and 12 until then).
const NAME_TEXT: f32 = 4.2;
/// How far each of the four copies of your name that make its stroke is
/// set from the name itself, across and down, in shares of the short side:
/// as thin for the name's size as the stroke round the chat's words is for
/// theirs (`chat`, 0.09 for words of 2.5).
const NAME_STROKE: f32 = NAME_TEXT * 0.036;
/// How many coins you have and, under it, how many gems: one size (Hajun,
/// 2026-11-22: "the coin number text is larger than the gem text. make the
/// coin number text same size as the gem text size"; the coins' was 5.0
/// until then, and the gems' this, three quarters of it, as drawn in
/// `bagandinventorypanel1.png`).
const BALANCE_TEXT: f32 = 3.8;
const BALANCE_ICON: f32 = 5.0;
/// How wide the gem's picture is drawn, for the gem itself to come out as
/// wide as the coin itself ("the gems icon is slightly larger than the coin
/// icon ... slightly decrease the gem icon size to match the coin icon
/// size"). It was as wide as the coin's picture until then, and so the gem
/// a twelfth wider than the coin, whose picture has clear round it.
const GEM_WIDE: f32 = BALANCE_ICON * COIN_FILLS / GEM_FILLS;
pub(crate) const SMALL_BUTTON: f32 = 8.0;
/// The buttons down the right edge: 9.5 until Hajun asked for "very
/// slightly" bigger (2026-11-01), and that much further in from the top
/// and the right than the name is ([`EDGE`]), "very slightly" more margin.
const SIDE_BUTTON: f32 = 10.5;
const SIDE_INSET: f32 = 1.5;
pub(crate) const ACTION_BUTTON: f32 = 14.0;
/// The buttons by the left edge of the screen ([`LeftEdge`]), which are
/// `customize`'s: how big each is, the gap between two of them, one over
/// the other, and how far down the screen the lowest of them ends, in
/// parts of a hundred of its height. Three of them, one over the other,
/// begin 26.5 down, under your gems, which end 24, and end over the stick,
/// whose disc begins 64 down; the one that is there alone is where the
/// middle of the three is, from 38.75 to 49.75 (Hajun, 2026-11-22: "a bit
/// too down, i think u can move it up a bit": where the lowest is until
/// then, under the chat's lines, which are by that edge from 38.5 to 48.6
/// and step aside of them now, `chat::Aside`).
pub(crate) const LEFT_BUTTON: f32 = 11.0;
pub(crate) const LEFT_GAP: f32 = 1.25;
const LEFT_FOOT: f32 = 62.0;
/// Where your bag's button is among those round the jump button, from the
/// corner of their box: and, in your car, the door out of it.
const BAG_PLACE: (f32, f32) = (18.5, 18.5);
/// Whether the gift is there, at the top of the right edge, and the person
/// beside your name. Neither since 2026-11-14 (Hajun: "there is too many
/// lobby ui buttons now. i think we can get rid of the gift icon for now,
/// as well as the profile icon next to the setting. u can shift the other
/// buttons in place of those"): the two people and the gamepad stand one
/// place higher each, and the gear right beside your name. The person did
/// nothing yet. The gift, held in the car, was its gun ([`GunButton`],
/// `drive::shoot`): with the gift away the gun was G on desktop and
/// nothing on a phone, and since 2026-11-15 the gun is turned off too
/// (`drive::GUN`), so that the gift put back would do nothing until that
/// is. Everything of both is still here, and each of these words puts its
/// button back.
const GIFT: bool = false;
const PERSON: bool = false;
/// The button for your bag has the packed box of its window's first tab on
/// it ([`cube`]; Hajun, 2026-11-22: "i dont like the bag icon for that
/// inventory panel. just reuse the box icon u used for the inventory tab as
/// the ui button for opening the inventory/shop panel"). The sack it had
/// for its first day ([`bag`]) is still here: this word puts it back.
const SACK: bool = false;
/// Bridge Wars' layout ([`Layout::Arena`]): the health bar under your name,
/// its rim; a resource's picture and its count, in a row wide enough for
/// the pictures to line up down the right edge; and the small buttons
/// among those bottom right, an empty slot over the sword and the dots.
const HP_WIDTH: f32 = 30.0;
const HP_HEIGHT: f32 = 3.6;
const HP_RIM: f32 = 0.35;
const RESOURCE_ICON: f32 = 4.0;
const RESOURCE_TEXT: f32 = 3.4;
const RESOURCE_ROW: f32 = 14.0;
/// The minimap top right (`minimap`; since 2026-11-09, in the bag's place,
/// Hajun: "put a minimap that only shows the current player's location as
/// a red dot and your teammate as a green dot"), and its rim.
const MINIMAP: f32 = 26.0;
const MINIMAP_RIM: f32 = 0.35;
/// The three buttons round the sword, all alike, on an arc round the
/// sword's middle (the night of 2026-11-07, Hajun: "make the 3 circles
/// surrounding the sword icon in that map the same size ... and even the
/// distances out"), each as its right and bottom offsets in the box round
/// the jump button, in shares of the short side: the jump button beside
/// the sword ("i want the bottom left one to be the jump icon and a jump
/// button"), which `hud_layout` in `src/lib.rs` reads presses on, and two
/// empty slots, up and to the left of it and over it; and the dots, smaller,
/// over those.
pub(crate) const ARENA_JUMP: (f32, f32, f32) = (24.5, 4.0, ACTION_BUTTON);
/// The empty slot up and to the left of the sword, and over the sword the
/// bridge button (since 2026-11-08, Hajun: "the bridge icon (maybe the 3rd
/// button above the sword icon) and it is a toggle button that enables
/// bridging mode vs normal mode"; an empty slot until then).
const ARENA_SLOT: (f32, f32) = (18.5, 18.5);
pub(crate) const ARENA_BRIDGE: (f32, f32) = (4.0, 24.5);
/// (Over the bridge button, at 0 and 42, until 2026-11-14: the chat's
/// button stands there now, [`CHAT_BUTTON`], and the dots, were they put
/// back, over that.)
const ARENA_DOTS: (f32, f32, f32) = (-1.5, 50.5, 8.0);
/// The small button that takes up the chat's box to type a line into
/// ([`ChatButton`]), in the town and in Bridge Wars alike, as its right and
/// bottom offsets in the box round the jump button and how big it is, in
/// shares of the short side (since 2026-11-14, Hajun: "the bottom right
/// chat icon is too big of a button to be used for the global chat. So
/// make a smaller circle above the current chat icon, a little bit to the
/// right, and. That button can be in the same location for both the
/// Bridgewaters minigame and the lobby map"). The big speech bubble it
/// stands over is 14 across, from 1 to 15 in from the box's right and up
/// to 38.3; in Bridge Wars the bridge button is about there. This one is 9
/// across, 1.6 over that and its middle 5.5 to the right of the bubble's:
/// under the gamepad in the town and the minimap in a game, and clear of
/// the button for things to use that is nearly straight over the sword.
/// How big and exactly where are mine.
pub(crate) const CHAT_BUTTON: (f32, f32, f32) = (-2.0, 39.9, 9.0);
/// The most the chat's small button steps in from the right for a safe area
/// that reaches it (`safe::StepsIn`), in shares of the short side. It hangs
/// 2 out of the right of the box round the jump button, half way down the
/// screen, which is where an iPhone's island lies with the phone held the
/// other way up: on a 16 Pro the island ends 51 points in and the button
/// began 44 in, 7 of it under the island (drawn over a window the size of
/// its screen, 2026-11-15). Its safe area asks for more than this, 24
/// points, being 11 deeper than the island; this is as far as leaves the
/// button more than 1.5 clear of the button for things to use that is
/// nearly straight over the sword, as every button here is of every other,
/// and it leaves it 4 points clear of that island. The dots, were they put
/// back, hang as far out, higher up.
pub(crate) const CHAT_STEPS_IN: f32 = 2.6;

/// How far at the most the bar across the top is moved in from the right
/// for a camera's hole (`safe::RightRoom`): as far as leaves the things to
/// say, fanned out of the speech bubble (`quick`), a share of the short
/// side clear of the buttons down the right edge, which stand at that end
/// of it. The fan's plates end `quick::RIGHT_END` left of the box round the
/// jump button, itself `JUMP_RIGHT_VW` of the width in, and the buttons
/// reach this far in from the edge as they stand. On a phone that is 4.5
/// shares, 18 points of an iPhone's 52 (2026-11-15: moved the whole way,
/// the game button lay under "Bye!" and "Good game!"); on a screen as
/// broad as a tablet's, none. In Bridge Wars the minimap is at that end
/// and there is no fan; it is moved the same, so that the bar is one
/// thing.
pub(crate) fn bar_room() -> RightRoom {
    RightRoom {
        vw: crate::JUMP_RIGHT_VW,
        vmin: crate::quick::RIGHT_END - (EDGE + SIDE_INSET + SIDE_BUTTON) - 1.0,
    }
}

/// How the chat's small button keeps clear of a camera's hole on its side
/// of the screen: from where it stands, in the box round the jump button.
fn chat_steps_in() -> StepsIn {
    StepsIn {
        vw: crate::JUMP_RIGHT_VW,
        vmin: CHAT_BUTTON.0,
        most: CHAT_STEPS_IN,
    }
}
/// The big speech bubble over the town's jump button ([`QuickChatButton`]),
/// the same way: where it has been since 2026-10-31.
pub(crate) const QUICK_BUTTON: (f32, f32, f32) = (1.0, 24.3, ACTION_BUTTON);
/// Whether the three dots are there at all. Not since 2026-11-14 (Hajun:
/// "i dont think the UI ... button is necessary. just remove it and keep
/// the default as the only option - only showing the button if the slot
/// is filled and not showing empty buttons"): a button for things to use
/// is there while something is on it, and there is nothing to show the
/// empty ones with (`utility::slot_shown`). The button, its picture and
/// what it turned on and off (`utility::MoreSlots`) are all still here:
/// this one word puts it back.
const DOTS: bool = false;
/// The four more buttons for things to use (since 2026-11-13, Hajun's
/// `new4morebuttonlayout.jpg`: "4 more buttons that are hidden by default
/// until either u press the ... toggle button or u buy an item that fills
/// up another slot"), on a second ring round the sword's middle: how far
/// out, in shares of the short side, where the jump button, the second
/// button and the bridge button are 20.5 out; and where on it each is, in
/// degrees up from straight left of the sword, in the order they fill.
/// Measured off the sketch, whose circles are 37 to 40 out at 0, 25, 52
/// and 80 degrees, and evened out as the first three were: 3.5 clear of
/// each other and of the first ring, the last ending beside the dots. The
/// order is mine: the one beside the second button first, then outward,
/// so that two or three things lie together under the thumb.
const MORE_RING: f32 = 38.0;
const MORE_ANGLES: [f32; UTILITY_SLOTS - 1] = [53.3, 26.7, 80.0, 0.0];

/// Where the button for things to use numbered `slot` is, as its right and
/// bottom offsets in the box round the jump button: the second button for
/// the first, and the four more on their ring.
pub(crate) fn slot_place(slot: usize) -> (f32, f32) {
    let Some(angle) = slot.checked_sub(1).and_then(|more| MORE_ANGLES.get(more)) else {
        return ARENA_SLOT;
    };
    let (up, left) = angle.to_radians().sin_cos();
    // From the sword's middle, which is the box's.
    let from = (crate::JUMP_SIZE_VMIN - ACTION_BUTTON) * 0.5;
    (from + MORE_RING * left, from + MORE_RING * up)
}
/// The metres of bridge you have, written small inside the bridge button,
/// under the plank (since 2026-11-12, Hajun: "above the ui bridge button
/// add a small white text "123m"", and a few minutes later, "instead of
/// above the button, put it inside the button, below the icon"): how big,
/// in shares of the short side, under the counts top left; how far up
/// from the middle of the button the plank is moved to make room for it,
/// and how far down the button the words begin, both as shares of the
/// button. "1234m" is still well inside the disc there.
const BRIDGE_COUNT_TEXT: f32 = 2.8;
const BRIDGE_PLANK_UP: f32 = 13.0;
const BRIDGE_COUNT_TOP: f32 = 52.0;
/// How much of a round button its picture takes up, as a share of it.
pub(crate) const PICTURE: f32 = 58.0;
pub(crate) const HEADLINE: f32 = 8.0;
pub(crate) const CAPTION: f32 = 4.5;
/// How far in from the top and the sides of the screen the balance sits:
/// further than a phone's rounded corner reaches.
pub(crate) const EDGE: f32 = 4.0;
/// How much of the jump button its arrow takes up, as a share of it.
pub(crate) const JUMP_ARROW: f32 = 56.0;
/// Room left under a headline, as a share of the screen's height, which lifts
/// it clear of your own character in the middle.
pub(crate) const ABOVE_YOU: f32 = 36.0;

/// How long a headline takes to come up and to go, in seconds, and how long
/// the welcome is up for altogether.
const HEADLINE_IN: f32 = 0.4;
const HEADLINE_OUT: f32 = 0.8;
const WELCOME_SECS: f32 = 3.5;
/// How quickly the town's name goes once a window comes up while it is on
/// the screen, in seconds, from however strong it is then (since
/// 2026-11-18; Hajun: "the Donggeurami Town intro text when u open the app
/// should automatically fade right away when u open some menu like game or
/// setting or friends"; mine, how quickly): the opening's (`intro`) and the
/// welcome's back in the town ([`hush_welcome`]). Any window that holds
/// [`DialogUp`] does it: the game selector, the settings, the friends'
/// panel, the chat's box, a talk.
pub(crate) const HUSH_SECS: f32 = 0.25;

// Every colour is `assets/ui.ron`'s (`ui`): the soft shadow under every
// word, so that it reads over sky and grass alike; the dark glass behind
// whatever has to stay legible over anything; gold, and the dark blue of
// words on it; every button, and a phone's stick, a see-through grey with
// no rim, a little more solid while the jump button is held, and the
// picture on each, the stick's knob and the jump button's arrow among
// them, a see-through white (Hajun, 2026-10-31; "a little greyed", 0.92 at
// 0.38, until later that day, when Hajun asked for "slightly more
// whiter"); and your name's plate, the same grey, a little more solid.

/// The pictures on the buttons that are not drawn in code, in
/// `assets/icons/`, white on clear: Google's Material icons (its
/// `LICENSE-material-icons.txt` says which), a person and the settings'
/// gear beside your name, a gamepad down the right edge, and a face round
/// the jump button; and
/// Font Awesome's gift and comment-dots (`LICENSE-font-awesome.txt`; the
/// ones Hajun's FlutterFlow screenshots showed, 2026-11-01, rendered from
/// their SVGs), the gift down the right edge and the bubble with its three
/// dots, `social`, which was your friends' button under the gift until
/// 2026-11-14 and is the big button over the jump button since (Hajun:
/// "there's 2 chat icons. I like the top chat icon design better, so use
/// that chat icon for the bottom chat icon. And instead of the top chat
/// icon, which was for friends, uh, use a, uh, person, 2-person icon for
/// the friend UI button"). Material's own bubble, `chat`, which was there
/// until then, is on nothing now. The fist with the face and the bubble,
/// and the keyboard over the bubble, are drawn in code ([`fist`],
/// [`keyboard`]). A test keeps every one in
/// the APK. The leaderboard's (`leaderboard.png`) and the boxing glove
/// (`punch.png`) are still beside them, unused since 2026-11-01. And, since
/// 2026-11-13, the friends' panel's (`friends`), Material's too: two people
/// (`group`, the friends' button's own picture too since 2026-11-14), a
/// person with a plus (`person_add`), a magnifying glass
/// (`search`), two sheets (`content_copy`), an envelope (`mail`), a cross
/// (`close`), a chevron (`arrow_back_ios_new`), a door (`logout`), and the
/// group of three (`groups`), unused for twelve days until then.
pub(crate) const ICONS: [&str; 16] = [
    "person", "settings", "gift", "social", "gamepad", "mood", "chat", "friends", "add_friend", "search", "copy", "mail",
    "close", "back", "leave", "groups",
];

/// Every picture of the buttons that is read off a file ([`ICONS`], and
/// the coin), held for as long as the app runs: one shown by the town's
/// buttons alone was let go of while you were in a game, which has its
/// own, and read again as you came back, so that for the first frames the
/// gear, the coin, the friends' and the gamepad's buttons were empty discs
/// (a check of the merged games, 2026-11-18; mine).
#[derive(Resource)]
#[allow(dead_code)]
struct KeptIcons(Vec<Handle<Image>>);

fn keep_icons(assets: Res<AssetServer>, mut commands: Commands) {
    let icons = ICONS
        .iter()
        .map(|name| assets.load(format!("icons/{name}.png")))
        .chain([assets.load(COIN_ICON), assets.load(GEM_ICON)]);
    commands.insert_resource(KeptIcons(icons.collect()));
}

/// Your balance, top left.
#[derive(Component)]
struct BalanceText;

/// How many gems you have, under it (`bag`, since 2026-11-21).
#[derive(Component)]
struct GemsText;

/// Your name, top left, on its plate.
#[derive(Component)]
struct NamePlate;

/// One of the four copies of your name that lie under it on its plate and
/// make the thin stroke round it while it is in your team's colour
/// ([`colour_name`], [`NAME_STROKE`]).
#[derive(Component)]
struct NameRim;

/// The tag after your name, as the server has it: the number your friends
/// tell you from anyone else of your name by (`friends`, which writes it
/// here as the server says it). Written after your name on its plate,
/// "Hajooon#5571" (since 2026-11-13; Hajun: "just put that number in the
/// main Ui top left corner name tag ... so that its Username#5571 instead
/// of just Username"). None offline, where there are no tags, and the
/// plate has your name alone.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct YourTag(pub Option<u16>);

/// Which buttons are round the screen: the town's, everywhere but in a game
/// of Bridge Wars, where they are the game's own (`arena`; Hajun,
/// the night of 2026-11-07: "in the pvpmap game ... the button ui should change to
/// pvpmapbuttonui.jpg layout"), and, since 2026-11-18, in a game of one of
/// the four played at tables (`table`), where each is its game's own, put
/// up by the game's own module (`blend_in::spawn_bar` and the rest). It
/// follows the island you are on ([`relayout`]).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Layout {
    #[default]
    Town,
    Arena,
    BlendIn,
    KnockOff,
    BridgeRace,
    LastStand,
}

impl Layout {
    /// The buttons of wherever `venue` is.
    pub(crate) fn of(venue: Venue) -> Self {
        match venue {
            Venue::Arena => Layout::Arena,
            Venue::BlendIn => Layout::BlendIn,
            Venue::KnockOff => Layout::KnockOff,
            Venue::BridgeRace => Layout::BridgeRace,
            Venue::LastStand => Layout::LastStand,
            Venue::Town | Venue::Home | Venue::Lobby | Venue::Plot(_) => Layout::Town,
        }
    }

    /// Whether these are a fight's buttons, laid out as Bridge Wars' are, the
    /// sword or the fist in the jump button's place (`island::fights`).
    pub(crate) fn fights(self) -> bool {
        matches!(self, Layout::Arena | Layout::BridgeRace | Layout::LastStand)
    }
}

/// The gamepad, down the right edge, which opens the game selector
/// (`games`).
#[derive(Component)]
pub(crate) struct GamesButton;

/// The two people over it, under the gift, which open your friends
/// (`friends`, since 2026-11-13; Hajun: "The friend panel popup will open
/// when u tap the top right corner chat bubblt icon under the gift icon";
/// a speech bubble until 2026-11-14, [`ICONS`]).
#[derive(Component)]
pub(crate) struct FriendsButton;

/// The fill of your health bar in Bridge Wars, as wide as your health (and
/// in any game whose bar has one, [`health_bar`]).
#[derive(Component)]
struct HealthFill;

/// The count beside the picture of one of the things you hold in Bridge
/// Wars.
#[derive(Component)]
struct ResourceCount(Item);

/// The metres of bridge you have, inside the bridge button in Bridge
/// Wars, under its plank.
#[derive(Component)]
struct BridgeCount;

/// Words across the middle of the screen, up for a moment, and how long they
/// have been up; and how long they take to go ([`HEADLINE_OUT`], or
/// [`HUSH_SECS`] once a window has come up over the welcome).
#[derive(Component)]
struct Headline {
    age: f32,
    secs: f32,
    out: f32,
}

impl Headline {
    /// How strongly it is drawn now: coming up, held, or going.
    fn solid(&self) -> f32 {
        (self.age / HEADLINE_IN).min((self.secs - self.age) / self.out).clamp(0.0, 1.0)
    }
}

/// The headline that is the town's name, as a welcome.
#[derive(Component)]
struct Welcome;

/// You have arrived in the town, and are to be welcomed as soon as it has
/// loaded.
#[derive(Resource, Default)]
struct WelcomeDue(bool);

/// Something on the screen that takes a press for itself: every button over
/// the world, the dark behind a dialog, the bubble over whoever you can talk
/// to, and the whole screen while you talk (`talk`). Whatever reads a finger
/// or the mouse for the world asks [`Presses::claimed`] before it takes a
/// press.
#[derive(Component, Default)]
pub(crate) struct TakesPress;

/// Something on the screen that takes no press itself, round which a
/// finger still does not take the stick where the stick is taken anywhere
/// (Knock Off's: `crate::stick_anywhere`): the bar of power between its up
/// and down buttons, which a finger meant for one of them lands on.
#[derive(Component)]
pub(crate) struct KeepsTheStickOff;

/// Where the things that take a press for themselves are on the screen, as
/// they were last laid out; and those that keep the stick off
/// ([`KeepsTheStickOff`]).
#[derive(SystemParam)]
pub(crate) struct Presses<'w, 's> {
    takers: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static InheritedVisibility,
        ),
        With<TakesPress>,
    >,
    keep_off: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static InheritedVisibility,
        ),
        (With<KeepsTheStickOff>, Without<TakesPress>),
    >,
}

impl Presses<'_, '_> {
    /// Whether `pos`, in logical window pixels, is on something showing that
    /// takes a press for itself. Tested the way picking tests it, against the
    /// shape each one was actually laid out in, so that the two always agree.
    pub(crate) fn claimed(&self, window: &Window, pos: Vec2) -> bool {
        let pos = pos * window.scale_factor();
        self.takers
            .iter()
            .any(|(node, place, shown)| shown.get() && node.contains_point(*place, pos))
    }

    /// Whether `pos`, in logical window pixels, is on or within `margin`
    /// logical pixels of something showing that takes a press, or that
    /// keeps the stick off ([`KeepsTheStickOff`]): where a finger may have
    /// meant a button and missed it by a little (since 2026-11-18, for
    /// Knock Off's stick, which is taken anywhere else). Each by the box it
    /// was laid out in, grown by the margin all round.
    pub(crate) fn near(&self, window: &Window, pos: Vec2, margin: f32) -> bool {
        let scale = window.scale_factor();
        let (pos, margin) = (pos * scale, margin * scale);
        self.takers.iter().chain(self.keep_off.iter()).any(|(node, place, shown)| {
            shown.get() && within_box(place.translation, node.size() * 0.5, pos, margin)
        })
    }
}

/// Whether `pos` is within the box of half-size `half` round `middle`,
/// grown by `margin` all round.
pub(crate) fn within_box(middle: Vec2, half: Vec2, pos: Vec2, margin: f32) -> bool {
    let off = (pos - middle).abs();
    off.x <= half.x + margin && off.y <= half.y + margin
}

/// A dialog is up, over everything else: nothing but its own buttons can be
/// pressed, you stand still, and on desktop the cursor is let go for clicking
/// them.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct DialogUp(pub bool);

/// Something out in the world is waiting on the pointer — a piece being put
/// down in House Builder, to be dragged and confirmed — so on desktop the
/// cursor is let go for as long as it is, the same as for a dialog. Unlike a
/// dialog, you can still walk about.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct WantsPointer(pub bool);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<WelcomeDue>()
        .init_resource::<DialogUp>()
        .init_resource::<WantsPointer>()
        .init_resource::<Held>()
        .init_resource::<Layout>()
        .init_resource::<Pictures>()
        .init_resource::<YourTag>()
        .add_observer(press_buttons)
        .add_systems(Startup, (show_top_bar, keep_icons))
        .add_systems(Update, shade)
        .add_systems(
            Update,
            (
                relayout,
                restyle.after(relayout),
                show_name.after(restyle),
                colour_name.after(restyle),
                count_balance.after(restyle),
                count_gems.after(restyle),
                show_health.after(restyle),
                show_resources.after(restyle),
                hide_round_the_jump,
                show_car_buttons.after(restyle),
                release_held,
                repicture.after(restyle),
                repaint,
                (arrive, welcome, hush_welcome, fade_headlines).chain(),
            ),
        );
}

/// The bar across the top: your name, the buttons and your balance; in a
/// game, the game's own ([`game_bar`]). What hangs anything of its own on a
/// bar put up afresh hangs it on `Added<TopBar>`.
#[derive(Component)]
pub(crate) struct TopBar;

/// The box round a phone's jump button, with the buttons about it, and
/// what the big speech bubble fans out from among them (`quick`).
#[derive(Component)]
pub(crate) struct RoundTheJump;

/// The fist among them, or in Bridge Wars the sword in the jump button's
/// place, which throws a punch (`start_punch` in `src/lib.rs`) and, in the
/// car, held, is the reverse (`drive`): its pictures, its own, and in the
/// car the jump button's arrow turned down. A game puts one up with
/// [`punch_button`].
#[derive(Component)]
pub(crate) struct PunchButton {
    picture: Handle<Image>,
    reverse: Handle<Image>,
}

/// The bag among them, where the face was, which opens your bag
/// (`bag::panel`, since 2026-11-21; Hajun: "im gonna change the emote
/// button in the lobby to a inventory/shop panel"). From 2026-11-05 until
/// then the face put you in the car and out of it ("make the emote button
/// to be a toggle on/off switch for car"): the car is in the bag now, and
/// the way out of it is the door ([`ExitButton`]).
#[derive(Component)]
pub(crate) struct BagButton;

/// The door, there while you are in your car, which gets you out of it
/// (`drive`; Hajun, 2026-11-21: "when ur in the car riding mode, a new ui
/// button ... that is like a leave door icon and u press that to get off ur
/// car"): where your bag's button is out of the car ([`BAG_PLACE`]; half
/// way down the left edge until 2026-11-22).
#[derive(Component)]
struct ExitButton;

/// What stands by the left edge of the screen, about half way down it: a
/// column, empty here, which `customize` puts its buttons in, the one that
/// begins changing what stands on your plot and, while you do, its three
/// (Hajun, 2026-11-22: "i want the left center spot to be a ui button that
/// only appears while you are in ur plot"). Taken down and put up with the
/// bar ([`relayout`], [`restyle`]), and put away while the camera is on
/// someone you talk to, as the buttons round the jump button are.
#[derive(Component)]
pub(crate) struct LeftEdge;

/// The gift, down the right edge, which in the car, held, is the gun
/// (`drive::shoot`; Hajun, 2026-11-06: "make the gift icon, while riding a
/// car, so that it shoots small fast spherical bullets out of the front of
/// the car while pressed"); out of the car it takes a press and drops it.
#[derive(Component)]
struct GunButton;

/// The plank over the sword in Bridge Wars, which turns building a bridge
/// as you walk on and off (`bridge`): lit while it is on.
#[derive(Component)]
pub(crate) struct BridgeButton;

/// The small keyboard over the speech bubble, in the town, and in the
/// same place in Bridge Wars ([`CHAT_BUTTON`]), which takes up the chat's
/// box to type a line into (`chat`). For a few hours of 2026-11-14 the big
/// speech bubble did that, in the town alone.
#[derive(Component)]
pub(crate) struct ChatButton;

/// The big speech bubble over the town's jump button, which fans out a few
/// things to say at a press (`quick`, since 2026-11-14; Hajun: "the current
/// larger chat icon button should be used for a quick chat selector for
/// preset chat messages that flannel out", which I take for "fan out").
#[derive(Component)]
pub(crate) struct QuickChatButton;

/// The buttons held down just now, each by the pointer holding it, a finger
/// or the mouse. A press on a button is picking's, which says when it goes
/// down but not, once the finger has slid off the button, when it comes up:
/// so each is held for as long as its pointer is down ([`release_held`]).
#[derive(Resource, Default)]
pub(crate) struct Held {
    fist: Option<PointerId>,
    gun: Option<PointerId>,
}

impl Held {
    /// Whether the fist is held down: in the car, the reverse.
    pub(crate) fn fist(&self) -> bool {
        self.fist.is_some()
    }

    /// Whether the gift is held down: in the car, the gun firing.
    pub(crate) fn gun(&self) -> bool {
        self.gun.is_some()
    }
}

/// A press on the fist, the car's door or the gift, as it goes down rather
/// than as it comes up, so that quick presses land as quickly as they are
/// made: a punch is asked for, and the fist is held from then; or out of
/// the car; or the gift is held from then.
fn press_buttons(
    press: On<Pointer<Press>>,
    fists: Query<(), With<PunchButton>>,
    faces: Query<(), With<ExitButton>>,
    guns: Query<(), With<GunButton>>,
    planks: Query<(), With<BridgeButton>>,
    mut held: ResMut<Held>,
    mut punches: MessageWriter<crate::PunchPressed>,
    mut cars: MessageWriter<CarPressed>,
    mut bridges: MessageWriter<BridgePressed>,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    if fists.contains(press.entity) {
        punches.write(crate::PunchPressed);
        held.fist = Some(press.pointer_id);
    }
    if faces.contains(press.entity) {
        cars.write(CarPressed);
    }
    if guns.contains(press.entity) {
        held.gun = Some(press.pointer_id);
    }
    if planks.contains(press.entity) {
        bridges.write(BridgePressed);
    }
}

/// Lets go of each held button once the pointer holding it is up.
fn release_held(touches: Res<Touches>, mouse: Res<ButtonInput<MouseButton>>, mut held: ResMut<Held>) {
    if let Some(pointer) = held.fist
        && !pointer_down(pointer, &touches, &mouse)
    {
        held.fist = None;
    }
    if let Some(pointer) = held.gun
        && !pointer_down(pointer, &touches, &mouse)
    {
        held.gun = None;
    }
}

/// Whether `pointer`, a finger or the mouse, is down.
fn pointer_down(pointer: PointerId, touches: &Touches, mouse: &ButtonInput<MouseButton>) -> bool {
    match pointer {
        PointerId::Touch(id) => touches.get_pressed(id).is_some(),
        PointerId::Mouse => mouse.pressed(MouseButton::Left),
        PointerId::Custom(_) => false,
    }
}

/// Shows the car's controls on the buttons as you get in and out of it
/// (`drive`): on the fist the jump button's arrow turned down, for the
/// reverse; and on buttons put up afresh ([`restyle`]). (Until 2026-11-21
/// the face was lit too while you were in it: the bag that stands there
/// now is put away instead, [`show_car_buttons`].)
fn repicture(
    driving: Res<Driving>,
    fists: Query<(&PunchButton, &Children)>,
    added: Query<(), Added<PunchButton>>,
    mut pictures: Query<&mut ImageNode>,
) {
    if !driving.is_changed() && added.is_empty() {
        return;
    }
    let on = driving.on();
    for (button, children) in &fists {
        for child in children.iter() {
            let Ok(mut picture) = pictures.get_mut(child) else {
                continue;
            };
            let (image, flipped) = if on { (&button.reverse, true) } else { (&button.picture, false) };
            if picture.image != *image || picture.flip_y != flipped {
                picture.image = image.clone();
                picture.flip_y = flipped;
            }
        }
    }
}

/// Puts the three buttons round the jump button away while the camera is on
/// someone you talk to (`talk`), as the stick and the jump button are
/// (Hajun, 2026-11-01: "the punch, quickchat, and emote button is not
/// hidden. hide those too").
fn hide_round_the_jump(
    close_up: Res<crate::CloseUp>,
    mut boxes: Query<&mut Visibility, Or<(With<RoundTheJump>, With<LeftEdge>)>>,
) {
    let shown = if close_up.on.is_some() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut boxes {
        visibility.set_if_neq(shown);
    }
}

/// Shows the bag only where it is to be had, the town alone, which is
/// where the car and your plot are (`drive::drivable`): the town's buttons
/// are up at home and in House Builder's lobby and plots too, where it
/// would be a button that does nothing, and there it is put away, the
/// others staying where they are. **In your car it is put away too, and
/// the door out of the car is there instead**, in its place: the bag is
/// not opened from the car (mine: Hajun asked for the door, and said
/// nothing of the bag there). Written only as it changes.
fn show_car_buttons(
    me: Query<&Venue, With<Player>>,
    driving: Res<Driving>,
    mut bags: Query<&mut Visibility, (With<BagButton>, Without<ExitButton>)>,
    mut doors: Query<&mut Visibility, (With<ExitButton>, Without<BagButton>)>,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    let shown = |shown: bool| if shown { Visibility::Inherited } else { Visibility::Hidden };
    let in_the_car = driving.on();
    for mut bag in &mut bags {
        bag.set_if_neq(shown(crate::drive::drivable(here) && !in_the_car));
    }
    for mut door in &mut doors {
        door.set_if_neq(shown(in_the_car));
    }
}

/// How far under its words every text's shadow falls, as a share of the
/// letters' size ([`shade`]): what 3 logical pixels was under the bar's
/// words on this PC.
const SHADOW_DROP: f32 = 0.12;

/// Sets every text's shadow as far under its words as its size asks, on
/// every screen alike (since 2026-11-10, Hajun, of the names over the
/// health bars in Bridge Wars: "the shadow of the nametag is too far below
/// the text": the shadow fell 3 logical pixels whatever the words' size,
/// which under those small names, on a phone whose logical pixels are
/// large, was about a third of the letters' height). Only a shadow that
/// has to move is written.
fn shade(window: Query<&Window, With<bevy::window::PrimaryWindow>>, mut texts: Query<(&TextFont, &mut TextShadow)>) {
    let Ok(window) = window.single() else {
        return;
    };
    let (width, height) = (window.width(), window.height());
    for (font, mut shadow) in &mut texts {
        let size = match font.font_size {
            FontSize::Px(px) => px,
            FontSize::Vw(share) => share * width / 100.0,
            FontSize::Vh(share) => share * height / 100.0,
            FontSize::VMin(share) => share * width.min(height) / 100.0,
            FontSize::VMax(share) => share * width.max(height) / 100.0,
            _ => continue,
        };
        let offset = Vec2::new(0.0, (size * SHADOW_DROP).max(0.5));
        if shadow.offset != offset {
            shadow.offset = offset;
        }
    }
}

/// Text in the house style: the default font, sized as a share of the short
/// side of the screen, with a soft shadow so that it reads over sky and grass
/// alike. The shadow is as see-through as the words, and falls under them
/// by a share of their size ([`shade`]).
pub(crate) fn words(text: impl Into<String>, size_vmin: f32, colour: Color) -> impl Bundle {
    let shadow = ui().colours.text_shadow;
    (
        Text::new(text),
        crate::font::sized(size_vmin),
        TextColor(colour),
        TextShadow {
            offset: Vec2::new(0.0, 3.0),
            color: shadow.with_alpha(shadow.alpha() * colour.alpha()),
        },
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    )
}

/// Everything round the edges of the screen, for as long as the app is open:
/// top left your name, the buttons beside it and your balance; down the
/// right edge three buttons; and round a phone's jump button three more,
/// placed about where it is on every platform, so that the layout can be
/// seen on desktop too.
pub(crate) fn show_top_bar(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    layout: Res<Layout>,
    minimap: Res<Minimap>,
) {
    spawn_bar(&mut commands, &assets, &mut images, *layout, &minimap);
}

/// Puts up the buttons of wherever you are ([`Layout`]) as you arrive
/// there: Bridge Wars' own in a game of it, each of the four games' own at
/// its table, and the town's everywhere else; taken down and put up
/// afresh, as [`restyle`] does.
fn relayout(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    mut layout: ResMut<Layout>,
    bars: Query<Entity, Or<(With<TopBar>, With<RoundTheJump>, With<LeftEdge>)>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    minimap: Res<Minimap>,
    mut commands: Commands,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    let wanted = Layout::of(here);
    if *layout == wanted {
        return;
    }
    *layout = wanted;
    for bar in &bars {
        commands.entity(bar).despawn();
    }
    spawn_bar(&mut commands, &assets, &mut images, wanted, &minimap);
}

/// Draws the bar and the buttons round the jump button again in the file's
/// colours as it changes on desktop (`ui::reread`): taken down and put up
/// afresh, which [`show_name`] and [`count_balance`] then fill in, and
/// `settings` puts its gear back on.
fn restyle(
    changed: Res<UiChanged>,
    layout: Res<Layout>,
    bars: Query<Entity, Or<(With<TopBar>, With<RoundTheJump>, With<LeftEdge>)>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    minimap: Res<Minimap>,
    mut commands: Commands,
) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    for bar in &bars {
        commands.entity(bar).despawn();
    }
    spawn_bar(&mut commands, &assets, &mut images, *layout, &minimap);
}

/// The bar across the top, and the box round the jump button, as `layout`
/// has them: the four games' each by its own module, which builds it of
/// the pieces here ([`game_bar`], [`name_plate`], [`round_the_jump`] and
/// the rest).
fn spawn_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>, layout: Layout, minimap: &Minimap) {
    match layout {
        Layout::Town => spawn_town_bar(commands, assets, images),
        Layout::Arena => spawn_arena_bar(commands, images, minimap),
        Layout::BlendIn => crate::blend_in::spawn_bar(commands, assets, images),
        Layout::KnockOff => crate::knock_off::spawn_bar(commands, assets, images),
        Layout::BridgeRace => crate::bridge_race::spawn_bar(commands, assets, images),
        Layout::LastStand => crate::last_stand::spawn_bar(commands, assets, images),
    }
}

/// The bar across the top in a game of one of the four (`table`), as Bridge
/// Wars' is ([`spawn_arena_bar`]): moved in from whichever side the
/// camera's hole is on, and on the right no further than there is room,
/// with a column down its left for your name and what goes under it, and
/// one down its right. Each game's own `spawn_bar` puts what it has in
/// them (`ChildOf`): the left one first, then the right.
pub(crate) fn game_bar(commands: &mut Commands) -> (Entity, Entity) {
    let bar = commands
        .spawn((
            TopBar,
            KeepsClear,
            bar_room(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(EDGE),
                left: Val::VMin(EDGE),
                right: Val::VMin(EDGE),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let left = commands
        .spawn((
            ChildOf(bar),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::VMin(1.4),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let right = commands
        .spawn((
            ChildOf(bar),
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::VMin(1.6),
                // A little further in from the corner than the name is, as
                // the town's buttons down the right edge are.
                margin: UiRect::new(Val::ZERO, Val::VMin(SIDE_INSET), Val::VMin(SIDE_INSET), Val::ZERO),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    (left, right)
}

/// The button that throws a punch, with `picture` on it (the fist, or a
/// sword), `size` across, where `place` puts it; in the car, were you in
/// it, the reverse, with `reverse` on it (the jump button's arrow, which
/// is turned down for it: [`PunchButton`]).
pub(crate) fn punch_button(picture: Handle<Image>, reverse: Handle<Image>, size: f32, place: Node) -> impl Bundle {
    (
        PunchButton {
            picture: picture.clone(),
            reverse,
        },
        round_button(picture, size, place),
    )
}

/// The small keyboard that takes up the chat's box to type a line into
/// ([`ChatButton`]), where the town and Bridge Wars have it
/// ([`CHAT_BUTTON`]), stepping in from an iPhone's island: for the box
/// round the jump button.
pub(crate) fn chat_button(images: &mut Assets<Image>) -> impl Bundle {
    let (right, bottom, size) = CHAT_BUTTON;
    (ChatButton, chat_steps_in(), round_button(images.add(keyboard()), size, beside_the_jump(right, bottom)))
}

/// The box round the jump button, in the bottom right corner where the
/// jump button is: the buttons about it are placed from its corner, in
/// shares of the short side.
pub(crate) fn round_the_jump() -> impl Bundle {
    (
        RoundTheJump,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Vw(crate::JUMP_RIGHT_VW),
            bottom: Val::Vh(crate::JUMP_BOTTOM_VH),
            width: Val::VMin(crate::JUMP_SIZE_VMIN),
            height: Val::VMin(crate::JUMP_SIZE_VMIN),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// Where a button sits in the box round the jump button, in shares of the
/// short side from its right and its bottom.
pub(crate) fn beside_the_jump(right: f32, bottom: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::VMin(right),
        bottom: Val::VMin(bottom),
        ..default()
    }
}

/// Bridge Wars' buttons (Hajun's `assets/pvpmapbuttonui.jpg`, the night of 2026-11-07):
/// top left your name, your health bar and under it the four resources
/// with their counts (since 2026-11-09, Hajun: "move the 4 item icon +
/// number thing on the top right side of that screen to the left side of
/// the screen below the hp bar"; top right under a bag until then): what
/// you hold of what the generators make, silver, gold, sapphire and
/// amethyst, each pictured as its model is seen from above and tinted its
/// colour (since 2026-11-11, Hajun: "the ui icon for the inventory icons
/// and number will be adjusted accordingly to silver, gold, sapphire,
/// amethyst (so remove the log icon - the bridge meter only shows as in
/// map text in front of the bridge like now) so change the icon to match
/// the model shape"; a log with the metres of bridge left, an ingot, a gem
/// and an emerald until then, all white); top right the minimap
/// (`minimap`, in the bag's place); and bottom right the sword in the jump
/// button's place, which punches (`start_punch`), an empty slot up and to
/// the left of it, the plank over it, which turns building on and off
/// (`bridge`), a small one over it, and the three dots over those, which
/// does nothing yet.
fn spawn_arena_bar(commands: &mut Commands, images: &mut Assets<Image>, minimap: &Minimap) {
    let colours = ui().colours;
    commands
        .spawn((
            TopBar,
            // Moved in from whichever side the camera's hole is on, and on
            // the right no further than there is room.
            KeepsClear,
            bar_room(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(EDGE),
                left: Val::VMin(EDGE),
                right: Val::VMin(EDGE),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::VMin(1.4),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|left| {
                left.spawn(name_plate());
                left.spawn(health_bar());
                for item in Item::ALL {
                    left.spawn(resource_row(item, images.add(item_picture(item)), colours.arena.item(item)));
                }
            });
            bar.spawn((
                MinimapNode,
                ImageNode::new(minimap.image.clone()),
                Node {
                    width: Val::VMin(MINIMAP),
                    height: Val::VMin(MINIMAP),
                    margin: UiRect::new(Val::ZERO, Val::VMin(SIDE_INSET), Val::VMin(SIDE_INSET), Val::ZERO),
                    border: UiRect::all(Val::VMin(MINIMAP_RIM)),
                    border_radius: BorderRadius::all(Val::VMin(1.6)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(colours.arena.map_back),
                BorderColor::all(colours.arena.hp_rim),
                Pickable::IGNORE,
            ));
        });
    commands.spawn(round_the_jump()).with_children(|round| {
        let sword = images.add(sword());
        let arrow = images.add(jump_arrow());
        round.spawn((
            PunchButton {
                picture: sword.clone(),
                reverse: arrow.clone(),
            },
            round_button(sword, crate::JUMP_SIZE_VMIN, beside_the_jump(0.0, 0.0)),
        ));
        // Round the sword, on an arc: the jump button beside it, which
        // takes no press for itself, as the one in its own place does not
        // (the touch controls read it, `hud_layout`), an empty slot, and
        // over the sword the plank, which turns building a bridge on and
        // off (`bridge`).
        let (right, bottom, size) = ARENA_JUMP;
        round.spawn(jump_button(arrow, size, beside_the_jump(right, bottom)));
        // The buttons for things to use: the second button, and four more
        // on a ring round these, put away until something is on them or
        // the dots show them (`utility`).
        for slot in 0..UTILITY_SLOTS {
            let (right, bottom) = slot_place(slot);
            round.spawn(utility_slot(slot, beside_the_jump(right, bottom)));
        }
        let (right, bottom) = ARENA_BRIDGE;
        round.spawn(bridge_button(images.add(plank()), beside_the_jump(right, bottom)));
        // The chat: a line to type (`chat`), where the town has it.
        let (right, bottom, size) = CHAT_BUTTON;
        round.spawn((ChatButton, chat_steps_in(), round_button(images.add(keyboard()), size, beside_the_jump(right, bottom))));
        // The three dots, which showed the empty buttons, are not put up
        // ([`DOTS`]).
        if DOTS {
            let (right, bottom, size) = ARENA_DOTS;
            round.spawn((MoreButton, round_button(images.add(dots()), size, beside_the_jump(right, bottom))));
        }
    });
}

/// Bridge Wars' bridge button, where `place` puts it: a round button as
/// the others are ([`round_button`]), with the plank on it a little above
/// its middle and, under the plank, inside the disc, the metres of bridge
/// you have, small and white ([`BridgeCount`], written by
/// [`show_resources`]: "25m", as in front of you while you build). A
/// press anywhere on it is the button's.
pub(crate) fn bridge_button(plank: Handle<Image>, place: Node) -> impl Bundle {
    let colours = ui().colours;
    (
        BridgeButton,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(ACTION_BUTTON),
            height: Val::VMin(ACTION_BUTTON),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..place
        },
        BackgroundColor(colours.hud.button),
        children![
            (
                ImageNode::new(plank).with_color(colours.hud.button_mark),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent((100.0 - PICTURE) * 0.5),
                    top: Val::Percent((100.0 - PICTURE) * 0.5 - BRIDGE_PLANK_UP),
                    width: Val::Percent(PICTURE),
                    height: Val::Percent(PICTURE),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    right: Val::ZERO,
                    top: Val::Percent(BRIDGE_COUNT_TOP),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
                children![(BridgeCount, words("", BRIDGE_COUNT_TEXT, colours.arena.bridge_count))],
            ),
        ],
    )
}

/// A jump button, `size_vmin` across, where `place` puts it, as the one
/// in its own place looks (`setup_hud` in `src/lib.rs`): the buttons' disc
/// with the arrow on it, lit while it is held (`update_jump_visual`). It
/// takes no press for itself: the touch controls know where it is.
pub(crate) fn jump_button(arrow: Handle<Image>, size_vmin: f32, place: Node) -> impl Bundle {
    let colours = ui().colours.hud;
    (
        crate::JumpButtonVisual,
        Node {
            width: Val::VMin(size_vmin),
            height: Val::VMin(size_vmin),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..place
        },
        BackgroundColor(colours.button),
        Pickable::IGNORE,
        children![(
            ImageNode::new(arrow).with_color(colours.button_mark),
            Node {
                width: Val::Percent(JUMP_ARROW),
                height: Val::Percent(JUMP_ARROW),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// Your health bar in Bridge Wars, under your name: a bar with a rim,
/// filled red as far as your health goes ([`show_health`]).
pub(crate) fn health_bar() -> impl Bundle {
    let colours = ui().colours.arena;
    (
        Node {
            width: Val::VMin(HP_WIDTH),
            height: Val::VMin(HP_HEIGHT),
            margin: UiRect::left(Val::VMin(0.6)),
            border: UiRect::all(Val::VMin(HP_RIM)),
            border_radius: BorderRadius::all(Val::VMin(0.8)),
            ..default()
        },
        BackgroundColor(colours.hp_back),
        BorderColor::all(colours.hp_rim),
        Pickable::IGNORE,
        children![(
            HealthFill,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::VMin(0.5)),
                ..default()
            },
            BackgroundColor(colours.hp),
            Pickable::IGNORE,
        )],
    )
}

/// Fills your health bar as far as your health goes (`arena::Health` on
/// your body, in a game of Bridge Wars; full with none), as it changes and
/// on a bar put up afresh.
fn show_health(
    me: Query<Ref<Health>, With<Player>>,
    added: Query<(), Added<HealthFill>>,
    mut fills: Query<&mut Node, With<HealthFill>>,
) {
    let health = me.single().ok();
    let changed = health.as_ref().is_some_and(|health| health.is_changed());
    if !changed && added.is_empty() {
        return;
    }
    // In steps of half a percent, so that health coming back a little
    // every frame does not have the screen laid out again every frame.
    let width = Val::Percent(health.map_or(200.0, |health| (health.share() * 200.0).round()) * 0.5);
    for mut fill in &mut fills {
        if fill.width != width {
            fill.width = width;
        }
    }
}

/// One of the things you hold in Bridge Wars, under your health bar: its
/// picture, and its count beside it ([`show_resources`]), in a row wide
/// enough that the pictures line up under each other.
pub(crate) fn resource_row(kind: Item, picture: Handle<Image>, tint: Color) -> impl Bundle {
    (
        Node {
            width: Val::VMin(RESOURCE_ROW),
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.4),
            padding: UiRect::left(Val::VMin(1.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                ImageNode::new(picture).with_color(tint),
                Node {
                    width: Val::VMin(RESOURCE_ICON),
                    height: Val::VMin(RESOURCE_ICON),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ),
            (ResourceCount(kind), words("0", RESOURCE_TEXT, ui().colours.text)),
        ],
    )
}

/// Writes how many of each thing you hold beside its picture, and the
/// metres of bridge you have inside the bridge button ("25m", as they are
/// written in front of you while you build), as they change and on a bar
/// put up afresh.
fn show_resources(
    stock: Res<Resources>,
    added: Query<(), Or<(Added<ResourceCount>, Added<BridgeCount>)>>,
    mut counts: Query<(&ResourceCount, &mut Text), Without<BridgeCount>>,
    mut metres: Query<&mut Text, With<BridgeCount>>,
) {
    if !stock.is_changed() && added.is_empty() {
        return;
    }
    for (count, mut text) in &mut counts {
        let number = stock.count(count.0);
        if text.0 != number {
            text.0 = number;
        }
    }
    for mut text in &mut metres {
        let left = stock.metres();
        if text.0 != left {
            text.0 = left;
        }
    }
}

/// One of Bridge Wars' buttons for things to use, where `place` puts it:
/// the second button (since 2026-11-12; an empty slot until then), which
/// is `slot` 0, or one of the four more (since 2026-11-13). Each is put
/// away until something is on it (the second button too since
/// 2026-11-14, `utility::slot_shown`; it was there empty until then). The
/// buttons' disc with a faint rim, and on it the picture of what was
/// bought for it at the item shop, how many of it in its corner, and
/// over it, while a potion lasts, how long it still does. `utility` fills
/// it in, shows it and takes its presses: a press for a potion, a hold
/// and a drag for what is aimed.
pub(crate) fn utility_slot(slot: usize, place: Node) -> impl Bundle {
    let colours = ui().colours;
    // The time over the second button stands a quarter of the button to
    // the right of its middle: straight over it, it ran into the count in
    // the corner of the button up and to its left, a "1" and "43s" read as
    // "143s". No other button has one there.
    let lean = if slot == 0 { UTILITY_TIMER_LEAN } else { 0.0 };
    (
        UtilitySlot(slot),
        Visibility::Hidden,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(ACTION_BUTTON),
            height: Val::VMin(ACTION_BUTTON),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::VMin(0.3)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..place
        },
        BackgroundColor(colours.hud.button),
        BorderColor::all(colours.arena.slot_rim),
        children![
            (
                UtilityPicture(slot),
                ImageNode::default(),
                Node {
                    width: Val::Percent(PICTURE),
                    height: Val::Percent(PICTURE),
                    ..default()
                },
                Visibility::Hidden,
                Pickable::IGNORE,
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(2.0),
                    bottom: Val::Percent(-4.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![(UtilityCount(slot), words("", UTILITY_COUNT_TEXT, colours.text))],
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(-50.0 + lean),
                    right: Val::Percent(-50.0 - lean),
                    bottom: Val::Percent(104.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
                children![(UtilityTimer(slot), words("", UTILITY_TIMER_TEXT, colours.text))],
            ),
        ],
    )
}

/// One of Bridge Wars' buttons for things to use ([`utility_slot`]), its
/// picture, the count in its corner and the time over it, each by which
/// button it is, from 0, the second button (`Kit::slots`).
#[derive(Component)]
pub(crate) struct UtilitySlot(pub(crate) usize);
#[derive(Component)]
pub(crate) struct UtilityPicture(pub(crate) usize);
#[derive(Component)]
pub(crate) struct UtilityCount(pub(crate) usize);
#[derive(Component)]
pub(crate) struct UtilityTimer(pub(crate) usize);

/// The three dots over Bridge Wars' buttons, which show the four more
/// buttons for things to use, empty as they may be, and put the empty ones
/// away again (`utility::MoreSlots`): lit while they are shown.
#[derive(Component)]
pub(crate) struct MoreButton;

/// How big the count in the second button's corner and the time over it
/// are, in shares of the short side.
const UTILITY_COUNT_TEXT: f32 = 3.4;
const UTILITY_TIMER_TEXT: f32 = 3.0;
/// How far to the right of its button's middle the time over the second
/// button stands, as a share of the button ([`utility_slot`]).
const UTILITY_TIMER_LEAN: f32 = 25.0;

/// The town's buttons, as Hajun sketched them (`assets/UIbuttonlayout.jpg`,
/// 2026-10-31): the bar across the top, and the box round the jump button.
fn spawn_town_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>) {
    let icon = |name: &str| assets.load(format!("icons/{name}.png"));
    commands
        .spawn((
            TopBar,
            // Moved in from whichever side the camera's hole is on, and on
            // the right no further than there is room.
            KeepsClear,
            bar_room(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(EDGE),
                left: Val::VMin(EDGE),
                right: Val::VMin(EDGE),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::VMin(1.4),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|left| {
                // Your name, and beside it the buttons that are about you:
                // a person, while there is one ([`PERSON`]), and the
                // settings' gear, which `settings` puts after it.
                left.spawn((
                    BarLeft,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::VMin(1.4),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn(name_plate());
                    if PERSON {
                        row.spawn(round_button(icon("person"), SMALL_BUTTON, Node::default()));
                    }
                });
                left.spawn(balance(assets));
                left.spawn(gems(assets));
            });
            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::VMin(1.6),
                    // A little further in from the corner than the name is.
                    margin: UiRect::new(Val::ZERO, Val::VMin(SIDE_INSET), Val::VMin(SIDE_INSET), Val::ZERO),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|right| {
                // A leaderboard stood between the gift and the group until
                // Hajun had it taken out (2026-11-01), and the group became
                // a speech bubble the same day, and two people again on
                // 2026-11-14 ([`ICONS`]). The gift is away for now
                // ([`GIFT`]), and the other two each a place higher for it.
                for name in ["gift", "friends", "gamepad"] {
                    if name == "gift" && !GIFT {
                        continue;
                    }
                    let button = round_button(icon(name), SIDE_BUTTON, Node::default());
                    match name {
                        // In the car, held, the gun (`drive::shoot`).
                        "gift" => {
                            right.spawn((GunButton, button));
                        }
                        // The game selector (`games`).
                        "gamepad" => {
                            right.spawn((GamesButton, button));
                        }
                        // Your friends (`friends`), which hangs a count of
                        // what waits in your inbox on its corner.
                        _ => {
                            right.spawn((FriendsButton, button));
                        }
                    }
                }
            });
        });
    // Round the jump button: a box where it is, and the three about it, each
    // placed from the box's corner, in shares of the short side: your bag's
    // up and to the left of it, where the face was, the speech bubble over
    // it and the fist beside it, each about 2.5 shares clear of it, as Hajun
    // sketched them; and over the bubble, a little to the right, the chat's
    // small button.
    commands.spawn(round_the_jump()).with_children(|round| {
        let fist = images.add(fist());
        let arrow = images.add(jump_arrow());
        // On your bag's button, the box that is on its window's first tab.
        let pack = images.add(if SACK { bag() } else { cube() });
        let (quick_right, quick_bottom, _) = QUICK_BUTTON;
        let (bag_right, bag_bottom) = BAG_PLACE;
        for (name, right, bottom) in [("bag", bag_right, bag_bottom), ("social", quick_right, quick_bottom), ("punch", 24.3, 1.2)] {
            let picture = match name {
                "punch" => fist.clone(),
                "bag" => pack.clone(),
                name => icon(name),
            };
            let mut button = round.spawn(round_button(picture, ACTION_BUTTON, beside_the_jump(right, bottom)));
            // The fist throws a punch each time it is pressed, and the
            // box opens your bag (`bag::panel`); in the car the fist is
            // the reverse (`drive`).
            match name {
                "punch" => {
                    button.insert(PunchButton {
                        picture: fist.clone(),
                        reverse: arrow.clone(),
                    });
                }
                "bag" => {
                    button.insert(BagButton);
                }
                // Things to say at a press (`quick`).
                _ => {
                    button.insert(QuickChatButton);
                }
            }
        }
        // The chat: a line to type (`chat`).
        let (right, bottom, size) = CHAT_BUTTON;
        round.spawn((ChatButton, chat_steps_in(), round_button(images.add(keyboard()), size, beside_the_jump(right, bottom))));
        // The door out of your car, there while you are in it, where your
        // bag's button is while you are not ([`show_car_buttons`]).
        round.spawn((
            ExitButton,
            Visibility::Hidden,
            round_button(icon("leave"), ACTION_BUTTON, beside_the_jump(bag_right, bag_bottom)),
        ));
    });
    // By the left edge, as far in as your name and kept clear of a
    // camera's hole as it is: the column `customize` puts its buttons in,
    // which grows up from its foot.
    commands.spawn((
        LeftEdge,
        KeepsClear,
        Node {
            position_type: PositionType::Absolute,
            left: Val::VMin(EDGE),
            bottom: Val::Vh(100.0 - LEFT_FOOT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::VMin(LEFT_GAP),
            ..default()
        },
        Pickable::IGNORE,
    ));
}

/// The row top left: your name, and whatever goes with it.
#[derive(Component)]
pub(crate) struct BarLeft;

/// A round button in the style of the stick and the jump button: a
/// see-through grey disc, `size_vmin` across, with `icon` on it in a
/// see-through white, and no rim; where `place` puts it. It takes a press
/// for itself; what the press does, if anything, is whoever spawns it to
/// say.
pub(crate) fn round_button(icon: Handle<Image>, size_vmin: f32, place: Node) -> impl Bundle {
    let colours = ui().colours.hud;
    (
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size_vmin),
            height: Val::VMin(size_vmin),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..place
        },
        BackgroundColor(colours.button),
        children![(
            ImageNode::new(icon).with_color(colours.button_mark),
            Node {
                width: Val::Percent(PICTURE),
                height: Val::Percent(PICTURE),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

// (The cross that shuts a window is the windows' own, a bare white one
// inside its corner: `window::cross`. Until 2026-11-14 there was one here,
// a grey disc hung over the corner with two white strokes across it, for
// the settings, the game selector and Bridge Wars' shops; House Builder's
// shop still draws its own like it, `shop::close_button`.)

/// Your name on a plate: the buttons' grey, a little more solid, with its
/// corners rounded a little. Under the name, laid out by themselves over
/// the same room, the four copies of it that are its stroke, put away
/// until the name is in your team's colour ([`colour_name`]).
pub(crate) fn name_plate() -> impl Bundle {
    let colours = ui().colours;
    (
        Node {
            padding: UiRect::axes(Val::VMin(2.4), Val::VMin(1.0)),
            border_radius: BorderRadius::all(Val::VMin(1.6)),
            ..default()
        },
        BackgroundColor(colours.hud.name_plate),
        Pickable::IGNORE,
        children![(
            Node::default(),
            Pickable::IGNORE,
            children![
                name_rim(Vec2::new(-1.0, -1.0)),
                name_rim(Vec2::new(1.0, -1.0)),
                name_rim(Vec2::new(-1.0, 1.0)),
                name_rim(Vec2::new(1.0, 1.0)),
                (
                    NamePlate,
                    Text::new(""),
                    crate::font::sized(NAME_TEXT),
                    TextColor(colours.text),
                    Pickable::IGNORE,
                ),
            ],
        )],
    )
}

/// One of the four copies of your name that make its stroke, set off
/// toward `corner`: in the colour of the stroke round the chat's words
/// (the file's `colours.chat.stroke`), and not drawn until it is wanted.
fn name_rim(corner: Vec2) -> impl Bundle {
    (
        NameRim,
        Text::new(""),
        crate::font::sized(NAME_TEXT),
        TextColor(ui().colours.chat.stroke),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::from_translation(Val2::new(Val::VMin(corner.x * NAME_STROKE), Val::VMin(corner.y * NAME_STROKE))),
        Visibility::Hidden,
        Pickable::IGNORE,
    )
}

/// Shows your name on its plate, as you are called now, with your tag after
/// it once the server has said what it is ([`YourTag`]): as the app opens,
/// again when you sign up or change it (`signup`), as the tag comes and
/// goes with the connection, and on a plate put up afresh ([`restyle`]).
fn show_name(
    me: Query<Ref<Name>, With<Player>>,
    tag: Res<YourTag>,
    // (The name, and the four copies of it that are its stroke.)
    mut plates: Query<&mut Text, Or<(With<NamePlate>, With<NameRim>)>>,
) {
    let Ok(name) = me.single() else {
        return;
    };
    for mut text in &mut plates {
        if name.is_changed() || tag.is_changed() || text.is_added() {
            let shown = on_the_plate(&name, tag.0);
            if text.0 != shown {
                text.0 = shown;
            }
        }
    }
}

/// Colours your name on its plate: in a game of Bridge Wars, from the
/// moment the fight begins, your team's colour, as it is in the chat, and
/// as your teammate's is over their head (since 2026-11-15; Hajun: "in the
/// bridge wars minigame, make the color of your name that u see in the top
/// left corner like NAME#5571 to also be the color of your team"); anywhere
/// else, and in the game's waiting room and through its countdown, the
/// colour of words (it was your team's from the waiting room on until
/// 2026-11-16; Hajun: "your team selection should not be shown/determine
/// until the game start. so when ur in the top middle box, in waiting
/// room, ur team color should not be displayed"; `Fighting::teams_out`). **In its team's
/// colour it has a thin black stroke round it**, as the chat's words have
/// (mine): the plate is see-through, light by day with the sky behind it,
/// and on it the blue team's name could not be read, nor the green's and
/// the red's easily (seen in the desktop build, by day). Written only as
/// it changes.
///
/// The team is the one on your body ([`Team`]), and since 2026-11-18 it is
/// so in any game laid out as Bridge Wars is ([`Layout::fights`]: Bridge
/// Race, Last Stand) once its teams are out, which each says as Bridge
/// Wars does (`Fighting::teams_out`); in Bridge Wars your body's team is
/// your seat's (`arena_team`), as it was read here until then.
fn colour_name(
    layout: Res<Layout>,
    fighting: Res<Fighting>,
    me: Query<Option<&Team>, With<Player>>,
    mut plates: Query<&mut TextColor, With<NamePlate>>,
    mut rims: Query<&mut Visibility, With<NameRim>>,
) {
    let colours = ui().colours;
    let team = me
        .single()
        .ok()
        .flatten()
        .filter(|_| layout.fights() && fighting.teams_out)
        .map(|team| team.0);
    let wanted = team.map_or(colours.text, |team| colours.arena.team(team));
    for mut colour in &mut plates {
        colour.set_if_neq(TextColor(wanted));
    }
    let stroke = if team.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    for mut visibility in &mut rims {
        visibility.set_if_neq(stroke);
    }
}

/// What the plate says of someone called `name` whose tag is `tag`, if it
/// is known: `Hajooon#5571`, or `Hajooon` alone.
fn on_the_plate(name: &str, tag: Option<u16>) -> String {
    match tag {
        Some(tag) => roundtown_net::social::handle(name, tag),
        None => name.to_string(),
    }
}

// ---------------------------------------------------------------- balance

/// Your balance: the coin and the number, under your name, on nothing (as
/// Hajun sketched it; on a dark glass until 2026-10-31).
fn balance(assets: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.4),
            padding: UiRect::left(Val::VMin(1.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                ImageNode::new(assets.load(COIN_ICON)),
                Node {
                    width: Val::VMin(BALANCE_ICON),
                    height: Val::VMin(BALANCE_ICON),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            (BalanceText, words(money(STARTING_BALANCE), BALANCE_TEXT, ui().colours.text)),
        ],
    )
}

/// Your gems: the gem and how many, under your coins, as Hajun drew them
/// (`bagandinventorypanel1.png`, 2026-11-21), the gem itself as wide as
/// the coin itself and under its middle ([`GEM_WIDE`]), and the number the
/// coins' size, beginning where theirs does.
fn gems(assets: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.4),
            padding: UiRect::left(Val::VMin(1.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                ImageNode::new(assets.load(GEM_ICON)),
                Node {
                    width: Val::VMin(GEM_WIDE),
                    height: Val::VMin(GEM_WIDE * GEM_SHAPE),
                    // In the room the coin's picture has over it.
                    margin: UiRect::horizontal(Val::VMin((BALANCE_ICON - GEM_WIDE) * 0.5)),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            (GemsText, words(money(roundtown_net::bag::GEMS), BALANCE_TEXT, ui().colours.text)),
        ],
    )
}

/// Shows how many gems you have whenever your bag changes, and on a number
/// put up afresh ([`restyle`]).
fn count_gems(bag: Res<Bag>, mut texts: Query<&mut Text, With<GemsText>>) {
    for mut text in &mut texts {
        if !(bag.is_changed() || text.is_added()) {
            continue;
        }
        let label = money(bag.has.gems);
        if text.0 != label {
            text.0 = label;
        }
    }
}

/// Shows your balance whenever it changes, and on a number put up afresh
/// ([`restyle`]).
fn count_balance(me: Query<Ref<Balance>, With<Player>>, mut texts: Query<&mut Text, With<BalanceText>>) {
    let Ok(balance) = me.single() else {
        return;
    };
    for mut text in &mut texts {
        if !(balance.is_changed() || text.is_added()) {
            continue;
        }
        let label = money(balance.0);
        if text.0 != label {
            text.0 = label;
        }
    }
}

/// `12,500`.
pub(crate) fn money(amount: u32) -> String {
    let digits = amount.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (n, digit) in digits.chars().enumerate() {
        if n > 0 && (digits.len() - n).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

// ---------------------------------------------------------------- buttons

/// A round button's fill, the dark glass, darker under the pointer and held.
pub(crate) fn button_fill(interaction: Interaction) -> Color {
    let panel = ui().colours.panel;
    let alpha = match interaction {
        Interaction::None => panel.alpha(),
        Interaction::Hovered => 0.56,
        Interaction::Pressed => 0.7,
    };
    panel.with_alpha(alpha)
}

/// Whenever you land on an island — in the town when the app opens, and back
/// in it after a game of House Builder — and it is the town, has you
/// welcomed.
fn arrive(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    welcomes: Query<Entity, With<Welcome>>,
    mut due: ResMut<WelcomeDue>,
    mut commands: Commands,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    // One still up from the last time is out of date, whichever way you went.
    for welcome in &welcomes {
        commands.entity(welcome).despawn();
    }
    due.0 = here == Venue::Town;
}

// -------------------------------------------------------------- headlines

/// The welcome, once there is a town to see behind it. On the phone the first
/// one can come a second or two before the island has loaded; and the first
/// time the app is run, it waits for the sign-up's pages to go (`signup`),
/// which stand over the town.
fn welcome(
    islands: Res<Islands>,
    signup: Res<Signup>,
    intro: Res<crate::intro::Intro>,
    mut due: ResMut<WelcomeDue>,
    mut commands: Commands,
) {
    // As the app opens, the town's name is the opening's own: on its cover,
    // and over the town for a moment as the cover opens (`intro`).
    if due.0 && intro.welcomes() {
        due.0 = false;
    }
    if !due.0 || signup.up() || islands.get(Venue::Town).is_none() {
        return;
    }
    due.0 = false;
    announce(&mut commands, None, &ui().words.welcome, HEADLINE, WELCOME_SECS).insert(Welcome);
}

/// While a window is up over the town (anything that holds [`DialogUp`]),
/// the town's name, if it is on the screen as a welcome, goes at once: in
/// [`HUSH_SECS`], from however strong it is (it stood over the game
/// selector, the settings and the friends' panel for the rest of its three
/// and a half seconds until 2026-11-18).
fn hush_welcome(up: Res<DialogUp>, mut welcomes: Query<&mut Headline, With<Welcome>>) {
    if !up.0 {
        return;
    }
    for mut headline in &mut welcomes {
        if headline.out == HUSH_SECS {
            continue;
        }
        let solid = headline.solid();
        headline.out = HUSH_SECS;
        headline.secs = headline.age + HUSH_SECS * solid;
    }
}

/// Puts `title` across the middle of the screen for `secs` seconds, in gold,
/// `size` big ([`HEADLINE`] unless it is long), with `caption` over it in
/// white, fading in and out.
pub(crate) fn announce<'a>(
    commands: &'a mut Commands,
    caption: Option<&str>,
    title: &str,
    size: f32,
    secs: f32,
) -> EntityCommands<'a> {
    let mut headline = announce_frame(commands, secs);
    // Put up clear, for `fade_headlines` to bring up: whenever in the frame
    // this is spawned, it is never drawn once at full strength first.
    let colours = ui().colours;
    headline.with_children(|lines| {
        if let Some(caption) = caption {
            lines.spawn(words(caption, CAPTION, colours.text.with_alpha(0.0)));
        }
        lines.spawn(words(title, size, colours.gold.with_alpha(0.0)));
    });
    headline
}

/// What a headline stands in, up for `secs`: over the whole screen, with
/// the words stacked in the middle and high enough to leave your
/// character, just below the middle, in sight.
fn announce_frame<'a>(commands: &'a mut Commands, secs: f32) -> EntityCommands<'a> {
    commands.spawn((
        Headline {
            age: 0.0,
            secs,
            out: HEADLINE_OUT,
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::bottom(Val::Vh(ABOVE_YOU)),
            ..default()
        },
        Pickable::IGNORE,
    ))
}

/// How far each of the four dark copies under a stroked headline
/// ([`announce_stroked`]) stands off its words, as a share of their size:
/// as thin for a headline as the chat's stroke is for a line (mine).
const HEADLINE_STROKE: f32 = 0.035;

/// One of the four dark copies under a stroked headline's words that are
/// its stroke ([`announce_stroked`]).
#[derive(Component)]
struct StrokeCopy;

/// As [`announce`] with no caption, its words with a thin dark stroke round
/// them (four dark copies under them, as the chat's lines and your name's
/// plate have, `colours.chat.stroke`): for a game whose ground is as light
/// as the gold, Last Stand's sand by day, where "The anchor is under
/// attack!" was hard to read (a check of the merged games, 2026-11-18).
/// Faded in and out as every headline is.
pub(crate) fn announce_stroked<'a>(commands: &'a mut Commands, title: &str, size: f32, secs: f32) -> EntityCommands<'a> {
    let mut headline = announce_frame(commands, secs);
    let colours = ui().colours;
    let stroke = colours.chat.stroke;
    let off = size * HEADLINE_STROKE;
    headline.with_children(|lines| {
        lines.spawn((Node::default(), Pickable::IGNORE)).with_children(|group| {
            for corner in [Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0), Vec2::new(-1.0, 1.0), Vec2::new(1.0, 1.0)] {
                group.spawn((
                    StrokeCopy,
                    Text::new(title),
                    crate::font::sized(size),
                    TextColor(stroke.with_alpha(0.0)),
                    TextLayout::justify(Justify::Center),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::ZERO,
                        top: Val::ZERO,
                        ..default()
                    },
                    UiTransform::from_translation(Val2::new(Val::VMin(corner.x * off), Val::VMin(corner.y * off))),
                    Pickable::IGNORE,
                ));
            }
            group.spawn(words(title, size, colours.gold.with_alpha(0.0)));
        });
    });
    headline
}

/// Brings each headline up, holds it, and takes it away again: its lines,
/// and the words and the stroke of a stroked one ([`announce_stroked`]),
/// the stroke as strong as its own colour has it.
fn fade_headlines(
    time: Res<Time>,
    mut headlines: Query<(Entity, &mut Headline, &Children)>,
    groups: Query<&Children, Without<Headline>>,
    mut lines: Query<(&mut TextColor, Option<&mut TextShadow>, Has<StrokeCopy>)>,
    mut commands: Commands,
) {
    let colours = ui().colours;
    let (tint, stroke) = (colours.text_shadow, colours.chat.stroke);
    for (entity, mut headline, children) in &mut headlines {
        headline.age += time.delta_secs();
        if headline.age >= headline.secs {
            commands.entity(entity).despawn();
            continue;
        }
        let solid = headline.solid();
        let inside = children.iter().filter_map(|child| groups.get(child).ok()).flat_map(|group| group.iter());
        let mut words = lines.iter_many_mut(children.iter().chain(inside));
        while let Some((mut colour, shadow, copy)) = words.fetch_next() {
            // Only while it is fading: held, the words are left alone.
            let faded = colour.0.with_alpha(if copy { stroke.alpha() * solid } else { solid });
            if colour.0 != faded {
                colour.0 = faded;
                if let Some(mut shadow) = shadow {
                    shadow.color = tint.with_alpha(tint.alpha() * solid);
                }
            }
        }
    }
}

// --------------------------------------------------------------- pictures

/// A picture filling a button, drawn from flat shapes the way the edit
/// buttons are, all in percentages of the button's own box.
pub(crate) fn picture() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// Where a shape sits in a picture.
pub(crate) fn frame(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(left),
        top: Val::Percent(top),
        width: Val::Percent(width),
        height: Val::Percent(height),
        ..default()
    }
}

/// A plain white shape.
pub(crate) fn shape(left: f32, top: f32, width: f32, height: f32) -> impl Bundle {
    (
        frame(left, top, width, height),
        BackgroundColor(ui().colours.hud.icon),
        Pickable::IGNORE,
    )
}

/// A fist, for the punch button: a closed hand seen from the front, four
/// fingers curled over the palm with the thumb lying across them and the
/// wrist under, white on clear for the button to tint, drawn as the gift is
/// (Hajun, 2026-11-01: "a fist punch icon that is more a hand fist rather
/// than boxing glove"; Material has no closed hand, only its boxing glove,
/// `sports_mma`, which the button had until then).
pub(crate) fn fist() -> Image {
    const SIZE: u32 = 128;
    const ROUND: f32 = 0.1;
    /// The gap round the thumb, so that it reads as one.
    const GAP: f32 = 0.05;
    // In shares of half the image, from its middle, y up: each finger by
    // its middle and the top of it, all one width from one bottom; the
    // palm and the wrist by their middles and half sizes; and the thumb,
    // a box lying across the palm, its middle, half size and tilt.
    const FINGERS: [(f32, f32); 4] = [(-0.57, 0.68), (-0.19, 0.8), (0.19, 0.8), (0.57, 0.68)];
    const FINGER_HALF_WIDTH: f32 = 0.17;
    const FINGER_BOTTOM: f32 = 0.0;
    let palm = (Vec2::new(0.0, -0.2), Vec2::new(0.76, 0.3));
    let wrist = (Vec2::new(0.0, -0.7), Vec2::new(0.48, 0.22));
    let thumb = (Vec2::new(-0.22, -0.2), Vec2::new(0.5, 0.15));
    let (sin, cos) = 0.2f32.sin_cos();
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            let fingers = FINGERS
                .into_iter()
                .map(|(x, top)| {
                    let middle = Vec2::new(x, (top + FINGER_BOTTOM) * 0.5);
                    let reach = Vec2::new(FINGER_HALF_WIDTH - ROUND, (top - FINGER_BOTTOM) * 0.5 - ROUND);
                    box_distance(at - middle, reach) - ROUND
                })
                .fold(f32::INFINITY, f32::min);
            let palm_edge = box_distance(at - palm.0, palm.1 - ROUND) - ROUND;
            let wrist_edge = box_distance(at - wrist.0, wrist.1 - ROUND) - ROUND;
            // `at` as the thumb sees it, tilted up to the right.
            let from_thumb = at - thumb.0;
            let local = Vec2::new(from_thumb.x * cos + from_thumb.y * sin, from_thumb.y * cos - from_thumb.x * sin);
            let thumb_edge = box_distance(local, thumb.1 - ROUND) - ROUND;
            // The hand, with a gap cut round the thumb, and the thumb laid on.
            let hand = fingers.min(palm_edge).min(wrist_edge);
            let outside = hand.max(-(thumb_edge - GAP)).min(thumb_edge);
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// The jump button's arrow, pointing up: a broad head on a short shaft, one
/// filled shape with its corners rounded off, white on clear for the button
/// to tint (Hajun, 2026-10-31: "a semi transparent whiter filled round
/// arrow"). Drawn rather than built from nodes, as the arrow in strokes before
/// it was, because nodes that overlap would show where they do through the
/// see-through white; and edged by how far each pixel is from it, so that it
/// is smooth at whatever size a screen shows it. Turned down, it is the
/// fist's picture in the car, for the reverse.
pub(crate) fn jump_arrow() -> Image {
    const SIZE: u32 = 128;
    /// How much its corners are rounded off, as a share of half the image.
    const ROUND: f32 = 0.1;
    // In shares of half the image, from its middle, y up: the head's tip and
    // the ends of its base, and the shaft's half width and its two ends, each
    // drawn in by `ROUND`, which rounding puts back out.
    let tip = Vec2::new(0.0, 0.92 - ROUND);
    let base = Vec2::new(0.88 - ROUND, 0.06 + ROUND * 0.5);
    let (half_width, top, bottom) = (0.34 - ROUND, 0.2, -0.92 + ROUND);
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            let head = triangle_distance(at, tip, Vec2::new(-base.x, base.y), base);
            let middle = Vec2::new(0.0, (top + bottom) * 0.5);
            let reach = Vec2::new(half_width, (top - bottom) * 0.5);
            let shaft = box_distance(at - middle, reach);
            let outside = head.min(shaft) - ROUND;
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// An arrowhead pointing left, for turning through a card's colours on the
/// sign-up's character page (`signup`; flipped across for the right): a
/// rounded triangle, white on clear, drawn as the jump button's arrow is.
pub(crate) fn side_arrow() -> Image {
    const SIZE: u32 = 128;
    /// How much its corners are rounded off, as a share of half the image.
    const ROUND: f32 = 0.12;
    // In shares of half the image, from its middle, y up: the tip and the
    // two ends of the base, each drawn in by `ROUND`, which rounding puts
    // back out.
    let tip = Vec2::new(-0.85 + ROUND, 0.0);
    let (top, bottom) = (Vec2::new(0.6 - ROUND, 0.9 - ROUND), Vec2::new(0.6 - ROUND, -0.9 + ROUND));
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            let outside = triangle_distance(at, tip, bottom, top) - ROUND;
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// A pencil, for the settings' Edit name (`settings`): a rounded shaft with
/// a point at its lower left, leaning to the upper right, white on clear
/// for the button to tint, drawn as the gear is.
pub(crate) fn pencil() -> Image {
    const SIZE: u32 = 128;
    const ROUND: f32 = 0.04;
    // In shares of half the image, along the pencil and across it: the
    // shaft, a box from the point's base up to the top, and the point, a
    // triangle on the base, its corners drawn in by `ROUND`, which rounding
    // puts back out.
    const HALF_WIDTH: f32 = 0.17;
    const BASE: f32 = -0.32;
    const TOP: f32 = 0.78;
    const TIP: f32 = -0.78;
    let shaft_reach = Vec2::new(HALF_WIDTH - ROUND, (TOP - BASE) * 0.5 - ROUND);
    let shaft_middle = Vec2::new(0.0, (TOP + BASE) * 0.5);
    let tip = Vec2::new(0.0, TIP + ROUND);
    let base_left = Vec2::new(-HALF_WIDTH + ROUND, BASE);
    let base_right = Vec2::new(HALF_WIDTH - ROUND, BASE);
    let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            // `at` as the pencil sees it, leaning 45 degrees to the right.
            let local = Vec2::new(at.x * cos - at.y * sin, at.x * sin + at.y * cos);
            let shaft = box_distance(local - shaft_middle, shaft_reach);
            let point = triangle_distance(local, tip, base_left, base_right);
            let outside = shaft.min(point) - ROUND;
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// A keyboard, for the small button that takes up the chat's box to type
/// into ([`ChatButton`]): a slab with rounded corners, seen from above,
/// with its keys cut out of it, two rows of five and the long one under
/// them, white on clear for the button to tint. The picture is mine: Hajun
/// asked for the button and named none for it, and a second speech bubble
/// over the big one would not have said which of the two types.
pub(crate) fn keyboard() -> Image {
    drawn(keyboard_distance)
}

/// How far `at` is from the edge of the keyboard's picture.
fn keyboard_distance(at: Vec2) -> f32 {
    const ROUND: f32 = 0.12;
    /// Half a key, and how far its corners are rounded out past that.
    const KEY: f32 = 0.08;
    const KEY_ROUND: f32 = 0.02;
    let slab = box_distance(at, Vec2::new(0.88 - ROUND, 0.58 - ROUND)) - ROUND;
    let mut keys = box_distance(at - Vec2::new(0.0, -0.31), Vec2::new(0.4, 0.07)) - KEY_ROUND;
    for row in [0.3, 0.0] {
        for column in -2..=2 {
            let key = box_distance(at - Vec2::new(column as f32 * 0.31, row), Vec2::splat(KEY)) - KEY_ROUND;
            keys = keys.min(key);
        }
    }
    slab.max(-keys)
}

/// A picture drawn from a shape, white on clear for a button to tint, and
/// edged by how far each pixel is from it, as the fist and the arrows are:
/// `outside` says how far a point is from the shape's edge, in shares of
/// half the image from its middle, y up, less than nothing inside it.
fn drawn(outside: impl Fn(Vec2) -> f32) -> Image {
    drawn_at(128, outside)
}

/// The same, `size` pixels across: 128 for a button's picture, and fewer
/// for one shown much smaller than that, which drawn big and shrunk on
/// the screen would come out ragged.
fn drawn_at(size: u32, outside: impl Fn(Vec2) -> f32) -> Image {
    let pixel = 2.0 / size as f32;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for row in 0..size {
        for column in 0..size {
            let at = Vec2::new((column as f32 + 0.5) * pixel - 1.0, 1.0 - (row as f32 + 0.5) * pixel);
            let cover = (0.5 - outside(at) / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// A picture in colours of its own, where [`drawn`] makes one white for
/// whoever shows it to tint: each of `parts` a shape, as `drawn` takes
/// one, and the colour it is filled with, a later one over an earlier.
/// Whoever shows it tints it white, to leave it as it is. For a picture of
/// two colours, which one tint cannot give: the dynamite's red sticks and
/// brown fuse.
fn painted(parts: &[(&dyn Fn(Vec2) -> f32, Color)]) -> Image {
    const SIZE: u32 = 128;
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new((column as f32 + 0.5) * pixel - 1.0, 1.0 - (row as f32 + 0.5) * pixel);
            // The pixel's colour so far, and how much of it is covered.
            let (mut colour, mut cover) = (Vec3::ZERO, 0.0f32);
            for (outside, paint) in parts {
                let paint = paint.to_srgba();
                let over = (0.5 - outside(at) / pixel).clamp(0.0, 1.0) * paint.alpha;
                let all = over + cover * (1.0 - over);
                if all > 0.0 {
                    colour = (Vec3::new(paint.red, paint.green, paint.blue) * over + colour * cover * (1.0 - over)) / all;
                }
                cover = all;
            }
            let byte = |share: f32| (share.clamp(0.0, 1.0) * 255.0).round() as u8;
            data.extend_from_slice(&[byte(colour.x), byte(colour.y), byte(colour.z), byte(cover)]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Makes the pictures that are in colours of their own again as the file
/// changes on desktop (`ui::reread`), each where it is, so that whatever
/// shows one shows it in the new colours: a tint follows the file by
/// itself, and a picture painted out of it does not.
fn repaint(changed: Res<UiChanged>, pictures: Res<Pictures>, mut images: ResMut<Assets<Image>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    for (picture, handle) in &pictures.0 {
        if picture.in_its_own_colours() && images.insert(handle, picture.image()).is_err() {
            warn!("the picture of {picture:?} is gone");
        }
    }
}

/// A sword, for Bridge Wars' main button (Hajun's sketch,
/// `assets/pvpmapbuttonui.jpg`): a broad blade with a point, a crossguard,
/// a grip and a round pommel, leaning so that the point is up and to the
/// right.
pub(crate) fn sword() -> Image {
    drawn(sword_distance)
}

/// How far `at` is from the sword's edge ([`sword`]).
fn sword_distance(at: Vec2) -> f32 {
    const ROUND: f32 = 0.03;
    const BLADE: [Vec2; 5] = [
        Vec2::new(-0.13, -0.1),
        Vec2::new(0.13, -0.1),
        Vec2::new(0.13, 0.6),
        Vec2::new(0.0, 0.93),
        Vec2::new(-0.13, 0.6),
    ];
    let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
    // `at` as the sword sees it, its blade up its own y.
    let local = Vec2::new(at.x * cos - at.y * sin, at.x * sin + at.y * cos);
    let blade = polygon_distance(local, &BLADE);
    let guard = box_distance(local - Vec2::new(0.0, -0.2), Vec2::new(0.42, 0.08));
    let grip = box_distance(local - Vec2::new(0.0, -0.52), Vec2::new(0.09, 0.26));
    let pommel = circle_distance(local, Vec2::new(0.0, -0.8), 0.13);
    blade.min(guard).min(grip).min(pommel) - ROUND
}

/// The pictures of what Bridge Wars' two shops sell (since 2026-11-12),
/// on their cards and on the second button: each drawn in code, white on
/// clear, for whoever shows it to tint. The plank and the sword are the
/// buttons' own; the rest are theirs: a shield, as the game has them, a
/// ball of force with a glint on it; a fireball with its tail; a builder
/// standing on a plank; a flask, for every potion; a generator's pad with
/// an arrow up; a sword and a shield each with a plus, for the team's
/// stronger swords and better shields; a cross, for the heal pool; and an
/// eye struck through, for the blindness trap.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Picture {
    Plank,
    Sword,
    Shield,
    Fireball,
    Builder,
    Flask,
    Forge,
    SwordPlus,
    ShieldPlus,
    Cross,
    Eye,
    Compass,
    Dynamite,
    /// The builder on its plank with a plus beside it: the team's
    /// building quickened (since 2026-11-14).
    BuilderPlus,
    /// A dart pointing to the right: what sends a line of chat (`chat`).
    Send,
    /// Last Stand's anchor crystal with a plus up and to its left, as the
    /// team's stronger swords have: Fortify Anchor (since 2026-11-18,
    /// `last_stand`).
    Crystal,
    /// The crystal small, and a bolt from it: Last Stand's Anchor Beam.
    Beam,
    /// A four-leaf clover: Last Stand's Lucky Loot.
    Clover,
}

impl Picture {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const ALL: [Picture; 18] = [
        Picture::Plank,
        Picture::Sword,
        Picture::Shield,
        Picture::Fireball,
        Picture::Builder,
        Picture::Flask,
        Picture::Forge,
        Picture::SwordPlus,
        Picture::ShieldPlus,
        Picture::Cross,
        Picture::Eye,
        Picture::Compass,
        Picture::Dynamite,
        Picture::BuilderPlus,
        Picture::Send,
        Picture::Crystal,
        Picture::Beam,
        Picture::Clover,
    ];

    /// Whether it is drawn in colours of its own out of the file
    /// ([`painted`]), to be shown tinted white, rather than white for a
    /// tint: the dynamite's, whose fuse is brown and whose sticks are red.
    pub(crate) fn in_its_own_colours(self) -> bool {
        self == Picture::Dynamite
    }

    /// The picture, drawn.
    pub(crate) fn image(self) -> Image {
        match self {
            Picture::Plank => plank(),
            Picture::Sword => sword(),
            Picture::Shield => drawn(shield_distance),
            Picture::Fireball => drawn(|at| {
                let ball = circle_distance(at, Vec2::new(0.24, -0.24), 0.5);
                let tail = polygon_distance(at, &[Vec2::new(-0.9, 0.9), Vec2::new(0.16, 0.24), Vec2::new(-0.24, -0.16)]);
                ball.min(tail) - 0.03
            }),
            Picture::Builder => drawn(builder_distance),
            // The same builder, smaller and down to the right, with a plus
            // up and to the left of it, as the team's stronger swords have.
            Picture::BuilderPlus => drawn(|at| {
                let builder = builder_distance((at - Vec2::new(0.16, -0.14)) / 0.82) * 0.82;
                builder.min(plus_distance(at - Vec2::new(-0.64, 0.62), 0.28, 0.09) - 0.02)
            }),
            // The crystal, smaller and down to the right, with a plus up
            // and to the left of it.
            Picture::Crystal => drawn(|at| {
                let crystal = crystal_distance((at - Vec2::new(0.16, -0.1)) / 0.86) * 0.86;
                crystal.min(plus_distance(at - Vec2::new(-0.62, 0.62), 0.28, 0.09) - 0.02)
            }),
            // The crystal small on the left, and a bolt of lightning on
            // the right, leaving it.
            Picture::Beam => drawn(|at| {
                let crystal = crystal_distance((at - Vec2::new(-0.5, -0.12)) / 0.6) * 0.6;
                let bolt = polygon_distance(
                    at,
                    &[
                        Vec2::new(0.42, 0.95),
                        Vec2::new(0.02, 0.05),
                        Vec2::new(0.32, 0.05),
                        Vec2::new(0.08, -0.95),
                        Vec2::new(0.78, 0.25),
                        Vec2::new(0.46, 0.25),
                        Vec2::new(0.74, 0.95),
                    ],
                ) - 0.02;
                crystal.min(bolt)
            }),
            // Four round leaves about a middle, and a stem out of it, down
            // and to the right.
            Picture::Clover => drawn(|at| {
                let leaves = [Vec2::new(0.0, 0.4), Vec2::new(0.4, 0.0), Vec2::new(0.0, -0.4), Vec2::new(-0.4, 0.0)]
                    .into_iter()
                    .map(|middle| circle_distance(at - Vec2::new(-0.08, 0.08), middle, 0.34))
                    .fold(f32::INFINITY, f32::min);
                let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
                let from = at - Vec2::new(0.3, -0.42);
                let turned = Vec2::new(from.x * cos + from.y * sin, from.y * cos - from.x * sin);
                let stem = box_distance(turned, Vec2::new(0.07, 0.42));
                leaves.min(stem)
            }),
            // A dart seen from above, pointing to the right, notched at
            // its tail.
            Picture::Send => drawn(|at| {
                polygon_distance(
                    at,
                    &[Vec2::new(0.9, 0.0), Vec2::new(-0.8, 0.74), Vec2::new(-0.4, 0.0), Vec2::new(-0.8, -0.74)],
                ) - 0.03
            }),
            Picture::Flask => drawn(|at| {
                let body = circle_distance(at, Vec2::new(0.0, -0.3), 0.56);
                let neck = box_distance(at - Vec2::new(0.0, 0.42), Vec2::new(0.17, 0.34));
                let lip = box_distance(at - Vec2::new(0.0, 0.8), Vec2::new(0.3, 0.07));
                body.min(neck).min(lip) - 0.03
            }),
            Picture::Forge => drawn(|at| {
                let pad = box_distance(at, Vec2::splat(0.74)).abs() - 0.08;
                let head = triangle_distance(at, Vec2::new(0.0, 0.5), Vec2::new(-0.36, 0.04), Vec2::new(0.36, 0.04));
                let shaft = box_distance(at - Vec2::new(0.0, -0.22), Vec2::new(0.13, 0.28));
                pad.min(head.min(shaft) - 0.03)
            }),
            Picture::SwordPlus => drawn(|at| {
                let sword = sword_distance((at - Vec2::new(0.14, -0.14)) / 0.84) * 0.84;
                sword.min(plus_distance(at - Vec2::new(-0.62, 0.62), 0.3, 0.09) - 0.02)
            }),
            Picture::ShieldPlus => drawn(|at| {
                let ring = circle_distance(at, Vec2::ZERO, 0.8).abs() - 0.09;
                ring.min(plus_distance(at, 0.4, 0.11) - 0.02)
            }),
            Picture::Cross => drawn(|at| plus_distance(at, 0.74, 0.25) - 0.05),
            Picture::Eye => drawn(|at| {
                // Where two circles overlap, outlined; a pupil; and a
                // stroke through it all.
                let lens = circle_distance(at, Vec2::new(0.0, -0.62), 1.05).max(circle_distance(at, Vec2::new(0.0, 0.62), 1.05));
                let pupil = circle_distance(at, Vec2::ZERO, 0.2);
                let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
                let turned = Vec2::new(at.x * cos + at.y * sin, at.y * cos - at.x * sin);
                let stroke = box_distance(turned, Vec2::new(0.07, 0.95));
                (lens.abs() - 0.07).min(pupil).min(stroke)
            }),
            // An arrowhead pointing up, notched at its tail: the compass's
            // needle, on its card and round you in the game (`compass`,
            // which turns it from pointing up the screen to pointing at
            // whoever it tracks). Its tip is at the top, y being up here:
            // for its first day (2026-11-12, APK 24) it was drawn with its
            // tip at the bottom, and round you it pointed back at you from
            // the side whoever it tracked was on (Hajun: "the arrow icon is
            // pointing inward at the player rather than outward").
            Picture::Compass => drawn(|at| {
                polygon_distance(
                    at,
                    &[Vec2::new(0.0, 0.92), Vec2::new(-0.62, -0.8), Vec2::new(0.0, -0.42), Vec2::new(0.62, -0.8)],
                ) - 0.03
            }),
            // Two sticks of dynamite side by side, leaning as the sword
            // does, with the fuse out from between them at the top (since
            // 2026-11-13): the sticks red and the fuse brown, as the ones
            // thrown are, in the file's own colours for them (Hajun, of a
            // picture that was one red all over, with a dot for a spark
            // at the fuse's end: "the dynamite's string part should be
            // brown, like a string"). The fuse first, so that it comes
            // out from behind the sticks.
            Picture::Dynamite => {
                let colours = ui().colours.arena;
                let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
                // A point as the sticks see it, standing up their own y.
                let local = move |at: Vec2| Vec2::new(at.x * cos - at.y * sin, at.x * sin + at.y * cos);
                let sticks = move |at: Vec2| {
                    let local = local(at);
                    [-0.24, 0.24]
                        .into_iter()
                        .map(|x| box_distance(local - Vec2::new(x, -0.24), Vec2::new(0.15, 0.5)) - 0.06)
                        .fold(f32::INFINITY, f32::min)
                };
                let fuse = move |at: Vec2| box_distance(local(at) - Vec2::new(0.0, 0.56), Vec2::new(0.05, 0.3)) - 0.03;
                painted(&[(&fuse, colours.dynamite_fuse), (&sticks, colours.dynamite)])
            }
        }
    }
}

/// How far `at` is from the edge of Last Stand's crystal's picture: a cut
/// stone standing on its point, as the anchor's crystal stands on its
/// plinth, with a line cut down its middle for its facets.
fn crystal_distance(at: Vec2) -> f32 {
    let stone = polygon_distance(
        at,
        &[
            Vec2::new(0.0, 0.95),
            Vec2::new(0.46, 0.38),
            Vec2::new(0.46, -0.2),
            Vec2::new(0.0, -0.95),
            Vec2::new(-0.46, -0.2),
            Vec2::new(-0.46, 0.38),
        ],
    ) - 0.02;
    let facet = box_distance(at - Vec2::new(0.0, 0.0), Vec2::new(0.035, 0.7));
    stone.max(-facet)
}

/// How far `at` is from the edge of the builder's picture: a head, a body,
/// and the plank it stands on.
fn builder_distance(at: Vec2) -> f32 {
    let head = circle_distance(at, Vec2::new(0.0, 0.6), 0.24);
    let body = box_distance(at - Vec2::new(0.0, 0.02), Vec2::new(0.2, 0.3)) - 0.06;
    let plank = box_distance(at - Vec2::new(0.0, -0.72), Vec2::new(0.82, 0.12)) - 0.04;
    head.min(body).min(plank)
}

/// How far `at` is from the edge of the shield's picture: a ring, and a
/// crescent of light inside it, up and to the left.
fn shield_distance(at: Vec2) -> f32 {
    let ring = circle_distance(at, Vec2::ZERO, 0.8).abs() - 0.09;
    let glint = circle_distance(at, Vec2::new(-0.2, 0.2), 0.42).max(-circle_distance(at, Vec2::new(-0.06, 0.06), 0.46));
    ring.min(glint)
}

/// How far `at` is from the edge of a plus sign reaching `reach` each way
/// from its middle, its strokes `thick` either side of their middles.
fn plus_distance(at: Vec2, reach: f32, thick: f32) -> f32 {
    box_distance(at, Vec2::new(reach, thick)).min(box_distance(at, Vec2::new(thick, reach)))
}

/// Every [`Picture`] that has been shown, each made once.
#[derive(Resource, Default)]
pub(crate) struct Pictures(HashMap<Picture, Handle<Image>>);

impl Pictures {
    /// The picture, made if this is the first time it is shown.
    pub(crate) fn of(&mut self, images: &mut Assets<Image>, picture: Picture) -> Handle<Image> {
        self.0.entry(picture).or_insert_with(|| images.add(picture.image())).clone()
    }
}

/// A bag: a round sack with its neck tied, flaring out over the tie. Drawn
/// for Bridge Wars' inventory button, which went on 2026-11-09, for the
/// minimap; on 2026-11-21 it was the town's button for your bag
/// ([`BagButton`]), which has the box on it since the day after ([`SACK`]).
pub(crate) fn bag() -> Image {
    const ROUND: f32 = 0.03;
    let neck = [
        Vec2::new(-0.22, 0.28),
        Vec2::new(0.22, 0.28),
        Vec2::new(0.34, 0.66),
        Vec2::new(-0.34, 0.66),
    ];
    drawn(move |at| {
        let body = circle_distance(at, Vec2::new(0.0, -0.26), 0.62);
        let neck = polygon_distance(at, &neck);
        let tie = box_distance(at - Vec2::new(0.0, 0.3), Vec2::new(0.36, 0.07));
        body.min(neck).min(tie) - ROUND
    })
}

/// One of the four things Bridge Wars' generators make, for its count top
/// left and for the shop (since 2026-11-11, Hajun: "change the icon to
/// match the model shape"): Hajun's model of it as it is seen from above,
/// which is how the game looks at it, a cut stone with as many sides as
/// the model has, three for silver, four for gold, five for sapphire and
/// six for amethyst, each corner cut off as the model's is (its girdle has
/// two corners to a side, 10 to 13 degrees either side of where one would
/// be), and the outline of the flat on top of it, the table, cut through,
/// as far in as the model's is (0.70 to 0.775 of the way out). White on
/// clear, for the row to tint in the item's colour.
pub(crate) fn item_picture(item: Item) -> Image {
    /// Half the width of the line cut round the table.
    const LINE: f32 = 0.045;
    let (girdle, table) = item_outline(item);
    drawn(move |at| {
        let stone = polygon_distance(at, &girdle);
        let line = polygon_distance(at, &table).abs() - LINE;
        stone.max(-line)
    })
}

/// The same stone as a mark on the minimap, where each sapphire and each
/// amethyst generator is (`minimap::mark_generators`, since 2026-11-12):
/// its outline alone, filled, and drawn 32 pixels across. It is shown
/// about 15 pixels across there, where the line round its table, a pixel
/// wide at the most, came out as a ring of flecks, and the picture
/// shrunk from 128 pixels came out ragged. A little smaller in its
/// picture than the stone is in its own ([`MARK_FILLS`]): with so few
/// pixels, the stone drawn to the picture's edge left its edge soft.
pub(crate) fn item_mark(item: Item) -> Image {
    let (girdle, _) = item_outline(item);
    drawn_at(32, move |at| polygon_distance(at / MARK_FILLS, &girdle) * MARK_FILLS)
}

/// How much of its picture an item's mark for the minimap takes up, as a
/// share of what the stone takes up of its own picture.
const MARK_FILLS: f32 = 0.9;

/// The corners of `item`'s stone as it is drawn: of its girdle, the
/// outline of it, and of the table on top of it.
fn item_outline(item: Item) -> (Vec<Vec2>, Vec<Vec2>) {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
    // How many sides; which way its first corner points, in radians round
    // from the right; how far either side of a corner it is cut, in
    // degrees; how far out its table reaches, as a share; and where its
    // middle is and how far it reaches, so that each fills the picture.
    let (sides, turn, cut, table, middle, reach): (usize, f32, f32, f32, Vec2, f32) = match item {
        Item::Silver => (3, FRAC_PI_2, 10.0, 0.70, Vec2::new(0.0, -0.17), 1.0),
        Item::Gold => (4, FRAC_PI_4, 12.0, 0.77, Vec2::ZERO, 1.08),
        Item::Sapphire => (5, FRAC_PI_2, 13.0, 0.775, Vec2::new(0.0, -0.05), 0.95),
        Item::Amethyst => (6, 0.0, 13.0, 0.77, Vec2::ZERO, 0.92),
    };
    let corner = move |angle: f32, out: f32| middle + Vec2::new(angle.cos(), angle.sin()) * out;
    let step = std::f32::consts::TAU / sides as f32;
    let cut = cut.to_radians();
    let girdle: Vec<Vec2> = (0..sides)
        .flat_map(|k| {
            let at = turn + k as f32 * step;
            [corner(at - cut, reach), corner(at + cut, reach)]
        })
        .collect();
    let table: Vec<Vec2> = (0..sides).map(|k| corner(turn + k as f32 * step, reach * table)).collect();
    (girdle, table)
}

/// A plank, for the bridge button in Bridge Wars: a board lying across,
/// seen from above, with the seams of two nailed-on cross pieces cut into
/// it, so that it reads as a bit of bridge.
pub(crate) fn plank() -> Image {
    const ROUND: f32 = 0.06;
    drawn(|at| {
        let board = box_distance(at, Vec2::new(0.84 - ROUND, 0.3 - ROUND)) - ROUND;
        let seams = [-0.36, 0.36]
            .into_iter()
            .map(|x| box_distance(at - Vec2::new(x, 0.0), Vec2::new(0.035, 0.4)))
            .fold(f32::INFINITY, f32::min);
        board.max(-seams)
    })
}

/// Three dots in a row, for the button over Bridge Wars' slots, which
/// shows the four more buttons ([`MoreButton`]).
pub(crate) fn dots() -> Image {
    drawn(|at| {
        [-0.56, 0.0, 0.56]
            .into_iter()
            .map(|x| circle_distance(at, Vec2::new(x, 0.0), 0.17))
            .fold(f32::INFINITY, f32::min)
    })
}

/// A packed box seen from a corner, in outline: the picture on the bag's
/// first tab, what you own (`bag::panel`; Hajun's
/// `bagandinventorypanel1.png`), and on the button that opens it
/// ([`BagButton`], [`SACK`]). Six sides round it, and from its middle
/// the three edges that meet at the corner nearest you, so that its top is
/// a diamond; and a strip of tape over that top and a little way down its
/// right face, as theirs has.
pub(crate) fn cube() -> Image {
    const LINE: f32 = 0.06;
    const OUT: f32 = 0.86;
    /// The tape: how far along the top's edges its two sides run, and how
    /// far down the right face it hangs, as a share of that face's height.
    const TAPE: [f32; 2] = [0.36, 0.64];
    const HANGS: f32 = 0.34;
    let corner = |turns: f32| Vec2::from_angle((turns * 60.0 + 30.0).to_radians()) * OUT;
    let round: Vec<Vec2> = (0..6).map(|k| corner(k as f32)).collect();
    // The top's far left edge, from its back corner to its left one; its
    // near right edge, from the right corner to the middle; and straight
    // down, the way the right face's upright edges run.
    let (back, left, right) = (corner(1.0), corner(2.0), corner(0.0));
    let down = corner(4.0) * HANGS;
    let mut lines: Vec<(Vec2, Vec2)> = [right, left, corner(4.0)].into_iter().map(|to| (Vec2::ZERO, to)).collect();
    let ends: Vec<(Vec2, Vec2)> = TAPE.iter().map(|&along| (back.lerp(left, along), right.lerp(Vec2::ZERO, along))).collect();
    for &(far, near) in &ends {
        lines.push((far, near));
        lines.push((near, near + down));
    }
    lines.push((ends[0].1 + down, ends[1].1 + down));
    drawn(move |at| {
        let rim = polygon_distance(at, &round).abs();
        let inside = lines.iter().map(|&(a, b)| line_distance(at, a, b)).fold(f32::INFINITY, f32::min);
        rim.min(inside) - LINE
    })
}

/// A house with a pencil across its corner, for the button that begins
/// changing what stands on your plot (`customize`; Hajun, 2026-11-22: "a
/// 'customize icon' maybe an icon that looks like editinghouse/sign/
/// customize icon"): the house in outline, as the box on the bag's button
/// is, and the pencil solid, leaning as the settings' does, with a little
/// clear round it where it crosses the house.
pub(crate) fn customize() -> Image {
    const LINE: f32 = 0.065;
    const CLEAR: f32 = 0.11;
    let house = [
        Vec2::new(-0.74, 0.08),
        Vec2::new(-0.16, 0.78),
        Vec2::new(0.42, 0.08),
        Vec2::new(0.42, -0.74),
        Vec2::new(-0.74, -0.74),
    ];
    drawn(move |at| {
        let pencil = pencil_distance(at, Vec2::new(0.42, -0.2), 0.62, 0.13);
        let outline = polygon_distance(at, &house).abs() - LINE;
        outline.max(CLEAR - pencil).min(pencil)
    })
}

/// How far `at` is from the edge of a pencil that leans 45 degrees to the
/// upper right about `middle`, reaching `reach` each way along itself and
/// `half` either side of its line, its point at the lower left.
fn pencil_distance(at: Vec2, middle: Vec2, reach: f32, half: f32) -> f32 {
    const ROUND: f32 = 0.03;
    let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
    let from = at - middle;
    // As the pencil sees it: along itself up its own y.
    let local = Vec2::new(from.x * cos - from.y * sin, from.x * sin + from.y * cos);
    // The point takes the lowest third of it.
    let base = -reach + reach * 0.62;
    let shaft = box_distance(local - Vec2::new(0.0, (reach + base) * 0.5), Vec2::new(half - ROUND, (reach - base) * 0.5 - ROUND));
    let point = triangle_distance(local, Vec2::new(0.0, -reach + ROUND), Vec2::new(-half + ROUND, base), Vec2::new(half - ROUND, base));
    shaft.min(point) - ROUND
}

/// A paintbrush, for the tool that paints what stands on your plot
/// (`customize`): a thin handle leaning to the upper right, a band where
/// the bristles are held, and the bristles, broad and coming to a point at
/// the lower left, with a line cut between the band and them.
pub(crate) fn brush() -> Image {
    const ROUND: f32 = 0.035;
    let bristles = [
        Vec2::new(-0.2, -0.26),
        Vec2::new(0.2, -0.26),
        Vec2::new(0.26, -0.5),
        Vec2::new(0.0, -0.9),
        Vec2::new(-0.26, -0.5),
    ];
    let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
    drawn(move |at| {
        // As the brush sees it: along itself up its own y.
        let local = Vec2::new(at.x * cos - at.y * sin, at.x * sin + at.y * cos);
        let handle = box_distance(local - Vec2::new(0.0, 0.5), Vec2::new(0.085 - ROUND, 0.42 - ROUND));
        let band = box_distance(local - Vec2::new(0.0, -0.06), Vec2::new(0.2 - ROUND, 0.14 - ROUND));
        let hair = polygon_distance(local, &bristles);
        handle.min(band).min(hair) - ROUND
    })
}

/// A tick, for the button that ends changing your plot (`customize`): a
/// short stroke down and a long one up, as the edit menu's is.
pub(crate) fn tick() -> Image {
    const LINE: f32 = 0.13;
    let (from, turn, to) = (Vec2::new(-0.64, 0.02), Vec2::new(-0.2, -0.44), Vec2::new(0.68, 0.5));
    drawn(move |at| line_distance(at, from, turn).min(line_distance(at, turn, to)) - LINE)
}

/// A shopping cart, in outline: the picture on the bag's second tab, the
/// shop (`bag::panel`; Hajun's `bagandinventorypanel2.png`). Its handle
/// from the left, its basket, and two wheels under it.
pub(crate) fn cart() -> Image {
    const LINE: f32 = 0.065;
    let frame = [
        // The handle, and down the basket's back to its floor.
        (Vec2::new(-0.92, 0.7), Vec2::new(-0.64, 0.7)),
        (Vec2::new(-0.64, 0.7), Vec2::new(-0.3, -0.24)),
        // The basket: its rim, its front and its floor.
        (Vec2::new(-0.53, 0.4), Vec2::new(0.88, 0.4)),
        (Vec2::new(0.88, 0.4), Vec2::new(0.66, -0.24)),
        (Vec2::new(0.66, -0.24), Vec2::new(-0.3, -0.24)),
    ];
    drawn(move |at| {
        let lines = frame.iter().map(|&(a, b)| line_distance(at, a, b)).fold(f32::INFINITY, f32::min) - LINE;
        let wheels = [-0.18, 0.56]
            .into_iter()
            .map(|x| circle_distance(at, Vec2::new(x, -0.66), 0.15))
            .fold(f32::INFINITY, f32::min);
        lines.min(wheels)
    })
}

/// How far `at` is from the line from `a` to `b`.
fn line_distance(at: Vec2, a: Vec2, b: Vec2) -> f32 {
    let (along, from) = (b - a, at - a);
    let share = (from.dot(along) / along.length_squared().max(1e-12)).clamp(0.0, 1.0);
    from.distance(along * share)
}

/// How far `at` is from the edge of a circle round `middle` of `radius`:
/// less than nothing inside it.
fn circle_distance(at: Vec2, middle: Vec2, radius: f32) -> f32 {
    at.distance(middle) - radius
}

/// How far `at` is from the edge of the polygon with these `corners`, in
/// order round it either way: less than nothing inside it. Inigo Quilez's
/// `sdPolygon`.
pub(crate) fn polygon_distance(at: Vec2, corners: &[Vec2]) -> f32 {
    let count = corners.len();
    let mut nearest = (at - corners[0]).length_squared();
    let mut sign = 1.0;
    let mut last = count - 1;
    for (k, &corner) in corners.iter().enumerate() {
        let edge = corners[last] - corner;
        let from = at - corner;
        let along = from - edge * (from.dot(edge) / edge.dot(edge)).clamp(0.0, 1.0);
        nearest = nearest.min(along.length_squared());
        let crossing = [at.y >= corner.y, at.y < corners[last].y, edge.x * from.y > edge.y * from.x];
        if crossing.iter().all(|&c| c) || crossing.iter().all(|&c| !c) {
            sign = -sign;
        }
        last = k;
    }
    sign * nearest.sqrt()
}

/// How far `at` is from the edge of the triangle `a`, `b`, `c`: less than
/// nothing inside it. Inigo Quilez's `sdTriangle`.
pub(crate) fn triangle_distance(at: Vec2, a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let (e0, e1, e2) = (b - a, c - b, a - c);
    let (v0, v1, v2) = (at - a, at - b, at - c);
    let along = |v: Vec2, e: Vec2| v - e * (v.dot(e) / e.dot(e)).clamp(0.0, 1.0);
    let (p0, p1, p2) = (along(v0, e0), along(v1, e1), along(v2, e2));
    let s = (e0.x * e2.y - e0.y * e2.x).signum();
    let d = Vec2::new(p0.dot(p0), s * (v0.x * e0.y - v0.y * e0.x))
        .min(Vec2::new(p1.dot(p1), s * (v1.x * e1.y - v1.y * e1.x)))
        .min(Vec2::new(p2.dot(p2), s * (v2.x * e2.y - v2.y * e2.x)));
    -d.x.sqrt() * d.y.signum()
}

/// How far `at` is from the edge of a box round the origin reaching `reach`
/// out each way: less than nothing inside it.
fn box_distance(at: Vec2, reach: Vec2) -> f32 {
    let d = at.abs() - reach;
    d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0)
}

/// Whether every two of `buttons` round the jump button are more than a
/// share and a half of the short side apart, as every button there is of
/// every other: each a disc, by its right and bottom offsets in the box
/// round the jump button ([`beside_the_jump`]) and how big it is; or which
/// two are not. For the tests of each game's own buttons.
#[cfg(test)]
pub(crate) fn clear_of_each_other(buttons: &[((f32, f32), f32)]) -> Result<(), String> {
    let middle = |((right, bottom), size): ((f32, f32), f32)| Vec2::new(right + size * 0.5, bottom + size * 0.5);
    for (index, &one) in buttons.iter().enumerate() {
        for &other in &buttons[..index] {
            let gap = middle(one).distance(middle(other)) - (one.1 + other.1) * 0.5;
            if gap <= 1.5 {
                return Err(format!("{one:?} and {other:?} are {gap} apart"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_welcome_goes_at_once_when_a_window_comes_up_over_it() {
        use bevy::ecs::system::RunSystemOnce;
        use std::time::Duration;
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(DialogUp(false));
        world
            .run_system_once(|mut commands: Commands| {
                announce(&mut commands, None, "Donggeurami Town", HEADLINE, WELCOME_SECS).insert(Welcome);
            })
            .expect("said");
        let step = |world: &mut World, secs: f32| {
            world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(secs));
            world.run_system_once(hush_welcome).expect("hushed");
            world.run_system_once(fade_headlines).expect("faded");
        };
        let mut words = world.query::<&TextColor>();
        // Up, with nothing over it: held.
        step(&mut world, 1.0);
        assert!(words.iter(&world).all(|colour| (colour.0.alpha() - 1.0).abs() < 1e-3));
        // The game selector, the settings, the friends' panel: it goes from
        // there, half gone half way through, and gone at the end.
        world.resource_mut::<DialogUp>().0 = true;
        step(&mut world, HUSH_SECS * 0.5);
        assert!(words.iter(&world).all(|colour| (colour.0.alpha() - 0.5).abs() < 0.02));
        step(&mut world, HUSH_SECS * 0.6);
        assert_eq!(world.query::<&Welcome>().iter(&world).count(), 0, "gone");
        // Any other headline is left as it is.
        world
            .run_system_once(|mut commands: Commands| {
                announce(&mut commands, None, "Wave 3", HEADLINE, 3.0);
            })
            .expect("said");
        step(&mut world, 1.0);
        assert!(words.iter(&world).all(|colour| (colour.0.alpha() - 1.0).abs() < 1e-3));
    }

    #[test]
    fn a_stroked_headline_fades_in_with_its_stroke_and_goes_whole() {
        use bevy::ecs::system::RunSystemOnce;
        use std::time::Duration;
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world
            .run_system_once(|mut commands: Commands| {
                announce_stroked(&mut commands, "The anchor is under attack!", 5.0, 3.0);
            })
            .expect("said");
        let step = |world: &mut World, secs: f32| {
            world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(secs));
            world.run_system_once(fade_headlines).expect("faded");
        };
        // Halfway in: the words half there, the four copies under them half
        // as strong as the stroke's own colour.
        step(&mut world, HEADLINE_IN * 0.5);
        let stroke = ui().colours.chat.stroke.alpha();
        let mut texts = world.query::<(&TextColor, Has<StrokeCopy>)>();
        let (copies, words): (Vec<_>, Vec<_>) = texts.iter(&world).map(|(colour, copy)| (colour.0.alpha(), copy)).partition(|(_, copy)| *copy);
        assert_eq!(copies.len(), 4);
        assert_eq!(words.len(), 1);
        assert!((words[0].0 - 0.5).abs() < 1e-3, "{words:?}");
        for (alpha, _) in copies {
            assert!((alpha - stroke * 0.5).abs() < 1e-3, "{alpha}");
        }
        // Held, then gone, the stroke with it.
        step(&mut world, 1.0);
        assert!(texts.iter(&world).all(|(colour, copy)| (colour.0.alpha() - if copy { stroke } else { 1.0 }).abs() < 1e-3));
        step(&mut world, 2.0);
        assert_eq!(world.query::<&Headline>().iter(&world).count(), 0);
        assert_eq!(texts.iter(&world).count(), 0);
    }

    #[test]
    fn the_bag_is_up_only_in_the_town_and_out_of_the_car_where_the_door_is_up_instead() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.insert_resource(Driving::for_tests(false));
        let you = world.spawn((Player, Venue::Home)).id();
        let bag = world.spawn((BagButton, Visibility::Inherited)).id();
        let door = world.spawn((ExitButton, Visibility::Inherited)).id();
        let shown = |world: &World, button: Entity| world.get::<Visibility>(button) == Some(&Visibility::Inherited);
        // At home, as in House Builder's lobby and plots, under the town's
        // buttons: away, and no door.
        for venue in [Venue::Home, Venue::Lobby, Venue::Plot(3)] {
            *world.get_mut::<Venue>(you).expect("a venue") = venue;
            world.run_system_once(show_car_buttons).expect("shown");
            assert!(!shown(&world, bag) && !shown(&world, door), "{venue:?}");
        }
        // In the town: up, as the box round the jump button is.
        *world.get_mut::<Venue>(you).expect("a venue") = Venue::Town;
        world.run_system_once(show_car_buttons).expect("shown");
        assert!(shown(&world, bag) && !shown(&world, door));
        // In your car: the door out of it, and the bag away.
        world.insert_resource(Driving::for_tests(true));
        world.run_system_once(show_car_buttons).expect("shown");
        assert!(!shown(&world, bag) && shown(&world, door));
        // Out again: as it was.
        world.insert_resource(Driving::for_tests(false));
        world.run_system_once(show_car_buttons).expect("shown");
        assert!(shown(&world, bag) && !shown(&world, door));
    }

    #[test]
    fn the_buttons_by_the_left_edge_stand_under_your_gems_and_over_the_stick_the_lowest_under_the_chat() {
        // On a phone lying on its side the short side is the height. Three
        // of them, one over the other, grow up from their foot.
        let three = 3.0 * LEFT_BUTTON + 2.0 * LEFT_GAP;
        let (top, bottom) = (LEFT_FOOT - three, LEFT_FOOT);
        // The stick's top is its bottom margin and its size up from the
        // bottom of the screen.
        let stick_top = 100.0 - crate::STICK_BOTTOM_VH - crate::STICK_SIZE_VMIN;
        assert!(bottom + 1.5 < stick_top, "they end {bottom} down and the stick begins {stick_top} down");
        // Your name, your coins and your gems end over them.
        let line = BALANCE_TEXT * 1.3;
        let rows = EDGE + (NAME_TEXT * 1.3 + 2.0) + 1.4 + BALANCE_ICON.max(line) + 1.4 + (GEM_WIDE * GEM_SHAPE).max(line);
        assert!(rows + 1.5 < top, "the gems end {rows} down and they begin {top} down");
        // The one that is there alone is the middle of the three, about
        // half way down the screen, where the chat's lines begin: they
        // step aside of it (`customize::aside`).
        let alone = (bottom - LEFT_BUTTON - LEFT_GAP - LEFT_BUTTON, bottom - LEFT_BUTTON - LEFT_GAP);
        assert!((alone.0 + alone.1) * 0.5 > 40.0 && (alone.0 + alone.1) * 0.5 < 50.0, "{alone:?}");
        let chat = (crate::chat::TOP, crate::chat::TOP + crate::chat::LOG_HEIGHT);
        assert!(alone.0 < chat.1 && chat.0 < alone.1, "{alone:?} and {chat:?}");
    }

    #[test]
    fn the_gem_is_drawn_as_wide_as_the_coin_and_under_its_middle() {
        use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
        // The first and the last column of a picture with anything solid
        // in it, and how wide and how tall the picture is.
        let drawn = |name: &str| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join(name);
            let bytes = std::fs::read(&path).expect(name);
            let image = Image::from_buffer(&bytes, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default())
                .expect(name);
            let (wide, high) = (image.width() as usize, image.height() as usize);
            let data = image.data.as_ref().expect("a picture's bytes");
            assert_eq!(data.len(), wide * high * 4, "{name}: four bytes to a dot");
            let solid = |column: usize| (0..high).any(|row| data[(row * wide + column) * 4 + 3] > 127);
            let first = (0..wide).find(|&column| solid(column)).expect("something drawn");
            let last = (0..wide).rev().find(|&column| solid(column)).expect("something drawn");
            (first as f32, last as f32 + 1.0, wide as f32, high as f32)
        };
        // The numbers here are the pictures' own: Hajun's to draw again,
        // and these to be measured again when they do.
        let (coin_from, coin_to, coin_wide, coin_high) = drawn(COIN_ICON);
        let (gem_from, gem_to, gem_wide, gem_high) = drawn(GEM_ICON);
        assert_eq!(coin_wide, coin_high, "the coin's picture is drawn square");
        assert!(((coin_to - coin_from) / coin_wide - COIN_FILLS).abs() < 0.004, "the coin is {} of {coin_wide}", coin_to - coin_from);
        assert!(((gem_to - gem_from) / gem_wide - GEM_FILLS).abs() < 0.004, "the gem is {} of {gem_wide}", gem_to - gem_from);
        assert!((gem_high / gem_wide - GEM_SHAPE).abs() < 1e-4);
        // So the gem itself is as wide on the screen as the coin itself
        // (Hajun, 2026-11-22: "slightly decrease the gem icon size to match
        // the coin icon size"), a little smaller than it was and no more.
        let (coin, gem) = (BALANCE_ICON * COIN_FILLS, GEM_WIDE * GEM_FILLS);
        assert!((coin - gem).abs() < 1e-4, "the coin is {coin} across and the gem {gem}");
        assert!((0.88..1.0).contains(&(GEM_WIDE / BALANCE_ICON)), "{GEM_WIDE}");
        // Each is in the middle of its picture, to a hundredth of it, and
        // the gem's picture in the middle of the room the coin's has: one
        // is under the other, and their numbers begin at the same place.
        for (name, from, to, wide) in [("coin", coin_from, coin_to, coin_wide), ("gem", gem_from, gem_to, gem_wide)] {
            assert!(((from + to) * 0.5 / wide - 0.5).abs() < 0.01, "the {name} is from {from} to {to} of {wide}");
        }
        // No taller than the coin either.
        assert!(GEM_WIDE * GEM_SHAPE <= BALANCE_ICON * COIN_FILLS);
    }

    #[test]
    fn the_pictures_on_the_bag_s_tabs_are_drawn_white_on_clear_and_are_not_empty() {
        let all = [
            ("the box", cube()),
            ("the cart", cart()),
            ("the bag", bag()),
            ("the house and pencil", customize()),
            ("the brush", brush()),
            ("the tick", tick()),
        ];
        for (name, image) in all {
            let data = image.data.as_ref().expect("a picture's bytes");
            assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]), "{name}");
            let covered = data.chunks(4).filter(|pixel| pixel[3] > 127).count() as f32 / (data.len() / 4) as f32;
            assert!((0.08..0.75).contains(&covered), "{name} covers {covered} of its picture");
            // Clear at its corners: a picture, not a block.
            assert_eq!(data[3], 0, "{name}");
        }
        // The box is an outline: its three near edges meet in its middle,
        // and its left face, which has no tape on it, is clear.
        let cube = cube();
        let data = cube.data.as_ref().expect("bytes");
        let at = |x: usize, y: usize| data[(y * 128 + x) * 4 + 3];
        assert!(at(64, 64) > 200, "the three edges meet in its middle");
        assert_eq!(at(40, 84), 0, "its left face is clear");
    }

    #[test]
    fn the_four_more_buttons_lie_on_a_ring_round_the_sword_clear_of_every_other_button() {
        // A button's middle from the sword's, to the left and up, in
        // shares of the short side, from its place in the box round the
        // jump button, whose middle is the sword's.
        let middle = |(right, bottom): (f32, f32), size: f32| {
            Vec2::new(right + size * 0.5, bottom + size * 0.5) - Vec2::splat(crate::JUMP_SIZE_VMIN * 0.5)
        };
        let slots: Vec<Vec2> = (0..UTILITY_SLOTS).map(|slot| middle(slot_place(slot), ACTION_BUTTON)).collect();
        // The second button is where it was, on the first ring; the four
        // more are on theirs, from straight left of the sword round to
        // nearly straight over it.
        assert_eq!(slot_place(0), ARENA_SLOT);
        assert!((slots[0].length() - 20.5).abs() < 0.05);
        for more in &slots[1..] {
            assert!((more.length() - MORE_RING).abs() < 1e-3);
            assert!(more.x > 0.0 && more.y > -1e-3, "{more}");
        }
        // They fill from the one beside the second button.
        let nearest = slots[1..].iter().map(|more| more.distance(slots[0])).fold(f32::INFINITY, f32::min);
        assert_eq!(slots[1].distance(slots[0]), nearest);
        // No two of all the buttons there touch: these five, the sword,
        // the jump button, the bridge button and the dots.
        let mut discs: Vec<(Vec2, f32)> = slots.iter().map(|at| (*at, ACTION_BUTTON)).collect();
        discs.push((Vec2::ZERO, crate::JUMP_SIZE_VMIN));
        let (right, bottom, size) = ARENA_JUMP;
        discs.push((middle((right, bottom), size), size));
        discs.push((middle(ARENA_BRIDGE, ACTION_BUTTON), ACTION_BUTTON));
        let (right, bottom, size) = ARENA_DOTS;
        discs.push((middle((right, bottom), size), size));
        // And the chat's small button, over the bridge button.
        let (right, bottom, size) = CHAT_BUTTON;
        discs.push((middle((right, bottom), size), size));
        for (index, (at, size)) in discs.iter().enumerate() {
            for (other, other_size) in &discs[..index] {
                let gap = at.distance(*other) - (size + other_size) * 0.5;
                assert!(gap > 1.5, "{at} and {other} are {gap} apart");
            }
        }
        // And the four more are as far from each other as from the first
        // ring's.
        let mut round: Vec<Vec2> = slots[1..].to_vec();
        round.sort_by(|a, b| a.y.total_cmp(&b.y));
        for pair in round.windows(2) {
            assert!((pair[0].distance(pair[1]) - ACTION_BUTTON - 3.5).abs() < 0.1);
        }
    }

    #[test]
    fn the_chat_s_button_is_small_over_the_speech_bubble_and_clear_of_everything() {
        let (right, bottom, size) = CHAT_BUTTON;
        let (quick_right, quick_bottom, quick_size) = QUICK_BUTTON;
        // Smaller than the bubble, over it, and a little to the right of it:
        // its middle to the right of the bubble's, and still over the
        // bubble.
        assert!(size < quick_size * 0.75);
        assert!(bottom > quick_bottom + quick_size + 1.0);
        let (middle, quick_middle) = (right + size * 0.5, quick_right + quick_size * 0.5);
        assert!(middle < quick_middle - 2.0 && middle > quick_right, "{middle} against {quick_middle}");
        // In the town: clear of the bubble, the face, the fist and the jump
        // button, each a disc, by its middle from the box's corner.
        let at = |(right, bottom): (f32, f32), size: f32| Vec2::new(right + size * 0.5, bottom + size * 0.5);
        let chat = at((right, bottom), size);
        let town = [
            (at((quick_right, quick_bottom), quick_size), quick_size),
            (at((18.5, 18.5), ACTION_BUTTON), ACTION_BUTTON),
            (at((24.3, 1.2), ACTION_BUTTON), ACTION_BUTTON),
            (at((0.0, 0.0), crate::JUMP_SIZE_VMIN), crate::JUMP_SIZE_VMIN),
        ];
        for (other, other_size) in town {
            let gap = chat.distance(other) - (size + other_size) * 0.5;
            assert!(gap > 1.5, "{other} is {gap} from the chat's button");
        }
        // And of what is top right, on a screen lying on its side as wide
        // as a tablet's, a monitor's and a phone's (the short side is the
        // height, 100 shares): the gamepad, the lowest of the town's
        // buttons down the right edge, under the two people and, when it is
        // there, the gift; and in a game the minimap. From the top and the
        // right of the screen, in shares of the short side.
        for wide in [4.0 / 3.0, 16.0 / 9.0, 19.5 / 9.0] {
            let chat = Vec2::new(
                crate::JUMP_RIGHT_VW * wide + right + size * 0.5,
                100.0 - crate::JUMP_BOTTOM_VH - bottom - size * 0.5,
            );
            let top = EDGE + SIDE_INSET;
            let over_it = if GIFT { 2.0 } else { 1.0 };
            let gamepad = Vec2::new(top + SIDE_BUTTON * 0.5, top + SIDE_BUTTON * (over_it + 0.5) + 1.6 * over_it);
            let gap = chat.distance(gamepad) - (size + SIDE_BUTTON) * 0.5;
            assert!(gap > 1.5, "{wide}: {gap} from the gamepad");
            let minimap_ends = top + MINIMAP;
            assert!(chat.y - size * 0.5 > minimap_ends + 1.5, "{wide}: under the minimap");
            // On the screen, with room to its right.
            assert!(chat.x - size * 0.5 > EDGE, "{wide}: {} from the right", chat.x - size * 0.5);
        }
    }

    #[test]
    fn the_bar_moved_in_on_the_right_is_still_clear_of_the_things_to_say() {
        use crate::quick;
        // On a screen lying on its side as wide as a monitor's and as a
        // phone's (the short side is the height, 100 shares), with the bar
        // moved in from the right as far as it ever is: the two buttons
        // down the right edge, and the gift over them when it is there,
        // each a disc, from the right and the top of the screen; and every
        // plate of the fan, a box from its right end leftward as far as its
        // words take it, say 40 shares.
        let room = bar_room();
        for wide in [16.0 / 9.0, 19.5 / 9.0, 874.0 / 402.0, 2.4] {
            let moved = (crate::JUMP_RIGHT_VW * wide + room.vmin).max(0.0);
            let side = EDGE + SIDE_INSET + moved;
            let buttons = if GIFT { 3 } else { 2 };
            for button in 0..buttons {
                let middle = Vec2::new(side + SIDE_BUTTON * 0.5, EDGE + SIDE_INSET + SIDE_BUTTON * (button as f32 + 0.5) + 1.6 * button as f32);
                for index in 0..quick::MOST {
                    let (right, bottom) = quick::place(index);
                    let (from, under) = (crate::JUMP_RIGHT_VW * wide + right, 100.0 - crate::JUMP_BOTTOM_VH - bottom);
                    let nearest = Vec2::new(middle.x.clamp(from, from + 40.0), middle.y.clamp(under - quick::HEIGHT, under));
                    let gap = nearest.distance(middle) - SIDE_BUTTON * 0.5;
                    assert!(gap >= 1.0, "{wide}: button {button} is {gap} from plate {index}, moved in {moved}");
                }
            }
        }
        // And there is room at all on a phone, enough for a notch: on an
        // iPhone 14, 844 by 390 points, whose notch reaches 33 points in
        // from the side it is on and whose safe area asks for more than the
        // room, what stands at that end of the bar ends up a share clear of
        // the notch. (An island reaches further, 51, but lies half way
        // down the edge, under where the buttons and the minimap end.)
        assert!(crate::JUMP_RIGHT_VW * 19.5 / 9.0 + room.vmin > 4.0);
        let (wide, point) = (844.0 / 390.0, 3.9);
        let stands = (EDGE + SIDE_INSET + crate::JUMP_RIGHT_VW * wide + room.vmin) * point;
        assert!(stands > 33.0 + point, "{stands} points in");
        let island_begins = (100.0 - 125.0 / 4.02) * 0.5;
        assert!(EDGE + SIDE_INSET + MINIMAP < island_begins - 1.0, "the minimap ends {} down", EDGE + SIDE_INSET + MINIMAP);
    }

    #[test]
    fn the_chat_s_button_stepped_in_from_an_island_is_still_clear_of_every_button() {
        let (right, bottom, size) = CHAT_BUTTON;
        let at = |(right, bottom): (f32, f32), size: f32| Vec2::new(right + size * 0.5, bottom + size * 0.5);
        // What is about it, as discs by their middles from the corner of
        // the box round the jump button: in a game the sword, the jump
        // button, the bridge button and the five buttons for things to use;
        // in the town the bubble, the face, the fist and the jump button.
        let (jump_right, jump_bottom, jump_size) = ARENA_JUMP;
        let mut about = vec![
            (at((0.0, 0.0), crate::JUMP_SIZE_VMIN), crate::JUMP_SIZE_VMIN),
            (at((jump_right, jump_bottom), jump_size), jump_size),
            (at(ARENA_BRIDGE, ACTION_BUTTON), ACTION_BUTTON),
            (at((QUICK_BUTTON.0, QUICK_BUTTON.1), QUICK_BUTTON.2), QUICK_BUTTON.2),
            (at((18.5, 18.5), ACTION_BUTTON), ACTION_BUTTON),
            (at((24.3, 1.2), ACTION_BUTTON), ACTION_BUTTON),
        ];
        about.extend((0..UTILITY_SLOTS).map(|slot| (at(slot_place(slot), ACTION_BUTTON), ACTION_BUTTON)));
        // Stepped in is further from the right: all the way, and part way.
        for step in [0.0, CHAT_STEPS_IN * 0.5, CHAT_STEPS_IN] {
            let chat = at((right + step, bottom), size);
            for (other, other_size) in &about {
                let gap = chat.distance(*other) - (size + other_size) * 0.5;
                assert!(gap > 1.5, "stepped in {step}: {other} is {gap} from the chat's button");
            }
        }
        // It is told where it stands as it is put there.
        let steps = chat_steps_in();
        assert_eq!((steps.vw, steps.vmin, steps.most), (crate::JUMP_RIGHT_VW, right, CHAT_STEPS_IN));
        // And that is out past the box, which is why it has to step in at
        // all: nothing else round the jump button is.
        assert!(right < 0.0);
        assert!(about.iter().all(|(other, other_size)| other.x - other_size * 0.5 >= 0.0));
    }

    #[test]
    fn each_of_the_four_games_has_its_own_buttons_and_the_fights_bridge_wars_own() {
        // Every venue its buttons: the town's wherever there is no game of
        // its own, Bridge Wars' on its map, and each of the four's on its.
        assert_eq!(Layout::of(Venue::Town), Layout::Town);
        for venue in [Venue::Home, Venue::Lobby, Venue::Plot(2)] {
            assert_eq!(Layout::of(venue), Layout::Town, "{venue:?}");
        }
        assert_eq!(Layout::of(Venue::Arena), Layout::Arena);
        assert_eq!(Layout::of(Venue::BlendIn), Layout::BlendIn);
        assert_eq!(Layout::of(Venue::KnockOff), Layout::KnockOff);
        assert_eq!(Layout::of(Venue::BridgeRace), Layout::BridgeRace);
        assert_eq!(Layout::of(Venue::LastStand), Layout::LastStand);
        // Laid out as Bridge Wars is exactly where it is fought as Bridge
        // Wars is.
        for venue in [Venue::Town, Venue::Arena, Venue::BlendIn, Venue::KnockOff, Venue::BridgeRace, Venue::LastStand] {
            assert_eq!(Layout::of(venue).fights(), crate::island::fights(venue), "{venue:?}");
        }
        // Clear of each other is what it says, either way.
        assert!(clear_of_each_other(&[((0.0, 0.0), 14.0), ((0.0, 16.0), 14.0)]).is_ok());
        assert!(clear_of_each_other(&[((0.0, 0.0), 14.0), ((0.0, 15.0), 14.0)]).is_err());
    }

    #[test]
    fn the_keyboard_is_a_slab_with_its_keys_cut_out() {
        let image = keyboard();
        // Solid between the keys and round them; clear in the middle of
        // each of the ten, and along the long one under them; clear outside
        // the slab, which is wider than it is tall.
        for (x, y) in [(0.155, 0.3), (-0.155, 0.0), (0.0, 0.15), (0.0, -0.16), (0.8, 0.0), (-0.8, 0.0), (0.0, 0.5), (0.0, -0.5), (0.6, -0.31)] {
            assert_eq!(alpha_at(&image, x, y), 255, "at {x} {y}");
        }
        for row in [0.3, 0.0] {
            for column in -2..=2 {
                assert_eq!(alpha_at(&image, column as f32 * 0.31, row), 0, "key {column} of row {row}");
            }
        }
        for x in [-0.3, 0.0, 0.3] {
            assert_eq!(alpha_at(&image, x, -0.31), 0, "the long key at {x}");
        }
        for (x, y) in [(0.0, 0.75), (0.0, -0.75), (0.95, 0.0), (-0.95, 0.0), (0.86, 0.56)] {
            assert_eq!(alpha_at(&image, x, y), 0, "outside at {x} {y}");
        }
        let data = image.data.as_ref().unwrap();
        assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_jump_arrow_points_up_with_a_broad_head_on_a_narrow_shaft() {
        let image = jump_arrow();
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        // How solid it is at `x`, `y`, in shares of half the image from its
        // middle, y up.
        let alpha = |x: f32, y: f32| {
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            data[((row * size + column) * 4 + 3) as usize]
        };
        // Solid up its middle, from the foot of the shaft to the tip.
        for y in [-0.85, -0.4, 0.0, 0.4, 0.8] {
            assert_eq!(alpha(0.0, y), 255, "at {y}");
        }
        // The head is wide, and the shaft beside it is not there.
        assert_eq!(alpha(-0.7, 0.1), 255);
        assert_eq!(alpha(0.7, 0.1), 255);
        assert_eq!(alpha(-0.6, -0.5), 0);
        assert_eq!(alpha(0.6, -0.5), 0);
        // Nothing in the corners, nor over the tip.
        for (x, y) in [(-0.97, 0.97), (0.97, 0.97), (-0.97, -0.97), (0.97, -0.97), (0.0, 0.99)] {
            assert_eq!(alpha(x, y), 0, "at {x} {y}");
        }
        // White throughout, for the button to tint.
        assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_fist_is_four_fingers_over_a_palm_with_the_thumb_across_it() {
        let image = fist();
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        let alpha = |x: f32, y: f32| {
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            data[((row * size + column) * 4 + 3) as usize]
        };
        // Solid on each finger and nothing between the middle two; solid
        // on the palm, the thumb and the wrist; nothing over the fingers,
        // beside the wrist, or in the corners.
        for x in [-0.57, -0.19, 0.19, 0.57] {
            assert_eq!(alpha(x, 0.5), 255, "finger at {x}");
        }
        assert_eq!(alpha(0.0, 0.5), 0);
        assert_eq!(alpha(0.6, -0.3), 255);
        assert_eq!(alpha(-0.2, -0.24), 255);
        assert_eq!(alpha(0.0, -0.75), 255);
        assert_eq!(alpha(0.0, 0.95), 0);
        assert_eq!(alpha(0.75, -0.75), 0);
        assert_eq!(alpha(0.95, 0.95), 0);
        assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_pencil_leans_to_the_upper_right_with_its_point_at_the_lower_left() {
        let image = pencil();
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        let alpha = |x: f32, y: f32| {
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            data[((row * size + column) * 4 + 3) as usize]
        };
        // Solid along the diagonal from the point to the top, and nothing
        // along the other diagonal or beyond either end.
        for t in [-0.45, -0.2, 0.0, 0.3, 0.5] {
            assert_eq!(alpha(t, t), 255, "at {t}");
        }
        assert_eq!(alpha(-0.5, 0.5), 0);
        assert_eq!(alpha(0.5, -0.5), 0);
        assert_eq!(alpha(0.7, 0.7), 0);
        assert_eq!(alpha(-0.7, -0.7), 0);
        // The point is narrower than the shaft: beside the diagonal near the
        // point there is nothing, beside it up the shaft there is pencil.
        assert_eq!(alpha(-0.42, -0.62), 0);
        assert_eq!(alpha(0.1, 0.3), 255);
    }

    /// How solid `image` is at `x`, `y`, in shares of half the image from
    /// its middle, y up.
    fn alpha_at(image: &Image, x: f32, y: f32) -> u8 {
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
        let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
        data[((row * size + column) * 4 + 3) as usize]
    }

    #[test]
    fn the_dynamite_s_picture_is_red_sticks_with_a_brown_fuse() {
        let image = Picture::Dynamite.image();
        let pixel = |x: f32, y: f32| -> [u8; 4] {
            let size = image.width();
            let data = image.data.as_ref().unwrap();
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            let at = ((row * size + column) * 4) as usize;
            [data[at], data[at + 1], data[at + 2], data[at + 3]]
        };
        let bytes = |colour: Color| {
            let colour = colour.to_srgba();
            [colour.red, colour.green, colour.blue].map(|share| (share * 255.0).round() as u8)
        };
        let colours = ui().colours.arena;
        // A place on the picture, given as the sticks see it: across them
        // and up them, the picture's upper right being their top.
        let seen = |across: f32, up: f32| ((across + up) * 0.7071, (up - across) * 0.7071);
        // In the middle of each stick, the file's red; up the fuse, its
        // brown; and both solid.
        for across in [-0.24, 0.24] {
            let (x, y) = seen(across, -0.3);
            assert_eq!(pixel(x, y)[..3], bytes(colours.dynamite), "a stick at {across}");
            assert_eq!(pixel(x, y)[3], 255);
        }
        let (x, y) = seen(0.0, 0.7);
        assert_eq!(pixel(x, y)[..3], bytes(colours.dynamite_fuse), "the fuse");
        assert_eq!(pixel(x, y)[3], 255);
        assert_ne!(bytes(colours.dynamite), bytes(colours.dynamite_fuse));
        // Nothing beside the fuse, past its end, or in the far corners.
        for (across, up) in [(0.3, 0.7), (-0.3, 0.7), (0.0, 1.05)] {
            let (x, y) = seen(across, up);
            assert_eq!(pixel(x, y)[3], 0, "at {across}, {up}");
        }
        assert_eq!(pixel(-0.9, 0.9)[3], 0);
        assert_eq!(pixel(0.9, -0.9)[3], 0);
        // It is the one picture that is not white for a tint.
        assert!(Picture::ALL.iter().all(|picture| picture.in_its_own_colours() == (*picture == Picture::Dynamite)));
    }

    #[test]
    fn bridge_wars_pictures_are_the_shapes_they_are_named_for() {
        // The sword runs from its pommel at the lower left to its point at
        // the upper right, with the guard across it; nothing off that
        // diagonal's ends, nor in the other corners.
        let sword = sword();
        for t in [-0.7, -0.4, 0.0, 0.3, 0.6] {
            assert_eq!(alpha_at(&sword, t * 0.7071, t * 0.7071), 255, "sword at {t}");
        }
        assert_eq!(alpha_at(&sword, -0.5 * 0.7071 + 0.2, -0.5 * 0.7071 - 0.2), 0);
        assert_eq!(alpha_at(&sword, 0.95, 0.95), 0);
        assert_eq!(alpha_at(&sword, -0.9, 0.9), 0);
        // The guard reaches out either side of the blade, across it.
        assert_eq!(alpha_at(&sword, -0.2 * 0.7071 - 0.3 * 0.7071, -0.2 * 0.7071 + 0.3 * 0.7071), 255);
        // The bag is solid through its body and its neck, and nothing
        // beside the neck.
        let bag = bag();
        assert_eq!(alpha_at(&bag, 0.0, -0.3), 255);
        assert_eq!(alpha_at(&bag, 0.0, 0.5), 255);
        assert_eq!(alpha_at(&bag, 0.7, 0.5), 0);
        // Three dots, and nothing between them.
        let dots = dots();
        for x in [-0.56, 0.0, 0.56] {
            assert_eq!(alpha_at(&dots, x, 0.0), 255, "dot at {x}");
        }
        assert_eq!(alpha_at(&dots, -0.28, 0.0), 0);
        assert_eq!(alpha_at(&dots, 0.0, 0.5), 0);
        // The plank is a board lying across, with two seams cut into it,
        // and nothing over or under it.
        let plank = plank();
        for x in [-0.7, -0.1, 0.1, 0.7] {
            assert_eq!(alpha_at(&plank, x, 0.0), 255, "board at {x}");
        }
        assert_eq!(alpha_at(&plank, -0.36, 0.0), 0);
        assert_eq!(alpha_at(&plank, 0.36, 0.2), 0);
        assert_eq!(alpha_at(&plank, 0.0, 0.6), 0);
        assert_eq!(alpha_at(&plank, 0.95, 0.0), 0);
        for image in [&sword, &bag, &dots, &plank] {
            assert!(image.data.as_ref().unwrap().chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
        }
    }

    #[test]
    fn the_compass_s_needle_points_up_its_picture() {
        // `compass::point` turns the picture from pointing up the screen
        // to pointing at whoever it tracks, so up is where its tip has to
        // be: solid down its middle from near the top, nothing either side
        // of the tip, wide at its tail, and the notch cut out of the tail's
        // middle. Drawn the other way up it pointed back at you.
        let needle = Picture::Compass.image();
        for y in [0.8, 0.4, 0.0, -0.3] {
            assert_eq!(alpha_at(&needle, 0.0, y), 255, "its middle at {y}");
        }
        assert_eq!(alpha_at(&needle, 0.3, 0.8), 0);
        assert_eq!(alpha_at(&needle, -0.3, 0.8), 0);
        assert_eq!(alpha_at(&needle, 0.52, -0.7), 255);
        assert_eq!(alpha_at(&needle, -0.52, -0.7), 255);
        assert_eq!(alpha_at(&needle, 0.0, -0.65), 0);
        assert_eq!(alpha_at(&needle, 0.0, -0.97), 0);
    }

    #[test]
    fn each_item_s_picture_is_its_model_seen_from_above() {
        use std::f32::consts::TAU;
        // Each a stone of its own number of sides: solid in its middle,
        // the outline of its table cut through it, solid again between
        // the table and its edge, and nothing past its edge; with as many
        // corners round it as it has sides, each cut off.
        for (item, sides, turn, cut, middle, reach, table) in [
            (Item::Silver, 3, TAU / 4.0, 10.0f32, Vec2::new(0.0, -0.17), 1.0, 0.70),
            (Item::Gold, 4, TAU / 8.0, 12.0, Vec2::ZERO, 1.08, 0.77),
            (Item::Sapphire, 5, TAU / 4.0, 13.0, Vec2::new(0.0, -0.05), 0.95, 0.775),
            (Item::Amethyst, 6, 0.0, 13.0, Vec2::ZERO, 0.92, 0.77),
        ] {
            let image = item_picture(item);
            let at = |angle: f32, out: f32| {
                let point = middle + Vec2::new(angle.cos(), angle.sin()) * out;
                alpha_at(&image, point.x, point.y)
            };
            assert_eq!(alpha_at(&image, middle.x, middle.y), 255, "{item:?}: its middle");
            let step = TAU / sides as f32;
            for k in 0..sides {
                let corner = turn + k as f32 * step;
                // Toward a corner: the table, its outline cut, the crown,
                // and nothing where the corner would be, it being cut off.
                assert_eq!(at(corner, reach * table * 0.5), 255, "{item:?}: its table, corner {k}");
                assert_eq!(at(corner, reach * table), 0, "{item:?}: its table's outline, corner {k}");
                assert_eq!(at(corner, reach * (table + 1.0) * 0.5 - 0.01), 255, "{item:?}: its crown, corner {k}");
                assert_eq!(at(corner, reach + 0.03), 0, "{item:?}: past corner {k}");
                // Toward the middle of a side: solid nearly as far as the
                // side, which is nearer than a corner, and nothing past it.
                let side = corner + step * 0.5;
                let edge = reach * (step * 0.5 - cut.to_radians()).cos();
                assert_eq!(at(side, edge * 0.86), 255, "{item:?}: inside side {k}");
                assert_eq!(at(side, edge + 0.04), 0, "{item:?}: past side {k}");
            }
            // All of it inside the picture, and white for the row to tint.
            let size = image.width();
            let data = image.data.as_ref().unwrap();
            for edge in 0..size {
                for (column, row) in [(edge, 0), (edge, size - 1), (0, edge), (size - 1, edge)] {
                    assert_eq!(data[((row * size + column) * 4 + 3) as usize], 0, "{item:?} runs off the picture");
                }
            }
            assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
        }
        // And no two alike: a triangle has nothing where a hexagon has
        // its upper left corner.
        let (silver, amethyst) = (item_picture(Item::Silver), item_picture(Item::Amethyst));
        assert_eq!(alpha_at(&amethyst, -0.6, 0.55), 255);
        assert_eq!(alpha_at(&silver, -0.6, 0.55), 0);
    }

    #[test]
    fn an_item_s_mark_for_the_minimap_is_its_stone_filled_and_small() {
        use std::f32::consts::TAU;
        // The sapphire's and the amethyst's, which the minimap shows: the
        // stone its picture is, with nothing cut through it, drawn small.
        for (item, sides, turn, middle, reach, table) in [
            (Item::Sapphire, 5, TAU / 4.0, Vec2::new(0.0, -0.05), 0.95, 0.775),
            (Item::Amethyst, 6, 0.0, Vec2::ZERO, 0.92, 0.77),
        ] {
            let (mark, picture) = (item_mark(item), item_picture(item));
            assert_eq!((mark.width(), mark.height()), (32, 32));
            assert!(mark.width() < picture.width());
            // The same stone, a little smaller in its picture.
            let at = |image: &Image, fills: f32, angle: f32, out: f32| {
                let point = (middle + Vec2::new(angle.cos(), angle.sin()) * out) * fills;
                alpha_at(image, point.x, point.y)
            };
            assert_eq!(at(&mark, MARK_FILLS, 0.0, 0.0), 255, "{item:?}: its middle");
            let step = TAU / sides as f32;
            for k in 0..sides {
                let corner = turn + k as f32 * step;
                // Solid where the picture has the line round its table
                // cut, and nothing past where a corner would be.
                assert_eq!(at(&picture, 1.0, corner, reach * table), 0);
                assert_eq!(at(&mark, MARK_FILLS, corner, reach * table), 255, "{item:?}: a hole in its mark, corner {k}");
                assert_eq!(at(&mark, MARK_FILLS, corner, reach + 0.12), 0, "{item:?}: past corner {k}");
            }
            let size = mark.width();
            let data = mark.data.as_ref().unwrap();
            for edge in 0..size {
                for (column, row) in [(edge, 0), (edge, size - 1), (0, edge), (size - 1, edge)] {
                    assert_eq!(data[((row * size + column) * 4 + 3) as usize], 0, "{item:?} runs off its mark");
                }
            }
            assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
        }
    }

    #[test]
    fn a_polygon_is_inside_out_neither_way_round() {
        let square = [Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0), Vec2::new(1.0, 1.0), Vec2::new(-1.0, 1.0)];
        let reversed: Vec<Vec2> = square.iter().rev().copied().collect();
        for corners in [&square[..], &reversed[..]] {
            assert!((polygon_distance(Vec2::ZERO, corners) + 1.0).abs() < 1e-5);
            assert!((polygon_distance(Vec2::new(2.0, 0.0), corners) - 1.0).abs() < 1e-5);
            assert!((polygon_distance(Vec2::new(0.5, 0.5), corners) + 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn every_icon_is_there_white_on_clear_and_in_the_apk() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim_start_matches("<dir>").trim_start_matches("<file>"))
            .collect();
        for icon in ICONS {
            let picture = format!("icons/{icon}.png");
            let bytes = std::fs::read(root.join("assets").join(&picture)).expect(&picture);
            assert_eq!(&bytes[1..4], b"PNG", "{picture}");
            for name in picture.split('/') {
                let left_out = patterns.iter().find(|pattern| {
                    let pattern: &str = pattern;
                    if let Some(end) = pattern.strip_prefix('*') {
                        name.ends_with(end)
                    } else if let Some(start) = pattern.strip_suffix('*') {
                        name.starts_with(start)
                    } else {
                        pattern == name
                    }
                });
                assert!(left_out.is_none(), "{picture} is left out of the APK by `{}`", left_out.unwrap());
            }
        }
    }

    #[test]
    fn a_triangle_is_inside_out_neither_way_round() {
        let (a, b, c) = (Vec2::new(0.0, 1.0), Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0));
        for (a, b, c) in [(a, b, c), (a, c, b)] {
            assert!(triangle_distance(Vec2::ZERO, a, b, c) < 0.0);
            assert!((triangle_distance(Vec2::new(0.0, -2.0), a, b, c) - 1.0).abs() < 1e-5);
        }
    }
}
