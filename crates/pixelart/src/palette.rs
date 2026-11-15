//! The one shared palette. Every sprite is painted with these 32 colours,
//! which is what makes the art read as one world.

use crate::canvas::{Canvas, Rgba, rgb};

/// Character outlines: a deep plum instead of pure black.
pub const OUTLINE: Rgba = rgb(0x2b1b33);
pub const COAL: Rgba = rgb(0x1d1a24);
pub const STONE_DARK: Rgba = rgb(0x45414f);
pub const STONE: Rgba = rgb(0x73738a);
pub const STONE_LIGHT: Rgba = rgb(0xa3a8b8);
pub const STEEL: Rgba = rgb(0xd3d9e2);
pub const WHITE: Rgba = rgb(0xfbf7ec);
pub const MAROON: Rgba = rgb(0x7c2238);
pub const RED: Rgba = rgb(0xc23a3e);
pub const CORAL: Rgba = rgb(0xee6a4c);
pub const ORANGE: Rgba = rgb(0xf5932e);
pub const GOLD: Rgba = rgb(0xfacd4e);
pub const CREAM: Rgba = rgb(0xfdf1b0);
pub const BARK_DARK: Rgba = rgb(0x54362e);
pub const BARK: Rgba = rgb(0x8a5a3a);
pub const TAN: Rgba = rgb(0xbc8c58);
pub const SAND: Rgba = rgb(0xdcc084);
pub const SAND_LIGHT: Rgba = rgb(0xefdba6);
pub const SKIN: Rgba = rgb(0xf2bd96);
pub const GREEN_DEEP: Rgba = rgb(0x1e5440);
pub const GREEN: Rgba = rgb(0x38824a);
pub const GRASS_SHADE: Rgba = rgb(0x4f9c4a);
pub const GRASS: Rgba = rgb(0x5aa84e);
pub const LIME: Rgba = rgb(0x98d660);
pub const VENOM: Rgba = rgb(0xcff257);
pub const NAVY: Rgba = rgb(0x263468);
pub const BLUE: Rgba = rgb(0x3b6cba);
pub const SKY: Rgba = rgb(0x5fafe4);
pub const ICE: Rgba = rgb(0xaee3f2);
pub const FROST: Rgba = rgb(0xe6f9fc);
pub const PLUM: Rgba = rgb(0x5c2c6e);
pub const VIOLET: Rgba = rgb(0x9c4eb4);

/// The ASCII-art key shared by all hand-drawn sprites.
const KEY: [(char, Rgba); 32] = [
    ('o', OUTLINE),
    ('k', COAL),
    ('1', STONE_DARK),
    ('2', STONE),
    ('3', STONE_LIGHT),
    ('4', STEEL),
    ('w', WHITE),
    ('m', MAROON),
    ('r', RED),
    ('R', CORAL),
    ('O', ORANGE),
    ('y', GOLD),
    ('Y', CREAM),
    ('b', BARK_DARK),
    ('B', BARK),
    ('t', TAN),
    ('s', SAND),
    ('S', SAND_LIGHT),
    ('f', SKIN),
    ('g', GREEN_DEEP),
    ('G', GREEN),
    ('h', GRASS_SHADE),
    ('l', GRASS),
    ('L', LIME),
    ('v', VENOM),
    ('n', NAVY),
    ('u', BLUE),
    ('U', SKY),
    ('i', ICE),
    ('I', FROST),
    ('p', PLUM),
    ('P', VIOLET),
];

/// Rasterises ASCII art drawn with the shared [`KEY`]; `.` is transparent.
pub fn art(rows: &[&str]) -> Canvas {
    assert!(
        rows.iter().all(|row| row.chars().count() == rows[0].chars().count()),
        "ragged sprite art: {rows:?}"
    );
    Canvas::from_art(rows, &KEY)
}

/// Like [`art`], with the plum character outline added around the shape.
pub fn outlined_art(rows: &[&str]) -> Canvas {
    art(rows).outlined(OUTLINE)
}
