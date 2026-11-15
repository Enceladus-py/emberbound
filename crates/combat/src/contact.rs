//! Touching the horde hurts.

use bevy::prelude::*;
use emberbound_core::{Body, ContactDamage, DamageRequest, Health, Player, Team};
use emberbound_domain::combat::AttackProfile;

/// Impulse that shoves the hero away from whatever touched them.
const CONTACT_KNOCKBACK: f32 = 170.0;

/// Needs no cooldown of its own: a hurt hero is briefly invulnerable, and
/// invulnerable targets ignore damage requests.
pub fn touch(
    hero: Single<(Entity, &Transform, &Body, &Health), With<Player>>,
    horde: Query<(&Transform, &Team, &ContactDamage, &Body, &Health), Without<Player>>,
    mut damage: MessageWriter<DamageRequest>,
) {
    let (target, hero_transform, hero_body, hero_health) = *hero;
    if hero_health.is_dead() {
        return;
    }
    let hero_position = hero_transform.translation.truncate();
    for (transform, team, contact, body, health) in &horde {
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
        }
    }
}
