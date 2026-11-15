//! Getting walkers around obstacles without path-finding.

use bevy::prelude::*;
use emberbound_core::GameRng;
use emberbound_domain::ports::RandomSource;

/// Moving less than this fraction of the intended step counts as blocked.
const BLOCKED_FRACTION: f32 = 0.35;
/// How long a walker sidesteps once it has been blocked.
const DETOUR_SECONDS: f32 = 0.55;
/// How much of the original heading is kept while sidestepping.
const FORWARD_BIAS: f32 = 0.25;

/// Notices when terrain stops a walker and steers it sideways for a moment,
/// so it rounds trees, rocks and ponds instead of grinding into them.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Detour {
    last_position: Vec2,
    /// Distance the walker meant to cover since `last_position` (0 = none).
    intended_step: f32,
    /// Seconds of sidestepping left.
    remaining: f32,
    /// Which way to sidestep (`1.0` = left of the heading, `-1.0` = right).
    side: f32,
}

impl Detour {
    pub fn new(rng: &mut GameRng) -> Self {
        Self {
            last_position: Vec2::ZERO,
            intended_step: 0.0,
            remaining: 0.0,
            side: if rng.chance(0.5) { 1.0 } else { -1.0 },
        }
    }

    /// Forgets the last step, e.g. after standing still or being moved by
    /// something other than walking.
    pub fn rest(&mut self) {
        self.intended_step = 0.0;
    }

    /// Call once per frame with the heading the walker wants (unit length or
    /// zero) and the step it will take; returns the heading to actually use.
    pub fn steer(&mut self, position: Vec2, desired: Vec2, step: f32, dt: f32) -> Vec2 {
        let moved = position.distance(self.last_position);
        let blocked = self.intended_step > 0.0 && moved < self.intended_step * BLOCKED_FRACTION;
        self.last_position = position;
        self.remaining -= dt;
        if blocked && self.remaining <= 0.0 {
            // Alternate sides so a dead end is escaped on the next try.
            self.side = -self.side;
            self.remaining = DETOUR_SECONDS;
        }
        let heading = if self.remaining > 0.0 {
            (desired.perp() * self.side + desired * FORWARD_BIAS).normalize_or_zero()
        } else {
            desired
        };
        self.intended_step = if heading == Vec2::ZERO { 0.0 } else { step };
        heading
    }
}
