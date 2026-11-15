//! World adapter: camera, tile map, the hero and movement.
//!
//! One module per concern:
//!
//! - [`camera`]: the persistent main camera, hero follow and screen shake.
//! - [`tilemap`]: turns the [`emberbound_core::Terrain`] into sprites.
//! - [`hero`]: spawns the hero and turns player intent into motion and dashes.
//! - [`movement`]: integrates velocities and resolves terrain collision.
//! - [`ysort`]: draws lower things in front of higher ones.

mod camera;
mod hero;
mod movement;
mod tilemap;
mod ysort;

use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        camera::register(app);
        tilemap::register(app);
        hero::register(app);
        movement::register(app);
        ysort::register(app);
    }
}
