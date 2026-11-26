//! The minimap in the top-right corner: the ground baked into a tiny image,
//! with live markers for the hero, the active dragon, nearby monsters and
//! the camera's view. `M` toggles it.
//!
//! What area it shows is decided in one place, [`shown_area`]: a whole
//! valley, or on the endless world a window around the hero that is
//! re-baked whenever the hero has walked a few tiles.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use emberbound_core::{Dragon, GameState, MainCamera, Monster, Player, PlayerIntent, Terrain};
use emberbound_domain::world::{TILE_SIZE, Tile, TileSource, WorldMap};

use crate::theme;

/// The HUD node the minimap is drawn into, under the run facts.
#[derive(Component)]
pub(crate) struct MinimapSlot;

/// Longest edge of the map, in design pixels.
const MAP_EDGE: f32 = 148.0;
/// Most monsters drawn at once; the rest are left off.
const MONSTER_MARKERS: usize = 48;
const HERO_MARKER: f32 = 6.0;
const DRAGON_MARKER: f32 = 10.0;
const MONSTER_MARKER: f32 = 3.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<MinimapShown>().add_systems(
        Update,
        (spawn_minimap, toggle, follow_hero, place_markers).chain().run_if(in_state(GameState::InRun)),
    );
}

/// Whether the player wants the minimap; kept across runs.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MinimapShown(pub(crate) bool);

impl Default for MinimapShown {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component)]
struct MinimapRoot;

/// A dot or frame on the map, placed every frame by [`place_markers`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Marker {
    Hero,
    Dragon,
    /// One of the pooled monster dots.
    Monster(usize),
    View,
}

/// Tiles across the window shown on an endless world. Even with the window
/// snapped up to two tiles off the hero, it stays inside the 32 tiles kept
/// loaded on every side of them.
const WINDOW_TILES: i32 = 60;
/// The window moves in steps of this many tiles, so it is re-baked every
/// few steps of the hero rather than on every tile.
const WINDOW_STEP: i32 = 4;

/// The part of the world the minimap shows, in world units: the whole of a
/// finite map, or a window around the hero on an endless one.
fn shown_area(tiles: &dyn TileSource, hero: Vec2) -> Rect {
    if let Some((width, height)) = tiles.bounds() {
        return Rect::new(0.0, 0.0, width, height);
    }
    let (hx, hy) = WorldMap::tile_coords(hero.x, hero.y);
    let snap = |v: i32| (v + WINDOW_STEP / 2).div_euclid(WINDOW_STEP) * WINDOW_STEP - WINDOW_TILES / 2;
    let min = Vec2::new(snap(hx) as f32, snap(hy) as f32) * TILE_SIZE;
    Rect::from_corners(min, min + Vec2::splat(WINDOW_TILES as f32 * TILE_SIZE))
}

/// Where a world position lands on the minimap, as fractions from its
/// top-left corner. World y grows up, UI y grows down.
fn to_minimap(area: Rect, world: Vec2) -> Vec2 {
    let size = area.size().max(Vec2::ONE);
    Vec2::new((world.x - area.min.x) / size.x, (area.max.y - world.y) / size.y)
}

/// The minimap's colour for a tile.
fn tile_color(tile: Tile) -> [u8; 4] {
    match tile {
        Tile::Grass | Tile::Flowers => [0x5a, 0xa8, 0x4e, 0xff],
        Tile::TallGrass => [0x4f, 0x9c, 0x4a, 0xff],
        Tile::Dirt => [0x8a, 0x5a, 0x3a, 0xff],
        Tile::Sand => [0xdc, 0xc0, 0x84, 0xff],
        Tile::Water => [0x3b, 0x6c, 0xba, 0xff],
        Tile::Tree => [0x1e, 0x54, 0x40, 0xff],
        Tile::Rock | Tile::Ruin => [0x73, 0x73, 0x8a, 0xff],
    }
}

/// The tiles of `area` at one pixel per tile, top row first.
fn bake(tiles: &dyn TileSource, area: Rect) -> Image {
    let (left, bottom) = WorldMap::tile_coords(area.min.x + 0.5, area.min.y + 0.5);
    let size = (area.size() / TILE_SIZE).round().as_uvec2().max(UVec2::ONE);
    let (width, height) = (size.x, size.y);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height as i32 {
        let y = bottom + height as i32 - 1 - row;
        for x in left..left + width as i32 {
            pixels.extend(tile_color(tiles.tile(x, y)));
        }
    }
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// The image the ground is baked into, and the area it currently shows.
#[derive(Component)]
struct Canvas {
    image: Handle<Image>,
    area: Rect,
}

/// Bakes the ground and draws the map into the HUD's slot, once per run
/// (the HUD, and so the slot, is spawned anew for every run).
fn spawn_minimap(
    mut commands: Commands,
    terrain: Res<Terrain>,
    shown: Res<MinimapShown>,
    mut images: ResMut<Assets<Image>>,
    slot: Query<Entity, Added<MinimapSlot>>,
    hero: Query<&Transform, With<Player>>,
) {
    let Ok(slot) = slot.single() else { return };
    let tiles = terrain.tiles();
    let hero =
        hero.single().map_or_else(|_| Vec2::from(tiles.spawn_point()), |hero| hero.translation.truncate());
    let area = shown_area(tiles, hero);
    let aspect = area.width() / area.height().max(1.0);
    let size = if aspect >= 1.0 {
        Vec2::new(MAP_EDGE, MAP_EDGE / aspect)
    } else {
        Vec2::new(MAP_EDGE * aspect, MAP_EDGE)
    };
    let image = images.add(bake(tiles, area));

    let root = commands
        .spawn((
            MinimapRoot,
            Pickable::IGNORE,
            visibility(shown.0),
            Node { border: px(3).all(), ..default() },
            BorderColor::all(theme::PANEL_BORDER),
            BackgroundColor(theme::PANEL),
            ChildOf(slot),
        ))
        .id();
    let canvas = commands
        .spawn((
            ImageNode::new(image.clone()).with_color(Color::WHITE.with_alpha(0.9)),
            Canvas { image, area },
            Node { width: px(size.x), height: px(size.y), overflow: Overflow::clip(), ..default() },
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();

    let mut marker = |marker: Marker, size: f32, color: Color, round: bool| {
        commands.spawn((
            marker,
            Node {
                position_type: PositionType::Absolute,
                width: px(size),
                height: px(size),
                margin: UiRect::new(px(-size / 2.0), px(0), px(-size / 2.0), px(0)),
                border_radius: if round { BorderRadius::MAX } else { BorderRadius::ZERO },
                ..default()
            },
            BackgroundColor(color),
            Visibility::Hidden,
            Pickable::IGNORE,
            ChildOf(canvas),
        ));
    };
    for index in 0..MONSTER_MARKERS {
        marker(Marker::Monster(index), MONSTER_MARKER, theme::HEALTH.with_alpha(0.55), false);
    }
    marker(Marker::Dragon, DRAGON_MARKER, theme::FLAME, true);
    marker(Marker::Hero, HERO_MARKER, theme::PARCHMENT, true);
    commands.spawn((
        Marker::View,
        Node { position_type: PositionType::Absolute, border: px(1).all(), ..default() },
        BorderColor::all(theme::PARCHMENT.with_alpha(0.6)),
        Visibility::Hidden,
        Pickable::IGNORE,
        ChildOf(canvas),
    ));
}

/// Re-bakes the map once the window around the hero has moved on.
fn follow_hero(
    shown: Res<MinimapShown>,
    terrain: Res<Terrain>,
    hero: Query<&Transform, With<Player>>,
    mut canvas: Query<&mut Canvas>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Ok(hero), Ok(mut canvas)) = (hero.single(), canvas.single_mut()) else { return };
    if !shown.0 {
        return;
    }
    let area = shown_area(terrain.tiles(), hero.translation.truncate());
    if area == canvas.area {
        return;
    }
    if let Some(mut image) = images.get_mut(&canvas.image) {
        *image = bake(terrain.tiles(), area);
    }
    canvas.area = area;
}

fn visibility(shown: bool) -> Visibility {
    if shown { Visibility::Inherited } else { Visibility::Hidden }
}

fn toggle(
    intent: Res<PlayerIntent>,
    mut shown: ResMut<MinimapShown>,
    mut root: Query<&mut Visibility, With<MinimapRoot>>,
) {
    if intent.minimap {
        shown.0 = !shown.0;
    }
    for mut root in &mut root {
        root.set_if_neq(visibility(shown.0));
    }
}

fn place_markers(
    time: Res<Time>,
    shown: Res<MinimapShown>,
    terrain: Res<Terrain>,
    hero: Query<&Transform, With<Player>>,
    dragons: Query<&Transform, With<Dragon>>,
    monsters: Query<&Transform, With<Monster>>,
    camera: Query<(&Transform, &Projection), With<MainCamera>>,
    mut markers: Query<(&Marker, &mut Node, &mut Visibility, Option<&mut BackgroundColor>)>,
) {
    if !shown.0 {
        return;
    }
    let hero = hero.single().ok().map(|transform| transform.translation.truncate());
    let area = shown_area(terrain.tiles(), hero.unwrap_or_default());
    let dragon = dragons.iter().next().map(|transform| transform.translation.truncate());
    let mut nearby: Vec<Vec2> = monsters.iter().map(|transform| transform.translation.truncate()).collect();
    if let Some(hero) = hero {
        // With more monsters than dots, the closest ones matter most.
        nearby.sort_by(|a, b| a.distance_squared(hero).total_cmp(&b.distance_squared(hero)));
    }
    let view = camera.single().ok().and_then(|(transform, projection)| match projection {
        Projection::Orthographic(ortho) => {
            let centre = transform.translation.truncate();
            Some(Rect::from_corners(centre + ortho.area.min, centre + ortho.area.max))
        }
        _ => None,
    });
    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * 6.0).sin();

    for (marker, mut node, mut visibility, color) in &mut markers {
        let at = match *marker {
            Marker::Hero => hero,
            Marker::Dragon => dragon,
            Marker::Monster(index) => nearby.get(index).copied(),
            Marker::View => {
                let shown = view.map(|view| {
                    let (min, max) = (to_minimap(area, view.min), to_minimap(area, view.max));
                    let (left, top) = (min.x.max(0.0), max.y.max(0.0));
                    let (right, bottom) = (max.x.min(1.0), min.y.min(1.0));
                    (left, top, (right - left).max(0.0), (bottom - top).max(0.0))
                });
                visibility.set_if_neq(if shown.is_some() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
                if let Some((left, top, width, height)) = shown {
                    set_rect(&mut node, left, top, Some((width, height)));
                }
                continue;
            }
        };
        let inside = at
            .map(|at| to_minimap(area, at))
            .filter(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y));
        visibility.set_if_neq(if inside.is_some() { Visibility::Inherited } else { Visibility::Hidden });
        let Some(spot) = inside else { continue };
        set_rect(&mut node, spot.x, spot.y, None);
        if *marker == Marker::Dragon
            && let Some(mut color) = color
        {
            color.0 = theme::FLAME.mix(&theme::XP, pulse * 0.6);
            let size = DRAGON_MARKER * (1.0 + 0.3 * pulse);
            node.width = px(size);
            node.height = px(size);
            node.margin = UiRect::new(px(-size / 2.0), px(0), px(-size / 2.0), px(0));
        }
    }
}

/// Moves a marker to fractions of the map, touching the node only when
/// something changed.
fn set_rect(node: &mut Mut<Node>, left: f32, top: f32, size: Option<(f32, f32)>) {
    let (left, top) = (percent(left * 100.0), percent(top * 100.0));
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
    if let Some((width, height)) = size {
        let (width, height) = (percent(width * 100.0), percent(height * 100.0));
        if node.width != width || node.height != height {
            node.width = width;
            node.height = height;
        }
    }
}

#[cfg(test)]
mod tests {
    use emberbound_core::GameRng;
    use emberbound_domain::chunk::ChunkedWorld;
    use emberbound_domain::world;

    use super::*;

    fn map() -> WorldMap {
        world::generate(40, 24, &mut GameRng::new(3))
    }

    #[test]
    fn the_whole_valley_is_shown() {
        let map = map();
        let area = shown_area(&map, Vec2::ZERO);
        assert_eq!(area, Rect::new(0.0, 0.0, 40.0 * TILE_SIZE, 24.0 * TILE_SIZE));
        assert_eq!(shown_area(&map, Vec2::splat(9_999.0)), area, "wherever the hero is");
    }

    #[test]
    fn world_y_up_is_minimap_up() {
        let area = shown_area(&map(), Vec2::ZERO);
        assert_eq!(to_minimap(area, Vec2::ZERO), Vec2::new(0.0, 1.0), "bottom-left");
        assert_eq!(to_minimap(area, area.max), Vec2::new(1.0, 0.0), "top-right");
        assert_eq!(to_minimap(area, area.center()), Vec2::splat(0.5));
    }

    #[test]
    fn the_baked_image_matches_the_terrain() {
        let map = map();
        let image = bake(&map, shown_area(&map, Vec2::ZERO));
        let data = image.data.as_ref().unwrap();
        let (width, height) = (map.width() as i32, map.height() as i32);
        assert_eq!(data.len(), (width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                // Image rows run top to bottom, world rows bottom to top.
                let index = (((height - 1 - y) * width + x) * 4) as usize;
                assert_eq!(&data[index..index + 4], &tile_color(map.tile(x, y)), "({x}, {y})");
            }
        }
    }

    #[test]
    fn on_an_endless_world_a_loaded_window_follows_the_hero() {
        let hero = Vec2::new(-3_000.0, 7_777.0);
        let mut world = ChunkedWorld::new(4, 25);
        let (hx, hy) = WorldMap::tile_coords(hero.x, hero.y);
        world.load_around(hx, hy, 1);
        let area = shown_area(&world, hero);
        assert_eq!(area.size(), Vec2::splat(WINDOW_TILES as f32 * TILE_SIZE));
        assert!(area.contains(hero));
        assert!((area.center() - hero).abs().max_element() <= (WINDOW_STEP as f32 / 2.0 + 1.0) * TILE_SIZE);
        // A small step keeps the window; a longer walk moves it.
        assert_eq!(shown_area(&world, hero + Vec2::X * TILE_SIZE * 0.4), area);
        assert_ne!(shown_area(&world, hero + Vec2::X * TILE_SIZE * 8.0), area);
        // Wherever the hero stands in their chunk, the window is loaded ground.
        for step in 0..64 {
            let hero = Vec2::new(step as f32 * 37.0 - 900.0, step as f32 * -23.0 + 400.0);
            let mut world = ChunkedWorld::new(4, 25);
            let (hx, hy) = WorldMap::tile_coords(hero.x, hero.y);
            world.load_around(hx, hy, 1);
            let area = shown_area(&world, hero);
            for corner in [area.min, area.max - Vec2::splat(1.0), Vec2::new(area.min.x, area.max.y - 1.0)] {
                let (x, y) = WorldMap::tile_coords(corner.x, corner.y);
                assert!(world.loaded_tile(x, y).is_some(), "corner ({x}, {y}) for hero at {hero}");
            }
        }

        let image = bake(&world, area);
        let data = image.data.as_ref().unwrap();
        assert_eq!(data.len(), (WINDOW_TILES * WINDOW_TILES * 4) as usize);
        let (left, bottom) = WorldMap::tile_coords(area.min.x, area.min.y);
        for row in 0..WINDOW_TILES {
            for column in 0..WINDOW_TILES {
                let (x, y) = (left + column, bottom + WINDOW_TILES - 1 - row);
                // The whole window lies in loaded ground.
                assert!(world.loaded_tile(x, y).is_some(), "({x}, {y}) is not loaded");
                let index = ((row * WINDOW_TILES + column) * 4) as usize;
                assert_eq!(&data[index..index + 4], &tile_color(world.tile_at(x, y)));
            }
        }
    }
}
