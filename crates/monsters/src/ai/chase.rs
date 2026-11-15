//! `Behavior::Chase`: walk at the hero, around whatever is in the way.

use bevy::prelude::*;
use emberbound_core::{Monster, Velocity};
use emberbound_domain::bestiary::Behavior;

use super::Brain;
use super::steering::Detour;
use crate::hero::HeroPos;
use crate::route::Trail;

/// Closer than this the chaser is touching the hero and stops pushing, so
/// it bites from the side instead of standing on top of them.
const ARRIVE_DISTANCE: f32 = 8.0;

pub(super) fn chase(
    time: Res<Time>,
    hero: HeroPos,
    trail: Res<Trail>,
    mut chasers: Query<(&Transform, &Brain, &mut Detour, &mut Velocity), With<Monster>>,
) {
    let dt = time.delta_secs();
    let hero_pos = hero.translation.truncate();
    for (transform, brain, mut detour, mut velocity) in &mut chasers {
        let Behavior::Chase = brain.behavior else { continue };
        let position = transform.translation.truncate();
        let desired = if position.distance(hero_pos) > ARRIVE_DISTANCE {
            trail.heading(position, hero_pos)
        } else {
            Vec2::ZERO
        };
        velocity.0 = detour.steer(position, desired, brain.speed * dt, dt) * brain.speed;
    }
}
