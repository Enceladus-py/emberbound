//! The hero's sword: a short arc swung along the facing direction.

use bevy::prelude::*;
use emberbound_core::{
    Body, DamageRequest, Facing, GameState, Health, HeroState, Player, PlayerIntent, SpriteId, SpriteLibrary,
    Team, layer,
};

/// Half-angle of the arc that a swing covers, in degrees.
const ARC_HALF_ANGLE_DEG: f32 = 75.0;
/// Anything this close to the hero is hit regardless of angle.
const POINT_BLANK: f32 = 10.0;
const SLASH_FPS: f32 = 20.0;
/// Size of the slash sprite and how far out it is drawn, in reaches.
const ARC_SPAN: f32 = 1.6;
const ARC_OFFSET: f32 = 0.6;

/// Seconds until the next swing is allowed. Kept as a plain countdown because
/// boons shorten `attack_cooldown` in the middle of a run.
#[derive(Resource, Default)]
pub struct SwingCooldown(f32);

pub fn reset(mut cooldown: ResMut<SwingCooldown>) {
    cooldown.0 = 0.0;
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
) {
    cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.0);
    let (hero_transform, facing, health) = *hero;
    if !intent.attack || health.is_dead() || cooldown.0 > 0.0 {
        return;
    }
    let stats = &hero_state.stats;
    cooldown.0 = stats.attack_cooldown;

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
            damage.write(DamageRequest { target, attack: stats.attack, origin });
        }
    }
}
