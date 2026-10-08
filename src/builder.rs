//! House Builder: the game the House Builder asks you to play, from the door
//! of their house in the town, called [`NAME`] on the screen.
//!
//! Walk up to them and a speech bubble comes up over their head. Tap it — on
//! desktop, press Space — and a dialog asks whether to play, for [`ENTRY`]
//! coins. Say yes, and you pay and are taken to the lobby (`lobbymap.glb`) to
//! wait for the game to fill, with the count of players at the top of the
//! screen. There is no server yet, so nobody else ever comes: over
//! [`QUEUE_FILL`] seconds the computer takes every seat still empty, one at a
//! time, each of its players paying its way in like anybody else.
//!
//! With all eight there, each builder goes to a plot of their own — a copy of
//! `homemap.glb` each, [`Venue::Plot`] — is given a theme, and has
//! [`BUILD_SECS`] to build a house to it: the hammer button opens the shop
//! (`shop`), and what is chosen there is put down on your plot (`build`). The
//! computer's players build nothing.
//! Then everyone visits every plot in turn, seat by seat, [`VISIT_SECS`] at
//! each, and gives the house there one to five stars. Nobody rates their own,
//! and the computer's players rate at random. The house with the most stars
//! wins, a tie being settled by lot, and a dialog says whose it was, with the
//! choice of playing again or going back to the town.
//!
//! Tapping the timer at the top of the screen ends the wait early. That is for
//! trying the game out, and not meant to stay.
//!
//! Building and visiting are seen from behind you, as the town is. A button
//! over the hammer swaps that for a view through your own eyes, and back.
//!
//! Everything a game puts in the world or on the screen carries [`InGame`] and
//! goes when the game does. Your own body is the only thing that comes back.
//!
//! Every button here says what it is for with [`AsksFor`], and a click on one
//! becomes an [`Ask`], the same as a key: [`answer`] deals with coming and
//! going, [`play`] with the game itself, and [`publish_view`] with how you see
//! it.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::world_serialization::WorldInstanceReady;

use bevy_replicon::prelude::Remote;
// What a game costs, how long each part of it lasts and the most stars a
// house can have: the rules the server plays by online, kept with it.
use roundtown_net::rules::{BUILD_SECS, ENTRY, FULL_HOLD, MOST_STARS, QUEUE_FILL, VISIT_SECS};
use roundtown_net::{Round, Seats, Tell};

use crate::hud::{self, DOOR, DialogUp, GOLD, PANEL, TakesPress};
use crate::island::{self, Islands, Venue};
use crate::lobby::{self, Balance, Lobby, Member, SEATS, STARTING_BALANCE, Seat, Whereabouts};
use crate::net::Online;
use crate::shop;
use crate::sky::Water;
use crate::{
    Collider, ColliderShape, FirstPerson, LAND_TOP, OrbitCamera, PLAYER_HEIGHT, PLAYER_RADIUS,
    Player, ThirdPersonCamera, WalkCycle,
};

/// How long the theme stays across the screen when building begins.
const THEME_SECS: f32 = 4.5;
/// Your seat in a game played offline: you are always the first into the
/// room. Online the server seats you.
const YOU: usize = 0;
/// How long an online game waits for the connection to come back after it
/// has gone, in seconds, before it is given up and you are back in the town.
const RECONNECT_WAIT: f32 = 15.0;

/// What the game is called on the screen.
const NAME: &str = "Build Your Own House";
/// What the House Builder is called in the town's model.
const NPC: &str = "House Builder NPC";
/// How near the House Builder you have to be to talk to them, in metres.
const TALK_RANGE: f32 = 4.5;
/// How high over the House Builder's feet the bubble points from, in metres:
/// a little over their head.
const BUBBLE_LIFT: f32 = 2.05;

/// What a house can be asked to be. The built-in font is plain ASCII, so these
/// are too.
const THEMES: &[Theme] = &[
    Theme::WhoLivesThere("A Grandma and Grandpa"),
    Theme::WhoLivesThere("A Young Girl"),
    Theme::WhoLivesThere("An Adult Man"),
    Theme::WhoLivesThere("A Family"),
    Theme::WhoLivesThere("A Restaurant Chef"),
    Theme::Is("Cafe"),
    Theme::Is("Bakery"),
    Theme::Is("Library"),
    Theme::Is("Art Gallery"),
    Theme::Is("Movie Night"),
    Theme::Is("Birthday Party"),
    Theme::Is("Haunted House"),
    Theme::Is("Minimalist"),
    Theme::Is("Summer"),
    Theme::Is("Winter"),
    Theme::Is("My Dream House"),
];

/// A theme: who is to live in the house, or what it is to be. Each is worded
/// to suit where it is written.
#[derive(Clone, Copy)]
enum Theme {
    WhoLivesThere(&'static str),
    Is(&'static str),
}

impl Theme {
    /// The theme as it is written: "Who Lives There: A Family", "Cafe".
    fn name(self) -> String {
        match self {
            Theme::WhoLivesThere(who) => format!("Who Lives There: {who}"),
            Theme::Is(what) => what.into(),
        }
    }

    /// Across the screen as building begins: a line over it, the theme, and
    /// how big the theme is written. Who lives there says what it is already.
    fn headline(self) -> (&'static str, String, f32) {
        match self {
            Theme::WhoLivesThere(_) => ("Build a house!", self.name(), WHO_TEXT),
            Theme::Is(_) => ("Build a house! The theme is", self.name(), hud::HEADLINE),
        }
    }

    /// Under the time, for as long as the building lasts, and how big.
    fn caption(self) -> (String, f32) {
        match self {
            Theme::WhoLivesThere(_) => (self.name(), WHO_CAPTION),
            Theme::Is(what) => (format!("Theme: {what}"), CAPTION),
        }
    }
}

/// What the House Builder says, and how to answer: a phone taps the bubble,
/// and a keyboard has Space.
const SAY: &str = "Want to build a house?";
const HINT: &str = if cfg!(any(target_os = "android", target_os = "ios")) {
    "Tap here to play"
} else {
    "Press Space to play"
};
/// The game in three lines, broken where they read best: the dialog is wide
/// enough for the longest.
const RULES: &str = "Build and decorate your own\nhouse according to a theme.\nThe best house wins!";

/// Sizes, as shares of the short side of the screen, like everything else
/// over the world.
const SAY_TEXT: f32 = 4.0;
const HINT_TEXT: f32 = 3.0;
const TAIL: f32 = 3.2;
/// How far down the screen the bubble's top has to stay: below the balance
/// and the button along the top.
const BUBBLE_TOP: f32 = 18.0;
const BOARD_TEXT: f32 = 7.0;
const CAPTION: f32 = 4.2;
/// Who lives there is written smaller than other themes, being longer: across
/// the screen as building begins, small enough for the longest to fit a 4:3
/// iPad on one line, and under the time.
const WHO_TEXT: f32 = 5.0;
const WHO_CAPTION: f32 = 3.6;
const STAR: f32 = 11.0;
const STAR_GAP: f32 = 2.2;
/// How far up from the bottom of the screen the stars sit: below the stick
/// and the jump button on a phone, and clear of both sideways.
const STARS_UP: f32 = 5.0;
const DIALOG_WIDTH: f32 = 90.0;
/// Small enough for the game's name to fit across the dialog on one line.
pub(crate) const TITLE: f32 = 6.5;
pub(crate) const BODY: f32 = 4.2;
pub(crate) const SMALL: f32 = 3.6;
pub(crate) const BUTTON_WIDTH: f32 = 36.0;
pub(crate) const BUTTON_HEIGHT: f32 = 11.0;
pub(crate) const BUTTON_TEXT: f32 = 4.8;
const PRICE_ICON: f32 = 5.5;
const PRICE_TEXT: f32 = 5.0;
/// Dialogs are over everything else on the screen.
pub(crate) const DIALOG_LAYER: i32 = 10;

const BUBBLE: Color = Color::srgba(1.0, 1.0, 1.0, 0.96);
const HINT_COLOUR: Color = Color::srgb(0.36, 0.50, 0.62);
/// The dark over the world behind a dialog, and the dialog itself.
pub(crate) const DIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
const DIALOG: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
const PANEL_LIT: Color = Color::srgba(0.04, 0.10, 0.18, 0.6);
pub(crate) const QUIET: Color = Color::srgba(1.0, 1.0, 1.0, 0.14);
pub(crate) const QUIET_LIT: Color = Color::srgba(1.0, 1.0, 1.0, 0.26);
const GOLD_LIT: Color = Color::srgb(1.0, 0.91, 0.55);
const OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.07);
const OFF_TEXT: Color = Color::srgba(1.0, 1.0, 1.0, 0.35);
pub(crate) const SOFT: Color = Color::srgba(1.0, 1.0, 1.0, 0.78);
pub(crate) const WARN: Color = Color::srgb(1.0, 0.62, 0.55);
/// A star given, and one not.
const STAR_ON: Color = GOLD;
const STAR_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);

/// Something asked of House Builder, by a tap, a click or a key.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
enum Ask {
    /// Talk to the House Builder, who asks whether you want to play.
    Talk,
    /// Pay, and join a game: from that question, or after one game for
    /// another.
    Join,
    /// No thank you: put the question away.
    Cancel,
    /// End the wait early, for trying the game out.
    Skip,
    /// Give the house being visited this many stars.
    Rate(u8),
    /// Go back to the town after a game.
    Leave,
    /// See yourself from behind rather than through your own eyes while you
    /// build and rate, or the other way round.
    SwapView,
}

/// What a button asks for when it is clicked or tapped.
#[derive(Component, Clone, Copy)]
struct AsksFor(Ask);

/// The House Builder, standing at the door of their house in the town: where
/// their model puts them, and where their feet are once the town's shape can
/// say.
#[derive(Component)]
struct HouseBuilder {
    at: Vec3,
    feet: Option<Vec3>,
}

/// Where the bubble over the House Builder points from, while you are near
/// enough to talk to them.
#[derive(Resource, Default)]
struct Talk {
    over: Option<Vec3>,
}

/// The speech bubble over the House Builder's head.
#[derive(Component)]
struct Bubble;

/// Which of House Builder's dialogs is up, if either.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Showing {
    #[default]
    Nothing,
    /// Whether to play.
    Offer,
    /// Who won, and whether to play again.
    Results,
}

/// The root of one of the dialogs.
#[derive(Component)]
struct Dialog;

/// Everything a game has put in the world or on the screen, to go with it.
#[derive(Component)]
pub(crate) struct InGame;

/// The plot you are building on, while the time to build runs; otherwise
/// nothing.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct Building(pub Option<u8>);

/// Whether you see yourself from behind while you build and rate, rather than
/// through your own eyes: from behind, until the button over the hammer is
/// pressed. Kept for as long as the app is open, from one game to the next.
#[derive(Resource)]
struct FromBehind(bool);

impl Default for FromBehind {
    fn default() -> Self {
        Self(true)
    }
}

/// The button over the hammer that swaps the view.
#[derive(Component)]
struct ViewButton;

/// One of the pictures on it: the view it swaps to. Only that one is shown.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum ViewPicture {
    /// A person: you, seen from behind.
    Behind,
    /// An eye: through your own.
    Eyes,
}

/// The theme, across the screen as building begins. When the time is cut
/// short it goes with the building rather than staying up over the rating.
#[derive(Component)]
struct ThemeHeadline;

/// The board top centre — the count of players, then the time left — the
/// words on it, and the line under it saying what they count.
#[derive(Component)]
struct Board;
#[derive(Component)]
struct BoardText;
#[derive(Component)]
struct BoardCaption;

/// The stars along the bottom while a house is visited, the line over them,
/// the row they are in, and each of them.
#[derive(Component)]
struct Rating;
#[derive(Component)]
struct RatingCaption;
#[derive(Component)]
struct StarRow;
#[derive(Component)]
struct StarButton(u8);

/// The star the rating buttons are made of.
#[derive(Resource)]
struct StarImage(Handle<Image>);

/// A button's colour at rest, and lit up while it is under the pointer or
/// held.
#[derive(Component, Clone, Copy)]
pub(crate) struct Fill {
    pub idle: Color,
    pub lit: Color,
}

/// A game of House Builder, from joining it to leaving it.
#[derive(Resource)]
struct Game {
    /// Everyone in it, in seat order: offline you first, then whoever came
    /// after. A builder's plot is their seat.
    seats: Vec<Member>,
    /// Their bodies, in the same order, offline. Online only yours is here:
    /// the server moves everyone else.
    bodies: Vec<Entity>,
    /// Your seat, and so your plot.
    me: usize,
    /// The server is running it ([`follow_round`]), rather than this device.
    online: bool,
    /// Online, how long the connection has been gone, if it has.
    cut_off: Option<f32>,
    stage: Stage,
    /// What every house is to be, this game.
    theme: Theme,
    /// The stars each house has been given so far, by plot.
    stars: [u32; SEATS],
    /// The stars you have given the house being visited, if any yet.
    mine: Option<u8>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    /// In the lobby, waiting for the room to fill.
    Queue { waited: f32 },
    /// Full, and about to go.
    Full { left: f32 },
    /// Everyone on their own plot, building.
    Build { left: f32 },
    /// Everyone at the house on `plot`, rating it.
    Visit { plot: usize, left: f32 },
    /// Done: the house on `winner` had the most stars.
    Over { winner: usize },
}

impl Game {
    /// A house is being visited that you may rate: anyone's but yours.
    fn rating(&self) -> bool {
        matches!(self.stage, Stage::Visit { plot, .. } if plot != self.me)
    }

    /// Everyone is building or visiting: up close to a house, and seeing it
    /// from behind themselves unless they would rather see it through their
    /// own eyes.
    fn up_close(&self) -> bool {
        matches!(self.stage, Stage::Build { .. } | Stage::Visit { .. })
    }

    /// Whose house `plot` is, as it is written on the screen.
    fn whose(&self, plot: usize) -> String {
        if plot == self.me {
            "Your house".into()
        } else {
            self.seats
                .get(plot)
                .map_or_else(|| "A house".into(), |seat| format!("{}'s house", seat.name))
        }
    }
}

/// Your plot in a game of House Builder played online, for as long as the
/// game lasts: what is put down on it goes to the server (`build`), and what
/// the server says is down on everyone else's is built there too.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct OnlinePlot(pub Option<u8>);

/// A game of House Builder is being played offline: nothing goes online in
/// the middle of it (`net`).
#[derive(Resource, Default, PartialEq)]
pub(crate) struct OfflineGame(pub bool);

/// House Builder's reading of taps, clicks and keys, and the game they drive.
/// Ahead of your own walking, so that a key it takes is not also a jump, and
/// of the islands being shown, so that wherever it sends you is drawn the
/// same frame.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GameSystems;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Ask>()
        .init_resource::<Talk>()
        .init_resource::<Showing>()
        .init_resource::<Building>()
        .init_resource::<FromBehind>()
        .init_resource::<OnlinePlot>()
        .init_resource::<OfflineGame>()
        .add_observer(find_house_builder)
        .add_observer(ask)
        .add_systems(Startup, (draw_star, spawn_bubble, spawn_view_button))
        .add_systems(
            Update,
            (
                settle_house_builder,
                (
                    talk,
                    keys,
                    answer,
                    join_online,
                    follow_round,
                    cut_off,
                    play,
                    publish_building,
                    publish_view,
                )
                    .chain()
                    .in_set(GameSystems)
                    .before(crate::read_player_input)
                    .before(island::show_where_i_am),
                place_bubble.after(crate::follow_camera),
                (show_board, show_rating, show_view_button, light_buttons).after(GameSystems),
            ),
        );
}

// ------------------------------------------------------- the House Builder

/// Finds the House Builder in the town's model once it has spawned, by name,
/// wherever in the town they have been put. They are built on the same
/// skeleton as everyone else, and stand the way everyone else stands still,
/// with their arms at their sides rather than out as the model has them.
fn find_house_builder(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    named: Query<(&Name, &Transform)>,
    placement: TransformHelper,
    mut commands: Commands,
) {
    let npcs: Vec<Entity> = children
        .iter_descendants(ready.entity)
        .filter(|&entity| {
            named
                .get(entity)
                .is_ok_and(|(name, _)| name.as_str() == NPC)
        })
        .collect();
    for npc in npcs {
        let Ok(place) = placement.compute_global_transform(npc) else {
            continue;
        };
        commands.entity(npc).insert((
            HouseBuilder {
                at: place.translation(),
                feet: None,
            },
            WalkCycle::default(),
        ));
        crate::bind_limbs(npc, &children, &named, &mut commands);
    }
}

/// Stands the House Builder on the town's ground once there is a shape to read
/// it off, and makes them solid: someone to walk up to, not through. The model
/// is only drawn, the island's shape leaving bodies out (`island::read_island`).
fn settle_house_builder(
    islands: Res<Islands>,
    mut builders: Query<&mut HouseBuilder>,
    mut commands: Commands,
) {
    let Some(town) = islands.get(Venue::Town) else {
        return;
    };
    for mut builder in &mut builders {
        if builder.feet.is_some() {
            continue;
        }
        let ground = town
            .floor(builder.at.xz(), builder.at.y)
            .unwrap_or(LAND_TOP);
        let feet = builder.at.with_y(ground);
        builder.feet = Some(feet);
        commands.spawn((
            Transform::from_translation(feet),
            Collider {
                shape: ColliderShape::Cylinder {
                    radius: PLAYER_RADIUS,
                    height: PLAYER_HEIGHT,
                },
            },
            Venue::Town,
        ));
    }
}

/// Whether you are near enough to the House Builder to talk to them — in the
/// town, with no dialog up — and on desktop, Space to do it. Space is taken
/// before your jump can read it, so that it is not a jump as well.
fn talk(
    me: Query<(&Transform, &Venue), With<Player>>,
    builders: Query<&HouseBuilder>,
    showing: Res<Showing>,
    up: Res<DialogUp>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut talk: ResMut<Talk>,
    mut asks: MessageWriter<Ask>,
) {
    let over = me
        .single()
        .ok()
        .filter(|&(_, &venue)| venue == Venue::Town && *showing == Showing::Nothing && !up.0)
        .and_then(|(you, _)| {
            builders
                .iter()
                .filter_map(|builder| builder.feet)
                .find(|feet| feet.xz().distance(you.translation.xz()) <= TALK_RANGE)
        })
        .map(|feet| feet + Vec3::Y * BUBBLE_LIFT);
    if talk.over != over {
        talk.over = over;
    }
    if over.is_some() && keyboard.clear_just_pressed(KeyCode::Space) {
        asks.write(Ask::Talk);
    }
}

// ---------------------------------------------------------------- answers

/// Whatever was clicked or tapped, if it asks for something. Only what the
/// press landed on, not what the click passed up through on its way out, so
/// that a tap on a dialog is not also a tap on the dark round it.
fn ask(click: On<Pointer<Click>>, buttons: Query<&AsksFor>, mut asks: MessageWriter<Ask>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(button) = buttons.get(click.entity) {
        asks.write(button.0);
    }
}

/// Keys, for desktop: Enter to say yes to a dialog and Escape to say no, 1 to
/// 5 for the stars to give the house being visited, and V for the button that
/// swaps the view, whenever it is there.
fn keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    showing: Res<Showing>,
    up: Res<DialogUp>,
    game: Option<Res<Game>>,
    mut asks: MessageWriter<Ask>,
) {
    let yes = keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    let no = keyboard.just_pressed(KeyCode::Escape);
    let viewing = !up.0 && game.as_ref().is_some_and(|game| game.up_close());
    if viewing && keyboard.just_pressed(KeyCode::KeyV) {
        asks.write(Ask::SwapView);
    }
    match *showing {
        Showing::Offer if yes => {
            asks.write(Ask::Join);
        }
        Showing::Offer if no => {
            asks.write(Ask::Cancel);
        }
        Showing::Results if yes => {
            asks.write(Ask::Join);
        }
        Showing::Results if no => {
            asks.write(Ask::Leave);
        }
        Showing::Nothing if game.is_some_and(|game| game.rating()) => {
            const STAR_KEYS: [[KeyCode; 2]; MOST_STARS as usize] = [
                [KeyCode::Digit1, KeyCode::Numpad1],
                [KeyCode::Digit2, KeyCode::Numpad2],
                [KeyCode::Digit3, KeyCode::Numpad3],
                [KeyCode::Digit4, KeyCode::Numpad4],
                [KeyCode::Digit5, KeyCode::Numpad5],
            ];
            for (stars, keys) in (1..).zip(STAR_KEYS) {
                if keyboard.any_just_pressed(keys) {
                    asks.write(Ask::Rate(stars));
                }
            }
        }
        _ => {}
    }
}

/// Puts the dialogs up and takes them down, and takes you into a game and out
/// of it again. Online it is the server that takes the fee and seats you
/// ([`join_online`]), and sends you back to the town after.
#[allow(clippy::too_many_arguments)]
fn answer(
    mut asks: MessageReader<Ask>,
    game: Option<Res<Game>>,
    online: Res<Online>,
    mut requests: MessageWriter<roundtown_net::Ask>,
    mut showing: ResMut<Showing>,
    mut up: ResMut<DialogUp>,
    dialogs: Query<Entity, With<Dialog>>,
    in_game: Query<Entity, With<InGame>>,
    mut me: Query<(Entity, &Seat, &mut Balance), With<Player>>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    assets: Res<AssetServer>,
    water: Res<Water>,
    star: Res<StarImage>,
    mut commands: Commands,
) {
    let Ok((you, seat, mut balance)) = me.single_mut() else {
        asks.clear();
        return;
    };
    for &ask in asks.read() {
        match (ask, *showing) {
            (Ask::Talk, Showing::Nothing) if game.is_none() => {
                offer(&mut commands, &assets, balance.0 >= ENTRY);
                show(&mut showing, &mut up, Showing::Offer);
            }
            (Ask::Cancel, Showing::Offer) => {
                take_down(&dialogs, &mut commands);
                show(&mut showing, &mut up, Showing::Nothing);
            }
            (Ask::Join, Showing::Offer | Showing::Results) if balance.0 >= ENTRY => {
                take_down(&dialogs, &mut commands);
                // The last game, if this is the next.
                clear_away(&in_game, &mut commands);
                show(&mut showing, &mut up, Showing::Nothing);
                if online.is() {
                    commands.remove_resource::<Game>();
                    requests.write(roundtown_net::Ask::Play);
                    continue;
                }
                balance.0 -= ENTRY;
                let game = start(&mut commands, &assets, &water, &star, you, YOU, false);
                commands.insert_resource(game);
                lobby::send(&mut bodies, &mut orbit, you, Venue::Lobby, YOU);
            }
            (Ask::Leave, Showing::Results) => {
                take_down(&dialogs, &mut commands);
                clear_away(&in_game, &mut commands);
                show(&mut showing, &mut up, Showing::Nothing);
                let online_game = game.as_ref().is_some_and(|game| game.online);
                commands.remove_resource::<Game>();
                if online_game && online.is() {
                    requests.write(roundtown_net::Ask::Leave);
                } else {
                    lobby::send(&mut bodies, &mut orbit, you, Venue::Town, seat.0);
                }
            }
            _ => {}
        }
    }
}

/// The server has seated you in a game of House Builder: its plots, the board
/// and the stars, to follow it with ([`follow_round`]). Seated again in the
/// game you were in when the connection went, that game goes on. Turned
/// away, from the end of the last game, you are sent back to the town.
///
/// Sent back to the town or home by the server with a game still up, that
/// game is over for you: the server has closed it, or let it go while the
/// connection was gone.
#[allow(clippy::too_many_arguments)]
fn join_online(
    mut tells: MessageReader<Tell>,
    game: Option<ResMut<Game>>,
    me: Query<(Entity, &Venue), With<Player>>,
    in_game: Query<Entity, With<InGame>>,
    dialogs: Query<Entity, With<Dialog>>,
    mut showing: ResMut<Showing>,
    mut up: ResMut<DialogUp>,
    assets: Res<AssetServer>,
    water: Res<Water>,
    star: Res<StarImage>,
    mut requests: MessageWriter<roundtown_net::Ask>,
    mut commands: Commands,
) {
    let mut game = game;
    for tell in tells.read() {
        let Ok((you, &here)) = me.single() else {
            continue;
        };
        match tell {
            Tell::Arrive {
                venue: Venue::Town | Venue::Home,
                ..
            } if game.as_ref().is_some_and(|game| game.online) => {
                take_down(&dialogs, &mut commands);
                clear_away(&in_game, &mut commands);
                commands.remove_resource::<Game>();
                game = None;
                show(&mut showing, &mut up, Showing::Nothing);
            }
            Tell::Joined { seat } => {
                let seat = usize::from(*seat);
                if let Some(game) = game.as_mut()
                    && game.online
                    && game.me == seat
                    && game.cut_off.is_some()
                {
                    game.cut_off = None;
                    continue;
                }
                clear_away(&in_game, &mut commands);
                let game = start(&mut commands, &assets, &water, &star, you, seat, true);
                commands.insert_resource(game);
            }
            Tell::Refused(refusal) => {
                info!("the server turned that down: {refusal:?}");
                if game.is_none() && matches!(here, Venue::Lobby | Venue::Plot(_)) {
                    requests.write(roundtown_net::Ask::Leave);
                }
            }
            _ => {}
        }
    }
}

/// Online, the game is the server's: whatever it says the round is, this
/// shows — the stage and the time left, who is in it, the theme — and as it
/// moves on, what goes with that here: the theme across the screen as
/// building begins, and the results at the end.
#[allow(clippy::too_many_arguments)]
fn follow_round(
    game: Option<ResMut<Game>>,
    rounds: Query<(&Round, &Seats), With<Remote>>,
    themes: Query<Entity, With<ThemeHeadline>>,
    me: Query<&Balance, With<Player>>,
    mut showing: ResMut<Showing>,
    mut up: ResMut<DialogUp>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    if !game.online {
        return;
    }
    let Ok((round, seats)) = rounds.single() else {
        return;
    };
    if game.seats != seats.0 {
        game.seats.clone_from(&seats.0);
    }
    game.theme = THEMES[usize::from(round.theme) % THEMES.len()];
    let left = f32::from(round.left);
    let stage = match round.stage {
        roundtown_net::Stage::Queue => Stage::Queue { waited: 0.0 },
        roundtown_net::Stage::Full => Stage::Full { left },
        roundtown_net::Stage::Build => Stage::Build { left },
        roundtown_net::Stage::Visit(plot) => Stage::Visit {
            plot: usize::from(plot),
            left,
        },
        roundtown_net::Stage::Over { winner, .. } => Stage::Over {
            winner: usize::from(winner),
        },
    };
    let was = game.stage;
    game.stage = stage;
    let building = |stage: Stage| matches!(stage, Stage::Build { .. });
    if building(stage) && matches!(was, Stage::Queue { .. } | Stage::Full { .. }) {
        let (caption, theme, size) = game.theme.headline();
        hud::announce(&mut commands, Some(caption), &theme, size, THEME_SECS)
            .insert((InGame, ThemeHeadline));
    }
    if building(was) && !building(stage) {
        for theme in &themes {
            commands.entity(theme).try_despawn();
        }
    }
    if let Stage::Visit { plot, .. } = stage
        && !matches!(was, Stage::Visit { plot: before, .. } if before == plot)
    {
        game.mine = None;
    }
    if let (Stage::Over { winner }, roundtown_net::Stage::Over { stars, .. }) =
        (stage, round.stage)
        && !matches!(was, Stage::Over { .. })
    {
        game.stars = stars;
        let affordable = me.single().is_ok_and(|balance| balance.0 >= ENTRY);
        results(&mut commands, &assets, &game, winner, affordable);
        show(&mut showing, &mut up, Showing::Results);
    }
}

/// Online, the connection gone in the middle of a game: it waits a moment
/// for it to come back, and then gives the game up, back to the town as it is
/// offline.
#[allow(clippy::too_many_arguments)]
fn cut_off(
    time: Res<Time>,
    online: Res<Online>,
    game: Option<ResMut<Game>>,
    me: Query<(Entity, &Seat), With<Player>>,
    in_game: Query<Entity, With<InGame>>,
    dialogs: Query<Entity, With<Dialog>>,
    mut showing: ResMut<Showing>,
    mut up: ResMut<DialogUp>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    if !game.online || online.is() {
        return;
    }
    let gone = match game.cut_off {
        Some(gone) => gone + time.delta_secs(),
        None => {
            hud::announce(&mut commands, None, "Reconnecting...", hud::CAPTION, 3.0)
                .insert(InGame);
            0.0
        }
    };
    game.cut_off = Some(gone);
    if gone < RECONNECT_WAIT {
        return;
    }
    let Ok((you, seat)) = me.single() else {
        return;
    };
    take_down(&dialogs, &mut commands);
    clear_away(&in_game, &mut commands);
    commands.remove_resource::<Game>();
    show(&mut showing, &mut up, Showing::Nothing);
    lobby::send(&mut bodies, &mut orbit, you, Venue::Town, seat.0);
    hud::announce(&mut commands, None, "Lost the connection", hud::CAPTION, 3.0);
}

/// Says which dialog is up, and so whether one is.
fn show(showing: &mut ResMut<Showing>, up: &mut ResMut<DialogUp>, what: Showing) {
    showing.set_if_neq(what);
    up.set_if_neq(DialogUp(what != Showing::Nothing));
}

fn take_down(dialogs: &Query<Entity, With<Dialog>>, commands: &mut Commands) {
    for dialog in dialogs {
        commands.entity(dialog).despawn();
    }
}

/// Everything a game put in the world or on the screen. Quietly, as the theme
/// may be fading out of its own accord this same frame.
fn clear_away(in_game: &Query<Entity, With<InGame>>, commands: &mut Commands) {
    for thing in in_game {
        commands.entity(thing).try_despawn();
    }
}

// ---------------------------------------------------------------- the game

/// A new game, with you in `seat` — offline the first, with the other seven
/// still to come — a plot for each builder, and the board and the stars on the
/// screen. Online, who else is in it comes from the server.
#[allow(clippy::too_many_arguments)]
fn start(
    commands: &mut Commands,
    assets: &AssetServer,
    water: &Water,
    star: &StarImage,
    you: Entity,
    seat: usize,
    online: bool,
) -> Game {
    for plot in 0..SEATS {
        for part in island::spawn_island(commands, assets, water, Venue::Plot(plot as u8)) {
            commands.entity(part).insert(InGame);
        }
    }
    spawn_board(commands);
    spawn_rating(commands, star);
    Game {
        seats: if online { Vec::new() } else { vec![lobby::me()] },
        bodies: vec![you],
        me: seat,
        online,
        cut_off: None,
        stage: Stage::Queue { waited: 0.0 },
        theme: THEMES[fastrand::usize(..THEMES.len())],
        stars: [0; SEATS],
        mine: None,
    }
}

/// Moves the game on: fills the room, sends everyone to their plots, round the
/// houses, and to the winner's; and takes the stars you give. Online all of
/// that is the server's, and the stars you give, and a tap on the time, go to
/// it.
#[allow(clippy::too_many_arguments)]
fn play(
    time: Res<Time>,
    mut asks: MessageReader<Ask>,
    game: Option<ResMut<Game>>,
    lobby: Res<Lobby>,
    me: Query<&Balance, With<Player>>,
    themes: Query<Entity, With<ThemeHeadline>>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut showing: ResMut<Showing>,
    mut up: ResMut<DialogUp>,
    assets: Res<AssetServer>,
    mut requests: MessageWriter<roundtown_net::Ask>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        asks.clear();
        return;
    };
    let game = &mut *game;
    if game.online {
        for &ask in asks.read() {
            match ask {
                Ask::Skip => {
                    requests.write(roundtown_net::Ask::Skip);
                }
                Ask::Rate(stars) if game.rating() => {
                    let stars = stars.clamp(1, MOST_STARS);
                    game.mine = Some(stars);
                    if let Stage::Visit { plot, .. } = game.stage {
                        requests.write(roundtown_net::Ask::Rate {
                            plot: plot as u8,
                            stars,
                        });
                    }
                }
                _ => {}
            }
        }
        return;
    }
    let mut skip = false;
    for &ask in asks.read() {
        match ask {
            Ask::Skip => skip = true,
            Ask::Rate(stars) if game.rating() => game.mine = Some(stars.clamp(1, MOST_STARS)),
            _ => {}
        }
    }

    let dt = time.delta_secs();
    match game.stage {
        Stage::Queue { waited } => {
            // Nobody else is coming. The computer takes the seats left one at
            // a time, the last as the wait runs out, and pays its way in like
            // anyone.
            let waited = waited + dt;
            let every = QUEUE_FILL / (SEATS - 1) as f32;
            while game.bodies.len() < SEATS && waited >= game.bodies.len() as f32 * every {
                lobby::seat_ai(&mut game.seats, &lobby.seats);
                let seat = game.bodies.len();
                let body = lobby::spawn_player(
                    &mut commands,
                    &assets,
                    &game.seats[seat],
                    seat,
                    Venue::Lobby,
                    false,
                );
                commands
                    .entity(body)
                    .insert((InGame, Balance(STARTING_BALANCE - ENTRY)));
                game.bodies.push(body);
            }
            game.stage = if game.bodies.len() < SEATS {
                Stage::Queue { waited }
            } else {
                Stage::Full { left: FULL_HOLD }
            };
        }
        Stage::Full { left } if left > dt => game.stage = Stage::Full { left: left - dt },
        Stage::Full { .. } => {
            for (seat, &body) in game.bodies.iter().enumerate() {
                lobby::send(&mut bodies, &mut orbit, body, Venue::Plot(seat as u8), seat);
            }
            let (caption, theme, size) = game.theme.headline();
            hud::announce(&mut commands, Some(caption), &theme, size, THEME_SECS)
                .insert((InGame, ThemeHeadline));
            game.stage = Stage::Build { left: BUILD_SECS };
        }
        Stage::Build { left } if left > dt && !skip => {
            game.stage = Stage::Build { left: left - dt };
        }
        Stage::Build { .. } => {
            // Quietly: it may be fading out of its own accord this frame.
            for theme in &themes {
                commands.entity(theme).try_despawn();
            }
            visit(game, 0, &mut bodies, &mut orbit);
        }
        Stage::Visit { plot, left } if left > dt && !skip => {
            game.stage = Stage::Visit {
                plot,
                left: left - dt,
            };
        }
        Stage::Visit { plot, .. } => {
            tally(game, plot);
            if plot + 1 < game.bodies.len() {
                visit(game, plot + 1, &mut bodies, &mut orbit);
            } else {
                let winner = winner(&game.stars);
                for (seat, &body) in game.bodies.iter().enumerate() {
                    lobby::send(
                        &mut bodies,
                        &mut orbit,
                        body,
                        Venue::Plot(winner as u8),
                        seat,
                    );
                }
                game.stage = Stage::Over { winner };
                let affordable = me.single().is_ok_and(|balance| balance.0 >= ENTRY);
                results(&mut commands, &assets, game, winner, affordable);
                show(&mut showing, &mut up, Showing::Results);
            }
        }
        Stage::Over { .. } => {}
    }
}

/// Says whether you are building, and on which plot, for the shop and for
/// putting pieces down.
fn publish_building(
    game: Option<Res<Game>>,
    mut building: ResMut<Building>,
    mut online_plot: ResMut<OnlinePlot>,
    mut offline: ResMut<OfflineGame>,
) {
    let plot = game
        .as_ref()
        .filter(|game| matches!(game.stage, Stage::Build { .. }))
        .map(|game| game.me as u8);
    building.set_if_neq(Building(plot));
    // While the connection is gone, what is put down waits for it.
    online_plot.set_if_neq(OnlinePlot(
        game.as_ref()
            .filter(|game| game.online && game.cut_off.is_none())
            .map(|game| game.me as u8),
    ));
    offline.set_if_neq(OfflineGame(game.is_some_and(|game| !game.online)));
}

/// Sees you from behind while you build and while you rate, as the town is
/// seen, unless you would rather look through your own eyes, so that looking
/// round a house is looking round a room: the button over the hammer swaps the
/// one for the other. The lobby and the winner's plot are always seen from
/// behind you.
fn publish_view(
    mut asks: MessageReader<Ask>,
    game: Option<Res<Game>>,
    mut behind: ResMut<FromBehind>,
    mut view: ResMut<FirstPerson>,
) {
    let up_close = game.is_some_and(|game| game.up_close());
    for &ask in asks.read() {
        if ask == Ask::SwapView && up_close {
            behind.0 = !behind.0;
        }
    }
    view.set_if_neq(FirstPerson(up_close && !behind.0));
}

/// Everyone to the house on `plot`, to rate it.
fn visit(game: &mut Game, plot: usize, bodies: &mut Whereabouts, orbit: &mut OrbitCamera) {
    for (seat, &body) in game.bodies.iter().enumerate() {
        lobby::send(bodies, orbit, body, Venue::Plot(plot as u8), seat);
    }
    game.mine = None;
    game.stage = Stage::Visit {
        plot,
        left: VISIT_SECS,
    };
}

/// Adds up the stars the house on `plot` was given: yours, if you gave any,
/// and the computer's players', at random. Nobody rates their own house.
fn tally(game: &mut Game, plot: usize) {
    let mine = game.mine.take();
    for (seat, member) in game.seats.iter().enumerate() {
        let stars = if seat == plot {
            0
        } else if seat == game.me {
            mine.unwrap_or(0)
        } else if member.ai {
            fastrand::u8(1..=MOST_STARS)
        } else {
            0
        };
        game.stars[plot] += u32::from(stars);
    }
}

/// The house with the most stars, a tie settled by lot.
fn winner(stars: &[u32]) -> usize {
    let most = stars.iter().copied().max().unwrap_or(0);
    let best: Vec<usize> = (0..stars.len())
        .filter(|&plot| stars[plot] == most)
        .collect();
    best[fastrand::usize(..best.len())]
}

// ------------------------------------------------------------- the bubble

/// The speech bubble, put away until you are near the House Builder. It is
/// laid out top left and moved over their head without being laid out again.
fn spawn_bubble(mut commands: Commands) {
    commands.spawn((
        Bubble,
        AsksFor(Ask::Talk),
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::VMin(0.4),
            padding: UiRect::axes(Val::VMin(3.2), Val::VMin(1.8)),
            border_radius: BorderRadius::all(Val::VMin(3.0)),
            ..default()
        },
        BackgroundColor(BUBBLE),
        UiTransform::default(),
        Visibility::Hidden,
        children![
            // The point under the bubble, at the House Builder: a square on
            // its corner, the top half of it lost in the bubble.
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::VMin(-TAIL * 0.5),
                    width: Val::VMin(TAIL),
                    height: Val::VMin(TAIL),
                    ..default()
                },
                BackgroundColor(BUBBLE),
                UiTransform {
                    translation: Val2::percent(-50.0, 0.0),
                    rotation: Rot2::degrees(45.0),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            label(SAY, SAY_TEXT, DOOR),
            label(HINT, HINT_TEXT, HINT_COLOUR),
        ],
    ));
}

/// Keeps the bubble over the House Builder's head wherever the camera goes,
/// for as long as you can talk to them.
fn place_bubble(
    talk: Res<Talk>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    mut bubbles: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), With<Bubble>>,
) {
    let Ok((mut shown, mut place, node)) = bubbles.single_mut() else {
        return;
    };
    let on_screen = talk.over.and_then(|over| {
        let (camera, eye) = cameras.single().ok()?;
        // The camera has no parent, so its transform is its global one, and
        // `follow_camera` has just set it for this frame.
        camera
            .world_to_viewport(&GlobalTransform::from(*eye), over)
            .ok()
    });
    let (Some(point), Ok(window)) = (on_screen, windows.single()) else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    // With its point there: centred over it, and standing on it. But never
    // off the screen, nor up among the balance and the button, so that it can
    // always be tapped: with the camera pulled in close, the House Builder's
    // head can be above the top of the screen.
    let size = node.size() * node.inverse_scale_factor();
    let vmin = window.width().min(window.height()) / 100.0;
    let lowest = Vec2::new(hud::EDGE, BUBBLE_TOP) * vmin;
    let highest = (window.size() - size - hud::EDGE * vmin).max(lowest);
    let corner = Vec2::new(point.x - size.x * 0.5, point.y - size.y).clamp(lowest, highest);
    place.set_if_neq(UiTransform::from_translation(Val2::px(corner.x, corner.y)));
    shown.set_if_neq(Visibility::Inherited);
}

// ------------------------------------------------------ the board, the stars

/// The board top centre: how many are in the room, then how long is left,
/// with a line under it saying what for.
fn spawn_board(commands: &mut Commands) {
    commands
        .spawn((
            InGame,
            Board,
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(hud::EDGE),
                left: Val::ZERO,
                right: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(1.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|board| {
            // Tapping the time left ends the wait, for trying the game out.
            board.spawn((
                AsksFor(Ask::Skip),
                TakesPress,
                Button,
                Node {
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(0.8)),
                    border_radius: BorderRadius::all(Val::VMin(3.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                Fill {
                    idle: PANEL,
                    lit: PANEL_LIT,
                },
                children![(BoardText, hud::words("", BOARD_TEXT, Color::WHITE))],
            ));
            board.spawn((BoardCaption, hud::words("", CAPTION, Color::WHITE)));
        });
}

/// Writes the count or the time on the board, and what it is for.
fn show_board(
    game: Option<Res<Game>>,
    mut boards: Query<&mut Visibility, With<Board>>,
    mut texts: Query<&mut Text, With<BoardText>>,
    mut captions: Query<(&mut Text, &mut TextFont), (With<BoardCaption>, Without<BoardText>)>,
) {
    let Some(game) = game else {
        return;
    };
    let players = |count: usize| format!("{count}/{SEATS} Players");
    let (big, small, size) = match game.stage {
        Stage::Queue { .. } => (
            players(game.seats.len()),
            "Waiting for players...".into(),
            CAPTION,
        ),
        Stage::Full { .. } => (players(game.seats.len()), "Match found!".into(), CAPTION),
        Stage::Build { left } => {
            let (theme, size) = game.theme.caption();
            (clock(left), theme, size)
        }
        Stage::Visit { plot, left } => (
            clock(left),
            format!(
                "{}, {} of {}",
                game.whose(plot),
                plot + 1,
                game.seats.len()
            ),
            CAPTION,
        ),
        // The results are up, over everything.
        Stage::Over { .. } => {
            for mut shown in &mut boards {
                shown.set_if_neq(Visibility::Hidden);
            }
            return;
        }
    };
    for mut shown in &mut boards {
        shown.set_if_neq(Visibility::Inherited);
    }
    for mut text in &mut texts {
        if text.0 != big {
            text.0.clone_from(&big);
        }
    }
    let size = FontSize::VMin(size);
    for (mut text, mut font) in &mut captions {
        if text.0 != small {
            text.0.clone_from(&small);
        }
        if font.font_size != size {
            font.font_size = size;
        }
    }
}

/// `4:59`: seconds left, rounded up, so that the last second reads `0:01`.
fn clock(left: f32) -> String {
    let seconds = left.max(0.0).ceil() as u32;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// The stars along the bottom of the screen, for rating the house being
/// visited.
fn spawn_rating(commands: &mut Commands, star: &StarImage) {
    commands
        .spawn((
            InGame,
            Rating,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::VMin(STARS_UP),
                left: Val::ZERO,
                right: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(1.2),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|rating| {
            rating.spawn((RatingCaption, hud::words("", CAPTION, Color::WHITE)));
            rating
                .spawn((
                    StarRow,
                    Node {
                        column_gap: Val::VMin(STAR_GAP),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    for stars in 1..=MOST_STARS {
                        row.spawn((
                            StarButton(stars),
                            AsksFor(Ask::Rate(stars)),
                            TakesPress,
                            Button,
                            Node {
                                width: Val::VMin(STAR),
                                height: Val::VMin(STAR),
                                ..default()
                            },
                            ImageNode::new(star.0.clone()).with_color(STAR_OFF),
                        ));
                    }
                });
        });
}

/// Shows the stars while a house is visited, lit up to the number you gave
/// it. At your own house there is nothing for you to give.
fn show_rating(
    game: Option<Res<Game>>,
    mut ratings: Query<&mut Visibility, (With<Rating>, Without<StarRow>)>,
    mut rows: Query<&mut Visibility, (With<StarRow>, Without<Rating>)>,
    mut captions: Query<&mut Text, With<RatingCaption>>,
    mut stars: Query<(&StarButton, &mut ImageNode)>,
) {
    let Some(game) = game else {
        return;
    };
    let visiting = match game.stage {
        Stage::Visit { plot, .. } => Some(plot),
        _ => None,
    };
    for mut shown in &mut ratings {
        shown.set_if_neq(if visiting.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let Some(plot) = visiting else {
        return;
    };
    let yours = plot == game.me;
    for mut shown in &mut rows {
        shown.set_if_neq(if yours {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    let caption = if yours {
        "Everyone is rating your house"
    } else {
        "Rate this house"
    };
    for mut text in &mut captions {
        if text.0 != caption {
            caption.clone_into(&mut text.0);
        }
    }
    let given = game.mine.unwrap_or(0);
    for (star, mut image) in &mut stars {
        let colour = if star.0 <= given { STAR_ON } else { STAR_OFF };
        if image.color != colour {
            image.color = colour;
        }
    }
}

/// Lights a button up while it is under the pointer or held.
fn light_buttons(
    mut buttons: Query<(&Interaction, &Fill, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, fill, mut colour) in &mut buttons {
        colour.set_if_neq(BackgroundColor(match interaction {
            Interaction::None => fill.idle,
            Interaction::Hovered | Interaction::Pressed => fill.lit,
        }));
    }
}

// --------------------------------------------------------------- the view

/// The button over the hammer that swaps seeing through your own eyes for
/// seeing yourself from behind, and back. Like the button that goes home, it
/// shows where it takes you: a person, to see yourself, or an eye, to see
/// through your own. Put away until it is wanted ([`show_view_button`]).
fn spawn_view_button(mut commands: Commands) {
    commands.spawn((
        ViewButton,
        AsksFor(Ask::SwapView),
        TakesPress,
        Button,
        // Over the hammer, as big and in line with it, and as high up as the
        // button that goes home is in the town.
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(hud::EDGE),
            right: Val::VMin(shop::HAMMER_RIGHT),
            width: Val::VMin(shop::HAMMER),
            height: Val::VMin(shop::HAMMER),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        Fill {
            idle: PANEL,
            lit: PANEL_LIT,
        },
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        Visibility::Hidden,
        children![
            (
                ViewPicture::Behind,
                hud::picture(),
                person(),
                Visibility::Hidden
            ),
            (ViewPicture::Eyes, hud::picture(), eye()),
        ],
    ));
}

/// The button is there while you build and while you rate, unless a dialog or
/// the shop is up, with the picture of the view it swaps to.
fn show_view_button(
    game: Option<Res<Game>>,
    up: Res<DialogUp>,
    behind: Res<FromBehind>,
    mut buttons: Query<&mut Visibility, (With<ViewButton>, Without<ViewPicture>)>,
    mut pictures: Query<(&ViewPicture, &mut Visibility), Without<ViewButton>>,
) {
    let shown = !up.0 && game.is_some_and(|game| game.up_close());
    for mut visibility in &mut buttons {
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let next = if behind.0 {
        ViewPicture::Eyes
    } else {
        ViewPicture::Behind
    };
    for (picture, mut visibility) in &mut pictures {
        visibility.set_if_neq(if *picture == next {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// A person, head and shoulders: you, the way you are seen from behind.
pub(crate) fn person() -> impl Bundle {
    children![
        hud::disc(37.0, 16.0, 26.0),
        (
            Node {
                border_radius: BorderRadius::top(Val::Percent(50.0)),
                ..hud::frame(23.0, 47.0, 54.0, 33.0)
            },
            BackgroundColor(hud::ICON),
            Pickable::IGNORE,
        ),
    ]
}

/// An eye: the white of it, the iris and a glint. The white is a square on its
/// corner, rounded off and squashed flat.
fn eye() -> impl Bundle {
    children![
        (
            hud::picture(),
            UiTransform::from_scale(Vec2::new(1.0, 0.56)),
            children![(
                Node {
                    border_radius: BorderRadius::all(Val::Percent(22.0)),
                    ..hud::frame(26.0, 26.0, 48.0, 48.0)
                },
                BackgroundColor(hud::ICON),
                UiTransform::from_rotation(Rot2::degrees(45.0)),
                Pickable::IGNORE,
            )],
        ),
        (
            Node {
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..hud::frame(37.0, 37.0, 26.0, 26.0)
            },
            BackgroundColor(DOOR),
            Pickable::IGNORE,
        ),
        hud::disc(52.0, 40.0, 8.0),
    ]
}

// ---------------------------------------------------------------- dialogs

/// Whether to play: what the game is, what it costs, and Cancel or Play.
fn offer(commands: &mut Commands, assets: &AssetServer, affordable: bool) {
    // A tap on the dark round it is a no.
    dialog(commands, Some(Ask::Cancel)).with_children(|panel| {
        panel.spawn(hud::words(NAME, TITLE, GOLD));
        panel.spawn(label(RULES, BODY, Color::WHITE));
        panel.spawn(price(assets, "to play"));
        if !affordable {
            panel.spawn(label(
                format!("You need {ENTRY} coins to play."),
                SMALL,
                WARN,
            ));
        }
        panel.spawn(choices()).with_children(|choices| {
            choices.spawn(button("Cancel", Ask::Cancel, true));
            choices.spawn(button("Play", Ask::Join, affordable));
        });
    });
}

/// Who won and with how many stars, how you did, and Exit or Play again.
fn results(
    commands: &mut Commands,
    assets: &AssetServer,
    game: &Game,
    winner: usize,
    affordable: bool,
) {
    let won = winner == game.me;
    let yours = game.stars[game.me];
    let place = 1 + game.stars.iter().filter(|&&stars| stars > yours).count();
    let tied = (0..game.seats.len()).any(|plot| plot != game.me && game.stars[plot] == yours);
    dialog(commands, None).with_children(|panel| {
        panel.spawn(hud::words(
            if won { "You won!" } else { "Winner!" },
            TITLE,
            GOLD,
        ));
        panel.spawn(label(
            format!("{}, with {}", game.whose(winner), stars(game.stars[winner])),
            BODY,
            Color::WHITE,
        ));
        if !won {
            let came = if tied { "tied for" } else { "came" };
            panel.spawn(label(
                format!("You {came} {}, with {}", ordinal(place), stars(yours)),
                SMALL,
                SOFT,
            ));
        }
        panel.spawn(price(assets, "to play again"));
        if !affordable {
            panel.spawn(label(
                format!("You need {ENTRY} coins to play again."),
                SMALL,
                WARN,
            ));
        }
        panel.spawn(choices()).with_children(|choices| {
            choices.spawn(button("Exit", Ask::Leave, true));
            choices.spawn(button("Play again", Ask::Join, affordable));
        });
    });
}

/// One of House Builder's dialogs: the screen dimmed behind a panel, over
/// everything else ([`dimmed`]). A tap on the dark, if it does anything, asks
/// `on_dark`.
fn dialog<'a>(commands: &'a mut Commands, on_dark: Option<Ask>) -> EntityCommands<'a> {
    match on_dark {
        Some(ask) => dimmed(commands, (Dialog, AsksFor(ask))),
        None => dimmed(commands, Dialog),
    }
}

/// The screen dimmed behind a panel, over everything else, and taking every
/// press there is while it is up: every dialog in the game looks like this.
/// The dark carries `dark`, which says whose dialog it is. Returns the panel,
/// to be filled in.
pub(crate) fn dimmed<'a>(commands: &'a mut Commands, dark: impl Bundle) -> EntityCommands<'a> {
    let dark = commands
        .spawn((
            dark,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(DIM),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    commands.spawn((
        ChildOf(dark),
        Node {
            width: Val::VMin(DIALOG_WIDTH),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::VMin(2.4),
            padding: UiRect::all(Val::VMin(4.5)),
            border_radius: BorderRadius::all(Val::VMin(4.0)),
            ..default()
        },
        BackgroundColor(DIALOG),
    ))
}

/// The coin, the entry fee, and what it is for.
fn price(assets: &AssetServer, what: &str) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.2),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                ImageNode::new(assets.load(hud::COIN_ICON)),
                Node {
                    width: Val::VMin(PRICE_ICON),
                    height: Val::VMin(PRICE_ICON),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            hud::words(hud::money(ENTRY), PRICE_TEXT, GOLD),
            label(what, BODY, Color::WHITE),
        ],
    )
}

/// The row the buttons at the foot of a dialog stand in.
pub(crate) fn choices() -> impl Bundle {
    (
        Node {
            column_gap: Val::VMin(4.0),
            margin: UiRect::top(Val::VMin(0.8)),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// One of a dialog's two buttons: the quiet one that backs out, or, when
/// `able`, the gold one that goes ahead — which is greyed out, and does
/// nothing, while it cannot.
fn button(text: &str, asks: Ask, able: bool) -> impl Bundle {
    let ahead = !matches!(asks, Ask::Cancel | Ask::Leave);
    let (fill, ink) = match (ahead, able) {
        (false, _) => (
            Fill {
                idle: QUIET,
                lit: QUIET_LIT,
            },
            Color::WHITE,
        ),
        (true, true) => (
            Fill {
                idle: GOLD,
                lit: GOLD_LIT,
            },
            DOOR,
        ),
        (true, false) => (
            Fill {
                idle: OFF,
                lit: OFF,
            },
            OFF_TEXT,
        ),
    };
    (
        AsksFor(asks),
        TakesPress,
        Button,
        Node {
            width: Val::VMin(BUTTON_WIDTH),
            height: Val::VMin(BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(BUTTON_HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![label(text, BUTTON_TEXT, ink)],
    )
}

/// Plain words, for a button, the bubble or a dialog, which have a colour of
/// their own behind them and need no shadow to read.
pub(crate) fn label(text: impl Into<String>, size_vmin: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::VMin(size_vmin),
            ..default()
        },
        TextColor(colour),
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    )
}

/// `1 star`, `28 stars`.
fn stars(count: u32) -> String {
    if count == 1 {
        "1 star".into()
    } else {
        format!("{count} stars")
    }
}

/// `1st`, `2nd`, `3rd`, `4th`: only ever up to eighth.
fn ordinal(place: usize) -> String {
    let suffix = match place {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{place}{suffix}")
}

// ---------------------------------------------------------------- the star

fn draw_star(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    commands.insert_resource(StarImage(images.add(star_image())));
}

/// A five-pointed star, white on clear, for the rating buttons to tint: gold
/// for the stars given, pale for the rest. Drawn here because the built-in
/// font has no star in it, and edged by how far each pixel is from it, so that
/// it is smooth at whatever size a screen shows it.
fn star_image() -> Image {
    const SIZE: u32 = 192;
    /// How far the points reach from the middle, as a share of half the image.
    const REACH: f32 = 0.96;
    /// How far in the notches between the points come, as a share of the
    /// points' reach: fuller than a drawn star's 0.38, to suit the round town.
    const NOTCH: f32 = 0.5;
    /// How much every corner is rounded off, as a share of the reach.
    const ROUND: f32 = 0.1;
    let half = SIZE as f32 * 0.5;
    let reach = half * REACH;
    let round = reach * ROUND;
    // The points below reach less far down than the one on top reaches up, so
    // the star is lowered by half the difference to sit in the middle.
    let lower = reach * (1.0 - 36f32.to_radians().cos()) * 0.5;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            // From the middle, in pixels, with y up.
            let at = Vec2::new(
                column as f32 + 0.5 - half,
                half - (row as f32 + 0.5) + lower,
            );
            let outside = star_distance(at, reach - round, NOTCH) - round;
            let cover = (0.5 - outside).clamp(0.0, 1.0);
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

/// How far `at` is from the edge of a five-pointed star round the origin, one
/// point straight up, whose points reach `reach` out and whose notches come
/// `notch` of the way in: less than nothing inside it. Inigo Quilez's
/// `sdStar5`.
fn star_distance(at: Vec2, reach: f32, notch: f32) -> f32 {
    const K1: Vec2 = Vec2::new(0.809_017, -0.587_785);
    const K2: Vec2 = Vec2::new(-0.809_017, -0.587_785);
    let mut p = Vec2::new(at.x.abs(), at.y);
    p -= 2.0 * K1.dot(p).max(0.0) * K1;
    p -= 2.0 * K2.dot(p).max(0.0) * K2;
    p.x = p.x.abs();
    p.y -= reach;
    let edge = notch * Vec2::new(-K1.y, K1.x) - Vec2::Y;
    let along = (p.dot(edge) / edge.dot(edge)).clamp(0.0, reach);
    (p - edge * along).length() * (p.y * edge.x - p.x * edge.y).signum()
}
