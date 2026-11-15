//! `Behavior::Chase`: walk straight at the hero.

use bevy::prelude::*;
use emberbound_core::{Monster, Velocity};
use emberbound_domain::bestiary::Behavior;

use super::Brain;
use super::steering::Detour;
use crate::hero::HeroPos;

/// Closer than this the chaser is on top of the hero and stops pushing.
const ARRIVE_DISTANCE: f32 = 2.0;

pub(super) fn chase(
    time: Res<Time>,
    hero: HeroPos,
    mut chasers: Query<(&Transform, &Brain, &mut Detour, &mut Velocity), With<Monster>>,
) {
    let dt = time.delta_secs();
    let hero_pos = hero.translation.truncate();
    for (transform, brain, mut detour, mut velocity) in &mut chasers {
        let Behavior::Chase = brain.behavior else { continue };
        let position = transform.translation.truncate();
        let to_hero = hero_pos - position;
        let desired =
            if to_hero.length() > ARRIVE_DISTANCE { to_hero.normalize() } else { Vec2::ZERO };
        velocity.0 = detour.steer(position, desired, brain.speed * dt, dt) * brain.speed;
    }
}
