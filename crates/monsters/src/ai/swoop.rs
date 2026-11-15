//! `Behavior::Swoop`: fly at the hero while weaving from side to side.

use std::f32::consts::TAU;

use bevy::prelude::*;
use emberbound_core::{GameRng, Monster, Velocity};
use emberbound_domain::bestiary::Behavior;
use emberbound_domain::ports::RandomSource;

use super::Brain;
use crate::hero::HeroPos;

/// Weave oscillations, in radians per second.
const WEAVE_RATE: f32 = 3.2;
/// Largest angle (radians) the heading swings away from the hero.
const WEAVE_ANGLE: f32 = 0.9;

/// Phase of a swooper's sideways weave.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Weave {
    phase: f32,
}

impl Weave {
    pub fn new(rng: &mut GameRng) -> Self {
        Self { phase: rng.range(0.0, TAU) }
    }
}

pub(super) fn swoop(
    time: Res<Time>,
    hero: HeroPos,
    mut swoopers: Query<(&Transform, &Brain, &mut Weave, &mut Velocity), With<Monster>>,
) {
    let hero_pos = hero.translation.truncate();
    for (transform, brain, mut weave, mut velocity) in &mut swoopers {
        let Behavior::Swoop = brain.behavior else { continue };
        weave.phase = (weave.phase + WEAVE_RATE * time.delta_secs()) % TAU;
        let to_hero = (hero_pos - transform.translation.truncate()).normalize_or_zero();
        let heading = Vec2::from_angle(weave.phase.sin() * WEAVE_ANGLE).rotate(to_hero);
        velocity.0 = heading * brain.speed;
    }
}
