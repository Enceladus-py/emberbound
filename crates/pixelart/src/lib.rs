//! # Emberbound pixel art
//!
//! Every sprite in the game is authored here, in code, as tiny palette-indexed
//! grids that are rasterised to RGBA at start-up. The crate is engine-free:
//! it only knows how to turn a [`SpriteId`] into pixels, and the UI font
//! into a TrueType file ([`pixel_font_ttf`]). The Bevy layer uploads those
//! pixels to the GPU.
//!
//! Conventions: characters face **right** (the engine flips them), frames of
//! one sprite all share the same size, and pixels are RGBA8, row-major,
//! top row first.

mod canvas;
mod font;
mod palette;
mod sheets;

pub use canvas::{Canvas, Rgba};
pub use font::{CAP_HEIGHT, EM_PIXELS, pixel_font_ttf};

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
    // --- Boon effects ---
    /// 16x16 burning patch left on the ground (looping flicker).
    FirePatch,
    /// 16x16 crescent of sword-light flying RIGHT (projectile).
    SlashWave,
    // --- UI ---
    /// 24x24 boon card icons: one frame per [`BoonGlyph`], in its order.
    BoonGlyphs,
}

/// Pictures for boon cards. Frame `glyph as usize` of
/// [`SpriteId::BoonGlyphs`] shows it. The art crate knows nothing about
/// boons; the UI maps each boon to the glyph that fits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoonGlyph {
    /// A sword with a glint on a whetstone.
    Whetstone,
    /// A shield with a heart on it.
    ShieldHeart,
    /// A winged boot.
    WingedBoot,
    /// A sharp eye.
    Eye,
    /// A heart made of flame.
    FlameHeart,
    /// A single dragon wing.
    Wing,
    /// A horseshoe magnet pulling embers.
    Magnet,
    /// Overlapping dragon scales.
    Scales,
    /// A wide crescent sweep.
    Crescent,
    /// A small campfire.
    Campfire,
    /// A sword with speed lines.
    QuickSword,
    /// A bellows puffing flame.
    Bellows,
    /// Burning footprints.
    FlameSteps,
    /// A thorny vine.
    Thorns,
    /// A blade dripping a blood drop.
    BloodBlade,
    /// A flying crescent wave of light.
    Wave,
    /// Two flames side by side.
    TwinFlames,
    /// A cracked shield, still standing.
    CrackedShield,
    /// A heart of cracked, glowing lava.
    MoltenHeart,
    /// A burning phoenix feather.
    Feather,
    /// A small pile of embers like coins.
    EmberPile,
    /// A skull wreathed in fire.
    BurningSkull,
    /// A hearth with a heart in its fire.
    Hearth,
}

impl BoonGlyph {
    pub const ALL: [BoonGlyph; 23] = [
        BoonGlyph::Whetstone,
        BoonGlyph::ShieldHeart,
        BoonGlyph::WingedBoot,
        BoonGlyph::Eye,
        BoonGlyph::FlameHeart,
        BoonGlyph::Wing,
        BoonGlyph::Magnet,
        BoonGlyph::Scales,
        BoonGlyph::Crescent,
        BoonGlyph::Campfire,
        BoonGlyph::QuickSword,
        BoonGlyph::Bellows,
        BoonGlyph::FlameSteps,
        BoonGlyph::Thorns,
        BoonGlyph::BloodBlade,
        BoonGlyph::Wave,
        BoonGlyph::TwinFlames,
        BoonGlyph::CrackedShield,
        BoonGlyph::MoltenHeart,
        BoonGlyph::Feather,
        BoonGlyph::EmberPile,
        BoonGlyph::BurningSkull,
        BoonGlyph::Hearth,
    ];
}

impl SpriteId {
    pub const ALL: [SpriteId; 35] = [
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
        SpriteId::FirePatch,
        SpriteId::SlashWave,
        SpriteId::BoonGlyphs,
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

    /// The frame size each sprite promises the engine.
    fn documented_size(id: SpriteId) -> (u32, u32) {
        use SpriteId::*;
        match id {
            SlashArc => (24, 24),
            CompanionHatchling => (12, 12),
            CompanionDrake => (20, 20),
            DragonMoss | DragonFrost => (48, 48),
            DragonAshen => (64, 64),
            Tree => (16, 32),
            Ruin => (16, 24),
            Firebolt | VenomSpit | FrostShard | AshFlame | SlingStone | EmberOrb | Heart | Spark => (8, 8),
            Shadow => (12, 4),
            BoonGlyphs => (24, 24),
            HeroIdle | HeroWalk | CompanionFledgling | Slime | Bat | Goblin | Wyrmling | TileGrass
            | TileTallGrass | TileFlowers | TileDirt | TileSand | TileWater | Rock | Puff | FirePatch
            | SlashWave => (16, 16),
        }
    }

    #[test]
    fn there_is_one_boon_icon_per_glyph() {
        assert_eq!(sheet(SpriteId::BoonGlyphs).frames.len(), BoonGlyph::ALL.len());
        assert!(BoonGlyph::ALL.iter().enumerate().all(|(i, glyph)| *glyph as usize == i));
    }

    #[test]
    fn every_sprite_has_its_documented_size() {
        for id in SpriteId::ALL {
            let sheet = sheet(id);
            assert_eq!((sheet.frame_width, sheet.frame_height), documented_size(id), "{id:?}");
        }
    }

    #[test]
    fn ground_tiles_are_fully_opaque() {
        use SpriteId::*;
        for id in [TileGrass, TileTallGrass, TileFlowers, TileDirt, TileSand, TileWater] {
            for (i, frame) in sheet(id).frames.iter().enumerate() {
                assert!(frame.chunks(4).all(|px| px[3] == 255), "{id:?} frame {i} has a see-through pixel");
            }
        }
    }

    #[test]
    fn animated_actors_have_at_least_two_frames() {
        use SpriteId::*;
        for id in [HeroIdle, HeroWalk, Slime, Bat, Goblin, Wyrmling, DragonMoss, DragonFrost, DragonAshen] {
            assert!(sheet(id).frames.len() >= 2, "{id:?} does not animate");
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
