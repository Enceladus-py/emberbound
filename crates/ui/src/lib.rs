//! # Emberbound UI
//!
//! Everything the player reads rather than fights: the title, boon, pause
//! and end screens, the HUD, announcement banners and cosmetic world-space feedback
//! (damage numbers, hit flashes, particles). One module per concern; they
//! share only [`theme`] and [`widgets`].
//!
//! The UI never touches devices: it reads [`emberbound_core::PlayerIntent`]
//! and observes the game through core's components, resources and messages.

mod announce;
mod boon_strip;
mod boons;
mod end_screens;
mod fx;
mod hud;
mod minimap;
mod pad;
mod pause;
mod theme;
mod title;
mod widgets;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            theme::plugin,
            widgets::plugin,
            title::plugin,
            hud::plugin,
            minimap::plugin,
            pad::plugin,
            boons::plugin,
            pause::plugin,
            boon_strip::plugin,
            end_screens::plugin,
            announce::plugin,
            fx::plugin,
        ));
    }
}
