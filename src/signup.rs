//! Signing up: the pages the app opens on the first time it is run, before
//! the town is seen (Hajun, 2026-10-31): the town's name and Start; a name
//! to go by, typed in; and a character, a boy or a girl. Kept on this device
//! as `profile.json`, beside the town save, so that the pages never come
//! again; and sent to the server, which keeps them with the account (`net`,
//! `Ask::Profile`), so that everyone online sees you by them. Deleting the
//! account (`settings`) brings the pages back, and the name can be changed
//! there ([`NameBox`], [`wear`]).
//!
//! The pages are black, with white buttons, and as little as will do (Hajun:
//! "as minimalistic as possible ... no outlines or any unnecessary text";
//! and that evening "black for the initial pages ... white instead of
//! yellow for the buttons"): a heading, what has to be said, and a button.
//! "Log In", under Start, is for an account made on another device, which
//! is not to be had yet, and says so when pressed.
//!
//! Typing the name: on desktop the keys, and whatever its input method
//! composes; on a phone the soft keyboard, which the window's `ime_enabled`
//! asks for. An iPhone's sends each letter as a key press, and Return as a
//! newline. Android's sends no keys at all and writes into a text of its
//! own, which the name is read off every frame ([`android_text`]). What is
//! kept is what the game writes (`roundtown_net::text`): English, Korean
//! and Vietnamese since 2026-11-14 (Hajun: "i need korean text to be able
//! to work in the global chat not just english. same for username and send
//! message", "also vietnamese text as well"), plain ASCII until then. A
//! phone lying on its side has its keyboard over most of the screen, so the
//! name's page, and the settings' Edit name, slide up out from under it
//! ([`Keyboard`], [`LiftsForKeyboard`]).

#[cfg(not(target_os = "android"))]
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
#[cfg(not(target_os = "android"))]
use bevy::window::{Ime, WindowEvent};
use roundtown_net::text as letters;
use roundtown_net::{Ask, BOY_LOOKS, GIRL_LOOKS, Member, PEOPLE_LOOKS, profile};
use serde::{Deserialize, Serialize};

use crate::Player;
use crate::builder::{self, Fill};
use crate::hud::{self, DialogUp, TakesPress};
use crate::lobby::{self, Lobby};
use crate::ui::{UiChanged, ui};

/// The file who you are lives in, beside the town save.
const FILE: &str = "profile.json";
/// Bumped if the file's shape changes.
const VERSION: u32 = 1;

// The pages' colours and words are `assets/ui.ron`'s (`ui`): the page,
// black, and what stands on it in greys, the box the name is typed into and
// the two characters' cards, one of them chosen, lighter; and the one
// button each page has, white with black words, or grey while it cannot be
// pressed (Hajun, 2026-10-31: "black for the initial pages ... and white
// instead of yellow for the buttons"; a dark blue page with blue boxes and
// gold buttons for the afternoon).

/// A white button with black words: the pages' own.
fn white_button() -> (Fill, Color) {
    let colours = ui().colours.signup;
    (
        Fill {
            idle: colours.paper,
            lit: colours.paper_lit,
        },
        colours.ink,
    )
}

/// A gold button with the dark blue of words on gold: the settings' Save, in
/// the look of the town's dialogs.
pub(crate) fn gold_button() -> (Fill, Color) {
    let colours = ui().colours;
    (
        Fill {
            idle: colours.gold,
            lit: colours.gold_lit,
        },
        colours.on_gold,
    )
}

/// Sizes, as shares of the short side of the screen: the town's name, a
/// page's heading, its words, its button and the name as typed.
const TITLE: f32 = 8.5;
const HEADING: f32 = 6.5;
const SMALL: f32 = 3.5;
const BUTTON_TEXT: f32 = 4.6;
const NAME_TEXT: f32 = 5.0;
const BUTTON_WIDTH: f32 = 52.0;
const BUTTON_HEIGHT: f32 = 12.0;
/// The box the name is typed into: as wide as the words over it, and the
/// cards: a picture 2 by 3 with the character's name under it.
const FIELD_WIDTH: f32 = 74.0;
const CARD_WIDTH: f32 = 34.0;
const PICTURE_WIDTH: f32 = 26.0;
const ROUNDED: f32 = 1.8;
/// The arrows either side of a card that turn its colours (Hajun,
/// 2026-11-02): the arrowhead, the room round it a finger lands in, and the
/// gap between it and the card. A row of both cards and their arrows is 108
/// shares of the short side wide.
const ARROW_WIDTH: f32 = 4.0;
const ARROW_TOUCH: f32 = 8.0;
const ARROW_GAP: f32 = 1.0;

/// How far down the screen things stand, as shares of its height: the town's
/// name and Start under it, in the lower middle; and a page's heading, high
/// enough that a phone's keyboard, over the lower half, covers nothing.
const TITLE_DOWN: f32 = 22.0;
const START_DOWN: f32 = 60.0;
const HEADING_DOWN: f32 = 9.0;

/// Over everything else on the screen.
const LAYER: i32 = 30;

/// How long "Log in is not available yet" stays up, in seconds; and how long
/// the caret is on and off for.
const NOTICE_SECS: f32 = 3.0;
const BLINK_SECS: f32 = 0.5;

/// Who you are, as you signed up: the name you go by, and which of
/// [`PEOPLE_LOOKS`] you wear. Both empty until you have.
#[derive(Resource, Clone, Default, PartialEq, Eq, Debug)]
pub(crate) struct Profile {
    pub name: String,
    pub look: String,
}

/// The file as written.
#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    name: String,
    look: String,
}

impl Profile {
    /// Whether you have signed up: a name and a look, both good.
    pub(crate) fn set(&self) -> bool {
        profile::name(&self.name).is_some_and(|name| name == self.name) && profile::look(&self.look)
    }

    /// What was kept from the last run, if it is whole.
    fn load() -> Self {
        let Some(path) = crate::map::data_file(FILE) else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let saved: Saved = match serde_json::from_str(&text) {
            Ok(saved) => saved,
            Err(error) => {
                warn!("{} is not a profile ({error}): signing up again", path.display());
                return Self::default();
            }
        };
        let profile = Self {
            name: saved.name,
            look: saved.look,
        };
        if saved.version != VERSION || !profile.set() {
            warn!("{} is not a whole profile: signing up again", path.display());
            return Self::default();
        }
        profile
    }

    /// Keeps it for the next run: written beside the file and renamed over
    /// it, so that being killed halfway through leaves nothing half written.
    pub(crate) fn save(&self) {
        let Some(path) = crate::map::data_file(FILE) else {
            warn!("nowhere to keep the profile: the sign-up will come again");
            return;
        };
        let saved = Saved {
            version: VERSION,
            name: self.name.clone(),
            look: self.look.clone(),
        };
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| serde_json::to_string_pretty(&saved).map_err(std::io::Error::other))
            .and_then(|text| {
                let partial = path.with_extension("json.new");
                std::fs::write(&partial, text)?;
                std::fs::rename(&partial, &path)
            });
        if let Err(error) = written {
            warn!("could not keep the profile in {}: {error}", path.display());
        }
    }

    /// Forgets who you were: the account is deleted.
    pub(crate) fn forget(&mut self) {
        *self = Self::default();
        if let Some(path) = crate::map::data_file(FILE) {
            let _ = std::fs::remove_file(path);
        }
    }

    /// What to tell the server, if it has you, as `me`, otherwise than you
    /// signed up.
    pub(crate) fn ask_for(&self, me: &Member) -> Option<Ask> {
        (self.set() && (me.name != self.name || me.look != self.look)).then(|| Ask::Profile {
            name: self.name.clone(),
            look: self.look.clone(),
        })
    }
}

/// Takes on who you are, as `profile` has it, for the rest of the town to
/// know you by: in the town's room, so that a game of House Builder offline
/// is played under it, and in the flesh, your name over your head and the
/// model you are drawn with. Online, `net` sees the profile change and tells
/// the server.
pub(crate) fn wear(
    profile: &Profile,
    lobby: &mut Lobby,
    me: &mut Query<(&mut Name, &mut WorldAssetRoot), With<Player>>,
    assets: &AssetServer,
) {
    lobby.sign_up(profile);
    if let Ok((mut name, mut body)) = me.single_mut() {
        name.set_if_neq(Name::new(profile.name.clone()));
        body.set_if_neq(lobby::dressed_as(assets, &profile.look));
    }
}

/// The pages, while they are up, and what has been done on them so far.
#[derive(Resource, Default)]
pub(crate) struct Signup {
    page: Option<Page>,
    /// The name, once its page has been saved.
    name: String,
    /// The side chosen, by its place in [`PEOPLE_LOOKS`]: the boy or the
    /// girl.
    look: Option<usize>,
    /// Which colour each side's card shows, by its place in that side's
    /// looks ([`side_looks`]), turned by the arrows beside it.
    colours: [usize; 2],
    /// The arrowhead the arrows are drawn with ([`hud::side_arrow`]), made
    /// once as the app starts ([`make_arrow`]).
    arrow: Handle<Image>,
    /// A button pressed, to be acted on this frame.
    pressed: Option<Action>,
    /// How long "Log in is not available yet" has been up, while it is.
    notice: Option<f32>,
}

impl Signup {
    /// Whether the pages are up.
    pub(crate) fn up(&self) -> bool {
        self.page.is_some()
    }
}

/// A box a name is typed into, with the caret blinking after it: on the
/// sign-up's page, and in the settings' Edit name; and, since 2026-11-13,
/// whatever else is typed, a name to look for and a message to a friend
/// (`friends`), which are the same letters and only longer
/// ([`NameBox::open_for`]). At most one is up at a time, and while it is,
/// it takes every key and asks a phone for its keyboard. [`name_box`] is
/// the box itself, and [`field`] one of any size.
#[derive(Resource)]
pub(crate) struct NameBox {
    up: bool,
    /// What has been typed so far, as the keyboard sent it: a letter may
    /// still be in its pieces (`roundtown_net::text::piece`).
    text: String,
    /// What a desktop's input method is in the middle of composing, a
    /// Hangul syllable not yet done: shown after what is typed, and not
    /// yet part of it.
    composing: String,
    /// The most it takes, in widths (`roundtown_net::text::length`: a
    /// Hangul syllable is two).
    most: usize,
    /// Seconds the caret has blinked for.
    blink: f32,
    /// Enter was pressed, or typed, and not yet taken.
    entered: bool,
    /// The soft keyboard is to be asked for again next frame: the box was
    /// tapped, on a phone where the keyboard may have been put away.
    keyboard_again: bool,
}

impl Default for NameBox {
    fn default() -> Self {
        Self {
            up: false,
            text: String::new(),
            composing: String::new(),
            most: profile::LONGEST,
            blink: 0.0,
            entered: false,
            keyboard_again: false,
        }
    }
}

impl NameBox {
    /// A box for a name has come up, with `text` in it to go on from.
    pub(crate) fn open(&mut self, text: &str) {
        self.open_for(text, profile::LONGEST);
    }

    /// A box has come up, with `text` in it to go on from, which takes up
    /// to `most` widths of what the game writes.
    pub(crate) fn open_for(&mut self, text: &str, most: usize) {
        self.up = true;
        self.text = text.to_string();
        self.composing.clear();
        self.most = most;
        self.blink = 0.0;
        self.entered = false;
        #[cfg(target_os = "android")]
        android_reset(text);
    }

    /// The box has gone.
    pub(crate) fn close(&mut self) {
        self.up = false;
        self.entered = false;
        self.composing.clear();
    }

    /// Whether a box is up.
    pub(crate) fn up(&self) -> bool {
        self.up
    }

    /// What has been typed so far, as the keyboard sent it, without what is
    /// still being composed: for going on from. What is in the box, to
    /// show or to take, is [`shown`](Self::shown).
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// What is in the box: what has been typed, each letter put together
    /// from its pieces, and after it what is being composed, which is
    /// taken with the rest when the box is done with by a button rather
    /// than by Enter (a syllable not yet done is not left behind).
    /// Whatever takes it for a name, a line or a message cleans it
    /// (`roundtown_net::text::clean`).
    pub(crate) fn shown(&self) -> String {
        let mut shown = letters::whole(&self.text);
        shown.push_str(&self.composing);
        shown
    }

    /// Whether nothing has been typed, nor is being composed.
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty() && self.composing.is_empty()
    }

    /// The name as typed, if it is a name.
    pub(crate) fn name(&self) -> Option<String> {
        profile::name(&self.shown())
    }

    /// Whether Enter has been pressed, or typed, since this was last asked.
    pub(crate) fn entered(&mut self) -> bool {
        std::mem::take(&mut self.entered)
    }

    /// The box has been tapped: a phone is asked for its keyboard again,
    /// which brings it back if it was put away ([`keyboard`]).
    pub(crate) fn ask_for_the_keyboard_again(&mut self) {
        if self.up {
            self.keyboard_again = true;
        }
    }

    /// Takes `c` as typed into the box: kept if the game writes it, or it
    /// is a piece of what it writes, and there is room; a newline is Enter.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn typed(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            self.entered = true;
        } else if letters::typed(c) && letters::length(&self.text) + letters::width(c) <= self.most {
            self.text.push(c);
        }
    }

    /// What a desktop's input method is composing now: nothing, once it is
    /// done ([`type_name`]). Shown only as far as the game writes it and
    /// there is room for it after what is typed.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn compose(&mut self, composing: &str) {
        let composing: String = composing.chars().filter(|c| letters::typed(*c)).collect();
        let fits = letters::length(&self.text) + letters::length(&composing) <= self.most;
        let composing = if fits { composing } else { String::new() };
        if self.composing != composing {
            self.composing = composing;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Page {
    /// The town's name, Start, and Log In.
    Start,
    /// The name, typed in.
    Name,
    /// A boy or a girl.
    Character,
}

/// What a press on one of the pages' buttons does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Start,
    LogIn,
    SaveName,
    /// This side's card.
    Pick(usize),
    /// This side's colours turned one on, or one back.
    Turn(usize, i32),
    SaveLook,
}

/// The looks of one side of the character page, the boys' or the girls'
/// (`roundtown_net::BOY_LOOKS`, `GIRL_LOOKS`): the first of each is the one
/// people chose between before the colours came (2026-11-02).
fn side_looks(side: usize) -> &'static [&'static str] {
    if side == 0 {
        &BOY_LOOKS
    } else {
        &GIRL_LOOKS
    }
}

/// The colour `by` on from `colour` of `count`, round and round.
fn turned(colour: usize, by: i32, count: usize) -> usize {
    (colour as i32 + by).rem_euclid(count as i32) as usize
}

/// The root of the page that is up.
#[derive(Component)]
struct PageRoot;

/// The box a name is typed into: tapped for the keyboard.
#[derive(Component)]
struct NameBoxTap;

/// The name as typed, with the caret after it.
#[derive(Component)]
struct NameText;

/// "Log in is not available yet".
#[derive(Component)]
struct Notice;

/// One side's card, by its place in [`PEOPLE_LOOKS`].
#[derive(Component)]
struct Card(usize);

/// The picture on a side's card, of the colour it shows ([`turn_cards`]).
#[derive(Component)]
struct CardPicture {
    side: usize,
    colour: usize,
}

/// An arrow beside a side's card, turning its colours one on (`by` 1) or
/// one back (-1).
#[derive(Component)]
pub(crate) struct ColourArrow {
    pub side: usize,
    pub by: i32,
}

/// The button that saves what a page asks for, or the settings' Edit name:
/// grey until there is something to save ([`light`]).
#[derive(Component)]
pub(crate) struct Save;

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(Profile::load())
        .init_resource::<Signup>()
        .init_resource::<NameBox>()
        .init_resource::<Keyboard>()
        .add_observer(press)
        .add_observer(tap_box)
        .add_systems(Startup, (make_arrow, open_if_unset).chain())
        .add_systems(
            Update,
            (
                #[cfg(target_os = "android")]
                android_text.in_set(Typed),
                #[cfg(not(target_os = "android"))]
                type_name.in_set(Typed),
                redraw,
                act,
                show_name,
                light_save,
                light_cards,
                turn_cards,
                fade_notice,
                keyboard,
                #[cfg(any(target_os = "android", target_os = "ios"))]
                read_keyboard,
                lift_for_keyboard,
            )
                .chain(),
        );
}

/// What is typed being taken into the box that is up ([`NameBox`]), each
/// frame: whatever opens a box on a key runs after this, so that the key
/// that opened it is not typed into it (`chat`).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Typed;

/// The first time the app is run, or the name and the character are not
/// set: the pages come before the town. And on desktop with `RT_SIGNUP` set,
/// every time, for working on the pages: saving on them sets the profile as
/// it would.
fn open_if_unset(
    profile: Res<Profile>,
    mut signup: ResMut<Signup>,
    mut up: ResMut<DialogUp>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let asked_for = cfg!(not(any(target_os = "android", target_os = "ios")))
        && std::env::var_os("RT_SIGNUP").is_some_and(|value| !value.is_empty());
    if !profile.set() || asked_for {
        open(&mut commands, &assets, &mut signup, &mut up);
    }
}

/// Puts the pages up, from the first: over everything, with the world's
/// controls put away under them as under a dialog.
pub(crate) fn open(
    commands: &mut Commands,
    assets: &AssetServer,
    signup: &mut Signup,
    up: &mut DialogUp,
) {
    if signup.up() {
        return;
    }
    *signup = Signup {
        arrow: signup.arrow.clone(),
        ..Signup::default()
    };
    show(commands, assets, signup, &mut NameBox::default(), Page::Start);
    if !up.0 {
        up.0 = true;
    }
}

/// Puts `page` up. Whatever page was up is taken down first by whoever calls
/// this with one up ([`act`]).
fn show(
    commands: &mut Commands,
    assets: &AssetServer,
    signup: &mut Signup,
    name_box: &mut NameBox,
    page: Page,
) {
    signup.page = Some(page);
    if page == Page::Name {
        name_box.open(&signup.name);
    }
    draw(commands, assets, signup, &name_box.text, page);
}

/// Spawns `page` as things stand: with `name` in its box, if it has one.
fn draw(commands: &mut Commands, assets: &AssetServer, signup: &Signup, name: &str, page: Page) {
    match page {
        Page::Start => start_page(commands),
        Page::Name => name_page(commands, name),
        Page::Character => character_page(commands, assets, signup),
    }
}

/// Draws the page that is up again in the file's colours and words as it
/// changes on desktop (`ui::reread`), as it stands: the name as typed so
/// far stays in its box. Not in a frame a press or Enter is waiting to be
/// acted on ([`act`]), which would take down the page it is about to put up.
fn redraw(
    changed: Res<UiChanged>,
    keyboard: Res<ButtonInput<KeyCode>>,
    signup: Res<Signup>,
    name_box: Res<NameBox>,
    pages: Query<Entity, With<PageRoot>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let Some(page) = signup.page else {
        return;
    };
    if signup.pressed.is_some()
        || name_box.entered
        || keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
    {
        return;
    }
    for page in &pages {
        commands.entity(page).despawn();
    }
    draw(&mut commands, &assets, &signup, &name_box.text, page);
}

/// Draws the arrowhead the character page's arrows share, once.
fn make_arrow(mut images: ResMut<Assets<Image>>, mut signup: ResMut<Signup>) {
    signup.arrow = images.add(hud::side_arrow());
}

/// The page itself: the whole screen, a solid colour, taking every press
/// there is, with what stands on it placed down it.
fn page() -> impl Bundle {
    (
        PageRoot,
        TakesPress,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(ui().colours.signup.page),
        GlobalZIndex(LAYER),
    )
}

/// A column across the page, `down` of its height from the top, with what
/// is in it stacked in the middle.
fn column(down: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            top: Val::Vh(down),
            left: Val::ZERO,
            right: Val::ZERO,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::VMin(3.5),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// A page's one button, white with black words, as it can be pressed;
/// [`light_save`] greys it while it cannot.
fn button(text: &str, action: Action) -> impl Bundle {
    let (fill, ink) = white_button();
    (
        action,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(BUTTON_WIDTH),
            height: Val::VMin(BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![builder::label(text, BUTTON_TEXT, ink)],
    )
}

/// The town's name, Start, and under it Log In for an account from another
/// device.
fn start_page(commands: &mut Commands) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.signup);
    commands.spawn(page()).with_children(|page| {
        page.spawn(column(TITLE_DOWN))
            .with_child(builder::label(words.title.as_str(), TITLE, colours.text));
        page.spawn(column(START_DOWN)).with_children(|column| {
            column.spawn(button(&words.start, Action::Start));
            column
                .spawn((
                    Node {
                        align_items: AlignItems::Baseline,
                        column_gap: Val::VMin(1.2),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|line| {
                    line.spawn(builder::label(words.have_account.as_str(), SMALL, colours.text_soft));
                    line.spawn((
                        Action::LogIn,
                        TakesPress,
                        Button,
                        Node::default(),
                        children![builder::label(words.log_in.as_str(), SMALL, colours.signup.paper)],
                    ));
                });
            column.spawn((
                Notice,
                Visibility::Hidden,
                builder::label(words.not_available.as_str(), SMALL, colours.warn),
            ));
        });
    });
}

/// A name to go by, typed into a box. The whole column moves up out from
/// under a phone's keyboard ([`LiftsForKeyboard`]).
fn name_page(commands: &mut Commands, name: &str) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.signup);
    commands.spawn(page()).with_children(|page| {
        page.spawn((column(HEADING_DOWN), LiftsForKeyboard::default())).with_children(|column| {
            column.spawn(builder::label(words.choose_name.as_str(), HEADING, colours.text));
            // One line (Hajun, 2026-11-01; three lines of explanation until
            // then), so not held to the box's width. A hyphen, not an en
            // dash, which the built-in font has no glyph for.
            column.spawn(builder::label(words.name_hint.as_str(), SMALL, colours.text_soft));
            column.spawn(name_box(name, FIELD_WIDTH));
            column.spawn((Save, button(&words.save, Action::SaveName)));
        });
    });
}

/// The box a name is typed into, `width` wide, with `name` in it to go on
/// from, which [`NameBox`] takes the keys for once it is [`open`](NameBox::open).
pub(crate) fn name_box(name: &str, width: f32) -> impl Bundle {
    let colours = ui().colours;
    field(
        name,
        Node {
            width: Val::VMin(width),
            height: Val::VMin(BUTTON_HEIGHT),
            padding: UiRect::horizontal(Val::VMin(3.5)),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
            ..default()
        },
        NAME_TEXT,
        colours.signup.field,
        colours.text,
    )
}

/// A box something is typed into, laid out as `node` says, with `text` in
/// it to go on from in words `size` big, in `ink` on `fill`: what
/// [`NameBox`] takes the keys for once it is open, and shows what is typed
/// in, with the caret after it. Tapped, it asks a phone for its keyboard
/// again.
pub(crate) fn field(text: &str, node: Node, size: f32, fill: Color, ink: Color) -> impl Bundle {
    (
        NameBoxTap,
        TakesPress,
        Button,
        node,
        BackgroundColor(fill),
        children![(
            NameText,
            Text::new(shown(text, true)),
            crate::font::sized(size),
            TextColor(ink),
            TextLayout::justify(Justify::Left),
            Pickable::IGNORE,
        )],
    )
}

/// The name as shown in its box: with the caret after it, or without.
fn shown(name: &str, caret: bool) -> String {
    if caret { format!("{name}|") } else { name.to_string() }
}

/// A boy or a girl, each on a card with their picture and nothing else
/// (Hajun, 2026-11-01: "dont put 'Boy' or 'Girl' just the character img";
/// named under their pictures until then), and Save.
/// "Choose Character": a card to each side, the boy's and the girl's, with
/// the picture of the colour it is turned to, and an arrow either side of it
/// to turn through that side's colours (Hajun, 2026-11-02: "put some arrow
/// thingy next to each guy and girl character selection in the start so u
/// can choose which color guy or which color girl"); and Save.
fn character_page(commands: &mut Commands, assets: &AssetServer, signup: &Signup) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.signup);
    commands.spawn(page()).with_children(|page| {
        page.spawn(column(HEADING_DOWN)).with_children(|column| {
            column.spawn(builder::label(words.choose_character.as_str(), HEADING, colours.text));
            column.spawn(builder::label(words.change_later.as_str(), SMALL, colours.text_soft));
            column
                .spawn((
                    Node {
                        column_gap: Val::VMin(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|cards| {
                    for side in 0..PEOPLE_LOOKS.len() {
                        let fill = card_fill(signup.look == Some(side));
                        let colour = signup.colours[side];
                        let look = side_looks(side)[colour];
                        cards
                            .spawn((
                                Node {
                                    column_gap: Val::VMin(ARROW_GAP),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|row| {
                                row.spawn(arrow(signup, side, -1));
                                row.spawn((
                                    Card(side),
                                    Action::Pick(side),
                                    TakesPress,
                                    Button,
                                    Node {
                                        width: Val::VMin(CARD_WIDTH),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::all(Val::VMin(2.5)),
                                        border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                                        ..default()
                                    },
                                    BackgroundColor(fill.idle),
                                    fill,
                                    children![(
                                        CardPicture { side, colour },
                                        ImageNode::new(assets.load(format!("signup/{look}.png"))),
                                        Node {
                                            width: Val::VMin(PICTURE_WIDTH),
                                            height: Val::VMin(PICTURE_WIDTH * 1.5),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    )],
                                ));
                                row.spawn(arrow(signup, side, 1));
                            });
                    }
                });
            column.spawn((Save, button(&words.save, Action::SaveLook)));
        });
    });
}

/// An arrow beside a side's card, turning its colours `by` one on or one
/// back: a white arrowhead on nothing, pointing left, or flipped across to
/// point right, with room round it for a finger.
fn arrow(signup: &Signup, side: usize, by: i32) -> impl Bundle {
    let arrow_ink = ui().colours.signup.arrow;
    (
        ColourArrow { side, by },
        Action::Turn(side, by),
        TakesPress,
        Button,
        Node {
            width: Val::VMin(ARROW_TOUCH),
            height: Val::VMin(ARROW_TOUCH * 1.6),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            ImageNode {
                flip_x: by > 0,
                color: arrow_ink,
                ..ImageNode::new(signup.arrow.clone())
            },
            Node {
                width: Val::VMin(ARROW_WIDTH),
                height: Val::VMin(ARROW_WIDTH * 1.4),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// A card's colour: the chosen one lighter, the rest lit only under the
/// pointer.
fn card_fill(chosen: bool) -> Fill {
    let colours = ui().colours.signup;
    if chosen {
        Fill {
            idle: colours.chosen,
            lit: colours.chosen,
        }
    } else {
        Fill {
            idle: colours.field,
            lit: colours.field_lit,
        }
    }
}

/// A press on one of the pages' buttons, to be acted on ([`act`]).
fn press(click: On<Pointer<Click>>, actions: Query<&Action>, mut signup: ResMut<Signup>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(&action) = actions.get(click.entity) {
        signup.pressed = Some(action);
    }
}

/// A tap on the box the name goes in: for the keyboard, on a phone.
fn tap_box(click: On<Pointer<Click>>, boxes: Query<(), With<NameBoxTap>>, mut name_box: ResMut<NameBox>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if boxes.contains(click.entity) && name_box.up {
        name_box.keyboard_again = true;
    }
}

/// Keys, on desktop, and on a phone the keyboard's Enter: typed into the
/// box that is up. Each letter comes as a key press with the letter in it,
/// and a space and a backspace as themselves. What a desktop's own input
/// method composes, a Hangul syllable or a Vietnamese vowel with its tone,
/// is shown after what is typed while it is composed (`Ime::Preedit`) and
/// typed once it is done (`Ime::Commit`).
///
/// **Both are read from one list, in the order they came**
/// (`WindowEvent`): a syllable is done by the very key that comes after
/// it, a space say, and read as two lists, every key and then everything
/// composed, the space went in before the syllable it was typed after.
/// (Until 2026-11-14 it was read so, and nothing composed was kept
/// anyway, the game's font having no Hangul.)
#[cfg(not(target_os = "android"))]
fn type_name(mut events: MessageReader<WindowEvent>, mut name_box: ResMut<NameBox>) {
    if !name_box.up {
        events.clear();
        return;
    }
    for event in events.read() {
        match event {
            WindowEvent::KeyboardInput(key) if key.state.is_pressed() => match &key.logical_key {
                Key::Character(text) => text.chars().for_each(|c| name_box.typed(c)),
                Key::Space => name_box.typed(' '),
                Key::Backspace => {
                    name_box.text.pop();
                }
                _ => {}
            },
            WindowEvent::Ime(Ime::Preedit { value, .. }) => name_box.compose(value),
            WindowEvent::Ime(Ime::Commit { value, .. }) => {
                name_box.compose("");
                value.chars().for_each(|c| name_box.typed(c));
            }
            WindowEvent::Ime(Ime::Disabled { .. }) => name_box.compose(""),
            _ => {}
        }
    }
}

/// What a text would leave in a box that takes `most` widths: every letter
/// of it that the game writes, or that is a piece of one, as many as fit;
/// and whether Enter was in it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn typed(text: &str, most: usize) -> (String, bool) {
    let mut name = String::new();
    let mut long = 0;
    let mut entered = false;
    for c in text.chars() {
        if c == '\n' || c == '\r' {
            entered = true;
        } else if letters::typed(c) && long + letters::width(c) <= most {
            long += letters::width(c);
            name.push(c);
        }
    }
    (name, entered)
}

/// How long `text` is as the activity's text counts it, which is as Java
/// does: in UTF-16 units, one to a Hangul syllable where UTF-8 has three
/// bytes. Where the keyboard's caret goes, after all of it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn java_length(text: &str) -> usize {
    text.encode_utf16().count()
}

/// On Android, the soft keyboard sends no keys: it writes into a text the
/// activity keeps (`GameTextInput`), which the name is read off every frame
/// while a box for it is up. Whatever the game does not write, or is past
/// the box's end, is taken back out of that text, so that the keyboard goes
/// on from what is shown. A syllable the keyboard is still composing is in
/// that text as far as it has got, and is left alone: the text is written
/// back only when something was taken out of it, which ends whatever was
/// being composed.
#[cfg(target_os = "android")]
fn android_text(mut name_box: ResMut<NameBox>, time: Res<Time<Real>>, mut seen: Local<(u32, f64)>) {
    // The keyboard's own Enter, which is no key and no letter: counted by
    // the activity ([`IME_ACTIONS`]). One pressed with no box up is
    // forgotten.
    let pressed = IME_ACTIONS.load(std::sync::atomic::Ordering::Relaxed);
    let by_its_key = enter_pressed(pressed, &mut seen, time.elapsed_secs_f64());
    if !name_box.up {
        return;
    }
    if by_its_key {
        name_box.entered = true;
    }
    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let mut state = app.text_input_state();
    let (name, entered) = typed(&state.text, name_box.most);
    if name != state.text {
        let end = java_length(&name);
        state.text.clone_from(&name);
        state.selection.start = end;
        state.selection.end = end;
        state.compose_region = None;
        app.set_text_input_state(state);
    }
    if name_box.text != name {
        name_box.text = name;
    }
    if entered {
        name_box.entered = true;
    }
}

/// How many times the soft keyboard's own Enter has been pressed, on
/// Android, since the app opened: its action key, "Done" or a tick or an
/// arrow, by the keyboard. That key is no key press and types no letter:
/// the keyboard tells the activity to do what the box is for
/// (`InputConnection.performEditorAction`, which `GameActivity` hands to
/// `onEditorAction`), and `MainActivity` counts it here ([`ime_action`]),
/// as it hands over the keyboard's height ([`IME_INSET`]). Until
/// 2026-11-14 nothing heard it (Hajun: "in the friend popup's Add Friend
/// popup, when i type the name, and i press enter nothing happens - only
/// when i press the search bar"): a name was looked for, or saved, only by
/// its button. A newline in the text would have done it ([`typed`]), but
/// the activity's text is one line, and takes none.
#[cfg(target_os = "android")]
static IME_ACTIONS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// `MainActivity.imeAction(int)`: the activity, on its own thread, saying
/// that the keyboard's action key was pressed. JNI finds it by this name.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_imeAction(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    _action: i32,
) {
    IME_ACTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// How long after one press of the keyboard's Enter another counts, in
/// seconds: an Enter that comes as a key, a keyboard's that is plugged in,
/// is handed over twice, once as it goes down and once as it comes up.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
const ENTER_AGAIN: f64 = 0.25;

/// Whether the keyboard's Enter has been pressed since this was last
/// asked, `pressed` being how many times it has been in all, `seen` how
/// many of those have been heard of and when the last one that counted
/// was, and `now` the time, in seconds: once for presses that come within
/// [`ENTER_AGAIN`] of each other.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn enter_pressed(pressed: u32, seen: &mut (u32, f64), now: f64) -> bool {
    if pressed == seen.0 {
        return false;
    }
    seen.0 = pressed;
    if now - seen.1 <= ENTER_AGAIN {
        return false;
    }
    seen.1 = now;
    true
}

/// Starts the activity's text from `name`, as a box for it comes up.
#[cfg(target_os = "android")]
fn android_reset(name: &str) {
    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let mut state = app.text_input_state();
    let end = java_length(name);
    state.text = name.to_string();
    state.selection.start = end;
    state.selection.end = end;
    state.compose_region = None;
    app.set_text_input_state(state);
}

/// Does what was pressed, or what Enter asks for on the page that is up: on
/// from Start, from a name that is one, and, with a character chosen, to the
/// town, signed up.
#[allow(clippy::too_many_arguments)]
fn act(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut signup: ResMut<Signup>,
    mut name_box: ResMut<NameBox>,
    mut profile: ResMut<Profile>,
    mut lobby: ResMut<Lobby>,
    mut up: ResMut<DialogUp>,
    pages: Query<Entity, With<PageRoot>>,
    mut me: Query<(&mut Name, &mut WorldAssetRoot), With<Player>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(page) = signup.page else {
        return;
    };
    let mut action = signup.pressed.take();
    let entered = keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
        || (page == Page::Name && name_box.entered());
    if action.is_none() && entered {
        action = Some(match page {
            Page::Start => Action::Start,
            Page::Name => Action::SaveName,
            Page::Character => Action::SaveLook,
        });
    }
    let Some(action) = action else {
        return;
    };
    let take_down = |commands: &mut Commands| {
        for page in &pages {
            commands.entity(page).despawn();
        }
    };
    match (page, action) {
        (Page::Start, Action::Start) => {
            take_down(&mut commands);
            show(&mut commands, &assets, &mut signup, &mut name_box, Page::Name);
        }
        (Page::Start, Action::LogIn) => {
            signup.notice = Some(0.0);
        }
        (Page::Name, Action::SaveName) => {
            if let Some(name) = name_box.name() {
                signup.name = name;
                name_box.close();
                take_down(&mut commands);
                show(&mut commands, &assets, &mut signup, &mut name_box, Page::Character);
            }
        }
        (Page::Character, Action::Pick(side)) => {
            signup.look = Some(side);
        }
        (Page::Character, Action::Turn(side, by)) => {
            signup.colours[side] = turned(signup.colours[side], by, side_looks(side).len());
            signup.look = Some(side);
        }
        (Page::Character, Action::SaveLook) => {
            let (Some(name), Some(side)) = (profile::name(&signup.name), signup.look) else {
                return;
            };
            *profile = Profile {
                name,
                look: side_looks(side)[signup.colours[side]].into(),
            };
            profile.save();
            info!("signed up as {}, a {}", profile.name, profile.look);
            wear(&profile, &mut lobby, &mut me, &assets);
            take_down(&mut commands);
            signup.page = None;
            up.set_if_neq(DialogUp(false));
        }
        _ => {}
    }
}

/// Keeps the name's box showing the name as typed, with the caret blinking
/// after it.
fn show_name(
    time: Res<Time>,
    mut name_box: ResMut<NameBox>,
    mut texts: Query<&mut Text, With<NameText>>,
) {
    if !name_box.up {
        return;
    }
    name_box.blink += time.delta_secs();
    let caret = (name_box.blink / BLINK_SECS) as u32 % 2 == 0;
    let text = shown(&name_box.shown(), caret);
    for mut shown in &mut texts {
        if shown.0 != text {
            shown.0.clone_from(&text);
        }
    }
}

/// A button that can be pressed, in `look`, or a grey one that cannot: its
/// fill and the colour of its words.
pub(crate) fn save_look(able: bool, look: (Fill, Color)) -> (Fill, Color) {
    if able {
        look
    } else {
        let colours = ui().colours;
        (
            Fill {
                idle: colours.quiet,
                lit: colours.quiet,
            },
            colours.text_soft,
        )
    }
}

/// Gives each of `buttons` the look of one that can be pressed, `look`, or
/// not, as `able` says, if it has not got it already.
pub(crate) fn light(
    able: bool,
    look: (Fill, Color),
    buttons: &mut Query<(&mut Fill, &mut BackgroundColor, &Children), With<Save>>,
    labels: &mut Query<&mut TextColor>,
) {
    let (fill, ink) = save_look(able, look);
    for (mut was, mut colour, children) in buttons {
        if was.idle == fill.idle {
            continue;
        }
        *was = fill;
        colour.0 = fill.idle;
        let mut words = labels.iter_many_mut(children);
        while let Some(mut label) = words.fetch_next() {
            label.0 = ink;
        }
    }
}

/// Greys the page's Save while there is nothing to save: no name yet, or no
/// character chosen.
fn light_save(
    signup: Res<Signup>,
    name_box: Res<NameBox>,
    mut saves: Query<(&mut Fill, &mut BackgroundColor, &Children), With<Save>>,
    mut labels: Query<&mut TextColor>,
) {
    let able = match signup.page {
        Some(Page::Name) => name_box.name().is_some(),
        Some(Page::Character) => signup.look.is_some(),
        _ => return,
    };
    light(able, white_button(), &mut saves, &mut labels);
}

/// Lights the card of the character chosen.
fn light_cards(
    signup: Res<Signup>,
    mut cards: Query<(&Card, &mut Fill, &mut BackgroundColor)>,
) {
    if signup.page != Some(Page::Character) {
        return;
    }
    for (card, mut fill, mut colour) in &mut cards {
        let wanted = card_fill(signup.look == Some(card.0));
        if fill.idle != wanted.idle {
            *fill = wanted;
            colour.0 = wanted.idle;
        }
    }
}

/// Keeps each card's picture that of the colour its arrows have turned it
/// to.
fn turn_cards(
    signup: Res<Signup>,
    assets: Res<AssetServer>,
    mut pictures: Query<(&mut CardPicture, &mut ImageNode)>,
) {
    if signup.page != Some(Page::Character) {
        return;
    }
    for (mut picture, mut image) in &mut pictures {
        let wanted = signup.colours[picture.side];
        if picture.colour != wanted {
            picture.colour = wanted;
            let look = side_looks(picture.side)[wanted];
            image.image = assets.load(format!("signup/{look}.png"));
        }
    }
}

/// Shows "Log in is not available yet" for a few seconds after Log In.
fn fade_notice(
    time: Res<Time>,
    mut signup: ResMut<Signup>,
    mut notices: Query<&mut Visibility, With<Notice>>,
) {
    if let Some(age) = signup.notice.as_mut() {
        *age += time.delta_secs();
        if *age >= NOTICE_SECS {
            signup.notice = None;
        }
    }
    let shown = if signup.notice.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut notices {
        visibility.set_if_neq(shown);
    }
}

/// Asks the window for the keyboard while a box for the name is up, which
/// on a phone shows the soft keyboard, and again when the box is tapped:
/// let go for a frame and asked for again, which brings it back if it was
/// put away. A desktop's keyboard needs no asking; its input method is let
/// in meanwhile, and composes nothing the name keeps.
fn keyboard(mut name_box: ResMut<NameBox>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let again = name_box.keyboard_again;
    if again {
        name_box.keyboard_again = false;
    }
    let wanted = name_box.up && !again;
    if window.ime_enabled != wanted {
        window.ime_enabled = wanted;
    }
}

// --------------------------------------------------- under the keyboard

/// How much of the bottom of the screen a phone's soft keyboard covers, in
/// the window's logical pixels: nothing on desktop, and nothing while it is
/// put away. On Android it is what the activity's window insets say
/// ([`read_keyboard`]); an iPhone's is taken to be [`IOS_KEYBOARD`] of the
/// screen while a box for the name is up, as winit says nothing about it.
#[derive(Resource, Default, PartialEq, Debug)]
pub(crate) struct Keyboard {
    height: f32,
}

/// How much of the screen's height an iPhone's keyboard is taken to cover
/// while a box for the name is up, lying on its side: a guess, not measured
/// on an iPhone.
#[cfg(target_os = "ios")]
const IOS_KEYBOARD: f32 = 0.55;

/// How near the keyboard, and the top of the screen, what is lifted clear of
/// it may come, as a share of the short side of the screen.
const CLEAR_OF_THE_KEYBOARD: f32 = 2.0;

/// Something on the screen that moves up out from under a phone's keyboard
/// ([`lift_for_keyboard`]): the sign-up's name page, and the settings' Edit
/// name dialog (Hajun, 2026-10-31: "when the keyboard goes up on mobile,
/// the keyboard hides the edit name box so u cant see the text anymore.
/// make it go up in reaction to keyboard. as well as when u first set ur
/// name"). It is moved by its own transform, so that it is laid out as it
/// would be with no keyboard and only slid up, by as much as it takes for
/// its bottom to clear the keyboard, and no further than keeps the box the
/// name goes in on the screen.
#[derive(Component, Default)]
pub(crate) struct LiftsForKeyboard {
    /// How far up it is slid now, in logical pixels.
    lifted: f32,
    /// Slid no further than brings its own top to the top of the screen
    /// ([`LiftsForKeyboard::whole`]), rather than as far as keeps the box
    /// typed into on it.
    whole: bool,
}

impl LiftsForKeyboard {
    /// One that stays on the screen from its top down: slid up until its
    /// bottom is clear of the keyboard or its top is at the top of the
    /// screen, whichever comes first, so that what heads it is never slid
    /// off, and what does not fit is under the keyboard instead. The
    /// friends' boxes (since 2026-11-13; Hajun: "the Add Friend popup's top
    /// edge should touch the top of the screen when keyboard is up. right
    /// now the top area is hidden by going over the border").
    pub(crate) fn whole() -> Self {
        Self { lifted: 0.0, whole: true }
    }
}

/// How far up from the bottom of the screen the soft keyboard reaches, on
/// Android, in the screen's own pixels: what `MainActivity` is told by the
/// window's insets, and hands over through [`ime_inset`] each time they
/// change. Nothing is read off the activity's native side: on the night of
/// 2026-10-31 the app crashed on launch calling `GameActivity_getWindowInsets`
/// on `AndroidApp::activity_as_ptr()`, which is a JNI reference to the Java
/// activity, not the native `GameActivity`.
#[cfg(target_os = "android")]
static IME_INSET: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// `MainActivity.imeInset(int)`: the activity, on its own thread, saying how
/// far up the screen the keyboard reaches now. JNI finds it by this name.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_imeInset(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    bottom: i32,
) {
    IME_INSET.store(bottom, std::sync::atomic::Ordering::Relaxed);
}

/// The soft keyboard's height, on Android: what the activity last said
/// ([`IME_INSET`]), in the screen's pixels, which the game draws fewer of
/// (`RENDER_SCALE`), brought to the window's logical pixels.
#[cfg(target_os = "android")]
fn read_keyboard(mut keyboard: ResMut<Keyboard>, windows: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = windows.single() else {
        return;
    };
    let bottom = IME_INSET.load(std::sync::atomic::Ordering::Relaxed).max(0);
    let height = bottom as f32 * crate::RENDER_SCALE / window.scale_factor();
    keyboard.set_if_neq(Keyboard { height });
}

#[cfg(target_os = "ios")]
fn read_keyboard(
    name_box: Res<NameBox>,
    mut keyboard: ResMut<Keyboard>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let height = if name_box.up { window.height() * IOS_KEYBOARD } else { 0.0 };
    keyboard.set_if_neq(Keyboard { height });
}

/// Slides whatever [`LiftsForKeyboard`] up out from under the keyboard, and
/// back down as it goes: as laid out, its bottom [`CLEAR_OF_THE_KEYBOARD`]
/// over the keyboard, unless that would take the box the name goes in off
/// the top of the screen, and then as far as keeps the box that clear of
/// the top.
fn lift_for_keyboard(
    keyboard: Res<Keyboard>,
    windows: Query<&Window, With<PrimaryWindow>>,
    boxes: Query<(&ComputedNode, &UiGlobalTransform), With<NameBoxTap>>,
    mut lifted: Query<(&mut LiftsForKeyboard, &ComputedNode, &UiGlobalTransform, &mut UiTransform)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let clear = window.width().min(window.height()) * CLEAR_OF_THE_KEYBOARD / 100.0;
    let keyboard_top = window.height() - keyboard.height;
    // The box's top, as laid out: it sits in what is lifted, so it has gone
    // up by as much.
    let box_top = boxes.single().ok().map(|(node, place)| {
        (place.translation.y - node.size().y * 0.5) * node.inverse_scale_factor()
    });
    for (mut lift, node, place, mut transform) in &mut lifted {
        let bottom = (place.translation.y + node.size().y * 0.5) * node.inverse_scale_factor() + lift.lifted;
        let wanted = if lift.whole {
            let top = (place.translation.y - node.size().y * 0.5) * node.inverse_scale_factor() + lift.lifted;
            lifting_whole(bottom, top, keyboard_top, clear)
        } else {
            lifting(bottom, box_top.map(|top| top + lift.lifted), keyboard_top, clear)
        };
        if (wanted - lift.lifted).abs() < 0.5 {
            continue;
        }
        lift.lifted = wanted;
        transform.translation = Val2::px(0.0, -wanted);
    }
}

/// How far up something whose bottom is laid out at `bottom`, with the box
/// the name goes in starting at `box_top` in it, is to be slid to clear a
/// keyboard whose top is at `keyboard_top` by `clear`, all in pixels down
/// the screen: as far as that takes, and no further than keeps the box
/// `clear` under the top of the screen.
fn lifting(bottom: f32, box_top: Option<f32>, keyboard_top: f32, clear: f32) -> f32 {
    let wanted = (bottom + clear - keyboard_top).max(0.0);
    let most = box_top.map_or(wanted, |top| (top - clear).max(0.0));
    wanted.min(most)
}

/// How far up something laid out from `top` to `bottom` is to be slid to
/// clear a keyboard whose top is at `keyboard_top` by `clear`, all in pixels
/// down the screen, when it is to stay on the screen from its top down
/// ([`LiftsForKeyboard::whole`]): as far as that takes, and no further than
/// brings its top to the top of the screen.
fn lifting_whole(bottom: f32, top: f32, keyboard_top: f32, clear: f32) -> f32 {
    (bottom + clear - keyboard_top).max(0.0).min(top.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_under_the_keyboard_is_lifted_just_clear_of_it_and_the_box_kept_on_screen() {
        // A screen 1000 tall, the keyboard over its lower half; something
        // laid out from 100 to 600, its box from 300 to 400.
        assert_eq!(lifting(600.0, Some(300.0), 1000.0, 20.0), 0.0, "no keyboard");
        assert_eq!(lifting(600.0, Some(300.0), 500.0, 20.0), 120.0);
        // Too tall to clear it whole: the box stops just under the top.
        assert_eq!(lifting(600.0, Some(300.0), 200.0, 20.0), 280.0);
        // Laid out clear of it already.
        assert_eq!(lifting(450.0, Some(300.0), 500.0, 20.0), 0.0);
        // With no box to keep, as far as it takes.
        assert_eq!(lifting(600.0, None, 500.0, 20.0), 120.0);
        // One kept on the screen from its top down, laid out from 100 to
        // 600: clear of the keyboard where that leaves its top on the
        // screen, and with its top at the very top where it does not.
        assert_eq!(lifting_whole(600.0, 100.0, 1000.0, 20.0), 0.0, "no keyboard");
        assert_eq!(lifting_whole(600.0, 100.0, 540.0, 20.0), 80.0);
        assert_eq!(lifting_whole(600.0, 100.0, 400.0, 20.0), 100.0, "its top at the top of the screen");
        assert_eq!(lifting_whole(600.0, -30.0, 400.0, 20.0), 0.0, "laid out past the top already");
    }

    #[test]
    fn the_keyboard_s_enter_counts_once_however_it_is_handed_over() {
        // Nothing pressed: nothing.
        let mut seen = (0, f64::NEG_INFINITY);
        assert!(!enter_pressed(0, &mut seen, 1.0));
        // Pressed once: Enter, once.
        assert!(enter_pressed(1, &mut seen, 2.0));
        assert!(!enter_pressed(1, &mut seen, 2.1));
        // Handed over twice for one press, down and up, a moment apart:
        // still once.
        assert!(enter_pressed(2, &mut seen, 5.0));
        assert!(!enter_pressed(3, &mut seen, 5.05));
        assert!(!enter_pressed(3, &mut seen, 9.0));
        // And again later is another press. One made with no box up is
        // forgotten by having been asked about.
        assert!(enter_pressed(4, &mut seen, 9.5));
        assert!(enter_pressed(7, &mut seen, 20.0));
        assert!(!enter_pressed(7, &mut seen, 21.0));
    }

    #[test]
    fn what_is_typed_is_kept_as_far_as_it_can_be_a_name() {
        let name = profile::LONGEST;
        assert_eq!(typed("Hajun", name), ("Hajun".into(), false));
        assert_eq!(typed("Hajun\n", name), ("Hajun".into(), true));
        assert_eq!(typed("하준 Park", name), ("하준 Park".into(), false));
        assert_eq!(typed("a very long name indeed", name), ("a very long name".into(), false));
        // A Hangul syllable takes the room of two letters: eight of them,
        // and a syllable still being composed is one as far as it has got.
        assert_eq!(typed("동그라미타운주민들", name), ("동그라미타운주민".into(), false));
        assert_eq!(typed("동그라미타운주ㅁ", name), ("동그라미타운주ㅁ".into(), false));
        assert_eq!(typed("동그라미타운주민a", name), ("동그라미타운주민".into(), false));
        // Vietnamese, a letter each whatever marks it has, in one piece or
        // in two; and what the game does not write is left out.
        assert_eq!(typed("Nguyễn Văn Hải!!!", name), ("Nguyễn Văn Hải!!".into(), false));
        assert_eq!(typed("Ha\u{309}i", name), ("Ha\u{309}i".into(), false));
        assert_eq!(typed("gg\u{1F600}!", name), ("gg!".into(), false));
        // Where the keyboard's caret goes after it, as Java counts.
        assert_eq!((java_length("Hajun"), java_length("하준"), java_length("Hải")), (5, 2, 3));
        assert_eq!("하준".len(), 6);
        // A box for something longer takes that much, and the same letters.
        assert_eq!(typed("a very long name indeed", 40), ("a very long name indeed".into(), false));
        let mut search = NameBox::default();
        search.open_for("", 21);
        for c in "Hajooon#1323 and more besides".chars() {
            search.typed(c);
        }
        assert_eq!(search.text(), "Hajooon#1323 and more");
        assert!(search.up());
        search.close();
        assert!(!search.up());
        let mut name_box = NameBox::default();
        name_box.open("");
        for c in "Mochi 482".chars() {
            name_box.typed(c);
        }
        assert_eq!(name_box.name().as_deref(), Some("Mochi 482"));
        assert!(!name_box.entered());
        name_box.typed('\n');
        assert!(name_box.entered());
        assert!(!name_box.entered(), "taken once");
        let mut short = NameBox::default();
        short.typed('a');
        assert_eq!(short.name(), None, "one letter is not a name");
        // Korean: one syllable is a name, and eight are as many as fit.
        let mut korean = NameBox::default();
        korean.open("");
        korean.typed('민');
        assert_eq!(korean.name().as_deref(), Some("민"));
        for c in "동그라미타운주민들".chars() {
            korean.typed(c);
        }
        assert_eq!(korean.text(), "민동그라미타운주");
        // What a desktop's input method is composing is shown after what
        // is typed and is no part of it until it is done.
        let mut composing = NameBox::default();
        composing.open("");
        composing.typed('하');
        composing.compose("ㅈ");
        assert_eq!((composing.text(), composing.shown().as_str()), ("하", "하ㅈ"));
        assert!(!composing.is_empty());
        composing.compose("준");
        assert_eq!(composing.shown(), "하준");
        composing.compose("");
        composing.typed('준');
        assert_eq!((composing.text(), composing.shown().as_str(), composing.name().as_deref()), ("하준", "하준", Some("하준")));
        // Taken with what is typed, if the box is done with before it is.
        composing.compose("ㅎ");
        assert_eq!(composing.name().as_deref(), Some("하준ㅎ"));
        // Not shown past the end of the box: eight syllables are a name.
        let mut full = NameBox::default();
        full.open("동그라미타운주민");
        full.compose("들");
        assert_eq!(full.shown(), "동그라미타운주민");
        composing.compose("x");
        composing.close();
        assert!(composing.composing.is_empty(), "nothing composed is kept for the next box");
        // Vietnamese typed in pieces is shown, and kept, put together.
        let mut vietnamese = NameBox::default();
        vietnamese.open("");
        for c in "Ha\u{309}i Vie\u{302}\u{323}t".chars() {
            vietnamese.typed(c);
        }
        assert_eq!(vietnamese.shown(), "Hải Việt");
        assert_eq!(vietnamese.name().as_deref(), Some("Hải Việt"));
        assert!(NameBox::default().is_empty());
    }

    #[test]
    fn the_arrows_turn_a_side_s_colours_round_and_round() {
        assert_eq!(turned(0, 1, 3), 1);
        assert_eq!(turned(2, 1, 3), 0);
        assert_eq!(turned(0, -1, 3), 2);
        assert_eq!(turned(1, -1, 3), 0);
        // Each side's first colour is the look people chose between before
        // the colours came.
        assert_eq!(side_looks(0)[0], PEOPLE_LOOKS[0]);
        assert_eq!(side_looks(1)[0], PEOPLE_LOOKS[1]);
    }

    #[test]
    fn a_profile_is_set_once_both_are_good() {
        let mut profile = Profile::default();
        assert!(!profile.set());
        profile.name = "Hajun".into();
        assert!(!profile.set());
        profile.look = "girl".into();
        assert!(profile.set());
        // Only as it would be kept: trimmed, and a look there is.
        assert!(!Profile { name: " Hajun".into(), look: "girl".into() }.set());
        assert!(!Profile { name: "Hajun".into(), look: "blue".into() }.set());
        // Any colour of either side is a look.
        assert!(Profile { name: "Hajun".into(), look: "girl_whiteblue".into() }.set());
        assert!(Profile { name: "Hajun".into(), look: "boy_blackbrown".into() }.set());
        // The server is asked to take it on only where it has you otherwise.
        let me = |name: &str, look: &str| Member {
            id: 7,
            name: name.into(),
            look: look.into(),
            ai: false,
        };
        assert!(profile.ask_for(&me("Hajun", "girl")).is_none());
        assert!(matches!(profile.ask_for(&me("Mochi 482", "blue")), Some(Ask::Profile { .. })));
        assert!(Profile::default().ask_for(&me("Mochi 482", "blue")).is_none());
    }

    #[test]
    fn the_characters_pictures_are_there_and_in_the_apk() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim_start_matches("<dir>").trim_start_matches("<file>"))
            .collect();
        for look in BOY_LOOKS.iter().chain(&GIRL_LOOKS) {
            let picture = format!("signup/{look}.png");
            assert!(root.join("assets").join(&picture).is_file(), "assets/{picture}");
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
}
