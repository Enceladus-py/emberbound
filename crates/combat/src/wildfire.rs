//! Wildfire: slain monsters may burst into a ring of fire bolts.

use bevy::prelude::*;
use emberbound_core::{ActorDied, ActorKind, GameRng, HeroState, SpawnProjectile, SpriteId, Team};
use emberbound_domain::combat::AttackProfile;
use emberbound_domain::ports::RandomSource;

const BOLTS: u32 = 6;
const BOLT_SPEED: f32 = 120.0;
const BOLT_RADIUS: f32 = 4.0;
const BOLT_LIFETIME: f32 = 0.7;
const BOLT_KNOCKBACK: f32 = 50.0;

pub fn burst_corpses(
    mut deaths: MessageReader<ActorDied>,
    hero_state: Res<HeroState>,
    mut rng: ResMut<GameRng>,
    mut shots: MessageWriter<SpawnProjectile>,
) {
    let stats = &hero_state.stats;
    for death in deaths.read() {
        if !matches!(death.kind, ActorKind::Monster(_)) || !rng.chance(stats.corpse_burst_chance) {
            continue;
        }
        // A random twist so bursts do not all fire along the same lines.
        let twist = rng.range(0.0, std::f32::consts::TAU);
        let attack = AttackProfile::flat(stats.companion_damage, BOLT_KNOCKBACK);
        for bolt in 0..BOLTS {
            let direction = Vec2::from_angle(twist + bolt as f32 * std::f32::consts::TAU / BOLTS as f32);
            shots.write(SpawnProjectile {
                team: Team::Hero,
                sprite: SpriteId::Firebolt,
                position: death.position,
                velocity: direction * BOLT_SPEED,
                attack,
                radius: BOLT_RADIUS,
                lifetime: BOLT_LIFETIME,
                pierce: 0,
            });
        }
    }
}
