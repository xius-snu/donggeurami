//! What is drawn over the world, laid out as Hajun sketched it
//! (`assets/UIbuttonlayout.jpg`, 2026-10-31): top left your name on a plate,
//! with a person button and the settings' gear beside it (`settings`) and
//! your balance under; down the right edge a gift, a speech bubble (your
//! friends, `friends`) and a gamepad; round a phone's jump button a face, a
//! speech bubble and a fist; and headlines across the middle — the town's
//! name, as a welcome, whenever you arrive in the town, and whatever else
//! is [`announce`]d, like House Builder's theme. And how the stick and the
//! jump button look (`setup_hud`; a phone's, and since 2026-11-06 the
//! desktop's too, driven by the mouse). Of the buttons only the gear, the stick,
//! the jump button, the fist, the face, the gamepad (the game selector,
//! `games`), the speech bubble under the gift (the friends' panel, since
//! 2026-11-13; [`FriendsButton`]) and, in the car, the gift do anything
//! yet (Hajun: "i dont need you to make them functional yet. just make
//! them in the screen").
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

use crate::Player;
use crate::arena::{Health, Resources};
use crate::bridge::BridgePressed;
use crate::drive::{CarPressed, Driving};
use crate::island::{Islands, Venue};
use crate::lobby::{Balance, STARTING_BALANCE};
use crate::minimap::{Minimap, MinimapNode};
use crate::signup::Signup;
use crate::ui::{UiChanged, ui};

pub(crate) const COIN_ICON: &str = "coin.png";

/// Sizes, as shares of the short side of the screen, so that a phone and a
/// monitor show the same thing: your name on its plate and your balance
/// under it; the round buttons beside your name, the bigger ones down the
/// right edge, and the bigger still round a phone's jump button (Hajun,
/// 2026-10-31: "keep the size of the user icon and the settings icon but
/// increase the size of every button the right hand side both top and
/// bottom"; 8 and 12 until then).
const NAME_TEXT: f32 = 4.2;
const BALANCE_TEXT: f32 = 5.0;
const BALANCE_ICON: f32 = 5.0;
pub(crate) const SMALL_BUTTON: f32 = 8.0;
/// The buttons down the right edge: 9.5 until Hajun asked for "very
/// slightly" bigger (2026-11-01), and that much further in from the top
/// and the right than the name is ([`EDGE`]), "very slightly" more margin.
const SIDE_BUTTON: f32 = 10.5;
const SIDE_INSET: f32 = 1.5;
const ACTION_BUTTON: f32 = 14.0;
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
const ARENA_BRIDGE: (f32, f32) = (4.0, 24.5);
const ARENA_DOTS: (f32, f32, f32) = (0.0, 42.0, 8.0);
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
fn slot_place(slot: usize) -> (f32, f32) {
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
const PICTURE: f32 = 58.0;
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
/// gear beside your name, a gamepad down the right edge, and a face and a
/// speech bubble round the jump button; and Font Awesome's gift and
/// comment-dots (`LICENSE-font-awesome.txt`; the ones Hajun's FlutterFlow
/// screenshots showed, 2026-11-01, rendered from their SVGs), the gift and
/// the social chat's bubble down the right edge. The fist with the face
/// and the bubble is drawn in code ([`fist`]). A test keeps every one in
/// the APK. The leaderboard's (`leaderboard.png`) and the boxing glove
/// (`punch.png`) are still beside them, unused since 2026-11-01. And, since
/// 2026-11-13, the friends' panel's (`friends`), Material's too: two people
/// (`group`), a person with a plus (`person_add`), a magnifying glass
/// (`search`), two sheets (`content_copy`), an envelope (`mail`), a cross
/// (`close`), a chevron (`arrow_back_ios_new`), a door (`logout`), and the
/// group of three (`groups`), unused for twelve days until then.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ICONS: [&str; 16] = [
    "person", "settings", "gift", "social", "gamepad", "mood", "chat", "friends", "add_friend", "search", "copy", "mail",
    "close", "back", "leave", "groups",
];

/// Your balance, top left.
#[derive(Component)]
struct BalanceText;

/// Your name, top left, on its plate.
#[derive(Component)]
struct NamePlate;

/// Which buttons are round the screen: the town's, everywhere but in a game
/// of Bridge Wars, where they are the game's own (`arena`; Hajun,
/// the night of 2026-11-07: "in the pvpmap game ... the button ui should change to
/// pvpmapbuttonui.jpg layout"). It follows the island you are on
/// ([`relayout`]).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Layout {
    #[default]
    Town,
    Arena,
}

/// The gamepad, down the right edge, which opens the game selector
/// (`games`).
#[derive(Component)]
pub(crate) struct GamesButton;

/// The speech bubble over it, under the gift, which opens your friends
/// (`friends`, since 2026-11-13; Hajun: "The friend panel popup will open
/// when u tap the top right corner chat bubblt icon under the gift icon").
#[derive(Component)]
pub(crate) struct FriendsButton;

/// The fill of your health bar in Bridge Wars, as wide as your health.
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
/// have been up.
#[derive(Component)]
struct Headline {
    age: f32,
    secs: f32,
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

/// Where the things that take a press for themselves are on the screen, as
/// they were last laid out.
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
        .add_observer(press_buttons)
        .add_systems(Startup, show_top_bar)
        .add_systems(Update, shade)
        .add_systems(
            Update,
            (
                relayout,
                restyle.after(relayout),
                show_name.after(restyle),
                count_balance.after(restyle),
                show_health.after(restyle),
                show_resources.after(restyle),
                hide_round_the_jump,
                release_held,
                repicture.after(restyle),
                (arrive, welcome, fade_headlines).chain(),
            ),
        );
}

/// The bar across the top: your name, the buttons and your balance.
#[derive(Component)]
struct TopBar;

/// The box round a phone's jump button, with the three buttons about it.
#[derive(Component)]
struct RoundTheJump;

/// The fist among them, or in Bridge Wars the sword in the jump button's
/// place, which throws a punch (`start_punch` in `src/lib.rs`) and, in the
/// car, held, is the reverse (`drive`): its pictures, its own, and in the
/// car the jump button's arrow turned down.
#[derive(Component)]
struct PunchButton {
    picture: Handle<Image>,
    reverse: Handle<Image>,
}

/// The face among them, which puts you in the car and out of it (`drive`;
/// Hajun, 2026-11-05: "make the emote button to be a toggle on/off switch
/// for car").
#[derive(Component)]
struct CarButton;

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

/// A press on the fist, the face or the gift, as it goes down rather than
/// as it comes up, so that quick presses land as quickly as they are made:
/// a punch is asked for, and the fist is held from then; or the car; or the
/// gift is held from then.
fn press_buttons(
    press: On<Pointer<Press>>,
    fists: Query<(), With<PunchButton>>,
    faces: Query<(), With<CarButton>>,
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
/// (`drive`): the face lit, as the jump button is held, while you are in
/// it, and on the fist the jump button's arrow turned down, for the
/// reverse; and on buttons put up afresh ([`restyle`]).
fn repicture(
    driving: Res<Driving>,
    faces: Query<Entity, With<CarButton>>,
    fists: Query<(&PunchButton, &Children)>,
    added: Query<(), Added<PunchButton>>,
    mut fills: Query<&mut BackgroundColor>,
    mut pictures: Query<&mut ImageNode>,
) {
    if !driving.is_changed() && added.is_empty() {
        return;
    }
    let on = driving.on();
    let colours = ui().colours.hud;
    for face in &faces {
        if let Ok(mut fill) = fills.get_mut(face) {
            fill.set_if_neq(BackgroundColor(if on { colours.button_held } else { colours.button }));
        }
    }
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
    mut boxes: Query<&mut Visibility, With<RoundTheJump>>,
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
/// there: Bridge Wars' own in a game of it, and the town's everywhere else;
/// taken down and put up afresh, as [`restyle`] does.
fn relayout(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    mut layout: ResMut<Layout>,
    bars: Query<Entity, Or<(With<TopBar>, With<RoundTheJump>)>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    minimap: Res<Minimap>,
    mut commands: Commands,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    let wanted = if here == Venue::Arena { Layout::Arena } else { Layout::Town };
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
    bars: Query<Entity, Or<(With<TopBar>, With<RoundTheJump>)>>,
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
/// has them.
fn spawn_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>, layout: Layout, minimap: &Minimap) {
    match layout {
        Layout::Town => spawn_town_bar(commands, assets, images),
        Layout::Arena => spawn_arena_bar(commands, images, minimap),
    }
}

/// The box round the jump button, in the bottom right corner where the
/// jump button is: the buttons about it are placed from its corner, in
/// shares of the short side.
fn round_the_jump() -> impl Bundle {
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
fn beside_the_jump(right: f32, bottom: f32) -> Node {
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
        let (right, bottom, size) = ARENA_DOTS;
        round.spawn((MoreButton, round_button(images.add(dots()), size, beside_the_jump(right, bottom))));
    });
}

/// Bridge Wars' bridge button, where `place` puts it: a round button as
/// the others are ([`round_button`]), with the plank on it a little above
/// its middle and, under the plank, inside the disc, the metres of bridge
/// you have, small and white ([`BridgeCount`], written by
/// [`show_resources`]: "25m", as in front of you while you build). A
/// press anywhere on it is the button's.
fn bridge_button(plank: Handle<Image>, place: Node) -> impl Bundle {
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
fn jump_button(arrow: Handle<Image>, size_vmin: f32, place: Node) -> impl Bundle {
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
fn health_bar() -> impl Bundle {
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
fn resource_row(kind: Item, picture: Handle<Image>, tint: Color) -> impl Bundle {
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
/// is `slot` 0, or one of the four more (since 2026-11-13), which are put
/// away until something is on them or the dots show them. The buttons'
/// disc with a faint rim, empty until something to use is bought at the
/// item shop, and then with its picture on it, how many of it in its
/// corner, and over it, while a potion lasts, how long it still does.
/// `utility` fills it in, shows it and takes its presses: a press for a
/// potion, a hold and a drag for what is aimed.
fn utility_slot(slot: usize, place: Node) -> impl Bundle {
    let colours = ui().colours;
    (
        UtilitySlot(slot),
        if slot == 0 { Visibility::Inherited } else { Visibility::Hidden },
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
                    left: Val::Percent(-50.0),
                    right: Val::Percent(-50.0),
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

/// The town's buttons, as Hajun sketched them (`assets/UIbuttonlayout.jpg`,
/// 2026-10-31): the bar across the top, and the box round the jump button.
fn spawn_town_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>) {
    let icon = |name: &str| assets.load(format!("icons/{name}.png"));
    commands
        .spawn((
            TopBar,
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
                // a person, and the settings' gear, which `settings` puts
                // after it.
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
                    row.spawn(round_button(icon("person"), SMALL_BUTTON, Node::default()));
                });
                left.spawn(balance(assets));
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
                // a speech bubble the same day.
                for name in ["gift", "social", "gamepad"] {
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
    // placed from the box's corner, in shares of the short side: the face up
    // and to the left of it, the speech bubble over it and the fist beside
    // it, each about 2.5 shares clear of it, as Hajun sketched them.
    commands.spawn(round_the_jump()).with_children(|round| {
        let fist = images.add(fist());
        let arrow = images.add(jump_arrow());
        for (name, right, bottom) in [("mood", 18.5, 18.5), ("chat", 1.0, 24.3), ("punch", 24.3, 1.2)] {
            let picture = if name == "punch" { fist.clone() } else { icon(name) };
            let mut button = round.spawn(round_button(picture, ACTION_BUTTON, beside_the_jump(right, bottom)));
            // The fist throws a punch each time it is pressed, and the
            // face puts you in the car and out of it; in the car the
            // fist is the reverse (`drive`).
            match name {
                "punch" => {
                    button.insert(PunchButton {
                        picture: fist.clone(),
                        reverse: arrow.clone(),
                    });
                }
                "mood" => {
                    button.insert(CarButton);
                }
                _ => {}
            }
        }
    });
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

/// The cross on the corner of a window or a dialog that shuts it: a red
/// disc `size_vmin` across, hung over the top right corner of whatever it
/// is spawned under, with two white strokes across it, carrying `marker`,
/// which says whose it is and what a press on it does.
pub(crate) fn close_cross(size_vmin: f32, marker: impl Bundle) -> impl Bundle {
    let colours = ui().colours;
    (
        marker,
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(-size_vmin * 0.35),
            right: Val::VMin(-size_vmin * 0.35),
            width: Val::VMin(size_vmin),
            height: Val::VMin(size_vmin),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(colours.close),
        BorderColor::all(colours.close_rim),
        children![
            (shape(22.0, 44.0, 56.0, 12.0), UiTransform::from_rotation(Rot2::degrees(45.0)),),
            (shape(22.0, 44.0, 56.0, 12.0), UiTransform::from_rotation(Rot2::degrees(-45.0)),),
        ],
    )
}

/// Your name on a plate: the buttons' grey, a little more solid, with its
/// corners rounded a little.
fn name_plate() -> impl Bundle {
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
            NamePlate,
            Text::new(""),
            crate::font::sized(NAME_TEXT),
            TextColor(colours.text),
            Pickable::IGNORE,
        )],
    )
}

/// Shows your name on its plate, as you are called now: as the app opens,
/// again when you sign up or change it (`signup`), and on a plate put up
/// afresh ([`restyle`]).
fn show_name(me: Query<Ref<Name>, With<Player>>, mut plates: Query<&mut Text, With<NamePlate>>) {
    let Ok(name) = me.single() else {
        return;
    };
    for mut text in &mut plates {
        if (name.is_changed() || text.is_added()) && text.0 != name.as_str() {
            text.0 = name.to_string();
        }
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
    let mut headline = commands.spawn((
        Headline { age: 0.0, secs },
        // Over the whole screen, with the words stacked in the middle and high
        // enough to leave your character, just below the middle, in sight.
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
    ));
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

/// Brings each headline up, holds it, and takes it away again.
fn fade_headlines(
    time: Res<Time>,
    mut headlines: Query<(Entity, &mut Headline, &Children)>,
    mut lines: Query<(&mut TextColor, &mut TextShadow)>,
    mut commands: Commands,
) {
    let tint = ui().colours.text_shadow;
    for (entity, mut headline, children) in &mut headlines {
        headline.age += time.delta_secs();
        if headline.age >= headline.secs {
            commands.entity(entity).despawn();
            continue;
        }
        let solid = (headline.age / HEADLINE_IN)
            .min((headline.secs - headline.age) / HEADLINE_OUT)
            .clamp(0.0, 1.0);
        let mut words = lines.iter_many_mut(children);
        while let Some((mut colour, mut shadow)) = words.fetch_next() {
            // Only while it is fading: held, the words are left alone.
            let faded = colour.0.with_alpha(solid);
            if colour.0 != faded {
                colour.0 = faded;
                shadow.color = tint.with_alpha(tint.alpha() * solid);
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
}

impl Picture {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const ALL: [Picture; 13] = [
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
    ];

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
            Picture::Builder => drawn(|at| {
                let head = circle_distance(at, Vec2::new(0.0, 0.6), 0.24);
                let body = box_distance(at - Vec2::new(0.0, 0.02), Vec2::new(0.2, 0.3)) - 0.06;
                let plank = box_distance(at - Vec2::new(0.0, -0.72), Vec2::new(0.82, 0.12)) - 0.04;
                head.min(body).min(plank)
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
            // does, with the fuse out of their top and a spark at its end
            // (since 2026-11-13).
            Picture::Dynamite => drawn(|at| {
                let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
                // `at` as the sticks see it, standing up their own y.
                let local = Vec2::new(at.x * cos - at.y * sin, at.x * sin + at.y * cos);
                let sticks = [-0.24, 0.24]
                    .into_iter()
                    .map(|x| box_distance(local - Vec2::new(x, -0.2), Vec2::new(0.15, 0.5)) - 0.06)
                    .fold(f32::INFINITY, f32::min);
                let fuse = box_distance(local - Vec2::new(0.0, 0.52), Vec2::new(0.045, 0.2));
                let spark = circle_distance(local, Vec2::new(0.0, 0.82), 0.13);
                sticks.min(fuse).min(spark)
            }),
        }
    }
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

/// A bag, for Bridge Wars' inventory button: a round sack with its neck
/// tied, flaring out over the tie. The button went on 2026-11-09, for the
/// minimap; the picture stays for when there is an inventory.
#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg(test)]
mod tests {
    use super::*;

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
