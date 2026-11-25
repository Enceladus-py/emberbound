//! Sprite definitions, grouped by what they depict.

mod companion;
mod dragons;
mod effects;
mod glyphs;
mod hero;
mod monsters;
mod projectiles;
mod props;
mod tiles;

use crate::{SpriteId, SpriteSheet};

pub fn sheet(id: SpriteId) -> SpriteSheet {
    let frames = match id {
        SpriteId::HeroIdle => hero::idle(),
        SpriteId::HeroWalk => hero::walk(),
        SpriteId::SlashArc => effects::slash_arc(),
        SpriteId::CompanionHatchling => companion::hatchling(),
        SpriteId::CompanionFledgling => companion::fledgling(),
        SpriteId::CompanionDrake => companion::drake(),
        SpriteId::Slime => monsters::slime(),
        SpriteId::Bat => monsters::bat(),
        SpriteId::Goblin => monsters::goblin(),
        SpriteId::Wyrmling => monsters::wyrmling(),
        SpriteId::DragonMoss => dragons::moss(),
        SpriteId::DragonFrost => dragons::frost(),
        SpriteId::DragonAshen => dragons::ashen(),
        SpriteId::TileGrass => tiles::grass(),
        SpriteId::TileTallGrass => tiles::tall_grass(),
        SpriteId::TileFlowers => tiles::flowers(),
        SpriteId::TileDirt => tiles::dirt(),
        SpriteId::TileSand => tiles::sand(),
        SpriteId::TileWater => tiles::water(),
        SpriteId::Tree => props::tree(),
        SpriteId::Rock => props::rock(),
        SpriteId::Ruin => props::ruin(),
        SpriteId::Firebolt => projectiles::firebolt(),
        SpriteId::VenomSpit => projectiles::venom_spit(),
        SpriteId::FrostShard => projectiles::frost_shard(),
        SpriteId::AshFlame => projectiles::ash_flame(),
        SpriteId::SlingStone => projectiles::sling_stone(),
        SpriteId::EmberOrb => effects::ember_orb(),
        SpriteId::Heart => effects::heart(),
        SpriteId::Spark => effects::spark(),
        SpriteId::Puff => effects::puff(),
        SpriteId::Shadow => effects::shadow(),
        SpriteId::FirePatch => glyphs::fire_patch(),
        SpriteId::SlashWave => glyphs::slash_wave(),
        SpriteId::AimPointer => effects::aim_pointer(),
        SpriteId::AimMark => effects::aim_mark(),
        SpriteId::BoonGlyphs => glyphs::boon_glyphs(),
    };
    SpriteSheet::from_frames(frames)
}

/// A small deterministic integer hash: the only "randomness" the art uses,
/// so every build paints exactly the same pixels.
fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}
