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
//! Typing the name: on desktop the keys; on a phone the soft keyboard, which
//! the window's `ime_enabled` asks for. An iPhone's sends each letter as a
//! key press, and Return as a newline. Android's sends no keys at all and
//! writes into a text of its own, which the name is read off every frame
//! ([`android_text`]). Only plain ASCII the game's font can draw is kept
//! (`roundtown_net::profile`): no Hangul. A phone lying on its side has its
//! keyboard over most of the screen, so the name's page, and the settings'
//! Edit name, slide up out from under it ([`Keyboard`],
//! [`LiftsForKeyboard`]).

#[cfg(not(target_os = "android"))]
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
#[cfg(not(target_os = "android"))]
use bevy::window::Ime;
use bevy::window::PrimaryWindow;
use roundtown_net::{Ask, Member, PEOPLE_LOOKS, profile};
use serde::{Deserialize, Serialize};

use crate::Player;
use crate::builder::{self, Fill, QUIET, SOFT};
use crate::hud::{DOOR, DialogUp, GOLD, TakesPress};
use crate::lobby::{self, Lobby};

/// The file who you are lives in, beside the town save.
const FILE: &str = "profile.json";
/// Bumped if the file's shape changes.
const VERSION: u32 = 1;

/// The pages' colours: the page, black, and what stands on it in greys: the
/// box the name is typed into and the two characters' cards, one of them
/// chosen, lighter; and the one button each page has, white with black
/// words, or grey while it cannot be pressed (Hajun, 2026-10-31: "black for
/// the initial pages ... and white instead of yellow for the buttons"; a
/// dark blue page with blue boxes and gold buttons for the afternoon).
const PAGE: Color = Color::BLACK;
const FIELD: Color = Color::srgb(0.15, 0.15, 0.16);
const FIELD_LIT: Color = Color::srgb(0.21, 0.21, 0.22);
const CHOSEN: Color = Color::srgb(0.36, 0.36, 0.38);
const PAPER: Color = Color::WHITE;
const PAPER_LIT: Color = Color::srgb(0.84, 0.84, 0.85);
const INK: Color = Color::srgb(0.05, 0.05, 0.06);
pub(crate) const GOLD_LIT: Color = Color::srgb(1.0, 0.9, 0.52);
const WARN: Color = Color::srgb(1.0, 0.62, 0.55);

/// A white button with black words: the pages' own.
const WHITE_BUTTON: (Fill, Color) = (
    Fill {
        idle: PAPER,
        lit: PAPER_LIT,
    },
    INK,
);
/// A gold button with the dark blue of words on gold: the settings' Save, in
/// the look of the town's dialogs.
pub(crate) const GOLD_BUTTON: (Fill, Color) = (
    Fill {
        idle: GOLD,
        lit: GOLD_LIT,
    },
    DOOR,
);

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
    /// The character chosen, by its place in [`PEOPLE_LOOKS`].
    look: Option<usize>,
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
/// sign-up's page, and in the settings' Edit name. At most one is up at a
/// time, and while it is, it takes every key and asks a phone for its
/// keyboard. [`name_box`] is the box itself.
#[derive(Resource, Default)]
pub(crate) struct NameBox {
    up: bool,
    /// The name as typed so far.
    text: String,
    /// Seconds the caret has blinked for.
    blink: f32,
    /// Enter was pressed, or typed, and not yet taken.
    entered: bool,
    /// The soft keyboard is to be asked for again next frame: the box was
    /// tapped, on a phone where the keyboard may have been put away.
    keyboard_again: bool,
}

impl NameBox {
    /// A box has come up, with `text` in it to go on from.
    pub(crate) fn open(&mut self, text: &str) {
        self.up = true;
        self.text = text.to_string();
        self.blink = 0.0;
        self.entered = false;
        #[cfg(target_os = "android")]
        android_reset(text);
    }

    /// The box has gone.
    pub(crate) fn close(&mut self) {
        self.up = false;
        self.entered = false;
    }

    /// The name as typed, if it is a name.
    pub(crate) fn name(&self) -> Option<String> {
        profile::name(&self.text)
    }

    /// Whether Enter has been pressed, or typed, since this was last asked.
    pub(crate) fn entered(&mut self) -> bool {
        std::mem::take(&mut self.entered)
    }

    /// Takes `c` as typed into the name: kept if it may be in one and there
    /// is room; a newline is Enter.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn typed(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            self.entered = true;
        } else if profile::allowed(c) && self.text.len() < profile::LONGEST {
            self.text.push(c);
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
    /// This character.
    Pick(usize),
    SaveLook,
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

/// One character's card, by its place in [`PEOPLE_LOOKS`].
#[derive(Component)]
struct Card(usize);

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
        .add_systems(Startup, open_if_unset)
        .add_systems(
            Update,
            (
                #[cfg(target_os = "android")]
                android_text,
                #[cfg(not(target_os = "android"))]
                type_name,
                act,
                show_name,
                light_save,
                light_cards,
                fade_notice,
                keyboard,
                #[cfg(any(target_os = "android", target_os = "ios"))]
                read_keyboard,
                lift_for_keyboard,
            )
                .chain(),
        );
}

/// The first time the app is run, or the name and the character are not
/// set: the pages come before the town.
fn open_if_unset(
    profile: Res<Profile>,
    mut signup: ResMut<Signup>,
    mut up: ResMut<DialogUp>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    if !profile.set() {
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
    *signup = Signup::default();
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
    match page {
        Page::Start => start_page(commands),
        Page::Name => {
            name_box.open(&signup.name);
            name_page(commands, &signup.name);
        }
        Page::Character => character_page(commands, assets, signup.look),
    }
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
        BackgroundColor(PAGE),
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
    let (fill, ink) = WHITE_BUTTON;
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
    commands.spawn(page()).with_children(|page| {
        page.spawn(column(TITLE_DOWN))
            .with_child(builder::label("Donggeurami Town", TITLE, Color::WHITE));
        page.spawn(column(START_DOWN)).with_children(|column| {
            column.spawn(button("Start", Action::Start));
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
                    line.spawn(builder::label("Already have an account?", SMALL, SOFT));
                    line.spawn((
                        Action::LogIn,
                        TakesPress,
                        Button,
                        Node::default(),
                        children![builder::label("Log In", SMALL, PAPER)],
                    ));
                });
            column.spawn((
                Notice,
                Visibility::Hidden,
                builder::label("Log in is not available yet", SMALL, WARN),
            ));
        });
    });
}

/// A name to go by, typed into a box. The whole column moves up out from
/// under a phone's keyboard ([`LiftsForKeyboard`]).
fn name_page(commands: &mut Commands, name: &str) {
    commands.spawn(page()).with_children(|page| {
        page.spawn((column(HEADING_DOWN), LiftsForKeyboard::default())).with_children(|column| {
            column.spawn(builder::label("Choose Username", HEADING, Color::WHITE));
            // One line (Hajun, 2026-11-01; three lines of explanation until
            // then), so not held to the box's width. A hyphen, not an en
            // dash, which the built-in font has no glyph for.
            column.spawn(builder::label("Shown to other players. 2-16 characters.", SMALL, SOFT));
            column.spawn(name_box(name, FIELD_WIDTH));
            column.spawn((Save, button("Save", Action::SaveName)));
        });
    });
}

/// The box a name is typed into, `width` wide, with `name` in it to go on
/// from, which [`NameBox`] takes the keys for once it is [`open`](NameBox::open).
pub(crate) fn name_box(name: &str, width: f32) -> impl Bundle {
    (
        NameBoxTap,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(width),
            height: Val::VMin(BUTTON_HEIGHT),
            padding: UiRect::horizontal(Val::VMin(3.5)),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
            ..default()
        },
        BackgroundColor(FIELD),
        children![(
            NameText,
            Text::new(shown(name, true)),
            TextFont {
                font_size: FontSize::VMin(NAME_TEXT),
                ..default()
            },
            TextColor(Color::WHITE),
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
fn character_page(commands: &mut Commands, assets: &AssetServer, chosen: Option<usize>) {
    commands.spawn(page()).with_children(|page| {
        page.spawn(column(HEADING_DOWN)).with_children(|column| {
            column.spawn(builder::label("Choose Character", HEADING, Color::WHITE));
            column.spawn(builder::label("You may change this later.", SMALL, SOFT));
            column
                .spawn((
                    Node {
                        column_gap: Val::VMin(4.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|cards| {
                    for (n, look) in PEOPLE_LOOKS.iter().enumerate() {
                        let fill = card_fill(chosen == Some(n));
                        cards.spawn((
                            Card(n),
                            Action::Pick(n),
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
                                ImageNode::new(assets.load(format!("signup/{look}.png"))),
                                Node {
                                    width: Val::VMin(PICTURE_WIDTH),
                                    height: Val::VMin(PICTURE_WIDTH * 1.5),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            )],
                        ));
                    }
                });
            column.spawn((Save, button("Save", Action::SaveLook)));
        });
    });
}

/// A card's colour: the chosen one lighter, the rest lit only under the
/// pointer.
fn card_fill(chosen: bool) -> Fill {
    if chosen {
        Fill {
            idle: CHOSEN,
            lit: CHOSEN,
        }
    } else {
        Fill {
            idle: FIELD,
            lit: FIELD_LIT,
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
/// name while a box for it is up. Each letter comes as a key press with the
/// letter in it, and a space and a backspace as themselves; what a desktop's
/// own input method composes, Hangul say, comes whole once it is done, and
/// is kept only as far as the game's font can draw it, which is not at all.
#[cfg(not(target_os = "android"))]
fn type_name(
    mut keys: MessageReader<KeyboardInput>,
    mut composed: MessageReader<Ime>,
    mut name_box: ResMut<NameBox>,
) {
    if !name_box.up {
        keys.clear();
        composed.clear();
        return;
    }
    for key in keys.read() {
        if !key.state.is_pressed() {
            continue;
        }
        match &key.logical_key {
            Key::Character(text) => text.chars().for_each(|c| name_box.typed(c)),
            Key::Space => name_box.typed(' '),
            Key::Backspace => {
                name_box.text.pop();
            }
            _ => {}
        }
    }
    for event in composed.read() {
        if let Ime::Commit { value, .. } = event {
            value.chars().for_each(|c| name_box.typed(c));
        }
    }
}

/// What a text would leave in the name: every character of it that may be
/// in one, as many as fit; and whether Enter was in it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn typed(text: &str) -> (String, bool) {
    let mut name = String::new();
    let mut entered = false;
    for c in text.chars() {
        if c == '\n' || c == '\r' {
            entered = true;
        } else if profile::allowed(c) && name.len() < profile::LONGEST {
            name.push(c);
        }
    }
    (name, entered)
}

/// On Android, the soft keyboard sends no keys: it writes into a text the
/// activity keeps (`GameTextInput`), which the name is read off every frame
/// while a box for it is up. Whatever may not be in a name, or past its
/// end, is taken back out of that text, so that the keyboard goes on from
/// what is shown.
#[cfg(target_os = "android")]
fn android_text(mut name_box: ResMut<NameBox>) {
    if !name_box.up {
        return;
    }
    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let mut state = app.text_input_state();
    let (name, entered) = typed(&state.text);
    if name != state.text {
        state.text.clone_from(&name);
        state.selection.start = name.len();
        state.selection.end = name.len();
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

/// Starts the activity's text from `name`, as a box for it comes up.
#[cfg(target_os = "android")]
fn android_reset(name: &str) {
    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let mut state = app.text_input_state();
    state.text = name.to_string();
    state.selection.start = name.len();
    state.selection.end = name.len();
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
        (Page::Character, Action::Pick(n)) => {
            signup.look = Some(n);
        }
        (Page::Character, Action::SaveLook) => {
            let (Some(name), Some(look)) = (profile::name(&signup.name), signup.look) else {
                return;
            };
            *profile = Profile {
                name,
                look: PEOPLE_LOOKS[look].into(),
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
    let text = shown(&name_box.text, caret);
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
        (
            Fill {
                idle: QUIET,
                lit: QUIET,
            },
            SOFT,
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
    light(able, WHITE_BUTTON, &mut saves, &mut labels);
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
        let wanted = lifting(bottom, box_top.map(|top| top + lift.lifted), keyboard_top, clear);
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
    }

    #[test]
    fn what_is_typed_is_kept_as_far_as_it_can_be_a_name() {
        assert_eq!(typed("Hajun"), ("Hajun".into(), false));
        assert_eq!(typed("Hajun\n"), ("Hajun".into(), true));
        assert_eq!(typed("하준 Park"), (" Park".into(), false));
        assert_eq!(typed("a very long name indeed"), ("a very long name".into(), false));
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
        for look in PEOPLE_LOOKS {
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
