//! The valley: one ground sprite per tile, obstacles on top.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use emberbound_core::{GameState, RunSetup, SpriteId, SpriteLibrary, Terrain, layer};
use emberbound_domain::world::{TILE_SIZE, Tile, WorldMap};

use crate::ysort;

/// Frames per second of the water ripple.
const WATER_FPS: f32 = 2.5;

/// The ground sprite under a tile. Obstacles stand on grass.
fn ground_sprite(tile: Tile) -> SpriteId {
    match tile {
        Tile::Grass | Tile::Tree | Tile::Rock | Tile::Ruin => SpriteId::TileGrass,
        Tile::TallGrass => SpriteId::TileTallGrass,
        Tile::Flowers => SpriteId::TileFlowers,
        Tile::Dirt => SpriteId::TileDirt,
        Tile::Sand => SpriteId::TileSand,
        Tile::Water => SpriteId::TileWater,
    }
}

/// The sprite standing on a tile, if it is an obstacle.
fn obstacle_sprite(tile: Tile) -> Option<SpriteId> {
    match tile {
        Tile::Tree => Some(SpriteId::Tree),
        Tile::Rock => Some(SpriteId::Rock),
        Tile::Ruin => Some(SpriteId::Ruin),
        _ => None,
    }
}

/// Deterministic, well-mixed hash of a tile position; picks sprite variants
/// so a map always looks the same without consuming the game's randomness.
/// `salt` decorrelates the ground from the obstacle standing on it.
fn tile_hash(x: i32, y: i32, salt: u32) -> usize {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    h as usize
}

/// Builds the sprites of the freshly generated [`Terrain`].
fn spawn_tile_map(mut commands: Commands, terrain: Res<Terrain>, library: Res<SpriteLibrary>) {
    let map = &terrain.0;
    let scope = DespawnOnExit(GameState::InRun);
    for y in 0..map.height() as i32 {
        for x in 0..map.width() as i32 {
            let tile = map.tile(x, y);
            let (cx, cy) = WorldMap::tile_center(x, y);
            let ground = Transform::from_xyz(cx, cy, layer::GROUND);
            if tile == Tile::Water {
                // Water frames are an animation; every tile ripples in step.
                commands.spawn((library.animated(SpriteId::TileWater, WATER_FPS), ground, scope.clone()));
            } else {
                let sprite = library.sprite_frame(ground_sprite(tile), tile_hash(x, y, 0));
                commands.spawn((sprite, ground, scope.clone()));
            }

            if let Some(id) = obstacle_sprite(tile) {
                // Anchored by its foot to the bottom edge of the tile and
                // depth-sorted once: it never moves, so it needs no `YSort`.
                let foot = cy - TILE_SIZE * 0.5;
                commands.spawn((
                    library.sprite_frame(id, tile_hash(x, y, 1)),
                    Anchor::BOTTOM_CENTER,
                    Transform::from_xyz(cx, foot, ysort::depth(foot)),
                    scope.clone(),
                ));
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(OnEnter(GameState::InRun), spawn_tile_map.in_set(RunSetup::Spawn));
}
