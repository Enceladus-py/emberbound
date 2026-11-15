//! Kinematics: integrates [`Velocity`] and [`Knockback`] and keeps walkers
//! out of water, trees, rocks and ruins.

use bevy::prelude::*;
use emberbound_core::{Body, GameSet, Knockback, Player, Terrain, Velocity};
use emberbound_domain::world::{TILE_SIZE, WorldMap};

/// Seconds in which knockback loses half of its speed.
const KNOCKBACK_HALF_LIFE: f32 = 0.08;
/// Knockback slower than this (units per second) is dropped.
const KNOCKBACK_REST_SPEED: f32 = 1.0;
/// Longest distance a walker moves between two collision checks. Half a
/// tile can never skip over an obstacle.
const MAX_STEP: f32 = TILE_SIZE * 0.5;

/// Moves a walker by `delta`, one axis at a time so it slides along
/// obstacles instead of sticking to them, in steps short enough not to
/// tunnel. A walker that already overlaps an obstacle (e.g. it was spawned
/// or shoved there) moves freely until it is out again.
fn walk(map: &WorldMap, mut position: Vec2, delta: Vec2, radius: f32) -> Vec2 {
    let steps = (delta.abs().max_element() / MAX_STEP).ceil().max(1.0);
    let step = delta / steps;
    for _ in 0..steps as u32 {
        let stuck = !map.is_walkable_at(position.x, position.y, radius);
        if stuck || map.is_walkable_at(position.x + step.x, position.y, radius) {
            position.x += step.x;
        }
        if stuck || map.is_walkable_at(position.x, position.y + step.y, radius) {
            position.y += step.y;
        }
    }
    position
}

/// Applies `(velocity + knockback) * dt` to everything that has a velocity.
/// Only bodies on foot collide with the terrain.
fn integrate_velocities(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut movers: Query<(&mut Transform, &Velocity, Option<&Knockback>, Option<&Body>)>,
) {
    let dt = time.delta_secs();
    for (mut transform, velocity, knockback, body) in &mut movers {
        let delta = (velocity.0 + knockback.map_or(Vec2::ZERO, |k| k.0)) * dt;
        if delta == Vec2::ZERO {
            continue;
        }
        let position = transform.translation.truncate();
        let moved = match body {
            Some(body) if body.walks => walk(&terrain.0, position, delta, body.radius),
            _ => position + delta,
        };
        if moved != position {
            transform.translation = moved.extend(transform.translation.z);
        }
    }
}

/// Lets knockback die away exponentially, independent of the frame rate.
fn decay_knockback(time: Res<Time>, mut knockbacks: Query<&mut Knockback>) {
    let keep = 0.5f32.powf(time.delta_secs() / KNOCKBACK_HALF_LIFE);
    for mut knockback in &mut knockbacks {
        if knockback.0 == Vec2::ZERO {
            continue;
        }
        let decayed = knockback.0 * keep;
        knockback.0 = if decayed.length() < KNOCKBACK_REST_SPEED { Vec2::ZERO } else { decayed };
    }
}

/// Last line of defence: whatever happens, the hero stays on the map.
fn confine_hero(terrain: Res<Terrain>, mut hero: Single<(&mut Transform, &Body), With<Player>>) {
    let (transform, body) = &mut *hero;
    let (width, height) = terrain.0.world_size();
    let min = Vec2::splat(body.radius);
    let max = (Vec2::new(width, height) - min).max(min);
    let position = transform.translation.truncate();
    let confined = position.clamp(min, max);
    if confined != position {
        transform.translation = confined.extend(transform.translation.z);
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        (integrate_velocities, decay_knockback, confine_hero).chain().in_set(GameSet::Movement),
    );
}
