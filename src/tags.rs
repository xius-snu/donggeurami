//! Names over heads: everyone else's while you are online, now that some of
//! them are people, and never your own (Hajun, 2026-10-31: "remove the in
//! world player name for the player's own character"; until then yours was
//! always over you, online or not: it is on its plate top left, `hud`). The
//! AIs this device runs in the town have one too online: there they are
//! players like anyone else.
//!
//! Each tag is a line of text laid out once, at the top left of the screen,
//! and moved over its body's head without being laid out again, the way the
//! bubble over whoever you can talk to is (`talk`); and scaled by how far
//! the head is from the camera, as words standing in the world would be
//! (Hajun, 2026-10-31: "like a in the world text rather than a on screen
//! text so further away the text look small naturally and closer look
//! larger"): its full size [`FULL_AT`] metres from the camera and nearer,
//! and smaller beyond by the same share as anything else is. It is put
//! away while its body is on another island, behind the camera, off the
//! screen, further off than [`READ_RANGE`] or under House Builder's
//! results ([`HidesNames`]).

use bevy::prelude::*;
use bevy_replicon::prelude::Remote;

use crate::builder::OnlinePlot;
use crate::hud;
use crate::island::Venue;
use crate::lobby::Townsfolk;
use crate::net::Online;
use crate::{Player, ThirdPersonCamera, WalkCycle};

/// How far off a name can still be read, in metres: 25 until 2026-10-31,
/// when Hajun asked for three times as far.
const READ_RANGE: f32 = 75.0;
/// How big a name is laid out, as a share of the short side of the screen:
/// as big as it is ever shown, so that it is only ever scaled down, which
/// keeps it sharp. 6 for an hour on 2026-10-31, until Hajun found the names
/// "too big, especially your own name", and asked for less than half: 2.8;
/// then, with their own gone, "slightly bigger", 3.2; then, 2026-11-01,
/// "slightly increase" again.
pub(crate) const TAG_TEXT: f32 = 3.6;
/// How far from the camera, in metres, a name is shown at the size it is
/// laid out: further off it is smaller by the same share, as the body under
/// it is. 8 m; 4 m was tried first, and a name 15 m off could not be read.
const FULL_AT: f32 = 8.0;
/// How high over a body's feet its name sits, in metres: just over its head,
/// which is 1.71 m up. 2.0, where the House Builder's bubble is, until
/// Hajun asked for "slightly lower, closer to the head" (2026-10-31).
const OVER_HEAD: f32 = 1.85;
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
pub(crate) struct Tagged;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (tag, rename, untag, place_tags)
            .chain()
            .after(crate::follow_camera),
    );
}

/// A name that has changed, as someone's does when they sign up (yours in
/// `signup`, anyone else's in `net::redress`), changes over their head.
fn rename(
    renamed: Query<(Entity, &Name), (With<Tagged>, Changed<Name>)>,
    mut tags: Query<(&Nametag, &mut Text)>,
) {
    for (body, name) in &renamed {
        for (tag, mut text) in &mut tags {
            if tag.0 == body && text.0 != name.as_str() {
                text.0 = name.to_string();
            }
        }
    }
}

/// A name for everyone else online: those the server sent, and the town's
/// AIs. Never for you.
fn tag(
    online: Res<Online>,
    bodies: Query<
        (Entity, &Name),
        (
            Without<Tagged>,
            Without<Player>,
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

/// Takes a name away with its body, and every one once you are offline.
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

/// How big a name is shown, as a share of the size it is laid out, with its
/// head `from` metres from the camera.
fn scale_at(from: f32) -> f32 {
    (FULL_AT / from.max(0.1)).min(1.0)
}

/// Keeps each name over its body's head, wherever the camera goes, scaled by
/// how far off it is, and puts it away when it could not be read, or would
/// be under something that hides names.
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
    let eye_at = eye.translation;
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
        let head = bodies
            .get(tag.0)
            .ok()
            .filter(|&(body, &venue)| {
                venue == here && body.translation.distance(you.translation) <= READ_RANGE
            })
            .map(|(body, _)| body.translation + Vec3::Y * OVER_HEAD);
        let point = head
            .and_then(|head| camera.world_to_viewport(&eye, head).ok())
            .filter(|point| {
                point.x >= 0.0
                    && point.y >= 0.0
                    && point.x <= window.width()
                    && point.y <= window.height()
            });
        // Scaled about its middle to the size it is seen at, and centred
        // over the head, standing on it.
        let size = node.size() * node.inverse_scale_factor();
        let scale = head.map_or(1.0, |head| scale_at(eye_at.distance(head)));
        let seen = size * scale;
        let Some(corner) = point
            .map(|point| Vec2::new(point.x - size.x * 0.5, point.y - seen.y * 0.5 - size.y * 0.5))
            .filter(|corner| corner.y + (size.y - seen.y) * 0.5 >= top)
            .filter(|&corner| {
                let name = Rect::from_center_size(corner + size * 0.5, seen);
                hidden.iter().all(|hider| hider.intersect(name).is_empty())
            })
        else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        place.set_if_neq(UiTransform {
            translation: Val2::px(corner.x, corner.y),
            scale: Vec2::splat(scale),
            rotation: Rot2::IDENTITY,
        });
        shown.set_if_neq(Visibility::Inherited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_full_size_near_and_smaller_by_distance_beyond() {
        assert_eq!(scale_at(FULL_AT), 1.0);
        assert_eq!(scale_at(1.0), 1.0, "never larger than laid out");
        assert!((scale_at(FULL_AT * 2.0) - 0.5).abs() < 1e-6);
        assert!((scale_at(FULL_AT * 10.0) - 0.1).abs() < 1e-6);
        // Three times as far as before 2026-10-31.
        assert_eq!(READ_RANGE, 75.0);
    }
}
