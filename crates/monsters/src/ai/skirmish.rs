//! `Behavior::Skirmish`: keep a distance, strafe and sling stones.

use bevy::prelude::*;
use emberbound_core::{Body, GameRng, Monster, SpawnProjectile, SpriteId, Team, Velocity};
use emberbound_domain::bestiary::Behavior;
use emberbound_domain::combat::AttackProfile;
use emberbound_domain::ports::RandomSource;

use super::Brain;
use super::steering::Detour;
use crate::hero::HeroPos;

/// Backs off when the hero is closer than this share of the preferred distance.
const RETREAT_FRACTION: f32 = 0.7;
/// Closes in when further than this share of the preferred distance.
const APPROACH_FRACTION: f32 = 1.05;
/// Shoots only within this multiple of the preferred distance.
const SHOT_RANGE_FACTOR: f32 = 1.6;
/// Sideways share of the movement while circling.
const STRAFE_WEIGHT: f32 = 0.45;
/// Seconds between changes of strafing direction.
const STRAFE_SWITCH: (f32, f32) = (1.5, 3.5);
const STONE_KNOCKBACK: f32 = 60.0;
const STONE_RADIUS: f32 = 3.0;
const STONE_LIFETIME: f32 = 2.0;

/// A skirmisher's shot timer and strafing state.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Sling {
    /// Seconds until the next shot is ready.
    cooldown: f32,
    strafe_side: f32,
    strafe_switch_in: f32,
}

impl Sling {
    /// The first shot is randomised so packs do not fire in volleys.
    pub fn new(shot_cooldown: f32, rng: &mut GameRng) -> Self {
        Self {
            cooldown: shot_cooldown * rng.range(0.4, 1.2),
            strafe_side: if rng.chance(0.5) { 1.0 } else { -1.0 },
            strafe_switch_in: rng.range(STRAFE_SWITCH.0, STRAFE_SWITCH.1),
        }
    }
}

pub(super) fn skirmish(
    time: Res<Time>,
    hero: HeroPos,
    mut rng: ResMut<GameRng>,
    mut skirmishers: Query<
        (&Transform, &Body, &Brain, &mut Detour, &mut Sling, &mut Velocity),
        With<Monster>,
    >,
    mut projectiles: MessageWriter<SpawnProjectile>,
) {
    let dt = time.delta_secs();
    let hero_pos = hero.translation.truncate();
    for (transform, body, brain, mut detour, mut sling, mut velocity) in &mut skirmishers {
        let Behavior::Skirmish { preferred_distance, shot_cooldown, shot_damage, shot_speed } =
            brain.behavior
        else {
            continue;
        };
        let position = transform.translation.truncate();
        let offset = hero_pos - position;
        let distance = offset.length();
        let to_hero = offset.normalize_or_zero();

        sling.strafe_switch_in -= dt;
        if sling.strafe_switch_in <= 0.0 {
            sling.strafe_side = -sling.strafe_side;
            sling.strafe_switch_in = rng.range(STRAFE_SWITCH.0, STRAFE_SWITCH.1);
        }
        let radial = if distance > preferred_distance * APPROACH_FRACTION {
            to_hero
        } else if distance < preferred_distance * RETREAT_FRACTION {
            -to_hero
        } else {
            Vec2::ZERO
        };
        let desired =
            (radial + to_hero.perp() * sling.strafe_side * STRAFE_WEIGHT).normalize_or_zero();
        velocity.0 = detour.steer(position, desired, brain.speed * dt, dt) * brain.speed;

        sling.cooldown -= dt;
        if sling.cooldown <= 0.0 && distance <= preferred_distance * SHOT_RANGE_FACTOR {
            sling.cooldown = shot_cooldown;
            projectiles.write(SpawnProjectile {
                team: Team::Horde,
                sprite: SpriteId::SlingStone,
                position: position + to_hero * body.radius,
                velocity: to_hero * shot_speed,
                attack: AttackProfile::flat(shot_damage, STONE_KNOCKBACK),
                radius: STONE_RADIUS,
                lifetime: STONE_LIFETIME,
                pierce: 0,
            });
        }
    }
}
