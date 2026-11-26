//! Kinematics: integrates [`Velocity`] and [`Knockback`] and keeps walkers
//! out of water, trees, rocks and ruins.

use bevy::prelude::*;
use emberbound_core::{Body, GameSet, Knockback, Player, Terrain, Velocity};
use emberbound_domain::world::{TILE_SIZE, WorldMap};

/// Seconds in which knockback loses half of its speed.
const KNOCKBACK_HALF_LIFE: f32 = 0.08;
/// Knockback slower than this (units per second) is dropped.
const KNOCKBACK_REST_SPEED: f32 = 1.0;
/// Longest distance a walker moves between two collision checks. Half a
/// tile can never skip over an obstacle.
const MAX_STEP: f32 = TILE_SIZE * 0.5;

/// Moves a walker by `delta`, one axis at a time so it slides along
/// obstacles instead of sticking to them, in steps short enough not to
/// tunnel. A walker that already overlaps an obstacle (e.g. it was spawned
/// or shoved there) moves freely until it is out again.
fn walk(map: &WorldMap, mut position: Vec2, delta: Vec2, radius: f32) -> Vec2 {
    let steps = (delta.abs().max_element() / MAX_STEP).ceil().max(1.0);
    let step = delta / steps;
    for _ in 0..steps as u32 {
        let stuck = !map.is_walkable_at(position.x, position.y, radius);
        if stuck || map.is_walkable_at(position.x + step.x, position.y, radius) {
            position.x += step.x;
        }
        if stuck || map.is_walkable_at(position.x, position.y + step.y, radius) {
            position.y += step.y;
        }
    }
    position
}

/// Applies `(velocity + knockback) * dt` to everything that has a velocity.
/// Only bodies on foot collide with the terrain.
fn integrate_velocities(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut movers: Query<(&mut Transform, &Velocity, Option<&Knockback>, Option<&Body>)>,
) {
    let dt = time.delta_secs();
    for (mut transform, velocity, knockback, body) in &mut movers {
        let delta = (velocity.0 + knockback.map_or(Vec2::ZERO, |k| k.0)) * dt;
        if delta == Vec2::ZERO {
            continue;
        }
        let position = transform.translation.truncate();
        let moved = match body {
            Some(body) if body.walks => walk(&terrain.0, position, delta, body.radius),
            _ => position + delta,
        };
        if moved != position {
            transform.translation = moved.extend(transform.translation.z);
        }
    }
}

/// Lets knockback die away exponentially, independent of the frame rate.
fn decay_knockback(time: Res<Time>, mut knockbacks: Query<&mut Knockback>) {
    let keep = 0.5f32.powf(time.delta_secs() / KNOCKBACK_HALF_LIFE);
    for mut knockback in &mut knockbacks {
        if knockback.0 == Vec2::ZERO {
            continue;
        }
        let decayed = knockback.0 * keep;
        knockback.0 = if decayed.length() < KNOCKBACK_REST_SPEED { Vec2::ZERO } else { decayed };
    }
}

/// Last line of defence: whatever happens, the hero stays on the map.
fn confine_hero(terrain: Res<Terrain>, mut hero: Single<(&mut Transform, &Body), With<Player>>) {
    let (transform, body) = &mut *hero;
    let (width, height) = terrain.0.world_size();
    let min = Vec2::splat(body.radius);
    let max = (Vec2::new(width, height) - min).max(min);
    let position = transform.translation.truncate();
    let confined = position.clamp(min, max);
    if confined != position {
        transform.translation = confined.extend(transform.translation.z);
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        (integrate_velocities, decay_knockback, confine_hero).chain().in_set(GameSet::Movement),
    );
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;
    use emberbound_domain::world::Tile;

    use super::*;

    const RADIUS: f32 = 5.0;

    /// A map drawn in text, top row first: `.` is grass, `#` is rock.
    fn map(rows: &[&str]) -> WorldMap {
        let width = rows[0].len() as u32;
        let tiles = rows
            .iter()
            .rev()
            .flat_map(|row| row.chars().map(|c| if c == '#' { Tile::Rock } else { Tile::Grass }))
            .collect();
        WorldMap::from_tiles(width, rows.len() as u32, tiles)
    }

    /// World position of the centre of a tile.
    fn at(x: i32, y: i32) -> Vec2 {
        Vec2::from(WorldMap::tile_center(x, y))
    }

    const OPEN: &[&str] = &["......", "......", "......", "......", "......", "......"];

    /// Column 3 is a one-tile-thick wall across the whole map.
    const WALL: &[&str] =
        &["...#....", "...#....", "...#....", "...#....", "...#....", "...#....", "...#....", "...#...."];

    #[test]
    fn walking_on_open_ground_moves_by_the_full_delta() {
        let map = map(OPEN);
        let start = at(2, 2);
        let delta = Vec2::new(13.0, -9.5);
        assert_eq!(walk(&map, start, delta, RADIUS), start + delta);
    }

    #[test]
    fn walking_diagonally_into_a_wall_slides_along_it() {
        let map = map(WALL);
        let start = at(2, 3);
        let moved = walk(&map, start, Vec2::new(20.0, 20.0), RADIUS);
        assert!(moved.x < 3.0 * TILE_SIZE - RADIUS + 0.01, "x stops at the wall: {moved}");
        assert_eq!(moved.y, start.y + 20.0, "the free axis keeps moving");
        assert!(map.is_walkable_at(moved.x, moved.y, RADIUS));
    }

    #[test]
    fn a_huge_step_cannot_tunnel_through_a_thin_wall() {
        let map = map(WALL);
        let start = at(1, 3);
        for distance in [TILE_SIZE, 3.0 * TILE_SIZE, 6.0 * TILE_SIZE, 40.0 * TILE_SIZE] {
            let moved = walk(&map, start, Vec2::new(distance, 0.0), RADIUS);
            assert!(moved.x < 3.0 * TILE_SIZE, "{distance} units went through the wall to {moved}");
            assert_eq!(moved.y, start.y);
        }
    }

    #[test]
    fn a_walker_stuck_in_an_obstacle_can_walk_out() {
        let map = map(WALL);
        let start = at(3, 3);
        assert!(!map.is_walkable_at(start.x, start.y, RADIUS));
        let moved = walk(&map, start, Vec2::new(2.0 * TILE_SIZE, 0.0), RADIUS);
        assert_eq!(moved, start + Vec2::new(2.0 * TILE_SIZE, 0.0));
        assert!(map.is_walkable_at(moved.x, moved.y, RADIUS));
    }

    fn app(rows: &[&str]) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(20)))
            .insert_resource(Terrain(map(rows)))
            .add_systems(Update, (integrate_velocities, decay_knockback).chain());
        // The first update only starts the clock.
        app.update();
        app
    }

    fn position(app: &App, entity: Entity) -> Vec2 {
        app.world().get::<Transform>(entity).unwrap().translation.truncate()
    }

    #[test]
    fn walkers_are_stopped_by_terrain_but_flyers_cross_it() {
        let mut app = app(WALL);
        let start = at(1, 3);
        let mover = |walks| {
            (
                Transform::from_translation(start.extend(0.0)),
                Velocity(Vec2::X * 200.0),
                Body { radius: RADIUS, walks },
            )
        };
        let walker = app.world_mut().spawn(mover(true)).id();
        let flyer = app.world_mut().spawn(mover(false)).id();
        let unbodied = app
            .world_mut()
            .spawn((Transform::from_translation(start.extend(0.0)), Velocity(Vec2::X * 200.0)))
            .id();
        for _ in 0..30 {
            app.update();
        }
        assert!(position(&app, walker).x < 3.0 * TILE_SIZE, "the walker stays west of the wall");
        assert!(position(&app, flyer).x > 4.0 * TILE_SIZE, "the flyer passes over it");
        assert!(position(&app, unbodied).x > 4.0 * TILE_SIZE, "no body, no collision");
    }

    #[test]
    fn knockback_halves_every_half_life_then_rests() {
        let mut app = app(OPEN);
        // Advance by exactly one half-life per update.
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(KNOCKBACK_HALF_LIFE)));
        let entity = app.world_mut().spawn(Knockback(Vec2::new(64.0, 0.0))).id();
        let knockback = |app: &App| app.world().get::<Knockback>(entity).unwrap().0;

        app.update();
        assert!((knockback(&app).x - 32.0).abs() < 0.5, "{}", knockback(&app));
        app.update();
        assert!((knockback(&app).x - 16.0).abs() < 0.5, "{}", knockback(&app));

        for _ in 0..10 {
            app.update();
        }
        assert_eq!(knockback(&app), Vec2::ZERO, "slow knockback snaps to rest");
    }

    #[test]
    fn knockback_just_above_rest_speed_snaps_to_zero_once_below_it() {
        let mut app = app(OPEN);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(KNOCKBACK_HALF_LIFE)));
        let entity = app.world_mut().spawn(Knockback(Vec2::new(0.0, KNOCKBACK_REST_SPEED * 1.5))).id();
        app.update();
        assert_eq!(app.world().get::<Knockback>(entity).unwrap().0, Vec2::ZERO);
    }

    #[test]
    fn the_hero_is_kept_on_the_map() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).insert_resource(Terrain(map(OPEN))).add_systems(Update, confine_hero);
        let (width, height) = (6.0 * TILE_SIZE, 6.0 * TILE_SIZE);
        let body = Body { radius: RADIUS, walks: true };
        let cases = [
            (Vec2::new(-50.0, -50.0), Vec2::splat(RADIUS)),
            (Vec2::new(width + 30.0, 40.0), Vec2::new(width - RADIUS, 40.0)),
            (Vec2::new(40.0, height + 1.0), Vec2::new(40.0, height - RADIUS)),
            (Vec2::new(40.0, 50.0), Vec2::new(40.0, 50.0)),
        ];
        let hero = app.world_mut().spawn((Player, Transform::from_xyz(0.0, 0.0, 7.0), body)).id();
        for (placed, expected) in cases {
            app.world_mut().get_mut::<Transform>(hero).unwrap().translation = placed.extend(7.0);
            app.update();
            let transform = app.world().get::<Transform>(hero).unwrap();
            assert_eq!(transform.translation.truncate(), expected, "placed at {placed}");
            assert_eq!(transform.translation.z, 7.0, "depth is kept");
        }
    }
}
