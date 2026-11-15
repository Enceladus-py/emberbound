//! Floating damage numbers.

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use emberbound_core::{ActorKind, DamageApplied, GameRng, GameState, layer};
use emberbound_domain::ports::RandomSource;

use crate::theme;

const LIFETIME: f32 = 0.6;
/// World units a number rises over its life.
const RISE: f32 = 14.0;
/// Rasterised large and scaled down: world units are art pixels, so text
/// drawn at its natural size would be huge, and a tiny font would be blurry.
const FONT_SIZE: f32 = 48.0;
const SCALE: f32 = 0.2;
const CRITICAL_SCALE: f32 = 0.3;
/// Horizontal scatter so stacked hits stay legible.
const JITTER: f32 = 5.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_numbers, float_numbers).chain().run_if(in_state(GameState::InRun)));
}

#[derive(Component)]
struct DamageNumber {
    age: f32,
    origin: Vec2,
    scale: f32,
}

fn spawn_numbers(mut commands: Commands, mut hits: MessageReader<DamageApplied>, mut rng: ResMut<GameRng>) {
    for hit in hits.read() {
        let amount = hit.amount.round().max(1.0);
        let (text, color, scale) = match (hit.victim, hit.critical) {
            (ActorKind::Hero, _) => (format!("{amount:.0}"), theme::HEALTH, SCALE),
            (_, true) => (format!("{amount:.0}!"), theme::XP, CRITICAL_SCALE),
            (_, false) => (format!("{amount:.0}"), Color::WHITE, SCALE),
        };
        let origin = hit.position + Vec2::new(rng.range(-JITTER, JITTER), 6.0);
        commands.spawn((
            DespawnOnExit(GameState::InRun),
            DamageNumber { age: 0.0, origin, scale },
            Text2d::new(text),
            TextFont { font_size: FontSize::Px(FONT_SIZE), ..default() },
            TextColor(color),
            Text2dShadow { offset: Vec2::new(4.0, -4.0), color: Color::BLACK.with_alpha(0.85) },
            Transform::from_translation(origin.extend(layer::FX + 1.0)).with_scale(Vec3::splat(scale)),
        ));
    }
}

fn float_numbers(
    time: Res<Time>,
    mut commands: Commands,
    mut numbers: Query<(Entity, &mut DamageNumber, &mut Transform, &mut TextColor, &mut Text2dShadow)>,
) {
    for (entity, mut number, mut transform, mut color, mut shadow) in &mut numbers {
        number.age += time.delta_secs();
        let t = number.age / LIFETIME;
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Rise quickly, then hang; pop in slightly oversized.
        let rise = RISE * (1.0 - (1.0 - t).powi(2));
        let pop = 1.0 + 0.35 * (1.0 - t * 5.0).max(0.0);
        let alpha = (1.0 - (t - 0.5).max(0.0) * 2.0).clamp(0.0, 1.0);
        transform.translation = (number.origin + Vec2::Y * rise).extend(transform.translation.z);
        transform.scale = Vec3::splat(number.scale * pop);
        color.0.set_alpha(alpha);
        shadow.color.set_alpha(0.85 * alpha);
    }
}
