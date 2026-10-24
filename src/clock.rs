//! The town's clock, which everything in the town that goes by the clock goes
//! by: its cars round the roads (`car`), the fountain's jet (`fountain`) and
//! the day and the night (`sky`). Each of them is worked out from the time
//! alone, so that every device in the town shows it alike with nothing about
//! it sent over the network: only the time has to agree.
//!
//! Offline it is this device's own time of day. But devices keep their time
//! of day to the world's only roughly: the PC this was made on was 0.67 s
//! out. On two phones in the town together the cars were in different
//! places, and a rider, drawn on the car as each phone had it, was somewhere
//! else on each (Hajun, 2026-10-24). So online it is the server's.
//! The device asks the server what time it is a few times as it comes online,
//! and once a minute after that (`net`), and goes by the answer that came
//! back quickest, put forward by half the time it took to come: the way there
//! and the way back can differ by no more than all of it, so it is out by
//! less than that half, a few milliseconds. Between answers it counts the
//! time on its own steady clock, which nothing puts forward or back, as the
//! time of day can be. That is one number, how far the server's time is ahead
//! of the steady clock, and for each answer two small messages, one each way.
//!
//! The steady clock stops while a phone sleeps. Going to sleep, the town's
//! clock goes back to the time of day, put forward by as much as the server's
//! time was ahead of it then, until the server is heard again on waking.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

/// How fast what the clock was set by is taken to drift out as it gets older,
/// in seconds a second: as fast as a phone's own clock runs fast or slow, and
/// then some. An answer that took longer than the one the clock was set by,
/// and is less sure, sets it once the older one could have drifted further
/// out than that.
const DRIFT: f64 = 2e-5;

/// The longest an answer can take to come back and still set the clock, in
/// seconds.
const LONGEST: f64 = 2.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<TownClock>();
}

/// The town's clock: the server's time online, this device's own offline.
#[derive(Resource)]
pub(crate) struct TownClock {
    /// When this device's steady clock was started, which [`Self::steady`]
    /// counts from.
    start: Instant,
    /// What it was last set by, from the server's answers.
    set: Option<Setting>,
    /// How far the server's time was ahead of this device's time of day when
    /// the app last went to sleep: what the time of day is put forward by
    /// until the server is heard again.
    slept: f64,
    /// How far this device's own time of day is put forward, in seconds, from
    /// `RT_CLOCK_OFF` on desktop: for trying the town's clock out with two
    /// copies of the game on one machine, whose time of day is the same.
    off: f64,
}

/// What the town's clock is set by.
#[derive(Clone, Copy, Debug)]
struct Setting {
    /// How far the server's time is ahead of the steady clock, in seconds.
    ahead: f64,
    /// How far out that can have been when it was set, in seconds: half the
    /// time the answer took to come back.
    doubt: f64,
    /// When it was set, by the steady clock.
    at: f64,
}

impl Default for TownClock {
    fn default() -> Self {
        let desktop = cfg!(not(any(target_os = "android", target_os = "ios")));
        Self {
            start: Instant::now(),
            set: None,
            slept: 0.0,
            off: std::env::var("RT_CLOCK_OFF")
                .ok()
                .filter(|_| desktop)
                .and_then(|said| said.trim().parse::<f64>().ok())
                .filter(|off| off.is_finite())
                .unwrap_or(0.0),
        }
    }
}

impl TownClock {
    /// The time by the town's clock, in seconds since 1970.
    pub(crate) fn now(&self) -> f64 {
        match self.set {
            Some(set) => self.steady() + set.ahead,
            None => self.time_of_day() + self.slept,
        }
    }

    /// The time on this device's steady clock, in seconds since the app
    /// started: what a question of the time is asked at, and its answer heard.
    pub(crate) fn steady(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// This device's own time of day, in seconds since 1970.
    fn time_of_day(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |since| since.as_secs_f64())
            + self.off
    }

    /// The server said it was `now`, in answer to a question asked at `asked`
    /// on the steady clock, and the answer was heard at `heard`. Sets the
    /// clock by it if it is surer than what the clock was set by: if it took
    /// no longer to come back than that could have drifted out since. Whether
    /// it did.
    pub(crate) fn heard(&mut self, asked: f64, now: f64, heard: f64) -> bool {
        let took = heard - asked;
        if !(0.0..=LONGEST).contains(&took) || !now.is_finite() {
            return false;
        }
        let doubt = took * 0.5;
        let first = self.set.is_none();
        let surer = self
            .set
            .is_none_or(|set| doubt <= set.doubt + (heard - set.at).max(0.0) * DRIFT);
        if !surer {
            return false;
        }
        self.set = Some(Setting {
            ahead: now + doubt - heard,
            doubt,
            at: heard,
        });
        if first {
            info!(
                "the town's clock is the server's: {:+.3} s on from this device's time of day, \
                 to within {:.0} ms",
                self.now() - self.time_of_day(),
                doubt * 1000.0
            );
        }
        true
    }

    /// The app is going to sleep, and its steady clock may stop until it
    /// wakes: from now until the server answers again, the town's clock is the
    /// time of day, put forward by as much as the server's time is ahead of it
    /// now.
    pub(crate) fn sleep(&mut self) {
        if self.set.is_some() {
            self.slept = self.now() - self.time_of_day();
            self.set = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The server's time on a clock that is `ahead` of the steady clock, at
    /// the steady clock's `at`.
    fn server(ahead: f64, at: f64) -> f64 {
        ahead + at
    }

    fn ahead(clock: &TownClock) -> f64 {
        clock.set.expect("set").ahead
    }

    #[test]
    fn an_answer_puts_the_clock_where_the_server_is() {
        // The server 1.79e9 s ahead of the steady clock. Asked at 10 s, and
        // answered 0.04 s later, the server telling the time halfway there:
        // to the microsecond.
        let mut clock = TownClock::default();
        let true_ahead = 1.79e9;
        assert!(clock.heard(10.0, server(true_ahead, 10.02), 10.04));
        assert!((ahead(&clock) - true_ahead).abs() < 1e-6, "{}", ahead(&clock) - true_ahead);
        // However the way there and back differ, never further out than half
        // the time the answer took.
        for told in [10.0, 10.01, 10.03, 10.04] {
            let mut clock = TownClock::default();
            assert!(clock.heard(10.0, server(true_ahead, told), 10.04));
            assert!((ahead(&clock) - true_ahead).abs() <= 0.02 + 1e-6);
        }
    }

    #[test]
    fn the_quickest_answer_sets_it() {
        let mut clock = TownClock::default();
        let true_ahead = 1.79e9;
        // A slow first answer, out by a tenth of a second the way it came.
        assert!(clock.heard(10.0, server(true_ahead, 10.0), 10.2));
        // A quicker one sets it better; a slower one after it does not.
        assert!(clock.heard(10.3, server(true_ahead, 10.31), 10.32));
        assert!((ahead(&clock) - true_ahead).abs() < 1e-6);
        assert!(!clock.heard(10.4, server(true_ahead, 10.4), 10.6));
        assert!((ahead(&clock) - true_ahead).abs() < 1e-6);
        // Long after, what it was set by could have drifted further out than
        // a slower answer is unsure of: that answer sets it.
        let later = 10.32 + 0.09 / DRIFT;
        assert!(clock.heard(later, server(true_ahead + 0.05, later + 0.1), later + 0.2));
        assert!((ahead(&clock) - (true_ahead + 0.05)).abs() < 1e-6);
    }

    #[test]
    fn an_answer_that_could_not_be_is_not_heard() {
        let mut clock = TownClock::default();
        assert!(!clock.heard(10.0, 1.79e9, 9.9), "came back before it was asked");
        assert!(!clock.heard(10.0, 1.79e9, 10.0 + LONGEST + 0.1), "took too long");
        assert!(!clock.heard(10.0, f64::NAN, 10.1));
        assert!(clock.set.is_none());
    }

    #[test]
    fn asleep_it_keeps_the_server_s_time_by_the_time_of_day() {
        let mut clock = TownClock::default();
        let steady = clock.steady();
        // The server three seconds on from this machine's time of day.
        let time_of_day = clock.time_of_day();
        assert!(clock.heard(steady, time_of_day + 3.0, steady));
        let was = clock.now();
        clock.sleep();
        assert!(clock.set.is_none());
        let now = clock.now();
        assert!((now - was).abs() < 0.05, "{was} then {now}");
        assert!((now - clock.time_of_day() - 3.0).abs() < 0.05);
    }
}
