//! The bubble that comes up by something you can talk to, open or rest on:
//! a thing in the world, since 2026-11-22.
//!
//! It was a button on the screen, as big there however far off the camera
//! was (`talk`, from 2026-10-31), and so it grew over the town as the
//! camera went out. Hajun: "i dont want proximity bubbles in general to
//! scale when u zoom the camera out, the scale should stay the same, so
//! instead of a screen based ui button, make it a in-world ui button that
//! just rotates according to camera viewing angle, similar to how the text
//! in the "player's land" sign is made".
//!
//! So it is a flat sheet with its picture on it, as the words on a plot's
//! sign are (`land`): [`RADIUS`] across its circle in the world wherever
//! the camera is, the point of its tail on a point of the world
//! ([`Bubble::tip`]), and turned every frame to face the camera squarely
//! ([`face_the_camera`]), which is all that the camera changes of it. It is
//! drawn in the world, behind whatever stands in front of it; unlit, its
//! own white by day and by night. It pops up as it comes, as it did.
//!
//! **A tap on it** ([`tap`]) is a tap on the world that lands on it, or
//! near it where it is small on the screen, never less than a finger's
//! width from its middle: whoever put the bubble up hears of it
//! ([`Tapped`]) and nothing else does, the editor's menu
//! (`EditMenu::swallow`) and the paintbrush ([`TookTap`]) among them. With
//! two in reach of one tap, the nearer to the finger.
//!
//! **What is in it** ([`Says`]) is drawn in code, as the bubble always was,
//! by how far each dot is from each edge, and drawn again at every smaller
//! size the picture is kept at, so that it is as clean far off as near:
//! three dots (to talk, to open), someone sat on a chair, someone lying on
//! a bed (`rest`); and since 2026-11-27 a chest (something that keeps
//! things), a bulb (a light switched by hand) and a drop of water (the
//! sink's tap), for `working`.
//!
//! **E on desktop is a tap on the bubble nearest you** (since 2026-11-27,
//! when there came to be three kinds of thing to use beside someone to
//! talk to and somewhere to rest, each of which had answered E for itself:
//! a chair by a cabinet by someone to talk to would have had all three).

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::ThirdPersonCamera;
use crate::editor::{self, EditMenu, EditPointer};
use crate::hud::{self, DialogUp};
use crate::island::Venue;
use crate::ui::ui;

/// How big the bubble's circle is, in metres from its middle to its edge:
/// a little more across than a head (0.53 m), which it mostly stands over.
/// Mine.
pub(crate) const RADIUS: f32 = 0.31;
/// The least a tap is taken within of a bubble's middle, as a share of the
/// short side of the screen in parts of a hundred: far off, a bubble is
/// smaller than a finger.
const TAP_WITHIN: f32 = 5.0;
/// How long it takes to pop up, in seconds.
const POP_SECS: f32 = 0.18;
/// How many times as bright as its picture it is drawn, being unlit, to
/// come out as white as its picture is (`land::WORDS_GAIN`, `range::GAIN`).
const GAIN: f32 = 2.5;
/// How many dots of its picture go to the radius of its circle, at the
/// picture's full size.
const DOTS: f32 = 128.0;

/// The bubble, in shares of the radius of its circle, from the middle of it,
/// y up: the edges of its picture, and its tail's point, down and to the left,
/// and the two corners where it meets the circle, inside it.
const LEFT: f32 = -1.06;
const RIGHT: f32 = 1.06;
const TOP: f32 = 1.06;
const BOTTOM: f32 = -1.4;
const TAIL_TIP: Vec2 = Vec2::new(-0.74, -1.3);
const TAIL_FROM: Vec2 = Vec2::new(-0.75, -0.35);
const TAIL_TO: Vec2 = Vec2::new(-0.12, -0.8);
/// How much the tail's point is rounded off, how smoothly it runs into the
/// circle, how thick the outline is, and the three dots: how far apart and
/// how big.
const TAIL_ROUND: f32 = 0.06;
const TAIL_JOIN: f32 = 0.22;
const OUTLINE: f32 = 0.1;
const DOT_APART: f32 = 0.42;
const DOT: f32 = 0.13;

/// What is in a bubble.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Says {
    /// Three dots: someone to talk to, something to open (`talk`).
    Dots,
    /// Someone sat on a chair: somewhere to sit (`rest`).
    Sit,
    /// Someone lying on a bed: somewhere to lie down (`rest`).
    Lie,
    /// A chest: something that keeps things, to open (`working`, `storage`).
    Store,
    /// A light bulb: a light to switch on or off (`working`).
    Light,
    /// A drop of water: a tap to run (`working`).
    Water,
    /// A piggy bank with a coin over its slot: to put coins in and take
    /// them out (`working`, `storage`).
    Piggy,
}

impl Says {
    pub(crate) const ALL: [Says; 7] = [Says::Dots, Says::Sit, Says::Lie, Says::Store, Says::Light, Says::Water, Says::Piggy];
}

/// A bubble in the world: the point of the world its tail's point is on,
/// and how long it has been up.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Bubble {
    pub(crate) tip: Vec3,
    age: f32,
}

/// A bubble was tapped: this one.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Tapped(pub(crate) Entity);

/// Whether a bubble took the tap that came up this frame: nothing else in
/// the world is to answer it.
#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub(crate) struct TookTap(pub(crate) bool);

/// The bubbles' sheet and what each is drawn with, made once: one for each
/// of [`Says::ALL`], in its order.
#[derive(Resource)]
pub(crate) struct Looks {
    sheet: Handle<Mesh>,
    paints: Vec<Handle<StandardMaterial>>,
}

impl Looks {
    /// A bubble with `says` in it, its point on `tip`, on the island
    /// `venue`, where alone it is drawn (`island::show_where_i_am`). It is
    /// nothing to look at until it has been turned to the camera
    /// ([`face_the_camera`]), the frame after.
    pub(crate) fn bubble(&self, says: Says, tip: Vec3, venue: Venue) -> impl Bundle {
        let paint = &self.paints[Says::ALL.iter().position(|&one| one == says).unwrap_or(0)];
        (
            Bubble { tip, age: 0.0 },
            Name::new("Bubble"),
            venue,
            Mesh3d(self.sheet.clone()),
            MeshMaterial3d(paint.clone()),
            Transform::from_translation(tip).with_scale(Vec3::ZERO),
            Visibility::Hidden,
            NotShadowCaster,
        )
    }
}

/// Turning the bubbles to the camera: after it is where it is this frame.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BubbleSystems;

/// Looking for a tap on one ([`tap`]): whoever put a bubble up reads
/// [`Tapped`] after.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Taps;

pub(crate) fn plugin(app: &mut App) {
    // Once the pointer has been read this frame, a finger's by the touch
    // controls and the mouse's just ahead of the editor, and before the
    // editor, which a tap on a bubble is not for.
    let tap = tap.in_set(Taps).after(crate::follow_camera).before(editor::EditSystems);
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let tap = tap.after(crate::read_touch_controls);
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let tap = tap.after(editor::read_desktop_pointer);
    app.add_message::<Tapped>()
        .init_resource::<TookTap>()
        .add_systems(Startup, make_looks)
        .add_systems(Update, (face_the_camera.in_set(BubbleSystems).after(crate::follow_camera), tap));
}

fn make_looks(mut meshes: ResMut<Assets<Mesh>>, mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let mut paint = |says: Says| {
        materials.add(StandardMaterial {
            base_color: Color::linear_rgb(GAIN, GAIN, GAIN),
            base_color_texture: Some(images.add(picture(says))),
            // Its own white and black in sun and in shade, by day and by
            // night, as it was on the screen.
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    };
    let looks = Looks {
        paints: Says::ALL.into_iter().map(&mut paint).collect(),
        sheet: meshes.add(Rectangle::new((RIGHT - LEFT) * RADIUS, (TOP - BOTTOM) * RADIUS)),
    };
    commands.insert_resource(looks);
}

/// How big a bubble is drawn `age` seconds after it came up, as a share of
/// its size: it pops up, overshooting a little before it settles.
fn popped(age: f32) -> f32 {
    let popped = (age / POP_SECS).clamp(0.0, 1.0);
    0.6 + 0.4 * (1.0 + 2.7 * (popped - 1.0).powi(3) + 1.7 * (popped - 1.0).powi(2))
}

/// Where the middle of a bubble's sheet is from the point of its tail, in
/// metres across and up the sheet, at its full size.
fn middle_from_tip() -> Vec2 {
    Vec2::new((LEFT + RIGHT) * 0.5 - TAIL_TIP.x, (TOP + BOTTOM) * 0.5 - TAIL_TIP.y) * RADIUS
}

/// Where the middle of a bubble's circle is from the point of its tail,
/// likewise.
fn circle_from_tip() -> Vec2 {
    -TAIL_TIP * RADIUS
}

/// How a bubble whose point is on `tip` stands to a camera turned as `eye`
/// is, `age` seconds after it came up: facing it squarely, the point of
/// its tail on `tip` whatever its size.
fn stood(tip: Vec3, eye: Quat, age: f32) -> Transform {
    let size = popped(age);
    Transform {
        translation: tip + eye * (middle_from_tip() * size).extend(0.0),
        rotation: eye,
        scale: Vec3::splat(size),
    }
}

/// Turns every bubble to the camera, its point where it is to be, and pops
/// it up as it comes. Only what has changed is written: with the camera
/// still, nothing.
fn face_the_camera(
    time: Res<Time>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Bubble>)>,
    mut bubbles: Query<(&mut Bubble, &mut Transform)>,
) {
    let Ok(eye) = cameras.single() else {
        return;
    };
    let dt = time.delta_secs();
    for (mut bubble, mut place) in &mut bubbles {
        if bubble.age < POP_SECS {
            bubble.age = (bubble.age + dt).min(POP_SECS);
        }
        let now = stood(bubble.tip, eye.rotation, bubble.age);
        if *place != now {
            *place = now;
        }
    }
}

/// A tap on the world that lands on a bubble, or near enough to one: that
/// bubble is told of ([`Tapped`]), and the tap is nobody else's
/// ([`TookTap`], `EditMenu::swallow`). Not while a window is up, which
/// every press is then for. And E on desktop: the bubble nearest you.
#[allow(clippy::too_many_arguments)]
fn tap(
    pointer: Res<EditPointer>,
    (dialog, keyboard): (Res<DialogUp>, Res<ButtonInput<KeyCode>>),
    windows: Query<&Window>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    me: Query<&Transform, With<crate::Player>>,
    bubbles: Query<(Entity, &Bubble, &InheritedVisibility)>,
    mut menu: ResMut<EditMenu>,
    mut took: ResMut<TookTap>,
    mut tapped: MessageWriter<Tapped>,
) {
    took.set_if_neq(TookTap(false));
    if dialog.0 || bubbles.is_empty() {
        return;
    }
    // Not the E that has just put a window away (a talk's goodbye).
    if keyboard.just_pressed(KeyCode::KeyE)
        && !dialog.is_changed()
        && let Ok(you) = me.single()
        && let Some(nearest) = nearest_to(you.translation, bubbles.iter().filter(|(.., shown)| shown.get()).map(|(entity, bubble, _)| (entity, bubble.tip)))
    {
        tapped.write(Tapped(nearest));
        return;
    }
    if !pointer.released || pointer.lost || pointer.travel > editor::TAP_SLOP {
        return;
    }
    let (Ok(window), Ok((camera, eye))) = (windows.single(), cameras.single()) else {
        return;
    };
    let least = TAP_WITHIN * window.width().min(window.height()) / 100.0;
    // The camera has no parent: this is where it is this frame.
    let view = GlobalTransform::from(*eye);
    let nearest = bubbles
        .iter()
        .filter(|(.., shown)| shown.get())
        .filter_map(|(entity, bubble, _)| {
            let size = popped(bubble.age);
            let middle = bubble.tip + eye.rotation * (circle_from_tip() * size).extend(0.0);
            let rim = middle + eye.rotation * Vec3::X * (RADIUS * size);
            let at = camera.world_to_viewport(&view, middle).ok()?;
            let across = camera.world_to_viewport(&view, rim).ok()?.distance(at);
            let off = at.distance(pointer.pos);
            (off <= across.max(least)).then_some((off, entity))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((_, bubble)) = nearest {
        tapped.write(Tapped(bubble));
        took.set_if_neq(TookTap(true));
        menu.swallow();
    }
}

/// Of `bubbles`, each with the point of its tail, the one whose point is
/// nearest `you`.
fn nearest_to(you: Vec3, bubbles: impl Iterator<Item = (Entity, Vec3)>) -> Option<Entity> {
    bubbles
        .map(|(entity, tip)| (tip.distance_squared(you), entity))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, entity)| entity)
}

// ------------------------------------------------------------- the picture

/// How far `at` is from the edge of the bubble, its circle and its tail, in
/// shares of the circle's radius: less than nothing inside it.
fn bubble_distance(at: Vec2) -> f32 {
    let circle = at.length() - 1.0;
    let tail = hud::triangle_distance(at, TAIL_FROM, TAIL_TIP, TAIL_TO) - TAIL_ROUND;
    // The two run into each other smoothly, rather than meeting at a corner.
    let h = (0.5 + 0.5 * (tail - circle) / TAIL_JOIN).clamp(0.0, 1.0);
    tail + (circle - tail) * h - TAIL_JOIN * h * (1.0 - h)
}

/// How far `at` is from a stroke `thick` across from `from` to `to`, its
/// ends round: less than nothing inside it.
fn stroke(at: Vec2, from: Vec2, to: Vec2, thick: f32) -> f32 {
    let along = to - from;
    let share = ((at - from).dot(along) / along.length_squared().max(1e-9)).clamp(0.0, 1.0);
    at.distance(from + along * share) - thick * 0.5
}

/// How far `at` is from what is drawn in a bubble that `says`, in shares of
/// the circle's radius: less than nothing on it.
fn mark_distance(says: Says, at: Vec2) -> f32 {
    let nearest = |strokes: &[(Vec2, Vec2, f32)], head: (Vec2, f32)| {
        strokes
            .iter()
            .map(|&(from, to, thick)| stroke(at, from, to, thick))
            .fold(at.distance(head.0) - head.1, f32::min)
    };
    let v = Vec2::new;
    match says {
        Says::Dots => [-DOT_APART, 0.0, DOT_APART]
            .into_iter()
            .map(|x| at.distance(v(x, 0.0)) - DOT)
            .fold(f32::INFINITY, f32::min),
        // Seen from the side, facing right: a head, a back, a lap and a
        // shin, on a chair's back, seat and two legs, drawn thinner.
        Says::Sit => nearest(
            &[
                (v(-0.14, 0.14), v(-0.14, -0.12), 0.27),
                (v(-0.12, -0.13), v(0.30, -0.13), 0.24),
                (v(0.31, -0.13), v(0.31, -0.55), 0.20),
                (v(-0.44, 0.34), v(-0.44, -0.62), 0.11),
                (v(-0.44, -0.33), v(0.12, -0.33), 0.11),
                (v(0.10, -0.33), v(0.10, -0.62), 0.11),
            ],
            (v(-0.13, 0.47), 0.17),
        ),
        // Seen from the side: a head on a pillow and a body along a bed,
        // on the bed's board and its head and foot, drawn thinner.
        Says::Lie => nearest(
            &[
                (v(-0.12, -0.02), v(0.50, -0.02), 0.25),
                (v(-0.50, -0.14), v(-0.26, -0.14), 0.12),
                (v(-0.64, -0.26), v(0.64, -0.26), 0.11),
                (v(-0.64, 0.24), v(-0.64, -0.48), 0.11),
                (v(0.64, -0.08), v(0.64, -0.48), 0.11),
            ],
            (v(-0.37, 0.06), 0.16),
        ),
        // A chest, from the front: its box and its rounded lid, a line
        // between them, and the clasp over that line left white.
        Says::Store => {
            let chest = rounded_box(at - v(0.0, -0.1), v(0.5, 0.34), 0.06).min(rounded_box(at - v(0.0, 0.26), v(0.5, 0.14), 0.12));
            let seam = rounded_box(at - v(0.0, 0.12), v(0.56, 0.028), 0.0);
            let clasp = rounded_box(at - v(0.0, 0.06), v(0.075, 0.1), 0.03);
            chest.max(-seam).max(-clasp)
        }
        // A light bulb, lit: its glass, its neck and the two rings of its
        // screw, and three rays over it.
        Says::Light => {
            let glass = at.distance(v(0.0, 0.16)) - 0.31;
            let neck = rounded_box(at - v(0.0, -0.2), v(0.15, 0.1), 0.02);
            let rings = rounded_box(at - v(0.0, -0.37), v(0.13, 0.035), 0.03).min(rounded_box(at - v(0.0, -0.47), v(0.09, 0.03), 0.03));
            let rays = [(v(0.0, 0.6), v(0.0, 0.72)), (v(-0.4, 0.46), v(-0.5, 0.55)), (v(0.4, 0.46), v(0.5, 0.55))]
                .into_iter()
                .map(|(from, to)| stroke(at, from, to, 0.09))
                .fold(f32::INFINITY, f32::min);
            glass.min(neck).min(rings).min(rays)
        }
        // A drop of water, round below and coming to a point above, with a
        // glint in it left white.
        Says::Water => {
            let round = at.distance(v(0.0, -0.17)) - 0.33;
            let point = hud::triangle_distance(at, v(-0.29, -0.02), v(0.0, 0.6), v(0.29, -0.02));
            let drop = round.min(point);
            let glint = stroke(at, v(-0.15, -0.12), v(-0.08, -0.3), 0.07);
            drop.max(-glint)
        }
        // A piggy bank from the side, facing right: its round body, its
        // snout, an ear, two legs and a curl of a tail; its eye and the slot
        // on its back left white; and a coin over the slot. (A coin alone,
        // drawn this small, read as a sign saying no entry.)
        Says::Piggy => {
            let body = ((at - v(-0.02, -0.06)) / v(0.5, 0.33)).length() - 1.0;
            let body = body * 0.33;
            let snout = rounded_box(at - v(0.5, -0.04), v(0.1, 0.11), 0.05);
            let ear = hud::triangle_distance(at, v(0.16, 0.2), v(0.3, 0.42), v(0.36, 0.18));
            let legs = rounded_box(at - v(-0.25, -0.38), v(0.075, 0.12), 0.04).min(rounded_box(at - v(0.2, -0.38), v(0.075, 0.12), 0.04));
            let tail = stroke(at, v(-0.5, 0.0), v(-0.62, 0.1), 0.07);
            let eye = at.distance(v(0.3, 0.04)) - 0.04;
            let slot = rounded_box(at - v(-0.06, 0.2), v(0.15, 0.035), 0.03);
            let coin = at.distance(v(-0.06, 0.52)) - 0.13;
            body.min(snout).min(ear).min(legs).min(tail).max(-eye).max(-slot).min(coin)
        }
    }
}

/// How far `at` is from the edge of a box reaching `reach` each way from its
/// middle, its corners rounded by `round`: less than nothing inside it.
fn rounded_box(at: Vec2, reach: Vec2, round: f32) -> f32 {
    let d = at.abs() - reach + Vec2::splat(round);
    d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0) - round
}

/// One size of the picture of a bubble that `says`, `wide` by `high` dots:
/// white inside a dark outline, what it says dark in it, and clear round
/// it; each dot by how much of it is inside each edge.
fn drawn(says: Says, wide: u32, high: u32, ink: f32) -> Vec<u8> {
    let dot = Vec2::new((RIGHT - LEFT) / wide as f32, (TOP - BOTTOM) / high as f32);
    let fine = dot.x.max(dot.y);
    let mut data = Vec::with_capacity((wide * high * 4) as usize);
    for row in 0..high {
        for column in 0..wide {
            let at = Vec2::new(LEFT + (column as f32 + 0.5) * dot.x, TOP - (row as f32 + 0.5) * dot.y);
            let edge = bubble_distance(at);
            let cover = |distance: f32| (0.5 - distance / fine).clamp(0.0, 1.0);
            let white = cover(edge + OUTLINE) * (1.0 - cover(mark_distance(says, at)));
            let value = ((ink + (1.0 - ink) * white) * 255.0).round() as u8;
            data.extend_from_slice(&[value, value, value, (cover(edge) * 255.0).round() as u8]);
        }
    }
    data
}

/// The picture of a bubble that `says` (Hajun, 2026-10-31, of the first:
/// "small minimalistic black outline, white inner, circular chatbubble with
/// 3 black dots"), with every smaller size of it down to a single dot, each
/// drawn for itself: far off it is drawn from one about its size on the
/// screen, and does not sparkle.
fn picture(says: Says) -> Image {
    // The outline's and the marks' shade of grey, the ink's red: it is
    // drawn grey on white, and the file's ink gives it its darkness.
    let ink = ui().colours.talk.ink.to_srgba().red;
    let wide = ((RIGHT - LEFT) * DOTS).round() as u32;
    let high = ((TOP - BOTTOM) * DOTS).round() as u32;
    let mut data = drawn(says, wide, high, ink);
    let (mut wide_now, mut high_now, mut levels) = (wide, high, 1);
    while wide_now > 1 || high_now > 1 {
        (wide_now, high_now) = ((wide_now / 2).max(1), (high_now / 2).max(1));
        data.extend(drawn(says, wide_now, high_now, ink));
        levels += 1;
    }
    let rest = data.split_off((wide * high * 4) as usize);
    let mut image = Image::new(
        Extent3d {
            width: wide,
            height: high,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.mip_level_count = levels;
    if let Some(data) = image.data.as_mut() {
        data.extend(rest);
    }
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bubble_keeps_the_point_of_its_tail_where_it_is_whichever_way_the_camera_looks() {
        let tip = Vec3::new(3.0, 1.9, -7.0);
        // Where the tail's point is on the sheet, from its middle.
        let point = -middle_from_tip().extend(0.0);
        for eye in [Quat::IDENTITY, Quat::from_rotation_y(2.1), Quat::from_euler(EulerRot::YXZ, -0.7, -1.2, 0.0)] {
            for age in [0.0, POP_SECS * 0.5, POP_SECS, 9.0] {
                let place = stood(tip, eye, age);
                assert!(place.transform_point(point).distance(tip) < 1e-5, "{age}");
                // Square to the camera: its face along the way the camera
                // looks back, and upright as the camera is.
                assert!((place.rotation * Vec3::Z).dot(eye * Vec3::Z) > 0.9999);
                assert!((place.rotation * Vec3::Y).dot(eye * Vec3::Y) > 0.9999);
            }
        }
        // As big as it is, whatever the camera does: its circle the same
        // metres across once it has popped up.
        assert_eq!(stood(tip, Quat::IDENTITY, 9.0).scale, Vec3::ONE);
        assert_eq!(popped(POP_SECS), 1.0);
        assert!(popped(0.0) < 0.7 && popped(POP_SECS * 0.7) > 1.0);
        // And its circle's middle is over its point, up and to the right.
        assert!(circle_from_tip().x > 0.0 && circle_from_tip().y > RADIUS);
    }

    #[test]
    fn each_picture_is_a_white_bubble_with_its_mark_dark_in_it_at_every_size() {
        let (wide, high) = (((RIGHT - LEFT) * DOTS).round() as u32, ((TOP - BOTTOM) * DOTS).round() as u32);
        let at = |data: &[u8], x: f32, y: f32| {
            let column = (((x - LEFT) / (RIGHT - LEFT)) * wide as f32) as usize;
            let row = (((TOP - y) / (TOP - BOTTOM)) * high as f32) as usize;
            let dot = (row * wide as usize + column) * 4;
            (data[dot], data[dot + 3])
        };
        for says in Says::ALL {
            let data = drawn(says, wide, high, 0.0);
            // Clear outside it, dark on its outline, white inside it away
            // from the mark, and dark on the mark.
            assert_eq!(at(&data, 1.0, 1.0).1, 0, "{says:?}");
            assert_eq!(at(&data, 0.0, 0.96), (0, 255), "{says:?}");
            assert_eq!(at(&data, 0.0, 0.8), (255, 255), "{says:?}");
            let on_it = match says {
                Says::Dots => Vec2::ZERO,
                Says::Sit => Vec2::new(-0.13, 0.47),
                Says::Lie => Vec2::new(0.2, -0.02),
                Says::Store => Vec2::new(0.3, -0.2),
                Says::Light => Vec2::new(0.0, 0.2),
                Says::Water => Vec2::new(0.1, -0.2),
                Says::Piggy => Vec2::new(-0.1, -0.12),
            };
            assert_eq!(at(&data, on_it.x, on_it.y), (0, 255), "{says:?}");
            // Nothing of a mark reaches the outline.
            for step in 0..64 {
                let round = Vec2::from_angle(step as f32 * std::f32::consts::TAU / 64.0) * 0.86;
                assert!(mark_distance(says, round) > 0.0, "{says:?} at {round}");
            }
            // Every size, down to one dot, in one picture.
            let image = picture(says);
            let levels = image.texture_descriptor.mip_level_count;
            assert_eq!(levels, 32 - wide.max(high).leading_zeros());
            let mut dots = 0;
            let (mut w, mut h) = (wide, high);
            for _ in 0..levels {
                dots += w * h;
                (w, h) = ((w / 2).max(1), (h / 2).max(1));
            }
            assert_eq!(image.data.as_ref().map(Vec::len), Some(dots as usize * 4));
        }
    }
}
