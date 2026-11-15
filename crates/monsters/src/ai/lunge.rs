//! `Behavior::Lunge`: close in, wind up visibly, dash in a straight line.

use bevy::prelude::*;
use emberbound_core::{Monster, Velocity};
use emberbound_domain::bestiary::Behavior;

use super::Brain;
use super::steering::Detour;
use crate::hero::HeroPos;
use crate::route::Trail;

/// Distance to the hero at which the wind-up starts.
const TRIGGER_DISTANCE: f32 = 60.0;
/// Seconds spent catching breath after a dash.
const RECOVER_SECONDS: f32 = 0.6;
/// Wind-up telegraph: the body squashes down and trembles.
const SQUASH: f32 = 0.25;
const TREMBLE: f32 = 0.07;
const TREMBLE_RATE: f32 = 55.0;
/// Stretch along the ground while dashing.
const DASH_STRETCH: Vec3 = Vec3::new(1.15, 0.9, 1.0);

/// Where a lunger is in its attack cycle.
#[derive(Component, Debug, Clone, Copy)]
pub(super) enum LungeState {
    Approach,
    /// Standing still, telegraphing; `left` seconds until the dash.
    Windup {
        left: f32,
    },
    /// Dashing along `direction` for `left` more seconds.
    Dash {
        left: f32,
        direction: Vec2,
    },
    Recover {
        left: f32,
    },
}

pub(super) fn lunge(
    time: Res<Time>,
    hero: HeroPos,
    trail: Res<Trail>,
    mut lungers: Query<(&mut Transform, &Brain, &mut Detour, &mut LungeState, &mut Velocity), With<Monster>>,
) {
    let dt = time.delta_secs();
    let hero_pos = hero.translation.truncate();
    for (mut transform, brain, mut detour, mut state, mut velocity) in &mut lungers {
        let Behavior::Lunge { windup, lunge_speed, lunge_time } = brain.behavior else { continue };
        let position = transform.translation.truncate();
        let offset = hero_pos - position;
        let to_hero = offset.normalize_or_zero();
        let mut scale = Vec3::ONE;

        *state = match *state {
            LungeState::Approach if offset.length() <= TRIGGER_DISTANCE => {
                detour.rest();
                velocity.0 = Vec2::ZERO;
                LungeState::Windup { left: windup }
            }
            LungeState::Approach => {
                let heading = trail.heading(position, hero_pos);
                velocity.0 = detour.steer(position, heading, brain.speed * dt, dt) * brain.speed;
                LungeState::Approach
            }
            LungeState::Windup { left } if left <= dt => {
                // The direction is locked here: sidestepping dodges the dash.
                LungeState::Dash { left: lunge_time, direction: to_hero }
            }
            LungeState::Windup { left } => {
                let progress = 1.0 - left / windup.max(f32::EPSILON);
                let tremble = (left * TREMBLE_RATE).sin() * TREMBLE;
                scale = Vec3::new(1.0 + SQUASH * progress + tremble, 1.0 - SQUASH * progress * 0.8, 1.0);
                velocity.0 = Vec2::ZERO;
                LungeState::Windup { left: left - dt }
            }
            LungeState::Dash { left, .. } if left <= 0.0 => {
                velocity.0 = Vec2::ZERO;
                LungeState::Recover { left: RECOVER_SECONDS }
            }
            LungeState::Dash { left, direction } => {
                scale = DASH_STRETCH;
                velocity.0 = direction * lunge_speed;
                LungeState::Dash { left: left - dt, direction }
            }
            LungeState::Recover { left } if left <= 0.0 => LungeState::Approach,
            LungeState::Recover { left } => {
                velocity.0 = Vec2::ZERO;
                LungeState::Recover { left: left - dt }
            }
        };
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}
