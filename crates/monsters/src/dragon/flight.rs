//! Dragon movement and the breath / charge state machine.

use std::f32::consts::PI;

use bevy::prelude::*;
use emberbound_core::{Dragon, GameRng, SpawnProjectile, Velocity};
use emberbound_domain::ports::RandomSource;

use super::attacks::breathe;
use super::{Action, CHARGE_EVERY, DragonBrain, REVERSE_EVERY};
use crate::hero::HeroPos;

/// Speed and breath-rate bonuses while enraged.
const ENRAGED_SPEED: f32 = 1.35;
const ENRAGED_BREATH_RATE: f32 = 1.25;
/// A dragon far outside its orbit hurries to catch up with the hero.
const CATCH_UP_BEYOND: f32 = 70.0;
const CATCH_UP_SPEED: f32 = 2.2;
/// Distance error (units) at which the dragon flies purely towards / away.
const ORBIT_SLACK: f32 = 30.0;
/// Breath and charges only start within this multiple of the orbit distance.
const ATTACK_RANGE_FACTOR: f32 = 1.8;
const BREATH_TELEGRAPH: f32 = 0.45;
const CHARGE_TELEGRAPH: f32 = 0.75;
/// Charge speed as a multiple of the cruising speed, within sane bounds.
const CHARGE_SPEED_FACTOR: f32 = 4.0;
const CHARGE_SPEED_LIMITS: (f32, f32) = (150.0, 210.0);
/// How far past the hero's position a charge carries.
const CHARGE_OVERSHOOT: f32 = 70.0;
const CHARGE_MAX_SECONDS: f32 = 1.6;
/// Breathing room after a charge before the next breath.
const BREATH_AFTER_CHARGE: f32 = 0.8;
/// Telegraph looks: swell before breathing, coil and tremble before charging.
const BREATH_SWELL: f32 = 0.16;
const CHARGE_COIL: f32 = 0.14;
const CHARGE_TREMBLE: f32 = 0.04;
const CHARGE_TREMBLE_RATE: f32 = 45.0;
const CHARGE_STRETCH: Vec3 = Vec3::new(1.12, 0.92, 1.0);

fn charge_speed(cruise: f32) -> f32 {
    (cruise * CHARGE_SPEED_FACTOR).clamp(CHARGE_SPEED_LIMITS.0, CHARGE_SPEED_LIMITS.1)
}

/// Velocity that brings the dragon to its orbit distance and circles there.
fn orbit_velocity(brain: &DragonBrain, to_hero: Vec2, distance: f32) -> Vec2 {
    let error = distance - brain.spec.orbit_distance;
    let radial = (error / ORBIT_SLACK).clamp(-1.0, 1.0);
    let tangential = 1.0 - radial.abs() * 0.6;
    let heading = to_hero * radial + to_hero.perp() * brain.orbit_side * tangential;
    let mut speed = brain.spec.speed;
    if brain.enraged {
        speed *= ENRAGED_SPEED;
    }
    if error > CATCH_UP_BEYOND {
        speed *= CATCH_UP_SPEED;
    }
    heading.clamp_length_max(1.0) * speed
}

/// Stalk, breathe and charge. Sets velocity only; `world` moves the dragon.
pub(super) fn fly(
    time: Res<Time>,
    hero: HeroPos,
    mut rng: ResMut<GameRng>,
    mut dragons: Query<(&Transform, &mut DragonBrain, &mut Velocity), With<Dragon>>,
    mut projectiles: MessageWriter<SpawnProjectile>,
) {
    let dt = time.delta_secs();
    let hero_pos = hero.translation.truncate();
    for (transform, mut brain, mut velocity) in &mut dragons {
        let position = transform.translation.truncate();
        let offset = hero_pos - position;
        let distance = offset.length();
        let to_hero = offset.normalize_or_zero();

        brain.action = match brain.action {
            Action::Orbit => {
                brain.breath_in -= dt;
                brain.charge_in -= dt;
                brain.reverse_in -= dt;
                if brain.reverse_in <= 0.0 {
                    brain.orbit_side = -brain.orbit_side;
                    brain.reverse_in = rng.range(REVERSE_EVERY.0, REVERSE_EVERY.1);
                }
                velocity.0 = orbit_velocity(&brain, to_hero, distance);
                let in_range = distance <= brain.spec.orbit_distance * ATTACK_RANGE_FACTOR;
                if in_range && brain.charge_in <= 0.0 {
                    Action::ChargeWindup { left: CHARGE_TELEGRAPH }
                } else if in_range && brain.breath_in <= 0.0 {
                    Action::BreathWindup { left: BREATH_TELEGRAPH }
                } else {
                    Action::Orbit
                }
            }
            Action::BreathWindup { left } if left <= dt => {
                breathe(&brain.spec, position, to_hero, &mut projectiles);
                let rate = if brain.enraged { ENRAGED_BREATH_RATE } else { 1.0 };
                brain.breath_in = brain.spec.breath_cooldown / rate;
                Action::Orbit
            }
            Action::BreathWindup { left } => {
                velocity.0 = Vec2::ZERO;
                Action::BreathWindup { left: left - dt }
            }
            Action::ChargeWindup { left } if left <= dt => {
                // Aimed at where the hero stands now: moving aside dodges it.
                let speed = charge_speed(brain.spec.speed);
                let seconds = ((distance + CHARGE_OVERSHOOT) / speed).min(CHARGE_MAX_SECONDS);
                Action::Charge { left: seconds, direction: to_hero }
            }
            Action::ChargeWindup { left } => {
                velocity.0 = Vec2::ZERO;
                Action::ChargeWindup { left: left - dt }
            }
            Action::Charge { left, .. } if left <= 0.0 => {
                brain.charge_in = rng.range(CHARGE_EVERY.0, CHARGE_EVERY.1);
                brain.breath_in = brain.breath_in.max(BREATH_AFTER_CHARGE);
                Action::Orbit
            }
            Action::Charge { left, direction } => {
                velocity.0 = direction * charge_speed(brain.spec.speed);
                Action::Charge { left: left - dt, direction }
            }
        };
    }
}

/// Shows what is coming through the body's scale (never through tint,
/// which belongs to the hit flash).
pub(super) fn telegraph(mut dragons: Query<(&DragonBrain, &mut Transform), With<Dragon>>) {
    for (brain, mut transform) in &mut dragons {
        let scale = match brain.action {
            Action::Orbit => Vec3::ONE,
            Action::BreathWindup { left } => {
                let progress = 1.0 - left / BREATH_TELEGRAPH;
                Vec2::splat(1.0 + BREATH_SWELL * (progress * PI).sin()).extend(1.0)
            }
            Action::ChargeWindup { left } => {
                let progress = 1.0 - left / CHARGE_TELEGRAPH;
                let tremble = (left * CHARGE_TREMBLE_RATE).sin() * CHARGE_TREMBLE;
                Vec3::new(1.0 - CHARGE_COIL * progress + tremble, 1.0 + CHARGE_COIL * progress * 0.7, 1.0)
            }
            Action::Charge { .. } => CHARGE_STRETCH,
        };
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}
