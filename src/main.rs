//! # Emberbound
//!
//! Composition root: wires the engine, the shared kernel and one plugin per
//! bounded context together. The only place that knows every crate.

#[cfg(feature = "cheats")]
mod cheats;
mod input;
mod records;
mod session;

use bevy::prelude::*;
use emberbound_combat::CombatPlugin;
use emberbound_core::CorePlugin;
use emberbound_monsters::MonstersPlugin;
use emberbound_ui::UiPlugin;
use emberbound_world::WorldPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            // Crisp pixels: never blur sprites when scaling.
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Emberbound".into(),
                    canvas: Some("#game".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb_u8(0x12, 0x10, 0x1a)))
    .add_plugins(CorePlugin)
    .add_plugins((input::InputPlugin, session::SessionPlugin, records::RecordsPlugin))
    .add_plugins((WorldPlugin, CombatPlugin, MonstersPlugin, UiPlugin));
    #[cfg(feature = "cheats")]
    app.add_plugins(cheats::CheatsPlugin);
    app.run();
}
