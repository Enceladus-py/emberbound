//! Loot dropped by the slain, and the pickups the hero gathers.

use bevy::prelude::*;
use emberbound_core::{
    ActorDied, Dragon, GameRng, GameState, Health, HeroState, Monster, Player, SpriteId, SpriteLibrary,
    XpCollected, layer,
};
use emberbound_domain::ports::RandomSource;

const PICKUP_FPS: f32 = 8.0;
const HEART_HEAL: f32 = 20.0;
const HEART_CHANCE: f32 = 0.06;
const MONSTER_ORBS_MAX: u32 = 3;
const DRAGON_ORBS: (u32, u32) = (10, 14);
const DRAGON_HEARTS: usize = 2;
/// Pop-out speed range of ordinary drops and of a dragon's shower.
const SCATTER_SPEED: (f32, f32) = (25.0, 70.0);
const SHOWER_SPEED: (f32, f32) = (60.0, 190.0);
/// How quickly the pop-out motion dies down (per second).
const SETTLE_RATE: f32 = 5.0;
/// Top speed of a magnetised pickup; well above the hero's so it always arrives.
const MAGNET_SPEED: f32 = 210.0;
/// How quickly a magnetised pickup turns towards the hero (per second).
const MAGNET_RATE: f32 = 7.0;
const COLLECT_RADIUS: f32 = 7.0;
const BOB_HEIGHT: f32 = 1.5;
const BOB_SPEED: f32 = 4.0;

#[derive(Clone, Copy)]
enum Reward {
    Embers(u32),
    Heart(f32),
}

/// Something lying on the ground for the hero to gather. It moves itself
/// (no `Velocity`/`Body`): terrain never blocks loot.
#[derive(Component)]
pub struct Pickup {
    reward: Reward,
    /// Resting position; the sprite bobs around it.
    position: Vec2,
    velocity: Vec2,
    /// Offset into the bob cycle so a pile of loot does not move in unison.
    phase: f32,
}

/// Splits `total` into at most `parts` positive shares that add up to it.
fn shares(total: u32, parts: u32) -> impl Iterator<Item = u32> {
    let parts = parts.clamp(1, total.max(1));
    (0..parts).map(move |i| total / parts + u32::from(i < total % parts)).filter(|share| *share > 0)
}

/// Spawns one pickup that pops out of `origin` in a random direction.
fn spawn_pickup(
    commands: &mut Commands,
    library: &SpriteLibrary,
    rng: &mut GameRng,
    reward: Reward,
    origin: Vec2,
    speed: (f32, f32),
) {
    let sprite = match reward {
        Reward::Embers(_) => SpriteId::EmberOrb,
        Reward::Heart(_) => SpriteId::Heart,
    };
    commands.spawn((
        library.animated(sprite, PICKUP_FPS),
        Transform::from_translation(origin.extend(layer::PICKUP)),
        Pickup {
            reward,
            position: origin,
            velocity: rng.direction() * rng.range(speed.0, speed.1),
            phase: rng.range(0.0, std::f32::consts::TAU),
        },
        DespawnOnExit(GameState::InRun),
    ));
}

/// Turns this frame's deaths into pickups. Runs before the dead are buried,
/// so their reward components are still there to read.
pub fn drop_loot(
    mut deaths: MessageReader<ActorDied>,
    monsters: Query<&Monster>,
    dragons: Query<&Dragon>,
    library: Res<SpriteLibrary>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    for death in deaths.read() {
        let at = death.position;
        if let Ok(monster) = monsters.get(death.entity) {
            let orbs = 1 + rng.index(MONSTER_ORBS_MAX as usize) as u32;
            for share in shares(monster.xp_reward, orbs) {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Embers(share), at, SCATTER_SPEED);
            }
            if rng.chance(HEART_CHANCE) {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Heart(HEART_HEAL), at, SCATTER_SPEED);
            }
        } else if let Ok(dragon) = dragons.get(death.entity) {
            let orbs = DRAGON_ORBS.0 + rng.index((DRAGON_ORBS.1 - DRAGON_ORBS.0 + 1) as usize) as u32;
            for share in shares(dragon.xp_reward, orbs) {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Embers(share), at, SHOWER_SPEED);
            }
            for _ in 0..DRAGON_HEARTS {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Heart(HEART_HEAL), at, SHOWER_SPEED);
            }
        }
    }
}

/// Moves pickups: the pop-out settles, the living hero's magnet pulls, and the
/// sprite bobs gently around the resting position.
pub fn drift(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    hero: Single<(&Transform, &Health), (With<Player>, Without<Pickup>)>,
    mut pickups: Query<(&mut Transform, &mut Pickup)>,
) {
    let dt = time.delta_secs();
    let (hero_transform, hero_health) = *hero;
    let hero_position = hero_transform.translation.truncate();
    let magnet_radius = if hero_health.is_dead() { 0.0 } else { hero_state.stats.pickup_radius };

    for (mut transform, mut pickup) in &mut pickups {
        let to_hero = hero_position - pickup.position;
        if to_hero.length() < magnet_radius {
            let pull = to_hero.normalize_or_zero() * MAGNET_SPEED;
            pickup.velocity = pickup.velocity.lerp(pull, 1.0 - (-MAGNET_RATE * dt).exp());
        } else {
            pickup.velocity *= (-SETTLE_RATE * dt).exp();
        }
        let step = pickup.velocity * dt;
        pickup.position += step;
        let bob = (time.elapsed_secs() * BOB_SPEED + pickup.phase).sin() * BOB_HEIGHT;
        transform.translation = (pickup.position + Vec2::Y * bob).extend(layer::PICKUP);
    }
}

/// Hands pickups that reached the living hero over to them.
pub fn collect(
    mut hero: Single<(&Transform, &mut Health), With<Player>>,
    pickups: Query<(Entity, &Pickup)>,
    mut commands: Commands,
    mut collected: MessageWriter<XpCollected>,
) {
    let (hero_transform, hero_health) = &mut *hero;
    if hero_health.is_dead() {
        return;
    }
    let hero_position = hero_transform.translation.truncate();
    for (entity, pickup) in &pickups {
        if pickup.position.distance(hero_position) > COLLECT_RADIUS {
            continue;
        }
        match pickup.reward {
            Reward::Embers(amount) => {
                collected.write(XpCollected { amount });
            }
            Reward::Heart(amount) => hero_health.heal(amount),
        }
        commands.entity(entity).try_despawn();
    }
}
