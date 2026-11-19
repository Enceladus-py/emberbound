//! The hero's sword: a short arc swung along the facing direction.

use bevy::prelude::*;
use emberbound_core::{
    Body, DamageRequest, Facing, GameState, Health, HeroState, Player, PlayerIntent, SpawnProjectile,
    SpriteId, SpriteLibrary, Team, layer,
};
use emberbound_domain::combat::AttackProfile;

/// Half-angle of the arc that a swing covers, in degrees.
const ARC_HALF_ANGLE_DEG: f32 = 75.0;
/// Anything this close to the hero is hit regardless of angle.
const POINT_BLANK: f32 = 10.0;
const SLASH_FPS: f32 = 20.0;
/// Size of the slash sprite and how far out it is drawn, in reaches.
const ARC_SPAN: f32 = 1.6;
const ARC_OFFSET: f32 = 0.6;

/// Crescent Wave: share of the sword's damage the flying crescent deals,
/// and how it flies.
const WAVE_DAMAGE: f32 = 0.7;
const WAVE_SPEED: f32 = 175.0;
const WAVE_RADIUS: f32 = 7.0;
const WAVE_LIFETIME: f32 = 0.75;
const WAVE_PIERCE: u32 = 4;

/// Seconds until the next swing is allowed. Kept as a plain countdown because
/// boons shorten `attack_cooldown` in the middle of a run. Also counts swings
/// for boons that act on every n-th one.
#[derive(Resource, Default)]
pub struct SwingCooldown {
    remaining: f32,
    swings: u32,
}

pub fn reset(mut cooldown: ResMut<SwingCooldown>) {
    *cooldown = SwingCooldown::default();
}

pub fn swing(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    hero_state: Res<HeroState>,
    library: Res<SpriteLibrary>,
    mut cooldown: ResMut<SwingCooldown>,
    hero: Single<(&Transform, &Facing, &Health), With<Player>>,
    targets: Query<(Entity, &Transform, &Team, &Body), With<Health>>,
    mut commands: Commands,
    mut damage: MessageWriter<DamageRequest>,
    mut projectiles: MessageWriter<SpawnProjectile>,
) {
    cooldown.remaining = (cooldown.remaining - time.delta_secs()).max(0.0);
    let (hero_transform, facing, health) = *hero;
    if !intent.attack || health.is_dead() || cooldown.remaining > 0.0 {
        return;
    }
    let stats = &hero_state.stats;
    cooldown.remaining = stats.attack_cooldown;
    cooldown.swings += 1;
    let attack = stats.sword_attack(health.fraction());

    let origin = hero_transform.translation.truncate();
    let facing = facing.normalize_or(Vec2::X);
    let reach = stats.attack_reach;

    // The crescent sits near the middle of its frame: drawn ARC_SPAN reaches
    // wide and ARC_OFFSET reaches out, its outer edge lands at the reach.
    let scale = reach * ARC_SPAN / library.size(SpriteId::SlashArc).x;
    commands.spawn((
        library.one_shot(SpriteId::SlashArc, SLASH_FPS),
        Transform::from_translation((origin + facing * reach * ARC_OFFSET).extend(layer::FX))
            .with_rotation(Quat::from_rotation_z(facing.to_angle()))
            .with_scale(Vec3::new(scale, scale, 1.0)),
        DespawnOnExit(GameState::InRun),
    ));

    let min_alignment = ARC_HALF_ANGLE_DEG.to_radians().cos();
    for (target, transform, team, body) in &targets {
        if *team != Team::Horde {
            continue;
        }
        let offset = transform.translation.truncate() - origin;
        let distance = offset.length();
        if distance > reach + body.radius {
            continue;
        }
        if distance < POINT_BLANK || offset.dot(facing) >= distance * min_alignment {
            damage.write(DamageRequest { target, attack, origin });
        }
    }

    if stats.wave_every > 0 && cooldown.swings.is_multiple_of(stats.wave_every) {
        projectiles.write(SpawnProjectile {
            team: Team::Hero,
            sprite: SpriteId::SlashWave,
            position: origin + facing * reach * 0.5,
            velocity: facing * WAVE_SPEED,
            attack: AttackProfile { damage: attack.damage * WAVE_DAMAGE, ..attack },
            radius: WAVE_RADIUS,
            lifetime: WAVE_LIFETIME,
            pierce: WAVE_PIERCE,
        });
    }
}
