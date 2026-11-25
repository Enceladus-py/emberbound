//! The aim pointer: a small arrowhead around the hero showing where the next
//! sword swing goes, bright when the sword is ready and dim while it
//! recovers. When aim assist picked the direction, the arrowhead glows ember
//! and brackets frame the enemy it locked onto. Shown only while fighting.

use bevy::prelude::*;
use emberbound_core::{
    Aim, Body, GameState, Health, HeroState, Player, RunPhase, RunSetup, SpriteId, SpriteLibrary, layer,
};

use crate::theme;

/// How far out the arrowhead sits, as a share of the sword's reach, and
/// never closer than this many world units (it must clear the hero).
const POINTER_REACH: f32 = 0.75;
const POINTER_MIN_DISTANCE: f32 = 13.0;
/// Pointer opacity when the sword is ready, and while it recovers.
const READY_ALPHA: f32 = 0.95;
const RECOVERING_ALPHA: f32 = 0.35;
/// Breathing of the lock-on brackets.
const MARK_PULSE_RATE: f32 = 7.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InRun), spawn_pointer.in_set(RunSetup::Spawn))
        .add_systems(Update, follow_aim.run_if(in_state(GameState::InRun)));
}

#[derive(Component)]
struct Pointer;

#[derive(Component)]
struct LockMark;

fn spawn_pointer(mut commands: Commands, library: Res<SpriteLibrary>) {
    let hidden = |id| {
        (
            library.sprite(id),
            Transform::from_xyz(0.0, 0.0, layer::FX),
            Visibility::Hidden,
            DespawnOnExit(GameState::InRun),
        )
    };
    commands.spawn((Pointer, hidden(SpriteId::AimPointer)));
    commands.spawn((LockMark, hidden(SpriteId::AimMark)));
}

fn follow_aim(
    time: Res<Time>,
    phase: Option<Res<State<RunPhase>>>,
    hero_state: Res<HeroState>,
    hero: Query<(&Transform, &Aim, &Health), With<Player>>,
    targets: Query<(&Transform, &Body), Without<Player>>,
    mut pointer: Single<
        (&mut Transform, &mut Sprite, &mut Visibility),
        (With<Pointer>, Without<LockMark>, Without<Player>, Without<Body>),
    >,
    mut mark: Single<
        (&mut Transform, &mut Visibility),
        (With<LockMark>, Without<Pointer>, Without<Player>, Without<Body>),
    >,
) {
    let fighting = phase.is_some_and(|phase| *phase.get() == RunPhase::Fighting);
    let hero = hero.single().ok().filter(|(.., health)| fighting && !health.is_dead());
    let Some((hero, aim, _)) = hero else {
        pointer.2.set_if_neq(Visibility::Hidden);
        mark.1.set_if_neq(Visibility::Hidden);
        return;
    };

    let (transform, sprite, visibility) = &mut *pointer;
    let distance = (hero_state.stats.attack_reach * POINTER_REACH).max(POINTER_MIN_DISTANCE);
    let origin = hero.translation.truncate();
    transform.translation = (origin + aim.direction * distance).extend(layer::FX);
    transform.rotation = Quat::from_rotation_z(aim.direction.to_angle());
    let tint = if aim.target.is_some() { theme::EMBER } else { Color::WHITE };
    let alpha = if aim.ready { READY_ALPHA } else { RECOVERING_ALPHA };
    sprite.color = tint.with_alpha(alpha);
    visibility.set_if_neq(Visibility::Inherited);

    let (mark_transform, mark_visibility) = &mut *mark;
    match aim.target.and_then(|target| targets.get(target).ok()) {
        Some((target, body)) => {
            let pulse = 1.0 + 0.08 * (time.elapsed_secs() * MARK_PULSE_RATE).sin();
            // The brackets hug the body with a pixel or two to spare.
            let size = (body.radius * 2.0 + 4.0) / 9.0 * pulse;
            mark_transform.translation = target.translation.truncate().extend(layer::FX);
            mark_transform.scale = Vec3::new(size, size, 1.0);
            mark_visibility.set_if_neq(Visibility::Inherited);
        }
        None => {
            mark_visibility.set_if_neq(Visibility::Hidden);
        }
    }
}
