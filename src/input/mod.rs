//! Input adapter: translates keyboard, mouse and touch into the
//! device-independent [`PlayerIntent`] that gameplay reads, and keeps
//! [`emberbound_core::InputMode`] in step with the device the player last used.

mod device;
mod keyboard;
mod touch;

use bevy::prelude::*;
use emberbound_core::PlayerIntent;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<touch::Fingers>()
            .add_systems(Startup, device::guess_input_mode)
            .add_systems(PreUpdate, (device::follow_last_device, touch::read_touches, combine).chain());
    }
}

/// What the fingers asked for this frame, before merging with the keyboard.
#[derive(Debug, Clone, Copy, Default)]
struct TouchIntent {
    movement: Vec2,
    attack: bool,
    dash: bool,
    confirm: bool,
    tap: Option<Vec2>,
}

/// Merges every device into this frame's [`PlayerIntent`].
fn combine(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    fingers: Res<touch::Fingers>,
    mut intent: ResMut<PlayerIntent>,
) {
    let keyboard = keyboard::read(&keys, &mouse);
    let touch = fingers.intent;
    let movement = if touch.movement != Vec2::ZERO { touch.movement } else { keyboard.movement };
    *intent = PlayerIntent {
        movement,
        attack: keyboard.attack || touch.attack,
        dash: keyboard.dash || touch.dash,
        confirm: keyboard.confirm || touch.confirm,
        pause: keyboard.pause,
        minimap: keyboard.minimap,
        choice: keyboard.choice,
        tap: touch.tap,
    };
}
