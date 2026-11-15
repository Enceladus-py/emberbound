//! Projectiles: 8x8, pointing right, two flicker frames each.

use crate::canvas::Canvas;
use crate::palette::{CORAL, CREAM, GOLD, MAROON, ORANGE, PLUM, RED, WHITE, art, outlined_art};

/// A comet: bright head on the right, embers trailing behind.
const BOLT: [[&str; 8]; 2] = [
    ["........", "...r.OO.", ".r.rOyyO", "r.rOyYwY", ".rrOyYwY", "...rOyyO", ".r...OO.", "........"],
    ["........", ".r...OO.", "...rOyyO", ".rrOyYwY", "r.rOyYwY", ".r.rOyyO", "...r.OO.", "........"],
];

pub fn firebolt() -> Vec<Canvas> {
    BOLT.iter().map(|rows| art(rows)).collect()
}

/// The firebolt shape burning in the Ashen Tyrant's dark colours.
pub fn ash_flame() -> Vec<Canvas> {
    BOLT.iter()
        .map(|rows| {
            // Recolour from the outside in so no step repaints an earlier one.
            art(rows)
                .recolored(RED, PLUM)
                .recolored(ORANGE, MAROON)
                .recolored(GOLD, RED)
                .recolored(CREAM, CORAL)
                .recolored(WHITE, GOLD)
        })
        .collect()
}

pub fn venom_spit() -> Vec<Canvas> {
    vec![
        art(&[
            "........", "........", "g..GLL..", "..GLvvL.", ".gGLvwvL", "..GLvvLG", "g...GLG.", "........",
        ]),
        art(&[
            "........", ".g......", "...GLL..", ".gGLvvL.", "..GLvwvL", "g.GLvvLG", "....GLG.", "........",
        ]),
    ]
}

pub fn frost_shard() -> Vec<Canvas> {
    vec![
        art(&[
            "........", "..U.....", ".uUii...", "..UiIIi.", "uUiIwwwI", "..UiIIi.", ".uUii...", "..U.....",
        ]),
        art(&[
            "........", ".u......", "..Uii...", ".uUiIIi.", "..UiwwwI", ".uUiIIi.", "..Uii...", ".u......",
        ]),
    ]
}

pub fn sling_stone() -> Vec<Canvas> {
    vec![
        outlined_art(&[
            "........", "........", "...33...", "..3432..", "..3322..", "...21...", "........", "........",
        ]),
        outlined_art(&[
            "........", "........", "...32...", "..3332..", "..2432..", "...21...", "........", "........",
        ]),
    ]
}
