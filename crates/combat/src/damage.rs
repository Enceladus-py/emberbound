//! Damage resolution: the only place where health is reduced and actors die.

use bevy::prelude::*;
use emberbound_core::{
    ActorDied, ActorKind, Announce, Armor, Body, DamageApplied, DamageRequest, Dragon, FxKind, FxRequest,
    GameRng, Health, HeroState, Invulnerable, Knockback, Monster, Player, ScreenShake, Team,
};
use emberbound_domain::combat::{AttackProfile, resolve_hit};
use emberbound_domain::ports::RandomSource;

/// Mercy window after the hero is hurt.
const HURT_INVULNERABILITY: f32 = 0.85;
const HURT_SHAKE: f32 = 0.25;
const DRAGON_DEATH_SHAKE: f32 = 0.9;
const DRAGON_DEATH_BURSTS: usize = 6;
/// Burst spread used when a dragon has no `Body` to take the radius from.
const DRAGON_BURST_RADIUS: f32 = 20.0;

/// Phoenix Feather: share of health a rebirth restores, how long the hero
/// is then untouchable, and the blast that clears space around them.
const REBIRTH_HEALTH: f32 = 0.5;
const REBIRTH_INVULNERABILITY: f32 = 2.0;
const REBIRTH_SHAKE: f32 = 0.8;
const REBIRTH_BURSTS: usize = 8;
const REBIRTH_BLAST_RADIUS: f32 = 80.0;
const REBIRTH_BLAST: AttackProfile = AttackProfile::flat(40.0, 260.0);

/// Rebirths spent this run, and a blast still owed from the latest one.
#[derive(Resource, Default)]
pub struct Rebirths {
    spent: u32,
    blast_at: Option<Vec2>,
}

pub fn reset_rebirths(mut rebirths: ResMut<Rebirths>) {
    *rebirths = Rebirths::default();
}

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
    hero_state: Res<HeroState>,
    mut rebirths: ResMut<Rebirths>,
    mut rng: ResMut<GameRng>,
    mut shake: ResMut<ScreenShake>,
    mut announce: MessageWriter<Announce>,
    mut victims: Query<Victim>,
    mut commands: Commands,
    mut applied: MessageWriter<DamageApplied>,
    mut died: MessageWriter<ActorDied>,
    mut fx: MessageWriter<FxRequest>,
) {
    // The hero's mercy window is inserted through `Commands`, so it is not
    // visible to later requests of this same frame; remember it here.
    let mut hero_hurt = false;
    // Sanguine Edge: health owed to the hero for critical hits this frame.
    let mut crit_healing = 0.0;
    let stats = &hero_state.stats;

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
        let mut fatal = health.damage(outcome.amount);
        if is_hero && fatal && rebirths.spent < stats.rebirths {
            // Phoenix Feather: the blow lands, then the hero rekindles.
            rebirths.spent += 1;
            rebirths.blast_at = Some(position);
            health.revive(REBIRTH_HEALTH);
            fatal = false;
            hero_hurt = true;
            shake.add(REBIRTH_SHAKE);
            commands.entity(request.target).try_insert(Invulnerable::seconds(REBIRTH_INVULNERABILITY));
            for _ in 0..REBIRTH_BURSTS {
                let offset = rng.direction() * rng.range(0.0, 18.0);
                fx.write(FxRequest { kind: FxKind::EmberBurst, position: position + offset });
            }
            announce.write(Announce {
                title: "REKINDLED".into(),
                subtitle: "The phoenix feather burns away".into(),
            });
        }
        if outcome.critical && !is_hero {
            crit_healing += stats.crit_heal;
        }
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

        if is_hero && !fatal && !hero_hurt {
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

    if crit_healing > 0.0
        && let Some((_, mut health, ..)) = victims.iter_mut().find(|victim| victim.6)
    {
        health.heal(crit_healing);
    }
}

/// The Phoenix Feather's blast: everything near the reborn hero is hurt and
/// thrown back, a frame after the rebirth.
pub fn rebirth_blast(
    mut rebirths: ResMut<Rebirths>,
    horde: Query<(Entity, &Transform, &Team), With<Health>>,
    mut damage: MessageWriter<DamageRequest>,
) {
    let Some(origin) = rebirths.blast_at.take() else { return };
    for (target, transform, team) in &horde {
        if *team == Team::Horde && transform.translation.truncate().distance(origin) <= REBIRTH_BLAST_RADIUS {
            damage.write(DamageRequest { target, attack: REBIRTH_BLAST, origin });
        }
    }
}

/// Despawns everything that died this frame.
pub fn bury(dying: Query<Entity, With<Dying>>, mut commands: Commands) {
    for entity in &dying {
        commands.entity(entity).try_despawn();
    }
}
