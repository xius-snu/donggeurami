//! How a game of Last Stand looks in the world, judged from the camera
//! fixed over you (Hajun: "similar cam and controls with bridgewars"),
//! the same online and offline: everything here is drawn from how the game
//! stands ([`Stand`]) and from the zombies' bodies, each carrying its kind
//! and health ([`Shambler`]), whoever moves them.
//!
//! * **The zombies** ([`dress`]): Bridge Wars' zombie, the boy painted
//!   green, each kind in a tint of its own ([`paint`]; the file's
//!   `colours.last_stand`), a brute a third bigger and a giant twice as big
//!   ([`size`]; the rules' `Kind::size`), their legs swinging the slower
//!   the bigger they are ([`stride`]), so that a giant's footfalls are
//!   heavy; each with its name, if its kind has one in the file, over a
//!   bar of its health (the kit's tags, `arena::place_tags`), and a number
//!   floating off it for every blow that takes any ([`hurt`]). They come
//!   up out of the void at the sand's edge, climbing over the rim, as the
//!   game has them.
//! * **The anchor's crystal** (the map's `spawn` pillar, [`Crystal`]): a
//!   bar over it, wider than a body's ([`anchor_bar`]); a white flash
//!   over it as it is hit, with a number of what the blows took, and a
//!   green one as a Fortify brings it all back; and once broken it
//!   shatters, its pieces flying out and falling, and it is gone, from
//!   sight and from underfoot ([`anchor`]); stood up again as the game goes
//!   and as the next begins. The beam, bought: a short bright line from
//!   the crystal to whatever it zaps ([`beam`]).
//! * **A spitter's ball** ([`SpitBall`]): a small green ball arcing from the
//!   spitter to where it comes down, and a green splash there.
//! * **The generators**: over each pad, what it makes ([`over_the_pads`]:
//!   "Lvl 2: gold, sapphire"); what it makes comes up on the pad as Bridge
//!   Wars' things do (`loot`).

use std::collections::HashMap;

use bevy::gltf::GltfMaterialName;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::ZOMBIE_TEAM;
use roundtown_net::last_stand::{self as rules, ANCHOR, CRYSTAL_RADIUS, Forges, Kind, PADS, PLINTH_TOP, SPIT_RADIUS, Shambler, Spit, Spittle};
use roundtown_net::Item;

use super::{Happened, Stand, StandSystems};
use crate::arena::{self, Health, InArena};
use crate::island::{Islands, Land, Venue, is_material};
use crate::lobby::WalkFromMotion;
use crate::market::Shops;
use crate::net::Online;
use crate::ui::{UiChanged, fill, ui};
use crate::{Player, Team, ThirdPersonCamera, WalkBone, WalkCycle};

/// The crystal's bar stands on the screen over the crystal as the screen
/// has it: over the highest on the screen of the crystal's foot and its
/// tip, some 3.5 m up, each as wide as the crystal, by a gap, in shares of
/// the short side of the screen. Seen from straight over it, the tip is
/// drawn further out from the middle of the screen than the foot, the
/// more the further out the crystal is: a bar stood a set way up the
/// screen from a point over the tip lay on the crystal once it was low on
/// the screen. How wide and tall the bar is, in shares of the short side
/// of the screen: half as wide again as Bridge Wars' anchors' and twice a
/// body's.
const CRYSTAL_TIP: f32 = PLINTH_TOP + 3.2;
const ANCHOR_BAR_GAP: f32 = 1.5;
const ANCHOR_BAR_WIDTH: f32 = 12.0;
const ANCHOR_BAR_HEIGHT: f32 = 1.3;
const BAR_RIM: f32 = 0.15;
/// How long the crystal's flash lasts, in seconds, and the least while
/// between two (mine: a heap of zombies clawing at it would have it white
/// all the time).
const FLASH_SECS: f32 = 0.18;
const FLASH_EVERY: f32 = 0.35;
/// How much bigger than the crystal its flash is, so as to lie over it and
/// never in it.
const FLASH_OVER: f32 = 1.07;
/// The crystal's pieces as it shatters: how many, how long they fly, how
/// fast they go out and up, in metres a second, and how fast they fall.
const SHARDS: usize = 16;
const SHARD_SECS: f32 = 1.7;
const SHARD_OUT: (f32, f32) = (2.5, 6.0);
const SHARD_UP: (f32, f32) = (3.0, 7.5);
const SHARD_FALL: f32 = 18.0;
/// How high over the anchor the crystal's middle is, where its pieces fly
/// from, in metres.
const CRYSTAL_MIDDLE: f32 = PLINTH_TOP + 1.6;
/// What of the map is the crystal, to be taken away from underfoot once it
/// has shattered: every triangle within this of the anchor across the
/// ground, with some of it higher than this over the plinth's top.
const CRYSTAL_REACH: f32 = CRYSTAL_RADIUS + 0.45;
const CRYSTAL_OVER: f32 = 0.3;
/// The beam: how long it is seen, in seconds, the first part of that at
/// its brightest; how thick it is, and the spark where it lands, in
/// metres; how high it leaves the crystal, just over its tip, and how
/// high over a zombie's feet it lands, just over its head: seen from
/// straight over them, a beam from the crystal's middle to a zombie's
/// chest ran inside the crystal and under the head of the zombie clawing
/// at it, which is where most are zapped, and was not seen at all.
const BEAM_SECS: f32 = 0.5;
const BEAM_HOLD: f32 = 0.2;
const BEAM_WIDTH: f32 = 0.12;
const BEAM_SPARK: f32 = 0.4;
const BEAM_FROM: f32 = PLINTH_TOP + 3.3;
const BEAM_AT: f32 = 1.8;
/// A spitter's ball: how big, in metres across its middle; and its splash,
/// how long it lies, in seconds, and how big it is as it begins, as a share
/// of where the ball splashes whoever is in it.
const SPIT_SIZE: f32 = 0.22;
const SPLAT_SECS: f32 = 1.0;
const SPLAT_FROM: f32 = 0.45;
/// How many times brighter than its colour the spit's ball is drawn,
/// unlit, as a fireball is: seen by day and by night alike.
const SPIT_GAIN: f32 = 1.6;
/// How long after a ball the server sent should have come down it is taken
/// away, if the server has not taken its own away first, in seconds.
const SPIT_SLACK: f32 = 0.6;
/// The slowest a walking body's legs are swung, as a share of a walk's
/// pace (`crate::animate_walk`'s own, written there as it is here).
const SLOWEST_SWING: f32 = 0.35;
/// How far down the screen the plate top centre and the lines under it
/// reach, and how far either side of the middle, in shares of the short
/// side of the screen: what is written over the world there is put away
/// rather than drawn into the plate's words.
const PLATE_DOWN: f32 = 21.0;
const PLATE_ACROSS: f32 = 42.0;

/// Whether `point`, in the window's logical pixels, is where the plate top
/// centre is ([`PLATE_DOWN`]).
fn under_the_plate(window: &Window, point: Vec2) -> bool {
    let vmin = window.width().min(window.height()) / 100.0;
    point.y < PLATE_DOWN * vmin && (point.x - window.width() * 0.5).abs() < PLATE_ACROSS * vmin
}

/// The column of words top left (your name, your health, and what you
/// hold of each of the four, `hud::game_bar`'s left column) where the
/// screen has it laid out: what this game writes over the world is put
/// away where it would stand among its rows, rather than drawn into them,
/// as it is under the plate (mine; seen in a hidden run, 2026-11-18, a
/// pad's "Silver" over the count of gold; Bridge Wars draws its names and
/// its generators' words on under its own).
#[derive(bevy::ecs::system::SystemParam)]
struct Column<'w, 's> {
    bars: Query<'w, 's, &'static Children, With<crate::hud::TopBar>>,
    nodes: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform)>,
}

impl Column<'_, '_> {
    /// Whether something drawn over `rect` of the screen, in the window's
    /// logical pixels, would stand among the column's rows.
    fn covers(&self, rect: Rect) -> bool {
        let Some(&column) = self.bars.iter().next().and_then(|bar| bar.first()) else {
            return false;
        };
        let Ok((node, place)) = self.nodes.get(column) else {
            return false;
        };
        let scale = node.inverse_scale_factor();
        let rows = Rect::from_center_size(place.translation * scale, node.size() * scale);
        !rows.intersect(rect).is_empty()
    }
}

/// What a thing whose foot is at `point` on the screen, over its middle,
/// covers, `size` big.
fn standing_over(point: Vec2, size: Vec2) -> Rect {
    Rect::new(point.x - size.x * 0.5, point.y - size.y, point.x + size.x * 0.5, point.y)
}

/// Whether what this game writes over the world is to be in sight: you on
/// its map, and not standing in a shop's outline, whose window comes down
/// over the middle of the screen (`market`), under which it would show
/// through the window's see-through dark among its cards' words (seen in a
/// hidden run by night, 2026-11-18: a pad's "Silver" by "Heal" and by
/// "Leap"; mine, as the line under the plate is put away there,
/// `screen::show_plate`).
fn in_sight(me: &Query<(&Venue, &Transform), With<Player>>, shops: &Shops) -> bool {
    me.single()
        .is_ok_and(|(venue, at)| *venue == Venue::LastStand && shops.at(Venue::LastStand, at.translation).is_none())
}

/// What is written over each pad: how high over the ground, in metres, and
/// how big, in shares of the short side of the screen.
const PAD_WORDS_OVER: f32 = 2.4;
const PAD_WORDS: f32 = 2.8;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Crystal>()
        .add_observer(read_crystal)
        .add_observer(last_blow)
        .add_systems(Startup, load)
        .add_systems(
            Update,
            (
                (dress, hurt, paint, size, follow_spits).after(StandSystems),
                // After the zombies have been moved, by the game here or
                // the server, and before their legs are swung.
                stride.after(StandSystems).after(crate::move_bodies).before(crate::animate_walk),
                (anchor, beam, fly_spits, shards, splats).after(StandSystems),
                (recolour, (anchor_bar, over_the_pads)).chain().after(crate::follow_camera),
            ),
        );
}

/// What this game draws in the world with, made as the app opens: a ball
/// for the spit, a beam and a crystal's piece, a flat disc for a splash,
/// and the materials those that do not fade share.
#[derive(Resource)]
pub(super) struct Looks {
    ball: Handle<Mesh>,
    spit: Handle<StandardMaterial>,
    beam: Handle<Mesh>,
    shard: Handle<Mesh>,
    disc: Handle<Mesh>,
}

fn load(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.last_stand;
    commands.insert_resource(Looks {
        ball: meshes.add(Sphere::new(1.0).mesh().ico(2).expect("a sphere")),
        // Unlit and brighter than its colour, as a fireball is: it is seen
        // by day and by night alike.
        spit: materials.add(StandardMaterial {
            base_color: bright(colours.spit, SPIT_GAIN),
            unlit: true,
            ..default()
        }),
        beam: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        shard: meshes.add(Tetrahedron::default()),
        disc: meshes.add(Circle::new(1.0)),
    });
}

/// `colour`, `times` as bright, alpha and all: an unlit thing's colour, which
/// the tonemapping takes down again (`fire`'s ball, `sky::UNLIT_GAIN`).
fn bright(colour: Color, times: f32) -> Color {
    let linear = colour.to_linear();
    LinearRgba::new(linear.red * times, linear.green * times, linear.blue * times, linear.alpha).into()
}

/// What of this game's in the world the file colours, drawn again as it
/// changes on desktop (`UiChanged`, the rule for whatever stays on the
/// screen): the spit's ball, whose look is made once as the app opens; and
/// the crystal's bar and the words over the pads, taken down here to be put
/// up again in the file's colours, as each is whenever there is none
/// ([`anchor_bar`], [`over_the_pads`]). (The zombies' paint, the flash, the
/// beam and the splash take theirs as they are drawn; none of these three
/// did until 2026-11-18.)
fn recolour(
    changed: Res<UiChanged>,
    looks: Res<Looks>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    drawn: Query<Entity, Or<(With<AnchorBar>, With<PadWords>)>>,
    mut commands: Commands,
) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    if let Some(mut spit) = materials.get_mut(&looks.spit) {
        spit.base_color = bright(ui().colours.last_stand.spit, SPIT_GAIN);
    }
    for thing in &drawn {
        commands.entity(thing).try_despawn();
    }
}

// -------------------------------------------------------------- the zombies

/// A zombie's body as this game has dressed it: its kind, the health last
/// seen on it, where it was last frame, for its stride, and whether it has
/// been painted and sized yet (each waits for its model).
#[derive(Component)]
pub(super) struct Dressed {
    kind: Kind,
    health: u16,
    last: Vec3,
    painted: bool,
    sized: bool,
}

/// What is written over a zombie of `kind`: the file's (`kinds`; nothing
/// over a walker, the most of them, but its bar).
fn kind_word(kind: Kind) -> String {
    let index = match kind {
        Kind::Walker => 0,
        Kind::Runner => 1,
        Kind::Brute => 2,
        Kind::Spitter => 3,
        Kind::Leaper => 4,
        Kind::Giant => 5,
    };
    ui().words.last_stand.kinds[index].clone()
}

/// Every zombie's body as it comes (one this device made, offline, or the
/// server's, once `net` has given it a body): on the zombies' team, with
/// its health for the bar over it (`arena::place_tags`), and its kind's
/// name; its legs swung by this game ([`stride`]) rather than read off how
/// it moves, which knows nothing of its size.
fn dress(fresh: Query<(Entity, &Transform, &Shambler), (With<WalkCycle>, Without<Dressed>)>, mut commands: Commands) {
    for (body, place, shambler) in &fresh {
        commands
            .entity(body)
            .insert((
                Dressed {
                    kind: shambler.kind,
                    health: shambler.health,
                    last: place.translation,
                    painted: false,
                    sized: false,
                },
                Team(ZOMBIE_TEAM),
                Health::at(f32::from(shambler.health), f32::from(shambler.most.max(1))),
                Name::new(kind_word(shambler.kind)),
            ))
            .remove::<WalkFromMotion>();
    }
}

/// A zombie's health as it changes: its bar, and the damage it took
/// floating off it.
fn hurt(mut zombies: Query<(&Transform, &Shambler, &mut Dressed, &mut Health), Changed<Shambler>>, mut commands: Commands) {
    for (place, shambler, mut dressed, mut health) in &mut zombies {
        if shambler.health < dressed.health {
            arena::spawn_number(&mut commands, place.translation + Vec3::Y * (shambler.kind.size() - 1.0), f32::from(dressed.health - shambler.health));
        }
        if dressed.health != shambler.health {
            dressed.health = shambler.health;
        }
        let (left, full) = (f32::from(shambler.health), f32::from(shambler.most.max(1)));
        if health.left() != left || health.full() != full {
            *health = Health::at(left, full);
        }
    }
}

/// A zombie gone in the middle of the game, killed: the blow that killed it
/// floating off it, as every blow does that takes health and leaves it
/// standing ([`hurt`]). Its body goes in the frame of the blow, before its
/// health is seen to go, so it is said here as what it had left (what a
/// blow could take from it). Not one gone into the void, which no blow
/// had, nor any once the game is over, as everything of it goes; nor,
/// online, any that goes with the connection (`net` takes everything the
/// server sent away as it is lost, and says so first): until 2026-11-18
/// every zombie on the sand floated what it had left as the connection went.
fn last_blow(
    gone: On<Remove, Dressed>,
    zombies: Query<(&Transform, &Dressed)>,
    stand: Option<Res<Stand>>,
    online: Option<Res<Online>>,
    me: Query<&Venue, With<Player>>,
    mut commands: Commands,
) {
    let killed = stand.is_some_and(|stand| stand.playing() && (!stand.online || online.is_some_and(|online| online.is())));
    if !killed || me.single().ok() != Some(&Venue::LastStand) {
        return;
    }
    let Ok((place, dressed)) = zombies.get(gone.entity) else {
        return;
    };
    if place.translation.y < rules::SAND_TOP - 1.0 || dressed.health == 0 {
        return;
    }
    arena::spawn_number(&mut commands, place.translation + Vec3::Y * (dressed.kind.size() - 1.0), f32::from(dressed.health));
}

/// Paints each zombie in its kind's colours as its model comes: its skin
/// and its hair, the parts the camera over it sees most of, as Bridge
/// Wars' zombies are (`arena::paint_zombies`); each a copy of the model's
/// own material, made once for each kind and shared by all of it, and
/// coloured again as the file changes.
#[allow(clippy::type_complexity)]
fn paint(
    mut zombies: Query<(Entity, &mut Dressed)>,
    children: Query<&Children>,
    parts: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    changed: Res<UiChanged>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut paints: Local<HashMap<(Kind, bool), Handle<StandardMaterial>>>,
    mut commands: Commands,
) {
    let colours = ui().colours.last_stand;
    if changed.is_changed() && !changed.is_added() {
        for (&(kind, hair), paint) in paints.iter() {
            let (skin_colour, hair_colour) = colours.paint(kind);
            if let Some(mut look) = materials.get_mut(paint) {
                look.base_color = if hair { hair_colour } else { skin_colour };
            }
        }
    }
    for (zombie, mut dressed) in &mut zombies {
        if dressed.painted {
            continue;
        }
        let kind = dressed.kind;
        let mut painted = false;
        for part in children.iter_descendants(zombie) {
            let Ok((name, worn)) = parts.get(part) else {
                continue;
            };
            let hair = is_material(&name.0, "hair");
            if !hair && !is_material(&name.0, "skin") {
                continue;
            }
            let paint = paints
                .entry((kind, hair))
                .or_insert_with(|| {
                    let (skin_colour, hair_colour) = colours.paint(kind);
                    let mut look = materials.get(&worn.0).cloned().unwrap_or_default();
                    look.base_color = if hair { hair_colour } else { skin_colour };
                    materials.add(look)
                })
                .clone();
            if worn.0 != paint {
                commands.entity(part).insert(MeshMaterial3d(paint));
            }
            painted = true;
        }
        // Not until its model is there: it comes a frame or two after.
        if painted {
            dressed.painted = true;
        }
    }
}

/// Makes each zombie its kind's size as its model comes (a brute a third
/// bigger, a giant twice as big): the model under its body, the part its
/// limbs are under, scaled about its feet, as `arena::squash` lays a body
/// flat. The body's own place is the server's or the game's to write, and
/// is left alone.
fn size(
    mut zombies: Query<(&mut Dressed, &Children)>,
    children: Query<&Children>,
    bones: Query<(), With<WalkBone>>,
    mut models: Query<&mut Transform>,
) {
    for (mut dressed, parts) in &mut zombies {
        if dressed.sized {
            continue;
        }
        let grown = dressed.kind.size();
        if grown == 1.0 {
            dressed.sized = true;
            continue;
        }
        // Its model is the part of it its limbs are under, once they are.
        let Some(model) = parts
            .iter()
            .find(|&part| children.iter_descendants(part).any(|under| bones.contains(under)))
        else {
            continue;
        };
        if let Ok(mut place) = models.get_mut(model) {
            place.translation *= grown;
            place.scale *= grown;
            dressed.sized = true;
        }
    }
}

/// Swings each zombie's legs by how far it has gone, as `lobby` does a body
/// it does not move itself, but the slower the bigger it is: a giant's legs
/// are twice as long, and take a stride half as often for the ground it
/// covers, so that its footfalls fall heavy and its feet never slide. Even
/// slower than the slowest a walk swings otherwise (`crate::animate_walk`,
/// which swings a body at the least a third as often as a walk), what that
/// would add is taken back first.
fn stride(time: Res<Time>, mut zombies: Query<(&Transform, &mut Dressed, &mut WalkCycle)>) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (place, mut dressed, mut walk) in &mut zombies {
        let moved = place.translation.xz().distance(dressed.last.xz());
        if dressed.last != place.translation {
            dressed.last = place.translation;
        }
        // A step through the void, as it is put back or rises, is no stride.
        let speed = if moved > 3.0 { 0.0 } else { (moved / dt / crate::MOVE_SPEED / dressed.kind.size()).min(2.0) };
        if walk.speed != speed {
            walk.speed = speed;
        }
        if walk.weight >= 0.01 && speed > 0.04 {
            let swung = speed.max(SLOWEST_SWING);
            walk.phase -= dt * crate::WALK_STRIDE_FREQ * (swung - speed);
        }
    }
}

// ----------------------------------------------------------- the anchor

/// The anchor's crystal as the map has it: its parts, each with the flash
/// over it; the map's shape as it was before it shattered, to stand it up
/// again; whether it has; the game it stands in (`Stand::fight`), the
/// health it last had there, what it has taken since a number last said
/// it, and how long ago it last flashed.
#[derive(Resource, Default)]
pub(super) struct Crystal {
    parts: Vec<Entity>,
    flashes: Vec<Entity>,
    flash: Option<Handle<StandardMaterial>>,
    original: Option<Vec<[Vec3; 3]>>,
    broken: bool,
    fight: Option<u32>,
    health: Option<u16>,
    unsaid: f32,
    since_flash: f32,
    flashing: f32,
}

/// Finds the crystal in Last Stand's map as it spawns: every part in the
/// anchors' material (`arena::SPAWN_MATERIAL`, the map's `spawn`), and over
/// each a copy of itself a little bigger, in the flash's white, unseen
/// until it is hit.
#[allow(clippy::too_many_arguments)]
fn read_crystal(
    ready: On<WorldInstanceReady>,
    lands: Query<&Venue, With<Land>>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, &GltfMaterialName)>,
    meshes: Res<Assets<Mesh>>,
    mut crystal: ResMut<Crystal>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if lands.get(ready.entity).ok() != Some(&Venue::LastStand) {
        return;
    }
    let flash = materials.add(StandardMaterial {
        base_color: ui().colours.last_stand.anchor_flash.with_alpha(0.0),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        ..default()
    });
    let mut found = Vec::new();
    let mut flashes = Vec::new();
    for part in children.iter_descendants(ready.entity) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        if !is_material(&material.0, arena::SPAWN_MATERIAL) {
            continue;
        }
        // Over itself, scaled about its own middle.
        let middle = meshes
            .get(&mesh.0)
            .and_then(|mesh| match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                Some(VertexAttributeValues::Float32x3(corners)) if !corners.is_empty() => {
                    let (low, high) = corners
                        .iter()
                        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| (low.min(corner.into()), high.max(corner.into())));
                    Some((low + high) * 0.5)
                }
                _ => None,
            })
            .unwrap_or_default();
        let over = commands
            .spawn((
                ChildOf(part),
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(flash.clone()),
                Transform::from_translation(middle - middle * FLASH_OVER).with_scale(Vec3::splat(FLASH_OVER)),
                Visibility::Hidden,
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
            ))
            .id();
        found.push(part);
        flashes.push(over);
    }
    info!("Last Stand's crystal: {} parts", found.len());
    *crystal = Crystal {
        parts: found,
        flashes,
        flash: Some(flash),
        ..default()
    };
}

/// The crystal as the anchor stands: a flash as it is hit, and over it a
/// number of what the blows took (Bridge Wars floats one off its anchors),
/// together no oftener than [`FLASH_EVERY`], the number saying all it took
/// since the last; a number in a heal's green as a Fortify brings it all
/// back; and the moment it is broken, shattered: its parts hidden (the
/// light in it goes with them, `lamps`), its pieces flying out from it
/// ([`Shard`]), and it taken away from underfoot. Stood up again, whole, as
/// the game goes and as every game begins, whatever the one before did to
/// it (until 2026-11-18 only on a frame with no game at all, which a
/// table followed by the next, Play again or a group taken on, never has:
/// every game after the first had no crystal, nothing to stand on, no light
/// and nothing to flash or shatter, its bar over nothing).
#[allow(clippy::too_many_arguments)]
fn anchor(
    time: Res<Time>,
    stand: Option<Res<Stand>>,
    looks: Res<Looks>,
    mut crystal: ResMut<Crystal>,
    mut islands: ResMut<Islands>,
    mut visibilities: Query<&mut Visibility>,
    worn: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let crystal = &mut *crystal;
    let fight = stand.as_ref().map(|stand| stand.fight);
    if crystal.fight != fight {
        crystal.fight = fight;
        crystal.health = None;
        crystal.unsaid = 0.0;
        crystal.flashing = 0.0;
        whole_again(crystal, &mut visibilities, &mut islands);
    }
    let Some(stand) = stand else {
        return;
    };
    let dt = time.delta_secs();
    crystal.since_flash += dt;
    let Some(now) = stand.round.as_ref().map(|round| round.anchor) else {
        return;
    };
    let was = crystal.health.replace(now);
    let foot = Vec3::new(ANCHOR.x, PLINTH_TOP, ANCHOR.y);
    match was {
        Some(was) if now < was => crystal.unsaid += f32::from(was - now),
        Some(was) if now > was => arena::spawn_healed(&mut commands, foot, f32::from(now - was)),
        _ => {}
    }
    // The blow that breaks it said at once, as it shatters.
    if crystal.unsaid > 0.0 && (crystal.since_flash >= FLASH_EVERY || now == 0) {
        arena::spawn_number(&mut commands, foot, crystal.unsaid);
        crystal.unsaid = 0.0;
        crystal.since_flash = 0.0;
        if now > 0 {
            crystal.flashing = FLASH_SECS;
        }
    }
    // The flash, fading as it goes.
    let strength = if crystal.flashing > 0.0 {
        crystal.flashing = (crystal.flashing - dt).max(0.0);
        crystal.flashing / FLASH_SECS
    } else {
        0.0
    };
    if let Some(flash) = &crystal.flash {
        let white = ui().colours.last_stand.anchor_flash;
        let wanted = white.with_alpha(white.alpha() * strength);
        if materials.get(flash).is_some_and(|look| look.base_color != wanted)
            && let Some(mut look) = materials.get_mut(flash)
        {
            look.base_color = wanted;
        }
        let shown = if strength > 0.0 && !crystal.broken { Visibility::Inherited } else { Visibility::Hidden };
        for &over in &crystal.flashes {
            if let Ok(mut visibility) = visibilities.get_mut(over) {
                visibility.set_if_neq(shown);
            }
        }
    }
    if now > 0 || crystal.broken {
        return;
    }
    // Broken: it shatters, unless it was already when this device came
    // into the game, which sees it gone.
    crystal.broken = true;
    let fresh = was.is_some();
    let material = crystal.parts.first().and_then(|&part| worn.get(part).ok()).map(|worn| worn.0.clone());
    for &part in &crystal.parts {
        if let Ok(mut shown) = visibilities.get_mut(part) {
            shown.set_if_neq(Visibility::Hidden);
        }
    }
    if fresh && let Some(material) = material {
        let middle = Vec3::new(ANCHOR.x, CRYSTAL_MIDDLE, ANCHOR.y);
        for piece in 0..SHARDS {
            let turn = piece as f32 / SHARDS as f32 * std::f32::consts::TAU + fastrand::f32() * 0.4;
            let out = SHARD_OUT.0 + (SHARD_OUT.1 - SHARD_OUT.0) * fastrand::f32();
            let up = SHARD_UP.0 + (SHARD_UP.1 - SHARD_UP.0) * fastrand::f32();
            let way = Vec2::from_angle(turn);
            let from = middle + Vec3::new(way.x, (fastrand::f32() - 0.5) * 2.4, way.y) * 0.45;
            commands.spawn((
                Shard {
                    velocity: Vec3::new(way.x * out, up, way.y * out),
                    spin: Vec3::new(fastrand::f32() - 0.5, fastrand::f32() - 0.5, fastrand::f32() - 0.5) * 14.0,
                    size: 0.2 + 0.25 * fastrand::f32(),
                    age: 0.0,
                },
                InArena,
                Venue::LastStand,
                Mesh3d(looks.shard.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(from).with_scale(Vec3::splat(0.3)),
                Visibility::Hidden,
                bevy::light::NotShadowCaster,
            ));
        }
    }
    // Nothing of it underfoot.
    let Some(corners) = islands.corners(Venue::LastStand).map(<[_]>::to_vec) else {
        return;
    };
    let left: Vec<[Vec3; 3]> = corners.iter().copied().filter(|triangle| !of_the_crystal(triangle)).collect();
    crystal.original = Some(corners);
    islands.reshape(Venue::LastStand, left, &[]);
}

/// The crystal stood up again whole, if it has shattered: its parts shown
/// (and the light in them, `lamps`), and back underfoot as the map has it.
fn whole_again(crystal: &mut Crystal, visibilities: &mut Query<&mut Visibility>, islands: &mut Islands) {
    if !std::mem::take(&mut crystal.broken) {
        return;
    }
    for &part in &crystal.parts {
        if let Ok(mut shown) = visibilities.get_mut(part) {
            shown.set_if_neq(Visibility::Inherited);
        }
    }
    if let Some(original) = crystal.original.take() {
        islands.reshape(Venue::LastStand, original, &[]);
    }
}

/// Whether `triangle` of the map is of the crystal: all of it near the
/// anchor across the ground, and some of it well over the plinth's top,
/// which nothing else near the anchor reaches.
fn of_the_crystal(triangle: &[Vec3; 3]) -> bool {
    triangle.iter().all(|corner| corner.xz().distance(ANCHOR) <= CRYSTAL_REACH)
        && triangle.iter().any(|corner| corner.y > PLINTH_TOP + CRYSTAL_OVER)
}

/// One of the crystal's pieces, flying out as it shatters: how fast it goes
/// and spins, how big it is, and how long it has flown.
#[derive(Component)]
struct Shard {
    velocity: Vec3,
    spin: Vec3,
    size: f32,
    age: f32,
}

/// Flies every piece of the crystal out and down, shrinking at the end,
/// and takes it away once it has flown its while.
fn shards(time: Res<Time>, mut shards: Query<(Entity, &mut Shard, &mut Transform)>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, mut shard, mut place) in &mut shards {
        shard.age += dt;
        if shard.age >= SHARD_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        shard.velocity.y -= SHARD_FALL * dt;
        let velocity = shard.velocity;
        place.translation += velocity * dt;
        let spin = shard.spin * dt;
        place.rotation = Quat::from_euler(EulerRot::XYZ, spin.x, spin.y, spin.z) * place.rotation;
        let left = ((SHARD_SECS - shard.age) / 0.4).min(1.0);
        place.scale = Vec3::new(shard.size, shard.size * 1.8, shard.size) * left;
    }
}

/// The bar over the crystal, wider than a body's (Hajun's brief: "a health
/// bar over it, wider than a body's"): put up with the game, filled as far
/// as the anchor's health goes of the most it has.
#[derive(Component)]
struct AnchorBar;
#[derive(Component)]
struct AnchorFill;

/// Keeps the bar over the crystal, filled as far as the anchor goes, put
/// away while it is off the screen, under the plate or among the rows top
/// left ([`Column`]), broken, or the game is not being played, or a shop's
/// window is up ([`in_sight`]); put up as the game begins and taken down
/// with it.
#[allow(clippy::type_complexity)]
fn anchor_bar(
    stand: Option<Res<Stand>>,
    me: Query<(&Venue, &Transform), With<Player>>,
    shops: Res<Shops>,
    column: Column,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut bars: Query<(Entity, &mut Visibility, &mut UiTransform, &ComputedNode), With<AnchorBar>>,
    mut fills: Query<&mut Node, With<AnchorFill>>,
    mut commands: Commands,
) {
    let Some(stand) = stand else {
        return;
    };
    let Some((bar, mut shown, mut place, node)) = bars.iter_mut().next() else {
        spawn_anchor_bar(&mut commands);
        return;
    };
    let _ = bar;
    let round = stand.round.as_ref().filter(|round| round.anchor > 0);
    let here = in_sight(&me, &shops);
    let point = round
        .filter(|_| here)
        .zip(cameras.single().ok())
        .zip(windows.single().ok())
        .and_then(|((_, (camera, eye)), window)| {
            let eye = GlobalTransform::from(*eye);
            // The crystal's outline on the screen: its foot and its tip,
            // each as wide as it is.
            let (mut top, mut left, mut right) = (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY);
            for height in [PLINTH_TOP, CRYSTAL_TIP] {
                for (x, z) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                    let at = Vec3::new(ANCHOR.x + x * CRYSTAL_RADIUS, height, ANCHOR.y + z * CRYSTAL_RADIUS);
                    let point = camera.world_to_viewport(&eye, at).ok()?;
                    top = top.min(point.y);
                    left = left.min(point.x);
                    right = right.max(point.x);
                }
            }
            let gap = window.width().min(window.height()) * ANCHOR_BAR_GAP / 100.0;
            // The bar's foot, over its middle; put away where the plate top
            // centre is, which has the anchor's bar already (just under it,
            // the two read as one bar twice).
            Some(Vec2::new((left + right) * 0.5, top - gap))
                .filter(|point| arena::on_screen(window, *point) && !under_the_plate(window, *point))
        });
    let size = node.size() * node.inverse_scale_factor();
    let (Some(point), Some(round)) = (point.filter(|point| !column.covers(standing_over(*point, size))), round) else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    place.set_if_neq(UiTransform {
        translation: Val2::px(point.x - size.x * 0.5, point.y - size.y),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    shown.set_if_neq(Visibility::Inherited);
    let width = arena::bar_width(f32::from(round.anchor) / f32::from(round.anchor_most.max(1)));
    for mut fill in &mut fills {
        if fill.width != width {
            fill.width = width;
        }
    }
}

fn spawn_anchor_bar(commands: &mut Commands) {
    let colours = ui().colours;
    commands.spawn((
        InArena,
        AnchorBar,
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            width: Val::VMin(ANCHOR_BAR_WIDTH),
            height: Val::VMin(ANCHOR_BAR_HEIGHT),
            border: UiRect::all(Val::VMin(BAR_RIM)),
            border_radius: BorderRadius::all(Val::VMin(0.5)),
            ..default()
        },
        BackgroundColor(colours.arena.hp_back),
        BorderColor::all(colours.arena.hp_rim),
        UiTransform::default(),
        Visibility::Hidden,
        GlobalZIndex(-1),
        Pickable::IGNORE,
        children![(
            AnchorFill,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::VMin(0.3)),
                ..default()
            },
            BackgroundColor(colours.last_stand.anchor),
            Pickable::IGNORE,
        )],
    ));
}

/// The anchor's beam as it zaps (Hajun's brief: "a short bright line from
/// the crystal to the zombie it zaps"): a thin bright bar from the
/// crystal's middle to the zombie's, with a spark where it lands, both
/// bright a moment and then fading.
#[derive(Component)]
struct Beam {
    age: f32,
}

fn beam(
    time: Res<Time>,
    mut happened: MessageReader<Happened>,
    looks: Res<Looks>,
    mut beams: Query<(Entity, &mut Beam, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let colour = ui().colours.last_stand.beam;
    for happening in happened.read() {
        let Happened::Zap(at) = *happening else {
            continue;
        };
        let from = Vec3::new(ANCHOR.x, BEAM_FROM, ANCHOR.y);
        let to = at + Vec3::Y * BEAM_AT;
        let along = to - from;
        let length = along.length();
        if length < 0.1 {
            continue;
        }
        let look = materials.add(StandardMaterial {
            base_color: bright(colour, 3.0),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            Beam { age: 0.0 },
            InArena,
            Venue::LastStand,
            Mesh3d(looks.ball.clone()),
            MeshMaterial3d(look.clone()),
            Transform::from_translation(to).with_scale(Vec3::splat(BEAM_SPARK)),
            // Seen from the frame it is made, as it is made only here.
            Visibility::Inherited,
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
        commands.spawn((
            Beam { age: 0.0 },
            InArena,
            Venue::LastStand,
            Mesh3d(looks.beam.clone()),
            MeshMaterial3d(look),
            Transform::from_translation(from + along * 0.5)
                .looking_to(along, Vec3::Y)
                .with_scale(Vec3::new(BEAM_WIDTH, BEAM_WIDTH, length)),
            Visibility::Inherited,
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
    }
    let dt = time.delta_secs();
    for (entity, mut beam, look) in &mut beams {
        beam.age += dt;
        if beam.age >= BEAM_SECS {
            materials.remove(&look.0);
            commands.entity(entity).despawn();
            continue;
        }
        // (The line and its spark share their look: whichever comes here
        // first in a frame sets it for both.)
        let left = (1.0 - (beam.age - BEAM_HOLD).max(0.0) / (BEAM_SECS - BEAM_HOLD)).clamp(0.0, 1.0);
        let wanted = bright(colour, 3.0).with_alpha(left);
        if materials.get(&look.0).is_some_and(|material| material.base_color != wanted)
            && let Some(mut material) = materials.get_mut(&look.0)
        {
            material.base_color = wanted;
        }
    }
}

// --------------------------------------------------------------- the spit

/// A spitter's ball, as it is drawn: on its way as the rules fly it, and
/// splashing where it comes down; online the server's own thing for it,
/// which it takes away as it lands.
#[derive(Component)]
pub(super) struct SpitBall {
    spittle: Spittle,
    source: Option<Entity>,
}

impl SpitBall {
    /// A ball spat as `spittle` has it, online for the server's `source`:
    /// what looks it needs is put on it a frame later ([`fly_spits`]).
    pub(super) fn of(spittle: Spittle, source: Option<Entity>) -> impl Bundle {
        (
            SpitBall { spittle, source },
            InArena,
            Venue::LastStand,
            Name::new("Spit"),
            Transform::from_translation(spittle.at()).with_scale(Vec3::splat(SPIT_SIZE)),
            Visibility::Hidden,
        )
    }
}

/// Online, a ball for every one the server sends, as it comes.
fn follow_spits(fresh: Query<(Entity, &Spit), (With<Remote>, Added<Spit>)>, mut commands: Commands) {
    for (source, spit) in &fresh {
        commands.spawn(SpitBall::of(spit.spittle(), Some(source)));
    }
}

/// Flies every ball on, and splashes it where it comes down: as the rules
/// have it land (the server's is taken away there too); one of the
/// server's that it has taken away a while after it should have landed is
/// gone with no splash.
fn fly_spits(
    time: Res<Time>,
    looks: Res<Looks>,
    sources: Query<(), With<Spit>>,
    mut balls: Query<(Entity, &mut SpitBall, &mut Transform, Has<Mesh3d>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut ball, mut place, dressed) in &mut balls {
        if !dressed {
            commands.entity(entity).insert((
                Mesh3d(looks.ball.clone()),
                MeshMaterial3d(looks.spit.clone()),
                bevy::light::NotShadowCaster,
            ));
        }
        ball.spittle.tick(dt);
        place.translation = ball.spittle.at();
        let gone = ball.source.is_some_and(|source| !sources.contains(source));
        if ball.spittle.landed() {
            splat(&mut commands, &looks, &mut materials, ball.spittle.to);
            commands.entity(entity).despawn();
        } else if gone && ball.spittle.age > ball.spittle.secs() + SPIT_SLACK {
            commands.entity(entity).despawn();
        }
    }
}

/// A splash of green where a ball came down.
#[derive(Component)]
struct Splat {
    age: f32,
}

fn splat(commands: &mut Commands, looks: &Looks, materials: &mut Assets<StandardMaterial>, at: Vec3) {
    let colour = ui().colours.last_stand.splat;
    commands.spawn((
        Splat { age: 0.0 },
        InArena,
        Venue::LastStand,
        Mesh3d(looks.disc.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: colour,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(at + Vec3::Y * 0.04)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(SPIT_RADIUS * SPLAT_FROM)),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
    ));
}

/// Spreads every splash out to where the ball splashed whoever was there,
/// fading as it lies, and takes it away once it has.
fn splats(
    time: Res<Time>,
    mut splats: Query<(Entity, &mut Splat, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let colour = ui().colours.last_stand.splat;
    let dt = time.delta_secs();
    for (entity, mut splat, mut place, look) in &mut splats {
        splat.age += dt;
        if splat.age >= SPLAT_SECS {
            materials.remove(&look.0);
            commands.entity(entity).despawn();
            continue;
        }
        let done = splat.age / SPLAT_SECS;
        let out = 1.0 - (1.0 - (done * 4.0).min(1.0)).powi(2);
        place.scale = Vec3::splat(SPIT_RADIUS * (SPLAT_FROM + (1.0 - SPLAT_FROM) * out));
        if let Some(mut material) = materials.get_mut(&look.0) {
            material.base_color = colour.with_alpha(colour.alpha() * (1.0 - done * done));
        }
    }
}

// ------------------------------------------------------- over the pads

/// What is written over one of the generators' pads, by its number.
#[derive(Component)]
struct PadWords(usize);

/// What the generators make, at `level` (the team's Generator line): silver
/// alone; then what each step adds, gold, sapphire and amethyst; and past
/// them, all four, so many times as fast (Hajun's brief: "Lvl 2: gold,
/// sapphire").
fn pad_words(level: u16) -> String {
    let file = ui();
    let words = &file.words.last_stand;
    let level_word = level.to_string();
    match level {
        0 => words.generator.clone(),
        1..=3 => {
            let items: Vec<String> = [Item::Gold, Item::Sapphire, Item::Amethyst]
                .iter()
                .take(usize::from(level))
                .map(|item| file.words.arena.item(*item).to_lowercase())
                .collect();
            fill(&words.generator_level, &[("level", &level_word), ("items", &items.join(", "))])
        }
        _ => fill(
            &words.generator_fast,
            &[("level", &level_word), ("fast", &format!("{:.2}", Forges::speed(level)).trim_end_matches('0').trim_end_matches('.'))],
        ),
    }
}

/// Writes over each pad what its generator makes, as the team raises it,
/// where the pad is on the screen; put away while it is off the screen,
/// under the plate or among the rows top left ([`Column`]), the game is not
/// being played, or a shop's window is up ([`in_sight`]).
#[allow(clippy::type_complexity)]
fn over_the_pads(
    stand: Option<Res<Stand>>,
    me: Query<(&Venue, &Transform), With<Player>>,
    shops: Res<Shops>,
    column: Column,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut words: Query<(&PadWords, &mut Visibility, &mut UiTransform, &ComputedNode, &mut Text)>,
    mut commands: Commands,
) {
    let Some(stand) = stand else {
        return;
    };
    if words.is_empty() {
        for pad in 0..PADS.len() {
            commands.spawn((
                InArena,
                PadWords(pad),
                crate::hud::words("", PAD_WORDS, ui().colours.text),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
                GlobalZIndex(-1),
            ));
        }
        return;
    }
    let here = in_sight(&me, &shops) && stand.playing();
    let said = pad_words(stand.upgrades.generator);
    let view = cameras.single().ok().zip(windows.single().ok());
    for (pad, mut shown, mut place, node, mut text) in &mut words {
        let point = view.filter(|_| here).and_then(|((camera, eye), window)| {
            let at = PADS[pad.0];
            camera
                .world_to_viewport(&GlobalTransform::from(*eye), Vec3::new(at.x, PAD_WORDS_OVER, at.y))
                .ok()
                .filter(|point| arena::on_screen(window, *point) && !under_the_plate(window, *point))
        });
        let size = node.size() * node.inverse_scale_factor();
        let Some(point) = point.filter(|point| !column.covers(standing_over(*point, size))) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        if text.0 != said {
            text.0.clone_from(&said);
        }
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, point.y - size.y),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        shown.set_if_neq(Visibility::Inherited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn over_the_pads_is_what_the_generators_make() {
        assert_eq!(pad_words(0), "Silver");
        assert_eq!(pad_words(1), "Lvl 1: gold");
        assert_eq!(pad_words(2), "Lvl 2: gold, sapphire");
        assert_eq!(pad_words(3), "Lvl 3: gold, sapphire, amethyst");
        assert_eq!(pad_words(4), "Lvl 4: all four, x1.25");
        assert_eq!(pad_words(5), "Lvl 5: all four, x1.56");
    }

    #[test]
    fn the_crystal_is_what_stands_over_the_plinth_at_the_anchor_and_nothing_else() {
        let up = PLINTH_TOP + 1.5;
        let at = |x: f32, y: f32, z: f32| Vec3::new(ANCHOR.x + x, y, ANCHOR.y + z);
        assert!(of_the_crystal(&[at(0.0, PLINTH_TOP, 0.0), at(0.5, up, 0.0), at(0.0, up, 0.5)]));
        // The plinth's own top, and the stone round it, are not.
        assert!(!of_the_crystal(&[at(-1.0, PLINTH_TOP, -1.0), at(1.0, PLINTH_TOP, -1.0), at(1.0, PLINTH_TOP, 1.0)]));
        assert!(!of_the_crystal(&[at(3.0, up, 0.0), at(3.5, up, 0.0), at(3.0, up, 0.5)]));
    }

    /// A game of Last Stand numbered `fight` whose anchor has `anchor` left.
    fn stand(fight: u32, anchor: u16) -> Stand {
        let mut stand = Stand::new(false, 0, fight);
        stand.round = Some(rules::StandRound {
            wave: 4,
            stage: rules::StandStage::Wave,
            anchor,
            anchor_most: 1500,
            left: 9,
            health: vec![if anchor > 0 { 100 } else { 0 }],
            back_at: vec![if anchor > 0 { 0.0 } else { f64::INFINITY }],
            scores: vec![0],
            ready: 0,
            gone: 0,
            began: 0.0,
        });
        stand
    }

    #[test]
    fn the_crystal_stands_whole_again_in_the_game_after_one_that_broke_it() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(Looks {
                ball: Handle::default(),
                spit: Handle::default(),
                beam: Handle::default(),
                shard: Handle::default(),
                disc: Handle::default(),
            })
            .add_systems(Update, anchor);
        // The map: the stone, and the crystal standing over the plinth.
        let at = |x: f32, y: f32, z: f32| Vec3::new(ANCHOR.x + x, y, ANCHOR.y + z);
        let stone = [at(-9.0, PLINTH_TOP, -9.0), at(9.0, PLINTH_TOP, -9.0), at(9.0, PLINTH_TOP, 9.0)];
        let crystal = [at(0.0, PLINTH_TOP, 0.0), at(0.4, PLINTH_TOP + 2.0, 0.0), at(0.0, PLINTH_TOP + 2.0, 0.4)];
        let mut islands = Islands::default();
        islands.reshape(Venue::LastStand, vec![stone, crystal], &[]);
        app.insert_resource(islands);
        let part = app.world_mut().spawn(Visibility::Inherited).id();
        app.insert_resource(Crystal {
            parts: vec![part],
            ..default()
        });
        let underfoot = |app: &App| app.world().resource::<Islands>().corners(Venue::LastStand).map_or(0, <[_]>::len);
        let shown = |app: &App| app.world().get::<Visibility>(part) == Some(&Visibility::Inherited);
        // A game whose anchor stands, and then breaks: it shatters, gone
        // from sight and from underfoot.
        app.insert_resource(stand(1, 1500));
        app.update();
        assert!(shown(&app) && underfoot(&app) == 2);
        app.insert_resource(stand(1, 0));
        app.update();
        assert!(!shown(&app) && underfoot(&app) == 1);
        // Play again: the next game's in its place, with no frame between
        // with none; it stands there whole, and breaks again in its turn.
        app.insert_resource(stand(2, 1500));
        app.update();
        assert!(shown(&app) && underfoot(&app) == 2, "the next game's crystal whole");
        app.insert_resource(stand(2, 0));
        app.update();
        assert!(!shown(&app) && underfoot(&app) == 1);
        // And with no game at all, as before.
        app.world_mut().remove_resource::<Stand>();
        app.update();
        assert!(shown(&app) && underfoot(&app) == 2);
    }

    #[test]
    fn the_file_changed_draws_the_spit_the_crystal_s_bar_and_the_pads_words_again() {
        let mut app = App::new();
        app.init_resource::<Assets<StandardMaterial>>().init_resource::<UiChanged>().add_systems(Update, recolour);
        let spit = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color: Color::BLACK,
            ..default()
        });
        app.insert_resource(Looks {
            ball: Handle::default(),
            spit: spit.clone(),
            beam: Handle::default(),
            shard: Handle::default(),
            disc: Handle::default(),
        });
        let bar = app.world_mut().spawn(AnchorBar).id();
        let words = app.world_mut().spawn(PadWords(0)).id();
        // As the app opens, nothing is drawn again.
        app.update();
        assert!(app.world().get_entity(bar).is_ok() && app.world().get_entity(words).is_ok());
        // The file read again: the spit in its colour, the bar and the
        // words taken down, to be put up again in theirs.
        app.world_mut().resource_mut::<UiChanged>().set_changed();
        app.update();
        let colour = app.world().resource::<Assets<StandardMaterial>>().get(&spit).expect("the spit").base_color;
        assert_eq!(colour, bright(ui().colours.last_stand.spit, SPIT_GAIN));
        assert!(app.world().get_entity(bar).is_err() && app.world().get_entity(words).is_err());
    }

    #[test]
    fn a_giant_s_legs_swing_half_as_often_as_a_walker_s_for_the_ground_it_covers() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, stride);
        let body = |kind: Kind| {
            (
                Transform::default(),
                Dressed {
                    kind,
                    health: 1,
                    last: Vec3::ZERO,
                    painted: true,
                    sized: true,
                },
                WalkCycle {
                    phase: 0.0,
                    weight: 1.0,
                    speed: 0.0,
                },
            )
        };
        let walker = app.world_mut().spawn(body(Kind::Walker)).id();
        let giant = app.world_mut().spawn(body(Kind::Giant)).id();
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(0.1));
        for entity in [walker, giant] {
            app.world_mut().get_mut::<Transform>(entity).expect("a body").translation.x = 0.7;
        }
        app.update();
        let speed = |entity| app.world().get::<WalkCycle>(entity).expect("a walk").speed;
        assert!((speed(walker) - 1.0).abs() < 1e-4, "{}", speed(walker));
        assert!((speed(giant) - 0.5).abs() < 1e-4, "{}", speed(giant));
    }
}
