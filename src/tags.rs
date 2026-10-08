//! Names over everyone else's heads while you are online: who is who, now
//! that some of them are people.
//!
//! Each tag is a line of text laid out once, at the top left of the screen,
//! and moved over its body's head without being laid out again, the way the
//! House Builder's bubble is (`builder::place_bubble`). It is put away while
//! its body is on another island, behind the camera, off the screen, further
//! off than [`READ_RANGE`] or under House Builder's results ([`HidesNames`]).
//! Offline the town is as it always was, and nobody has one.
//!
//! The AIs this device runs in the town have one too: online they are
//! players like anyone else.

use bevy::prelude::*;
use bevy_replicon::prelude::Remote;

use crate::builder::OnlinePlot;
use crate::hud;
use crate::island::Venue;
use crate::lobby::Townsfolk;
use crate::net::Online;
use crate::{Player, ThirdPersonCamera, WalkCycle};

/// How far off a name can still be read, in metres.
const READ_RANGE: f32 = 25.0;
/// How big a name is, as a share of the short side of the screen.
const TAG_TEXT: f32 = 3.2;
/// How high over a body's feet its name sits, in metres: a little over its
/// head, as the House Builder's bubble is.
const OVER_HEAD: f32 = 2.0;
/// How far down the screen a name has to be to be shown in a game of House
/// Builder, as a share of its short side: below the time and what it is
/// for, as the House Builder's bubble is. Over a house, the names of those
/// behind it came up among them. In the town nothing is up there to be in
/// the way of.
const BELOW_THE_TOP: f32 = 18.0;
/// Names are under everything else on the screen.
const TAG_LAYER: i32 = -1;

/// A name on the screen, and the body it is over.
#[derive(Component)]
struct Nametag(Entity);

/// Something on the screen that no name is shown under: House Builder's
/// results, which a name behind would show through, among their own names.
#[derive(Component)]
pub(crate) struct HidesNames;

/// A body with its name over it.
#[derive(Component)]
struct Tagged;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (tag, untag, place_tags)
            .chain()
            .after(crate::follow_camera),
    );
}

/// A name for everyone else, online: those the server sent, and the town's
/// AIs.
fn tag(
    online: Res<Online>,
    bodies: Query<
        (Entity, &Name),
        (
            Without<Tagged>,
            With<WalkCycle>,
            Or<(With<Remote>, With<Townsfolk>)>,
        ),
    >,
    mut commands: Commands,
) {
    if !online.is() {
        return;
    }
    for (body, name) in &bodies {
        commands.entity(body).insert(Tagged);
        commands.spawn((
            Nametag(body),
            hud::words(name.as_str(), TAG_TEXT, Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                ..default()
            },
            UiTransform::default(),
            GlobalZIndex(TAG_LAYER),
            Visibility::Hidden,
        ));
    }
}

/// Takes a name away with its body, and every name once you are offline.
fn untag(
    online: Res<Online>,
    tags: Query<(Entity, &Nametag)>,
    bodies: Query<(), With<Tagged>>,
    mut commands: Commands,
) {
    for (tag, &Nametag(body)) in &tags {
        let gone = !bodies.contains(body);
        if gone || !online.is() {
            commands.entity(tag).despawn();
            if !gone {
                commands.entity(body).remove::<Tagged>();
            }
        }
    }
}

/// Keeps each name over its body's head, wherever the camera goes, and puts it
/// away when it could not be read, or would be under something that hides
/// names.
fn place_tags(
    me: Query<(&Transform, &Venue), With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    bodies: Query<(&Transform, &Venue), With<Tagged>>,
    windows: Query<&Window>,
    in_a_game: Res<OnlinePlot>,
    hiding: Query<(&ComputedNode, &UiGlobalTransform, &InheritedVisibility), With<HidesNames>>,
    mut tags: Query<(&Nametag, &mut Visibility, &mut UiTransform, &ComputedNode)>,
) {
    let (Ok((you, &here)), Ok((camera, eye)), Ok(window)) =
        (me.single(), cameras.single(), windows.single())
    else {
        return;
    };
    // The camera has no parent, so its transform is its global one, and
    // `follow_camera` has just set it for this frame.
    let eye = GlobalTransform::from(*eye);
    let top = if in_a_game.0.is_some() {
        window.width().min(window.height()) * BELOW_THE_TOP / 100.0
    } else {
        0.0
    };
    // Where they are, in the window's logical pixels, as names are placed.
    let hidden: Vec<Rect> = hiding
        .iter()
        .filter(|(.., shown)| shown.get())
        .map(|(node, place, _)| {
            let scale = node.inverse_scale_factor();
            Rect::from_center_size(place.translation * scale, node.size() * scale)
        })
        .collect();
    for (tag, mut shown, mut place, node) in &mut tags {
        let point = bodies
            .get(tag.0)
            .ok()
            .filter(|&(body, &venue)| {
                venue == here && body.translation.distance(you.translation) <= READ_RANGE
            })
            .and_then(|(body, _)| {
                camera
                    .world_to_viewport(&eye, body.translation + Vec3::Y * OVER_HEAD)
                    .ok()
            })
            .filter(|point| {
                point.x >= 0.0
                    && point.y >= 0.0
                    && point.x <= window.width()
                    && point.y <= window.height()
            });
        // Centred over the head, standing on it.
        let size = node.size() * node.inverse_scale_factor();
        let Some(corner) = point
            .map(|point| Vec2::new(point.x - size.x * 0.5, point.y - size.y))
            .filter(|corner| corner.y >= top)
            .filter(|&corner| {
                let name = Rect::from_corners(corner, corner + size);
                hidden.iter().all(|hider| hider.intersect(name).is_empty())
            })
        else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        place.set_if_neq(UiTransform::from_translation(Val2::px(corner.x, corner.y)));
        shown.set_if_neq(Visibility::Inherited);
    }
}
