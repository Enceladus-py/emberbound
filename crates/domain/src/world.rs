//! World context: the valley map and its generation.
//!
//! World space: one unit is one art pixel. Tile `(0, 0)` covers world
//! `(0, 0)..(TILE_SIZE, TILE_SIZE)`; x grows right and y grows up.

use crate::ports::RandomSource;

/// Edge length of a tile in world units.
pub const TILE_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Grass,
    TallGrass,
    Flowers,
    Dirt,
    Sand,
    Water,
    Tree,
    Rock,
    Ruin,
}

impl Tile {
    /// Whether creatures on foot can stand here.
    pub fn is_walkable(self) -> bool {
        !matches!(self, Tile::Water | Tile::Tree | Tile::Rock | Tile::Ruin)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldMap {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl WorldMap {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Size of the whole map in world units.
    pub fn world_size(&self) -> (f32, f32) {
        (self.width as f32 * TILE_SIZE, self.height as f32 * TILE_SIZE)
    }

    /// Tile at grid coordinates; anything outside the map is solid forest.
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return Tile::Tree;
        }
        self.tiles[(y as u32 * self.width + x as u32) as usize]
    }

    /// Grid coordinates of the tile containing a world position.
    pub fn tile_coords(wx: f32, wy: f32) -> (i32, i32) {
        ((wx / TILE_SIZE).floor() as i32, (wy / TILE_SIZE).floor() as i32)
    }

    /// World position of the centre of a tile.
    pub fn tile_center(x: i32, y: i32) -> (f32, f32) {
        ((x as f32 + 0.5) * TILE_SIZE, (y as f32 + 0.5) * TILE_SIZE)
    }

    /// Whether a walker of the given radius fits at a world position.
    pub fn is_walkable_at(&self, wx: f32, wy: f32, radius: f32) -> bool {
        [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].iter().all(|(sx, sy)| {
            let (tx, ty) = Self::tile_coords(wx + sx * radius, wy + sy * radius);
            self.tile(tx, ty).is_walkable()
        })
    }

    /// Where the hero starts: the centre of the map (always a clearing).
    pub fn spawn_point(&self) -> (f32, f32) {
        Self::tile_center(self.width as i32 / 2, self.height as i32 / 2)
    }

    /// Number of walkable tiles reachable on foot from the spawn point.
    pub fn reachable_from_spawn(&self) -> usize {
        let start = (self.width as i32 / 2, self.height as i32 / 2);
        let mut seen = vec![false; self.tiles.len()];
        let mut stack = vec![start];
        let mut count = 0;
        while let Some((x, y)) = stack.pop() {
            if !self.tile(x, y).is_walkable() {
                continue;
            }
            let index = (y as u32 * self.width + x as u32) as usize;
            if std::mem::replace(&mut seen[index], true) {
                continue;
            }
            count += 1;
            stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
        }
        count
    }

    pub fn walkable_count(&self) -> usize {
        self.tiles.iter().filter(|t| t.is_walkable()).count()
    }
}

/// Smooth value noise sampled from a coarse lattice of random values.
struct ValueNoise {
    cols: usize,
    values: Vec<f32>,
    cell: f32,
}

impl ValueNoise {
    fn new(width: u32, height: u32, cell: f32, rng: &mut dyn RandomSource) -> Self {
        let cols = (width as f32 / cell).ceil() as usize + 2;
        let rows = (height as f32 / cell).ceil() as usize + 2;
        Self { cols, values: (0..cols * rows).map(|_| rng.next_f32()).collect(), cell }
    }

    fn sample(&self, x: f32, y: f32) -> f32 {
        let (gx, gy) = (x / self.cell, y / self.cell);
        let (x0, y0) = (gx.floor() as usize, gy.floor() as usize);
        let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
        let (tx, ty) = (smooth(gx.fract()), smooth(gy.fract()));
        let at = |x: usize, y: usize| self.values[y * self.cols + x];
        let top = at(x0, y0) + (at(x0 + 1, y0) - at(x0, y0)) * tx;
        let bottom = at(x0, y0 + 1) + (at(x0 + 1, y0 + 1) - at(x0, y0 + 1)) * tx;
        top + (bottom - top) * ty
    }
}

/// Generates a valley: meadows, ponds, copses and ruins, walled in by forest
/// with a guaranteed clearing in the middle.
pub fn generate(width: u32, height: u32, rng: &mut dyn RandomSource) -> WorldMap {
    let (width, height) = (width.max(16), height.max(16));
    let elevation = ValueNoise::new(width, height, 7.0, rng);
    let forest = ValueNoise::new(width, height, 5.0, rng);
    let meadow = ValueNoise::new(width, height, 4.0, rng);
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);

    let mut tiles = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let (fx, fy) = (x as f32, y as f32);
            let border = x < 2 || y < 2 || x >= width - 2 || y >= height - 2;
            let from_centre = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            let e = elevation.sample(fx, fy);
            let f = forest.sample(fx, fy);
            let m = meadow.sample(fx, fy);

            let tile = if border {
                Tile::Tree
            } else if from_centre < 5.0 {
                if from_centre < 2.0 { Tile::Dirt } else { Tile::Grass }
            } else if e < 0.24 {
                Tile::Water
            } else if e < 0.30 {
                Tile::Sand
            } else if f > 0.66 && rng.chance(0.55) {
                Tile::Tree
            } else if e > 0.74 && rng.chance(0.35) {
                Tile::Rock
            } else if rng.chance(0.006) {
                Tile::Ruin
            } else if m > 0.68 {
                Tile::Flowers
            } else if m < 0.34 {
                Tile::TallGrass
            } else if e > 0.62 {
                Tile::Dirt
            } else {
                Tile::Grass
            };
            tiles.push(tile);
        }
    }
    WorldMap { width, height, tiles }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::testing::TestRng;

    #[test]
    fn spawn_is_open_and_borders_are_sealed() {
        for seed in 1..20 {
            let map = generate(64, 48, &mut TestRng(seed));
            let (sx, sy) = map.spawn_point();
            assert!(map.is_walkable_at(sx, sy, 6.0));
            for x in 0..64 {
                assert!(!map.tile(x, 0).is_walkable() && !map.tile(x, 47).is_walkable());
            }
            for y in 0..48 {
                assert!(!map.tile(0, y).is_walkable() && !map.tile(63, y).is_walkable());
            }
        }
    }

    #[test]
    fn most_of_the_valley_is_reachable() {
        for seed in 1..40 {
            let map = generate(72, 72, &mut TestRng(seed * 7919));
            let (reachable, walkable) = (map.reachable_from_spawn(), map.walkable_count());
            assert!(walkable > 72 * 72 / 2, "seed {seed}: too little open ground");
            assert!(
                reachable as f32 > walkable as f32 * 0.85,
                "seed {seed}: only {reachable}/{walkable} reachable"
            );
        }
    }

    #[test]
    fn outside_the_map_is_solid() {
        let map = generate(32, 32, &mut TestRng(1));
        assert_eq!(map.tile(-1, 5), Tile::Tree);
        assert_eq!(map.tile(5, 99), Tile::Tree);
        assert!(!map.is_walkable_at(-20.0, 10.0, 4.0));
    }

    #[test]
    fn coordinates_round_trip() {
        let (wx, wy) = WorldMap::tile_center(3, 9);
        assert_eq!(WorldMap::tile_coords(wx, wy), (3, 9));
        assert_eq!(WorldMap::tile_coords(-0.5, 0.0), (-1, 0));
    }
}
