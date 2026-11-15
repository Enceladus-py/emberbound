//! The look of the game: palette, font sizes and UI scaling.
//!
//! All UI is laid out in "design pixels" for a 720 pixel tall window;
//! [`UiScale`] stretches that to whatever the window (or browser canvas)
//! actually is, so text stays readable from 800x450 up to 4K.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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

/// Font sizes in design pixels.
pub mod font {
    pub const LOGO: f32 = 112.0;
    pub const HEADLINE: f32 = 64.0;
    pub const TITLE: f32 = 34.0;
    pub const BODY: f32 = 22.0;
    pub const SMALL: f32 = 17.0;
}

/// Stacking order of the full-screen UI layers.
pub mod z {
    pub const HUD: i32 = 0;
    pub const BOONS: i32 = 10;
    pub const BANNER: i32 = 20;
}

/// Window height the layouts are designed for.
const DESIGN_HEIGHT: f32 = 720.0;
/// Narrowest design width that still fits every layout; narrower windows
/// (portrait phones) shrink the UI further instead of clipping it.
const DESIGN_MIN_WIDTH: f32 = 960.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, fit_ui_to_window);
}

fn fit_ui_to_window(window: Single<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    let fit = (window.height() / DESIGN_HEIGHT).min(window.width() / DESIGN_MIN_WIDTH).clamp(0.4, 6.0);
    if (scale.0 - fit).abs() > 0.005 {
        scale.0 = fit;
    }
}
