//! Boon card icons and the sprites of boon effects.

mod draw;
mod icons;

use crate::BoonGlyph;
use crate::canvas::{Canvas, Rgba};
use crate::palette::{CREAM, GOLD, MAROON, ORANGE, RED, WHITE};
use crate::sheets::hash;
use draw::{FIRE, arc, flame, paint};

/// `color` at a given opacity.
fn alpha(color: Rgba, a: u8) -> Rgba {
    [color[0], color[1], color[2], a]
}

/// A small patch of ground fire: an ember bed with flames licking up.
/// Four frames that loop.
pub fn fire_patch() -> Vec<Canvas> {
    // (height, lean) of the left, middle and right tongues per frame.
    const TONGUES: [[(f32, f32); 3]; 4] = [
        [(6.0, -0.15), (10.0, 0.05), (5.0, 0.2)],
        [(5.0, -0.05), (8.5, -0.1), (7.0, 0.1)],
        [(7.0, -0.2), (9.0, 0.12), (4.5, 0.25)],
        [(5.5, -0.1), (11.0, -0.05), (6.0, 0.15)],
    ];
    TONGUES
        .iter()
        .enumerate()
        .map(|(frame, tongues)| {
            let seed = frame as u32 + 11;
            let mut patch = Canvas::new(16, 16);
            paint(&mut patch, |x, y| {
                let d = ((x - 8.0) / 6.8).hypot((y - 12.8) / 2.6);
                let speck = hash(x as u32, y as u32, seed) % 7;
                match d {
                    d if d > 1.0 => None,
                    d if d > 0.8 => Some(alpha(MAROON, 130)),
                    d if d > 0.55 => Some(if speck == 0 { ORANGE } else { alpha(RED, 230) }),
                    _ => Some(match speck {
                        0 => CREAM,
                        1 | 2 => GOLD,
                        _ => ORANGE,
                    }),
                }
            });
            for (&(h, lean), (cx, base, w)) in
                tongues.iter().zip([(4.8, 13.0, 2.0), (8.0, 13.5, 3.0), (11.3, 13.0, 2.0)])
            {
                flame(&mut patch, cx, base, h, w, lean, &FIRE);
            }
            // Soften the outer flame edge and loose sparks.
            let mut soft = patch.recolored(RED, alpha(RED, 215));
            let spark = (frame as i32 * 5) % 9;
            soft.set(3 + spark, 2 + frame as i32 % 2, alpha(GOLD, 200));
            soft.set(12 - spark / 2, 4 - frame as i32 % 2, alpha(ORANGE, 160));
            soft
        })
        .collect()
}

/// A crescent of sword light flying right, convex side leading.
pub fn slash_wave() -> Vec<Canvas> {
    const LIGHT: [Rgba; 4] = [WHITE, CREAM, GOLD, ORANGE];
    [(5.6, 8.0, 0), (5.0, 8.3, 1), (6.0, 7.7, 2)]
        .into_iter()
        .map(|(thickness, r, frame)| {
            let mut wave = Canvas::new(16, 16);
            arc(&mut wave, 13.5 - r, 8.0, r, thickness, -1.15, 2.3, &LIGHT, 1.0);
            // Glittering motes left behind.
            for (i, (x, y)) in [(2, 5), (1, 9), (3, 12), (4, 3)].into_iter().enumerate() {
                if (i + frame) % 2 == 0 {
                    wave.set(x, y, alpha(CREAM, 170));
                }
            }
            wave
        })
        .collect()
}

pub fn boon_glyphs() -> Vec<Canvas> {
    BoonGlyph::ALL.iter().map(|&glyph| icons::icon(glyph)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boon_icons_keep_a_transparent_margin() {
        let touching: Vec<_> = BoonGlyph::ALL
            .iter()
            .zip(boon_glyphs())
            .filter(|(_, icon)| {
                (0..24).any(|i| [(i, 0), (i, 23), (0, i), (23, i)].iter().any(|&(x, y)| icon.get(x, y)[3] > 0))
            })
            .map(|(glyph, _)| *glyph)
            .collect();
        assert!(touching.is_empty(), "icons touching the frame edge: {touching:?}");
    }
}
