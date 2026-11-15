//! Dragon attacks: breath fans, enraged novas and calling in minions.

use std::f32::consts::TAU;

use bevy::prelude::*;
use emberbound_core::{Announce, Dragon, Health, Monster, ScreenShake, SpawnProjectile, SpriteId, Team};
use emberbound_domain::bestiary::{DragonKind, DragonPhase, DragonSpec};
use emberbound_domain::combat::AttackProfile;
use emberbound_domain::ports::RandomSource;

use super::{Action, DragonBrain};
use crate::director::RunDirector;
use crate::spawning::Spawner;

const PROJECTILE_KNOCKBACK: f32 = 90.0;
const PROJECTILE_RADIUS: f32 = 4.0;
const PROJECTILE_LIFETIME: f32 = 3.0;
/// Novas fly a little slower than breath so the ring can be threaded.
const NOVA_SPEED_FACTOR: f32 = 0.85;
const NOVA_SHAKE: f32 = 0.2;
const ENRAGE_SHAKE: f32 = 0.5;
/// Seconds between calls for minions while enraged.
const SUMMON_EVERY: f32 = 7.0;
/// No minions are called while this many lesser monsters are alive.
const SUMMON_CAP: usize = 10;
/// Minions appear this far from the dragon.
const SUMMON_RING: (f32, f32) = (28.0, 56.0);

fn projectile_of(kind: DragonKind) -> SpriteId {
    match kind {
        DragonKind::MossWyrm => SpriteId::VenomSpit,
        DragonKind::FrostDrake => SpriteId::FrostShard,
        DragonKind::AshenTyrant => SpriteId::AshFlame,
    }
}

/// Launches one projectile per angle (radians) from `origin`.
fn volley(
    spec: &DragonSpec,
    origin: Vec2,
    speed: f32,
    angles: impl Iterator<Item = f32>,
    projectiles: &mut MessageWriter<SpawnProjectile>,
) {
    projectiles.write_batch(angles.map(|angle| SpawnProjectile {
        team: Team::Horde,
        sprite: projectile_of(spec.kind),
        position: origin,
        velocity: Vec2::from_angle(angle) * speed,
        attack: AttackProfile::flat(spec.breath_damage, PROJECTILE_KNOCKBACK),
        radius: PROJECTILE_RADIUS,
        lifetime: PROJECTILE_LIFETIME,
        pierce: 0,
    }));
}

/// A fan of projectiles centred on the hero, from the mouth side of the body.
pub(super) fn breathe(
    spec: &DragonSpec,
    position: Vec2,
    to_hero: Vec2,
    projectiles: &mut MessageWriter<SpawnProjectile>,
) {
    let mouth = position + Vec2::new(to_hero.x.signum() * spec.radius * 0.8, spec.radius * 0.2);
    let centre = to_hero.to_angle();
    let count = spec.breath_projectiles.max(1);
    let spread = spec.breath_spread_degrees.to_radians();
    let step = if count > 1 { spread / (count - 1) as f32 } else { 0.0 };
    let first = centre - step * (count - 1) as f32 * 0.5;
    let angles = (0..count).map(|i| first + step * i as f32);
    volley(spec, mouth, spec.projectile_speed, angles, projectiles);
}

/// Flags the dragon as enraged the first time its health crosses the
/// threshold, with a banner and a rumble.
pub(super) fn enrage(
    mut dragons: Query<(&Health, &mut DragonBrain), With<Dragon>>,
    mut announce: MessageWriter<Announce>,
    mut shake: ResMut<ScreenShake>,
) {
    for (health, mut brain) in &mut dragons {
        if brain.enraged || brain.spec.phase(health.fraction()) != DragonPhase::Enraged {
            continue;
        }
        brain.enraged = true;
        announce.write(Announce {
            title: format!("{} ENRAGES", brain.spec.name.to_uppercase()),
            subtitle: format!("{} will not fall quietly", brain.spec.epithet),
        });
        shake.add(ENRAGE_SHAKE);
    }
}

/// While enraged and circling: a radial burst, rotated every time so the
/// gaps move.
pub(super) fn nova(
    time: Res<Time>,
    mut dragons: Query<(&Transform, &mut DragonBrain), With<Dragon>>,
    mut projectiles: MessageWriter<SpawnProjectile>,
    mut shake: ResMut<ScreenShake>,
) {
    for (transform, mut brain) in &mut dragons {
        if !brain.enraged || brain.action != Action::Orbit {
            continue;
        }
        brain.nova_in -= time.delta_secs();
        if brain.nova_in > 0.0 {
            continue;
        }
        brain.nova_in = brain.spec.nova_cooldown;
        let count = brain.spec.nova_projectiles.max(1);
        let step = TAU / count as f32;
        let start = brain.nova_angle;
        brain.nova_angle = (start + step * 0.5) % TAU;
        let speed = brain.spec.projectile_speed * NOVA_SPEED_FACTOR;
        let angles = (0..count).map(|i| start + step * i as f32);
        volley(&brain.spec, transform.translation.truncate(), speed, angles, &mut projectiles);
        shake.add(NOVA_SHAKE);
    }
}

/// While enraged: now and then a few minions answer the dragon's call,
/// unless the valley is already crowded.
pub(super) fn summon(
    time: Res<Time>,
    director: Res<RunDirector>,
    monsters: Query<(), With<Monster>>,
    mut dragons: Query<(&Transform, &mut DragonBrain), With<Dragon>>,
    mut spawner: Spawner,
) {
    for (transform, mut brain) in &mut dragons {
        if !brain.enraged {
            continue;
        }
        brain.summon_in -= time.delta_secs();
        if brain.summon_in > 0.0 {
            continue;
        }
        brain.summon_in = SUMMON_EVERY;
        let Some(kind) = brain.spec.summons else { continue };
        if monsters.iter().len() >= SUMMON_CAP {
            continue;
        }
        let spec = kind.spec(director.difficulty());
        let lair = transform.translation.truncate();
        let pack = if spawner.rng.chance(0.5) { 3 } else { 2 };
        for _ in 0..pack {
            if let Some(spot) = spawner.spot_around(lair, SUMMON_RING, spec.radius, !spec.flies) {
                spawner.spawn_monster(&spec, spot);
            }
        }
    }
}
