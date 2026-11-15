//! Lesser monster AI: one small system per domain [`Behavior`].
//!
//! Every system only sets [`Velocity`](emberbound_core::Velocity); `world`
//! integrates it and resolves terrain collisions.

mod chase;
mod crowd;
mod lunge;
mod skirmish;
mod steering;
mod swoop;

use bevy::prelude::*;
use emberbound_core::GameRng;
use emberbound_domain::bestiary::{Behavior, MonsterSpec};

use crate::AiStep;
use crate::hero::hero_alive;

/// What a lesser monster does and how fast, straight from its spec.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Brain {
    pub behavior: Behavior,
    /// Walking / flying speed in world units per second.
    pub speed: f32,
}

/// Gives a freshly spawned monster its brain and the private state its
/// behaviour needs.
pub(crate) fn attach_brain(entity: &mut EntityCommands, spec: &MonsterSpec, rng: &mut GameRng) {
    entity.insert(Brain { behavior: spec.behavior, speed: spec.speed });
    match spec.behavior {
        Behavior::Chase => entity.insert(steering::Detour::new(rng)),
        Behavior::Swoop => entity.insert(swoop::Weave::new(rng)),
        Behavior::Skirmish { shot_cooldown, .. } => {
            entity.insert((steering::Detour::new(rng), skirmish::Sling::new(shot_cooldown, rng)))
        }
        Behavior::Lunge { .. } => entity.insert((steering::Detour::new(rng), lunge::LungeState::Approach)),
    };
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (chase::chase, swoop::swoop, skirmish::skirmish, lunge::lunge).in_set(AiStep::Decide),
            crowd::keep_apart.in_set(AiStep::Settle),
        )
            .run_if(hero_alive),
    );
}
