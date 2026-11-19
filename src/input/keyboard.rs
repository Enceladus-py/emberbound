//! Keyboard and mouse.

use bevy::prelude::*;
use emberbound_core::PlayerIntent;

/// This frame's intent from the keyboard and mouse alone.
pub(super) fn read(keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>) -> PlayerIntent {
    let axis = |negative: [KeyCode; 2], positive: [KeyCode; 2]| {
        keys.any_pressed(positive) as i8 as f32 - keys.any_pressed(negative) as i8 as f32
    };
    let movement = Vec2::new(
        axis([KeyCode::KeyA, KeyCode::ArrowLeft], [KeyCode::KeyD, KeyCode::ArrowRight]),
        axis([KeyCode::KeyS, KeyCode::ArrowDown], [KeyCode::KeyW, KeyCode::ArrowUp]),
    );

    PlayerIntent {
        movement: movement.normalize_or_zero(),
        attack: keys.any_pressed([KeyCode::Space, KeyCode::KeyJ]) || mouse.pressed(MouseButton::Left),
        dash: keys.any_just_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight, KeyCode::KeyK]),
        confirm: keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]),
        choice: [
            [KeyCode::Digit1, KeyCode::Numpad1],
            [KeyCode::Digit2, KeyCode::Numpad2],
            [KeyCode::Digit3, KeyCode::Numpad3],
        ]
        .iter()
        .position(|pair| keys.any_just_pressed(*pair)),
        tap: None,
    }
}
