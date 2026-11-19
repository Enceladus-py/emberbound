//! Which device the player is using, so the UI can speak their language.

use bevy::prelude::*;
use emberbound_core::InputMode;

/// Starts in touch mode on phones and tablets, before the first tap, so
/// the title screen already says "tap" instead of "press enter".
pub(super) fn guess_input_mode(mut mode: ResMut<InputMode>) {
    if primary_pointer_is_coarse() {
        *mode = InputMode::Touch;
    }
}

/// Switches mode whenever the other kind of device is used.
pub(super) fn follow_last_device(
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut mode: ResMut<InputMode>,
) {
    let next = if touches.any_just_pressed() {
        InputMode::Touch
    } else if keys.get_just_pressed().next().is_some() {
        InputMode::Keyboard
    } else {
        return;
    };
    mode.set_if_neq(next);
}

#[cfg(target_arch = "wasm32")]
fn primary_pointer_is_coarse() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media("(pointer: coarse)").ok().flatten())
        .is_some_and(|query| query.matches())
}

#[cfg(not(target_arch = "wasm32"))]
fn primary_pointer_is_coarse() -> bool {
    false
}
