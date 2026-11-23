//! Changing what stands on your plot: the button by the left edge of the
//! screen, and the three it becomes (since 2026-11-22). Hajun: "i want the
//! left center spot to be a ui button that only appears while you are in ur
//! plot, and its a 'customize icon' ... only when u press it, it toggles on
//! the edit mode, which alllows u to tap any object in ur plot to move it
//! around - with edit mode off, tapping objects should do nothing, and also
//! when u open invetory and tap an object to place it, then edit mode gets
//! toggled on until u turn it off. and i think when edit mode is off, its
//! just one ui button, but when its on, it switches to three buttons in a
//! column on the left middle, the top one being a normal edit/customize
//! icon, the second one being a paintbrush icon ... when the paintbrush icon
//! is selected, u also get a circular icon on the right side of it that just
//! shows a color, and u tap that color button and it opens a rgb selector
//! panel ... and the third one being a check mark button (which is the edit
//! mode off button)."
//!
//! **Edit mode** ([`Customize`]) is off until the button is pressed
//! ([`Press::Begin`]), or something is brought out of your bag to put down
//! (`build::stand`, [`Customize::begin`]); and on until the tick is
//! pressed, or you are no longer by your plot: more than a few steps off
//! it, in your car, or out of the town ([`keep_to_the_plot`]). While it is
//! off nothing of yours answers a tap (`build::stand::hands_off`).
//!
//! **Two tools.** The house and pencil is the hand ([`Tool::Hand`]): a tap
//! on a thing of yours picks it up, to be moved, turned or binned, as
//! always (`editor`). The paintbrush ([`Tool::Brush`]) paints: a tap on a
//! part of a thing paints that part the colour on the round button beside
//! the brush (`build::paint`), and a tap on that button opens the window
//! the colour is chosen in ([`picker`]). Whatever is in your hand when the
//! brush is taken up, or the tick pressed, is put down where it is first,
//! as by its own tick; and if it cannot stand there the hand stays, the
//! menu open on it again ([`Wants`], [`settle`]).
//!
//! **Where the buttons are** (`hud::LeftEdge`, which this fills, [`draw`]):
//! one, while you stand on your plot, and in edit mode three, one over the
//! other, the brush where the one was, the house and pencil over it and
//! the tick under it (Hajun, later that day: "the house edit button, is a
//! bit too down, i think u can move it up a bit, and when it flanels out to
//! the 3 buttons, the middle one can be the location of the original house
//! edit button"; it stood where the tick is until then). They stand where
//! the chat's lines begin, which step aside to the right of them for as
//! long as they are up (`chat::Aside`).
//!
//! **What the brush paints with** ([`Ink`]) is chosen beside it: the round
//! button the colour, which a tap takes up and a second tap opens the
//! window for; and beside that a cross, which paints nothing on but takes
//! the paint off, the part as its model has it again ("on the right side of
//! the rgb color icon, it should be an x/clear icon and u can select
//! between the color or the clear, and if u paint sth with the clear, it
//! will clear that color to its original"). The colours painted with are
//! kept, the last seven, newest first, for the window to offer again
//! ([`Customize::used`]: "one row of the previous used color history"), and
//! with the colour chosen they are kept for the next time the app is
//! opened, on this device (`paint.json`, [`keep`]; mine).
//!
//! **Whether edit mode is on is plain to see** (since 2026-11-23; Hajun: "i
//! think its hard to tell if you are in house edit mode or not. maybe make
//! it obvious that you are in house edit mode somehow, maybe adding a text
//! "Editing Mode Enabled" on center top of your screen while its active?
//! ... i need it to be obvious so that this confusion doesnt happen: my
//! friend was confused cuz she couldnt tap the sleep button next to the
//! bed, but its cuz she was in house edit mode. she was also confused why
//! she couldnt move the object by tapping, but its because she was not in
//! house edit mode"). For as long as it is on, a plate along the top of the
//! screen, in the middle, says so in their words ([`say_so`],
//! `words.bag.editing`), and the four edges of your plot are lit on the
//! ground ([`light_the_plot`]). And with it off, a tap at a thing of yours
//! makes the button that turns it on beat ([`Pointed`], [`beat`];
//! `build::stand::hint_at_edit_mode` reads the tap), every time and
//! without a word: the words that were said the first few times are turned
//! off ([`HINTS`]; "i think the tip notification to go into house edit
//! mode is not the real fix - should delete that and use another way").

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Player;
use crate::build::stand::{Home, Mine};
use crate::build::{BuildSystems, Ghost, Yard};
use crate::chat;
use crate::drive::Driving;
use crate::editor::{self, Edit, EditMenu};
use crate::hud::{self, LeftEdge, TakesPress};
use crate::island::Venue;
use crate::safe::KeepsClear;
use crate::ui::UiChanged;

pub(crate) mod picker;

/// How far off your plot edit mode lasts, in metres: a step over its edge
/// with something in your hand does not end it.
pub(crate) const NEAR: f32 = 3.0;
/// The round button that shows the colour, beside the brush: how big, how
/// far from the brush, and how thick its rim is, which is also the ring
/// round the tool in hand; in shares of the short side of the screen.
const SWATCH: f32 = 8.5;
const SWATCH_GAP: f32 = 1.5;
const RIM: f32 = 0.45;
/// How many of the colours painted with are kept, to be offered again: one
/// row of the window.
pub(crate) const USED: usize = 7;
/// The file the colour chosen and those painted with are kept in, beside
/// the bag's, and its version.
const FILE: &str = "paint.json";
const VERSION: u32 = 1;
/// Whether someone who taps twice at a thing of theirs with edit mode off
/// is told in words how it is moved (`stand::hint_at_edit_mode`,
/// `words.bag.edit_hint`, the first few times ever: [`MOST_HINTS`]). Not
/// since 2026-11-23 (Hajun: "i think the tip notification to go into house
/// edit mode is not the real fix - should delete that and use another
/// way"): the button that turns edit mode on beats instead ([`Pointed`]),
/// every time, and edit mode itself is plain to see ([`say_so`],
/// [`light_the_plot`]). The words, the count of them kept in `paint.json`
/// and their tests are all still here: this one word puts them back.
pub(crate) const HINTS: bool = false;
/// How many times, ever, someone tapping at a thing of theirs with edit
/// mode off is told how to move it, while they are told at all ([`HINTS`];
/// Hajun, 2026-11-22: "just for the first few times"). Mine.
const MOST_HINTS: u8 = 3;
/// The plate that says edit mode is on ([`say_so`]), drawn as your name's
/// plate is (`hud::name_plate`), in shares of the short side of the
/// screen: how big its words are, your name's own size; the house and
/// pencil before them, a little under a line of them tall; the gap between
/// the two; the room round them on the plate, across and down, and how
/// round its corners are, all three your name's plate's; and how far clear
/// of your name and the gear beside it the plate keeps where the screen is
/// too narrow for it to stand in the middle ([`make_room`]).
const PLATE_TEXT: f32 = 4.2;
const PLATE_PICTURE: f32 = 4.8;
const PLATE_GAP: f32 = 1.2;
const PLATE_PAD: (f32, f32) = (2.4, 1.0);
const PLATE_ROUND: f32 = 1.6;
const PLATE_CLEAR: f32 = 2.0;
/// The beat of the button that turns edit mode on, while it is pointed at
/// ([`Pointed`]): how long it lasts, in seconds; how many times the button
/// swells and comes back in that; and how much bigger than itself it is at
/// the top of each. Mine: Hajun asked for "another way", and not which.
const BEAT_SECS: f32 = 1.2;
const BEATS: f32 = 3.0;
const BEAT_SWELL: f32 = 0.35;
/// The line lit along the edges of your plot while edit mode is on
/// ([`light_the_plot`]): how wide it is, in metres, just inside the plot's
/// edge; and how far over the plot's ground it lies, so as not to flicker
/// into it, as the ring under a thing in your hand does (`editor`).
const EDGE_WIDE: f32 = 0.35;
const EDGE_OVER: f32 = 0.03;

/// What a tap on a thing of yours does in edit mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Tool {
    /// Picks it up, to be moved.
    #[default]
    Hand,
    /// Paints the part of it under the finger.
    Brush,
}

/// What the brush paints with.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Ink {
    /// The colour chosen.
    #[default]
    Colour,
    /// Nothing: it takes the paint off, and the part is as its model has
    /// it.
    Clear,
}

/// What was asked for with something still in your hand, to be done once
/// that is down: the brush, or out of edit mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Wants {
    Brush,
    Off,
}

/// A colour as the window has it chosen: which way round the wheel, in
/// turns from red, the way the hours do not go; how far out from the white
/// in its middle, to the full colour at its rim; and how bright, from
/// black.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Hsv {
    pub hue: f32,
    pub out: f32,
    pub bright: f32,
}

impl Hsv {
    /// The colour as it may be, whatever a file said: every part of it a
    /// number, and within what it can be.
    fn sane(self) -> Self {
        let within = |part: f32| if part.is_finite() { part.clamp(0.0, 1.0) } else { 0.0 };
        Self {
            hue: if self.hue.is_finite() { self.hue.rem_euclid(1.0) } else { 0.0 },
            out: within(self.out),
            bright: within(self.bright),
        }
    }

    /// Its red, green and blue as a screen has them.
    pub(crate) fn rgb(self) -> [u8; 3] {
        let (hue, out, bright) = (self.hue.rem_euclid(1.0) * 6.0, self.out.clamp(0.0, 1.0), self.bright.clamp(0.0, 1.0));
        // Each of the three: full for a third of the way round, falling
        // and rising over a sixth each side, and nothing opposite.
        let part = |from: f32| {
            let along = (hue + from).rem_euclid(6.0);
            let share = (2.0 - (along - 3.0).abs()).clamp(0.0, 1.0);
            ((1.0 - out * share) * bright * 255.0).round() as u8
        };
        [part(0.0), part(4.0), part(2.0)]
    }

    pub(crate) fn color(self) -> Color {
        let [red, green, blue] = self.rgb();
        Color::srgb_u8(red, green, blue)
    }

    /// The same colour as bright as it goes: what the wheel shows.
    pub(crate) fn full(self) -> Self {
        Self { bright: 1.0, ..self }
    }
}

/// Changing what stands on your plot: whether edit mode is on, which tool
/// is in hand, what the brush paints with, the colour chosen, the colours
/// painted with before, newest first, and what waits on something being
/// put down.
#[derive(Resource, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Customize {
    on: bool,
    tool: Tool,
    ink: Ink,
    colour: Hsv,
    used: [Option<Hsv>; USED],
    /// How many times it has been said how a thing is moved.
    hinted: u8,
    wants: Option<Wants>,
}

impl Default for Customize {
    fn default() -> Self {
        Self {
            on: false,
            tool: Tool::Hand,
            ink: Ink::Colour,
            // A blue, to begin with (mine).
            colour: Hsv {
                hue: 0.6,
                out: 0.7,
                bright: 0.9,
            },
            used: [None; USED],
            hinted: 0,
            wants: None,
        }
    }
}

/// The colour chosen and those painted with, as they are kept in the file,
/// and how many times it has been said how a thing is moved (nothing, in a
/// file from before that was).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Kept {
    version: u32,
    colour: Hsv,
    used: Vec<Hsv>,
    #[serde(default)]
    hinted: u8,
}

impl Customize {
    pub(crate) fn on(&self) -> bool {
        self.on
    }

    /// Whether a tap on a thing of yours picks it up.
    pub(crate) fn moving(&self) -> bool {
        self.on && self.tool == Tool::Hand
    }

    /// Whether a tap on a thing of yours paints it.
    pub(crate) fn painting(&self) -> bool {
        self.on && self.tool == Tool::Brush
    }

    /// Whether a tap on a thing of yours takes the paint off it.
    pub(crate) fn clearing(&self) -> bool {
        self.painting() && self.ink == Ink::Clear
    }

    /// The colour the brush paints.
    pub(crate) fn colour(&self) -> [u8; 3] {
        self.colour.rgb()
    }

    pub(crate) fn chosen(&self) -> Hsv {
        self.colour
    }

    /// The colours painted with before, newest first.
    pub(crate) fn used(&self) -> impl Iterator<Item = Hsv> {
        self.used.into_iter().flatten()
    }

    /// Whether the colour chosen is the last one painted with: nothing to
    /// note of it ([`Customize::note_used`]).
    pub(crate) fn last_used(&self) -> bool {
        self.used[0].is_some_and(|used| used.rgb() == self.colour.rgb())
    }

    /// The colour chosen has been painted with: first of those kept, the
    /// rest after it, as they were but for itself, and no more than the
    /// window has room for.
    pub(crate) fn note_used(&mut self) {
        let mut used = [None; USED];
        used[0] = Some(self.colour);
        let rest = self.used.into_iter().flatten().filter(|other| other.rgb() != self.colour.rgb());
        for (kept, other) in used[1..].iter_mut().zip(rest) {
            *kept = Some(other);
        }
        self.used = used;
    }

    /// Whether it is still to be said how a thing is moved: the first few
    /// times only. (Whether it is said at all is [`HINTS`].)
    pub(crate) fn may_hint(&self) -> bool {
        self.hinted < MOST_HINTS
    }

    /// It has been said once more.
    pub(crate) fn note_hint(&mut self) {
        self.hinted = self.hinted.saturating_add(1);
    }

    /// As it was kept, if it was.
    fn remembering(kept: Option<Kept>) -> Self {
        let Some(kept) = kept.filter(|kept| kept.version == VERSION) else {
            return Self::default();
        };
        let mut used = [None; USED];
        for (slot, colour) in used.iter_mut().zip(kept.used) {
            *slot = Some(colour.sane());
        }
        Self {
            colour: kept.colour.sane(),
            used,
            hinted: kept.hinted,
            ..default()
        }
    }

    fn to_keep(&self) -> Kept {
        Kept {
            version: VERSION,
            colour: self.colour,
            used: self.used().collect(),
            hinted: self.hinted,
        }
    }

    /// Edit mode is on from here, the hand its tool: the button was
    /// pressed, or something has been brought out of your bag to put down.
    pub(crate) fn begin(&mut self) {
        (self.on, self.tool, self.wants) = (true, Tool::Hand, None);
    }

    /// Edit mode with the brush in hand.
    #[cfg(test)]
    pub(crate) fn with_the_brush() -> Self {
        Self {
            on: true,
            tool: Tool::Brush,
            ..default()
        }
    }

    /// Another colour chosen.
    fn choose(&mut self, colour: Hsv) {
        self.colour = colour;
    }

    /// `wants` is asked for: done at once with nothing in your hand, and
    /// otherwise once that is down, which it is told to be now, as by its
    /// own tick ([`settle`]).
    fn ask(&mut self, wants: Wants, menu: &mut EditMenu, edits: &mut MessageWriter<Edit>) {
        if menu.is_open() {
            menu.confirm(edits);
            self.wants = Some(wants);
        } else {
            self.take(wants);
        }
    }

    fn take(&mut self, wants: Wants) {
        match wants {
            Wants::Brush => self.tool = Tool::Brush,
            Wants::Off => self.on = false,
        }
        self.wants = None;
    }
}

/// What a press on one of the buttons by the left edge does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Press {
    /// The one that is there alone: edit mode on.
    Begin,
    /// The three: the hand, the brush, and the tick, which is edit mode
    /// off.
    Hand,
    Brush,
    Done,
    /// The round button beside the brush: the colour to paint with, and
    /// with that in hand already, the window it is chosen in.
    Swatch,
    /// The cross beside that: nothing to paint with, which takes paint
    /// off.
    Clear,
}

/// The round button that shows the colour.
#[derive(Component)]
struct Swatch;

/// Which buttons are by the left edge: with the three, which tool is in
/// hand and what the brush paints with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shown {
    Nothing,
    One,
    Three(Tool, Ink),
}

/// The pictures on the buttons, each made once: the house and pencil, the
/// brush and the tick, drawn here, and the windows' own cross.
#[derive(Resource)]
struct Pictures {
    house: Handle<Image>,
    brush: Handle<Image>,
    tick: Handle<Image>,
    cross: Handle<Image>,
}

/// The button that turns edit mode on is pointed at: for how many seconds
/// it has been beating ([`beat`]), or not at all. Someone who taps at a
/// thing of theirs with edit mode off is shown where edit mode is, every
/// time and without a word (`build::stand::hint_at_edit_mode`; Hajun,
/// 2026-11-23: "she was also confused why she couldnt move the object by
/// tapping, but its because she was not in house edit mode", and of the
/// words said until then, "not the real fix - should delete that and use
/// another way").
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Pointed(Option<f32>);

impl Pointed {
    /// It is pointed at from now, for all its beats. One that is beating
    /// already goes on from where it is in its beat, so that it never
    /// jumps, with all of them still to come from the start of that one.
    pub(crate) fn begin(&mut self) {
        self.0 = Some(self.0.map_or(0.0, |age| age.rem_euclid(BEAT_SECS / BEATS)));
    }
}

/// The plate along the top of the screen that says edit mode is on: the
/// row it stands in, from one side of the screen to the other.
#[derive(Component)]
struct EditingPlate;

/// The length of that row that lies before the plate, over your name and
/// the gear beside it, top left: no shorter than they are wide
/// ([`make_room`]).
#[derive(Component)]
struct NameRoom;

/// The line lit along the edges of your plot while edit mode is on, and the
/// plot it was made for.
#[derive(Component)]
struct PlotEdge(Yard);

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(Customize::remembering(load()))
        .init_resource::<Pointed>()
        .add_observer(press)
        .add_systems(Startup, draw_pictures)
        .add_systems(
            Update,
            (
                keep_to_the_plot.before(editor::EditSystems),
                settle.after(BuildSystems),
                (draw, tint_swatch, keep).chain().after(settle),
                // Once the button there is to beat is up.
                beat.after(draw),
                // What says that edit mode is on, once it is known whether
                // it still is this frame.
                ((say_so, make_room).chain(), light_the_plot).after(settle),
            ),
        );
    picker::plugin(app);
}

fn draw_pictures(mut images: ResMut<Assets<Image>>, assets: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Pictures {
        house: images.add(hud::customize()),
        brush: images.add(hud::brush()),
        tick: images.add(hud::tick()),
        cross: assets.load("icons/close.png"),
    });
}

/// The colour chosen and those painted with, as the file has them, if there
/// is one that reads.
fn load() -> Option<Kept> {
    let path = crate::map::data_file(FILE)?;
    let text = std::fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<Kept>(&text) {
        Ok(kept) => Some(kept),
        Err(error) => {
            warn!("{} does not read ({error}): begun afresh", path.display());
            None
        }
    }
}

/// Keeps the colour chosen and those painted with for the next run, as
/// they change: written beside the file and renamed over it, as the bag is.
/// Not while the window is up, where a finger dragged over the wheel
/// changes the colour every frame: once it is shut.
fn keep(customize: Res<Customize>, picker: Res<picker::Picker>, mut kept: Local<Option<Kept>>) {
    if !(customize.is_changed() || picker.is_changed()) || picker.is_open() {
        return;
    }
    let now = customize.to_keep();
    if kept.as_ref() == Some(&now) {
        return;
    }
    // What it was loaded as is what the file has.
    let loaded = kept.is_none() && customize.is_added();
    *kept = Some(now.clone());
    if loaded {
        return;
    }
    let Some(path) = crate::map::data_file(FILE) else {
        return;
    };
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| serde_json::to_string(&now).map_err(std::io::Error::other))
        .and_then(|text| {
            let partial = path.with_extension("json.new");
            std::fs::write(&partial, text)?;
            std::fs::rename(&partial, &path)
        });
    if let Err(error) = written {
        warn!("could not keep the colours in {}: {error}", path.display());
    }
}

/// A press on one of the buttons, as it comes up on it.
fn press(
    click: On<Pointer<Click>>,
    buttons: Query<&Press>,
    mut customize: ResMut<Customize>,
    mut menu: ResMut<EditMenu>,
    mut edits: MessageWriter<Edit>,
    mut picker: ResMut<picker::Picker>,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    let Ok(&press) = buttons.get(click.entity) else {
        return;
    };
    match press {
        Press::Begin => customize.begin(),
        Press::Hand => {
            if customize.tool != Tool::Hand {
                (customize.tool, customize.wants) = (Tool::Hand, None);
            }
        }
        Press::Brush => {
            if customize.tool != Tool::Brush {
                customize.ask(Wants::Brush, &mut menu, &mut edits);
            }
        }
        Press::Done => customize.ask(Wants::Off, &mut menu, &mut edits),
        // The colour: taken up, or with it in hand already, chosen.
        Press::Swatch => {
            if customize.ink == Ink::Colour {
                picker.ask();
            } else {
                customize.ink = Ink::Colour;
            }
        }
        Press::Clear => {
            if customize.ink != Ink::Clear {
                customize.ink = Ink::Clear;
            }
        }
    }
}

/// Whether edit mode may go on where you are: in the town, by your plot,
/// and not in your car.
fn by_the_plot(here: Option<Venue>, home: &Home, driving: bool) -> bool {
    here == Some(Venue::Town) && home.near && !driving
}

/// Ends edit mode once you are no longer by your plot ([`by_the_plot`]):
/// whatever was in your hand is put away (`build::stand::leave`).
fn keep_to_the_plot(mut customize: ResMut<Customize>, home: Res<Home>, driving: Res<Driving>, me: Query<&Venue, With<Player>>) {
    if customize.on && !by_the_plot(me.single().ok().copied(), &home, driving.on()) {
        (customize.on, customize.wants) = (false, None);
    }
}

/// Does what was asked for with something in your hand ([`Wants`]), now
/// that the building code has had its say on that: it is done if the
/// thing is down, and dropped if it could not be put down where it was,
/// its menu open on it again.
fn settle(mut customize: ResMut<Customize>, in_hand: Query<(), (With<Mine>, With<Ghost>)>) {
    let Some(wants) = customize.wants else {
        return;
    };
    if in_hand.is_empty() {
        customize.take(wants);
    } else {
        customize.wants = None;
    }
}

/// Which buttons are to be by the left edge: none anywhere but in the
/// town, out of your car; the one while you stand on your plot; and the
/// three for as long as edit mode is on.
fn shown(customize: &Customize, home: &Home, here: Option<Venue>, driving: bool) -> Shown {
    if here != Some(Venue::Town) || driving {
        Shown::Nothing
    } else if customize.on {
        Shown::Three(customize.tool, customize.ink)
    } else if home.on_it {
        Shown::One
    } else {
        Shown::Nothing
    }
}

/// How far the chat steps aside for the buttons, which stand where its
/// lines begin: past the one; and in edit mode past the three and the two
/// round ones beside the brush, whichever tool is in hand, so that it does
/// not move as the brush is taken up.
fn aside(shown: Shown) -> f32 {
    match shown {
        Shown::Nothing => 0.0,
        Shown::One => hud::LEFT_BUTTON + SWATCH_GAP,
        Shown::Three(..) => hud::LEFT_BUTTON + 2.0 * (SWATCH_GAP + SWATCH) + SWATCH_GAP,
    }
}

/// Puts the buttons that are to be there by the left edge ([`shown`]), as
/// that changes and as the edge is put up afresh.
#[allow(clippy::too_many_arguments)]
fn draw(
    customize: Res<Customize>,
    home: Res<Home>,
    driving: Res<Driving>,
    me: Query<&Venue, With<Player>>,
    edges: Query<(Entity, Option<&Children>), With<LeftEdge>>,
    fresh: Query<(), Added<LeftEdge>>,
    pictures: Res<Pictures>,
    mut was: Local<Option<Shown>>,
    mut chat: ResMut<chat::Aside>,
    mut commands: Commands,
) {
    let now = shown(&customize, &home, me.single().ok().copied(), driving.on());
    if *was == Some(now) && fresh.is_empty() {
        return;
    }
    *was = Some(now);
    chat.set_if_neq(chat::Aside(aside(now)));
    for (edge, children) in &edges {
        for child in children.into_iter().flat_map(|children| children.iter()) {
            commands.entity(child).despawn();
        }
        match now {
            Shown::Nothing => {}
            // Where the brush is among the three: the room of the tick left
            // under it.
            Shown::One => {
                commands.spawn((
                    ChildOf(edge),
                    Node {
                        padding: UiRect::bottom(Val::VMin(hud::LEFT_BUTTON + hud::LEFT_GAP)),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![button(Press::Begin, pictures.house.clone(), false)],
                ));
            }
            Shown::Three(tool, ink) => {
                commands.spawn((ChildOf(edge), button(Press::Hand, pictures.house.clone(), tool == Tool::Hand)));
                let row = commands
                    .spawn((
                        ChildOf(edge),
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::VMin(SWATCH_GAP),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .id();
                commands.spawn((ChildOf(row), button(Press::Brush, pictures.brush.clone(), tool == Tool::Brush)));
                if tool == Tool::Brush {
                    commands.spawn((ChildOf(row), swatch(customize.colour.color(), ink == Ink::Colour)));
                    commands.spawn((ChildOf(row), clear(pictures.cross.clone(), ink == Ink::Clear)));
                }
                commands.spawn((ChildOf(edge), button(Press::Done, pictures.tick.clone(), false)));
            }
        }
    }
}

/// One of the buttons: a round one as the others round the screen are,
/// and, `in_hand`, the tool that is in your hand, a little more solid with
/// a ring round it and its picture white.
fn button(press: Press, picture: Handle<Image>, in_hand: bool) -> impl Bundle {
    let colours = crate::ui::ui().colours.hud;
    (
        press,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(hud::LEFT_BUTTON),
            height: Val::VMin(hud::LEFT_BUTTON),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(if in_hand { colours.button_held } else { colours.button }),
        ring(in_hand),
        children![(
            ImageNode::new(picture).with_color(if in_hand { colours.icon } else { colours.button_mark }),
            Node {
                width: Val::Percent(hud::PICTURE),
                height: Val::Percent(hud::PICTURE),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// The ring round one of the buttons, right on its edge: round the tool in
/// hand, and round the button that turns edit mode on while it beats
/// ([`beat`]). On any other it is there with no width, so that it is given
/// one and loses it without the button being made again.
fn ring(on: bool) -> Outline {
    Outline {
        width: Val::VMin(if on { RIM } else { 0.0 }),
        offset: Val::ZERO,
        color: crate::ui::ui().colours.hud.icon,
    }
}

/// The ring round whichever of the two round buttons the brush paints
/// with: a little way out from it, so that it shows round the colour's own
/// rim.
fn chosen_ring(chosen: bool) -> Outline {
    Outline {
        width: Val::VMin(if chosen { RIM } else { 0.0 }),
        offset: Val::VMin(RIM),
        color: crate::ui::ui().colours.hud.icon,
    }
}

/// The round button that shows the colour the brush paints, with a rim
/// so that it stands out whatever the colour; and, `chosen`, a ring round
/// it.
fn swatch(colour: Color, chosen: bool) -> impl Bundle {
    (
        Press::Swatch,
        Swatch,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(SWATCH),
            height: Val::VMin(SWATCH),
            border: UiRect::all(Val::VMin(RIM)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(colour),
        BorderColor::all(crate::ui::ui().colours.hud.icon),
        chosen_ring(chosen),
    )
}

/// The round button beside it with a cross on it: nothing to paint with,
/// which takes paint off.
fn clear(cross: Handle<Image>, chosen: bool) -> impl Bundle {
    let colours = crate::ui::ui().colours.hud;
    (
        Press::Clear,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(SWATCH),
            height: Val::VMin(SWATCH),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(if chosen { colours.button_held } else { colours.button }),
        chosen_ring(chosen),
        children![(
            ImageNode::new(cross).with_color(if chosen { colours.icon } else { colours.button_mark }),
            Node {
                width: Val::Percent(62.0),
                height: Val::Percent(62.0),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// Keeps the round button the colour chosen, as that changes.
fn tint_swatch(customize: Res<Customize>, mut swatches: Query<&mut BackgroundColor, With<Swatch>>) {
    if !customize.is_changed() {
        return;
    }
    let colour = customize.colour.color();
    for mut fill in &mut swatches {
        if fill.0 != colour {
            fill.0 = colour;
        }
    }
}

// ------------------------------------------- whether edit mode is on, shown

/// How many times its own size the button that turns edit mode on is drawn
/// `age` seconds into its beat ([`Pointed`]): its own to begin with,
/// [`BEAT_SWELL`] more at the top of each of the [`BEATS`], its own again
/// between them, and exactly its own from the end on.
fn swell(age: f32) -> f32 {
    if !(0.0..BEAT_SECS).contains(&age) {
        return 1.0;
    }
    let through = age / BEAT_SECS * BEATS;
    1.0 + BEAT_SWELL * (std::f32::consts::PI * through).sin().powi(2)
}

/// Beats the button that turns edit mode on for as long as it is pointed
/// at ([`Pointed`]): it swells and comes back ([`swell`]) with the ring of
/// a tool in hand round it, and at the end is exactly its own size again
/// with no ring. The beat is over at once as edit mode comes on, the button
/// having been found. With no beat on, which is nearly always, nothing is
/// looked at and nothing written.
fn beat(
    time: Res<Time>,
    customize: Res<Customize>,
    mut pointed: ResMut<Pointed>,
    mut buttons: Query<(&Press, &mut UiTransform, &mut Outline)>,
) {
    let Some(age) = pointed.0 else {
        return;
    };
    let age = age + time.delta_secs();
    let over = age >= BEAT_SECS || customize.on();
    let (size, ringed) = if over { (1.0, false) } else { (swell(age), true) };
    for (press, mut place, mut outline) in &mut buttons {
        if *press != Press::Begin {
            continue;
        }
        place.set_if_neq(UiTransform::from_scale(Vec2::splat(size)));
        outline.set_if_neq(ring(ringed));
    }
    pointed.0 = (!over).then_some(age);
}

/// Whether the plate that says edit mode is on is up: for as long as it is
/// on, and never otherwise.
fn plate_up(customize: &Customize) -> bool {
    customize.on()
}

/// Says that edit mode is on, for as long as it is ([`plate_up`]): a plate
/// along the top of the screen, in the middle ([`editing_plate`]; Hajun,
/// 2026-11-23: "maybe adding a text "Editing Mode Enabled" on center top of
/// your screen while its active"). Put up as edit mode comes on and taken
/// down as it goes off, and drawn again from the file as that changes on
/// desktop; nothing is touched otherwise.
fn say_so(
    customize: Res<Customize>,
    changed: Res<UiChanged>,
    pictures: Res<Pictures>,
    plates: Query<Entity, With<EditingPlate>>,
    mut commands: Commands,
) {
    let (wanted, up) = (plate_up(&customize), !plates.is_empty());
    let reworded = changed.is_changed() && !changed.is_added();
    if wanted == up && !reworded {
        return;
    }
    for plate in &plates {
        commands.entity(plate).despawn();
    }
    if wanted {
        commands.spawn(editing_plate(pictures.house.clone()));
    }
}

/// The plate that says edit mode is on, in the row it stands in. The plate
/// is something over the world and no window, so it is drawn as your
/// name's plate is (`hud::name_plate`): that grey, its words white and the
/// size of your name, with the house and pencil of the button that began
/// it before them. The row is where the bar across the top is, as tall as
/// the gear beside your name, so that the plate stands on your name's
/// line, and kept clear of a camera's hole as the bar is
/// (`safe::KeepsClear`), with the plate between two lengths that share
/// what room it leaves, which puts it in the middle ([`room`]). It is
/// under the screen's own buttons and the four on a thing in your hand,
/// and over the names over heads; and neither it nor anything of it takes
/// a press, so that what it lies over is pressed through it.
fn editing_plate(house: Handle<Image>) -> impl Bundle {
    let file = crate::ui::ui();
    let colours = file.colours;
    (
        EditingPlate,
        KeepsClear,
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(hud::EDGE),
            left: Val::VMin(hud::EDGE),
            right: Val::VMin(hud::EDGE),
            min_height: Val::VMin(hud::SMALL_BUTTON),
            align_items: AlignItems::Center,
            ..default()
        },
        GlobalZIndex(-1),
        ZIndex(1),
        Pickable::IGNORE,
        children![
            (NameRoom, room(), Pickable::IGNORE),
            (
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(PLATE_GAP),
                    padding: UiRect::axes(Val::VMin(PLATE_PAD.0), Val::VMin(PLATE_PAD.1)),
                    margin: UiRect::horizontal(Val::VMin(PLATE_CLEAR)),
                    border_radius: BorderRadius::all(Val::VMin(PLATE_ROUND)),
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(colours.hud.name_plate),
                Pickable::IGNORE,
                children![
                    (
                        ImageNode::new(house).with_color(colours.hud.icon),
                        Node {
                            width: Val::VMin(PLATE_PICTURE),
                            height: Val::VMin(PLATE_PICTURE),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ),
                    (
                        Text::new(file.words.bag.editing.clone()),
                        crate::font::sized(PLATE_TEXT),
                        TextColor(colours.text),
                        // On one line, however long: the plate is as wide
                        // as its words.
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        Pickable::IGNORE,
                    ),
                ],
            ),
            (room(), Pickable::IGNORE),
        ],
    )
}

/// One of the two lengths of its row the plate stands between: each takes
/// an equal share of the room the plate leaves, however little, so that
/// the plate is in the middle of the row; but for the one before it, which
/// is no shorter than your name and the gear are wide ([`make_room`]).
fn room() -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: Val::ZERO,
        ..default()
    }
}

/// How long the row before the plate has to be for the plate to begin past
/// your name and the gear beside it, which are `wide` across as they were
/// laid out, in the screen's own pixels, of which `logical` of a window's
/// go to one.
fn name_room(wide: f32, logical: f32) -> Val {
    Val::Px((wide * logical).ceil())
}

/// Keeps the plate clear of your name and the gear beside it
/// (`hud::BarLeft`), which begin where its row does and stand on its line:
/// the length of the row before the plate ([`NameRoom`]) is no shorter
/// than they are wide. So on a screen too narrow for the plate to stand in
/// the middle and be clear of them, a tablet's, or a phone's under a long
/// name, it stands just to the right of them instead. Written on a plate
/// just put up, and as your name's row is laid out another width, the name
/// or its tag having changed; and with no plate up nothing is looked at.
fn make_room(bars: Query<Ref<ComputedNode>, With<hud::BarLeft>>, mut rooms: Query<(Ref<NameRoom>, &mut Node)>) {
    if rooms.is_empty() {
        return;
    }
    let Ok(bar) = bars.single() else {
        return;
    };
    let wanted = name_room(bar.size().x, bar.inverse_scale_factor());
    for (room, mut node) in &mut rooms {
        if (bar.is_changed() || room.is_added()) && node.min_width != wanted {
            node.min_width = wanted;
        }
    }
}

/// The plot whose edges are lit: yours, for as long as edit mode is on.
fn lit(customize: &Customize, home: &Home) -> Option<Yard> {
    home.yard.filter(|_| customize.on())
}

/// Lights the edges of your plot for as long as edit mode is on ([`lit`]):
/// a thing in the world, a line lying on the plot's ground ([`frame`]),
/// drawn in the town alone (`island::show_where_i_am`). It is unlit and
/// see-through, the file's colour by day and by night
/// (`colours.bag.editing_edge`), and drawn brighter than it is written so
/// that it comes out that colour, as the punch's range is (`range::look`).
/// Made as edit mode comes on, made again as your plot becomes another one
/// or the file's colours change on desktop, and taken away as edit mode
/// goes off; nothing is touched otherwise.
fn light_the_plot(
    customize: Res<Customize>,
    home: Res<Home>,
    changed: Res<UiChanged>,
    edges: Query<(Entity, &PlotEdge)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let wanted = lit(&customize, &home);
    let lies = edges.iter().next().map(|(_, edge)| edge.0);
    let recoloured = changed.is_changed() && !changed.is_added();
    if wanted == lies && !recoloured {
        return;
    }
    for (edge, _) in &edges {
        commands.entity(edge).despawn();
    }
    let Some(yard) = wanted else {
        return;
    };
    commands.spawn((
        PlotEdge(yard),
        Name::new("Plot edge"),
        // On the town, and drawn only there.
        Venue::Town,
        Mesh3d(meshes.add(frame(yard.half))),
        MeshMaterial3d(materials.add(crate::range::look(crate::ui::ui().colours.bag.editing_edge))),
        edge_place(&yard),
        Visibility::Hidden,
        NotShadowCaster,
    ));
}

/// Where the line along the edges of `yard` lies: in the yard's own space
/// (`Yard::place`), its middle the plot's and turned as the plot is,
/// [`EDGE_OVER`] over the plot's ground.
fn edge_place(yard: &Yard) -> Transform {
    let mut place = yard.place();
    place.translation.y += EDGE_OVER;
    place
}

/// The line along the edges of a plot whose sides are `half` from its
/// middle, in the plot's own space, flat about its origin: a band
/// [`EDGE_WIDE`] wide just inside the four edges, mitred where two of them
/// meet, facing up. Eight corners and eight triangles whatever the plot:
/// never a mesh with nothing in it, which Bevy is not to be given.
fn frame(half: f32) -> Mesh {
    let inner = (half - EDGE_WIDE).max(0.0);
    // Round the plot: its own four corners, and then the band's inner four.
    let round: [(f32, f32); 4] = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let positions: Vec<[f32; 3]> = [half, inner]
        .into_iter()
        .flat_map(|far| round.map(|(x, z)| [x * far, 0.0, z * far]))
        .collect();
    // Along each side, from the plot's edge in to the band's: two
    // triangles, each wound to face up.
    let mut indices: Vec<u32> = Vec::with_capacity(24);
    for side in 0..4 {
        let next = (side + 1) % 4;
        indices.extend([side, 4 + next, next, side, 4 + side, 4 + next]);
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 8])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 8])
        .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn a_colour_is_white_in_the_middle_of_the_wheel_full_at_its_rim_and_black_at_the_foot_of_the_bar() {
        let colour = |hue: f32, out: f32, bright: f32| Hsv { hue, out, bright }.rgb();
        // The middle is white, whichever way round; darker down the bar.
        assert_eq!(colour(0.0, 0.0, 1.0), [255, 255, 255]);
        assert_eq!(colour(0.7, 0.0, 1.0), [255, 255, 255]);
        assert_eq!(colour(0.3, 0.0, 0.5), [128, 128, 128]);
        assert_eq!(colour(0.3, 1.0, 0.0), [0, 0, 0]);
        // Round the rim: red, yellow, green, cyan, blue, magenta, and red
        // again.
        let rim: Vec<[u8; 3]> = (0..=6).map(|sixth| colour(sixth as f32 / 6.0, 1.0, 1.0)).collect();
        assert_eq!(rim, [[255, 0, 0], [255, 255, 0], [0, 255, 0], [0, 255, 255], [0, 0, 255], [255, 0, 255], [255, 0, 0]]);
        // Half way out it is half way to white; and half as bright, half
        // of each.
        assert_eq!(colour(0.0, 0.5, 1.0), [255, 128, 128]);
        assert_eq!(colour(1.0 / 3.0, 1.0, 0.5), [0, 128, 0]);
        // Between two of them, both.
        assert_eq!(colour(1.0 / 12.0, 1.0, 1.0), [255, 128, 0]);
        // What the wheel shows of a colour is it as bright as it goes.
        let dark = Hsv { hue: 0.6, out: 0.7, bright: 0.2 };
        assert_eq!(dark.full().rgb(), colour(0.6, 0.7, 1.0));
    }

    fn home(on_it: bool, near: bool) -> Home {
        Home::for_tests(on_it, near)
    }

    #[test]
    fn the_button_is_there_on_your_plot_and_its_three_for_as_long_as_edit_mode_is_on() {
        let town = Some(Venue::Town);
        let mut customize = Customize::default();
        // Off your plot, nothing; on it, the one; in your car or anywhere
        // but the town, nothing.
        assert_eq!(shown(&customize, &home(false, true), town, false), Shown::Nothing);
        assert_eq!(shown(&customize, &home(true, true), town, false), Shown::One);
        assert_eq!(shown(&customize, &home(true, true), town, true), Shown::Nothing);
        assert_eq!(shown(&customize, &home(true, true), Some(Venue::Home), false), Shown::Nothing);
        // The one stands where the chat's lines begin, which step aside of
        // it; and of nothing where there is none.
        assert!(aside(Shown::One) > hud::LEFT_BUTTON);
        assert_eq!(aside(Shown::Nothing), 0.0);
        // Pressed: the three, the hand in hand; and a step off the plot
        // they are there still.
        customize.begin();
        assert!(customize.moving() && !customize.painting());
        let hand = Shown::Three(Tool::Hand, Ink::Colour);
        assert_eq!(shown(&customize, &home(true, true), town, false), hand);
        assert_eq!(shown(&customize, &home(false, true), town, false), hand);
        // The brush: the same three, and the chat as far aside as it was,
        // past the two round buttons beside the brush.
        customize.take(Wants::Brush);
        assert!(customize.painting() && !customize.moving() && !customize.clearing());
        let brush = Shown::Three(Tool::Brush, Ink::Colour);
        assert_eq!(shown(&customize, &home(true, true), town, false), brush);
        assert_eq!(aside(brush), aside(hand));
        assert!(aside(hand) > hud::LEFT_BUTTON + 2.0 * SWATCH);
        // The cross beside the colour: the brush takes paint off, until the
        // colour is taken up again.
        customize.ink = Ink::Clear;
        assert!(customize.clearing() && customize.painting());
        assert_eq!(shown(&customize, &home(true, true), town, false), Shown::Three(Tool::Brush, Ink::Clear));
        customize.ink = Ink::Colour;
        // The tick: the one again, and nothing of yours answers a tap.
        customize.take(Wants::Off);
        assert!(!customize.on() && !customize.moving() && !customize.painting());
        assert_eq!(shown(&customize, &home(true, true), town, false), Shown::One);
        // Something brought out of your bag turns it on, with the hand,
        // whatever tool was in it last.
        customize.begin();
        assert!(customize.moving());
    }

    #[test]
    fn the_colours_painted_with_are_kept_newest_first_and_no_more_than_a_row() {
        let colour = |n: usize| Hsv {
            hue: n as f32 / 10.0,
            out: 1.0,
            bright: 1.0,
        };
        let mut customize = Customize::default();
        // None to begin with (Hajun: "in the beginning it can be empty").
        assert_eq!(customize.used().count(), 0);
        assert!(!customize.last_used());
        // Painted with: kept, and the same again is nothing new.
        customize.choose(colour(1));
        customize.note_used();
        assert!(customize.last_used());
        customize.note_used();
        assert_eq!(customize.used().collect::<Vec<_>>(), [colour(1)]);
        // Another, and then the first again: newest first, each once.
        customize.choose(colour(2));
        assert!(!customize.last_used());
        customize.note_used();
        customize.choose(colour(1));
        customize.note_used();
        assert_eq!(customize.used().collect::<Vec<_>>(), [colour(1), colour(2)]);
        // More than a row of them: the oldest go.
        for n in 3..=9 {
            customize.choose(colour(n));
            customize.note_used();
        }
        let kept: Vec<Hsv> = customize.used().collect();
        assert_eq!(kept.len(), USED);
        assert_eq!((kept[0], kept[USED - 1]), (colour(9), colour(3)));
        // Kept in the file with the colour chosen, and read back the same:
        // edit mode itself is not, nor the tool.
        customize.take(Wants::Brush);
        customize.on = true;
        let text = serde_json::to_string(&customize.to_keep()).expect("written");
        let back = Customize::remembering(serde_json::from_str(&text).ok());
        assert_eq!((back.chosen(), back.used().collect::<Vec<_>>()), (customize.chosen(), kept));
        assert!(!back.on() && !back.painting());
        // With no file, or one of another kind, or one that says nonsense:
        // as it begins, and nothing that is no colour.
        assert_eq!(Customize::remembering(None), Customize::default());
        let other = Kept {
            version: VERSION + 1,
            ..customize.to_keep()
        };
        assert_eq!(Customize::remembering(Some(other)), Customize::default());
        let nonsense = Kept {
            version: VERSION,
            colour: Hsv {
                hue: f32::NAN,
                out: 7.0,
                bright: -1.0,
            },
            used: Vec::new(),
            hinted: 0,
        };
        let put_right = Customize::remembering(Some(nonsense)).chosen();
        assert_eq!((put_right.hue, put_right.out, put_right.bright), (0.0, 1.0, 0.0));
    }

    #[test]
    fn how_a_thing_is_moved_is_said_a_few_times_and_never_again() {
        // (While it is said at all: the words are turned off, [`HINTS`],
        // and this is what they would go by, turned back on.)
        let mut customize = Customize::default();
        for _ in 0..MOST_HINTS {
            assert!(customize.may_hint());
            customize.note_hint();
        }
        assert!(!customize.may_hint());
        // Kept with the colours, so that it is never again on this device;
        // and a file from before it was kept reads as never said.
        let back = Customize::remembering(serde_json::from_str(&serde_json::to_string(&customize.to_keep()).unwrap()).ok());
        assert!(!back.may_hint());
        let before: Kept = serde_json::from_str(r#"{"version":1,"colour":{"hue":0.6,"out":0.7,"bright":0.9},"used":[]}"#).unwrap();
        assert!(Customize::remembering(Some(before)).may_hint());
    }

    #[test]
    fn edit_mode_ends_once_you_are_no_longer_by_your_plot() {
        let mut world = World::new();
        world.insert_resource(Customize::default());
        world.insert_resource(Driving::for_tests(false));
        world.insert_resource(home(true, true));
        let you = world.spawn((Player, Venue::Town)).id();
        world.resource_mut::<Customize>().begin();
        let on = |world: &mut World| {
            world.run_system_once(keep_to_the_plot).expect("kept");
            world.resource::<Customize>().on()
        };
        // On your plot, and a step off it: on.
        assert!(on(&mut world));
        world.insert_resource(home(false, true));
        assert!(on(&mut world));
        // Well off it: off, and it stays off when you come back.
        world.insert_resource(home(false, false));
        assert!(!on(&mut world));
        world.insert_resource(home(true, true));
        assert!(!on(&mut world));
        // In your car: off.
        world.resource_mut::<Customize>().begin();
        world.insert_resource(Driving::for_tests(true));
        assert!(!on(&mut world));
        // Out of the town: off.
        world.insert_resource(Driving::for_tests(false));
        world.resource_mut::<Customize>().begin();
        *world.get_mut::<Venue>(you).expect("a venue") = Venue::Arena;
        assert!(!on(&mut world));
    }

    #[test]
    fn what_is_asked_for_with_a_thing_in_your_hand_waits_for_it_to_be_down() {
        let mut world = World::new();
        world.insert_resource(Customize::default());
        world.resource_mut::<Customize>().begin();
        // The brush asked for with a thing in your hand: not yet.
        world.resource_mut::<Customize>().wants = Some(Wants::Brush);
        let ghost = world.spawn((Mine, Ghost)).id();
        world.run_system_once(settle).expect("settled");
        // It could not be put down: the hand still, and nothing waiting.
        assert!(world.resource::<Customize>().moving());
        assert_eq!(world.resource::<Customize>().wants, None);
        // Asked again, and this time it is down: the brush.
        world.resource_mut::<Customize>().wants = Some(Wants::Brush);
        world.entity_mut(ghost).remove::<Ghost>();
        world.run_system_once(settle).expect("settled");
        assert!(world.resource::<Customize>().painting());
        // The tick, with nothing in your hand: off.
        world.resource_mut::<Customize>().wants = Some(Wants::Off);
        world.run_system_once(settle).expect("settled");
        assert!(!world.resource::<Customize>().on());
    }

    #[test]
    fn a_plate_says_edit_mode_is_on_for_as_long_as_it_is_and_takes_no_press() {
        // Up with edit mode on, whichever tool is in hand, and never with
        // it off.
        let mut customize = Customize::default();
        assert!(!plate_up(&customize));
        customize.begin();
        assert!(plate_up(&customize));
        customize.take(Wants::Brush);
        assert!(plate_up(&customize));
        customize.take(Wants::Off);
        assert!(!plate_up(&customize));
        // Put up as edit mode comes on.
        let mut world = World::new();
        world.insert_resource(Customize::default());
        world.init_resource::<UiChanged>();
        world.insert_resource(Pictures {
            house: Handle::default(),
            brush: Handle::default(),
            tick: Handle::default(),
            cross: Handle::default(),
        });
        let plates = |world: &mut World| -> Vec<Entity> {
            world.run_system_once(say_so).expect("said");
            world.query_filtered::<Entity, With<EditingPlate>>().iter(world).collect()
        };
        assert!(plates(&mut world).is_empty());
        world.resource_mut::<Customize>().begin();
        let up = plates(&mut world);
        assert_eq!(up.len(), 1);
        // It says so in the file's words (Hajun's: "Editing Mode Enabled"),
        // on the grey of your name's plate.
        let said: Vec<String> = world.query::<&Text>().iter(&world).map(|text| text.0.clone()).collect();
        assert_eq!(said, [crate::ui::ui().words.bag.editing.clone()]);
        let grey = crate::ui::ui().colours.hud.name_plate;
        assert!(world.query::<&BackgroundColor>().iter(&world).any(|fill| fill.0 == grey));
        // Nothing of it takes a press: what it lies over is pressed
        // through it.
        let mut parts = world.query_filtered::<(Option<&Pickable>, Has<TakesPress>), With<Node>>();
        assert_eq!(parts.iter(&world).count(), 6);
        for (pickable, takes_press) in parts.iter(&world) {
            assert_eq!(pickable, Some(&Pickable::IGNORE));
            assert!(!takes_press);
        }
        // Left as it is from frame to frame, and as the brush is taken up.
        assert_eq!(plates(&mut world), up);
        world.resource_mut::<Customize>().take(Wants::Brush);
        assert_eq!(plates(&mut world), up);
        // Taken down as edit mode goes off, all of it.
        world.resource_mut::<Customize>().take(Wants::Off);
        assert!(plates(&mut world).is_empty());
        assert_eq!(world.query::<&Text>().iter(&world).count(), 0);
    }

    #[test]
    fn the_plate_begins_past_your_name_where_the_screen_is_too_narrow_for_the_middle() {
        // As wide as your name and the gear were laid out, in a window's
        // own pixels, and never a part of one short.
        assert_eq!(name_room(300.0, 0.5), Val::Px(150.0));
        assert_eq!(name_room(301.0, 1.0 / 3.0), Val::Px(101.0));
        assert_eq!(name_room(0.0, 1.0), Val::Px(0.0));
        // The two lengths the plate stands between share what it leaves
        // alike, which is what puts it in the middle.
        let (before, after) = (room(), room());
        assert_eq!((before.flex_grow, before.flex_basis), (after.flex_grow, after.flex_basis));
        assert!(before.flex_grow > 0.0 && before.flex_basis == Val::ZERO);
        // A plate just put up: the length before it is no shorter than
        // your name's row is wide, and the one after it is left alone.
        let mut world = World::new();
        let row = ComputedNode {
            size: Vec2::new(300.0, 60.0),
            inverse_scale_factor: 0.5,
            ..default()
        };
        let bar = world.spawn((hud::BarLeft, row)).id();
        let before = world.spawn((NameRoom, room())).id();
        let after = world.spawn(room()).id();
        world.run_system_once(make_room).expect("made");
        assert_eq!(world.get::<Node>(before).expect("a node").min_width, Val::Px(150.0));
        assert_eq!(world.get::<Node>(after).expect("a node").min_width, Node::default().min_width);
        // Your name another width, its tag having come: so is the room.
        world.get_mut::<ComputedNode>(bar).expect("laid out").size.x = 420.0;
        world.run_system_once(make_room).expect("made");
        assert_eq!(world.get::<Node>(before).expect("a node").min_width, Val::Px(210.0));
    }

    #[test]
    fn the_button_pointed_at_swells_three_times_and_is_exactly_its_own_size_again() {
        use std::time::Duration;
        // Its own size to begin with, and exactly its own at the end, after
        // it, and between two beats.
        assert_eq!(swell(0.0), 1.0);
        assert_eq!(swell(BEAT_SECS), 1.0);
        assert_eq!(swell(BEAT_SECS + 3.0), 1.0);
        assert_eq!(swell(-0.5), 1.0);
        assert!((swell(BEAT_SECS / BEATS) - 1.0).abs() < 1e-4);
        // Never smaller than itself, about a third bigger at the most,
        // and up and back three times, in a little over a second.
        let sizes: Vec<f32> = (0..=1200).map(|step| swell(BEAT_SECS * step as f32 / 1200.0)).collect();
        assert!(sizes.iter().all(|&size| size >= 1.0));
        let most = sizes.iter().copied().fold(0.0, f32::max);
        assert!((most - (1.0 + BEAT_SWELL)).abs() < 1e-3 && (most - 1.35).abs() < 0.06, "{most}");
        let half = 1.0 + BEAT_SWELL * 0.5;
        let ups = sizes.windows(2).filter(|pair| pair[0] < half && pair[1] >= half).count();
        assert_eq!((ups, BEATS), (3, 3.0));
        assert!((1.0..=1.5).contains(&BEAT_SECS));
        // Pointed at: from its start. Pointed at again while it beats: on
        // from where it is in its beat, the size it was, with all three
        // still to come from the start of that one.
        let mut pointed = Pointed::default();
        pointed.begin();
        assert_eq!(pointed, Pointed(Some(0.0)));
        pointed = Pointed(Some(0.95));
        pointed.begin();
        let age = pointed.0.expect("beating");
        assert!((0.0..BEAT_SECS / BEATS).contains(&age), "{age}");
        assert!((swell(age) - swell(0.95)).abs() < 1e-4);
        // The button that turns edit mode on, and another beside it.
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(Customize::default());
        world.init_resource::<Pointed>();
        let one = world.spawn((Press::Begin, UiTransform::default(), ring(false))).id();
        let other = world.spawn((Press::Done, UiTransform::default(), ring(false))).id();
        let step = |world: &mut World, secs: f32| {
            world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(secs));
            world.run_system_once(beat).expect("beaten");
        };
        let size = |world: &World, button: Entity| *world.get::<UiTransform>(button).expect("a button");
        let ringed = |world: &World, button: Entity| *world.get::<Outline>(button).expect("a button");
        let written = |world: &World, button: Entity| {
            let entity = world.entity(button);
            (entity.get_change_ticks::<UiTransform>().map(|ticks| ticks.changed), entity.get_change_ticks::<Outline>().map(|ticks| ticks.changed))
        };
        // Not pointed at: nothing of it is written, frame after frame.
        let before = written(&world, one);
        step(&mut world, 0.1);
        step(&mut world, 0.1);
        assert_eq!(written(&world, one), before);
        assert_eq!((size(&world, one), ringed(&world, one)), (UiTransform::IDENTITY, ring(false)));
        // Pointed at: a tenth of a second on it is bigger, the same across
        // as down, and ringed as the tool in hand is; the other is as it
        // was, never written.
        let beside = written(&world, other);
        world.resource_mut::<Pointed>().begin();
        step(&mut world, 0.1);
        let swollen = size(&world, one).scale;
        assert!(swollen.x > 1.1 && swollen.x == swollen.y, "{swollen}");
        assert!((swollen.x - swell(0.1)).abs() < 1e-4, "{swollen}");
        assert_eq!(ringed(&world, one), ring(true));
        assert!(ring(true).width != ring(false).width);
        // At its end: exactly its own size, no ring, and over, with
        // nothing written from then.
        for _ in 0..12 {
            step(&mut world, 0.1);
        }
        assert_eq!((size(&world, one), ringed(&world, one)), (UiTransform::IDENTITY, ring(false)));
        assert_eq!(*world.resource::<Pointed>(), Pointed(None));
        let after = written(&world, one);
        step(&mut world, 0.1);
        assert_eq!(written(&world, one), after);
        assert_eq!(written(&world, other), beside);
        assert_eq!((size(&world, other), ringed(&world, other)), (UiTransform::IDENTITY, ring(false)));
        // Edit mode turned on while it beats, the button found: over at
        // once.
        world.resource_mut::<Pointed>().begin();
        step(&mut world, 0.1);
        assert!(size(&world, one).scale.x > 1.1);
        world.resource_mut::<Customize>().begin();
        step(&mut world, 0.1);
        assert_eq!((size(&world, one), ringed(&world, one)), (UiTransform::IDENTITY, ring(false)));
        assert_eq!(*world.resource::<Pointed>(), Pointed(None));
    }

    /// Every corner of `mesh`, and the three corners of each of its
    /// triangles.
    fn corners(mesh: &Mesh) -> (Vec<Vec3>, Vec<[Vec3; 3]>) {
        let points: Vec<Vec3> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .expect("positions")
            .iter()
            .map(|&point| Vec3::from(point))
            .collect();
        let order: Vec<usize> = mesh.indices().expect("indices").iter().collect();
        let triangles = order.chunks(3).map(|tri| [points[tri[0]], points[tri[1]], points[tri[2]]]).collect();
        (points, triangles)
    }

    fn yard(front: Vec2) -> Yard {
        Yard {
            middle: Vec2::new(25.0, 90.0),
            ground: 0.27,
            front,
            half: 12.0,
        }
    }

    #[test]
    fn the_line_along_your_plot_s_edges_lies_inside_it_on_its_edges_whichever_way_it_faces() {
        for front in [Vec2::NEG_Y, Vec2::X, Vec2::Y, Vec2::new(0.6, 0.8), Vec2::new(-0.8, -0.6)] {
            let yard = yard(front);
            let right = Vec2::new(-front.y, front.x);
            let place = edge_place(&yard);
            let (points, triangles) = corners(&frame(yard.half));
            assert_eq!((points.len(), triangles.len()), (8, 8));
            // Every corner of it is on the plot, just over its ground: four
            // of them the plot's own corners, and four the same way in from
            // them, the width of the line.
            let mut on_the_edge = 0;
            for point in &points {
                let at = place.transform_point(*point);
                assert!((at.y - (yard.ground + EDGE_OVER)).abs() < 1e-4, "{front}: {at}");
                assert!(yard.holds(at.xz(), 1e-3), "{front}: {at} is off the plot");
                let off = at.xz() - yard.middle;
                let (along, across) = (off.dot(front).abs(), off.dot(right).abs());
                assert!((along - across).abs() < 1e-3, "{front}: {at}");
                if (along - yard.half).abs() < 1e-3 {
                    on_the_edge += 1;
                } else {
                    assert!((along - (yard.half - EDGE_WIDE)).abs() < 1e-3, "{front}: {at}");
                }
            }
            assert_eq!(on_the_edge, 4, "{front}");
            // Every triangle of it faces up, and lies in the band along the
            // edges, none of it over the middle of the plot; and the eight
            // are all of that band.
            let mut area = 0.0;
            for corners in triangles {
                let [a, b, c] = corners.map(|corner| place.transform_point(corner));
                let up = (b - a).cross(c - a);
                assert!(up.y > 0.0 && up.xz().length() < 1e-3, "{front}: {up}");
                area += up.y * 0.5;
                let off = (a + b + c).xz() / 3.0 - yard.middle;
                let out = off.dot(front).abs().max(off.dot(right).abs());
                assert!(out > yard.half - EDGE_WIDE && out < yard.half, "{front}: {out} m out");
            }
            let band = (2.0 * yard.half).powi(2) - (2.0 * (yard.half - EDGE_WIDE)).powi(2);
            assert!((area - band).abs() < 0.05, "{front}: {area} of {band}");
        }
        // A thin line, lying lower than anything is stepped onto. And a
        // plot too small for it still has corners to draw, never none.
        assert!(EDGE_WIDE > 0.0 && EDGE_WIDE < 1.0 && EDGE_OVER > 0.0 && EDGE_OVER < 0.1);
        assert_eq!(frame(0.1).count_vertices(), 8);
        assert_eq!(frame(0.0).count_vertices(), 8);
    }

    #[test]
    fn the_edges_of_your_plot_are_lit_for_as_long_as_edit_mode_is_on() {
        let plot = yard(Vec2::new(0.6, 0.8));
        // Nothing with edit mode off, nor before it is known which plot is
        // yours.
        let (mut customize, mut home) = (Customize::default(), home(true, true));
        assert_eq!(lit(&customize, &home), None);
        customize.begin();
        assert_eq!(lit(&customize, &home), None);
        home.yard = Some(plot);
        assert_eq!(lit(&customize, &home), Some(plot));
        customize.take(Wants::Brush);
        assert_eq!(lit(&customize, &home), Some(plot));
        customize.take(Wants::Off);
        assert_eq!(lit(&customize, &home), None);
        // Made as edit mode comes on.
        let mut world = World::new();
        world.insert_resource(Customize::default());
        world.insert_resource(home);
        world.init_resource::<UiChanged>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let edges = |world: &mut World| -> Vec<Entity> {
            world.run_system_once(light_the_plot).expect("lit");
            world.query_filtered::<Entity, With<PlotEdge>>().iter(world).collect()
        };
        assert!(edges(&mut world).is_empty());
        world.resource_mut::<Customize>().begin();
        let lies = edges(&mut world);
        assert_eq!(lies.len(), 1);
        // A thing of the town's, not drawn until it is known you are
        // there, lying on your plot, and casting no shadow.
        let edge = world.entity(lies[0]);
        assert_eq!(edge.get::<Venue>(), Some(&Venue::Town));
        assert_eq!(edge.get::<Visibility>(), Some(&Visibility::Hidden));
        assert_eq!(edge.get::<Transform>(), Some(&edge_place(&plot)));
        assert!(edge.contains::<NotShadowCaster>());
        // Unlit and truly see-through, as see-through as the file says,
        // and brighter than the file's colour, to come out that colour.
        let ink = crate::ui::ui().colours.bag.editing_edge;
        let paint = edge.get::<MeshMaterial3d<StandardMaterial>>().expect("a material");
        let look = world.resource::<Assets<StandardMaterial>>().get(&paint.0).expect("its material");
        assert!(look.unlit && look.alpha_mode == AlphaMode::Blend);
        assert!((look.base_color.alpha() - ink.alpha()).abs() < 1e-6 && ink.alpha() > 0.0 && ink.alpha() < 1.0);
        assert!(look.base_color.to_linear().red > ink.to_linear().red);
        // Its mesh has corners in it.
        let sheet = edge.get::<Mesh3d>().expect("a mesh");
        assert_eq!(world.resource::<Assets<Mesh>>().get(&sheet.0).expect("its mesh").count_vertices(), 8);
        // Left as it is from frame to frame, and as the brush is taken up.
        assert_eq!(edges(&mut world), lies);
        world.resource_mut::<Customize>().take(Wants::Brush);
        assert_eq!(edges(&mut world), lies);
        // Your plot another one: made again, there.
        let other = Yard {
            middle: Vec2::new(-40.0, 10.0),
            front: Vec2::X,
            ..plot
        };
        world.resource_mut::<Home>().yard = Some(other);
        let moved = edges(&mut world);
        assert_eq!(moved.len(), 1);
        assert_ne!(moved, lies);
        assert_eq!(world.get::<Transform>(moved[0]), Some(&edge_place(&other)));
        // Taken away as edit mode goes off.
        world.resource_mut::<Customize>().take(Wants::Off);
        assert!(edges(&mut world).is_empty());
    }
}
