//! The time, for every device to set the town's clock by: the cars round the
//! town, the fountain's jet and the day all go by it, and it has to be one
//! time on all of them (the game's `clock`). A device asks what time it is
//! ([`WhatTime`]) and is told at once, by this machine's clock, which keeps
//! to the world's by NTP ([`TheTime`]).

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::{TheTime, WhatTime};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        tell_the_time
            .after(ServerSystems::Receive)
            .run_if(in_state(ServerState::Running)),
    );
}

/// Answers everyone who has asked what time it is, as soon as the question
/// is read: the answer goes out at the end of this same loop, so that how
/// long it took to come back is no more than the way there and back.
fn tell_the_time(
    mut asks: MessageReader<FromClient<WhatTime>>,
    mut answers: MessageWriter<ToClients<TheTime>>,
) {
    for ask in asks.read() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |since| since.as_secs_f64());
        answers.write(ToClients {
            targets: SendTargets::Single(ask.client_id),
            message: TheTime {
                asked: ask.message.asked,
                now,
            },
        });
    }
}
