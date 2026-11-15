//! Projectiles of both sides: launched on request, flown by `world`'s
//! movement system, and tested here against the opposing team.

use bevy::prelude::*;
use emberbound_core::{
    Body, DamageRequest, FxKind, FxRequest, GameState, Health, Invulnerable, SpawnProjectile, SpriteLibrary,
    Team, Velocity, layer,
};
use emberbound_domain::combat::AttackProfile;

const PROJECTILE_FPS: f32 = 10.0;
/// Knockback originates this far behind the projectile, so victims are pushed
/// along its flight path rather than away from an arbitrary overlap point.
const IMPACT_SETBACK: f32 = 6.0;

/// A shot in flight.
#[derive(Component)]
pub struct Projectile {
    team: Team,
    attack: AttackProfile,
    radius: f32,
    lifetime: Timer,
    /// Extra targets it may still pass through.
    pierce: u32,
    /// Targets it already hurt; a piercing shot hits each only once.
    already_hit: Vec<Entity>,
    /// It hit its last target and waits for clean-up.
    spent: bool,
}

pub fn launch(
    mut requests: MessageReader<SpawnProjectile>,
    library: Res<SpriteLibrary>,
    mut commands: Commands,
) {
    for request in requests.read() {
        let heading = request.velocity.normalize_or(Vec2::X);
        commands.spawn((
            library.animated(request.sprite, PROJECTILE_FPS),
            Transform::from_translation(request.position.extend(layer::PROJECTILE))
                .with_rotation(Quat::from_rotation_z(heading.to_angle())),
            Velocity(request.velocity),
            Projectile {
                team: request.team,
                attack: request.attack,
                radius: request.radius,
                lifetime: Timer::from_seconds(request.lifetime, TimerMode::Once),
                pierce: request.pierce,
                already_hit: Vec::new(),
                spent: false,
            },
            DespawnOnExit(GameState::InRun),
        ));
    }
}

/// Ages every projectile and lets it hit at most one new target per frame.
pub fn strike(
    time: Res<Time>,
    mut projectiles: Query<(&Transform, &Velocity, &mut Projectile)>,
    targets: Query<(Entity, &Transform, &Team, &Health, &Body, Has<Invulnerable>)>,
    mut damage: MessageWriter<DamageRequest>,
    mut fx: MessageWriter<FxRequest>,
) {
    for (transform, velocity, mut projectile) in &mut projectiles {
        projectile.lifetime.tick(time.delta());
        if projectile.spent || projectile.lifetime.is_finished() {
            continue;
        }
        let position = transform.translation.truncate();
        let victim = targets.iter().find(|(entity, target, team, health, body, invulnerable)| {
            **team != projectile.team
                && !health.is_dead()
                && !invulnerable
                && !projectile.already_hit.contains(entity)
                && target.translation.truncate().distance(position) < projectile.radius + body.radius
        });
        let Some((target, ..)) = victim else { continue };

        damage.write(DamageRequest {
            target,
            attack: projectile.attack,
            origin: position - velocity.normalize_or_zero() * IMPACT_SETBACK,
        });
        fx.write(FxRequest { kind: FxKind::HitSpark, position });
        if projectile.pierce > 0 {
            projectile.pierce -= 1;
            projectile.already_hit.push(target);
        } else {
            projectile.spent = true;
        }
    }
}

/// Removes projectiles that hit their last target or ran out of time.
pub fn fizzle(projectiles: Query<(Entity, &Projectile)>, mut commands: Commands) {
    for (entity, projectile) in &projectiles {
        if projectile.spent || projectile.lifetime.is_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}
