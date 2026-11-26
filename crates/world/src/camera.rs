//! The main camera: follows the hero (inside the map, if it has an edge)
//! and shakes on impact.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use emberbound_core::{GameSet, GameState, MainCamera, Player, RunSetup, ScreenShake, Terrain, VIEW_HEIGHT};
use emberbound_domain::world::TileSource;

/// How quickly the camera closes the gap to the hero (1/s); the gap shrinks
/// by `e^(-FOLLOW_RATE * dt)` each frame, whatever the frame rate.
const FOLLOW_RATE: f32 = 9.0;
/// Shake offset in world units at full trauma.
const SHAKE_STRENGTH: f32 = 6.0;
/// Trauma lost per second.
const SHAKE_DECAY: f32 = 1.5;
/// Aspect ratio assumed while the window has no usable size.
const FALLBACK_ASPECT: f32 = 16.0 / 9.0;

/// Where the camera rests without shake. Kept apart from the camera's
/// `Transform` so that shake offsets never leak into the follow position.
#[derive(Resource, Debug, Default)]
pub(crate) struct CameraFocus(Vec2);

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { viewport_height: VIEW_HEIGHT },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Half of the world area the camera shows, from the window's aspect ratio.
fn half_view(window: Option<&Window>) -> Vec2 {
    let aspect = window
        .filter(|window| window.width() > 0.0 && window.height() > 0.0)
        .map_or(FALLBACK_ASPECT, |window| window.width() / window.height());
    Vec2::new(VIEW_HEIGHT * aspect, VIEW_HEIGHT) * 0.5
}

/// Keeps the view inside the map, if it has an edge. On an axis where the
/// map is smaller than the view, the map is centred instead.
fn clamp_to_map(target: Vec2, map: &dyn TileSource, half_view: Vec2) -> Vec2 {
    let Some((width, height)) = map.bounds() else { return target };
    let size = Vec2::new(width, height);
    let clamp_axis = |value: f32, size: f32, half: f32| {
        if size > half * 2.0 { value.clamp(half, size - half) } else { size * 0.5 }
    };
    Vec2::new(clamp_axis(target.x, size.x, half_view.x), clamp_axis(target.y, size.y, half_view.y))
}

/// A new run starts with the camera already on the hero's start position
/// and without leftover shake.
pub(crate) fn snap_to_start(
    terrain: Res<Terrain>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut focus: ResMut<CameraFocus>,
    mut shake: ResMut<ScreenShake>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    let (x, y) = terrain.tiles().spawn_point();
    focus.0 = clamp_to_map(Vec2::new(x, y), terrain.tiles(), half_view(window.single().ok()));
    shake.trauma = 0.0;
    camera.translation = focus.0.extend(camera.translation.z);
}

/// Eases the focus towards the hero.
fn follow_hero(
    time: Res<Time>,
    terrain: Res<Terrain>,
    window: Query<&Window, With<PrimaryWindow>>,
    hero: Single<&Transform, With<Player>>,
    mut focus: ResMut<CameraFocus>,
) {
    let target = clamp_to_map(hero.translation.truncate(), terrain.tiles(), half_view(window.single().ok()));
    let blend = 1.0 - (-FOLLOW_RATE * time.delta_secs()).exp();
    focus.0 = focus.0.lerp(target, blend);
}

/// Places the camera at the focus plus this frame's shake, and lets the
/// trauma fade. The offset is trauma squared, so small hits barely register
/// while big ones kick.
pub(crate) fn apply_shake(
    time: Res<Time>,
    focus: Res<CameraFocus>,
    mut shake: ResMut<ScreenShake>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    let mut offset = Vec2::ZERO;
    if shake.trauma > 0.0 {
        // Two incommensurate sine pairs: smooth, never repeating visibly.
        let t = time.elapsed_secs();
        let noise = Vec2::new(
            (t * 47.0).sin() * 0.6 + (t * 83.0 + 1.7).sin() * 0.4,
            (t * 59.0 + 4.1).sin() * 0.6 + (t * 71.0 + 2.3).sin() * 0.4,
        );
        offset = noise * shake.trauma * shake.trauma * SHAKE_STRENGTH;
        shake.trauma = (shake.trauma - SHAKE_DECAY * time.delta_secs()).max(0.0);
    }
    let position = (focus.0 + offset).extend(camera.translation.z);
    if camera.translation != position {
        camera.translation = position;
    }
}

pub(crate) fn register(app: &mut App) {
    app.init_resource::<CameraFocus>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::InRun), snap_to_start.in_set(RunSetup::Spawn))
        .add_systems(
            Update,
            (follow_hero, apply_shake).chain().after(GameSet::Movement).run_if(in_state(GameState::InRun)),
        );
}
