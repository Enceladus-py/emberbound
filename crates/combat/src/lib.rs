//! Combat adapter: attacks, projectiles, damage, pickups and the companion.
//!
//! Every concern lives in its own module and talks to the others only through
//! core's messages: attackers write [`emberbound_core::DamageRequest`],
//! [`damage`] settles them, [`loot`] reacts to the resulting deaths.
//!
//! Frame order inside this crate:
//! `aim -> sword -> companion -> pickup motion -> projectiles / contact ->
//! invulnerability, damage, loot, collection, regeneration -> burial`.

mod aim;
mod companion;
mod contact;
mod damage;
mod loot;
mod projectile;
mod regen;
mod sword;
mod wake;
mod wildfire;

use bevy::prelude::*;
use emberbound_core::{GameSet, GameState, RunSetup};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<sword::SwingCooldown>()
            .init_resource::<damage::Rebirths>()
            .init_resource::<wake::Trail>()
            .add_systems(
                OnEnter(GameState::InRun),
                (
                    (sword::reset, damage::reset_rebirths, wake::reset).in_set(RunSetup::Reset),
                    companion::spawn.in_set(RunSetup::Spawn),
                ),
            )
            .add_systems(
                Update,
                (
                    (aim::aim, sword::swing).chain().in_set(GameSet::Input),
                    (companion::grow, companion::follow, companion::aim_and_fire).chain().in_set(GameSet::Ai),
                    loot::drift.in_set(GameSet::Movement),
                    (projectile::launch, projectile::strike).chain().in_set(GameSet::Combat),
                    (contact::touch, damage::rebirth_blast, wake::lay_trail, wake::burn)
                        .in_set(GameSet::Combat),
                    (
                        damage::tick_invulnerability,
                        damage::resolve,
                        loot::drop_loot,
                        wildfire::burst_corpses,
                        loot::collect,
                        regen::regenerate,
                    )
                        .chain()
                        .in_set(GameSet::Resolve),
                    (damage::bury, projectile::fizzle).in_set(GameSet::Cleanup),
                ),
            );
    }
}
