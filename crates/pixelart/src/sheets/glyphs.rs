//! Boon icons and boon effect sprites. Placeholder shapes until the real
//! art lands.

use crate::BoonGlyph;
use crate::canvas::{Canvas, rgb};

pub fn fire_patch() -> Vec<Canvas> {
    vec![Canvas::filled(16, 16, rgb(0xe4572e)), Canvas::filled(16, 16, rgb(0xf2a65a))]
}

pub fn slash_wave() -> Vec<Canvas> {
    vec![Canvas::filled(16, 16, rgb(0xf4e9d8)), Canvas::filled(16, 16, rgb(0xf7d354))]
}

pub fn boon_glyphs() -> Vec<Canvas> {
    BoonGlyph::ALL
        .iter()
        .map(|glyph| {
            let mut icon = Canvas::new(24, 24);
            let shade = 0x40 + (*glyph as u32 * 8);
            icon.fill_rect(4, 4, 16, 16, rgb(0xf2a600 | shade));
            icon
        })
        .collect()
}
