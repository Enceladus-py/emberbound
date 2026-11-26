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

#[cfg(test)]
mod tests {
    use emberbound_domain::bestiary::{DragonKind, MonsterKind};

    use super::*;

    const AT: Vec2 = Vec2::new(120.0, -40.0);

    fn app(chance: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ActorDied>()
            .add_message::<SpawnProjectile>()
            .init_resource::<HeroState>()
            .insert_resource(GameRng::new(11))
            .add_systems(Update, burst_corpses);
        app.world_mut().resource_mut::<HeroState>().stats.corpse_burst_chance = chance;
        app
    }

    fn die(app: &mut App, kind: ActorKind) -> Vec<SpawnProjectile> {
        let entity = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(ActorDied { entity, kind, position: AT });
        app.update();
        app.world_mut().resource_mut::<Messages<SpawnProjectile>>().drain().collect()
    }

    #[test]
    fn a_slain_monster_bursts_into_an_even_ring_of_hero_bolts() {
        let mut app = app(1.0);
        let bolts = die(&mut app, ActorKind::Monster(MonsterKind::Goblin));
        assert_eq!(bolts.len(), BOLTS as usize);
        for bolt in &bolts {
            assert_eq!(bolt.team, Team::Hero);
            assert_eq!(bolt.position, AT);
            assert!((bolt.velocity.length() - BOLT_SPEED).abs() < 1e-3);
        }
        let mut angles: Vec<f32> = bolts.iter().map(|bolt| bolt.velocity.to_angle()).collect();
        angles.sort_by(f32::total_cmp);
        let gap = std::f32::consts::TAU / BOLTS as f32;
        for pair in angles.windows(2) {
            assert!((pair[1] - pair[0] - gap).abs() < 1e-3, "uneven gap in {angles:?}");
        }
    }

    #[test]
    fn heroes_and_dragons_never_burst() {
        let mut app = app(1.0);
        assert!(die(&mut app, ActorKind::Hero).is_empty());
        assert!(die(&mut app, ActorKind::Dragon(DragonKind::AshenTyrant)).is_empty());
    }

    #[test]
    fn no_burst_without_the_boon() {
        let mut app = app(0.0);
        for _ in 0..20 {
            assert!(die(&mut app, ActorKind::Monster(MonsterKind::Slime)).is_empty());
        }
    }
}
