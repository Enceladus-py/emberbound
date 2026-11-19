//! Ember Wake: a dash leaves burning patches behind that hurt the horde.

use bevy::prelude::*;
use emberbound_core::SpriteId;
use emberbound_core::{
    Body, DamageRequest, DashCooldown, GameState, Health, HeroState, Player, SpriteLibrary, Team, layer,
};
use emberbound_domain::combat::AttackProfile;

/// How long after a dash starts the trail keeps being laid (the dash
/// itself lasts a little less).
const TRAIL_SECONDS: f32 = 0.22;
/// Distance between two patches of one trail.
const PATCH_SPACING: f32 = 9.0;
const PATCH_LIFETIME: f32 = 2.5;
const BURN_INTERVAL: f32 = 0.4;
const BURN_RADIUS: f32 = 9.0;
const BURN_KNOCKBACK: f32 = 20.0;
const PATCH_FPS: f32 = 9.0;
/// Burning patches lie on the ground, under everything standing on it.
const PATCH_Z: f32 = layer::SHADOW + 0.5;

/// The trail being laid by the current dash.
#[derive(Resource, Default)]
pub struct Trail {
    /// Seconds of laying left (0 = not dashing).
    remaining: f32,
    last_patch: Option<Vec2>,
    /// Dash cooldown seen last frame; a jump up means a dash started.
    last_cooldown: f32,
}

pub fn reset(mut trail: ResMut<Trail>) {
    *trail = Trail::default();
}

/// One burning patch.
#[derive(Component)]
pub struct FirePatch {
    damage: f32,
    life: Timer,
    burn: Timer,
}

/// Starts a trail when the hero dashes and drops patches along it.
pub fn lay_trail(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    library: Res<SpriteLibrary>,
    hero: Single<(&Transform, &DashCooldown), With<Player>>,
    mut trail: ResMut<Trail>,
    mut commands: Commands,
) {
    let (transform, cooldown) = *hero;
    let dashed = cooldown.remaining > trail.last_cooldown + 0.05;
    trail.last_cooldown = cooldown.remaining;
    let damage = hero_state.stats.dash_trail_damage;
    if damage <= 0.0 {
        return;
    }
    if dashed {
        trail.remaining = TRAIL_SECONDS;
        trail.last_patch = None;
    }
    if trail.remaining <= 0.0 {
        return;
    }
    trail.remaining -= time.delta_secs();

    let here = transform.translation.truncate();
    if trail.last_patch.is_some_and(|last| last.distance(here) < PATCH_SPACING) {
        return;
    }
    trail.last_patch = Some(here);
    // Patches sit at the hero's feet.
    let feet = here - Vec2::Y * 4.0;
    commands.spawn((
        library.animated(SpriteId::FirePatch, PATCH_FPS),
        Transform::from_translation(feet.extend(PATCH_Z)),
        FirePatch {
            damage,
            life: Timer::from_seconds(PATCH_LIFETIME, TimerMode::Once),
            burn: Timer::from_seconds(BURN_INTERVAL, TimerMode::Repeating),
        },
        DespawnOnExit(GameState::InRun),
    ));
}

/// Patches burn whatever of the horde stands in them, then gutter out.
pub fn burn(
    time: Res<Time>,
    mut patches: Query<(Entity, &Transform, &mut FirePatch, &mut Sprite)>,
    horde: Query<(Entity, &Transform, &Team, &Body, &Health)>,
    mut commands: Commands,
    mut damage: MessageWriter<DamageRequest>,
) {
    for (entity, transform, mut patch, mut sprite) in &mut patches {
        if patch.life.tick(time.delta()).is_finished() {
            commands.entity(entity).try_despawn();
            continue;
        }
        // Fade over the last third of its life.
        let left = patch.life.remaining_secs() / PATCH_LIFETIME;
        sprite.color.set_alpha((left * 3.0).min(1.0));
        if !patch.burn.tick(time.delta()).just_finished() {
            continue;
        }
        let origin = transform.translation.truncate();
        for (target, victim, team, body, health) in &horde {
            if *team == Team::Horde
                && !health.is_dead()
                && victim.translation.truncate().distance(origin) <= BURN_RADIUS + body.radius
            {
                damage.write(DamageRequest {
                    target,
                    attack: AttackProfile::flat(patch.damage, BURN_KNOCKBACK),
                    origin,
                });
            }
        }
    }
}
