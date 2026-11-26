//! Endless world: terrain generated chunk by chunk from a seed.
//!
//! Part of the world context (see [`crate::world`]); it uses the same
//! [`Tile`]s and grid conventions, but over unbounded `i32` coordinates.
//! Every tile is a pure function of `(seed, x, y)`: noise is sampled from a
//! lattice hashed from global coordinates, so a chunk never needs its
//! neighbours to agree with them and revisiting an area gives the same
//! ground. The hero starts in a clearing around tile `(0, 0)`.

use std::collections::HashMap;

use crate::world::Tile;

/// Edge length of a chunk in tiles.
pub const CHUNK_SIZE: i32 = 32;

/// Chunk holding the tile at global grid coordinates, and the tile's
/// position inside that chunk.
pub fn chunk_coords(x: i32, y: i32) -> ((i32, i32), (i32, i32)) {
    (
        (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE)),
        (x.rem_euclid(CHUNK_SIZE), y.rem_euclid(CHUNK_SIZE)),
    )
}

/// A square block of generated terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    coords: (i32, i32),
    tiles: Vec<Tile>,
}

impl Chunk {
    /// Chunk coordinates (in chunks, not tiles).
    pub fn coords(&self) -> (i32, i32) {
        self.coords
    }

    /// Global grid coordinates of the chunk's bottom-left tile.
    pub fn origin(&self) -> (i32, i32) {
        (self.coords.0 * CHUNK_SIZE, self.coords.1 * CHUNK_SIZE)
    }

    /// Tile at local coordinates in `0..CHUNK_SIZE`.
    ///
    /// # Panics
    /// If the coordinates are outside the chunk.
    pub fn tile(&self, lx: i32, ly: i32) -> Tile {
        assert!(
            (0..CHUNK_SIZE).contains(&lx) && (0..CHUNK_SIZE).contains(&ly),
            "({lx}, {ly}) is outside the chunk"
        );
        self.tiles[(ly * CHUNK_SIZE + lx) as usize]
    }
}

/// Generates chunk `(cx, cy)` of the world for `seed`. Deterministic, and
/// seamless with every neighbouring chunk.
pub fn generate_chunk(seed: u64, cx: i32, cy: i32) -> Chunk {
    let (ox, oy) = (cx * CHUNK_SIZE, cy * CHUNK_SIZE);
    let tiles = (0..CHUNK_SIZE)
        .flat_map(|ly| (0..CHUNK_SIZE).map(move |lx| terrain_at(seed, ox + lx, oy + ly)))
        .collect();
    Chunk { coords: (cx, cy), tiles }
}

/// Noise layers; each gets its own lattice from the seed.
mod layer {
    pub const ELEVATION: u64 = 1;
    pub const FOREST: u64 = 2;
    pub const MEADOW: u64 = 3;
    pub const TREE: u64 = 4;
    pub const ROCK: u64 = 5;
    pub const RUIN: u64 = 6;
}

/// The terrain at one global tile. Same rules and look as the finite valley
/// ([`crate::world::generate`]) minus the forest wall around it.
fn terrain_at(seed: u64, x: i32, y: i32) -> Tile {
    let from_origin = ((x as f32).powi(2) + (y as f32).powi(2)).sqrt();
    if from_origin < 2.0 {
        return Tile::Dirt;
    }
    if from_origin < 5.0 {
        return Tile::Grass;
    }
    let e = value_noise(seed, layer::ELEVATION, 7, x, y);
    let f = value_noise(seed, layer::FOREST, 5, x, y);
    let m = value_noise(seed, layer::MEADOW, 4, x, y);
    let chance = |layer, p| unit(hash(seed, layer, x, y)) < p;

    if e < 0.24 {
        Tile::Water
    } else if e < 0.30 {
        Tile::Sand
    } else if f > 0.66 && chance(layer::TREE, 0.55) {
        Tile::Tree
    } else if e > 0.74 && chance(layer::ROCK, 0.35) {
        Tile::Rock
    } else if chance(layer::RUIN, 0.006) {
        Tile::Ruin
    } else if m > 0.68 {
        Tile::Flowers
    } else if m < 0.34 {
        Tile::TallGrass
    } else if e > 0.62 {
        Tile::Dirt
    } else {
        Tile::Grass
    }
}

/// Smooth value noise in `[0, 1)` at tile `(x, y)`, interpolated between
/// random lattice points every `cell` tiles. Integer lattice maths keeps it
/// exact however far from the origin the hero wanders.
fn value_noise(seed: u64, layer: u64, cell: i32, x: i32, y: i32) -> f32 {
    let (gx, gy) = (x.div_euclid(cell), y.div_euclid(cell));
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let tx = smooth(x.rem_euclid(cell) as f32 / cell as f32);
    let ty = smooth(y.rem_euclid(cell) as f32 / cell as f32);
    let at = |ix: i32, iy: i32| unit(hash(seed, layer, ix, iy));
    let bottom = at(gx, gy) + (at(gx + 1, gy) - at(gx, gy)) * tx;
    let top = at(gx, gy + 1) + (at(gx + 1, gy + 1) - at(gx, gy + 1)) * tx;
    bottom + (top - bottom) * ty
}

/// Well-mixed 64-bit hash of a lattice point (SplitMix64 finaliser).
fn hash(seed: u64, layer: u64, x: i32, y: i32) -> u64 {
    let mut h = seed
        ^ layer.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (x as u32 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ (y as u32 as u64).wrapping_mul(0x94D0_49BB_1331_11EB).rotate_left(32);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}

/// Maps a hash to `[0, 1)`.
fn unit(h: u64) -> f32 {
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// The endless world, generated lazily. Chunks load when a tile in them is
/// asked for; once more than `capacity` are loaded the least recently used
/// ones are dropped, so walking back over recent ground stays cheap while
/// memory stays flat on a long walk.
#[derive(Debug, Clone)]
pub struct ChunkedWorld {
    seed: u64,
    capacity: usize,
    /// Loaded chunks with the tick they were last used on.
    chunks: HashMap<(i32, i32), (Chunk, u64)>,
    tick: u64,
}

impl ChunkedWorld {
    /// A world for `seed` that keeps at most `capacity` chunks (at least one).
    pub fn new(seed: u64, capacity: usize) -> Self {
        Self { seed, capacity: capacity.max(1), chunks: HashMap::new(), tick: 0 }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Maximum number of chunks kept loaded.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of chunks currently loaded.
    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    /// Whether chunk `(cx, cy)` is loaded right now.
    pub fn is_loaded(&self, cx: i32, cy: i32) -> bool {
        self.chunks.contains_key(&(cx, cy))
    }

    /// Tile at global grid coordinates, loading its chunk if needed.
    pub fn tile_at(&mut self, x: i32, y: i32) -> Tile {
        let ((cx, cy), (lx, ly)) = chunk_coords(x, y);
        self.chunk(cx, cy).tile(lx, ly)
    }

    /// Tile at global grid coordinates if its chunk is loaded; never loads.
    pub fn loaded_tile(&self, x: i32, y: i32) -> Option<Tile> {
        let ((cx, cy), (lx, ly)) = chunk_coords(x, y);
        self.chunks.get(&(cx, cy)).map(|(chunk, _)| chunk.tile(lx, ly))
    }

    /// Chunk `(cx, cy)`, loading it if needed and marking it recently used.
    pub fn chunk(&mut self, cx: i32, cy: i32) -> &Chunk {
        self.touch(cx, cy);
        self.evict();
        &self.chunks[&(cx, cy)].0
    }

    /// Loads every chunk within `radius` chunks (a square) of the chunk
    /// holding tile `(x, y)`, then evicts the least recently used chunks
    /// outside it. Capacity grows if the square would not fit.
    pub fn load_around(&mut self, x: i32, y: i32, radius: i32) {
        let ((cx, cy), _) = chunk_coords(x, y);
        let radius = radius.max(0);
        let side = (2 * radius + 1) as usize;
        self.capacity = self.capacity.max(side * side);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                self.touch(cx + dx, cy + dy);
            }
        }
        self.evict();
    }

    fn touch(&mut self, cx: i32, cy: i32) {
        self.tick += 1;
        let (seed, tick) = (self.seed, self.tick);
        self.chunks.entry((cx, cy)).or_insert_with(|| (generate_chunk(seed, cx, cy), tick)).1 = tick;
    }

    fn evict(&mut self) {
        while self.chunks.len() > self.capacity {
            let Some(oldest) = self.chunks.iter().min_by_key(|(_, (_, used))| *used).map(|(key, _)| *key)
            else {
                return;
            };
            self.chunks.remove(&oldest);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_coords_split_negative_tiles_correctly() {
        assert_eq!(chunk_coords(0, 0), ((0, 0), (0, 0)));
        assert_eq!(chunk_coords(31, 32), ((0, 1), (31, 0)));
        assert_eq!(chunk_coords(-1, -32), ((-1, -1), (31, 0)));
        assert_eq!(chunk_coords(-33, 5), ((-2, 0), (31, 5)));
    }

    #[test]
    fn same_seed_and_coords_give_the_same_chunk() {
        for (cx, cy) in [(0, 0), (3, -7), (-120, 45), (100_000, -100_000)] {
            assert_eq!(generate_chunk(42, cx, cy), generate_chunk(42, cx, cy));
        }
        assert_ne!(generate_chunk(1, 4, 4), generate_chunk(2, 4, 4), "seeds should matter");
        assert_ne!(generate_chunk(1, 4, 4).tiles, generate_chunk(1, 5, 4).tiles, "chunks should differ");
    }

    #[test]
    fn chunks_tile_seamlessly() {
        let seed = 7;
        let (cx, cy) = (2, -3);
        let here = generate_chunk(seed, cx, cy);
        let right = generate_chunk(seed, cx + 1, cy);
        let above = generate_chunk(seed, cx, cy + 1);
        let (ox, oy) = here.origin();

        // Every tile matches the global terrain, whichever chunk holds it.
        for (chunk, (x0, y0)) in
            [(&here, (ox, oy)), (&right, (ox + CHUNK_SIZE, oy)), (&above, (ox, oy + CHUNK_SIZE))]
        {
            for l in 0..CHUNK_SIZE {
                for edge in [0, CHUNK_SIZE - 1] {
                    assert_eq!(chunk.tile(edge, l), terrain_at(seed, x0 + edge, y0 + l));
                    assert_eq!(chunk.tile(l, edge), terrain_at(seed, x0 + l, y0 + edge));
                }
            }
        }

        // The noise is continuous across the seam: no bigger jump there than
        // anywhere inside a chunk.
        let step = |x: i32, y: i32, dx: i32, dy: i32| {
            (value_noise(seed, layer::ELEVATION, 7, x, y)
                - value_noise(seed, layer::ELEVATION, 7, x + dx, y + dy))
            .abs()
        };
        let mut inside = 0.0f32;
        for y in oy..oy + CHUNK_SIZE {
            for x in ox..ox + CHUNK_SIZE - 1 {
                inside = inside.max(step(x, y, 1, 0));
            }
        }
        for l in 0..CHUNK_SIZE {
            assert!(step(ox + CHUNK_SIZE - 1, oy + l, 1, 0) <= inside + 1e-6);
            assert!(step(ox + l, oy + CHUNK_SIZE - 1, 0, 1) <= inside + 1e-6);
        }
    }

    #[test]
    fn hero_origin_is_a_walkable_clearing_with_room_to_roam() {
        for seed in 0..40 {
            let mut world = ChunkedWorld::new(seed * 7919, 16);
            for y in -3..=3 {
                for x in -3..=3 {
                    assert!(world.tile_at(x, y).is_walkable(), "seed {seed}: ({x}, {y}) is blocked");
                }
            }
            // The clearing opens onto plenty of walkable ground.
            let radius: i32 = 40;
            let mut seen = std::collections::HashSet::new();
            let mut stack = vec![(0i32, 0i32)];
            while let Some((x, y)) = stack.pop() {
                if x.abs() > radius
                    || y.abs() > radius
                    || !world.tile_at(x, y).is_walkable()
                    || !seen.insert((x, y))
                {
                    continue;
                }
                stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
            }
            let area = (2 * radius + 1).pow(2) as usize;
            assert!(seen.len() > area / 3, "seed {seed}: only {} tiles reachable", seen.len());
        }
    }

    #[test]
    fn world_tiles_match_generated_chunks() {
        let mut world = ChunkedWorld::new(9, 4);
        let chunk = generate_chunk(9, -1, 2);
        let (ox, oy) = chunk.origin();
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                assert_eq!(world.tile_at(ox + lx, oy + ly), chunk.tile(lx, ly));
            }
        }
        assert_eq!(world.loaded_count(), 1);
    }

    #[test]
    fn least_recently_used_chunks_are_evicted() {
        let mut world = ChunkedWorld::new(3, 3);
        world.tile_at(0, 0); // chunk (0, 0)
        world.tile_at(CHUNK_SIZE, 0); // chunk (1, 0)
        world.tile_at(2 * CHUNK_SIZE, 0); // chunk (2, 0)
        world.tile_at(1, 1); // (0, 0) is used again, so (1, 0) is now oldest
        world.tile_at(3 * CHUNK_SIZE, 0); // chunk (3, 0) pushes one out
        assert_eq!(world.loaded_count(), 3);
        assert!(world.is_loaded(0, 0) && world.is_loaded(2, 0) && world.is_loaded(3, 0));
        assert!(!world.is_loaded(1, 0));
        assert_eq!(world.loaded_tile(CHUNK_SIZE, 0), None);

        // Coming back regenerates identical ground.
        let before = generate_chunk(3, 1, 0);
        assert_eq!(world.chunk(1, 0), &before);
    }

    #[test]
    fn a_long_walk_keeps_memory_flat() {
        let mut world = ChunkedWorld::new(11, 12);
        for step in 0..2_000 {
            let x = step * 4;
            world.load_around(x, -x / 2, 1);
            assert!(world.loaded_count() <= 12);
            let ((cx, cy), _) = chunk_coords(x, -x / 2);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    assert!(world.is_loaded(cx + dx, cy + dy), "the hero's surroundings stay loaded");
                }
            }
        }
        assert!(!world.is_loaded(0, 0), "the start was left far behind");
    }

    #[test]
    fn load_around_grows_capacity_to_fit_the_window() {
        let mut world = ChunkedWorld::new(5, 4);
        world.load_around(0, 0, 2);
        assert_eq!(world.capacity(), 25);
        assert_eq!(world.loaded_count(), 25);
    }
}
