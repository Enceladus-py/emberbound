//! Touching the horde hurts.

use bevy::prelude::*;
use emberbound_core::{Body, ContactDamage, DamageRequest, Health, HeroState, Invulnerable, Player, Team};
use emberbound_domain::combat::AttackProfile;

/// Impulse that shoves the hero away from whatever touched them.
const CONTACT_KNOCKBACK: f32 = 170.0;

/// Needs no cooldown of its own: a hurt hero is briefly invulnerable, and
/// invulnerable targets ignore damage requests. Thornhide strikes back only
/// when the touch can actually hurt, so it shares that rhythm.
pub fn touch(
    hero: Single<(Entity, &Transform, &Body, &Health, Has<Invulnerable>), With<Player>>,
    hero_state: Res<HeroState>,
    horde: Query<(Entity, &Transform, &Team, &ContactDamage, &Body, &Health), Without<Player>>,
    mut damage: MessageWriter<DamageRequest>,
) {
    let (target, hero_transform, hero_body, hero_health, shielded) = *hero;
    let thorns = hero_state.stats.thorns;
    if hero_health.is_dead() {
        return;
    }
    let hero_position = hero_transform.translation.truncate();
    for (attacker, transform, team, contact, body, health) in &horde {
        if *team != Team::Horde || health.is_dead() {
            continue;
        }
        let origin = transform.translation.truncate();
        if origin.distance(hero_position) < body.radius + hero_body.radius {
            damage.write(DamageRequest {
                target,
                attack: AttackProfile::flat(contact.0, CONTACT_KNOCKBACK),
                origin,
            });
            if thorns > 0.0 && !shielded {
                damage.write(DamageRequest {
                    target: attacker,
                    attack: AttackProfile::flat(thorns, CONTACT_KNOCKBACK * 0.5),
                    origin: hero_position,
                });
            }
        }
    }
}
