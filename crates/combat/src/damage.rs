//! Damage resolution: the only place where health is reduced and actors die.

use bevy::prelude::*;
use emberbound_core::{
    ActorDied, ActorKind, Armor, Body, DamageApplied, DamageRequest, Dragon, FxKind, FxRequest, GameRng,
    Health, Invulnerable, Knockback, Monster, Player, ScreenShake,
};
use emberbound_domain::combat::resolve_hit;
use emberbound_domain::ports::RandomSource;

/// Mercy window after the hero is hurt.
const HURT_INVULNERABILITY: f32 = 0.6;
const HURT_SHAKE: f32 = 0.25;
const DRAGON_DEATH_SHAKE: f32 = 0.9;
const DRAGON_DEATH_BURSTS: usize = 6;
/// Burst spread used when a dragon has no `Body` to take the radius from.
const DRAGON_BURST_RADIUS: f32 = 20.0;

/// A slain monster or dragon, despawned at the end of the frame it died in.
/// Until then its components stay readable (loot needs the rewards).
#[derive(Component)]
pub struct Dying;

type Victim = (
    &'static Transform,
    &'static mut Health,
    Option<&'static Armor>,
    Option<&'static mut Knockback>,
    Option<&'static Body>,
    Has<Invulnerable>,
    Has<Player>,
    Option<&'static Monster>,
    Option<&'static Dragon>,
);

/// Counts down every invulnerability window, including the ones `world`
/// grants while dashing.
pub fn tick_invulnerability(
    time: Res<Time>,
    mut shielded: Query<(Entity, &mut Invulnerable)>,
    mut commands: Commands,
) {
    for (entity, mut invulnerable) in &mut shielded {
        if invulnerable.0.tick(time.delta()).is_finished() {
            commands.entity(entity).try_remove::<Invulnerable>();
        }
    }
}

/// Applies every pending [`DamageRequest`]. Requests against anything that is
/// gone, dead, invulnerable or not an actor (hero, monster, dragon) are dropped.
pub fn resolve(
    mut requests: MessageReader<DamageRequest>,
    mut rng: ResMut<GameRng>,
    mut shake: ResMut<ScreenShake>,
    mut victims: Query<Victim>,
    mut commands: Commands,
    mut applied: MessageWriter<DamageApplied>,
    mut died: MessageWriter<ActorDied>,
    mut fx: MessageWriter<FxRequest>,
) {
    // The hero's mercy window is inserted through `Commands`, so it is not
    // visible to later requests of this same frame; remember it here.
    let mut hero_hurt = false;

    for request in requests.read() {
        let Ok((transform, mut health, armor, knockback, body, invulnerable, is_hero, monster, dragon)) =
            victims.get_mut(request.target)
        else {
            continue;
        };
        if health.is_dead() || invulnerable || (is_hero && hero_hurt) {
            continue;
        }
        let kind = match (is_hero, monster, dragon) {
            (true, ..) => ActorKind::Hero,
            (_, Some(monster), _) => ActorKind::Monster(monster.kind),
            (_, _, Some(dragon)) => ActorKind::Dragon(dragon.kind),
            _ => continue,
        };

        let position = transform.translation.truncate();
        let outcome = resolve_hit(&request.attack, armor.map_or(0.0, |armor| armor.0), &mut *rng);
        let fatal = health.damage(outcome.amount);
        if let Some(mut knockback) = knockback {
            knockback.0 += (position - request.origin).normalize_or_zero() * request.attack.knockback;
        }
        applied.write(DamageApplied {
            target: request.target,
            victim: kind,
            amount: outcome.amount,
            critical: outcome.critical,
            position,
            fatal,
        });

        if is_hero && !fatal {
            hero_hurt = true;
            shake.add(HURT_SHAKE);
            commands.entity(request.target).try_insert(Invulnerable::seconds(HURT_INVULNERABILITY));
        }
        if !fatal {
            continue;
        }

        died.write(ActorDied { entity: request.target, kind, position });
        match kind {
            ActorKind::Hero => {}
            ActorKind::Monster(_) => {
                fx.write(FxRequest { kind: FxKind::DeathPuff, position });
                commands.entity(request.target).try_insert(Dying);
            }
            ActorKind::Dragon(_) => {
                let radius = body.map_or(DRAGON_BURST_RADIUS, |body| body.radius);
                for _ in 0..DRAGON_DEATH_BURSTS {
                    let offset = rng.direction() * rng.range(0.0, radius);
                    fx.write(FxRequest { kind: FxKind::EmberBurst, position: position + offset });
                }
                shake.add(DRAGON_DEATH_SHAKE);
                commands.entity(request.target).try_insert(Dying);
            }
        }
    }
}

/// Despawns everything that died this frame.
pub fn bury(dying: Query<Entity, With<Dying>>, mut commands: Commands) {
    for entity in &dying {
        commands.entity(entity).try_despawn();
    }
}
