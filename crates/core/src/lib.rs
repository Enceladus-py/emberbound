//! # Emberbound core (shared kernel)
//!
//! The vocabulary every gameplay crate shares: states, schedule sets,
//! components, messages and resources. Gameplay crates (`world`, `combat`,
//! `monsters`, `ui`) depend on this crate and on the pure domain - **never on
//! each other**. They cooperate only through what is defined here.
//!
//! ## Who owns what
//!
//! | Concern | Owner |
//! |---|---|
//! | Camera, tile map, hero entity, hero movement and dash, integrating [`Velocity`]/[`Knockback`] with terrain collision, y-sorting, screen shake | `world` |
//! | Sword attack, projectiles, contact damage, resolving [`DamageRequest`], death, ember/heart pickups, the dragon companion, regeneration | `combat` |
//! | The run director, spawning and AI of monsters and elder dragons | `monsters` |
//! | HUD, title / boon / game-over / victory screens, announcements, damage numbers and other effects | `ui` |
//! | Reading devices into [`PlayerIntent`], run reset, XP and levels, applying boons, state transitions | the `emberbound` binary |
//!
//! ## Frame flow
//!
//! While [`RunPhase::Fighting`], `Update` runs the [`GameSet`]s in order:
//! `Input -> Ai -> Movement -> Combat -> Resolve -> Cleanup`.

mod components;
mod messages;
mod resources;
mod sprites;
mod state;

pub use components::*;
pub use messages::*;
pub use resources::*;
pub use sprites::*;
pub use state::*;

pub use emberbound_pixelart::SpriteId;

use bevy::prelude::*;

/// Z layers. World units are art pixels; the camera shows roughly
/// [`VIEW_HEIGHT`] of them vertically.
pub mod layer {
    pub const GROUND: f32 = 0.0;
    pub const SHADOW: f32 = 1.0;
    pub const PICKUP: f32 = 2.0;
    /// Base z of y-sorted things (actors, trees, rocks). See [`crate::YSort`].
    pub const ACTORS: f32 = 10.0;
    pub const PROJECTILE: f32 = 30.0;
    pub const FX: f32 = 40.0;
}

/// World units visible vertically.
pub const VIEW_HEIGHT: f32 = 216.0;

/// Registers the shared states, sets, messages and resources.
/// Must be added after Bevy's `DefaultPlugins`.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        state::register(app);
        messages::register(app);
        resources::register(app);
        sprites::register(app);
    }
}
