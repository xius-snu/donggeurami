//! Bridge Wars' second button, and the things that go on it (since
//! 2026-11-12; Hajun's `bridgewarsgameevent.txt`: "Utilities (these are
//! items that goes on your 2nd empty ui button ...)"): the fireball and the
//! Bridge Builder, which are aimed, and the three potions, which are drunk.
//! What each does is the rules' (`roundtown_net::bridge_wars`), carried out
//! by this device offline and by the server online (`arena::use_things`);
//! this is the button: what it shows, and what a finger on it does.
//!
//! **What it shows** ([`fill_slot`]): the picture of what is on it, tinted
//! as on its card in the shop (`market::look_of`), how many of it in its
//! corner, and, while a potion's effect lasts, the seconds it still has
//! over the button ("45s"). With none of it left and its effect still
//! lasting, the picture is faint and the time still counts; once that is
//! over the button is empty again (the rules empty it, `Outfits::tick`).
//!
//! **What is aimed** ([`aim`]; "rather than a push button, its a hold &
//! drag / letgo to shoot or cancel, and shows a long narrow rectangular hit
//! range indicator similar to Brock in brawlstars"): press the button and
//! drag, in any direction, and a narrow band lies on the ground from you
//! that way, as far as a fireball flies, or along the first of a builder's
//! bridge and as wide; let go and it is thrown, or sent, that way. Dragged
//! back to where the finger went down ([`AIM_DEAD`]), the band is gone,
//! and letting go there cancels. A quick tap that never dragged uses it
//! the way you face ([`QUICK_TAP`]; mine, as Brawl Stars fires at a tap:
//! a button that did nothing when tapped would seem broken). Up on the
//! screen is north, the camera being fixed over you. Q, on desktop, uses
//! it the way you face.
//!
//! **What is drunk** ([`drink`]; "push it once and it shows 1.6s timer
//! running to 0 in front of ur character while ur movement is slowed 50%
//! while drinking ... if attacked or press the button again before finish,
//! it cancels"): a press, or Q, begins it (`crate::Drinking` on your body:
//! half your pace, no punch, the hand at the mouth, the sword put away,
//! `gear`), with the time left written in front of you in tenths
//! ([`show_drink`]); a press again, or any hit that takes health off you,
//! or dying, stops it with nothing drunk; left to run out, the potion is
//! drunk (`arena::Used::Potion`). All three potions are drunk this way
//! (the healing one too, since the file of 2026-11-12).
//!
//! All of it is this device's own: nobody else sees the band, the count
//! in front of you, or your hand at your mouth.
//!
//! **There are five such buttons** (since 2026-11-13, Hajun's
//! `new4morebuttonlayout.jpg`: "4 more buttons that are hidden by default
//! until either u press the ... toggle button or u buy an item that fills
//! up another slot", and "it fills up the slots according to buy order so
//! u can buy more than 1 item and hold it at once"): the second button,
//! and four more on a ring round it and the others (`hud::utility_slot`),
//! each with one kind of thing on it, in the order the kinds were bought
//! (`Kit::slots`). Each works as the one button did: its own picture and
//! count, a potion's time over its own button, a press or a hold and a
//! drag by what is on it. **A button is there while something is on it,
//! and not otherwise** (since 2026-11-14, Hajun: "i dont think the UI ...
//! button is necessary. just remove it and keep the default as the only
//! option - only showing the button if the slot is filled and not showing
//! empty buttons"). Until then the second button was always there, an
//! empty outline with nothing on it, and three dots over the buttons
//! ([`MoreSlots`], `hud::MoreButton`) showed the four more empty as well
//! and put them away again; the dots are not put up any more
//! (`hud::DOTS`), and what they turned on and off stays off. One thing
//! at a time: while a finger
//! aims, no other button aims or drinks; with a potion at your mouth, a
//! press on its button puts it down and one on another potion's does
//! nothing. Q is the second button on desktop, and 1 to 5 the five.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use roundtown_net::bridge_wars::{
    DRINK_SECS, DYNAMITE_FURTHEST, DYNAMITE_HAND, DYNAMITE_NEAREST, DYNAMITE_RADIUS, DYNAMITE_TAP, FIREBALL_AHEAD, FIREBALL_RADIUS,
    FIREBALL_RANGE, Lob, lob_track,
};
use roundtown_net::{Stack, UTILITY_SLOTS, Utility};

use crate::arena::{ArenaSystems, FightTime, Fighting, Health, InArena, Outfit, Used};
use crate::bridge;
use crate::hud::{self, DialogUp, MoreButton, Pictures, UtilityCount, UtilityPicture, UtilitySlot, UtilityTimer};
use crate::island::{Islands, Venue};
use crate::market;
use crate::range;
use crate::ui::{UiChanged, ui};
use crate::{Dead, Drinking, LAND_STEP_UP, PLAYER_RADIUS, Player, ThirdPersonCamera, camera_ahead};

/// How far the finger has to be dragged from where it went down for the
/// button to be aiming, as a share of the short side of the screen: about
/// a third of the button.
const AIM_DEAD: f32 = 4.0;
/// How short a press that never dragged is a tap, in seconds, which uses
/// the thing the way you face.
const QUICK_TAP: f32 = 0.3;
/// How long after using something aimed before the next can be, in
/// seconds: longer than the server takes to say one is gone, so that a
/// second is never asked for with none left.
const AIM_REST: f32 = 0.5;
/// The band a fireball's aim lays on the ground, in metres: as wide as the
/// ball and a little, from where it sets out to where it bursts.
const FIREBALL_BAND: f32 = 2.0 * FIREBALL_RADIUS + 0.2;
/// And a builder's: as wide as its bridge, and this long, which is off the
/// screen whichever way it points.
const BUILDER_BAND: f32 = 2.0 * bridge::HALF_WIDTH;
const BUILDER_BAND_LENGTH: f32 = 30.0;
/// How far the finger has to be dragged from where it went down to throw
/// a stick of dynamite as far as one can be thrown, as a share of the
/// short side of the screen: from [`AIM_DEAD`], where it is thrown as near
/// as one can be, the further the finger the further the throw ("with
/// ability to change the total distance with the magnitude of the
/// hold&drag&letgo"). About a button and a half.
const AIM_FULL: f32 = 22.0;
/// How wide the band a stick of dynamite is aimed with is, in metres:
/// about as wide as the two sticks; and how many stretches its arch is
/// drawn in.
const DYNAMITE_BAND: f32 = 0.6;
const DYNAMITE_BAND_PIECES: usize = 24;
/// The cross in the middle of the ring where a stick will come down: how
/// far each of its two strokes reaches from its middle, and how thick
/// they are, either side of their own middles, in metres. About a metre
/// across, a fiftieth of the screen's height under the camera over you
/// for each stroke's thickness: small, and still to be seen.
const LANDING_CROSS: (f32, f32) = (0.5, 0.085);
/// How far up from the ground you have to be to be in the air, in metres:
/// a builder is sent off from the ground.
const IN_THE_AIR: f32 = 0.3;
/// The time left of a drink, written in front of you: how big, in shares
/// of the short side, as the metres of bridge are written; and how far in
/// front of you, and over your feet, in metres, nearer you than those.
const DRINK_TEXT: f32 = 3.6;
const DRINK_AHEAD: f32 = 1.35;
const DRINK_OVER: f32 = 0.25;
/// How faint the picture on the button is with none of it left, its
/// effect still lasting: this share of its alpha.
const NONE_LEFT: f32 = 0.38;

/// The keys that use each button on desktop: Q the second button, as when
/// it was the only one, and 1 to 5 the five.
const SLOT_KEYS: [&[KeyCode]; UTILITY_SLOTS] = [
    &[KeyCode::KeyQ, KeyCode::Digit1],
    &[KeyCode::Digit2],
    &[KeyCode::Digit3],
    &[KeyCode::Digit4],
    &[KeyCode::Digit5],
];

/// One of the buttons was pressed: which (`Kit::slots`), by this finger
/// or the mouse.
#[derive(Message, Clone, Copy)]
struct SlotPressed(usize, PointerId);

/// Whether the four more buttons are shown though nothing is on them:
/// what the three dots turn on and off (`hud::MoreButton`). One with
/// something on it is shown either way.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct MoreSlots(pub(crate) bool);

/// What your hand is doing at the buttons.
#[derive(Resource, Default)]
struct Hand {
    /// The finger holding one down to aim, if one is.
    hold: Option<Hold>,
    /// How long until something aimed can be used again, in seconds.
    rest: f32,
    /// The potion at your mouth, and how much health you had when last
    /// looked at: any less, and you were hit.
    potion: Option<Utility>,
    health: f32,
}

/// A finger, or the mouse, holding a button down to aim: which, and what
/// is on the button it holds; where it went down, once that is read, in
/// the window's own pixels; for how long; whether it has ever been
/// dragged out far enough to aim; which way it aims now, as a plank
/// heads, if it is out that far; and how far out it is dragged, from 0
/// where it begins to aim to 1 at its furthest, which is how far a stick
/// of dynamite is thrown.
#[derive(Clone, Copy, Debug)]
struct Hold {
    pointer: PointerId,
    utility: Utility,
    from: Option<Vec2>,
    age: f32,
    out: bool,
    aim: Option<f32>,
    reach: f32,
}

impl Hold {
    fn new(pointer: PointerId, utility: Utility) -> Self {
        Self {
            pointer,
            utility,
            from: None,
            age: 0.0,
            out: false,
            aim: None,
            reach: 0.0,
        }
    }

    /// The finger is `at` now, `dt` seconds on: aiming the way it has been
    /// dragged from where it went down, across a screen whose up is
    /// `ahead` on the ground, once it is `dead` or more from there; and
    /// reaching the further the further it is dragged, to its furthest at
    /// `full` from there.
    fn drag(&mut self, at: Vec2, dt: f32, dead: f32, full: f32, ahead: Vec3) {
        let drag = at - *self.from.get_or_insert(at);
        self.age += dt;
        self.aim = (drag.length() >= dead).then(|| heading_of_drag(drag, ahead));
        self.out |= self.aim.is_some();
        self.reach = ((drag.length() - dead) / (full - dead).max(1.0)).clamp(0.0, 1.0);
    }

    /// What letting go does, with you facing `facing`: the heading to use
    /// the thing along, the way it aimed, or, a quick tap that never
    /// dragged, the way you face; or nothing, cancelled.
    fn let_go(&self, facing: f32) -> Option<f32> {
        self.aim.or((!self.out && self.age <= QUICK_TAP).then_some(facing))
    }

    /// How far a stick of dynamite let go of now is thrown, in metres:
    /// aimed, from as near as one can be to as far, by how far the finger
    /// is dragged; a quick tap that never dragged, a few metres
    /// ([`DYNAMITE_TAP`]).
    fn far(&self) -> f32 {
        if self.aim.is_some() {
            DYNAMITE_NEAREST + (DYNAMITE_FURTHEST - DYNAMITE_NEAREST) * self.reach
        } else {
            DYNAMITE_TAP
        }
    }
}

/// The heading, as a plank heads (radians from the game's -Z about the
/// upright, a turn to the left the greater), of a finger dragged `drag`
/// across the screen, in pixels, down the screen being the greater y,
/// when the top of the screen is `ahead` on the ground.
fn heading_of_drag(drag: Vec2, ahead: Vec3) -> f32 {
    let right = ahead.cross(Vec3::Y);
    let way = right * drag.x - ahead * drag.y;
    (-way.x).atan2(-way.z)
}

/// The heading a body turned `rotation` faces.
fn heading_of(rotation: Quat) -> f32 {
    let facing = rotation * Vec3::NEG_Z;
    (-facing.x).atan2(-facing.z)
}

/// The band on the ground that shows where what you aim will go, and
/// which of the things that are aimed it is drawn for.
#[derive(Component)]
struct AimBand;
#[derive(Component)]
struct BandFor(Utility);

/// The band a stick of dynamite is aimed with (since 2026-11-13; Hajun:
/// "instead of one straight rectangular hit range indicator its like an
/// long arched curved rectangular, with ability to change the total
/// distance with the magnitude of the hold&drag&letgo"): its three sheets,
/// built again as the throw changes, and the throw they were last built
/// for, how far it comes down, in metres, and which way it goes. **It is
/// the arch the stick will be seen to fly through** ([`lob_arch`]): from
/// your hand, up as high as the stick will go and up the screen with it,
/// and down again to the middle of the ring round where it will land,
/// where a cross marks the spot ([`LandingRing`]; Hajun, later that day:
/// "i want the dynamites hit indicator's curved part to be curved only in
/// the Y axis (up and down) not the sideways axis. its a arch projectile
/// not a boomarang. also i want there to be some indicator of where the
/// dynamite will land, maybe a x in the middle of the circle, and the
/// arched hit indicator rectangular part should be connected to the
/// middle of the circle"). For its first day it lay flat on the ground,
/// bent out to the thrower's right and back whichever way they threw,
/// and stopped at the ring's edge. The ring is mine, as Brawl Stars shows
/// where a thrower's shot lands.
#[derive(Resource)]
struct LobBand {
    sheets: [Handle<Mesh>; 3],
    built_for: (f32, f32),
    /// Whether there is any of it to draw.
    drawn: bool,
}

/// One of the three sheets of the dynamite's band, to put away should
/// there ever be none of it to draw.
#[derive(Component)]
struct LobSheet;

/// The ring at the end of the dynamite's band: where it comes down, as
/// far round that as its blast will reach, and the cross in its middle.
#[derive(Component)]
struct LandingRing;

/// The arch a stick of dynamite thrown to come down `far` from you, along
/// `heading`, will be seen to fly through, in the band's own space, which
/// is turned with the aim: straight ahead is -Z, and the ground the ring
/// lies on is no height at all (`range::HEIGHT` over your feet). From
/// where it leaves the hand to the middle of the ring, in so many
/// stretches: as high as the stick will be over the straight line of its
/// throw, and as far up the screen from that line as the stick is drawn
/// ([`lob_track`]), the top of the screen being to whichever side of the
/// aim north is.
fn lob_arch(far: f32, heading: f32) -> Vec<Vec3> {
    let lob = Lob::thrown(Vec3::ZERO, 0.0, far, 0.0);
    let (from, to) = (lob.from.xz(), lob.to.xz());
    // Up the map (north, -Z in the world), as the band turned with the
    // aim has it.
    let up = Vec2::new(heading.sin(), -heading.cos());
    let hand = (DYNAMITE_HAND - range::HEIGHT).max(0.0);
    let pieces = DYNAMITE_BAND_PIECES;
    (0..=pieces)
        .map(|piece| {
            let share = piece as f32 / pieces as f32;
            let across = lob_track(from, to, lob.apex(), up, share);
            Vec3::new(across.x, hand * (1.0 - share) + lob.rise(share), across.y)
        })
        .collect()
}
/// Its sheets' looks, to recolour as the file changes: the fill, the
/// outline just inside its edge, and the dark rim just outside it.
#[derive(Resource)]
struct BandLook {
    fill: Handle<StandardMaterial>,
    outline: Handle<StandardMaterial>,
    rim: Handle<StandardMaterial>,
}

/// The time left of your drink, written in front of you.
#[derive(Component)]
struct DrinkText;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<SlotPressed>()
        .init_resource::<Hand>()
        .init_resource::<MoreSlots>()
        .add_observer(press)
        .add_systems(Startup, spawn_band)
        .add_systems(
            Update,
            (
                // Ahead of the game's own reading of what was used, so
                // that it is used the frame it is let go.
                (buttons, aim, drink).chain().before(ArenaSystems),
                (fold_away, fill_slots).chain(),
                recolour,
                // The camera's place for this frame is set by then.
                show_drink.after(crate::follow_camera),
            ),
        );
}

/// A press on one of the buttons, as it goes down; or on the dots, which
/// show the four more buttons, empty as they may be, or put the empty
/// ones away again.
fn press(
    press: On<Pointer<Press>>,
    slots: Query<&UtilitySlot>,
    dots: Query<(), With<MoreButton>>,
    mut pressed: MessageWriter<SlotPressed>,
    mut more: ResMut<MoreSlots>,
) {
    if press.button != PointerButton::Primary {
        return;
    }
    if let Ok(slot) = slots.get(press.entity) {
        pressed.write(SlotPressed(slot.0, press.pointer_id));
    }
    if dots.contains(press.entity) {
        more.0 = !more.0;
        info!("the four more buttons are {}", if more.0 { "shown" } else { "put away" });
    }
}

/// Puts the empty buttons away again as a new game begins: the dots are
/// off at the start of each.
fn fold_away(fighting: Res<Fighting>, mut game: Local<u32>, mut more: ResMut<MoreSlots>) {
    if fighting.game != *game {
        *game = fighting.game;
        more.set_if_neq(MoreSlots(false));
    }
}

/// What is on the button numbered `slot` of `kit`, if anything is.
fn on_slot(kit: &roundtown_net::Kit, slot: usize) -> Option<Stack> {
    kit.slots.get(slot).copied().flatten()
}

/// Whether the second button is there with nothing on it, an empty
/// outline, as it was until 2026-11-14 and as the first sketch of the
/// buttons had it. Not since Hajun had the three dots taken out: "only
/// showing the button if the slot is filled and not showing empty
/// buttons". Whether that was meant of this one too is my reading of it:
/// this word puts its outline back.
const SECOND_ALWAYS: bool = false;

/// Whether the button numbered `slot` is to be seen: while something is
/// on it. (And with nothing on it while the three dots are on, which
/// there are none of to press since 2026-11-14, `hud::DOTS`; until then
/// the second button was there always.)
fn slot_shown(slot: usize, filled: bool, more: bool) -> bool {
    filled || more || (SECOND_ALWAYS && slot == 0)
}

/// Whether you are in a fight, alive, with nothing in the way of using
/// the button.
fn in_play(fighting: &Fighting, dialog: &DialogUp, venue: Venue, dead: bool) -> bool {
    fighting.playing && !dialog.0 && venue == Venue::Arena && !dead
}

/// Takes a press on one of the buttons, or its key: a potion is begun,
/// or, its button pressed again, put down; what is aimed is held to aim
/// from then, or, by the key, used the way you face. One thing at a
/// time: nothing while a finger aims, and no aiming with a potion at your
/// mouth.
#[allow(clippy::too_many_arguments)]
fn buttons(
    mut pressed: MessageReader<SlotPressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<DialogUp>,
    fighting: Res<Fighting>,
    outfit: Res<Outfit>,
    islands: Res<Islands>,
    mut me: Query<(Entity, &mut Transform, &Venue, Has<Dead>, Has<Drinking>, Option<&Health>), With<Player>>,
    mut hand: ResMut<Hand>,
    mut used: MessageWriter<Used>,
    mut commands: Commands,
) {
    let by_finger = pressed.read().last().map(|pressed| (pressed.0, Some(pressed.1)));
    let by_key = || SLOT_KEYS.iter().position(|keys| keyboard.any_just_pressed(keys.iter().copied()));
    let Some((slot, by_finger)) = by_finger.or_else(|| by_key().map(|slot| (slot, None))) else {
        return;
    };
    let Ok((you, mut body, &venue, dead, drinking, health)) = me.single_mut() else {
        return;
    };
    if !in_play(&fighting, &dialog, venue, dead) || hand.hold.is_some() {
        return;
    }
    let Some(utility) = on_slot(&outfit.kit, slot).filter(|stack| stack.count > 0).map(|stack| stack.utility) else {
        return;
    };
    if utility.drunk() {
        if !drinking {
            commands.entity(you).insert(Drinking { left: DRINK_SECS });
            hand.potion = Some(utility);
            hand.health = health.map_or(1.0, Health::share);
            info!("drinking {utility:?}");
        } else if hand.potion == Some(utility) {
            commands.entity(you).remove::<Drinking>();
            hand.potion = None;
            info!("put the potion down");
        }
        return;
    }
    // Aimed: no drinking and aiming at once, and not too soon after the
    // last.
    if drinking || hand.rest > 0.0 {
        return;
    }
    match by_finger {
        Some(pointer) => hand.hold = Some(Hold::new(pointer, utility)),
        None => {
            // By its key, a stick of dynamite goes as far as a quick tap
            // throws one.
            let heading = heading_of(body.rotation);
            if throw(utility, heading, DYNAMITE_TAP, &mut body, &islands, &mut used) {
                hand.rest = AIM_REST;
            }
        }
    }
}

/// Uses `utility`, which is aimed, along `heading`, from where your `body`
/// stands, turning you to face that way: whether it was. A builder is sent
/// off only from the ground; a stick of dynamite is thrown to come down
/// `far` from you.
fn throw(utility: Utility, heading: f32, far: f32, body: &mut Transform, islands: &Islands, used: &mut MessageWriter<Used>) -> bool {
    match utility {
        Utility::Fireball => {
            used.write(Used::Fireball { heading });
        }
        Utility::Dynamite => {
            used.write(Used::Dynamite { heading, far });
        }
        Utility::Builder => {
            let feet = body.translation;
            let standing = islands
                .get(Venue::Arena)
                .and_then(|island| island.floor(feet.xz(), feet.y + LAND_STEP_UP))
                .is_some_and(|ground| feet.y - ground <= IN_THE_AIR);
            if !standing {
                return false;
            }
            used.write(Used::Builder { heading });
        }
        _ => return false,
    }
    body.rotation = Quat::from_rotation_y(heading);
    true
}

/// Follows the finger holding the button: the band on the ground the way
/// it has been dragged, and, let go, the thing used that way, or the way
/// you face for a quick tap, or not at all ([`Hold::let_go`]).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn aim(
    time: Res<Time>,
    // (Together, to keep to the sixteen things a system may ask for.)
    (touches, mouse): (Res<Touches>, Res<ButtonInput<MouseButton>>),
    windows: Query<&Window>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    dialog: Res<DialogUp>,
    fighting: Res<Fighting>,
    outfit: Res<Outfit>,
    islands: Res<Islands>,
    mut me: Query<(&mut Transform, &Venue, Has<Dead>), With<Player>>,
    mut hand: ResMut<Hand>,
    mut bands: Query<(&mut Transform, &mut Visibility), (With<AimBand>, Without<Player>, Without<ThirdPersonCamera>, Without<BandFor>)>,
    mut kinds: Query<(&BandFor, &mut Visibility), (Without<AimBand>, Without<Player>)>,
    (mut rings, mut lob_sheets): (
        Query<&mut Transform, (With<LandingRing>, Without<AimBand>, Without<Player>, Without<ThirdPersonCamera>)>,
        Query<&mut Visibility, (With<LobSheet>, Without<AimBand>, Without<BandFor>)>,
    ),
    mut lob: ResMut<LobBand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut used: MessageWriter<Used>,
) {
    let dt = time.delta_secs();
    if hand.rest > 0.0 {
        hand.rest = (hand.rest - dt).max(0.0);
    }
    let Ok((mut band, mut shown)) = bands.single_mut() else {
        return;
    };
    let Some(mut hold) = hand.hold else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    let aimed = Some(hold.utility).filter(|utility| utility.aimed() && outfit.kit.held(*utility) > 0);
    let (Some(utility), Ok((mut body, &venue, dead)), Ok(window)) = (aimed, me.single_mut(), windows.single()) else {
        hand.hold = None;
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    if !in_play(&fighting, &dialog, venue, dead) {
        hand.hold = None;
        shown.set_if_neq(Visibility::Hidden);
        return;
    }
    let at = match hold.pointer {
        PointerId::Touch(id) => touches.get_pressed(id).map(|touch| touch.position()),
        PointerId::Mouse => mouse.pressed(MouseButton::Left).then(|| window.cursor_position()).flatten(),
        PointerId::Custom(_) => None,
    };
    let Some(at) = at else {
        // Let go.
        hand.hold = None;
        shown.set_if_neq(Visibility::Hidden);
        if let Some(heading) = hold.let_go(heading_of(body.rotation))
            && throw(utility, heading, hold.far(), &mut body, &islands, &mut used)
        {
            hand.rest = AIM_REST;
        }
        return;
    };
    let ahead = cameras.single().map_or(Vec3::NEG_Z, camera_ahead);
    let short_side = window.width().min(window.height());
    hold.drag(at, dt, short_side * AIM_DEAD / 100.0, short_side * AIM_FULL / 100.0, ahead);
    hand.hold = Some(hold);
    let Some(heading) = hold.aim else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    // A stick of dynamite's band is the arch of the throw: built again as
    // that grows or shrinks by a hand's breadth or turns by half a degree
    // (which way the top of the screen is from it changes as it turns),
    // with the ring where it comes down.
    if utility == Utility::Dynamite {
        let far = hold.far();
        if (far - lob.built_for.0).abs() > 0.04 || (heading - lob.built_for.1).abs() > 0.01 {
            lob.built_for = (far, heading);
            // Should there be none of it, its sheets are put away as they
            // are: a mesh of nothing is none the GPU can be given (Bevy's
            // mesh allocator logs a use after free for each).
            let line = lob_arch(far, heading);
            lob.drawn = line.len() >= 2;
            if lob.drawn {
                let (fill, outline, rim) = range::arch(&line, DYNAMITE_BAND);
                for (handle, mesh) in lob.sheets.iter().zip([rim, fill, outline]) {
                    if meshes.insert(handle, mesh).is_err() {
                        warn!("the dynamite's band has lost a mesh");
                    }
                }
            }
        }
        let sheets_shown = if lob.drawn { Visibility::Inherited } else { Visibility::Hidden };
        for mut sheet in &mut lob_sheets {
            sheet.set_if_neq(sheets_shown);
        }
        // The ring where it comes down: round, so that it turns with the
        // aim unseen; turned back by as much, so that the cross in its
        // middle lies aslant on the screen, an X, whichever way you aim.
        for mut ring in &mut rings {
            ring.set_if_neq(Transform {
                translation: Vec3::new(0.0, 0.0, -far),
                rotation: Quat::from_rotation_y(-heading),
                scale: Vec3::ONE,
            });
        }
    }
    band.set_if_neq(Transform {
        translation: body.translation + Vec3::Y * range::HEIGHT,
        rotation: Quat::from_rotation_y(heading),
        scale: Vec3::ONE,
    });
    shown.set_if_neq(Visibility::Inherited);
    for (kind, mut shown) in &mut kinds {
        shown.set_if_neq(if kind.0 == utility { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// Counts your drink down, and stops it if you are hit, or die, or the
/// potion is no longer on the button; run out, the potion is drunk.
fn drink(
    time: Res<Time>,
    dialog: Res<DialogUp>,
    fighting: Res<Fighting>,
    outfit: Res<Outfit>,
    mut me: Query<(Entity, &Venue, Has<Dead>, &mut Drinking, Option<&Health>), With<Player>>,
    mut hand: ResMut<Hand>,
    mut used: MessageWriter<Used>,
    mut commands: Commands,
) {
    let Ok((you, &venue, dead, mut drinking, health)) = me.single_mut() else {
        return;
    };
    let share = health.map_or(1.0, Health::share);
    let hit = share < hand.health - 1e-4;
    let still_held = hand.potion.is_some_and(|potion| outfit.kit.held(potion) > 0);
    if !in_play(&fighting, &dialog, venue, dead) || hit || !still_held {
        commands.entity(you).remove::<Drinking>();
        hand.potion = None;
        info!("the drink is stopped{}", if hit { ": hit" } else { "" });
        return;
    }
    hand.health = share;
    drinking.left -= time.delta_secs();
    if drinking.left <= 0.0 {
        commands.entity(you).remove::<Drinking>();
        if let Some(potion) = hand.potion.take() {
            used.write(Used::Potion(potion));
        }
    }
}

/// Shows on each button what is on it: its picture, how many, and the
/// time its effect still lasts; lights the one held to aim, or whose
/// potion is at your mouth; shows each of the four more while something
/// is on it or the dots are on, and puts it away otherwise
/// ([`slot_shown`]); and lights the dots while they are on. Only what has
/// changed is written.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fill_slots(
    outfit: Res<Outfit>,
    clock: Res<FightTime>,
    changed: Res<UiChanged>,
    hand: Res<Hand>,
    more: Res<MoreSlots>,
    drinking: Query<(), (With<Player>, With<Drinking>)>,
    added: Query<(), Added<UtilitySlot>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
    mut shown: Query<(&UtilityPicture, &mut ImageNode, &mut Visibility), Without<UtilitySlot>>,
    mut counts: Query<(&UtilityCount, &mut Text), Without<UtilityTimer>>,
    mut timers: Query<(&UtilityTimer, &mut Text), Without<UtilityCount>>,
    mut slots: Query<(&UtilitySlot, &mut BackgroundColor, &mut Visibility), Without<UtilityPicture>>,
    mut dots: Query<&mut BackgroundColor, (With<MoreButton>, Without<UtilitySlot>)>,
) {
    let kit = outfit.kit;
    if outfit.is_changed() || !added.is_empty() || (changed.is_changed() && !changed.is_added()) {
        for (of, mut image, mut visibility) in &mut shown {
            match on_slot(&kit, of.0) {
                Some(stack) => {
                    let (picture, tint) = market::look_of(stack.utility);
                    let handle = pictures.of(&mut images, picture);
                    let tint = if stack.count == 0 { tint.with_alpha(tint.alpha() * NONE_LEFT) } else { tint };
                    if image.image != handle || image.color != tint {
                        image.image = handle;
                        image.color = tint;
                    }
                    visibility.set_if_neq(Visibility::Inherited);
                }
                None => {
                    visibility.set_if_neq(Visibility::Hidden);
                }
            }
        }
        for (of, mut text) in &mut counts {
            let count = on_slot(&kit, of.0).filter(|stack| stack.count > 0).map_or(String::new(), |stack| stack.count.to_string());
            if text.0 != count {
                text.0 = count;
            }
        }
    }
    for (of, mut text) in &mut timers {
        let lasting = timer_text(f32::from(lasts_until(&kit, of.0)) - clock.0);
        if text.0 != lasting {
            text.0 = lasting;
        }
    }
    let colours = ui().colours.hud;
    let in_hand = hand.hold.map(|hold| hold.utility).or(hand.potion.filter(|_| !drinking.is_empty()));
    let lit = in_hand.and_then(|utility| kit.slot_of(utility));
    for (slot, mut fill, mut visibility) in &mut slots {
        fill.set_if_neq(BackgroundColor(if lit == Some(slot.0) { colours.button_held } else { colours.button }));
        let seen = slot_shown(slot.0, on_slot(&kit, slot.0).is_some(), more.0);
        visibility.set_if_neq(if seen { Visibility::Inherited } else { Visibility::Hidden });
    }
    for mut fill in &mut dots {
        fill.set_if_neq(BackgroundColor(if more.0 { colours.button_held } else { colours.button }));
    }
}

/// Until when, in whole seconds into the fight, what was drunk off the
/// button numbered `slot` of `kit` lasts: the speed potion's button by how
/// long they run fast, the invisibility's by how long they are unseen, and
/// 0 for any other, or an empty one.
fn lasts_until(kit: &roundtown_net::Kit, slot: usize) -> u16 {
    match on_slot(kit, slot).map(|stack| stack.utility) {
        Some(Utility::Speed) => kit.fast_until,
        Some(Utility::Invisibility) => kit.unseen_until,
        _ => 0,
    }
}

/// The time a potion's effect still lasts, as written over the button:
/// whole seconds, counted up, "45s" down to "1s"; nothing once it is over.
fn timer_text(left: f32) -> String {
    if left <= 0.0 { String::new() } else { format!("{}s", left.ceil() as u32) }
}

/// Writes the time left of your drink in front of you, in tenths of a
/// second, while a potion is at your mouth.
#[allow(clippy::type_complexity)]
fn show_drink(
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    me: Query<(&Transform, &Drinking), With<Player>>,
    mut written: Query<(&mut Text, &mut Visibility, &mut UiTransform, &ComputedNode), With<DrinkText>>,
    mut commands: Commands,
) {
    let drinking = me.single().ok();
    let Ok((mut text, mut shown, mut place, node)) = written.single_mut() else {
        if drinking.is_some() {
            commands.spawn((
                InArena,
                DrinkText,
                Name::new("Time left of a drink"),
                hud::words(String::new(), DRINK_TEXT, ui().colours.arena.drink),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
            ));
        }
        return;
    };
    let point = drinking.and_then(|(body, drinking)| {
        let (camera, eye) = cameras.single().ok()?;
        let facing = (body.rotation * Vec3::NEG_Z).with_y(0.0).normalize_or(Vec3::NEG_Z);
        let at = body.translation + facing * DRINK_AHEAD + Vec3::Y * DRINK_OVER;
        let point = camera.world_to_viewport(&GlobalTransform::from(*eye), at).ok()?;
        Some((point, drinking.left))
    });
    let Some((point, left)) = point else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    let said = format!("{:.1}", left.max(0.0));
    if text.0 != said {
        text.0 = said;
    }
    let size = node.size() * node.inverse_scale_factor();
    place.set_if_neq(UiTransform {
        translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    shown.set_if_neq(Visibility::Inherited);
}

/// Makes the band, hidden until something is aimed: for each of the two
/// things that are, its fill and, a hair over it, its outline, as the
/// punch's range is made (`range`), and round those a dark rim
/// (`range::rim`, in the file's `colours.arena.aim_rim`), under one root
/// with no `Venue`, so that it is drawn wherever you are and is in
/// nothing's way. The rim is what shows it over the void (since
/// 2026-11-12; Hajun: "make the hit indicator for fireball appear even
/// when ur on a bridge (the hit indicator should appear even on void
/// area)"): the band was drawn there all along, a white as pale as the
/// sky by day, and from a bridge nearly all of it lies over the void.
fn spawn_band(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.arena;
    let look = BandLook {
        fill: materials.add(range::look(colours.aim_fill)),
        outline: materials.add(range::look(colours.aim_outline)),
        rim: materials.add(range::look(colours.aim_rim)),
    };
    let reach = PLAYER_RADIUS + 0.05;
    let shapes = [
        (Utility::Fireball, FIREBALL_BAND, reach, FIREBALL_AHEAD + FIREBALL_RANGE),
        (Utility::Builder, BUILDER_BAND, reach, reach + BUILDER_BAND_LENGTH),
    ]
    .map(|(utility, width, from, to)| (utility, range::band(width, from, to), range::rim(width, from, to)));
    // A stick of dynamite's: the band, built again as the throw changes
    // (`aim`); the ring round where it comes down, which is always as
    // wide as the blast; and the cross in the middle of that.
    let (lob_fill, lob_outline, lob_rim) = range::arch(&lob_arch(DYNAMITE_TAP, 0.0), DYNAMITE_BAND);
    let lob = LobBand {
        sheets: [meshes.add(lob_rim), meshes.add(lob_fill), meshes.add(lob_outline)],
        built_for: (DYNAMITE_TAP, 0.0),
        drawn: true,
    };
    let (ring_fill, ring_outline, ring_rim) = range::disc(DYNAMITE_RADIUS);
    let ring = [meshes.add(ring_rim), meshes.add(ring_fill), meshes.add(ring_outline)];
    let (cross_fill, cross_rim) = range::cross(LANDING_CROSS.0, LANDING_CROSS.1);
    let cross = [(meshes.add(cross_rim), look.rim.clone(), 0.004), (meshes.add(cross_fill), look.outline.clone(), 0.006)];
    // A band's three sheets, the outline a hair over the rest.
    let sheets = |sheets: [Handle<Mesh>; 3]| {
        let looks = [look.rim.clone(), look.fill.clone(), look.outline.clone()];
        let over = [0.0, 0.0, 0.002];
        std::array::from_fn::<_, 3, _>(|sheet| {
            (
                Mesh3d(sheets[sheet].clone()),
                MeshMaterial3d(looks[sheet].clone()),
                Transform::from_xyz(0.0, over[sheet], 0.0),
                NotShadowCaster,
                NotShadowReceiver,
            )
        })
    };
    commands
        .spawn((Name::new("Aim band"), AimBand, Transform::default(), Visibility::Hidden))
        .with_children(|band| {
            for (utility, (fill, outline), rim) in shapes {
                band.spawn((BandFor(utility), Transform::default(), Visibility::Hidden)).with_children(|kind| {
                    for sheet in sheets([meshes.add(rim), meshes.add(fill), meshes.add(outline)]) {
                        kind.spawn(sheet);
                    }
                });
            }
            band.spawn((BandFor(Utility::Dynamite), Transform::default(), Visibility::Hidden)).with_children(|kind| {
                // (Its shape changes under it as you aim: never left out
                // for where it was a moment ago.)
                for sheet in sheets(lob.sheets.clone()) {
                    kind.spawn((sheet, LobSheet, NoFrustumCulling));
                }
                kind.spawn((LandingRing, Transform::from_xyz(0.0, 0.0, -DYNAMITE_TAP), Visibility::Inherited))
                    .with_children(|landing| {
                        for sheet in sheets(ring.clone()) {
                            landing.spawn(sheet);
                        }
                        // The cross, white as the outlines are, with the
                        // dark rim round it, a hair over the ring.
                        for (mesh, material, over) in cross.clone() {
                            landing.spawn((
                                Mesh3d(mesh),
                                MeshMaterial3d(material),
                                Transform::from_xyz(0.0, over, 0.0),
                                NotShadowCaster,
                                NotShadowReceiver,
                            ));
                        }
                    });
            });
        });
    commands.insert_resource(lob);
    commands.insert_resource(look);
}

/// Takes the band's colours, and the drink's, again as the file changes
/// on desktop.
fn recolour(
    changed: Res<UiChanged>,
    look: Res<BandLook>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut written: Query<&mut TextColor, With<DrinkText>>,
) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colours = ui().colours.arena;
    for (handle, colour) in [(&look.fill, colours.aim_fill), (&look.outline, colours.aim_outline), (&look.rim, colours.aim_rim)] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = range::bright(colour);
        }
    }
    for mut colour in &mut written {
        colour.set_if_neq(TextColor(colours.drink));
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;

    /// Which way across the ground a heading goes, as a plank heads.
    fn way(heading: f32) -> Vec3 {
        Vec3::new(-heading.sin(), 0.0, -heading.cos())
    }

    #[test]
    fn a_drag_aims_the_way_it_goes_on_the_screen() {
        // The camera over you has the top of the screen north (-Z): a drag
        // up the screen aims north, to the right east (+X), down south.
        let north = Vec3::NEG_Z;
        assert!(way(heading_of_drag(Vec2::new(0.0, -50.0), north)).distance(Vec3::NEG_Z) < 1e-5);
        assert!(way(heading_of_drag(Vec2::new(50.0, 0.0), north)).distance(Vec3::X) < 1e-5);
        assert!(way(heading_of_drag(Vec2::new(0.0, 50.0), north)).distance(Vec3::Z) < 1e-5);
        assert!(way(heading_of_drag(Vec2::new(-30.0, -30.0), north)).distance(Vec3::new(-1.0, 0.0, -1.0).normalize()) < 1e-5);
        // With the screen's top another way, the same drag goes with it.
        assert!(way(heading_of_drag(Vec2::new(0.0, -50.0), Vec3::X)).distance(Vec3::X) < 1e-5);
        assert!(way(heading_of_drag(Vec2::new(50.0, 0.0), Vec3::X)).distance(Vec3::Z) < 1e-5);
        // And a body's heading is the way it faces.
        assert!(heading_of(Quat::IDENTITY).abs() < 1e-6);
        assert!((heading_of(Quat::from_rotation_y(FRAC_PI_2)) - FRAC_PI_2).abs() < 1e-5);
        assert!(way(heading_of(Quat::from_rotation_y(2.5))).distance(Quat::from_rotation_y(2.5) * Vec3::NEG_Z) < 1e-5);
    }

    #[test]
    fn letting_go_uses_it_the_way_it_aimed_and_dragging_back_cancels() {
        let (dead, full, north, dt) = (40.0, 220.0, Vec3::NEG_Z, 1.0 / 60.0);
        let facing = PI;
        // Held and dragged east, past the dead zone: it aims east, and
        // goes east let go, whichever way you face.
        let mut hold = Hold::new(PointerId::Mouse, Utility::Fireball);
        hold.drag(Vec2::new(500.0, 300.0), dt, dead, full, north);
        assert!(hold.aim.is_none() && !hold.out);
        hold.drag(Vec2::new(530.0, 300.0), dt, dead, full, north);
        assert!(hold.aim.is_none(), "30 pixels is short of the dead zone");
        hold.drag(Vec2::new(600.0, 300.0), dt, dead, full, north);
        assert!(hold.out && way(hold.aim.unwrap()).distance(Vec3::X) < 1e-5);
        assert!(way(hold.let_go(facing).unwrap()).distance(Vec3::X) < 1e-5);
        // Dragged back to where it went down, it aims nowhere, and letting
        // go there cancels, however quick.
        hold.drag(Vec2::new(505.0, 302.0), dt, dead, full, north);
        assert!(hold.aim.is_none() && hold.out && hold.age < QUICK_TAP);
        assert_eq!(hold.let_go(facing), None);
        // A quick tap that never dragged goes the way you face; a long
        // press that never dragged is nothing.
        let mut tap = Hold::new(PointerId::Mouse, Utility::Builder);
        tap.drag(Vec2::new(500.0, 300.0), 0.1, dead, full, north);
        assert_eq!(tap.let_go(facing), Some(facing));
        tap.drag(Vec2::new(501.0, 300.0), 0.5, dead, full, north);
        assert_eq!(tap.let_go(facing), None);
    }

    #[test]
    fn a_stick_of_dynamite_is_thrown_as_far_as_the_finger_is_dragged_along_a_band_that_bends_to_a_ring() {
        let (dead, full, north, dt) = (40.0, 220.0, Vec3::NEG_Z, 1.0 / 60.0);
        // A quick tap that never dragged: a few metres, the way you face.
        let mut tap = Hold::new(PointerId::Mouse, Utility::Dynamite);
        tap.drag(Vec2::new(500.0, 300.0), 0.1, dead, full, north);
        assert_eq!((tap.let_go(1.0), tap.far()), (Some(1.0), DYNAMITE_TAP));
        // Dragged just past where it begins to aim: as near as one can be
        // thrown. Halfway out: halfway between. All the way, and past it:
        // as far as one can be.
        let mut hold = Hold::new(PointerId::Mouse, Utility::Dynamite);
        hold.drag(Vec2::new(500.0, 300.0), dt, dead, full, north);
        hold.drag(Vec2::new(500.0, 259.0), dt, dead, full, north);
        assert!(hold.aim.is_some() && (hold.far() - DYNAMITE_NEAREST).abs() < 0.1, "{}", hold.far());
        hold.drag(Vec2::new(500.0, 300.0 - 130.0), dt, dead, full, north);
        assert!((hold.far() - (DYNAMITE_NEAREST + DYNAMITE_FURTHEST) * 0.5).abs() < 1e-3, "{}", hold.far());
        hold.drag(Vec2::new(500.0, 300.0 - 220.0), dt, dead, full, north);
        assert!((hold.far() - DYNAMITE_FURTHEST).abs() < 1e-4);
        hold.drag(Vec2::new(500.0, -400.0), dt, dead, full, north);
        assert!((hold.far() - DYNAMITE_FURTHEST).abs() < 1e-4 && way(hold.let_go(0.0).unwrap()).distance(Vec3::NEG_Z) < 1e-5);
        // Its band is the arch of the throw, from the hand to the very
        // middle of the ring round where it comes down, whichever way it
        // is thrown and however far: up from the hand's height to as high
        // as the stick goes, and down to the ground the ring lies on.
        use roundtown_net::bridge_wars::DYNAMITE_AHEAD;
        let hand = DYNAMITE_HAND - range::HEIGHT;
        for far in [DYNAMITE_NEAREST, DYNAMITE_TAP, DYNAMITE_FURTHEST] {
            for heading in [0.0, FRAC_PI_2, -FRAC_PI_2, PI, 0.7] {
                let line = lob_arch(far, heading);
                assert_eq!(line.len(), DYNAMITE_BAND_PIECES + 1);
                assert!(line[0].distance(Vec3::new(0.0, hand, -DYNAMITE_AHEAD)) < 1e-4, "{:?}", line[0]);
                assert!(line[line.len() - 1].distance(Vec3::new(0.0, 0.0, -far)) < 1e-4, "it ends in the middle of the ring");
                let top = line.iter().map(|point| point.y).fold(0.0, f32::max);
                assert!(top > hand + 0.3 && top < 3.2, "it rises to {top}");
            }
        }
        // Thrown up the screen, north, or down it, it is a straight line
        // from above: the arch is seen in its height alone.
        for heading in [0.0, PI] {
            assert!(lob_arch(DYNAMITE_FURTHEST, heading).iter().all(|point| point.x.abs() < 1e-4));
        }
        // Thrown to the east, the top of the screen is on the left of the
        // aim, and the arch leans that way, by as much as the stick rises;
        // thrown to the west, on its right. Either way that is up the
        // screen, never down it.
        let east = lob_arch(DYNAMITE_FURTHEST, -FRAC_PI_2);
        let west = lob_arch(DYNAMITE_FURTHEST, FRAC_PI_2);
        let lean = east.iter().map(|point| -point.x).fold(0.0, f32::max);
        assert!(lean > 1.5 && lean < 2.6, "it leans {lean} m up the screen");
        assert!(east.iter().all(|point| point.x <= 1e-4) && west.iter().all(|point| point.x >= -1e-4));
        assert!(east.iter().zip(&west).all(|(a, b)| (a.x + b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4 && (a.z - b.z).abs() < 1e-4));
        // The band is a narrow one, and its sheets are built whole, each
        // corner as high as the arch is there.
        assert!(DYNAMITE_BAND < 1.0 && AIM_FULL > 3.0 * AIM_DEAD);
        let (fill, outline, rim) = range::arch(&east, DYNAMITE_BAND);
        for sheet in [&fill, &outline, &rim] {
            assert!(sheet.count_vertices() > 8 && sheet.indices().is_some_and(|indices| indices.len() >= 24));
        }
        let corners = fill.attribute(Mesh::ATTRIBUTE_POSITION).and_then(|corners| corners.as_float3()).expect("corners");
        let highest = corners.iter().map(|corner| corner[1]).fold(0.0, f32::max);
        let top = east.iter().map(|point| point.y).fold(0.0, f32::max);
        assert!((highest - top).abs() < 1e-4, "{highest} against {top}");
        let (fill, outline, rim) = range::arch(&[], DYNAMITE_BAND);
        assert!([fill, outline, rim].iter().all(|sheet| sheet.count_vertices() == 0));
        // The cross in the ring's middle is small beside the ring, and its
        // two strokes are whole.
        let (cross, round) = range::cross(LANDING_CROSS.0, LANDING_CROSS.1);
        assert!(LANDING_CROSS.0 < 0.25 * DYNAMITE_RADIUS && LANDING_CROSS.1 < 0.25 * LANDING_CROSS.0);
        assert_eq!(cross.indices().map(|indices| indices.len()), Some(36));
        assert!(round.count_vertices() == 24);
    }

    #[test]
    fn the_time_over_the_button_counts_whole_seconds_down_to_nothing() {
        assert_eq!(timer_text(45.0), "45s");
        assert_eq!(timer_text(44.2), "45s");
        assert_eq!(timer_text(0.4), "1s");
        assert_eq!(timer_text(0.0), "");
        assert_eq!(timer_text(-200.0), "");
        // The bands are narrow and long: the fireball's as far as it
        // flies, the builder's as wide as its bridge.
        assert!(FIREBALL_BAND < 1.2 && FIREBALL_AHEAD + FIREBALL_RANGE > 10.0 * FIREBALL_BAND);
        assert!((BUILDER_BAND - 2.726).abs() < 1e-3);
    }

    #[test]
    fn the_four_more_buttons_show_while_filled_or_the_dots_are_on_and_each_potion_times_its_own() {
        use roundtown_net::Kit;
        // A button is there with something on it, and not with nothing,
        // the second button like the four more; the dots, were there any
        // to press, would show the empty ones too, and never put away
        // what you hold.
        for slot in 0..UTILITY_SLOTS {
            assert!(slot_shown(slot, true, false) && slot_shown(slot, true, true));
            assert!(slot_shown(slot, false, true));
            assert_eq!(slot_shown(slot, false, false), SECOND_ALWAYS && slot == 0, "an empty button is not shown");
        }
        // A key to each button, Q the second button's as it was, and no
        // key two buttons'.
        assert!(SLOT_KEYS[0].contains(&KeyCode::KeyQ));
        let keys: Vec<KeyCode> = SLOT_KEYS.iter().flat_map(|keys| keys.iter().copied()).collect();
        assert!(keys.iter().enumerate().all(|(at, key)| !keys[..at].contains(key)));
        // Fireballs, then a speed potion, then an invisibility potion:
        // three buttons in that order; each potion's time is over its own
        // button, and a fireball's has none.
        let kit = Kit {
            fast_until: 70,
            unseen_until: 55,
            ..Kit::default()
        }
        .holding(Utility::Fireball, 2)
        .holding(Utility::Speed, 0)
        .holding(Utility::Invisibility, 1);
        assert_eq!(on_slot(&kit, 1), Some(Stack { utility: Utility::Speed, count: 0 }));
        assert_eq!((on_slot(&kit, 3), on_slot(&kit, UTILITY_SLOTS)), (None, None));
        assert_eq!([0, 1, 2, 3, 4].map(|slot| lasts_until(&kit, slot)), [0, 70, 55, 0, 0]);
    }
}
