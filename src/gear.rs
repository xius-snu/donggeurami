//! What is to be seen of what Bridge Wars' shops sell (since 2026-11-12):
//! the sword in a body's hand, the shield round it, yourself gone
//! see-through while nobody else can see you, and the dark over the screen
//! while a trap has you blinded. Who has what is the game's
//! (`arena::Wears` on every body in a game, `arena::Outfit` for you), this
//! device's own offline and the server's online, so everyone sees
//! everyone's sword and shield; this only draws it.
//!
//! **The sword** ([`arm`], [`hold_swords`]) is Hajun's model for its tier
//! ([`SWORDS`]: `basicsword.glb`, `silver_sword.glb`, `aura_sword.glb`,
//! each with its blade along its own +y and its grip at its origin), a
//! child of the body, put in the right hand every frame once the limbs
//! have been posed: where the hand is (the rig's `hand.R`, followed up to
//! the body), with the blade pointing the way the pose asks
//! (`crate::blade_way`), its flat side up, so that the camera over you
//! sees all of it. At the ready it is out in front; in a slash it sweeps
//! across with the arm (`crate::sword_offset`: Hajun, "you may also need
//! to make the sword animation for the swords rather than using punch
//! swing animation for it"). It is put away while its bearer carries a
//! plank or drinks ("sword unequips while drinking"). The models are a
//! real sword's size, a metre of thin blade, which from the camera 15 m
//! over you is a hair: they are drawn bigger ([`SWORD_SCALE`]), broader
//! across the blade than along it, and every slash leaves a trail where
//! the blade went ([`Trail`]), in the sword's colour, for a moment, so
//! that a swing is seen from up there.
//!
//! **Since 2026-11-12 a sword is held on its edge and cut downward**
//! (the file's `arena.sword_swing: Chop`, `crate::chop_offset`; Hajun:
//! "this time instead of holding the sword flat, hold it narrow and slash
//! downward(?) and use that animation instead"), and the way above,
//! `Sweep`, is kept, all of it, to go back to by that one word ("i might
//! wanna change it back after trying this new way"). On its edge the
//! blade's flat is square to the plane it cuts in (`crate::chop_blade`,
//! [`edge_on`]), which leans, so that the camera over you sees a slim
//! blade rather than a hair; it is drawn thicker through its flat for the
//! same reason ([`CHOP_SCALE`]). At the ready it is held straighter than
//! it cuts, as far forward of straight up and over to the right as the
//! file says (`crate::Guard`, `arena.sword_tilt_forward` and
//! `sword_tilt_side`; Hajun, of the first try: "the sword holding is too
//! tilted now (both forward tilt and side tilt). make it more straight"). Its trail is the ribbon the blade
//! itself has cut ([`Cut`], [`Cutting`]): where its edge has been from
//! the start of the cut, a sample to every few degrees whatever the frame
//! rate, built again each frame of the cut and faded after as the other
//! is. From over you that is a crescent from out on your right round to
//! in front of you.
//!
//! **The shield** ([`shield`]) is a ball of force round the body ("a very
//! transparent (slightly semitransparent) spherical forcefield around your
//! character"), in the colour of its tier and as see-through as that
//! colour's alpha (`colours.arena.shield_*` in `assets/ui.ron`).
//!
//! **Unseen** ([`ghost`]), nobody of another team draws you at all
//! (`island::show_where_i_am`); you see yourself see-through, so that you
//! know where you are and that it is working, and so does your own team:
//! every part of you drawn with a copy of its own material at [`GHOST`] of
//! itself, blended, and given back when it is over.
//!
//! **Blinded** ([`blindfold`]: "they can only see a very small circle
//! around their character and the rest of their screen is black except
//! their ui menu/buttons/minimap"), the screen is covered in the file's
//! `colours.arena.blind` all but a circle round you as far as the rules
//! let a blinded player see (`BLIND_SIGHT`), under the buttons, the
//! minimap and the rest of the screen's own, and over everything in the
//! world and written over it.

use std::collections::HashMap;

use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use roundtown_net::bridge_wars::BLIND_SIGHT;

use crate::arena::{ArenaSystems, InArena, Outfit, Wears};
use crate::island::Venue;
use crate::ui::{SwordSwing, UiChanged, ui};
use crate::{Carrying, Dead, Drinking, Player, Punching, Team, ThirdPersonCamera, WalkCycle};

/// The swords' models, by tier, the basic first.
pub(crate) const SWORDS: [&str; 3] = ["basicsword.glb", "silver_sword.glb", "aura_sword.glb"];
/// What the hand that holds a sword is called in the body's rig.
const HAND: &str = "hand.R";
/// How far along the hand, from the wrist, the grip is held, in the rig's
/// own measure: the middle of the palm.
const PALM: f32 = 0.1;
/// How much bigger than its model a sword is drawn: across its guard,
/// along its blade, and through its flat. Twice as broad, so that the
/// camera over you sees a blade and a guard rather than a line.
const SWORD_SCALE: Vec3 = Vec3::new(2.0, 1.7, 1.7);
/// The trail a slash leaves: from how far out from the body's middle to
/// how far, in metres, the blade's reach; how high over the feet; how
/// see-through at its head; and how long it takes to fade once the blade
/// has swept, in seconds.
const TRAIL_INNER: f32 = 0.75;
const TRAIL_OUTER: f32 = 1.95;
const TRAIL_HEIGHT: f32 = 1.05;
const TRAIL_ALPHA: f32 = 0.85;
const TRAIL_FADE: f32 = 0.16;
/// In how many steps a trail grows behind the blade as it sweeps: a shape
/// for each, made once.
const TRAIL_STEPS: usize = 8;
/// A sword held on its edge (`SwordSwing::Chop`) is drawn this much
/// bigger than its model: as the other way across its guard and along its
/// blade, and thicker through its flat, which is what the camera over you
/// sees most of.
const CHOP_SCALE: Vec3 = Vec3::new(2.0, 1.7, 3.0);
/// How far up its own y each sword's blade reaches from the grip, in its
/// model's metres, by tier, the basic first; and from how far up the
/// blade, as a share of that, the trail of a cut begins.
const BLADE: [f32; 3] = [0.78, 0.94, 0.855];
const CUT_FROM: f32 = 0.3;
/// How far a cut's blade turns between two samples of its trail, at the
/// most, in radians: about five degrees, whatever the frame rate.
const CUT_STEP: f32 = 0.09;
/// How a cut's trail comes up from nothing where the cut began to all of
/// it at the blade: the share of the way along it, to this power. Under
/// 1, so that most of it is there to see: the camera over you sees the
/// ribbon from nearer its edge than its face, a crescent half as broad
/// as the other way's arc.
const CUT_RAMP: f32 = 0.75;
/// The ball of force round a body with a shield: how far out from its
/// middle, and how high over its feet that middle is, in metres.
const BUBBLE_RADIUS: f32 = 1.05;
const BUBBLE_HEIGHT: f32 = 0.85;
/// How much of yourself you see while nobody else sees you.
const GHOST: f32 = 0.38;
/// How high over your feet the middle of what you see while blinded is,
/// in metres: your chest, where the camera over you looks.
const BLIND_CENTRE: f32 = 0.9;

/// A body with a sword in its hand: the sword, which tier it is, the hand
/// it is in, once that has been found in the rig, and the cut it is in
/// the middle of, if it is in one (`SwordSwing::Chop`).
#[derive(Component)]
struct Armed {
    sword: Entity,
    tier: u8,
    hand: Option<Entity>,
    cutting: Option<Cutting>,
}

/// A cut under way: the shape of its trail, built again as the blade
/// comes down; where the blade has been so far, in the body's space, from
/// a little way up it to its tip, a pair to each sample; and how far
/// through its sweep the cut was at the last of them, with where the grip
/// was then.
struct Cutting {
    shape: Handle<Mesh>,
    edge: Vec<(Vec3, Vec3)>,
    swept: f32,
    grip: Vec3,
}

/// The trail of a cut (`SwordSwing::Chop`): how long ago the cut began,
/// and the colour it fades from. Its shape is its bearer's to grow
/// ([`Cutting`]) until the blade is down.
#[derive(Component)]
struct Cut {
    age: f32,
    tint: Color,
}

/// A sword in a hand.
#[derive(Component)]
struct Sword;

/// The trail of a slash: how long ago the slash began, the colour it
/// fades from, and how far round it has grown, in steps.
#[derive(Component)]
struct Trail {
    age: f32,
    tint: Color,
    step: usize,
}

/// The shapes of a slash's trail, as it grows behind the blade: the arc
/// the blade has swept so far, a step further in each ([`TRAIL_STEPS`]),
/// faint where the slash began and solid at the blade.
#[derive(Resource)]
struct TrailLook(Vec<Handle<Mesh>>);

/// A body with a shield round it: the ball, and which tier it is.
#[derive(Component)]
struct Shielded {
    bubble: Entity,
    tier: u8,
}

/// The ball of force of a shield.
#[derive(Component)]
struct Bubble;

/// What the balls of force are made of, by tier, and their shape.
#[derive(Resource)]
struct BubbleLook {
    ball: Handle<Mesh>,
    looks: [Handle<StandardMaterial>; 3],
}

/// A part of someone drawn see-through while they are unseen: the material
/// it is really made of, to give back, and whose part it is.
#[derive(Component)]
struct Ghosted {
    real: Handle<StandardMaterial>,
    of: Entity,
}

/// The see-through copy of each material a part of you is made of.
#[derive(Resource, Default)]
struct Ghosts(HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>);

/// The dark over the screen while you are blinded.
#[derive(Component)]
struct Blindfold;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Ghosts>().add_systems(Startup, setup).add_systems(
        Update,
        (
            // Once the game has said who wears what this frame.
            (arm, shield, ghost).after(ArenaSystems),
            // Once the limbs are where they are this frame.
            (hold_swords, fade_trails, fade_cuts).chain().after(crate::animate_walk).after(arm),
            // The camera's place for this frame is set by then.
            blindfold.after(crate::follow_camera),
            restyle,
        ),
    );
}

fn setup(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.arena;
    commands.insert_resource(BubbleLook {
        ball: meshes.add(Sphere::new(1.0).mesh().ico(5).expect("a sphere")),
        looks: [1, 2, 3].map(|tier| materials.add(bubble(colours.shield(tier)))),
    });
    let steps = (1..=TRAIL_STEPS).map(|step| meshes.add(trail(step as f32 / TRAIL_STEPS as f32)));
    commands.insert_resource(TrailLook(steps.collect()));
}

/// The arc a slash has swept `so_far` of the way through its sweep, 0 to
/// 1, lying flat: from where the blade begins, to the right of straight
/// ahead (-z), round toward where it ends, to the left, between the
/// trail's two radii; its colour white, nothing of it where the slash
/// began and all of it at the blade.
fn trail(so_far: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    const STEPS: usize = 24;
    let (mut positions, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    let swept = (crate::SLASH_TO - crate::SLASH_FROM) * so_far.clamp(0.0, 1.0);
    for step in 0..=STEPS {
        let along = step as f32 / STEPS as f32;
        let way = Quat::from_rotation_y(crate::SLASH_FROM + swept * along) * Vec3::NEG_Z;
        for radius in [TRAIL_INNER, TRAIL_OUTER] {
            positions.push((way * radius).to_array());
            colours.push([1.0, 1.0, 1.0, along.powf(1.6)]);
        }
        if step > 0 {
            let at = (step * 2) as u32;
            indices.extend([at - 2, at - 1, at, at, at - 1, at + 1]);
        }
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// Which of a trail's shapes it has grown to, the slash `swept` of the
/// way through the time its sweep takes: as far round as the blade is,
/// which eases through its sweep (`crate::slash_yaw`), to the step above.
fn trail_step(swept: f32) -> usize {
    let round = (crate::slash_yaw(swept.clamp(0.0, 1.0)) - crate::SLASH_FROM) / (crate::SLASH_TO - crate::SLASH_FROM);
    ((round * TRAIL_STEPS as f32).ceil() as usize).clamp(1, TRAIL_STEPS) - 1
}

/// What a trail in `tint` is made of, `left` of it still to fade, 0 to 1:
/// unlit and brighter than its colour, as the punch's range is, from
/// either side.
fn trail_look(tint: Color, left: f32) -> StandardMaterial {
    let mut look = crate::range::look(tint.with_alpha(TRAIL_ALPHA * left.clamp(0.0, 1.0)));
    look.fog_enabled = false;
    look
}

/// What a ball of force in `colour` is made of: that colour, as
/// see-through as its alpha says, smooth enough to catch the light as a
/// bubble does, and giving off a little of its own colour, so that it is
/// there by night as well.
fn bubble(colour: Color) -> StandardMaterial {
    let linear = colour.to_linear();
    StandardMaterial {
        base_color: colour,
        emissive: LinearRgba::rgb(linear.red, linear.green, linear.blue) * 0.25,
        perceptual_roughness: 0.12,
        reflectance: 0.7,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

/// Takes the shields' colours again as the file changes on desktop.
fn restyle(changed: Res<UiChanged>, look: Res<BubbleLook>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colours = ui().colours.arena;
    for (tier, handle) in (1..).zip(&look.looks) {
        if let Some(mut material) = materials.get_mut(handle) {
            *material = bubble(colours.shield(tier));
        }
    }
}

/// Gives every body the sword it has (`Wears::sword`) and takes away the
/// one it has no longer: the model for its tier, a child of the body,
/// which [`hold_swords`] puts in its hand.
fn arm(
    assets: Res<AssetServer>,
    bodies: Query<(Entity, Option<&Wears>, Option<&Armed>), With<WalkCycle>>,
    swords: Query<(), With<Sword>>,
    mut commands: Commands,
) {
    for (body, wears, armed) in &bodies {
        let tier = wears.map_or(0, |wears| wears.sword.min(3));
        // The sword it has, if that is still there: a body given another
        // model loses what hung on the old one.
        let armed = armed.filter(|armed| swords.contains(armed.sword));
        if armed.map_or(0, |armed| armed.tier) == tier {
            continue;
        }
        if let Some(armed) = armed {
            commands.entity(armed.sword).try_despawn();
        }
        if tier == 0 {
            commands.entity(body).remove::<Armed>();
            continue;
        }
        let model = SWORDS[usize::from(tier) - 1];
        let sword = commands
            .spawn((
                Sword,
                Name::new("Sword"),
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(model))),
                Transform::default(),
                // Until it is in the hand.
                Visibility::Hidden,
                ChildOf(body),
            ))
            .id();
        commands.entity(body).insert(Armed {
            sword,
            tier,
            hand: None,
            cutting: None,
        });
    }
}

/// Puts every sword in its bearer's right hand, as the limbs stand this
/// frame, pointing the way its bearer's pose asks, flat side up, or, held
/// on its edge, with its flat square to its cut (the file's
/// `arena.sword_swing`); and puts it away while they carry a plank or
/// drink. A slash begun leaves its trail: swept flat, the arc it is about
/// to sweep ([`Trail`]); cut on its edge, the ribbon the blade cuts, grown
/// here as it comes down ([`Cut`]).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn hold_swords(
    time: Res<Time>,
    trail: Res<TrailLook>,
    mut bearers: Query<(Entity, &mut Armed, Option<&Punching>, Has<Carrying>, Has<Drinking>)>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    places: Query<&Transform, Without<Sword>>,
    mut swords: Query<(&mut Transform, &mut Visibility), With<Sword>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let file = ui();
    let melee = file.melee;
    let swing = file.arena.sword_swing;
    // A slash begun this frame is as old as the frame.
    let fresh = time.delta_secs() + 1e-5;
    for (body, mut armed, punch, carrying, drinking) in &mut bearers {
        let Ok((mut place, mut shown)) = swords.get_mut(armed.sword) else {
            continue;
        };
        // The hand, found in the rig once, and again should the body have
        // been given another model since.
        if armed.hand.is_none_or(|hand| !places.contains(hand)) {
            armed.hand = children
                .iter_descendants(body)
                .find(|&part| names.get(part).is_ok_and(|name| name.as_str() == HAND));
        }
        let hand = armed.hand.and_then(|hand| in_body(hand, body, &parents, &places));
        let Some(hand) = hand.filter(|_| !carrying && !drinking) else {
            shown.set_if_neq(Visibility::Hidden);
            // Whatever cut it was in the middle of is left as it is.
            armed.cutting = None;
            continue;
        };
        let slash = crate::slashing(&melee, punch);
        let grip = hand.transform_point(Vec3::Y * PALM);
        let (rotation, scale) = match swing {
            SwordSwing::Sweep => (pointing(crate::blade_way(slash)), SWORD_SCALE),
            SwordSwing::Chop => {
                let (along, flat) = crate::chop_blade(crate::Guard::of(&file.arena), slash);
                (edge_on(along, flat), CHOP_SCALE)
            }
        };
        place.set_if_neq(Transform {
            translation: grip,
            rotation,
            scale,
        });
        shown.set_if_neq(Visibility::Inherited);
        let begun = punch.is_some_and(|punch| punch.age <= fresh);
        let tint = file.colours.arena.sword(armed.tier);
        match swing {
            // A slash begun: its trail, where the blade is about to sweep.
            SwordSwing::Sweep => {
                armed.cutting = None;
                if begun {
                    commands.spawn((
                        Trail { age: 0.0, tint, step: 0 },
                        Name::new("Slash"),
                        Mesh3d(trail.0[0].clone()),
                        MeshMaterial3d(materials.add(trail_look(tint, 1.0))),
                        Transform::from_xyz(0.0, TRAIL_HEIGHT, 0.0),
                        NotShadowCaster,
                        NotShadowReceiver,
                        ChildOf(body),
                    ));
                }
            }
            SwordSwing::Chop => {
                // A cut begun: its trail, from where the blade is raised.
                if begun {
                    let cutting = Cutting::begun(meshes.add(ribbon(&[])), grip, reach_of(armed.tier));
                    commands.spawn((
                        Cut { age: 0.0, tint },
                        Name::new("Cut"),
                        Mesh3d(cutting.shape.clone()),
                        MeshMaterial3d(materials.add(trail_look(tint, 1.0))),
                        Transform::default(),
                        NotShadowCaster,
                        NotShadowReceiver,
                        ChildOf(body),
                    ));
                    armed.cutting = Some(cutting);
                }
                // And as the blade comes down, the trail grows behind it.
                let reach = reach_of(armed.tier);
                let done = match (armed.cutting.as_mut(), slash) {
                    (Some(cutting), Some((swept, _))) => {
                        if cutting.follow(swept, grip, reach) {
                            // The shape is the trail's own: it is still
                            // there unless the trail has gone already.
                            let _ = meshes.insert(&cutting.shape, ribbon(&cutting.edge));
                        }
                        swept >= 1.0
                    }
                    _ => true,
                };
                if done {
                    armed.cutting = None;
                }
            }
        }
    }
}

/// From how far up a sword of `tier`'s blade to how far, from its grip,
/// the trail of a cut is, in metres as the sword is drawn: from a little
/// way up it to its tip.
fn reach_of(tier: u8) -> (f32, f32) {
    let tip = BLADE[usize::from(tier.clamp(1, 3)) - 1] * CHOP_SCALE.y;
    (tip * CUT_FROM, tip)
}

impl Cutting {
    /// A cut begun, the blade raised, with the grip at `grip`: its trail,
    /// whose shape is `shape`, begins where the blade is, `reach` along it.
    fn begun(shape: Handle<Mesh>, grip: Vec3, reach: (f32, f32)) -> Self {
        let (along, _) = crate::cut_blade_at(crate::cut_angle(0.0));
        Self {
            shape,
            edge: vec![(grip + along * reach.0, grip + along * reach.1)],
            swept: 0.0,
            grip,
        }
    }

    /// The cut is `swept` of the way through its sweep now, 0 to 1, with
    /// the grip at `grip`: wherever the blade has been since the last
    /// sample is added to the trail, a sample to every [`CUT_STEP`] it has
    /// turned (evenly by its turn, not by the time: the blade whips
    /// through the middle of its cut), the grip taken to have gone
    /// straight from where it was to where it is. Whether anything was
    /// added.
    fn follow(&mut self, swept: f32, grip: Vec3, reach: (f32, f32)) -> bool {
        let swept = swept.clamp(0.0, 1.0);
        if swept <= self.swept {
            return false;
        }
        let (from, to) = (crate::cut_angle(self.swept), crate::cut_angle(swept));
        let steps = (((to - from).abs() / CUT_STEP).ceil() as usize).clamp(1, 64);
        for step in 1..=steps {
            let share = step as f32 / steps as f32;
            let (along, _) = crate::cut_blade_at(from + (to - from) * share);
            let grip = self.grip.lerp(grip, share);
            self.edge.push((grip + along * reach.0, grip + along * reach.1));
        }
        self.swept = swept;
        self.grip = grip;
        true
    }
}

/// The ribbon a blade has cut, through where it has been (`edge`, a pair
/// to each sample, the nearer the grip first): white, nothing of it where
/// the cut began and all of it at the blade, as the other trail is
/// ([`trail`]). With fewer than two samples there is nothing of it yet.
fn ribbon(edge: &[(Vec3, Vec3)]) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let (mut positions, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    let last = edge.len().saturating_sub(1).max(1) as f32;
    for (sample, (inner, outer)) in edge.iter().enumerate() {
        let along = sample as f32 / last;
        for point in [inner, outer] {
            positions.push(point.to_array());
            colours.push([1.0, 1.0, 1.0, along.powf(CUT_RAMP)]);
        }
        if sample > 0 {
            let at = (sample * 2) as u32;
            indices.extend([at - 2, at - 1, at, at, at - 1, at + 1]);
        }
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// Fades every cut's trail once its blade is down, and takes it away:
/// as [`fade_trails`] does the other's, whose shapes are made once. A
/// cut's shape is its own, and goes with it.
fn fade_cuts(
    time: Res<Time>,
    mut cuts: Query<(Entity, &mut Cut, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let sweep = ui().melee.swing_out;
    for (entity, mut cut, look) in &mut cuts {
        cut.age += time.delta_secs();
        let left = 1.0 - (cut.age - sweep) / TRAIL_FADE;
        if left <= 0.0 {
            materials.remove(&look.0);
            commands.entity(entity).despawn();
        } else if left < 1.0
            && let Some(mut material) = materials.get_mut(&look.0)
        {
            *material = trail_look(cut.tint, left);
        }
    }
}

/// Grows every slash's trail behind its blade as the blade sweeps, fades
/// it once it has swept, and takes it away.
fn fade_trails(
    time: Res<Time>,
    shapes: Res<TrailLook>,
    mut trails: Query<(Entity, &mut Trail, &mut Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let melee = ui().melee;
    let sweep = melee.swing_out;
    for (entity, mut trail, mut shape, look) in &mut trails {
        trail.age += time.delta_secs();
        // As far round as the blade is now.
        let step = trail_step(trail.age / sweep);
        if step != trail.step {
            trail.step = step;
            shape.0 = shapes.0[step].clone();
        }
        let left = 1.0 - (trail.age - sweep) / TRAIL_FADE;
        if left <= 0.0 {
            materials.remove(&look.0);
            commands.entity(entity).despawn();
        } else if left < 1.0
            && let Some(mut material) = materials.get_mut(&look.0)
        {
            *material = trail_look(trail.tint, left);
        }
    }
}

/// Where `node`, somewhere under `body`, is in `body`'s own space, as
/// their transforms stand now.
fn in_body(mut node: Entity, body: Entity, parents: &Query<&ChildOf>, places: &Query<&Transform, Without<Sword>>) -> Option<Transform> {
    let mut place = *places.get(node).ok()?;
    // No rig is this deep.
    for _ in 0..32 {
        let parent = parents.get(node).ok()?.parent();
        if parent == body {
            return Some(place);
        }
        place = places.get(parent).ok()?.mul_transform(place);
        node = parent;
    }
    None
}

/// The turn that points a sword's blade, its own +y, along `blade`, with
/// its flat side, its own z, up as far as that allows.
fn pointing(blade: Vec3) -> Quat {
    let y = blade.normalize_or(Vec3::NEG_Z);
    let z = (Vec3::Y - y * Vec3::Y.dot(y)).normalize_or(Vec3::Z);
    Quat::from_mat3(&Mat3::from_cols(y.cross(z), y, z))
}

/// The turn that points a sword's blade, its own +y, along `along`, with
/// its flat side, its own z, facing `flat`, as near as a flat square to
/// its blade can: a sword held on its edge, its flat square to the plane
/// it cuts in.
fn edge_on(along: Vec3, flat: Vec3) -> Quat {
    let y = along.normalize_or(Vec3::NEG_Z);
    let z = (flat - y * flat.dot(y)).normalize_or(y.any_orthonormal_vector());
    Quat::from_mat3(&Mat3::from_cols(y.cross(z), y, z))
}

/// Puts a ball of force round every body with a shield (`Wears::shield`),
/// in its tier's colour, and takes it away from one with none.
fn shield(
    look: Res<BubbleLook>,
    bodies: Query<(Entity, Option<&Wears>, Option<&Shielded>), With<WalkCycle>>,
    mut bubbles: Query<&mut MeshMaterial3d<StandardMaterial>, With<Bubble>>,
    mut commands: Commands,
) {
    for (body, wears, shielded) in &bodies {
        let tier = wears.map_or(0, |wears| wears.shield.min(3));
        // The ball it has, if that is still there, as with a sword.
        let shielded = shielded.filter(|shielded| bubbles.contains(shielded.bubble));
        if shielded.map_or(0, |shielded| shielded.tier) == tier {
            continue;
        }
        let material = || look.looks[usize::from(tier) - 1].clone();
        match shielded {
            Some(shielded) if tier == 0 => {
                commands.entity(shielded.bubble).try_despawn();
                commands.entity(body).remove::<Shielded>();
            }
            // A better shield: the same ball in another colour.
            Some(shielded) => {
                if let Ok(mut worn) = bubbles.get_mut(shielded.bubble) {
                    worn.0 = material();
                }
                commands.entity(body).insert(Shielded {
                    bubble: shielded.bubble,
                    tier,
                });
            }
            None => {
                let bubble = commands
                    .spawn((
                        Bubble,
                        Name::new("Shield"),
                        Mesh3d(look.ball.clone()),
                        MeshMaterial3d(material()),
                        Transform::from_xyz(0.0, BUBBLE_HEIGHT, 0.0).with_scale(Vec3::splat(BUBBLE_RADIUS)),
                        NotShadowCaster,
                        NotShadowReceiver,
                        ChildOf(body),
                    ))
                    .id();
                commands.entity(body).insert(Shielded { bubble, tier });
            }
        }
    }
}

/// Draws you see-through while nobody of another team sees you
/// (`Wears::unseen`), and anyone of your own team who is unseen the same:
/// each part of them, their sword with it, in a copy of its own material
/// at [`GHOST`] of itself; and gives every part its own back once they are
/// seen again. The shield round them is see-through as it is.
#[allow(clippy::type_complexity)]
fn ghost(
    me: Query<Option<&Team>, With<Player>>,
    bodies: Query<(Entity, &Wears, Option<&Team>, Has<Player>)>,
    children: Query<&Children>,
    parts: Query<&MeshMaterial3d<StandardMaterial>, (Without<Ghosted>, Without<Bubble>)>,
    ghosted: Query<(Entity, &Ghosted)>,
    mut ghosts: ResMut<Ghosts>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mine = me.single().ok().flatten();
    let unseen: Vec<Entity> = bodies
        .iter()
        .filter(|(_, wears, team, you)| wears.unseen && (*you || (mine.is_some() && *team == mine)))
        .map(|(body, ..)| body)
        .collect();
    for (part, was) in &ghosted {
        if !unseen.contains(&was.of) {
            commands.entity(part).insert(MeshMaterial3d(was.real.clone())).remove::<Ghosted>();
        }
    }
    for body in unseen {
        for part in children.iter_descendants(body) {
            let Ok(real) = parts.get(part) else {
                continue;
            };
            let faint = match ghosts.0.get(&real.0.id()) {
                Some(faint) => faint.clone(),
                None => {
                    // Not until the model has loaded.
                    let Some(mut look) = materials.get(&real.0).cloned() else {
                        continue;
                    };
                    look.base_color = look.base_color.with_alpha(look.base_color.alpha() * GHOST);
                    look.alpha_mode = AlphaMode::Blend;
                    let faint = materials.add(look);
                    ghosts.0.insert(real.0.id(), faint.clone());
                    faint
                }
            };
            commands.entity(part).insert((
                Ghosted {
                    real: real.0.clone(),
                    of: body,
                },
                MeshMaterial3d(faint),
            ));
        }
    }
}

/// Covers the screen while a trap has you blinded (`Outfit::kit`), all but
/// a circle round you as far as a blinded player sees: one node with a
/// round hole, the hole kept over you as the camera has you this frame.
/// Under the buttons and the minimap, over the world and what is written
/// over it.
#[allow(clippy::type_complexity)]
fn blindfold(
    outfit: Res<Outfit>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    mut folds: Query<(Entity, &mut Node, &mut UiTransform, &mut BorderColor), With<Blindfold>>,
    mut commands: Commands,
) {
    let seen = outfit.kit.blind.then_some(()).and_then(|()| {
        let (body, &venue, dead) = me.single().ok()?;
        let (camera, eye) = cameras.single().ok()?;
        let window = windows.single().ok()?;
        if venue != Venue::Arena || dead {
            return None;
        }
        // Where you are on the screen, and how far across it a blinded
        // player's sight reaches, along the camera's own right.
        let eye_place = GlobalTransform::from(*eye);
        let middle = body.translation + Vec3::Y * BLIND_CENTRE;
        let centre = camera.world_to_viewport(&eye_place, middle).ok()?;
        let edge = camera.world_to_viewport(&eye_place, middle + eye.right() * BLIND_SIGHT).ok()?;
        Some((centre, centre.distance(edge), Vec2::new(window.width(), window.height())))
    });
    let Some((centre, radius, screen)) = seen else {
        for (fold, ..) in &folds {
            commands.entity(fold).despawn();
        }
        return;
    };
    // As thick as reaches every corner of the screen from anywhere on it.
    let (radius, thick) = (radius.round(), screen.length().ceil());
    let side = 2.0 * (radius + thick);
    let colour = BorderColor::all(ui().colours.arena.blind);
    let Ok((_, mut node, mut place, mut border)) = folds.single_mut() else {
        commands.spawn((
            InArena,
            Blindfold,
            Name::new("Blindfold"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Px(side),
                height: Val::Px(side),
                border: UiRect::all(Val::Px(thick)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            colour,
            UiTransform {
                translation: Val2::px(centre.x - side * 0.5, centre.y - side * 0.5),
                scale: Vec2::ONE,
                rotation: Rot2::IDENTITY,
            },
            // Over the names, the numbers and the words in the world
            // (which are a layer under the screen's own), and under the
            // screen's own.
            GlobalZIndex(-1),
            ZIndex(1),
            Pickable::IGNORE,
        ));
        return;
    };
    // The node itself only as the window or the camera's height changes:
    // writing it has the screen laid out again.
    if node.width != Val::Px(side) || node.border.left != Val::Px(thick) {
        node.width = Val::Px(side);
        node.height = Val::Px(side);
        node.border = UiRect::all(Val::Px(thick));
    }
    place.set_if_neq(UiTransform {
        translation: Val2::px(centre.x - side * 0.5, centre.y - side * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    border.set_if_neq(colour);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sword_points_where_it_is_asked_to_flat_side_up() {
        for blade in [crate::blade_way(None), Vec3::NEG_Z, Vec3::X, Vec3::new(-0.7, 0.0, -0.7), Vec3::new(0.3, 0.9, -0.2)] {
            let turn = pointing(blade);
            assert!((turn * Vec3::Y).distance(blade.normalize()) < 1e-5, "{blade}");
            // Its flat side as near up as a blade that way allows, and
            // never down.
            let flat = turn * Vec3::Z;
            assert!(flat.y >= 0.0 && flat.dot(blade.normalize()).abs() < 1e-5, "{flat}");
            assert!((turn.length() - 1.0).abs() < 1e-5);
        }
        // Level, its flat side is straight up.
        assert!((pointing(Vec3::NEG_Z) * Vec3::Z).distance(Vec3::Y) < 1e-5);
        // Straight up, there is no up left to turn it to: it is still a
        // turn, and the blade still points up.
        assert!((pointing(Vec3::Y) * Vec3::Y).distance(Vec3::Y) < 1e-5);
    }

    #[test]
    fn a_sword_on_its_edge_has_its_flat_square_to_its_cut() {
        let melee = ui().melee;
        for age in [None, Some(0.0), Some(melee.swing_out * 0.5), Some(melee.swing_out), Some(melee.swing_out + melee.swing_back * 0.5)] {
            let slash = age.map(|age: f32| ((age / melee.swing_out).clamp(0.0, 1.0), ((age - melee.swing_out) / melee.swing_back).clamp(0.0, 1.0)));
            let (along, flat) = crate::chop_blade(crate::Guard::tilted(30.0, 10.0), slash);
            let turn = edge_on(along, flat);
            assert!((turn * Vec3::Y).distance(along) < 1e-5, "{along}");
            assert!((turn * Vec3::Z).distance(flat) < 1e-5, "{flat}");
            assert!((turn.length() - 1.0).abs() < 1e-5);
            // Its breadth, the guard's way, is in the plane of the cut:
            // more up and down than to the side, which is what "narrow"
            // from over you is.
            let breadth = turn * Vec3::X;
            assert!(breadth.dot(flat).abs() < 1e-5 && breadth.x.abs() < 0.6, "{breadth}");
        }
        // A flat that is not square to the blade is squared to it; one
        // along it leaves some turn that still points the blade.
        let turn = edge_on(Vec3::NEG_Z, Vec3::new(1.0, 0.0, -1.0));
        assert!((turn * Vec3::Y).distance(Vec3::NEG_Z) < 1e-5 && (turn * Vec3::Z).distance(Vec3::X) < 1e-5);
        assert!((edge_on(Vec3::X, Vec3::X) * Vec3::Y).distance(Vec3::X) < 1e-5);
    }

    #[test]
    fn a_cut_s_trail_is_the_ribbon_the_blade_has_cut_whatever_the_frame_rate() {
        let reach = reach_of(1);
        assert!(reach.0 > 0.2 && reach.0 < reach.1 && reach.1 > 1.2 && reach.1 < 1.5, "{reach:?}");
        // A longer sword cuts a longer trail.
        assert!(reach_of(2).1 > reach_of(3).1 && reach_of(3).1 > reach_of(1).1);
        let grip = |swept: f32| Vec3::new(0.3, 1.5 - 0.6 * swept, -0.3 - 0.3 * swept);
        let cut_in = |frames: usize| {
            let mut cutting = Cutting::begun(Handle::default(), grip(0.0), reach);
            for frame in 1..=frames {
                let swept = frame as f32 / frames as f32;
                assert!(cutting.follow(swept, grip(swept), reach));
            }
            // Nothing more once it is down, nor going back.
            assert!(!cutting.follow(1.0, grip(1.0), reach) && !cutting.follow(0.4, grip(0.4), reach));
            cutting
        };
        let (smooth, coarse) = (cut_in(30), cut_in(3));
        for cutting in [&smooth, &coarse] {
            // No two samples further apart than a step of the blade's
            // turn, however few frames the cut took.
            let whole = crate::CHOP_FROM - crate::CHOP_TO;
            assert!(cutting.edge.len() as f32 >= whole / CUT_STEP, "{} samples", cutting.edge.len());
            for pair in cutting.edge.windows(2) {
                let (before, after) = ((pair[0].1 - pair[0].0).normalize(), (pair[1].1 - pair[1].0).normalize());
                assert!(before.angle_between(after) <= CUT_STEP + 1e-3);
            }
            // From the blade raised, up and back over the right shoulder,
            // to the blade down in front, to the left: each sample as long
            // as the trail's share of the blade.
            let (first, last) = (cutting.edge[0], cutting.edge[cutting.edge.len() - 1]);
            let (from, to) = ((first.1 - first.0).normalize(), (last.1 - last.0).normalize());
            assert!(from.distance(crate::cut_blade_at(crate::CHOP_FROM).0) < 1e-4 && to.distance(crate::cut_blade_at(crate::CHOP_TO).0) < 1e-4);
            assert!(from.y > 0.6 && to.y < -0.3 && to.z < -0.8 && to.x < 0.0, "{from} to {to}");
            assert!(cutting.edge.iter().all(|(inner, outer)| (inner.distance(*outer) - (reach.1 - reach.0)).abs() < 1e-4));
            assert!(first.0.distance(grip(0.0) + from * reach.0) < 1e-4 && last.1.distance(grip(1.0) + to * reach.1) < 1e-4);
        }
        // Its shape: two corners to a sample, two triangles between each
        // two, nothing of it where the cut began and all of it at the
        // blade, and more the further on.
        let mesh = ribbon(&smooth.edge);
        let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).and_then(|positions| positions.as_float3()).expect("positions");
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("the trail has no colours");
        };
        assert_eq!((points.len(), colours.len()), (smooth.edge.len() * 2, smooth.edge.len() * 2));
        assert_eq!(mesh.indices().expect("indices").len(), (smooth.edge.len() - 1) * 6);
        assert_eq!((colours[0][3], colours[colours.len() - 1][3]), (0.0, 1.0));
        assert!(colours.windows(3).step_by(2).all(|three| three[2][3] >= three[0][3]));
        // Seen from over you it is a crescent with some breadth to it,
        // from out on the right round to in front: not a line.
        let across = points.iter().map(|point| point[0]).fold(f32::NEG_INFINITY, f32::max) - points.iter().map(|point| point[0]).fold(f32::INFINITY, f32::min);
        let ahead = points.iter().map(|point| -point[2]).fold(f32::NEG_INFINITY, f32::max);
        assert!(across > 0.8 && ahead > 1.4, "{across} m across, {ahead} m ahead");
        // With one sample, or none, there is nothing of it.
        assert_eq!(ribbon(&smooth.edge[..1]).indices().expect("indices").len(), 0);
        assert_eq!(ribbon(&[]).indices().expect("indices").len(), 0);
        // The blades are as long as their models: the trail ends at the tip.
        for (name, blade) in SWORDS.iter().zip(BLADE) {
            let parts = crate::island::glb::read(name);
            let tip = parts.iter().flat_map(|part| part.triangles.iter().flatten()).map(|corner| corner.y).fold(f32::NEG_INFINITY, f32::max);
            assert!((tip - blade).abs() < 0.02, "{name}: its tip is {tip} up, not {blade}");
        }
    }

    #[test]
    fn every_sword_has_a_model_in_the_apk_with_its_blade_along_its_own_y() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim_start_matches("<dir>").trim_start_matches("<file>"))
            .collect();
        for name in SWORDS {
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
            assert!(left_out.is_none(), "{name} is left out of the APK by `{}`", left_out.unwrap());
            // As the game places it: the blade up its own y from a grip at
            // its origin, about a metre of it; the guard across its x; and
            // thin, its flat side its z.
            let parts = crate::island::glb::read(name);
            let corners: Vec<Vec3> = parts.iter().flat_map(|part| part.triangles.iter().flatten().copied()).collect();
            assert!(!corners.is_empty(), "{name}");
            let (low, high) = corners
                .iter()
                .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), corner| (low.min(*corner), high.max(*corner)));
            let size = high - low;
            assert!(high.y > 0.7 && high.y < 1.0 && low.y < -0.05 && low.y > -0.3, "{name}: from {} to {}", low.y, high.y);
            assert!(size.x > 0.25 && size.x < 0.4, "{name}: {} across its guard", size.x);
            assert!(size.z < 0.15 && size.z < size.x, "{name}: {} thick", size.z);
        }
        // The ball of force clears the body it is round: its head, 1.64 m
        // up, and its shoulders.
        assert!(BUBBLE_HEIGHT + BUBBLE_RADIUS > 1.75 && BUBBLE_RADIUS > 0.9);
    }

    #[test]
    fn a_slash_s_trail_is_the_arc_the_blade_sweeps_solid_where_it_ends() {
        // It grows behind the blade, a step at a time, to the whole sweep:
        // a sliver as the slash begins, half of it as the blade passes
        // straight ahead, all of it at the end and after.
        assert_eq!((trail_step(0.0), trail_step(0.5), trail_step(1.0), trail_step(3.0)), (0, TRAIL_STEPS / 2 - 1, TRAIL_STEPS - 1, TRAIL_STEPS - 1));
        assert!((0..=20).map(|k| trail_step(k as f32 / 20.0)).is_sorted());
        let reach_of = |mesh: &Mesh| -> f32 {
            let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).and_then(|positions| positions.as_float3()).expect("positions");
            points.iter().map(|point| point[0]).fold(f32::INFINITY, f32::min)
        };
        assert!(reach_of(&trail(0.125)) > 0.5, "an eighth of the way it is still to the right");
        assert!(reach_of(&trail(0.5)) > -0.2 && reach_of(&trail(0.5)) < 0.5);
        let mesh = trail(1.0);
        let points: Vec<Vec3> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .expect("positions")
            .iter()
            .map(|&point| Vec3::from(point))
            .collect();
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("the trail has no colours");
        };
        assert_eq!(points.len(), colours.len());
        // Flat, between its two radii, in front of the body from its right
        // (+x) round to its left.
        for point in &points {
            let far = point.length();
            assert!(point.y == 0.0 && (far - TRAIL_INNER).abs().min((far - TRAIL_OUTER).abs()) < 1e-4, "{point}");
        }
        let (first, last) = (points[1], points[points.len() - 1]);
        assert!(first.x > 1.5 && last.x < -1.4 && first.z < 0.0 && last.z < 0.0, "{first} to {last}");
        assert!(points.iter().any(|point| point.z < -TRAIL_OUTER + 0.05), "it passes straight ahead");
        // Nothing of it where the slash began, all of it where it ends,
        // and more the further round.
        assert_eq!((colours[0][3], colours[colours.len() - 1][3]), (0.0, 1.0));
        assert!(colours.windows(3).step_by(2).all(|three| three[2][3] >= three[0][3]));
        // It reaches as far as a punch does, about, and no further.
        let reach = ui().melee.reach;
        assert!(TRAIL_OUTER <= reach + 0.2 && TRAIL_OUTER > reach * 0.8, "{reach}");
        // Fading, it is fainter and fainter.
        let white = Color::WHITE;
        assert!(trail_look(white, 1.0).base_color.alpha() > trail_look(white, 0.5).base_color.alpha());
        assert_eq!(trail_look(white, 0.0).base_color.alpha(), 0.0);
    }
}
