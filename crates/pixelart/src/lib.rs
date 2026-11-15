//! # Emberbound pixel art
//!
//! Every sprite in the game is authored here, in code, as tiny palette-indexed
//! grids that are rasterised to RGBA at start-up. The crate is engine-free:
//! it only knows how to turn a [`SpriteId`] into pixels. The Bevy layer
//! uploads those pixels to the GPU.
//!
//! Conventions: characters face **right** (the engine flips them), frames of
//! one sprite all share the same size, and pixels are RGBA8, row-major,
//! top row first.

mod canvas;
mod sheets;

pub use canvas::{Canvas, Rgba};

/// Identifies every sprite (possibly animated) in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpriteId {
    // --- Hero (16x16) ---
    HeroIdle,
    HeroWalk,
    /// Crescent sword-swing effect pointing right (24x24).
    SlashArc,
    // --- Dragon companion (12x12, 16x16, 20x20) ---
    CompanionHatchling,
    CompanionFledgling,
    CompanionDrake,
    // --- Lesser monsters (16x16) ---
    Slime,
    Bat,
    Goblin,
    Wyrmling,
    // --- Elder dragons (48x48, 48x48, 64x64) ---
    DragonMoss,
    DragonFrost,
    DragonAshen,
    // --- Ground tiles (16x16; frames are random variants, water animates) ---
    TileGrass,
    TileTallGrass,
    TileFlowers,
    TileDirt,
    TileSand,
    TileWater,
    // --- Obstacles drawn on top of a ground tile ---
    /// 16x32, trunk at the bottom centre.
    Tree,
    /// 16x16.
    Rock,
    /// 16x24 broken pillar.
    Ruin,
    // --- Projectiles (8x8) ---
    Firebolt,
    VenomSpit,
    FrostShard,
    AshFlame,
    SlingStone,
    // --- Pickups (8x8) ---
    EmberOrb,
    Heart,
    // --- Effects ---
    /// 8x8 hit spark burst.
    Spark,
    /// 16x16 smoke puff on death.
    Puff,
    /// 12x4 soft ground shadow.
    Shadow,
}

impl SpriteId {
    pub const ALL: [SpriteId; 32] = [
        SpriteId::HeroIdle,
        SpriteId::HeroWalk,
        SpriteId::SlashArc,
        SpriteId::CompanionHatchling,
        SpriteId::CompanionFledgling,
        SpriteId::CompanionDrake,
        SpriteId::Slime,
        SpriteId::Bat,
        SpriteId::Goblin,
        SpriteId::Wyrmling,
        SpriteId::DragonMoss,
        SpriteId::DragonFrost,
        SpriteId::DragonAshen,
        SpriteId::TileGrass,
        SpriteId::TileTallGrass,
        SpriteId::TileFlowers,
        SpriteId::TileDirt,
        SpriteId::TileSand,
        SpriteId::TileWater,
        SpriteId::Tree,
        SpriteId::Rock,
        SpriteId::Ruin,
        SpriteId::Firebolt,
        SpriteId::VenomSpit,
        SpriteId::FrostShard,
        SpriteId::AshFlame,
        SpriteId::SlingStone,
        SpriteId::EmberOrb,
        SpriteId::Heart,
        SpriteId::Spark,
        SpriteId::Puff,
        SpriteId::Shadow,
    ];
}

/// The frames of one sprite. All frames are `frame_width` x `frame_height`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteSheet {
    pub frame_width: u32,
    pub frame_height: u32,
    /// RGBA8 pixels per frame, row-major, top row first.
    pub frames: Vec<Vec<u8>>,
}

impl SpriteSheet {
    pub fn from_frames(frames: Vec<Canvas>) -> Self {
        assert!(!frames.is_empty(), "a sprite needs at least one frame");
        let (frame_width, frame_height) = (frames[0].width(), frames[0].height());
        assert!(
            frames.iter().all(|f| f.width() == frame_width && f.height() == frame_height),
            "all frames of a sprite must share one size"
        );
        Self { frame_width, frame_height, frames: frames.into_iter().map(Canvas::into_rgba).collect() }
    }

    /// All frames side by side as one RGBA8 strip: `(width, height, pixels)`.
    pub fn to_strip(&self) -> (u32, u32, Vec<u8>) {
        let (fw, fh, n) = (self.frame_width as usize, self.frame_height as usize, self.frames.len());
        let mut strip = vec![0u8; fw * n * fh * 4];
        for (i, frame) in self.frames.iter().enumerate() {
            for row in 0..fh {
                let src = row * fw * 4;
                let dst = (row * fw * n + i * fw) * 4;
                strip[dst..dst + fw * 4].copy_from_slice(&frame[src..src + fw * 4]);
            }
        }
        ((fw * n) as u32, fh as u32, strip)
    }
}

/// Rasterises the sprite for `id`.
pub fn sheet(id: SpriteId) -> SpriteSheet {
    sheets::sheet(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sprite_rasterises_with_consistent_frames() {
        for id in SpriteId::ALL {
            let sheet = sheet(id);
            assert!(!sheet.frames.is_empty(), "{id:?} has no frames");
            let expected = (sheet.frame_width * sheet.frame_height * 4) as usize;
            assert!(sheet.frames.iter().all(|f| f.len() == expected), "{id:?} frame size mismatch");
            assert!(
                sheet.frames.iter().all(|f| f.chunks(4).any(|px| px[3] > 0)),
                "{id:?} has an empty frame"
            );
        }
    }

    #[test]
    fn strips_place_frames_side_by_side() {
        let mut a = Canvas::new(2, 1);
        a.set(0, 0, [1, 2, 3, 255]);
        let mut b = Canvas::new(2, 1);
        b.set(1, 0, [9, 8, 7, 255]);
        let (w, h, px) = SpriteSheet::from_frames(vec![a, b]).to_strip();
        assert_eq!((w, h), (4, 1));
        assert_eq!(&px[0..4], &[1, 2, 3, 255]);
        assert_eq!(&px[12..16], &[9, 8, 7, 255]);
    }
}
