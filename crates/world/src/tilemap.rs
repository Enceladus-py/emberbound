//! The valley: the ground baked into a few large chunk images, water and
//! obstacles as sprites on top.
//!
//! A 72x72 valley would otherwise be more than five thousand ground
//! sprites, which phones feel every frame. Ground never changes during a
//! run, so it is drawn once into chunks on the CPU instead.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Anchor;
use emberbound_core::{GameState, RunSetup, SpriteId, SpriteLibrary, Terrain, layer};
use emberbound_domain::world::{TILE_SIZE, Tile, WorldMap};

use crate::ysort;

/// Frames per second of the water ripple.
const WATER_FPS: f32 = 2.5;
/// Edge length of a baked ground chunk, in tiles.
const CHUNK_TILES: i32 = 8;
/// Water ripples just above the baked ground.
const WATER_Z: f32 = layer::GROUND + 0.1;

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

/// Builds the ground chunks, water and obstacles of the fresh [`Terrain`].
fn spawn_tile_map(
    mut commands: Commands,
    terrain: Res<Terrain>,
    library: Res<SpriteLibrary>,
    mut images: ResMut<Assets<Image>>,
) {
    let map = &terrain.0;
    let scope = DespawnOnExit(GameState::InRun);
    let (width, height) = (map.width() as i32, map.height() as i32);

    for chunk_y in (0..height).step_by(CHUNK_TILES as usize) {
        for chunk_x in (0..width).step_by(CHUNK_TILES as usize) {
            let image = images.add(bake_chunk(map, &library, chunk_x, chunk_y));
            commands.spawn((
                Sprite::from_image(image),
                Anchor::BOTTOM_LEFT,
                Transform::from_xyz(chunk_x as f32 * TILE_SIZE, chunk_y as f32 * TILE_SIZE, layer::GROUND),
                scope.clone(),
            ));
        }
    }

    for y in 0..height {
        for x in 0..width {
            let tile = map.tile(x, y);
            let (cx, cy) = WorldMap::tile_center(x, y);
            if tile == Tile::Water {
                // Water frames are an animation; every tile ripples in step.
                commands.spawn((
                    library.animated(SpriteId::TileWater, WATER_FPS),
                    Transform::from_xyz(cx, cy, WATER_Z),
                    scope.clone(),
                ));
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

/// Draws the ground of the chunk whose bottom-left tile is `(chunk_x,
/// chunk_y)`. Tiles past the map edge stay transparent.
fn bake_chunk(map: &WorldMap, library: &SpriteLibrary, chunk_x: i32, chunk_y: i32) -> Image {
    let tile_px = TILE_SIZE as usize;
    let side = CHUNK_TILES as usize * tile_px;
    let mut pixels = vec![0u8; side * side * 4];
    for ty in 0..CHUNK_TILES {
        for tx in 0..CHUNK_TILES {
            let (x, y) = (chunk_x + tx, chunk_y + ty);
            if x >= map.width() as i32 || y >= map.height() as i32 {
                continue;
            }
            let tile = map.tile(x, y);
            let frame = library.pixels(ground_sprite(tile), tile_hash(x, y, 0));
            // Images are stored top row first while world y grows upwards.
            let top = (CHUNK_TILES - 1 - ty) as usize * tile_px;
            let left = tx as usize * tile_px;
            for row in 0..tile_px {
                let dst = ((top + row) * side + left) * 4;
                let src = row * tile_px * 4;
                pixels[dst..dst + tile_px * 4].copy_from_slice(&frame[src..src + tile_px * 4]);
            }
        }
    }
    Image::new(
        Extent3d { width: side as u32, height: side as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(OnEnter(GameState::InRun), spawn_tile_map.in_set(RunSetup::Spawn));
}
