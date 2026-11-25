//! The look of the game: palette, the pixel font and its sizes, and UI
//! scaling.
//!
//! All UI is laid out in "design pixels" for a 720 pixel tall window;
//! [`UiScale`] stretches that to whatever the window (or browser canvas)
//! actually is, so text stays readable from 800x450 up to 4K.

use bevy::asset::uuid_handle;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy::window::PrimaryWindow;
use emberbound_pixelart::pixel_font_ttf;

pub const NIGHT: Color = Color::srgb_u8(0x12, 0x10, 0x1a);
pub const PANEL: Color = Color::srgb_u8(0x1f, 0x1b, 0x2e);
pub const PANEL_BORDER: Color = Color::srgb_u8(0x3a, 0x33, 0x52);
pub const PARCHMENT: Color = Color::srgb_u8(0xf4, 0xe9, 0xd8);
pub const MUTED: Color = Color::srgb_u8(0xa5, 0x9f, 0xb8);
pub const EMBER: Color = Color::srgb_u8(0xf2, 0xa6, 0x5a);
pub const FLAME: Color = Color::srgb_u8(0xe4, 0x57, 0x2e);
pub const HEALTH: Color = Color::srgb_u8(0xd6, 0x45, 0x50);
pub const XP: Color = Color::srgb_u8(0xf7, 0xd3, 0x54);
pub const FROST: Color = Color::srgb_u8(0x7f, 0xd1, 0xe8);
pub const MOSS: Color = Color::srgb_u8(0x6f, 0xbf, 0x73);

/// Font sizes in design pixels. Each is a whole multiple of the pixel
/// font's em ([`emberbound_pixelart::EM_PIXELS`] font pixels), so every font
/// pixel is a whole, square number of design pixels.
pub mod font {
    pub const LOGO: f32 = 112.0;
    pub const HEADLINE: f32 = 64.0;
    pub const TITLE: f32 = 32.0;
    pub const BODY: f32 = 24.0;
    pub const SMALL: f32 = 16.0;
}

/// The game's pixel font (see `emberbound_pixelart::pixel_font_ttf`),
/// generated at start-up. Text gets it through [`text_font`].
pub const PIXEL_FONT: Handle<Font> = uuid_handle!("6a0d3c1e-5b7f-4f2e-9a43-8e1d2c7b5f90");

/// The pixel font at `size`, drawn without smoothing so its edges stay sharp.
pub fn text_font(size: f32) -> TextFont {
    TextFont {
        font: PIXEL_FONT.into(),
        font_size: FontSize::Px(size),
        font_smoothing: FontSmoothing::None,
        ..default()
    }
}

/// Stacking order of the full-screen UI layers.
pub mod z {
    pub const HUD: i32 = 0;
    pub const PAD: i32 = 5;
    pub const BOONS: i32 = 10;
    pub const BANNER: i32 = 20;
}

/// Window height the layouts are designed for.
const DESIGN_HEIGHT: f32 = 720.0;
/// Narrowest design width that still fits every layout; narrower windows
/// (portrait phones) shrink the UI further instead of clipping it.
const DESIGN_MIN_WIDTH: f32 = 960.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, load_pixel_font).add_systems(PreUpdate, fit_ui_to_window);
}

fn load_pixel_font(mut fonts: ResMut<Assets<Font>>) {
    fonts.insert(&PIXEL_FONT, Font::from_bytes(pixel_font_ttf())).expect("the font handle is a constant");
}

fn fit_ui_to_window(window: Single<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    let fit = (window.height() / DESIGN_HEIGHT).min(window.width() / DESIGN_MIN_WIDTH).clamp(0.4, 6.0);
    if (scale.0 - fit).abs() > 0.005 {
        scale.0 = fit;
    }
}
