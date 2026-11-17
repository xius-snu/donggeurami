//! What every game that costs coins has in common: a fee being taken, and
//! what a credit leaves an account with.
//!
//! House Builder was the only game with a fee until 2026-11-18, and its
//! answers from the database were its own: whoever's fee was taken was
//! seated in House Builder, whatever they had paid it for. The four games
//! played at tables (`tables`) take fees too, so a charge now says what it
//! is for ([`Charged`](crate::accounts::Charged): the game and which of its
//! games), each game seats only those whose answers are its own, and
//! whoever has a fee in flight ([`Paying`]) joins nothing else until it is
//! answered. Prizes and fees given back are credits, whichever game pays
//! them, and every one of them leaves its account with what it says
//! ([`credited`]).

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::{Minigame, Tell};

use crate::accounts::Credited;
use crate::players::{Body, Everyone, Who, tell};

pub(crate) fn plugin(app: &mut App) {
    // A prize is always asked for before a fee for the next game, and the
    // database answers in the order it was asked: read before every game's
    // fees (`builder::paid`, `tables::paid`, each `.after` this), a body is
    // left with the later of the two.
    app.add_systems(
        PreUpdate,
        credited
            .after(crate::accounts::answers)
            .run_if(in_state(ServerState::Running)),
    );
}

/// A body whose fee is being taken in the database: for a game of `game`,
/// to be seated in the one numbered `into` once it has been, the one that
/// was filling as they asked, which everyone of a group going in together
/// was given alike (`social`), so that they are seated in one game
/// whichever of their fees is taken first. While it is on them they join
/// nothing else, of any game (until 2026-11-18 only House Builder held
/// back, and someone could be seated in Bridge Wars with a fee for House
/// Builder on the way, and then moved to House Builder as it was taken).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Paying {
    pub game: Minigame,
    pub into: Option<u32>,
}

/// A prize, or a fee given back, paid into the database: whoever it was for,
/// if they are still here, has that much to spend now, and their device is
/// told so. (House Builder's own until 2026-11-18, `builder::credited`.)
pub(crate) fn credited(
    mut credited: MessageReader<Credited>,
    everyone: Res<Everyone>,
    mut bodies: Query<&mut Body>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    for answer in credited.read() {
        let Some(left) = answer.left else {
            continue;
        };
        let Some(mut body_of) = everyone
            .0
            .get(&Who(answer.id))
            .and_then(|&body| bodies.get_mut(body).ok())
        else {
            continue;
        };
        body_of.coins = left;
        if let Some(client) = body_of.client {
            tell(&mut tells, client, Tell::Balance(left));
        }
    }
}
