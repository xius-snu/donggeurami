//! A list that is flicked goes on by itself, as a list on the phone does.
//!
//! Hajun, 2026-11-22, of the bag's window: "right now, scrolling in the
//! shop/inventory is like as soon as u take ur finger out, it stops the
//! scroll. i want it so if its like a strong flick scroll then it shouldnt
//! stop scroll right away there should be some like velocity and friction
//! slowing it down ... if u scroll it like with ur finger pressed down or
//! sth it just stops right when u take ur finger out but if u like finger
//! flick upward to scroll down then it doesnt stop right away ... just do it
//! the way samsung settings page scrolls on mobile touch input."
//!
//! So this is Android's own arithmetic, the one every list on such a phone
//! goes by, with its own numbers:
//!
//! - **How fast the finger was going as it came up** ([`Track`]) is
//!   `VelocityTracker`'s: a curve fitted through where the finger was over
//!   its last tenth of a second, and how steep that is at its end. A finger
//!   that has been still for 40 ms was going nowhere: so a list dragged and
//!   held stops where it is let go.
//! - **How far and how long a list thrown that fast goes on** ([`Throw`])
//!   is `OverScroller`'s: no slower than 50 a second is a throw at all, none
//!   is faster than 8,000, and it runs out along the same curve, fast at
//!   first and a long while coming to rest.
//! - **Which way a press that moves is going** ([`way`]) is a list's and a
//!   `ViewPager`'s between them: nothing until it has gone further than a
//!   touch does, 8; then across, if it has gone twice as far across as up
//!   or down, and along the list otherwise.
//!
//! Every length here is one of Android's `dp`, 160 to the inch, which is
//! what a logical pixel of the window is on a phone (`keep_ui_size`): so a
//! flick of the same finger throws a list as far here as it does there.
//!
//! A list is thrown by giving it [`Coasting`], and [`coast`] carries it on
//! until it has run out or come to its end, where it stops (nothing
//! stretches or bounces there). Whoever owns the list takes `Coasting` off
//! it when a finger is put on it. The bag's window is the one that does
//! (`bag::panel`); the other lists stop where they are let go, as they did.

use std::collections::VecDeque;

use bevy::prelude::*;

/// How far a press has to go before it is a drag and no longer a touch:
/// `ViewConfiguration`'s touch slop.
pub(crate) const SLOP: f32 = 8.0;
/// The slowest a list is thrown at all, and the fastest, a second:
/// `ViewConfiguration`'s minimum and maximum fling velocity.
const SLOWEST: f32 = 50.0;
const FASTEST: f32 = 8000.0;
/// `OverScroller`'s: how much a list rubs, where its curve turns, and what
/// pulls it up: gravity (9.80665 m/s²) in inches (39.37 to the metre) at
/// 160 to the inch, by the 0.84 Android has there for the feel of it.
const FRICTION: f32 = 0.015;
const INFLEXION: f32 = 0.35;
const PULL: f32 = 9.80665 * 39.37 * 160.0 * 0.84;
/// `OverScroller`'s rate of slowing, `ln 0.78 / ln 0.9`.
const RATE: f32 = 2.358_201_8;
/// `VelocityTracker`'s: how far back it looks, in seconds, and how long
/// without moving is a finger that has stopped.
const HORIZON: f64 = 0.1;
const STOPPED: f64 = 0.04;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, coast);
}

/// Which way a press that has moved is going.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Way {
    /// Not far enough to say: a touch still.
    #[default]
    Unsure,
    /// Up or down, along a list.
    Along,
    /// Across, to the left or the right.
    Across,
}

/// Which way a press that has gone `gone` from where it went down is
/// going. Asked as it moves until it says one or the other, and that is
/// the way it is going from then on, wherever it goes after.
pub(crate) fn way(gone: Vec2) -> Way {
    let (across, along) = (gone.x.abs(), gone.y.abs());
    if across > SLOP && across * 0.5 > along {
        Way::Across
    } else if along > SLOP {
        Way::Along
    } else {
        Way::Unsure
    }
}

/// Where a press has been over its last tenth of a second, a place to a
/// frame: for how fast it was going as it came up.
#[derive(Default, Debug)]
pub(crate) struct Track(VecDeque<(f64, Vec2)>);

impl Track {
    /// A press has gone down, at `now` in seconds: nothing before it counts.
    pub(crate) fn begin(&mut self, now: f64) {
        self.0.clear();
        self.0.push_back((now, Vec2::ZERO));
    }

    /// The press is `to` from where it went down, at `now`; `frame` is how
    /// long a frame is taking. A second place in one frame is the frame's;
    /// and after a while of not moving, what came before is forgotten.
    pub(crate) fn moved(&mut self, now: f64, frame: f64, to: Vec2) {
        if let Some(&(last, _)) = self.0.back() {
            if now <= last {
                if let Some(place) = self.0.back_mut() {
                    place.1 = to;
                }
                return;
            }
            if now - last > still(frame) {
                self.0.clear();
            }
        }
        self.0.push_back((now, to));
        while self.0.front().is_some_and(|&(first, _)| now - first > reach(frame)) {
            self.0.pop_front();
        }
    }

    /// How fast the press is going at `now`, a second, across and down:
    /// nothing if it has not moved for a while.
    pub(crate) fn speed(&self, now: f64, frame: f64) -> Vec2 {
        let Some(&(newest, _)) = self.0.back() else {
            return Vec2::ZERO;
        };
        if now - newest > still(frame) {
            return Vec2::ZERO;
        }
        let recent = || self.0.iter().filter(move |&&(when, _)| newest - when <= reach(frame));
        let across: Vec<(f64, f64)> = recent().map(|&(when, at)| (when - newest, f64::from(at.x))).collect();
        let down: Vec<(f64, f64)> = recent().map(|&(when, at)| (when - newest, f64::from(at.y))).collect();
        Vec2::new(steepness(&across) as f32, steepness(&down) as f32)
    }
}

/// How long without moving is a finger that has stopped: Android's 40 ms,
/// which it counts between touches that come every few; here they come a
/// frame apart, so never less than two frames and a half.
fn still(frame: f64) -> f64 {
    STOPPED.max(2.5 * frame)
}

/// How far back a finger's places count: Android's tenth of a second, and
/// never less than the last three frames, should frames be that slow.
fn reach(frame: f64) -> f64 {
    HORIZON.max(3.5 * frame)
}

/// How steep the curve through `points` is at the last of them: each is
/// when, in seconds before the last, and where. With three or more, the
/// parabola that fits them best (`VelocityTracker`'s own, least squares of
/// the second degree, unweighted); with two, the line through them.
fn steepness(points: &[(f64, f64)]) -> f64 {
    let line = |points: &[(f64, f64)]| {
        let count = points.len() as f64;
        let (st, sp) = points.iter().fold((0.0, 0.0), |(st, sp), &(t, p)| (st + t, sp + p));
        let (stt, stp) = points.iter().fold((0.0, 0.0), |(stt, stp), &(t, p)| (stt + t * t, stp + t * p));
        let spread = stt - st * st / count;
        if spread > 0.0 { (stp - st * sp / count) / spread } else { 0.0 }
    };
    if points.len() < 3 {
        return if points.len() == 2 { line(points) } else { 0.0 };
    }
    let count = points.len() as f64;
    let (mut st, mut st2, mut st3, mut st4, mut sp, mut stp, mut st2p) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(t, p) in points {
        st += t;
        st2 += t * t;
        st3 += t * t * t;
        st4 += t * t * t * t;
        sp += p;
        stp += t * p;
        st2p += t * t * p;
    }
    let sxx = st2 - st * st / count;
    let sxy = stp - st * sp / count;
    let sxx2 = st3 - st * st2 / count;
    let sx2y = st2p - st2 * sp / count;
    let sx2x2 = st4 - st2 * st2 / count;
    let under = sxx * sx2x2 - sxx2 * sxx2;
    // Where the finger was at the last of them is where the times are
    // nothing, so the curve is as steep there as its middle term.
    if under > f64::EPSILON * sxx * sx2x2 { (sxy * sx2x2 - sx2y * sxx2) / under } else { line(points) }
}

/// A list thrown: which way, how far it goes before it stops, and how
/// long that takes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Throw {
    way: f32,
    far: f32,
    lasts: f32,
}

impl Throw {
    /// A list let go at `speed` a second, one way or the other: nothing,
    /// if that is slower than a throw is.
    pub(crate) fn at(speed: f32) -> Option<Self> {
        // (A speed that is no number is no throw either.)
        if speed.is_nan() || speed.abs() < SLOWEST {
            return None;
        }
        let fast = speed.abs().min(FASTEST);
        let log = (INFLEXION * fast / (FRICTION * PULL)).ln();
        Some(Self {
            way: speed.signum(),
            far: FRICTION * PULL * (RATE / (RATE - 1.0) * log).exp(),
            lasts: (log / (RATE - 1.0)).exp(),
        })
    }

    /// How far it has gone `secs` after it was let go.
    pub(crate) fn gone(&self, secs: f32) -> f32 {
        self.way * self.far * run_out((secs / self.lasts).clamp(0.0, 1.0))
    }

    /// Whether it has stopped by then.
    pub(crate) fn over(&self, secs: f32) -> bool {
        secs >= self.lasts
    }

    /// Which way it goes: 1 or -1.
    pub(crate) fn way(&self) -> f32 {
        self.way
    }
}

/// How much of its way a thrown list has gone when `time` of its time has
/// passed, each from nothing to 1: `OverScroller`'s spline, the curve from
/// (0, 0) to (1, 1) drawn toward (0.175, 0.5) and then (0.35, 1), which
/// leaves at the finger's own speed and comes in flat.
fn run_out(time: f32) -> f32 {
    const START: f32 = 0.5;
    const FIRST: f32 = START * INFLEXION;
    const SECOND: f32 = INFLEXION;
    if time <= 0.0 {
        return 0.0;
    }
    if time >= 1.0 {
        return 1.0;
    }
    let at = |s: f32| 3.0 * s * (1.0 - s) * ((1.0 - s) * FIRST + s * SECOND) + s * s * s;
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let middle = (low + high) * 0.5;
        if at(middle) > time {
            high = middle;
        } else {
            low = middle;
        }
    }
    let s = (low + high) * 0.5;
    3.0 * s * (1.0 - s) * ((1.0 - s) * START + s) + s * s * s
}

/// How far down a list can be scrolled: what it holds, less what of that
/// shows at once, in logical pixels.
pub(crate) fn room(node: &ComputedNode) -> f32 {
    ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0)
}

/// A list going on by itself: how it was thrown, when, and how far it has
/// been carried so far.
#[derive(Component, Debug)]
pub(crate) struct Coasting {
    throw: Throw,
    began: f64,
    gone: f32,
}

impl Coasting {
    /// Thrown at `now`, in seconds.
    pub(crate) fn new(throw: Throw, now: f64) -> Self {
        Self { throw, began: now, gone: 0.0 }
    }
}

/// Carries on every list that was thrown, by what its throw has gone since
/// the last frame, until the throw has run out or the list has come to its
/// end that way. Does nothing while none is thrown.
pub(crate) fn coast(time: Res<Time<Real>>, mut lists: Query<(Entity, &ComputedNode, &mut ScrollPosition, &mut Coasting)>, mut commands: Commands) {
    for (list, node, mut scroll, mut coasting) in &mut lists {
        let secs = (time.elapsed_secs_f64() - coasting.began) as f32;
        let gone = coasting.throw.gone(secs);
        let wanted = scroll.y + gone - coasting.gone;
        coasting.gone = gone;
        let most = room(node);
        let to = wanted.clamp(0.0, most);
        if scroll.y != to {
            scroll.y = to;
        }
        let at_its_end = if coasting.throw.way() > 0.0 { wanted >= most } else { wanted <= 0.0 };
        if at_its_end || coasting.throw.over(secs) {
            commands.entity(list).try_remove::<Coasting>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_throw_goes_as_far_and_as_long_as_android_s() {
        assert!((RATE - 0.78_f32.ln() / 0.9_f32.ln()).abs() < 1e-5);
        // Android's `Scroller`, let go at 4,000 dots a second on a screen
        // of three dots to the dp, goes 961 dots in 686 ms (the numbers
        // Flutter's own copy of this was fitted to): 1,333 a second, 320
        // and a third.
        let throw = Throw::at(4000.0 / 3.0).expect("a throw");
        assert!((throw.lasts - 0.686).abs() < 0.004, "{throw:?}");
        assert!((throw.far - 961.0 / 3.0).abs() < 2.0, "{throw:?}");
        // How far is 0.35 of how fast by how long, whatever the speed.
        for speed in [60.0, 500.0, 2000.0, 3000.0, 8000.0] {
            let throw = Throw::at(speed).expect("a throw");
            assert!((throw.far - INFLEXION * speed * throw.lasts).abs() < throw.far * 1e-3, "{speed}: {throw:?}");
        }
        // A flick, 3,000 a second: over 1,300, a second and a quarter.
        let flick = Throw::at(3000.0).expect("a throw");
        assert!((1250.0..1350.0).contains(&flick.far) && (1.2..1.3).contains(&flick.lasts), "{flick:?}");
        // A list barely moving as it is let go drifts a finger's breadth.
        let drift = Throw::at(200.0).expect("a throw");
        assert!((8.0..16.0).contains(&drift.far) && drift.lasts < 0.25, "{drift:?}");
        // Slower than 50 is no throw; faster than 8,000 is 8,000.
        assert_eq!(Throw::at(49.0), None);
        assert_eq!(Throw::at(-49.0), None);
        assert_eq!(Throw::at(0.0), None);
        assert_eq!(Throw::at(f32::NAN), None);
        assert_eq!(Throw::at(20_000.0), Throw::at(8000.0));
        // The other way is the same throw the other way.
        let (down, up) = (Throw::at(1500.0).expect("a throw"), Throw::at(-1500.0).expect("a throw"));
        assert_eq!((down.far, down.lasts), (up.far, up.lasts));
        assert_eq!(down.gone(0.3), -up.gone(0.3));
        assert!(down.gone(0.3) > 0.0);
    }

    #[test]
    fn a_thrown_list_leaves_at_the_finger_s_speed_and_comes_slowly_to_rest() {
        let throw = Throw::at(2400.0).expect("a throw");
        // It begins as fast as the finger was going: nothing jumps as the
        // finger comes off.
        let first = throw.gone(0.002) / 0.002;
        assert!((first - 2400.0).abs() < 2400.0 * 0.03, "{first}");
        // It only ever goes on, and slower all the time.
        let steps: Vec<f32> = (0..=200).map(|n| throw.gone(throw.lasts * n as f32 / 200.0)).collect();
        for three in steps.windows(3) {
            let (before, after) = (three[1] - three[0], three[2] - three[1]);
            assert!(before >= 0.0 && after >= 0.0, "{three:?}");
            assert!(after <= before + throw.far * 1e-4, "{three:?}");
        }
        // Two thirds of the way in the first third of its time, and the
        // last fifth of its time over the last fortieth of the way.
        assert!(throw.gone(throw.lasts / 3.0) > throw.far * 0.66);
        assert!(throw.gone(throw.lasts * 0.8) > throw.far * 0.975);
        // And there it stays.
        assert_eq!(throw.gone(throw.lasts), throw.far);
        assert_eq!(throw.gone(throw.lasts + 5.0), throw.far);
        assert_eq!(throw.gone(0.0), 0.0);
        assert!(!throw.over(throw.lasts * 0.99) && throw.over(throw.lasts));
        // The curve itself: Android's, through its two ends.
        assert_eq!((run_out(0.0), run_out(1.0)), (0.0, 1.0));
        assert!((run_out(0.35) - 0.72).abs() < 0.01, "{}", run_out(0.35));
    }

    /// A finger's places, a frame apart, from where it went down: `at`
    /// says where it is each second.
    fn tracked(frames: usize, frame: f64, at: impl Fn(f64) -> Vec2) -> (Track, f64) {
        let mut track = Track::default();
        track.begin(10.0);
        let mut now = 10.0;
        for n in 1..=frames {
            now = 10.0 + n as f64 * frame;
            track.moved(now, frame, at(n as f64 * frame));
        }
        (track, now)
    }

    #[test]
    fn how_fast_a_finger_was_going_is_how_fast_it_was_going_as_it_came_up() {
        let frame = 1.0 / 120.0;
        // Steadily: that speed, each way.
        let (track, now) = tracked(30, frame, |secs| Vec2::new(-300.0, 1800.0) * secs as f32);
        let speed = track.speed(now, frame);
        assert!((speed.x + 300.0).abs() < 1.0 && (speed.y - 1800.0).abs() < 1.0, "{speed}");
        // Quickening, as a flick does: how fast it was going at the last,
        // not over the whole of it. From nothing to 3,000 a second in a
        // tenth of a second, it was going 3,000, where its whole way over
        // its whole time is 1,500.
        let (track, now) = tracked(12, frame, |secs| Vec2::Y * (15_000.0 * secs * secs) as f32);
        let speed = track.speed(now, frame);
        assert!((speed.y - 3000.0).abs() < 30.0, "{speed}");
        // At sixty frames a second too.
        let (track, now) = tracked(6, 1.0 / 60.0, |secs| Vec2::Y * (15_000.0 * secs * secs) as f32);
        assert!((track.speed(now, 1.0 / 60.0).y - 3000.0).abs() < 30.0);
        // Two places make a line; one, or none, is going nowhere.
        let (track, now) = tracked(1, frame, |_| Vec2::Y * 10.0);
        assert!((track.speed(now, frame).y - 1200.0).abs() < 1.0);
        let (track, now) = tracked(0, frame, |_| Vec2::ZERO);
        assert_eq!(track.speed(now, frame), Vec2::ZERO);
        assert_eq!(Track::default().speed(3.0, frame), Vec2::ZERO);
    }

    #[test]
    fn a_finger_that_stopped_before_it_came_up_was_going_nowhere() {
        let frame = 1.0 / 120.0;
        // Dragged fast and then held: let go 50 ms after it last moved.
        let (track, now) = tracked(30, frame, |secs| Vec2::Y * 2000.0 * secs as f32);
        assert!(track.speed(now, frame).y > 1900.0);
        assert!(track.speed(now + 0.03, frame).y > 1900.0);
        assert_eq!(track.speed(now + 0.05, frame), Vec2::ZERO);
        // Held and then flicked: what it did before it was held is
        // forgotten, and the flick alone counts.
        let mut track = Track::default();
        track.begin(0.0);
        for n in 1..=10 {
            track.moved(n as f64 * frame, frame, Vec2::Y * -40.0 * n as f32);
        }
        let held = 10.0 * frame + 0.3;
        for n in 1..=6 {
            track.moved(held + n as f64 * frame, frame, Vec2::Y * (-400.0 + 12.0 * n as f32));
        }
        let speed = track.speed(held + 6.0 * frame, frame);
        assert!((speed.y - 1440.0).abs() < 2.0, "{speed}");
        // A slow frame is no stop: at ten frames a second, a tenth of a
        // second between two places is a finger still moving.
        let (track, now) = tracked(5, 0.1, |secs| Vec2::Y * 500.0 * secs as f32);
        assert!((track.speed(now, 0.1).y - 500.0).abs() < 1.0);
        // Two places in one frame are one, the later.
        let mut track = Track::default();
        track.begin(1.0);
        track.moved(1.0 + frame, frame, Vec2::Y * 5.0);
        track.moved(1.0 + frame, frame, Vec2::Y * 9.0);
        assert!((track.speed(1.0 + frame, frame).y - 9.0 / frame as f32).abs() < 1.0);
    }

    #[test]
    fn a_press_is_a_touch_until_it_has_gone_eight_and_then_goes_one_way() {
        // Not far: a touch still.
        assert_eq!(way(Vec2::ZERO), Way::Unsure);
        assert_eq!(way(Vec2::new(7.0, -7.0)), Way::Unsure);
        assert_eq!(way(Vec2::new(0.0, 8.0)), Way::Unsure);
        // Up or down: along the list.
        assert_eq!(way(Vec2::new(0.0, 8.5)), Way::Along);
        assert_eq!(way(Vec2::new(3.0, -20.0)), Way::Along);
        // Across, by twice as far as up or down.
        assert_eq!(way(Vec2::new(9.0, 0.0)), Way::Across);
        assert_eq!(way(Vec2::new(-30.0, 14.0)), Way::Across);
        // Slanting: the list's, once it has gone far enough down; nothing
        // until then.
        assert_eq!(way(Vec2::new(-30.0, 16.0)), Way::Along);
        assert_eq!(way(Vec2::new(12.0, 7.0)), Way::Unsure);
    }

    #[test]
    fn a_thrown_list_is_carried_on_to_its_end_and_no_further() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        let mut time = Time::<Real>::default();
        time.update();
        world.insert_resource(time);
        // A list that holds 500 and shows 200 of it: 300 to scroll.
        let node = ComputedNode {
            size: Vec2::new(100.0, 200.0),
            content_size: Vec2::new(100.0, 500.0),
            ..default()
        };
        assert_eq!(room(&node), 300.0);
        let throw = Throw::at(3000.0).expect("a throw");
        let now = world.resource::<Time<Real>>().elapsed_secs_f64();
        // Thrown a hundredth of a second ago: carried that far on.
        let list = world.spawn((node, ScrollPosition(Vec2::new(0.0, 40.0)), Coasting::new(throw, now - 0.01))).id();
        world.run_system_once(coast).expect("carried");
        let at = world.get::<ScrollPosition>(list).expect("a list").y;
        assert!((at - 40.0 - throw.gone(0.01)).abs() < 0.5, "{at}");
        assert!(world.get::<Coasting>(list).is_some());
        // And put somewhere else meanwhile, it goes on from there.
        world.get_mut::<ScrollPosition>(list).expect("a list").y = 10.0;
        world.run_system_once(coast).expect("carried");
        assert!(world.get::<ScrollPosition>(list).expect("a list").y - 10.0 < 1.0);
        // Thrown a second ago, it would be past its end: it is at its end,
        // and thrown no more.
        world.entity_mut(list).insert((ScrollPosition(Vec2::new(0.0, 40.0)), Coasting::new(throw, now - 1.0)));
        world.run_system_once(coast).expect("carried");
        assert_eq!(world.get::<ScrollPosition>(list).expect("a list").y, 300.0);
        assert!(world.get::<Coasting>(list).is_none());
        // The other way: to its top.
        let back = Throw::at(-3000.0).expect("a throw");
        world.entity_mut(list).insert((ScrollPosition(Vec2::new(0.0, 250.0)), Coasting::new(back, now - 1.0)));
        world.run_system_once(coast).expect("carried");
        assert_eq!(world.get::<ScrollPosition>(list).expect("a list").y, 0.0);
        assert!(world.get::<Coasting>(list).is_none());
        // A gentle one runs out short of the end and stops there.
        let gentle = Throw::at(300.0).expect("a throw");
        world.entity_mut(list).insert((ScrollPosition(Vec2::new(0.0, 100.0)), Coasting::new(gentle, now - 2.0)));
        world.run_system_once(coast).expect("carried");
        let at = world.get::<ScrollPosition>(list).expect("a list").y;
        assert!((at - 100.0 - gentle.far).abs() < 0.01 && at < 300.0, "{at}");
        assert!(world.get::<Coasting>(list).is_none());
    }
}
