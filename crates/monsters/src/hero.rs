//! How this crate looks at the hero (who is owned by `world`).

use bevy::prelude::*;
use emberbound_core::{Dragon, Health, Monster, Player};

/// The hero's transform. The `Without` filters keep it disjoint from every
/// query that mutates monster or dragon transforms.
pub(crate) type HeroPos<'w, 's> =
    Single<'w, 's, &'static Transform, (With<Player>, Without<Monster>, Without<Dragon>)>;

/// Run condition: there is a hero and they are still standing.
pub(crate) fn hero_alive(hero: Query<&Health, With<Player>>) -> bool {
    hero.single().is_ok_and(|health| !health.is_dead())
}
