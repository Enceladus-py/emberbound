//! Things every creature of the horde does regardless of its brain.

use bevy::prelude::*;
use emberbound_core::{Dragon, Facing, Monster, Velocity};

use crate::AiStep;
use crate::hero::{HeroPos, hero_alive};

/// Horizontal dead zone so sprites do not flicker when right above/below.
const FLIP_DEAD_ZONE: f32 = 1.0;

type Horde = Or<(With<Monster>, With<Dragon>)>;

/// Looks at the hero. Sprites are drawn facing right.
fn face_hero(hero: HeroPos, mut horde: Query<(&Transform, &mut Facing, &mut Sprite), Horde>) {
    let hero_pos = hero.translation.truncate();
    for (transform, mut facing, mut sprite) in &mut horde {
        let to_hero = hero_pos - transform.translation.truncate();
        if let Some(direction) = to_hero.try_normalize() {
            facing.0 = direction;
        }
        let flip = to_hero.x < 0.0;
        if to_hero.x.abs() > FLIP_DEAD_ZONE && sprite.flip_x != flip {
            sprite.flip_x = flip;
        }
    }
}

/// With the hero dead the horde simply stops.
fn stand_still(mut horde: Query<&mut Velocity, Horde>) {
    for mut velocity in &mut horde {
        velocity.0 = Vec2::ZERO;
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (face_hero.run_if(hero_alive), stand_still.run_if(not(hero_alive))).in_set(AiStep::Settle),
    );
}
