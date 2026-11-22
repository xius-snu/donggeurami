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

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Player;
use crate::build::stand::{Home, Mine};
use crate::build::{BuildSystems, Ghost};
use crate::chat;
use crate::drive::Driving;
use crate::editor::{self, Edit, EditMenu};
use crate::hud::{self, LeftEdge, TakesPress};
use crate::island::Venue;

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
/// How many times, ever, someone tapping at a thing of theirs with edit
/// mode off is told how to move it (`stand::hint_at_edit_mode`; Hajun: "just
/// for the first few times"). Mine.
const HINTS: u8 = 3;

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
    /// times only.
    pub(crate) fn may_hint(&self) -> bool {
        self.hinted < HINTS
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

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(Customize::remembering(load()))
        .add_observer(press)
        .add_systems(Startup, draw_pictures)
        .add_systems(
            Update,
            (
                keep_to_the_plot.before(editor::EditSystems),
                settle.after(BuildSystems),
                (draw, tint_swatch, keep).chain().after(settle),
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
        Outline {
            width: Val::VMin(if in_hand { RIM } else { 0.0 }),
            offset: Val::ZERO,
            color: colours.icon,
        },
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
        let mut customize = Customize::default();
        for _ in 0..HINTS {
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
}
