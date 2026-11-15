//! Renders a contact sheet of every sprite plus a small scene mock-up and
//! writes it as an uncompressed 24-bit BMP, so the art can be looked at
//! without running the game.
//!
//! `cargo run -p emberbound_pixelart --example preview -- out.bmp [all|actors|dragons|world|scene]`

use emberbound_pixelart::{SpriteId, SpriteSheet, sheet};
use std::io::Write;

const GRASS: [u8; 3] = [0x5a, 0xa8, 0x4e];

/// An opaque RGB picture that sprites are alpha-blended onto.
struct Picture {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl Picture {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height, rgb: GRASS.iter().copied().cycle().take(width * height * 3).collect() }
    }

    /// Draws one frame with its top-left corner at `(x, y)`, magnified `scale` times.
    fn draw(&mut self, sprite: &SpriteSheet, frame: usize, x: i32, y: i32, scale: i32) {
        let (fw, fh) = (sprite.frame_width as i32, sprite.frame_height as i32);
        let pixels = &sprite.frames[frame % sprite.frames.len()];
        for sy in 0..fh {
            for sx in 0..fw {
                let px = &pixels[((sy * fw + sx) * 4) as usize..][..4];
                let alpha = px[3] as u32;
                for (dx, dy) in (0..scale * scale).map(|i| (i % scale, i / scale)) {
                    let (tx, ty) = (x + sx * scale + dx, y + sy * scale + dy);
                    if alpha == 0 || tx < 0 || ty < 0 || tx >= self.width as i32 || ty >= self.height as i32 {
                        continue;
                    }
                    let at = (ty as usize * self.width + tx as usize) * 3;
                    for c in 0..3 {
                        self.rgb[at + c] =
                            ((px[c] as u32 * alpha + self.rgb[at + c] as u32 * (255 - alpha)) / 255) as u8;
                    }
                }
            }
        }
    }

    /// Draws a sprite standing on the point `(x, y)` (bottom-centre anchor).
    fn stand(&mut self, id: SpriteId, frame: usize, x: i32, y: i32, scale: i32) {
        let sprite = sheet(id);
        let (w, h) = (sprite.frame_width as i32, sprite.frame_height as i32);
        self.draw(&sprite, frame, x - w * scale / 2, y - h * scale, scale);
    }

    /// Stacks `other` below this picture.
    fn append(&mut self, other: Picture) {
        let width = self.width.max(other.width);
        let mut joined = Picture::new(width, self.height + other.height);
        for (source, top) in [(&*self, 0), (&other, self.height)] {
            for row in 0..source.height {
                let from = row * source.width * 3;
                let to = (top + row) * width * 3;
                joined.rgb[to..to + source.width * 3]
                    .copy_from_slice(&source.rgb[from..from + source.width * 3]);
            }
        }
        *self = joined;
    }

    fn write_bmp(&self, path: &str) -> std::io::Result<()> {
        let row_bytes = (self.width * 3).div_ceil(4) * 4;
        let data_bytes = row_bytes * self.height;
        let mut out = Vec::with_capacity(54 + data_bytes);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&((54 + data_bytes) as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(self.width as i32).to_le_bytes());
        out.extend_from_slice(&(self.height as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(data_bytes as u32).to_le_bytes());
        out.extend_from_slice(&[0u8; 16]);
        // BMP rows run bottom-up, pixels are BGR, rows are padded to 4 bytes.
        for row in (0..self.height).rev() {
            for px in self.rgb[row * self.width * 3..][..self.width * 3].chunks(3) {
                out.extend_from_slice(&[px[2], px[1], px[0]]);
            }
            out.resize(out.len() + row_bytes - self.width * 3, 0);
        }
        std::fs::File::create(path)?.write_all(&out)
    }
}

/// Every frame of the given sprites, flowing left to right and wrapping.
fn contact_sheet(ids: &[SpriteId], scale: i32, width: usize) -> Picture {
    const GAP: i32 = 8;
    let mut placed = Vec::new();
    let (mut x, mut y, mut row_height) = (GAP, GAP, 0);
    for &id in ids {
        let sprite = sheet(id);
        let w = sprite.frame_width as i32 * scale * sprite.frames.len() as i32;
        let h = sprite.frame_height as i32 * scale;
        if x + w + GAP > width as i32 && x > GAP {
            (x, y, row_height) = (GAP, y + row_height + GAP, 0);
        }
        placed.push((sprite, x, y));
        x += w + GAP * 2;
        row_height = row_height.max(h);
    }
    let mut picture = Picture::new(width, (y + row_height + GAP) as usize);
    for (sprite, x, y) in placed {
        for frame in 0..sprite.frames.len() {
            picture.draw(&sprite, frame, x + frame as i32 * sprite.frame_width as i32 * scale, y, scale);
        }
    }
    picture
}

/// A cheap deterministic scatter for choosing tile variants.
fn scatter(x: i32, y: i32) -> usize {
    ((x * 7 + y * 13 + (x * y) % 5 + (x ^ y) * 3) & 0xffff) as usize
}

/// Each ground tile as a field of mixed variants, to judge seams.
fn tile_fields(scale: i32) -> Picture {
    use SpriteId::*;
    let tiles = [TileGrass, TileTallGrass, TileFlowers, TileDirt, TileSand, TileWater];
    let (cols, rows, gap) = (6, 4, 8);
    let field = 16 * scale * cols + gap;
    let mut picture =
        Picture::new((field * 3 + gap) as usize, ((16 * scale * rows + gap) * 2 + gap) as usize);
    for (i, id) in tiles.into_iter().enumerate() {
        let sprite = sheet(id);
        let (left, top) = (gap + (i as i32 % 3) * field, gap + (i as i32 / 3) * (16 * scale * rows + gap));
        for ty in 0..rows {
            for tx in 0..cols {
                // Water shows a single animation frame, like the game would.
                let frame = if id == TileWater { 1 } else { scatter(tx, ty) };
                picture.draw(&sprite, frame, left + tx * 16 * scale, top + ty * 16 * scale, scale);
            }
        }
    }
    picture
}

/// A little valley: ground, obstacles, the hero's party, monsters and a dragon.
fn scene(scale: i32) -> Picture {
    use SpriteId::*;
    let (cols, rows) = (22, 13);
    let mut picture = Picture::new((cols * 16 * scale) as usize, (rows * 16 * scale) as usize);
    for ty in 0..rows {
        for tx in 0..cols {
            let pond = (tx - 18) * (tx - 18) + (ty - 10) * (ty - 10);
            let id = if pond <= 5 {
                TileWater
            } else if pond <= 11 {
                TileSand
            } else if ty == 6 || (tx == 9 && ty > 6) {
                TileDirt
            } else if scatter(tx, ty) % 7 == 0 {
                TileTallGrass
            } else if scatter(tx, ty) % 11 == 3 {
                TileFlowers
            } else {
                TileGrass
            };
            picture.draw(&sheet(id), scatter(tx, ty), tx * 16 * scale, ty * 16 * scale, scale);
        }
    }
    // Back-to-front so nearer things overlap farther ones.
    let mut actors = vec![
        (Tree, 0, 24, 40),
        (Tree, 1, 56, 34),
        (Tree, 0, 150, 30),
        (Tree, 1, 330, 60),
        (Tree, 0, 20, 150),
        (Tree, 1, 120, 190),
        (Rock, 0, 100, 60),
        (Rock, 1, 210, 150),
        (Ruin, 0, 190, 60),
        (DragonMoss, 0, 270, 90),
        (Slime, 0, 60, 120),
        (Bat, 0, 90, 150),
        (Goblin, 0, 200, 110),
        (Wyrmling, 0, 240, 190),
        (HeroIdle, 0, 130, 108),
        (CompanionFledgling, 0, 112, 100),
        (SlashArc, 1, 150, 114),
        (EmberOrb, 0, 170, 130),
        (Heart, 0, 182, 130),
        (Firebolt, 0, 150, 90),
        (SlingStone, 0, 180, 104),
    ];
    actors.sort_by_key(|&(_, _, _, y)| y);
    for (id, frame, x, y) in actors {
        if !matches!(id, Tree | Rock | Ruin | SlashArc | EmberOrb | Heart | Firebolt | SlingStone) {
            picture.stand(Shadow, 0, x * scale, (y + 2) * scale, scale);
        }
        picture.stand(id, frame, x * scale, y * scale, scale);
    }
    picture
}

fn main() -> std::io::Result<()> {
    use SpriteId::*;
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: preview <out.bmp> [all|actors|dragons|world|scene]");
    let group = args.next().unwrap_or_else(|| "all".into());
    let wants = |name: &str| group == "all" || group == name;

    let mut picture = Picture::new(1, 0);
    if wants("actors") {
        picture.append(contact_sheet(
            &[
                HeroIdle,
                HeroWalk,
                SlashArc,
                CompanionHatchling,
                CompanionFledgling,
                CompanionDrake,
                Slime,
                Bat,
                Goblin,
                Wyrmling,
            ],
            6,
            1120,
        ));
        picture.append(contact_sheet(
            &[Firebolt, VenomSpit, FrostShard, AshFlame, SlingStone, EmberOrb, Heart, Spark, Puff, Shadow],
            6,
            1120,
        ));
    }
    if wants("dragons") {
        picture.append(contact_sheet(&[DragonMoss, DragonFrost, DragonAshen], 4, 1120));
    }
    if wants("world") {
        picture.append(contact_sheet(
            &[TileGrass, TileTallGrass, TileFlowers, TileDirt, TileSand, TileWater, Tree, Rock, Ruin],
            4,
            1120,
        ));
        picture.append(tile_fields(3));
    }
    if wants("scene") {
        picture.append(scene(3));
    }
    picture.write_bmp(&path)
}
