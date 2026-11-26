//! The ground: baked into small chunk images, with water and obstacles as
//! sprites on top, streamed in and out around the camera.
//!
//! Drawing every tile as a sprite would be thousands of sprites, which
//! phones feel every frame. Ground never changes during a run, so each
//! chunk is drawn once on the CPU when it comes into view and dropped
//! (with its water and obstacles) once it is well out of view. The chunks
//! of terrain this reads are kept loaded around the hero by
//! [`keep_terrain_loaded`].

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Anchor;
use emberbound_core::{
    GameSet, GameState, MainCamera, Player, RunSetup, SpriteId, SpriteLibrary, Terrain, VIEW_HEIGHT, layer,
};
use emberbound_domain::world::{TILE_SIZE, Tile, TileSource, WorldMap};

use crate::camera;
use crate::ysort::{DepthOrigin, StaticDepth};

/// Frames per second of the water ripple.
const WATER_FPS: f32 = 2.5;
/// Edge length of a baked ground chunk, in tiles.
const CHUNK_TILES: i32 = 8;
/// Water ripples just above the baked ground.
const WATER_Z: f32 = layer::GROUND + 0.1;
/// Chunks drawn beyond the edge of the view on every side, so walking
/// never reveals a gap.
const DRAW_MARGIN: i32 = 1;
/// Chunks are dropped only this far beyond the view, so walking back and
/// forth over a chunk border does not bake the same chunk again and again.
const KEEP_MARGIN: i32 = 2;
/// Half the widest view the camera is assumed to show before it has
/// worked out its real area (an ultra-wide 2.4:1 window).
const FALLBACK_HALF_VIEW: Vec2 = Vec2::new(VIEW_HEIGHT * 1.2, VIEW_HEIGHT * 0.5);

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

/// The chunks drawn right now, with the entities that make up each one.
#[derive(Resource, Debug, Default)]
struct ShownGround(HashMap<(i32, i32), Vec<Entity>>);

/// Loads the terrain around the hero before anything reads it this frame.
fn keep_terrain_loaded(hero: Query<&Transform, With<Player>>, mut terrain: ResMut<Terrain>) {
    let Ok(hero) = hero.single() else { return };
    let position = hero.translation.truncate();
    // Checked through `Deref` first so the terrain is only marked changed
    // when the hero actually reaches new ground.
    if !terrain.is_loaded_around(position) {
        terrain.load_around(position);
    }
}

/// The chunks (in [`CHUNK_TILES`] units) overlapping a world rectangle
/// grown by `margin` chunks on every side.
fn chunks_over(area: Rect, margin: i32) -> (IVec2, IVec2) {
    let chunk = CHUNK_TILES as f32 * TILE_SIZE;
    let min = (area.min / chunk).floor().as_ivec2() - IVec2::splat(margin);
    let max = (area.max / chunk).floor().as_ivec2() + IVec2::splat(margin);
    (min, max)
}

/// The world area the camera shows.
fn view_area(camera: Option<(&Transform, &Projection)>, fallback_centre: Vec2) -> Rect {
    let Some((transform, projection)) = camera else {
        return Rect::from_center_half_size(fallback_centre, FALLBACK_HALF_VIEW);
    };
    let centre = transform.translation.truncate();
    match projection {
        Projection::Orthographic(ortho) if ortho.area.width() > 0.0 => {
            Rect::from_corners(centre + ortho.area.min, centre + ortho.area.max)
        }
        _ => Rect::from_center_half_size(centre, FALLBACK_HALF_VIEW),
    }
}

/// Bakes the chunks coming into view and drops those well out of it.
fn stream_ground(
    mut commands: Commands,
    terrain: Res<Terrain>,
    library: Res<SpriteLibrary>,
    depth: Res<DepthOrigin>,
    mut images: ResMut<Assets<Image>>,
    mut shown: ResMut<ShownGround>,
    camera: Query<(&Transform, &Projection), With<MainCamera>>,
) {
    let tiles = terrain.tiles();
    let view = view_area(camera.single().ok(), Vec2::from(tiles.spawn_point()));

    let (keep_min, keep_max) = chunks_over(view, KEEP_MARGIN);
    shown.0.retain(|&(x, y), entities| {
        let keep = (keep_min.x..=keep_max.x).contains(&x) && (keep_min.y..=keep_max.y).contains(&y);
        if !keep {
            for entity in entities.drain(..) {
                commands.entity(entity).despawn();
            }
        }
        keep
    });

    let (min, max) = chunks_over(view, DRAW_MARGIN);
    for chunk_y in min.y..=max.y {
        for chunk_x in min.x..=max.x {
            if let Entry::Vacant(slot) = shown.0.entry((chunk_x, chunk_y)) {
                slot.insert(spawn_chunk(
                    &mut commands,
                    tiles,
                    &library,
                    &depth,
                    &mut images,
                    chunk_x,
                    chunk_y,
                ));
            }
        }
    }
}

/// Spawns the ground image, water and obstacles of one chunk.
fn spawn_chunk(
    commands: &mut Commands,
    tiles: &dyn TileSource,
    library: &SpriteLibrary,
    depth: &DepthOrigin,
    images: &mut Assets<Image>,
    chunk_x: i32,
    chunk_y: i32,
) -> Vec<Entity> {
    let scope = DespawnOnExit(GameState::InRun);
    let (left, bottom) = (chunk_x * CHUNK_TILES, chunk_y * CHUNK_TILES);
    let image = images.add(bake_chunk(tiles, library, left, bottom));
    let mut entities = vec![
        commands
            .spawn((
                Sprite::from_image(image),
                Anchor::BOTTOM_LEFT,
                Transform::from_xyz(left as f32 * TILE_SIZE, bottom as f32 * TILE_SIZE, layer::GROUND),
                scope.clone(),
            ))
            .id(),
    ];

    for y in bottom..bottom + CHUNK_TILES {
        for x in left..left + CHUNK_TILES {
            let tile = tiles.tile(x, y);
            let (cx, cy) = WorldMap::tile_center(x, y);
            if tile == Tile::Water {
                // Water frames are an animation; every tile ripples in step.
                entities.push(
                    commands
                        .spawn((
                            library.animated(SpriteId::TileWater, WATER_FPS),
                            Transform::from_xyz(cx, cy, WATER_Z),
                            scope.clone(),
                        ))
                        .id(),
                );
            }
            if let Some(id) = obstacle_sprite(tile) {
                // Anchored by its foot to the bottom edge of the tile. It
                // never moves, so it is depth-sorted only when spawned or
                // when the depth origin moves.
                let foot = cy - TILE_SIZE * 0.5;
                entities.push(
                    commands
                        .spawn((
                            library.sprite_frame(id, tile_hash(x, y, 1)),
                            Anchor::BOTTOM_CENTER,
                            Transform::from_xyz(cx, foot, depth.depth(foot)),
                            StaticDepth,
                            scope.clone(),
                        ))
                        .id(),
                );
            }
        }
    }
    entities
}

/// Draws the ground of the chunk whose bottom-left tile is `(left,
/// bottom)`.
fn bake_chunk(tiles: &dyn TileSource, library: &SpriteLibrary, left: i32, bottom: i32) -> Image {
    let tile_px = TILE_SIZE as usize;
    let side = CHUNK_TILES as usize * tile_px;
    let mut pixels = vec![0u8; side * side * 4];
    for ty in 0..CHUNK_TILES {
        for tx in 0..CHUNK_TILES {
            let (x, y) = (left + tx, bottom + ty);
            let frame = library.pixels(ground_sprite(tiles.tile(x, y)), tile_hash(x, y, 0));
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

fn forget_ground(mut shown: ResMut<ShownGround>) {
    // The entities themselves went with the previous run's state.
    shown.0.clear();
}

pub(crate) fn register(app: &mut App) {
    app.init_resource::<ShownGround>()
        .add_systems(OnEnter(GameState::InRun), forget_ground.in_set(RunSetup::Reset))
        .add_systems(
            OnEnter(GameState::InRun),
            stream_ground.in_set(RunSetup::Spawn).after(camera::snap_to_start),
        )
        .add_systems(
            Update,
            (
                keep_terrain_loaded.after(GameSet::Movement).before(GameSet::Combat),
                stream_ground.after(camera::apply_shake),
            )
                .run_if(in_state(GameState::InRun)),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_over_covers_the_area_and_its_margin() {
        let chunk = CHUNK_TILES as f32 * TILE_SIZE;
        let area = Rect::new(-10.0, 5.0, chunk + 1.0, chunk - 1.0);
        assert_eq!(chunks_over(area, 0), (IVec2::new(-1, 0), IVec2::new(1, 0)));
        assert_eq!(chunks_over(area, 2), (IVec2::new(-3, -2), IVec2::new(3, 2)));
    }

    #[test]
    fn the_view_falls_back_to_a_wide_area_around_the_start() {
        let area = view_area(None, Vec2::new(100.0, 50.0));
        assert_eq!(area.center(), Vec2::new(100.0, 50.0));
        assert_eq!(area.half_size(), FALLBACK_HALF_VIEW);
    }
}
