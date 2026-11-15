//! Sprite definitions. Placeholder art until the real sheets land.

use crate::canvas::{Canvas, rgb};
use crate::{SpriteId, SpriteSheet};

pub fn sheet(id: SpriteId) -> SpriteSheet {
    let (w, h, color) = match id {
        SpriteId::SlashArc => (24, 24, rgb(0xffffff)),
        SpriteId::CompanionHatchling => (12, 12, rgb(0xff8844)),
        SpriteId::CompanionDrake => (20, 20, rgb(0xff8844)),
        SpriteId::DragonMoss | SpriteId::DragonFrost => (48, 48, rgb(0x44aa66)),
        SpriteId::DragonAshen => (64, 64, rgb(0x883333)),
        SpriteId::Tree => (16, 32, rgb(0x226633)),
        SpriteId::Ruin => (16, 24, rgb(0x888899)),
        SpriteId::Firebolt
        | SpriteId::VenomSpit
        | SpriteId::FrostShard
        | SpriteId::AshFlame
        | SpriteId::SlingStone
        | SpriteId::EmberOrb
        | SpriteId::Heart
        | SpriteId::Spark => (8, 8, rgb(0xffcc44)),
        SpriteId::Shadow => (12, 4, [0, 0, 0, 90]),
        SpriteId::TileGrass | SpriteId::TileTallGrass | SpriteId::TileFlowers => (16, 16, rgb(0x4c9a4a)),
        SpriteId::TileDirt => (16, 16, rgb(0x9a7a4a)),
        SpriteId::TileSand => (16, 16, rgb(0xd8c88a)),
        SpriteId::TileWater => (16, 16, rgb(0x3a6ac8)),
        _ => (16, 16, rgb(0xcc66cc)),
    };
    SpriteSheet::from_frames(vec![Canvas::filled(w, h, color), Canvas::filled(w, h, color)])
}
