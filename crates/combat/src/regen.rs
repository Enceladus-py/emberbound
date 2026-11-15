//! Passive regeneration of the hero.

use bevy::prelude::*;
use emberbound_core::{Health, HeroState, Player};

pub fn regenerate(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    mut health: Single<&mut Health, With<Player>>,
) {
    let rate = hero_state.stats.regen_per_second;
    // `heal` never revives, so a fallen hero stays down.
    if rate > 0.0 {
        health.heal(rate * time.delta_secs());
    }
}
