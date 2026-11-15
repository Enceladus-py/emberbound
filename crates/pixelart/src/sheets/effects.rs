//! Sword slash, pickups and short-lived effects.

use super::hash;
use crate::canvas::Canvas;
use crate::palette::{CREAM, GOLD, ORANGE, STEEL, STONE_LIGHT, WHITE, art};

/// A crescent bulging to the right. `sweep` is how far the blade has
/// travelled (the arc grows from the top), `thickness` thins it as it fades.
fn crescent(sweep: f32, thickness: f32, alpha: u8, seed: u32) -> Canvas {
    let mut frame = Canvas::new(24, 24);
    let (cx, cy, outer) = (5.5_f32, 12.0_f32, 11.0_f32);
    for y in 0..24 {
        for x in 0..24 {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let dist = dx.hypot(dy);
            // Angle along the arc: 0 at the top tip, 1 at the bottom tip.
            let along = (dy.atan2(dx) / std::f32::consts::PI + 0.5).clamp(0.0, 1.0);
            // The blade is thickest mid-swing and tapers to both tips.
            let width = thickness * (1.0 - (2.0 * along - 1.0).powi(2)).max(0.0).sqrt();
            if dx < 0.0 || along > sweep || dist > outer || dist < outer - width {
                continue;
            }
            let depth = (outer - dist) / width.max(0.01);
            let color = if depth < 0.34 {
                WHITE
            } else if depth < 0.62 {
                CREAM
            } else if depth < 0.85 {
                GOLD
            } else {
                ORANGE
            };
            // Later frames break up into embers.
            if alpha < 255 && hash(x as u32, y as u32, seed) % 255 > alpha as u32 {
                continue;
            }
            frame.set(x, y, [color[0], color[1], color[2], alpha.max(120)]);
        }
    }
    frame
}

pub fn slash_arc() -> Vec<Canvas> {
    vec![crescent(0.55, 5.0, 255, 1), crescent(1.0, 5.5, 255, 2), crescent(1.0, 3.6, 200, 3), crescent(1.0, 2.2, 110, 4)]
}

pub fn ember_orb() -> Vec<Canvas> {
    vec![
        art(&[
            "........",
            "..rOOr..",
            ".rOyyOr.",
            ".OyYwyO.",
            ".OyYYyO.",
            ".rOyyOr.",
            "..rOOr..",
            "........",
        ]),
        art(&[
            "...rr...",
            "..OOOO..",
            ".OyyyyO.",
            "rOyYwyOr",
            "rOyYYyOr",
            ".OyyyyO.",
            "..OOOO..",
            "...rr...",
        ]),
        art(&[
            "...OO...",
            "..OyyO..",
            ".OyYYyO.",
            "OyYwwYyO",
            "OyYwYYyO",
            ".OyYYyO.",
            "..OyyO..",
            "...OO...",
        ]),
        art(&[
            "...rr...",
            "..OOOO..",
            ".OyyyyO.",
            "rOyYwyOr",
            "rOyYYyOr",
            ".OyyyyO.",
            "..OOOO..",
            "...rr...",
        ]),
    ]
}

pub fn heart() -> Vec<Canvas> {
    let heart = art(&[
        "........",
        ".mm.mm..",
        "mRRmRRm.",
        "mRwRRRm.",
        "mRRRRrm.",
        ".mRRrm..",
        "..mrm...",
        "...m....",
    ]);
    vec![heart.shifted(0, -1), heart]
}

pub fn spark() -> Vec<Canvas> {
    vec![
        art(&[
            "........",
            "........",
            "...YY...",
            "..YwwY..",
            "..YwwY..",
            "...YY...",
            "........",
            "........",
        ]),
        art(&[
            "...yy...",
            ".y.YY.y.",
            "..YwwY..",
            "yYwwwwYy",
            "yYwwwwYy",
            "..YwwY..",
            ".y.YY.y.",
            "...yy...",
        ]),
        art(&[
            "y..OO..y",
            "...yy...",
            "..y..y..",
            "Oy....yO",
            "Oy....yO",
            "..y..y..",
            "...yy...",
            "y..OO..y",
        ]),
        art(&[
            "O......O",
            "........",
            "...O....",
            "........",
            "....O...",
            "........",
            "........",
            "O......O",
        ]),
    ]
}

/// A smoke cloud made of a few lobes that drift apart and thin out.
pub fn puff() -> Vec<Canvas> {
    const LOBES: [(f32, f32, f32); 5] = [(0.0, 0.0, 1.0), (-1.0, -0.4, 0.75), (0.9, -0.6, 0.8), (0.5, 0.8, 0.7), (-0.7, 0.8, 0.65)];
    [(1.2, 2.6, 255), (2.4, 3.0, 255), (3.6, 2.6, 190), (4.6, 1.7, 110)]
        .into_iter()
        .map(|(spread, radius, alpha): (f32, f32, u8)| {
            let mut cloud = Canvas::new(16, 16);
            for (dx, dy, size) in LOBES {
                cloud.disc(8.0 + dx * spread, 8.0 + dy * spread, radius * size, STEEL);
            }
            cloud.bevel(STEEL, WHITE, STONE_LIGHT, 1, 1).with_alpha(alpha)
        })
        .collect()
}

/// A soft translucent ground shadow.
pub fn shadow() -> Vec<Canvas> {
    let mut shadow = Canvas::new(12, 4);
    shadow.ellipse(6.0, 2.0, 6.0, 2.0, [20, 12, 28, 90]);
    vec![shadow]
}
