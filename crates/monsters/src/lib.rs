//! Monsters adapter: the run director, monster and dragon AI.
//!
//! * [`director`] wraps the domain's encounter director and executes its
//!   spawn orders.
//! * [`spawning`] knows *where* and *how* monsters enter the valley.
//! * [`ai`] drives lesser monsters, one small system per domain `Behavior`.
//! * [`dragon`] spawns and drives the elder dragons.
//! * [`route`] is the walkers' path-finding towards the hero.
//! * [`hero`] and [`posture`] hold what all of the above share.

mod ai;
mod director;
mod dragon;
mod hero;
mod posture;
mod route;
mod spawning;

use bevy::prelude::*;
use emberbound_core::GameSet;

/// Order of this crate's work inside [`GameSet::Ai`].
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AiStep {
    /// The director spawns, strays are brought back.
    Direct,
    /// Every creature picks its velocity and attacks.
    Decide,
    /// Corrections on top of the chosen velocities (crowding, facing).
    Settle,
}

pub struct MonstersPlugin;

impl Plugin for MonstersPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (AiStep::Direct, AiStep::Decide, AiStep::Settle).chain().in_set(GameSet::Ai),
        );
        director::plugin(app);
        spawning::plugin(app);
        ai::plugin(app);
        dragon::plugin(app);
        posture::plugin(app);
        route::plugin(app);
    }
}
