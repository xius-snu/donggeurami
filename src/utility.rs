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

use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use roundtown_net::Utility;
use roundtown_net::bridge_wars::{DRINK_SECS, FIREBALL_AHEAD, FIREBALL_RADIUS, FIREBALL_RANGE};

use crate::arena::{ArenaSystems, FightTime, Fighting, Health, InArena, Outfit, Used};
use crate::bridge;
use crate::hud::{self, DialogUp, Pictures, UtilityCount, UtilityPicture, UtilitySlot, UtilityTimer};
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

/// The second button was pressed, by this finger or the mouse.
#[derive(Message, Clone, Copy)]
struct SlotPressed(PointerId);

/// What your hand is doing at the second button.
#[derive(Resource, Default)]
struct Hand {
    /// The finger holding it down to aim, if one is.
    hold: Option<Hold>,
    /// How long until something aimed can be used again, in seconds.
    rest: f32,
    /// The potion at your mouth, and how much health you had when last
    /// looked at: any less, and you were hit.
    potion: Option<Utility>,
    health: f32,
}

/// A finger, or the mouse, holding the button down to aim: which; where
/// it went down, once that is read, in the window's own pixels; for how
/// long; whether it has ever been dragged out far enough to aim; and
/// which way it aims now, as a plank heads, if it is out that far.
#[derive(Clone, Copy, Debug)]
struct Hold {
    pointer: PointerId,
    from: Option<Vec2>,
    age: f32,
    out: bool,
    aim: Option<f32>,
}

impl Hold {
    fn new(pointer: PointerId) -> Self {
        Self {
            pointer,
            from: None,
            age: 0.0,
            out: false,
            aim: None,
        }
    }

    /// The finger is `at` now, `dt` seconds on: aiming the way it has been
    /// dragged from where it went down, across a screen whose up is
    /// `ahead` on the ground, once it is `dead` or more from there.
    fn drag(&mut self, at: Vec2, dt: f32, dead: f32, ahead: Vec3) {
        let drag = at - *self.from.get_or_insert(at);
        self.age += dt;
        self.aim = (drag.length() >= dead).then(|| heading_of_drag(drag, ahead));
        self.out |= self.aim.is_some();
    }

    /// What letting go does, with you facing `facing`: the heading to use
    /// the thing along, the way it aimed, or, a quick tap that never
    /// dragged, the way you face; or nothing, cancelled.
    fn let_go(&self, facing: f32) -> Option<f32> {
        self.aim.or((!self.out && self.age <= QUICK_TAP).then_some(facing))
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
/// which of the two it is drawn for.
#[derive(Component)]
struct AimBand;
#[derive(Component)]
struct BandFor(Utility);
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
        .add_observer(press)
        .add_systems(Startup, spawn_band)
        .add_systems(
            Update,
            (
                // Ahead of the game's own reading of what was used, so
                // that it is used the frame it is let go.
                (buttons, aim, drink).chain().before(ArenaSystems),
                (fill_slot, recolour),
                // The camera's place for this frame is set by then.
                show_drink.after(crate::follow_camera),
            ),
        );
}

/// A press on the second button, as it goes down.
fn press(press: On<Pointer<Press>>, slots: Query<(), With<UtilitySlot>>, mut pressed: MessageWriter<SlotPressed>) {
    if press.button == PointerButton::Primary && slots.contains(press.entity) {
        pressed.write(SlotPressed(press.pointer_id));
    }
}

/// Whether you are in a fight, alive, with nothing in the way of using
/// the button.
fn in_play(fighting: &Fighting, dialog: &DialogUp, venue: Venue, dead: bool) -> bool {
    fighting.playing && !dialog.0 && venue == Venue::Arena && !dead
}

/// Takes a press on the button, or Q: a potion is begun, or, pressed
/// again, put down; what is aimed is held to aim from then, or, by the
/// key, used the way you face.
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
    let by_finger = pressed.read().last().map(|pressed| pressed.0);
    let by_key = keyboard.just_pressed(KeyCode::KeyQ);
    if by_finger.is_none() && !by_key {
        return;
    }
    let Ok((you, mut body, &venue, dead, drinking, health)) = me.single_mut() else {
        return;
    };
    if !in_play(&fighting, &dialog, venue, dead) {
        return;
    }
    let Some(utility) = outfit.kit.utility.filter(|_| outfit.kit.count > 0) else {
        return;
    };
    if utility.drunk() {
        if drinking {
            commands.entity(you).remove::<Drinking>();
            hand.potion = None;
            info!("put the potion down");
        } else {
            commands.entity(you).insert(Drinking { left: DRINK_SECS });
            hand.potion = Some(utility);
            hand.health = health.map_or(1.0, Health::share);
            info!("drinking {utility:?}");
        }
        return;
    }
    // Aimed: no drinking and aiming at once, and not too soon after the
    // last.
    if drinking || hand.rest > 0.0 {
        return;
    }
    match by_finger {
        Some(pointer) => hand.hold = Some(Hold::new(pointer)),
        None => {
            let heading = heading_of(body.rotation);
            if throw(utility, heading, &mut body, &islands, &mut used) {
                hand.rest = AIM_REST;
            }
        }
    }
}

/// Uses `utility`, which is aimed, along `heading`, from where your `body`
/// stands, turning you to face that way: whether it was. A builder is sent
/// off only from the ground.
fn throw(utility: Utility, heading: f32, body: &mut Transform, islands: &Islands, used: &mut MessageWriter<Used>) -> bool {
    match utility {
        Utility::Fireball => {
            used.write(Used::Fireball { heading });
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
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
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
    let aimed = outfit.kit.utility.filter(|utility| utility.aimed() && outfit.kit.count > 0);
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
            && throw(utility, heading, &mut body, &islands, &mut used)
        {
            hand.rest = AIM_REST;
        }
        return;
    };
    let ahead = cameras.single().map_or(Vec3::NEG_Z, camera_ahead);
    let dead_zone = window.width().min(window.height()) * AIM_DEAD / 100.0;
    hold.drag(at, dt, dead_zone, ahead);
    hand.hold = Some(hold);
    let Some(heading) = hold.aim else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
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
    let still_held = outfit.kit.utility == hand.potion && hand.potion.is_some() && outfit.kit.count > 0;
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
        hand.potion = None;
        used.write(Used::Potion);
    }
}

/// Shows on the button what is on it: its picture, how many, and the time
/// its effect still lasts; and lights it while it is held to aim or a
/// potion is at your mouth. Only what has changed is written.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fill_slot(
    outfit: Res<Outfit>,
    clock: Res<FightTime>,
    changed: Res<UiChanged>,
    hand: Res<Hand>,
    drinking: Query<(), (With<Player>, With<Drinking>)>,
    added: Query<(), Added<UtilitySlot>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
    mut shown: Query<(&mut ImageNode, &mut Visibility), With<UtilityPicture>>,
    mut counts: Query<&mut Text, (With<UtilityCount>, Without<UtilityTimer>)>,
    mut timers: Query<&mut Text, (With<UtilityTimer>, Without<UtilityCount>)>,
    mut slots: Query<&mut BackgroundColor, With<UtilitySlot>>,
) {
    let kit = outfit.kit;
    if outfit.is_changed() || !added.is_empty() || (changed.is_changed() && !changed.is_added()) {
        for (mut image, mut visibility) in &mut shown {
            match kit.utility {
                Some(utility) => {
                    let (picture, tint) = market::look_of(utility);
                    let handle = pictures.of(&mut images, picture);
                    let tint = if kit.count == 0 { tint.with_alpha(tint.alpha() * NONE_LEFT) } else { tint };
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
        let count = if kit.utility.is_some() && kit.count > 0 { kit.count.to_string() } else { String::new() };
        for mut text in &mut counts {
            if text.0 != count {
                text.0.clone_from(&count);
            }
        }
    }
    let lasting = timer_text(f32::from(kit.fast_until.max(kit.unseen_until)) - clock.0);
    for mut text in &mut timers {
        if text.0 != lasting {
            text.0.clone_from(&lasting);
        }
    }
    let colours = ui().colours.hud;
    let lit = hand.hold.is_some() || !drinking.is_empty();
    for mut fill in &mut slots {
        fill.set_if_neq(BackgroundColor(if lit { colours.button_held } else { colours.button }));
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
    commands
        .spawn((Name::new("Aim band"), AimBand, Transform::default(), Visibility::Hidden))
        .with_children(|band| {
            for (utility, (fill, outline), rim) in shapes {
                band.spawn((BandFor(utility), Transform::default(), Visibility::Hidden)).with_children(|kind| {
                    kind.spawn((
                        Mesh3d(meshes.add(rim)),
                        MeshMaterial3d(look.rim.clone()),
                        Transform::default(),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                    kind.spawn((
                        Mesh3d(meshes.add(fill)),
                        MeshMaterial3d(look.fill.clone()),
                        Transform::default(),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                    kind.spawn((
                        Mesh3d(meshes.add(outline)),
                        MeshMaterial3d(look.outline.clone()),
                        Transform::from_xyz(0.0, 0.002, 0.0),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                });
            }
        });
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
        let (dead, north, dt) = (40.0, Vec3::NEG_Z, 1.0 / 60.0);
        let facing = PI;
        // Held and dragged east, past the dead zone: it aims east, and
        // goes east let go, whichever way you face.
        let mut hold = Hold::new(PointerId::Mouse);
        hold.drag(Vec2::new(500.0, 300.0), dt, dead, north);
        assert!(hold.aim.is_none() && !hold.out);
        hold.drag(Vec2::new(530.0, 300.0), dt, dead, north);
        assert!(hold.aim.is_none(), "30 pixels is short of the dead zone");
        hold.drag(Vec2::new(600.0, 300.0), dt, dead, north);
        assert!(hold.out && way(hold.aim.unwrap()).distance(Vec3::X) < 1e-5);
        assert!(way(hold.let_go(facing).unwrap()).distance(Vec3::X) < 1e-5);
        // Dragged back to where it went down, it aims nowhere, and letting
        // go there cancels, however quick.
        hold.drag(Vec2::new(505.0, 302.0), dt, dead, north);
        assert!(hold.aim.is_none() && hold.out && hold.age < QUICK_TAP);
        assert_eq!(hold.let_go(facing), None);
        // A quick tap that never dragged goes the way you face; a long
        // press that never dragged is nothing.
        let mut tap = Hold::new(PointerId::Mouse);
        tap.drag(Vec2::new(500.0, 300.0), 0.1, dead, north);
        assert_eq!(tap.let_go(facing), Some(facing));
        tap.drag(Vec2::new(501.0, 300.0), 0.5, dead, north);
        assert_eq!(tap.let_go(facing), None);
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
}
