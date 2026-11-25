//! The mouse as an aiming device: the sword points at the cursor, but only
//! while the player is actually using the mouse. A keyboard-only player
//! whose cursor happens to rest over the canvas keeps aiming by facing.

use bevy::prelude::*;
use bevy::window::{CursorMoved, PrimaryWindow};
use emberbound_core::{InputMode, MainCamera, Player};

/// Seconds after the last mouse movement or click during which the cursor
/// still aims.
pub(super) const MOUSE_AIM_SECONDS: f32 = 2.0;
/// Closer to the hero than this (world units), the cursor gives no aim.
const AIM_DEAD_ZONE: f32 = 4.0;

/// When the mouse was last moved or clicked, in seconds of real time.
#[derive(Resource, Debug, Default)]
pub(super) struct MouseActivity {
    last: Option<f32>,
}

impl MouseActivity {
    fn note(&mut self, now: f32) {
        self.last = Some(now);
    }

    /// Whether the mouse counts as the aiming device at `now`.
    pub(super) fn is_active(&self, now: f32) -> bool {
        self.last.is_some_and(|last| now - last <= MOUSE_AIM_SECONDS)
    }
}

pub(super) fn track_activity(
    time: Res<Time<Real>>,
    mut moved: MessageReader<CursorMoved>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut activity: ResMut<MouseActivity>,
) {
    // Only real motion counts: a cursor event without a delta is the
    // pointer entering the window, not the player reaching for the mouse.
    let moved = moved.read().any(|event| event.delta.is_some_and(|delta| delta != Vec2::ZERO));
    if moved || buttons.get_just_pressed().next().is_some() {
        activity.note(time.elapsed_secs());
    }
}

/// The direction from the hero to the cursor, while the mouse is in use.
pub(super) fn aim(
    time: &Time<Real>,
    activity: &MouseActivity,
    mode: InputMode,
    window: Option<&Window>,
    camera: Option<(&Camera, &GlobalTransform)>,
    hero: Option<&GlobalTransform>,
) -> Option<Vec2> {
    // Browsers turn touches into mouse events too; fingers aim their own way.
    if mode == InputMode::Touch || !activity.is_active(time.elapsed_secs()) {
        return None;
    }
    let cursor = window?.cursor_position()?;
    let (camera, camera_transform) = camera?;
    let target = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    let offset = target - hero?.translation().truncate();
    (offset.length() > AIM_DEAD_ZONE).then(|| offset.normalize())
}

/// Everything [`aim`] needs from the world, bundled for the input system.
pub(super) type AimQueries<'w, 's> = (
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<MainCamera>>,
    Query<'w, 's, &'static GlobalTransform, With<Player>>,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mouse_aims_only_for_a_while_after_it_was_used() {
        let mut activity = MouseActivity::default();
        assert!(!activity.is_active(0.0), "never used");
        activity.note(10.0);
        assert!(activity.is_active(10.0));
        assert!(activity.is_active(10.0 + MOUSE_AIM_SECONDS));
        assert!(!activity.is_active(10.1 + MOUSE_AIM_SECONDS), "a parked mouse stops aiming");
    }

    #[test]
    fn touch_players_never_aim_with_the_cursor() {
        let mut activity = MouseActivity::default();
        activity.note(0.0);
        let time = Time::<Real>::default();
        assert_eq!(aim(&time, &activity, InputMode::Touch, None, None, None), None);
    }
}
