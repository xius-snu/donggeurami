//! In a house in the town, the camera stays in the house with you (since
//! 2026-11-25; Hajun: "it doesnt rly make sense for your camera to be
//! outside the building when you are inside the house", and, of the ways
//! offered that day, "lets try the zoom indoors version"). It is Roblox's
//! own way with walls, which it calls Zoom (beside Invisicam, which fades
//! what is in the way, as `see_through` does), but in a house alone, and
//! against that house alone:
//!
//! - **Only how far out it stands changes, never which way it looks.** It
//!   comes in along its own line (`CameraLine`), looking as it would zoomed
//!   in to there, so that a swipe turns it as much as ever: a camera that
//!   slid along walls and climbed them was "glitchy" to Hajun (2026-09-27).
//! - **Only the house stops it**: each of its walls as the box it was
//!   modelled as, with no holes for its doors and windows, so that it never
//!   goes out through either; and its floors, ceilings, roof and stairs.
//!   Never furniture, nor a door, a window or a painting in a wall, nor
//!   anyone, nor anything outside: out of doors nothing moves it (Hajun,
//!   2026-10-09).
//! - **In at once, out slowly**: a wall come between, it is in front of it
//!   that frame, so that nothing outside is ever seen; with room again, it
//!   eases back out ([`OUT_SECS`]).
//! - **Your zoom is kept**: it is only held nearer than that, and goes back
//!   out to it once there is room, or once you are out of the house.
//! - **With no room for it behind you** (nearer than it ever stands,
//!   `CAMERA_DISTANCE_MIN`), **it is your eyes**, until there is a tenth
//!   more room than that (`SWAP_ZOOM`, as a zoom comes out of them).
//! - **Through a doorway it waits a moment on the side it was on**
//!   ([`WAIT`]): coming in, outside where it was, until there is room for
//!   it behind you in there, and then it is there; going out, in there
//!   where it was, until there is room behind you outside, and then it is
//!   there, and keeps off the house until it stands clear of it. With still
//!   no room once the moment is over, it is your eyes.
//!
//! Turned off ([`STAYS_IN`]), the camera is as it was before: where you
//! turned and zoomed it, in a house or out, and walls in the way hide you.

use bevy::prelude::*;

use crate::build::{Ghost, OnWall, Piece, Wall, YouAreIn};
use crate::island::{self, Faces, Venue};
use crate::{
    CAMERA_DISTANCE_MIN, CAMERA_RADIUS, CameraLine, HeldCamera, Lookout, OrbitCamera, Player, SWAP_ZOOM,
};

/// Whether the camera stays in the house you are in, in the town: on since
/// 2026-11-25, to be tried. Off, it is as it was until then.
pub(crate) const STAYS_IN: bool = true;

/// How quickly the camera goes back out once there is room again, in
/// seconds: as a zoom is measured, by how many times further out it is, it
/// goes most of the way (all but a third) in this long, and all but a
/// twentieth in three times as long.
const OUT_SECS: f32 = 0.1;

/// How long the camera waits on the far side of a doorway for room behind
/// you on yours, in seconds, before it is your eyes instead. Walked through,
/// a doorway leaves room in about a sixth of a second; stood in, it may not.
const WAIT: f32 = 0.4;

/// Where the camera is, as to the house you are in or were last in.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum Side {
    /// Out of every house, the camera where you zoomed it.
    #[default]
    Out,
    /// You just into `house`, the camera still outside it, since `since`.
    Coming { house: Entity, since: f32 },
    /// You and the camera in `house`.
    In(Entity),
    /// You just out of `house`, the camera still in it, since `since`.
    Going { house: Entity, since: f32 },
    /// You and the camera out of `house`, the camera kept off it until it
    /// stands clear of it.
    Clear(Entity),
}

/// How the camera is kept this frame (`follow_camera`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Keep {
    /// Where you turned and zoomed it.
    Free,
    /// No further out along its line than this, in metres.
    Within(f32),
    /// In your eyes.
    Eyes,
}

/// The camera in a house: where it is, and how far out it may stand.
#[derive(Resource, Default, Debug)]
pub(crate) struct Indoors {
    side: Side,
    /// How far out along its line it may stand just now, in metres; `None`,
    /// as far as it is zoomed.
    reach: Option<f32>,
    /// No room for it behind you: it is your eyes.
    eyes: bool,
}

/// What one frame of it goes by.
#[derive(Clone, Copy, Debug)]
struct Frame {
    /// The house you stand in, if you do.
    inside: Option<Entity>,
    /// How far out along its line your zoom puts the camera; how far out it
    /// is at the nearest it ever stands behind you; and how far it needs
    /// room for to come out of your eyes, a tenth further.
    out: f32,
    nearest: f32,
    room: f32,
    /// The time now, and how long the frame was, in seconds.
    now: f32,
    dt: f32,
}

impl Indoors {
    /// How the camera is to be kept.
    pub(crate) fn keep(&self) -> Keep {
        match self.side {
            Side::Out | Side::Coming { .. } => Keep::Free,
            _ if self.eyes => Keep::Eyes,
            _ => self.reach.map_or(Keep::Free, Keep::Within),
        }
    }

    /// As it is anywhere a house keeps nothing.
    fn let_go(&mut self) {
        *self = Self::default();
    }

    /// A frame. `stop` says how far out along its line the camera gets
    /// before it meets a house, measured as far as the further of `out` and
    /// `room`: `None` if it does not meet it.
    fn step(&mut self, at: &Frame, mut stop: impl FnMut(Entity) -> Option<f32>) {
        let room = |met: Option<f32>| met.is_none_or(|met| met >= at.room);
        // Where you are says which side it is on before anything is measured.
        self.side = match (self.side, at.inside) {
            (Side::Out | Side::Clear(_), Some(house)) => Side::Coming { house, since: at.now },
            (Side::Coming { house, .. } | Side::In(house) | Side::Going { house, .. }, Some(other)) if other != house => {
                Side::Coming { house: other, since: at.now }
            }
            // Back out before it came in: it never went anywhere.
            (Side::Coming { .. }, None) => Side::Out,
            // Out of the house: in there it stays, where it was, a moment.
            (Side::In(house), None) => {
                self.reach = Some(self.reach.map_or(at.out, |reach| reach.min(at.out)));
                Side::Going { house, since: at.now }
            }
            // Back in before it went out: it is in there still.
            (Side::Going { house, .. }, Some(_)) => Side::In(house),
            (side, _) => side,
        };
        match self.side {
            Side::Out => {
                self.reach = None;
                self.eyes = false;
            }
            Side::Coming { house, since } => {
                let met = stop(house);
                if room(met) {
                    // Room behind you in there: it is there at once.
                    self.side = Side::In(house);
                    self.eyes = false;
                    self.reach = met.filter(|&met| met < at.out);
                } else if at.now - since >= WAIT {
                    self.side = Side::In(house);
                    self.eyes = true;
                } else {
                    self.reach = None;
                    self.eyes = false;
                }
            }
            Side::In(house) => self.keep_off(stop(house), at),
            Side::Going { house, since } => {
                let met = stop(house);
                if room(met) {
                    // Room behind you out here: it is here at once.
                    self.side = Side::Clear(house);
                    self.eyes = false;
                    self.reach = met.filter(|&met| met < at.out);
                } else if at.now - since >= WAIT {
                    self.side = Side::Clear(house);
                    self.eyes = true;
                }
            }
            Side::Clear(house) => {
                let met = stop(house);
                self.keep_off(met, at);
                if met.is_none() && self.reach.is_none() && !self.eyes {
                    self.side = Side::Out;
                }
            }
        }
    }

    /// Keeps the camera off the house it meets `met` out along its line, if
    /// it does: in front of it at once, and back out slowly once there is
    /// room; in your eyes with no room for it behind you.
    fn keep_off(&mut self, met: Option<f32>, at: &Frame) {
        if self.eyes {
            if met.is_none_or(|met| met >= at.room) {
                // Out of your eyes as a zoom comes out of them: as near as
                // it stands, and on out from there.
                self.eyes = false;
                self.reach = Some(at.nearest.min(at.out));
            }
        } else if met.is_some_and(|met| met < at.nearest) {
            self.eyes = true;
        }
        let wanted = met.map_or(at.out, |met| met.min(at.out));
        self.reach = match self.reach {
            None => (wanted < at.out).then_some(wanted),
            Some(reach) if wanted <= reach => Some(wanted),
            Some(reach) => {
                // Within a millimetre it is there, and stops: eased for ever,
                // it would move the camera by less every frame.
                let eased = ease_out(reach, wanted, at.dt);
                let eased = if wanted - eased < 0.001 { wanted } else { eased };
                (eased < at.out).then_some(eased)
            }
        };
    }
}

/// From `from` out toward `to`, further out, after `dt` seconds, eased as a
/// zoom is, by how many times further out ([`OUT_SECS`]): never past it.
fn ease_out(from: f32, to: f32, dt: f32) -> f32 {
    let (from, to) = (from.max(0.05), to.max(0.05));
    let kept = (-dt / OUT_SECS).exp();
    (to.ln() + (from.ln() - to.ln()) * kept).exp().min(to)
}

/// What a house is made of, as the camera stands clear of it.
type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static Mesh3d>,
        &'static GlobalTransform,
        Option<&'static Wall>,
        Has<Piece>,
        Has<OnWall>,
    ),
>;

/// Says, for the frame, how far out the camera may stand: in a house in the
/// town, kept in it with you; anywhere else, where you turned and zoomed it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn mind(
    time: Res<Time>,
    orbit: Res<OrbitCamera>,
    (lookout, held): (Res<Lookout>, Res<HeldCamera>),
    me: Query<(&Transform, &Venue), With<Player>>,
    house_you_are_in: Query<Entity, (With<YouAreIn>, Without<Ghost>)>,
    (places, children, parts): (Query<&GlobalTransform>, Query<&Children>, Parts),
    meshes: Res<Assets<Mesh>>,
    mut shell: Local<Option<(Entity, usize, Faces)>>,
    mut indoors: ResMut<Indoors>,
) {
    let Ok((you, &venue)) = me.single() else {
        indoors.let_go();
        return;
    };
    // Only in the town; never over you, in the car, nor in a game, nor
    // looking on from where someone else stands.
    if !STAYS_IN || venue != Venue::Town || orbit.overhead || orbit.locked || lookout.0.is_some() || held.0.is_some() {
        indoors.let_go();
        return;
    }
    let inside = house_you_are_in.iter().next();
    // In your eyes by your own zoom: nothing to keep, in a house or out.
    if orbit.distance <= 0.0 {
        *indoors = Indoors {
            side: inside.map_or(Side::Out, Side::In),
            ..default()
        };
        return;
    }
    let line = CameraLine::of(&orbit, you);
    let frame = Frame {
        inside,
        out: line.out(orbit.distance),
        nearest: line.out(CAMERA_DISTANCE_MIN),
        room: line.out(CAMERA_DISTANCE_MIN * SWAP_ZOOM),
        now: time.elapsed_secs(),
        dt: time.delta_secs(),
    };
    let probe = frame.out.max(frame.room);
    indoors.step(&frame, |house| {
        let place = places.get(house).ok()?;
        // Read once for each house you go into, and again should anything
        // be added to it or taken from it meanwhile.
        let count = children.get(house).map_or(0, |below| below.len());
        let read = matches!(&*shell, Some((of, was, _)) if *of == house && *was == count);
        if !read {
            *shell = Some((house, count, shell_of(house, place, &children, &parts, &meshes)?));
        }
        let Some((_, _, faces)) = &*shell else {
            return None;
        };
        stop_at(faces, place, line.from, line.way, probe)
    });
}

/// How far a ball as big as the camera gets from `from` along `way`, a
/// metre long, before it meets `shell`, the shape of a house standing at
/// `place`, if it does within `probe`.
fn stop_at(shell: &Faces, place: &GlobalTransform, from: Vec3, way: Vec3, probe: f32) -> Option<f32> {
    let into = place.affine().inverse();
    let (start, end) = (into.transform_point3(from), into.transform_point3(from + way * probe));
    // A metre in the world, in the house's own space.
    let metre = start.distance(end) / probe;
    if metre <= 1e-6 {
        return None;
    }
    shell.sweep(start, end, CAMERA_RADIUS * metre).map(|met| met / metre)
}

/// The shape the camera stands clear of in `house`, standing at `place`, in
/// the house's own space: each of its walls as the box it was modelled as,
/// with no holes for its doors and windows, and every other part of it, its
/// floors, ceilings, roof and stairs, as it is. Not what is in its walls, a
/// door, a window or a painting, which are pieces of their own. `None`
/// until every part of it has loaded.
fn shell_of(
    house: Entity,
    place: &GlobalTransform,
    children: &Query<&Children>,
    parts: &Parts,
    meshes: &Assets<Mesh>,
) -> Option<Faces> {
    let into = place.affine().inverse();
    let mut corners: Vec<[Vec3; 3]> = Vec::new();
    let mut next: Vec<Entity> = Vec::new();
    if let Ok(below) = children.get(house) {
        next.extend(below);
    }
    while let Some(part) = next.pop() {
        let Ok((mesh, part_place, wall, piece, on_wall)) = parts.get(part) else {
            continue;
        };
        if piece || on_wall {
            continue;
        }
        if let Some(wall) = wall {
            // The whole of it, and so whatever of it is drawn by parts of
            // its own under it (`build::Band`, the bands of a wall that runs
            // up past a floor).
            let (low, high) = wall.whole();
            corners.extend(box_corners(low, high));
            continue;
        } else if let Some(mesh) = mesh {
            let local = GlobalTransform::from(into * part_place.affine());
            island::add_mesh(&mut corners, meshes.get(&mesh.0)?, &local);
        }
        if let Ok(below) = children.get(part) {
            next.extend(below);
        }
    }
    (!corners.is_empty()).then(|| Faces::of(&corners))
}

/// The twelve triangles of the box from `low` to `high`. Which way each
/// faces does not matter: either side of a face stops a ball (`Faces`).
fn box_corners(low: Vec3, high: Vec3) -> [[Vec3; 3]; 12] {
    let at = |x: bool, y: bool, z: bool| {
        Vec3::new(
            if x { high.x } else { low.x },
            if y { high.y } else { low.y },
            if z { high.z } else { low.z },
        )
    };
    let side = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| [[a, b, c], [a, c, d]];
    let (f, t) = (false, true);
    let sides = [
        side(at(f, f, f), at(f, t, f), at(f, t, t), at(f, f, t)),
        side(at(t, f, f), at(t, t, f), at(t, t, t), at(t, f, t)),
        side(at(f, f, f), at(t, f, f), at(t, f, t), at(f, f, t)),
        side(at(f, t, f), at(t, t, f), at(t, t, t), at(f, t, t)),
        side(at(f, f, f), at(t, f, f), at(t, t, f), at(f, t, f)),
        side(at(f, f, t), at(t, f, t), at(t, t, t), at(f, t, t)),
    ];
    let mut corners = [[Vec3::ZERO; 3]; 12];
    for (index, triangle) in sides.into_iter().flatten().enumerate() {
        corners[index] = triangle;
    }
    corners
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    /// A frame with your zoom 10 m out, the nearest 0.85 and room at a tenth
    /// further, as the camera's line is looking level or down at you.
    fn frame(inside: Option<Entity>, now: f32) -> Frame {
        Frame {
            inside,
            out: 10.0,
            nearest: CAMERA_DISTANCE_MIN,
            room: CAMERA_DISTANCE_MIN * SWAP_ZOOM,
            now,
            dt: DT,
        }
    }

    fn a_house() -> Entity {
        World::new().spawn_empty().id()
    }

    #[test]
    fn in_a_house_it_comes_in_at_once_and_goes_back_out_slowly() {
        let house = a_house();
        let mut indoors = Indoors::default();
        // Out of doors, nothing keeps it.
        indoors.step(&frame(None, 0.0), |_| Some(2.0));
        assert_eq!(indoors.keep(), Keep::Free);
        // Into a house with room in it: it is in there with you that frame,
        // in front of the wall 3 m behind you.
        indoors.step(&frame(Some(house), 0.0), |_| Some(3.0));
        assert_eq!(indoors.keep(), Keep::Within(3.0));
        // Turned toward a nearer wall: in front of it that frame.
        indoors.step(&frame(Some(house), DT), |_| Some(1.5));
        assert_eq!(indoors.keep(), Keep::Within(1.5));
        // Turned back: out again slowly, never past the wall, and most of
        // the way by three tenths of a second.
        let mut reach = 1.5;
        let mut now = DT;
        while now < 0.3 {
            now += DT;
            indoors.step(&frame(Some(house), now), |_| Some(4.0));
            let Keep::Within(next) = indoors.keep() else {
                panic!("{:?}", indoors.keep());
            };
            assert!(next > reach && next <= 4.0, "{reach} to {next}");
            reach = next;
        }
        assert!(reach > 3.7, "{reach}");
        // Once there it is there exactly, and does not move at all.
        for _ in 0..120 {
            now += DT;
            indoors.step(&frame(Some(house), now), |_| Some(4.0));
        }
        assert_eq!(indoors.keep(), Keep::Within(4.0));
        // Nothing in the way as far out as you zoomed: out to that, and then
        // nothing keeps it.
        for _ in 0..240 {
            now += DT;
            indoors.step(&frame(Some(house), now), |_| None);
        }
        assert_eq!(indoors.keep(), Keep::Free);
    }

    #[test]
    fn with_no_room_behind_you_it_is_your_eyes_until_there_is_a_tenth_more() {
        let house = a_house();
        let mut indoors = Indoors::default();
        indoors.step(&frame(Some(house), 0.0), |_| Some(3.0));
        // Backed up to a wall: your eyes.
        indoors.step(&frame(Some(house), DT), |_| Some(0.5));
        assert_eq!(indoors.keep(), Keep::Eyes);
        // As much room as the nearest it stands, and no more: still.
        indoors.step(&frame(Some(house), 2.0 * DT), |_| Some(CAMERA_DISTANCE_MIN * 1.05));
        assert_eq!(indoors.keep(), Keep::Eyes);
        // A tenth more: out of them at the nearest, and on out from there.
        indoors.step(&frame(Some(house), 3.0 * DT), |_| Some(2.0));
        let Keep::Within(reach) = indoors.keep() else {
            panic!("{:?}", indoors.keep());
        };
        assert!(reach > CAMERA_DISTANCE_MIN && reach < 1.0, "{reach}");
    }

    #[test]
    fn through_a_doorway_it_waits_on_its_side_for_room_on_yours() {
        let house = a_house();
        let mut indoors = Indoors::default();
        // Just in, the wall right behind you: it stays out where it was.
        indoors.step(&frame(Some(house), 0.0), |_| Some(0.2));
        assert_eq!(indoors.keep(), Keep::Free);
        indoors.step(&frame(Some(house), 0.1), |_| Some(0.6));
        assert_eq!(indoors.keep(), Keep::Free);
        // A metre in, room behind you: in there at once.
        indoors.step(&frame(Some(house), 0.15), |_| Some(1.0));
        assert_eq!(indoors.keep(), Keep::Within(1.0));
        // Out again, the wall right behind you outside: it stays in where it
        // was...
        indoors.step(&frame(None, 1.0), |_| Some(0.2));
        assert_eq!(indoors.keep(), Keep::Within(1.0));
        // ...until there is room out here, and then keeps off the house...
        indoors.step(&frame(None, 1.1), |_| Some(1.2));
        assert_eq!(indoors.keep(), Keep::Within(1.2));
        assert_eq!(indoors.side, Side::Clear(house));
        // ...until it stands clear of it, and nothing keeps it.
        let mut now = 1.1;
        for _ in 0..240 {
            now += DT;
            indoors.step(&frame(None, now), |_| None);
        }
        assert_eq!(indoors.keep(), Keep::Free);
        assert_eq!(indoors.side, Side::Out);
        // Stood in the doorway with no room behind you: your eyes, once it
        // has waited.
        indoors.step(&frame(Some(house), 10.0), |_| Some(0.2));
        assert_eq!(indoors.keep(), Keep::Free);
        indoors.step(&frame(Some(house), 10.0 + WAIT * 1.1), |_| Some(0.2));
        assert_eq!(indoors.keep(), Keep::Eyes);
        // And back out before it came out: in your eyes still, outside, until
        // there is room out there.
        indoors.step(&frame(None, 11.0), |_| Some(0.3));
        assert_eq!(indoors.keep(), Keep::Eyes);
        indoors.step(&frame(None, 11.1), |_| Some(5.0));
        assert_eq!(indoors.keep(), Keep::Within(5.0));
    }

    #[test]
    fn back_in_before_it_went_out_it_is_in_there_still() {
        let house = a_house();
        let mut indoors = Indoors::default();
        indoors.step(&frame(Some(house), 0.0), |_| Some(3.0));
        indoors.step(&frame(None, 0.1), |_| Some(0.2));
        assert_eq!(indoors.side, Side::Going { house, since: 0.1 });
        indoors.step(&frame(Some(house), 0.2), |_| Some(2.5));
        assert_eq!(indoors.side, Side::In(house));
        assert_eq!(indoors.keep(), Keep::Within(2.5));
    }

    #[test]
    fn a_ball_as_big_as_the_camera_stops_short_of_a_wall_and_a_ceiling() {
        // A room 12 m square and 5 m high, walls 0.2 m thick, turned and
        // standing somewhere in the town: from 1.2 m over its middle, out
        // level toward a wall and straight up.
        let mut corners = Vec::new();
        for (low, high) in [
            (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 5.0, -5.8)),
            (Vec3::new(-6.0, 0.0, 5.8), Vec3::new(6.0, 5.0, 6.0)),
            (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(-5.8, 5.0, 6.0)),
            (Vec3::new(5.8, 0.0, -6.0), Vec3::new(6.0, 5.0, 6.0)),
            (Vec3::new(-6.0, 5.0, -6.0), Vec3::new(6.0, 5.2, 6.0)),
        ] {
            corners.extend(box_corners(low, high));
        }
        let shell = Faces::of(&corners);
        let place = GlobalTransform::from(
            Transform::from_xyz(40.0, 0.25, -12.0).with_rotation(Quat::from_rotation_y(0.7)),
        );
        let middle = place.transform_point(Vec3::new(0.0, 1.2, 0.0));
        let toward_the_wall = place.rotation() * Vec3::Z;
        let met = stop_at(&shell, &place, middle, toward_the_wall, 18.0).expect("it meets the wall");
        assert!((met - (5.8 - CAMERA_RADIUS)).abs() < 1e-3, "{met}");
        let met = stop_at(&shell, &place, middle, Vec3::Y, 18.0).expect("it meets the ceiling");
        assert!((met - (5.0 - 1.2 - CAMERA_RADIUS)).abs() < 1e-3, "{met}");
        // Nearer than that, it is not met.
        assert_eq!(stop_at(&shell, &place, middle, toward_the_wall, 5.0), None);
    }
}
