//! Obstacles that stand on top of a ground tile, anchored bottom-centre.

use super::hash;
use crate::canvas::Canvas;
use crate::palette::{BARK, BARK_DARK, GRASS, GREEN, GREEN_DEEP, LIME, OUTLINE, TAN, outlined_art};

fn trunk(top: i32) -> Canvas {
    let mut trunk = Canvas::new(16, 32);
    trunk.fill_rect(6, top, 4, 31 - top, BARK);
    trunk.fill_rect(5, 29, 6, 2, BARK);
    trunk.fill_rect(8, top, 2, 31 - top, BARK_DARK);
    trunk.fill_rect(10, 30, 1, 1, BARK_DARK);
    for y in (top..29).step_by(3) {
        trunk.set(6, y, TAN);
    }
    trunk
}

/// A round oak: leaf clumps lit from the top-left.
fn oak() -> Canvas {
    const CLUMPS: [(f32, f32, f32); 7] = [
        (8.0, 18.5, 3.6),
        (5.0, 16.5, 3.9),
        (11.0, 16.5, 3.9),
        (8.0, 13.0, 5.5),
        (4.8, 11.0, 3.8),
        (11.2, 11.0, 3.8),
        (8.0, 7.5, 6.0),
    ];
    let mut tree = trunk(20);
    for (x, y, r) in CLUMPS {
        tree.disc(x, y, r, GREEN_DEEP);
        tree.disc(x - 0.4, y - 0.6, r - 0.9, GREEN);
        tree.disc(x - 1.1, y - 1.5, r - 2.3, GRASS);
    }
    for y in 0..24 {
        for x in 0..16 {
            if tree.get(x, y) == GRASS && hash(x as u32, y as u32, 5) % 4 == 0 {
                tree.set(x, y, LIME);
            }
        }
    }
    tree.outlined(OUTLINE)
}

/// A pine: three stacked tiers, each lit on its left flank.
fn pine() -> Canvas {
    let mut tree = trunk(24);
    for (top, bottom, half) in [(13.0, 26.0, 7.0), (7.0, 19.0, 5.6), (1.0, 12.0, 4.2_f32)] {
        tree.polygon(&[(8.0, top), (8.0 + half, bottom), (8.0 - half, bottom)], GREEN_DEEP);
        tree.polygon(&[(8.0, top + 1.5), (8.5, bottom - 1.0), (9.5 - half, bottom - 1.0)], GREEN);
        tree.line((7.5, top + 2.0), (9.5 - half * 0.8, bottom - 2.0), 1.0, GRASS);
    }
    tree.outlined(OUTLINE)
}

pub fn tree() -> Vec<Canvas> {
    vec![oak(), pine()]
}

pub fn rock() -> Vec<Canvas> {
    vec![
        outlined_art(&[
            "................",
            "................",
            "................",
            "................",
            ".....3333.......",
            "...334443322....",
            "..3344433322....",
            "..34433332221...",
            ".3343333222211..",
            ".3333332222211..",
            ".33332222221111.",
            ".23322222211111.",
            ".22222221111111.",
            "..222211111111..",
            "................",
            "................",
        ]),
        outlined_art(&[
            "................",
            "................",
            "................",
            "................",
            "................",
            "................",
            ".......lGl......",
            "......34G32.....",
            ".....3443322....",
            "..33.34333221...",
            ".3432333322211..",
            ".33223332222111.",
            ".32212222221111.",
            "..211.2211111l..",
            "................",
            "................",
        ]),
    ]
}

pub fn ruin() -> Vec<Canvas> {
    vec![outlined_art(&[
        "................",
        "................",
        ".....3..........",
        "....343..33.....",
        "....3433.343....",
        "....34333332....",
        "....3433l322....",
        "....343lGl22....",
        "....3433G221....",
        "....34332221....",
        "....3o332221....",
        "....34o32221....",
        "....34332221....",
        "....34332o21....",
        "....3433o221....",
        "...lG4332221....",
        "...Gl4332221....",
        "....34332221....",
        "...3444433222...",
        "..344444332221..",
        "..333333222211..",
        ".lG3322222211Gl.",
        ".GlGG1111111lGG.",
        "................",
    ])]
}
