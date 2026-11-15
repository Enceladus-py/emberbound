//! Elder dragons: the bosses of a run.
//!
//! Every number that matters for balance comes from the domain's
//! [`DragonSpec`]; this module only adds pacing and presentation.

mod attacks;
mod flight;
mod spawn;

pub(crate) use spawn::spawn_dragon;

use bevy::prelude::*;
use emberbound_core::GameRng;
use emberbound_domain::bestiary::DragonSpec;
use emberbound_domain::ports::RandomSource;

use crate::AiStep;
use crate::hero::hero_alive;

/// Seconds between charge passes.
const CHARGE_EVERY: (f32, f32) = (9.0, 12.0);
/// Seconds between reversals of the circling direction.
const REVERSE_EVERY: (f32, f32) = (3.0, 7.0);
/// Delay before an enraged dragon first calls for minions.
const FIRST_SUMMON_DELAY: f32 = 2.5;

/// What a dragon is doing right now.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    /// Closing in to the orbit distance and circling the hero.
    Orbit,
    /// Rearing back; breathes when `left` runs out.
    BreathWindup { left: f32 },
    /// Coiling up; charges when `left` runs out.
    ChargeWindup { left: f32 },
    /// Flying straight along `direction` for `left` more seconds.
    Charge { left: f32, direction: Vec2 },
}

/// A dragon's state machine and attack timers (all in seconds).
#[derive(Component, Debug, Clone)]
pub(crate) struct DragonBrain {
    spec: DragonSpec,
    action: Action,
    /// Set once health drops below the spec's enrage threshold.
    enraged: bool,
    /// `1.0` circles counter-clockwise, `-1.0` clockwise.
    orbit_side: f32,
    reverse_in: f32,
    breath_in: f32,
    charge_in: f32,
    nova_in: f32,
    /// Start angle of the next nova, rotated after each one.
    nova_angle: f32,
    summon_in: f32,
}

impl DragonBrain {
    fn new(spec: DragonSpec, rng: &mut GameRng) -> Self {
        Self {
            spec,
            action: Action::Orbit,
            enraged: false,
            orbit_side: if rng.chance(0.5) { 1.0 } else { -1.0 },
            reverse_in: rng.range(REVERSE_EVERY.0, REVERSE_EVERY.1),
            breath_in: spec.breath_cooldown,
            charge_in: rng.range(CHARGE_EVERY.0, CHARGE_EVERY.1),
            nova_in: spec.nova_cooldown * 0.5,
            nova_angle: 0.0,
            summon_in: FIRST_SUMMON_DELAY,
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (attacks::enrage, flight::fly, attacks::nova, attacks::summon, flight::telegraph)
            .chain()
            .run_if(hero_alive)
            .in_set(AiStep::Decide),
    );
}
