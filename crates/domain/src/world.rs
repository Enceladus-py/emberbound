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
    /// A map with the given tiles, row by row from `y = 0` upwards. Maps for
    /// play come from [`generate`]; this is for small hand-made layouts.
    ///
    /// # Panics
    /// If `tiles` does not hold exactly `width * height` tiles.
    pub fn from_tiles(width: u32, height: u32, tiles: Vec<Tile>) -> Self {
        assert_eq!(tiles.len(), (width * height) as usize, "a {width}x{height} map needs that many tiles");
        Self { width, height, tiles }
    }

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

/// Shortest walking distances from every tile to one target tile. Walkers
/// follow it downhill to reach the target around ponds, copses and rocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowField {
    width: i32,
    height: i32,
    target: (i32, i32),
    /// Steps to the target per tile; [`FlowField::UNREACHABLE`] if cut off.
    steps: Vec<u16>,
}

impl FlowField {
    const UNREACHABLE: u16 = u16::MAX;

    /// Floods outwards from `target` over walkable tiles (4-connected).
    pub fn towards(map: &WorldMap, target: (i32, i32)) -> Self {
        let (width, height) = (map.width as i32, map.height as i32);
        let mut field =
            Self { width, height, target, steps: vec![Self::UNREACHABLE; (width * height) as usize] };
        let Some(start) = field.index(target.0, target.1) else { return field };
        field.steps[start] = 0;
        let mut queue = std::collections::VecDeque::from([target]);
        while let Some((x, y)) = queue.pop_front() {
            let next = field.steps_at(x, y) + 1;
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                if let Some(i) = field.index(nx, ny)
                    && field.steps[i] == Self::UNREACHABLE
                    && map.tile(nx, ny).is_walkable()
                {
                    field.steps[i] = next;
                    queue.push_back((nx, ny));
                }
            }
        }
        field
    }

    pub fn target(&self) -> (i32, i32) {
        self.target
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && x < self.width && y < self.height).then(|| (y * self.width + x) as usize)
    }

    fn steps_at(&self, x: i32, y: i32) -> u16 {
        self.index(x, y).map_or(Self::UNREACHABLE, |i| self.steps[i])
    }

    /// Walking steps from a tile to the target, if it can be reached.
    pub fn steps(&self, x: i32, y: i32) -> Option<u32> {
        let steps = self.steps_at(x, y);
        (steps != Self::UNREACHABLE).then_some(steps as u32)
    }

    /// Unit direction a walker at this world position should head in. `None`
    /// when already on the target tile or when the target cannot be reached.
    /// Diagonal moves never cut across the corner of an obstacle.
    pub fn direction(&self, wx: f32, wy: f32) -> Option<(f32, f32)> {
        let (x, y) = WorldMap::tile_coords(wx, wy);
        let here = self.steps_at(x, y);
        if here == 0 || here == Self::UNREACHABLE {
            return None;
        }
        let mut best: Option<(u16, bool, (i32, i32))> = None;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let diagonal = dx != 0 && dy != 0;
            let steps = self.steps_at(x + dx, y + dy);
            let open_corner = !diagonal
                || (self.steps_at(x + dx, y) != Self::UNREACHABLE
                    && self.steps_at(x, y + dy) != Self::UNREACHABLE);
            if steps >= here || !open_corner {
                continue;
            }
            // Fewest steps wins; a diagonal breaks ties because it is shorter.
            if best.is_none_or(|(s, d, _)| steps < s || (steps == s && diagonal && !d)) {
                best = Some((steps, diagonal, (x + dx, y + dy)));
            }
        }
        let (_, _, (tx, ty)) = best?;
        let (cx, cy) = WorldMap::tile_center(tx, ty);
        let (dx, dy) = (cx - wx, cy - wy);
        let length = (dx * dx + dy * dy).sqrt();
        (length > f32::EPSILON).then(|| (dx / length, dy / length))
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
    fn a_hand_made_map_keeps_its_layout() {
        let map = WorldMap::from_tiles(
            3,
            2,
            vec![Tile::Grass, Tile::Rock, Tile::Grass, Tile::Water, Tile::Dirt, Tile::Sand],
        );
        assert_eq!((map.width(), map.height()), (3, 2));
        assert_eq!(map.tile(1, 0), Tile::Rock);
        assert_eq!(map.tile(0, 1), Tile::Water);
        assert_eq!(map.tile(2, 1), Tile::Sand);
        assert_eq!(map.tile(3, 0), Tile::Tree, "outside is still forest");
    }

    #[test]
    #[should_panic]
    fn a_hand_made_map_needs_every_tile() {
        WorldMap::from_tiles(2, 2, vec![Tile::Grass; 3]);
    }

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

    #[test]
    fn flow_field_leads_every_reachable_tile_to_the_target() {
        let map = generate(48, 48, &mut TestRng(99));
        let target = (24, 24);
        let field = FlowField::towards(&map, target);
        assert_eq!(field.steps(24, 24), Some(0));
        let mut walked = 0;
        for y in 0..48 {
            for x in 0..48 {
                let Some(mut remaining) = field.steps(x, y) else { continue };
                // Follow the field tile by tile: it must strictly descend.
                let (mut tx, mut ty) = (x, y);
                while remaining > 0 {
                    let (wx, wy) = WorldMap::tile_center(tx, ty);
                    let (dx, dy) = field.direction(wx, wy).expect("reachable tile has a way on");
                    (tx, ty) = WorldMap::tile_coords(wx + dx * TILE_SIZE * 1.2, wy + dy * TILE_SIZE * 1.2);
                    assert!(map.tile(tx, ty).is_walkable(), "field led into an obstacle");
                    let next = field.steps(tx, ty).unwrap();
                    assert!(next < remaining);
                    remaining = next;
                }
                assert_eq!((tx, ty), target);
                walked += 1;
            }
        }
        assert_eq!(walked, map.reachable_from_spawn());
    }

    #[test]
    fn flow_field_has_no_direction_where_cut_off() {
        let map = generate(32, 32, &mut TestRng(5));
        let field = FlowField::towards(&map, (16, 16));
        assert_eq!(field.direction(-50.0, -50.0), None);
        let (cx, cy) = WorldMap::tile_center(16, 16);
        assert_eq!(field.direction(cx, cy), None);
        assert_eq!(field.steps(0, 0), None, "the forest border is solid");
    }
}
