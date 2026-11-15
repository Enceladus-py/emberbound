//! Keeps the horde from stacking into a single blob.

use bevy::prelude::*;
use emberbound_core::{Body, Monster, Velocity};

/// Speed (units / second) of the push at full overlap.
const PUSH_SPEED: f32 = 36.0;
/// Extra personal space on top of the two body radii.
const PERSONAL_SPACE: f32 = 2.0;

/// Soft separation: overlapping monsters nudge each other apart. Quadratic,
/// which is fine for the few dozen monsters the director allows.
pub(super) fn keep_apart(
    mut monsters: Query<(&Transform, &Body, &mut Velocity), With<Monster>>,
    mut bodies: Local<Vec<(Vec2, f32)>>,
) {
    bodies.clear();
    bodies.extend(monsters.iter().map(|(t, body, _)| (t.translation.truncate(), body.radius)));
    for (index, (_, _, mut velocity)) in monsters.iter_mut().enumerate() {
        let (position, radius) = bodies[index];
        let mut push = Vec2::ZERO;
        for (other, &(other_position, other_radius)) in bodies.iter().enumerate() {
            if other == index {
                continue;
            }
            let reach = radius + other_radius + PERSONAL_SPACE;
            let away = position - other_position;
            let distance = away.length();
            if distance >= reach {
                continue;
            }
            // Perfectly stacked pairs split along a direction unique to each.
            let direction = away.try_normalize().unwrap_or(Vec2::from_angle(index as f32));
            push += direction * (1.0 - distance / reach);
        }
        if push != Vec2::ZERO {
            velocity.0 += push.clamp_length_max(1.0) * PUSH_SPEED;
        }
    }
}
