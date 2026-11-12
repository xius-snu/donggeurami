//! The opening, as the app starts (since 2026-11-12; Hajun: "right now,
//! when i open the game, there is no smooth introduction animation. i need
//! a short 0.5~1s animation in the beginning. also when i open the game,
//! some part of the map is not loaded yet so it looks bad. only show the
//! map after it gets loaded. im thinking maybe it can be falling from an
//! extreme height, and when it lands, maybe a short ... landing animation,
//! and then get up/standup and then u can play"; of the ways I put to
//! them, they chose the circle opening with the drop and the squat
//! landing).
//!
//! **The cover.** From the first frame the screen is one plain colour with
//! the town's name on it (`colours.intro` and `words.welcome` in
//! `assets/ui.ron`), over everything but the sign-up's pages, and the
//! town loads under it. It takes every press, and you stand still under
//! it, as under a dialog (`hud::DialogUp`). It stays until the town is
//! there to be seen ([`Loading`]): the town's own shape read, every model
//! in the world spawned, the buses' way round worked out, your body on its
//! feet; and until the GPU has nothing left to compile for what is in
//! sight, for a few frames running ([`Compiling`]: Bevy compiles each
//! material's pipeline in the background and leaves out whatever is not
//! ready, which is how a town came up a piece at a time). Should any of
//! that never come, it opens anyway after [`MOST_COVERED`] seconds. The
//! first time the app is run, the sign-up's pages stand over it, and it
//! opens as they go.
//!
//! **The opening**, [`UP_AT`] seconds of it ([`pose`], [`hole`]): the
//! cover opens as a growing circle about where you stand, 동그라미, the
//! way a blinded player's sight is cut out of the dark in Bridge Wars
//! (`gear::blindfold`: one node, all border, with a round hole); you drop
//! in from over the top of the screen with your arms up, land in a squat,
//! with a ring of dust going out over the ground from your feet and the
//! camera dipping a little, and spring up; and from then on you can move.
//! The town's name stays where it was on the cover, over the town now,
//! for a moment more, and fades: it is the welcome (`hud::welcome` leaves
//! this one to it).
//!
//! **Nothing of the fall is your body's.** Your body stands where it
//! arrived all along; what drops is its model, hung that much higher over
//! it ([`Model`]), so that nothing of it is a move to the island, to
//! anyone else, or to the server. The squat is your limbs' ([`Landing`],
//! `animate_walk`), and the body drawn a little taller in the air and
//! shorter as it lands.
//!
//! `RT_INTRO_SLOW=<times>` on desktop plays the opening that many times
//! slower, for looking at it; `RT_INTRO=off` leaves all of it out, cover
//! and all, for a scripted run that presses or looks from its first
//! frame.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::render_resource::PipelineCache;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::window::PrimaryWindow;
use bevy::world_serialization::{WorldInstance, WorldInstanceSpawner};

use crate::hud::{self, DialogUp, TakesPress};
use crate::island::{Islands, Venue};
use crate::signup::Signup;
use crate::ui::ui;
use crate::{Landing, Player, ThirdPersonCamera, WalkBone, WalkBoneKind, car, range, sky};

/// Over everything on the screen, dialogs and the black screen a game
/// loads under among them, and under the sign-up's pages (`signup`'s
/// layer, 30): the cover, the circle that opens it, and the town's name.
const LAYER: i32 = 25;

/// How long the town's name takes to come up on the cover as the app
/// starts, in seconds; and, once the cover has begun to open, how long
/// the name stays and how long it takes to go.
const TITLE_IN: f32 = 0.25;
const TITLE_STAYS: f32 = 2.2;
const TITLE_OUT: f32 = 0.8;

/// How many frames running the GPU has to have had nothing to compile,
/// with everything there, before the cover opens; how long a frame may
/// have taken for the opening to begin on it, in seconds (not on the heels
/// of a frame that hung); and how long the cover stays at the most, the
/// sign-up's pages aside, whatever has not come.
const QUIET_FRAMES: u32 = 8;
const HITCH: f32 = 0.05;
const MOST_COVERED: f32 = 10.0;

/// The most of the opening one frame plays, in seconds: a frame that hung
/// is not that much of it skipped.
const MOST_STEP: f32 = 1.0 / 30.0;

/// The opening, in seconds from when the cover begins to open: when the
/// circle has reached every corner of the screen; when you land; how long
/// you take to sink into the squat, stay down, and come back up.
const OPEN_SECS: f32 = 0.55;
const FALL_SECS: f32 = 0.4;
const SQUAT_SECS: f32 = 0.07;
const DOWN_SECS: f32 = 0.06;
const RISE_SECS: f32 = 0.32;
/// When you are up, and can move.
const UP_AT: f32 = FALL_SECS + SQUAT_SECS + DOWN_SECS + RISE_SECS;

/// How high over where you stand you fall from, in metres: over the top
/// of the screen, from the camera as the app opens it. And how the fall
/// gathers speed: 1 would be one speed all the way down.
const FALL_FROM: f32 = 11.0;
const FALL_GATHERS: f32 = 1.3;

/// In the air: how far your knees are bent, of the squat's whole bend,
/// and how much taller than you are you are drawn. Landed: how much
/// shorter, at the deepest; how far up your arms still are, out at your
/// sides; how far you lean forward, in radians; and how much taller than
/// you are you spring up through.
const AIR_SQUAT: f32 = 0.3;
const AIR_TALL: f32 = 1.07;
const SQUASHED: f32 = 0.86;
const ARMS_OUT: f32 = 0.42;
const LEAN: f32 = 0.2;
const SPRING: f32 = 0.045;

/// The ring of dust: how long it lasts, in seconds; how far out from your
/// middle it begins and ends, in metres; how much of the way out its
/// inner edge is; and how high over the ground it lies.
const DUST_SECS: f32 = 0.45;
const DUST_FROM: f32 = 0.35;
const DUST_TO: f32 = 2.0;
const DUST_INNER: f32 = 0.84;
const DUST_OVER: f32 = 0.04;

/// How far the camera dips as you land, in metres, and for how long.
const DIP: f32 = 0.12;
const DIP_SECS: f32 = 0.22;

/// How high over your feet the middle of the circle is: your chest, which
/// the camera looks at.
const CENTRE: f32 = crate::LOOK_HEIGHT;

/// The opening, from the first frame: covered, opening, and over.
#[derive(Resource)]
pub(crate) struct Intro {
    stage: Stage,
    wait: Wait,
    /// Your model, once the cover has begun to open, for the drop.
    model: Option<Model>,
    /// How many times slower than it is the opening is played
    /// (`RT_INTRO_SLOW`).
    slow: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    /// The cover is up, and the town loading under it.
    Covered,
    /// The cover is opening, or has opened: for how long, and whether you
    /// have been let go yet.
    Opening { age: f32, free: bool },
    /// Nothing of it is left.
    Over,
}

impl Default for Intro {
    fn default() -> Self {
        let desktop = cfg!(not(any(target_os = "android", target_os = "ios")));
        let slow = if desktop {
            std::env::var("RT_INTRO_SLOW").ok().and_then(|times| times.trim().parse::<f32>().ok())
        } else {
            None
        };
        let skipped = desktop && std::env::var("RT_INTRO").is_ok_and(|say| say.trim().eq_ignore_ascii_case("off"));
        Self {
            stage: if skipped { Stage::Over } else { Stage::Covered },
            wait: Wait::default(),
            model: None,
            slow: slow.filter(|times| times.is_finite()).map_or(1.0, |times| times.max(1.0)),
        }
    }
}

impl Intro {
    /// Whether the town's name is the opening's to show: until all of it
    /// is over (`hud::welcome`).
    pub(crate) fn welcomes(&self) -> bool {
        self.stage != Stage::Over
    }

    fn over(&self) -> bool {
        self.stage == Stage::Over
    }
}

/// Your model, the part of your body that is drawn: the node your limbs
/// hang under, as it rests on your body, and how high your hips are over
/// your feet, in metres.
#[derive(Clone, Copy)]
struct Model {
    root: Entity,
    rest: Transform,
    hip: f32,
}

/// What the cover waits for, and for how long it has.
#[derive(Default, Debug)]
struct Wait {
    /// Seconds the cover has been up with no sign-up page over it.
    secs: f32,
    /// Frames drawn running with everything there and nothing compiling.
    quiet: u32,
    /// The last frame drawn that was counted.
    frame: u32,
    /// When each thing waited for was first there ([`There`]), for the log.
    since: [Option<f32>; 4],
}

impl Wait {
    /// Counts a frame of `dt` seconds: whether everything is `there`, how
    /// many pipelines the GPU still had to `compile` as of the frame drawn
    /// last, number `frame`. Whether the cover is to open now.
    fn step(&mut self, there: There, compiling: usize, frame: u32, dt: f32) -> bool {
        self.secs += dt;
        for (since, there) in self.since.iter_mut().zip(there.0) {
            if there && since.is_none() {
                *since = Some(self.secs);
            }
        }
        if there.all() && compiling == 0 {
            if frame != self.frame {
                self.quiet += 1;
            }
        } else {
            self.quiet = 0;
        }
        self.frame = frame;
        (self.quiet >= QUIET_FRAMES && dt <= HITCH) || self.secs >= MOST_COVERED
    }
}

/// Whether each thing the cover waits for is there: the town's shape,
/// every model in the world, the buses' way round, and your body's limbs.
#[derive(Clone, Copy, PartialEq, Debug)]
struct There([bool; 4]);

impl There {
    fn all(self) -> bool {
        self.0.iter().all(|&there| there)
    }
}

/// What the town is made of, as it loads.
#[derive(SystemParam)]
struct Loading<'w, 's> {
    islands: Res<'w, Islands>,
    spawner: Res<'w, WorldInstanceSpawner>,
    models: Query<'w, 's, Option<&'static WorldInstance>, With<WorldAssetRoot>>,
    cars: car::Planned<'w>,
    me: Query<'w, 's, Entity, With<Player>>,
    limbs: Query<'w, 's, &'static WalkBone>,
}

impl Loading<'_, '_> {
    fn there(&self) -> There {
        let town = self.islands.get(Venue::Town).is_some();
        // Every model anything is drawn with: the islands, the buses,
        // everyone's bodies. One whose file has not been read yet has no
        // instance, or one not spawned.
        let models = self
            .models
            .iter()
            .all(|instance| instance.is_some_and(|instance| self.spawner.instance_is_ready(**instance)));
        let cars = town && !self.cars.pending();
        let body = self
            .me
            .single()
            .is_ok_and(|me| self.limbs.iter().filter(|limb| limb.owner == me).count() >= 6);
        There([town, models, cars, body])
    }
}

/// How much the GPU still has to compile, as of the frame it drew last:
/// kept by the render world ([`count_compiling`]) for this one to read.
#[derive(Resource, Clone, Default)]
struct Compiling(Arc<Counts>);

#[derive(Default)]
struct Counts {
    /// Pipelines asked for and not ready yet.
    waiting: AtomicUsize,
    /// Frames drawn.
    frames: AtomicU32,
}

/// The cover: the whole screen, a plain colour, taking every press there
/// is.
#[derive(Component)]
struct Cover;

/// The circle the cover opens as: one node, all border, with a round
/// hole in it.
#[derive(Component)]
struct Ring;

/// The town's name, on the cover and then over the town.
#[derive(Component)]
struct Title;

/// The ring of dust that goes out from your feet as you land.
#[derive(Component)]
struct Dust;

pub(crate) fn plugin(app: &mut App) {
    let compiling = Compiling::default();
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render
            .insert_resource(compiling.clone())
            // After the frame's pipelines have been looked at and it has
            // been drawn (`RenderSystems::Render`).
            .add_systems(Render, count_compiling.in_set(RenderSystems::Cleanup));
    }
    app.insert_resource(compiling)
        .init_resource::<Intro>()
        .add_systems(Startup, cover)
        .add_systems(
            Update,
            (
                // Before your stick is read, which the cover holds still;
                // and once the camera is placed, for where you are on the
                // screen.
                play.before(crate::read_player_input),
                show.after(crate::follow_camera),
            )
                .run_if(|intro: Res<Intro>| !intro.over()),
        );
}

/// In the render world, once a frame has been drawn: how many pipelines
/// are still being compiled.
fn count_compiling(cache: Option<Res<PipelineCache>>, compiling: Res<Compiling>) {
    let waiting = cache.map_or(0, |cache| cache.waiting_pipelines().count());
    compiling.0.waiting.store(waiting, Ordering::Relaxed);
    compiling.0.frames.fetch_add(1, Ordering::Relaxed);
}

/// Puts the cover up, before the first frame is drawn: the cover itself,
/// the town's name over it, and the ring of dust, which waits at nothing
/// under it so that the GPU has what draws it ready by the time you land.
fn cover(
    intro: Res<Intro>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Skipped (`RT_INTRO=off`): the town as it comes, from the first frame.
    if intro.over() {
        return;
    }
    let file = ui();
    let colours = file.colours.intro;
    commands.spawn((
        Cover,
        Name::new("Opening cover"),
        TakesPress,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(colours.cover),
        GlobalZIndex(LAYER),
    ));
    // Where the welcome stands (`hud::announce`): across the screen, high
    // enough to leave you, just below the middle, in sight.
    commands.spawn((
        Title,
        Name::new("Opening title"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::bottom(Val::Vh(hud::ABOVE_YOU)),
            ..default()
        },
        GlobalZIndex(LAYER + 2),
        Pickable::IGNORE,
        children![hud::words(file.words.welcome.clone(), hud::HEADLINE, colours.title.with_alpha(0.0))],
    ));
    commands.spawn((
        Dust,
        Name::new("Landing dust"),
        Mesh3d(meshes.add(Annulus::new(DUST_INNER, 1.0).mesh().resolution(48))),
        MeshMaterial3d(materials.add(range::look(colours.dust.with_alpha(0.0)))),
        Transform::from_scale(Vec3::splat(0.01)),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

/// Keeps the cover up until the town is there, then plays the opening:
/// your model's drop, your limbs' landing and the dust, and lets you go
/// as you stand up.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn play(
    time: Res<Time>,
    signup: Res<Signup>,
    loading: Loading,
    compiling: Res<Compiling>,
    daylight: Res<sky::Daylight>,
    mut intro: ResMut<Intro>,
    mut up: ResMut<DialogUp>,
    mut me: Query<(Entity, &Transform, Option<&mut Landing>), With<Player>>,
    limbs: Query<(Entity, &WalkBone, &GlobalTransform)>,
    parents: Query<&ChildOf>,
    mut placed: Query<&mut Transform, Without<Player>>,
    dust: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<Dust>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shown: Query<Entity, Or<(With<Cover>, With<Ring>, With<Title>, With<Dust>)>>,
    covers: Query<Entity, With<Cover>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let colour = ui().colours.intro.dust;
    let feet = me.single().ok().map(|(_, body, _)| body.translation);
    match intro.stage {
        Stage::Over => {}
        Stage::Covered => {
            up.set_if_neq(DialogUp(true));
            // The dust waits at your feet, too small and too faint to see.
            if let (Some(feet), Ok((ring, _))) = (feet, dust.single()) {
                lay_dust(&mut placed, ring, feet, 0.01);
            }
            // The sign-up's pages stand over the cover, and the wait is
            // theirs.
            if signup.up() {
                return;
            }
            let (waiting, frame) = (
                compiling.0.waiting.load(Ordering::Relaxed),
                compiling.0.frames.load(Ordering::Relaxed),
            );
            let there = loading.there();
            if !intro.wait.step(there, waiting, frame, dt) {
                return;
            }
            let at = |since: Option<f32>| since.map_or("never".to_string(), |secs| format!("{secs:.2} s"));
            let since = intro.wait.since;
            info!(
                "the opening: covered for {:.2} s (the town there at {}, every model at {}, the buses' way at {}, \
                 your body at {}; {waiting} pipelines still compiling){}",
                intro.wait.secs,
                at(since[0]),
                at(since[1]),
                at(since[2]),
                at(since[3]),
                if there.all() && waiting == 0 { "" } else { ": not all there, the wait ran out" },
            );
            // Your model, to drop: with none yet, the cover opens on you
            // as you are.
            intro.model = me.single().ok().and_then(|(you, body, _)| {
                let root = model_of(you, &limbs, &parents)?;
                let rest = *placed.get(root).ok()?;
                let hips: Vec<f32> = limbs
                    .iter()
                    .filter(|(_, limb, _)| {
                        limb.owner == you && matches!(limb.kind, WalkBoneKind::ThighL | WalkBoneKind::ThighR)
                    })
                    .map(|(_, _, place)| place.translation().y - body.translation.y)
                    .collect();
                let hip = hips.iter().sum::<f32>() / hips.len().max(1) as f32;
                Some(Model { root, rest, hip: hip.clamp(0.3, 1.2) })
            });
            intro.stage = Stage::Opening { age: 0.0, free: false };
            if let (Some(model), Ok((you, ..))) = (intro.model, me.single()) {
                let first = pose(0.0);
                commands.entity(you).insert(first.limbs());
                hang(&mut placed, model, first);
            }
        }
        Stage::Opening { age, free } => {
            let age = age + dt.min(MOST_STEP) / intro.slow;
            let now = pose(age);
            if let (Some(model), Ok((_, _, Some(mut landing)))) = (intro.model, me.single_mut()) {
                landing.set_if_neq(now.limbs());
                hang(&mut placed, model, now);
            }
            if let Ok((ring, look)) = dust.single() {
                match dust_at(age) {
                    Some((radius, solid)) => {
                        if let Some(feet) = feet {
                            lay_dust(&mut placed, ring, feet, radius);
                        }
                        if let Some(mut look) = materials.get_mut(&look.0) {
                            // As brightly as the sky lights the ground:
                            // unlit, it would shine through the night.
                            let lit = colour.to_linear() * daylight.unlit;
                            look.base_color =
                                range::bright(Color::from(lit).with_alpha(colour.alpha() * solid));
                        }
                    }
                    None if age >= FALL_SECS => {
                        commands.entity(ring).despawn();
                    }
                    None => {}
                }
            }
            let mut free = free;
            if age < UP_AT {
                up.set_if_neq(DialogUp(true));
            } else if !free {
                // On your feet: your limbs and your model your own again,
                // and the screen's presses the screen's.
                free = true;
                if let Ok((me, ..)) = me.single() {
                    commands.entity(me).remove::<Landing>();
                }
                if let Some(model) = intro.model.take() {
                    if let Ok(mut root) = placed.get_mut(model.root) {
                        root.set_if_neq(model.rest);
                    }
                }
                for cover in &covers {
                    commands.entity(cover).despawn();
                }
                if !signup.up() {
                    up.set_if_neq(DialogUp(false));
                }
            }
            if age >= TITLE_STAYS + TITLE_OUT && free {
                for left in &shown {
                    commands.entity(left).despawn();
                }
                intro.stage = Stage::Over;
            } else {
                intro.stage = Stage::Opening { age, free };
            }
        }
    }
}

/// The node of `me`'s model that its limbs hang under: the one child of
/// its body that is drawn with it, whatever else its body carries.
fn model_of(
    me: Entity,
    limbs: &Query<(Entity, &WalkBone, &GlobalTransform)>,
    parents: &Query<&ChildOf>,
) -> Option<Entity> {
    let (mut at, ..) = limbs.iter().find(|(_, limb, _)| limb.owner == me)?;
    loop {
        let above = parents.get(at).ok()?.parent();
        if above == me {
            return Some(at);
        }
        at = above;
    }
}

/// Hangs `model` over your body as `pose` has it: that high over where
/// you stand, down by as much as its bent knees take off its legs, drawn
/// that much taller or shorter, and leant forward.
fn hang(placed: &mut Query<&mut Transform, Without<Player>>, model: Model, pose: Pose) {
    let Ok(mut root) = placed.get_mut(model.root) else {
        return;
    };
    let wide = pose.tall.recip().sqrt();
    let up = pose.height - pose.tall * pose.limbs().sink(model.hip);
    root.set_if_neq(Transform {
        translation: model.rest.translation + Vec3::Y * up,
        // Forward is the body's -z, whatever way its model is turned on
        // it.
        rotation: Quat::from_rotation_x(-pose.lean) * model.rest.rotation,
        scale: model.rest.scale * Vec3::new(wide, pose.tall, wide),
    });
}

/// Lays the ring of dust on the ground round `feet`, `radius` metres
/// across its outer edge from its middle.
fn lay_dust(placed: &mut Query<&mut Transform, Without<Player>>, ring: Entity, feet: Vec3, radius: f32) {
    if let Ok(mut place) = placed.get_mut(ring) {
        place.set_if_neq(Transform {
            translation: feet + Vec3::Y * DUST_OVER,
            // The ring is made standing up, facing the body's back.
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(radius),
        });
    }
}

/// Draws the cover as it stands, once the camera is placed: the town's
/// name coming up, staying and going; the circle opening about where you
/// are on the screen; and the camera dipping as you land.
#[allow(clippy::type_complexity)]
fn show(
    time: Res<Time>,
    intro: Res<Intro>,
    windows: Query<&Window, With<PrimaryWindow>>,
    me: Query<&Transform, With<Player>>,
    mut cameras: Query<(&mut Camera, &mut Transform), (With<ThirdPersonCamera>, Without<Player>)>,
    mut covers: Query<&mut BackgroundColor, With<Cover>>,
    mut rings: Query<(Entity, &mut Node, &mut UiTransform, &mut BorderColor), With<Ring>>,
    titles: Query<&Children, With<Title>>,
    mut lines: Query<(&mut TextColor, &mut TextShadow)>,
    mut commands: Commands,
) {
    let file = ui();
    let colours = file.colours.intro;
    let age = match intro.stage {
        Stage::Covered => None,
        Stage::Opening { age, .. } => Some(age),
        Stage::Over => return,
    };

    // The town's name.
    let solid = match age {
        None => (time.elapsed_secs() / TITLE_IN).min(1.0),
        Some(age) => (1.0 - (age - TITLE_STAYS) / TITLE_OUT).clamp(0.0, 1.0),
    };
    let tint = file.colours.text_shadow;
    for children in &titles {
        let mut words = lines.iter_many_mut(children);
        while let Some((mut colour, mut shadow)) = words.fetch_next() {
            let faded = colours.title.with_alpha(colours.title.alpha() * solid);
            if colour.0 != faded {
                colour.0 = faded;
                shadow.color = tint.with_alpha(tint.alpha() * solid);
            }
        }
    }

    let Ok((mut camera, mut eye)) = cameras.single_mut() else {
        return;
    };
    // Covered, what the camera clears the frame to is the cover's colour
    // too: the first frames are drawn before the GPU has what draws the
    // cover itself ready, and would be the bare sky.
    let clear = match age {
        None => ClearColorConfig::Custom(colours.cover),
        Some(_) => ClearColorConfig::Default,
    };
    let cleared = match (&camera.clear_color, &clear) {
        (ClearColorConfig::Custom(now), ClearColorConfig::Custom(wanted)) => now == wanted,
        (ClearColorConfig::Default, ClearColorConfig::Default) => true,
        _ => false,
    };
    if !cleared {
        camera.clear_color = clear;
    }
    let Some(age) = age else {
        for mut fill in &mut covers {
            fill.set_if_neq(BackgroundColor(colours.cover));
        }
        return;
    };

    // The dip, as you land.
    let dip = dip_at(age);
    if dip > 0.0 {
        eye.translation.y -= dip;
    }

    // The circle, about where you stand on the screen.
    let open = hole(age);
    if open >= 1.0 {
        for (ring, ..) in &rings {
            commands.entity(ring).despawn();
        }
        for mut fill in &mut covers {
            fill.set_if_neq(BackgroundColor(Color::NONE));
        }
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let screen = Vec2::new(window.width(), window.height());
    let centre = me
        .single()
        .ok()
        .and_then(|body| {
            camera
                .world_to_viewport(&GlobalTransform::from(*eye), body.translation + Vec3::Y * CENTRE)
                .ok()
        })
        .filter(|centre| centre.is_finite())
        .map_or(screen * 0.5, |centre| centre.clamp(Vec2::ZERO, screen));
    let (radius, thick) = ring(centre, open, screen);
    let side = 2.0 * (radius + thick);
    let place = UiTransform {
        translation: Val2::px(centre.x - side * 0.5, centre.y - side * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    };
    let colour = BorderColor::all(colours.cover);
    let Ok((_, mut node, mut placed, mut border)) = rings.single_mut() else {
        // The circle, shut: put up over the cover, which is taken from
        // under it once it is there.
        commands.spawn((
            Ring,
            Name::new("Opening circle"),
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
            place,
            GlobalZIndex(LAYER + 1),
            Pickable::IGNORE,
        ));
        return;
    };
    if node.width != Val::Px(side) || node.border.left != Val::Px(thick) {
        node.width = Val::Px(side);
        node.height = Val::Px(side);
        node.border = UiRect::all(Val::Px(thick));
    }
    placed.set_if_neq(place);
    border.set_if_neq(colour);
    for mut fill in &mut covers {
        fill.set_if_neq(BackgroundColor(Color::NONE));
    }
}

/// How you are drawn at one moment of the opening.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Pose {
    /// How high over where you stand your feet are, in metres.
    height: f32,
    /// How far your knees are bent, 0 to 1 ([`Landing`]).
    squat: f32,
    /// How far up your arms are, 0 to 1.
    arms: f32,
    /// How tall you are drawn, 1 as you are; and as much thinner as keeps
    /// you as much of you.
    tall: f32,
    /// How far forward you lean, in radians.
    lean: f32,
}

impl Pose {
    fn limbs(&self) -> Landing {
        Landing {
            squat: self.squat,
            arms: self.arms,
        }
    }
}

/// How you are drawn `age` seconds into the opening: falling, arms up
/// and stretched a little, faster as you come; landing, down into a
/// squat, squashed, arms out and leaning in; and springing back up to
/// stand as you stand, from [`UP_AT`] on.
fn pose(age: f32) -> Pose {
    if age < FALL_SECS {
        let fallen = (age / FALL_SECS).max(0.0);
        return Pose {
            height: FALL_FROM * (1.0 - fallen.powf(FALL_GATHERS)),
            squat: AIR_SQUAT,
            arms: 1.0,
            tall: AIR_TALL,
            lean: 0.0,
        };
    }
    let landed = age - FALL_SECS;
    let rising = ((landed - SQUAT_SECS - DOWN_SECS) / RISE_SECS).clamp(0.0, 1.0);
    let (sunk, risen) = (ease_out((landed / SQUAT_SECS).min(1.0)), ease_out(rising));
    let down = AIR_TALL + (SQUASHED - AIR_TALL) * sunk;
    Pose {
        height: 0.0,
        squat: (AIR_SQUAT + (1.0 - AIR_SQUAT) * sunk) * (1.0 - risen),
        arms: (1.0 + (ARMS_OUT - 1.0) * sunk) * (1.0 - risen),
        tall: down + (1.0 - down) * risen + SPRING * (std::f32::consts::PI * rising).sin(),
        lean: LEAN * sunk * (1.0 - risen),
    }
}

/// How far open the circle is `age` seconds into the opening, 0 shut to 1
/// past every corner of the screen: slowly away, quickly through the
/// middle, slowly in.
fn hole(age: f32) -> f32 {
    let share = (age / OPEN_SECS).clamp(0.0, 1.0);
    share * share * (3.0 - 2.0 * share)
}

/// The ring of dust `age` seconds into the opening, from the moment you
/// land for [`DUST_SECS`]: how far out its outer edge is, in metres,
/// quickly at first, and how much of its colour is left.
fn dust_at(age: f32) -> Option<(f32, f32)> {
    let share = (age - FALL_SECS) / DUST_SECS;
    (0.0..1.0).contains(&share).then(|| {
        let out = ease_out(share);
        (DUST_FROM + (DUST_TO - DUST_FROM) * out, (1.0 - share) * (1.0 - share))
    })
}

/// How far the camera has dipped `age` seconds into the opening, in
/// metres: down at once as you land, and back up.
fn dip_at(age: f32) -> f32 {
    let share = (age - FALL_SECS) / DIP_SECS;
    if !(0.0..1.0).contains(&share) {
        return 0.0;
    }
    DIP * (share / 0.25).min(1.0) * (1.0 - share) * (1.0 - share)
}

/// Quickly away and slowly in, 0 to 1.
fn ease_out(share: f32) -> f32 {
    let left = 1.0 - share.clamp(0.0, 1.0);
    1.0 - left * left * left
}

/// The circle `open` of the way, 0 to 1, about `centre` on a `screen`
/// that big, in the screen's own pixels: how far out its hole reaches,
/// and how thick the dark round it has to be to reach every corner of the
/// screen from wherever the middle is. Wide open, the hole is past every
/// corner.
fn ring(centre: Vec2, open: f32, screen: Vec2) -> (f32, f32) {
    let furthest = [Vec2::ZERO, Vec2::new(screen.x, 0.0), Vec2::new(0.0, screen.y), screen]
        .into_iter()
        .map(|corner| corner.distance(centre))
        .fold(0.0, f32::max);
    ((open * (furthest + 2.0)).round(), screen.length().ceil() + 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn you_fall_in_from_over_the_screen_land_squat_and_stand_as_you_stand() {
        // From far up, faster and faster, and on the ground from the
        // moment you land.
        assert_eq!(pose(0.0).height, FALL_FROM);
        let mut last = pose(0.0);
        let mut speed = 0.0;
        for step in 1..=40 {
            let now = pose(FALL_SECS * step as f32 / 40.0);
            let fell = last.height - now.height;
            assert!(fell > speed, "falling faster: {fell} after {speed}");
            (last, speed) = (now, fell);
        }
        assert!(last.height < 1e-3, "{last:?}");
        assert_eq!(pose(FALL_SECS + 0.001).height, 0.0);
        // Over the top of the screen as it begins: the camera the app
        // opens with sees no higher than about 7 m over where you stand.
        assert!(pose(0.0).height > 9.0);
        // In the air: arms up, knees a little bent, drawn a little tall.
        let air = pose(FALL_SECS * 0.5);
        assert!(air.arms == 1.0 && air.squat > 0.0 && air.squat < 0.5 && air.tall > 1.0 && air.lean == 0.0);
        // Landed: down into the squat at once, squashed, arms out, leaning.
        let deep = pose(FALL_SECS + SQUAT_SECS + DOWN_SECS * 0.5);
        assert!(deep.height == 0.0 && (deep.squat - 1.0).abs() < 1e-5, "{deep:?}");
        assert!((deep.tall - SQUASHED).abs() < 1e-5 && (deep.arms - ARMS_OUT).abs() < 1e-5 && deep.lean > 0.1);
        // And up again: through a little taller than you are, to stand
        // just as you stand, and for good.
        let springing = pose(FALL_SECS + SQUAT_SECS + DOWN_SECS + RISE_SECS * 0.5);
        assert!(springing.tall > 1.0 && springing.squat < 0.3, "{springing:?}");
        for after in [0.0, 0.1, 5.0] {
            let up = pose(UP_AT + after);
            let standing = Pose { height: 0.0, squat: 0.0, arms: 0.0, tall: 1.0, lean: 0.0 };
            for (now, stood) in [
                (up.height, standing.height),
                (up.squat, standing.squat),
                (up.arms, standing.arms),
                (up.tall, standing.tall),
                (up.lean, standing.lean),
            ] {
                assert!((now - stood).abs() < 1e-5, "{up:?}");
            }
        }
        // All of it short: Hajun asked for half a second to a second.
        assert!((0.5..=1.0).contains(&UP_AT), "{UP_AT}");
        assert!(OPEN_SECS <= UP_AT);
    }

    #[test]
    fn the_circle_opens_from_nothing_to_past_every_corner_and_is_wide_by_the_time_you_are_in_sight() {
        assert_eq!(hole(0.0), 0.0);
        assert_eq!(hole(OPEN_SECS), 1.0);
        assert_eq!(hole(OPEN_SECS + 1.0), 1.0);
        let mut last = 0.0;
        for step in 1..=50 {
            let now = hole(OPEN_SECS * step as f32 / 50.0);
            assert!(now > last);
            last = now;
        }
        // Shut, the dark covers every corner of the screen wherever its
        // middle is on it; open, its hole is past them all.
        let screen = Vec2::new(1600.0, 900.0);
        for centre in [Vec2::new(800.0, 540.0), Vec2::ZERO, screen, Vec2::new(1600.0, 0.0)] {
            let (shut, thick) = ring(centre, 0.0, screen);
            assert_eq!(shut, 0.0);
            let (wide, _) = ring(centre, 1.0, screen);
            for corner in [Vec2::ZERO, Vec2::new(1600.0, 0.0), Vec2::new(0.0, 900.0), screen] {
                assert!(corner.distance(centre) <= thick, "the dark reaches {corner:?} from {centre:?}");
                assert!(corner.distance(centre) < wide, "the hole is past {corner:?} from {centre:?}");
            }
        }
        // The circle has reached you well before you land, so that you are
        // seen coming down: two thirds of the way through the fall its
        // edge is further from its middle, your chest once you stand,
        // than your feet are (the furthest corner of the screen is 13 m
        // of the town or more from there, at your distance from the
        // camera the app opens with); and landing, it is most of the way
        // open.
        let age = FALL_SECS * 0.65;
        assert!(13.0 * hole(age) > pose(age).height - CENTRE, "{} {}", hole(age), pose(age).height);
        assert!(hole(FALL_SECS) > 0.75, "{}", hole(FALL_SECS));
    }

    #[test]
    fn the_dust_goes_out_from_your_feet_as_you_land_and_the_camera_dips() {
        assert_eq!(dust_at(FALL_SECS - 0.01), None);
        let (begun, solid) = dust_at(FALL_SECS).unwrap();
        assert_eq!((begun, solid), (DUST_FROM, 1.0));
        let (later, fainter) = dust_at(FALL_SECS + DUST_SECS * 0.5).unwrap();
        assert!(later > begun && later < DUST_TO && fainter < 0.5 && fainter > 0.0);
        assert_eq!(dust_at(FALL_SECS + DUST_SECS + 0.001), None);
        assert_eq!(dip_at(FALL_SECS - 0.01), 0.0);
        assert_eq!(dip_at(FALL_SECS + DIP_SECS + 0.001), 0.0);
        let deepest = (0..=100).map(|step| dip_at(FALL_SECS + DIP_SECS * step as f32 / 100.0)).fold(0.0, f32::max);
        assert!(deepest > 0.04 && deepest <= DIP, "{deepest}");
    }

    #[test]
    fn the_cover_waits_for_the_town_and_for_the_gpu_and_no_longer_than_its_most() {
        let frame = 1.0 / 120.0;
        let (all, none) = (There([true; 4]), There([false; 4]));
        // Nothing there: it stays, however quiet the GPU is.
        let mut wait = Wait::default();
        for drawn in 1..=100 {
            assert!(!wait.step(none, 0, drawn, frame));
        }
        // Everything but the buses' way: it stays.
        for drawn in 101..=200 {
            assert!(!wait.step(There([true, true, false, true]), 0, drawn, frame));
        }
        // All there, the GPU still compiling: it stays.
        for drawn in 201..=300 {
            assert!(!wait.step(all, 3, drawn, frame));
        }
        // Quiet: it opens once that many frames have been drawn so, and
        // not on the same frame counted twice.
        for _ in 0..50 {
            assert!(!wait.step(all, 0, 301, frame));
        }
        for drawn in 302..(300 + QUIET_FRAMES) {
            assert!(!wait.step(all, 0, drawn, frame), "frame {drawn}");
        }
        assert!(wait.step(all, 0, 300 + QUIET_FRAMES, frame));
        assert_eq!(wait.since.map(|since| since.is_some()), [true; 4]);
        // Something to compile again starts the count over; and it does
        // not open on a frame that hung, but on the next that did not.
        let mut wait = Wait::default();
        for drawn in 1..=QUIET_FRAMES - 1 {
            assert!(!wait.step(all, 0, drawn, frame));
        }
        assert!(!wait.step(all, 1, QUIET_FRAMES, frame));
        for drawn in 1..QUIET_FRAMES {
            assert!(!wait.step(all, 0, QUIET_FRAMES + drawn, frame));
        }
        assert!(!wait.step(all, 0, 2 * QUIET_FRAMES, 0.3));
        assert!(wait.step(all, 0, 2 * QUIET_FRAMES + 1, frame));
        // And with something never coming, it opens anyway in the end.
        let mut wait = Wait::default();
        let mut opened = None;
        for drawn in 1..=3000 {
            if wait.step(none, 5, drawn, frame) {
                opened = Some(wait.secs);
                break;
            }
        }
        assert!(opened.is_some_and(|secs| (secs - MOST_COVERED).abs() < 0.05), "{opened:?}");
    }
}
