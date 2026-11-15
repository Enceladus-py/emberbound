//! Ground tiles. Everything is drawn through [`Tile::put`], which wraps
//! around the 16x16 tile, so details crossing an edge continue on the other
//! side and every tile repeats seamlessly next to itself and its variants.

use super::hash;
use crate::canvas::{Canvas, Rgba};
use crate::palette::{
    BARK, BLUE, CORAL, GOLD, GRASS, GRASS_SHADE, GREEN, GREEN_DEEP, ICE, LIME, NAVY, SAND, SAND_LIGHT, SKY, STONE_LIGHT, TAN,
    VIOLET, WHITE,
};

const SIZE: i32 = 16;
const VARIANTS: u32 = 4;

struct Tile(Canvas);

impl Tile {
    fn filled(color: Rgba) -> Self {
        Self(Canvas::filled(SIZE as u32, SIZE as u32, color))
    }

    fn put(&mut self, x: i32, y: i32, color: Rgba) {
        self.0.set(x.rem_euclid(SIZE), y.rem_euclid(SIZE), color);
    }

    fn row(&mut self, x: i32, y: i32, len: i32, color: Rgba) {
        for i in 0..len {
            self.put(x + i, y, color);
        }
    }
}

/// Evenly spread detail spots: at most one per `cell`x`cell` block, each
/// kept with `percent` probability. Yields `(x, y, random bits)`.
fn spots(seed: u32, cell: i32, percent: u32) -> impl Iterator<Item = (i32, i32, u32)> {
    let per_side = SIZE / cell;
    (0..per_side * per_side).filter_map(move |i| {
        let h = hash(i as u32, 11, seed);
        (h % 100 < percent).then(|| {
            let (cx, cy) = (i % per_side, i / per_side);
            (cx * cell + ((h >> 8) as i32 & 0xff) % cell, cy * cell + ((h >> 16) as i32 & 0xff) % cell, h >> 24)
        })
    })
}

fn variants(paint: impl Fn(u32) -> Tile) -> Vec<Canvas> {
    (0..VARIANTS).map(|variant| paint(variant).0).collect()
}

fn grass_tile(seed: u32) -> Tile {
    let mut tile = Tile::filled(GRASS);
    for (x, y, bits) in spots(seed, 4, 60) {
        match bits % 5 {
            0 | 1 => {
                tile.row(x, y, 2, GRASS_SHADE);
                tile.put(x + 1, y + 1, GRASS_SHADE);
            }
            2 => {
                tile.put(x - 1, y - 1, GRASS_SHADE);
                tile.put(x, y, GRASS_SHADE);
                tile.put(x + 1, y - 1, GRASS_SHADE);
            }
            3 => {
                tile.put(x, y, GREEN);
                tile.put(x, y - 1, LIME);
            }
            _ => tile.put(x, y, LIME),
        }
    }
    tile
}

pub fn grass() -> Vec<Canvas> {
    variants(|v| grass_tile(100 + v))
}

pub fn tall_grass() -> Vec<Canvas> {
    variants(|v| {
        let mut tile = grass_tile(200 + v);
        for (x, y, bits) in spots(300 + v, 4, 70) {
            // A tuft: dark roots, mid blades, sunlit tips leaning either way.
            let lean = if bits % 2 == 0 { 1 } else { -1 };
            tile.put(x, y + 1, GREEN_DEEP);
            tile.row(x - 1, y, 3, GREEN);
            tile.put(x - 1, y - 1, GREEN);
            tile.put(x + 1, y - 1, GREEN);
            tile.put(x, y - 1, LIME);
            tile.put(x - lean, y - 2, LIME);
            tile.put(x + lean, y - 2, GREEN);
        }
        tile
    })
}

pub fn flowers() -> Vec<Canvas> {
    variants(|v| {
        let mut tile = grass_tile(400 + v);
        for (x, y, bits) in spots(500 + v, 8, 80) {
            let petal = [WHITE, CORAL, VIOLET, GOLD][(bits % 4) as usize];
            let heart = if petal == GOLD { CORAL } else { GOLD };
            tile.put(x, y - 1, petal);
            tile.put(x - 1, y, petal);
            tile.put(x + 1, y, petal);
            tile.put(x, y + 1, petal);
            tile.put(x, y, heart);
            tile.put(x, y + 2, GREEN);
            // A lone bud keeps it company.
            tile.put(x + 4, y + 3, petal);
            tile.put(x + 4, y + 4, GREEN);
        }
        tile
    })
}

pub fn dirt() -> Vec<Canvas> {
    variants(|v| {
        let mut tile = Tile::filled(TAN);
        for (x, y, bits) in spots(600 + v, 4, 65) {
            match bits % 4 {
                0 => tile.row(x, y, 2, BARK),
                1 => {
                    tile.put(x, y, SAND);
                    tile.put(x, y + 1, BARK);
                }
                2 => tile.put(x, y, BARK),
                _ => tile.row(x, y, 2, SAND),
            }
        }
        tile
    })
}

pub fn sand() -> Vec<Canvas> {
    variants(|v| {
        let mut tile = Tile::filled(SAND);
        for (x, y, bits) in spots(700 + v, 4, 60) {
            match bits % 4 {
                0 => tile.row(x, y, 3, SAND_LIGHT),
                1 => tile.row(x, y, 2, TAN),
                2 => tile.put(x, y, SAND_LIGHT),
                _ => {
                    tile.put(x, y, STONE_LIGHT);
                    tile.put(x, y + 1, TAN);
                }
            }
        }
        tile
    })
}

/// Shimmering water: wave crests drift one quarter tile per frame, so the
/// four frames loop and still tile.
pub fn water() -> Vec<Canvas> {
    const CRESTS: [(i32, i32, i32); 6] = [(1, 2, 5), (10, 4, 4), (5, 7, 4), (13, 9, 5), (2, 12, 4), (8, 14, 5)];
    (0..4)
        .map(|frame| {
            let mut tile = Tile::filled(BLUE);
            for (i, (x, y, len)) in CRESTS.into_iter().enumerate() {
                // Neighbouring crests drift in opposite directions.
                let drift = if i % 2 == 0 { frame * 4 } else { -frame * 4 };
                let bob = (frame + i as i32) % 2;
                tile.row(x + drift + 1, y + 1, len, NAVY);
                tile.row(x + drift, y + bob - 1 + 1, len, SKY);
                if (frame + i as i32) % 4 == 0 {
                    tile.put(x + drift + 1, y + bob, ICE);
                }
            }
            tile.0
        })
        .collect()
}
