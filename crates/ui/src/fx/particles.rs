//! One-shot effects requested through [`FxRequest`], and the level-up ring.

use std::f32::consts::TAU;

use bevy::prelude::*;
use emberbound_core::{
    FxKind, FxRequest, GameRng, GameState, LeveledUp, Player, SpriteId, SpriteLibrary, layer,
};
use emberbound_domain::ports::RandomSource;

use crate::theme;

const SPARK_FPS: f32 = 24.0;
const PUFF_FPS: f32 = 14.0;
const EMBER_LIFETIME: f32 = 0.7;
/// Downward pull on burst embers, world units per second squared.
const EMBER_GRAVITY: f32 = 90.0;
const RING_EMBERS: usize = 18;
const RING_SPEED: f32 = 74.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (play_requests, ring_on_level_up, move_embers).chain().run_if(in_state(GameState::InRun)),
    );
}

/// A flying ember. Moves itself (no `Velocity`): it is decoration and must
/// keep drifting while gameplay is frozen for a boon choice.
#[derive(Component)]
struct Ember {
    velocity: Vec2,
    gravity: f32,
    age: f32,
    size: f32,
    tint: Color,
}

fn ember(
    library: &SpriteLibrary,
    position: Vec2,
    velocity: Vec2,
    gravity: f32,
    size: f32,
    tint: Color,
) -> impl Bundle {
    (
        DespawnOnExit(GameState::InRun),
        Ember { velocity, gravity, age: 0.0, size, tint },
        Sprite { color: tint, ..library.sprite(SpriteId::EmberOrb) },
        Transform::from_translation(position.extend(layer::FX)).with_scale(Vec3::splat(size)),
    )
}

fn play_requests(
    mut commands: Commands,
    mut requests: MessageReader<FxRequest>,
    library: Res<SpriteLibrary>,
    mut rng: ResMut<GameRng>,
) {
    for request in requests.read() {
        let at = Transform::from_translation(request.position.extend(layer::FX));
        match request.kind {
            FxKind::HitSpark => {
                commands.spawn((
                    DespawnOnExit(GameState::InRun),
                    library.one_shot(SpriteId::Spark, SPARK_FPS),
                    at,
                ));
            }
            FxKind::DeathPuff => {
                commands.spawn((
                    DespawnOnExit(GameState::InRun),
                    library.one_shot(SpriteId::Puff, PUFF_FPS),
                    at,
                ));
            }
            FxKind::EmberBurst => {
                for _ in 0..rng.range(10.0, 15.0) as usize {
                    let velocity = rng.direction() * rng.range(35.0, 95.0) + Vec2::Y * 25.0;
                    let size = rng.range(0.55, 1.0);
                    commands.spawn(ember(
                        &library,
                        request.position,
                        velocity,
                        EMBER_GRAVITY,
                        size,
                        Color::WHITE,
                    ));
                }
            }
        }
    }
}

/// A level-up throws an even ring of golden embers out from the hero.
fn ring_on_level_up(
    mut commands: Commands,
    mut levels: MessageReader<LeveledUp>,
    library: Res<SpriteLibrary>,
    hero: Single<&Transform, With<Player>>,
) {
    let centre = hero.translation.truncate();
    for _ in levels.read() {
        for i in 0..RING_EMBERS {
            let direction = Vec2::from_angle(TAU * i as f32 / RING_EMBERS as f32);
            commands.spawn(ember(
                &library,
                centre + direction * 6.0,
                direction * RING_SPEED,
                0.0,
                0.9,
                theme::XP,
            ));
        }
    }
}

fn move_embers(
    time: Res<Time>,
    mut commands: Commands,
    mut embers: Query<(Entity, &mut Ember, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut ember, mut transform, mut sprite) in &mut embers {
        ember.age += dt;
        let t = ember.age / EMBER_LIFETIME;
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        ember.velocity.y -= ember.gravity * dt;
        // Air drag: the burst is fast at first and then lingers.
        ember.velocity *= (-2.2 * dt).exp();
        transform.translation += (ember.velocity * dt).extend(0.0);
        transform.scale = Vec3::splat(ember.size * (1.0 - 0.7 * t));
        sprite.color = ember.tint.with_alpha(1.0 - t * t);
    }
}
