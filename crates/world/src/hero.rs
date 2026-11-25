//! The hero: spawning, steering from [`PlayerIntent`], and the dash.

use bevy::prelude::*;
use emberbound_core::{
    Aim, Armor, Body, DashCooldown, Facing, GameSet, GameState, Health, HeroState, Invulnerable, Knockback,
    Player, PlayerIntent, RunSetup, SpriteAnim, SpriteId, SpriteLibrary, Team, Terrain, Velocity, YSort,
};

/// Collision / hurt radius of the hero.
const BODY_RADIUS: f32 = 5.0;
/// Frames per second of the idle and walk cycles.
const ANIMATION_FPS: f32 = 8.0;
/// The shadow sits just under the feet of the 16x16 sprite...
const SHADOW_OFFSET_Y: f32 = -7.0;
/// ...and far enough below the hero's y-sorted depth to be under every
/// actor, yet above the ground and pickups.
const SHADOW_OFFSET_Z: f32 = -5.0;

/// Dash speed as a multiple of the hero's move speed.
const DASH_SPEED_FACTOR: f32 = 3.4;
/// Seconds a dash lasts.
const DASH_SECONDS: f32 = 0.18;
/// Seconds the hero cannot be hurt after starting a dash.
const DASH_INVULNERABLE_SECONDS: f32 = 0.3;

/// The dash in progress, if any. Lives on the [`Player`].
#[derive(Component, Debug, Default)]
struct Dash {
    /// Unit direction of the current dash.
    direction: Vec2,
    /// Seconds of dash left (`0.0` = not dashing).
    remaining: f32,
}

fn spawn_hero(
    mut commands: Commands,
    terrain: Res<Terrain>,
    hero: Res<HeroState>,
    library: Res<SpriteLibrary>,
) {
    let (x, y) = terrain.0.spawn_point();
    let stats = &hero.stats;
    commands.spawn((
        Player,
        library.animated(SpriteId::HeroIdle, ANIMATION_FPS),
        // `YSort` settles the real depth before the first frame is drawn.
        Transform::from_xyz(x, y, 0.0),
        (Health::full(stats.max_health), Armor(stats.armor), Team::Hero),
        (Velocity::default(), Knockback::default(), Body { radius: BODY_RADIUS, walks: true }),
        (Facing::default(), Aim::default(), YSort),
        (DashCooldown { remaining: 0.0, total: stats.dash_cooldown }, Dash::default()),
        DespawnOnExit(GameState::InRun),
        children![(
            library.sprite(SpriteId::Shadow),
            Transform::from_xyz(0.0, SHADOW_OFFSET_Y, SHADOW_OFFSET_Z),
        )],
    ));
}

/// Recharges the dash and starts a new one when asked and ready.
fn update_dash(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    hero: Res<HeroState>,
    mut commands: Commands,
    mut player: Single<
        (Entity, &Health, &Facing, &mut DashCooldown, &mut Dash, Option<&Invulnerable>),
        With<Player>,
    >,
) {
    let (entity, health, facing, cooldown, dash, invulnerable) = &mut *player;
    let dt = time.delta_secs();

    // Boons can shorten the cooldown mid-run, so re-read it every frame.
    cooldown.total = hero.stats.dash_cooldown;
    cooldown.remaining = (cooldown.remaining - dt).clamp(0.0, cooldown.total);
    dash.remaining = (dash.remaining - dt).max(0.0);

    if health.is_dead() {
        dash.remaining = 0.0;
        return;
    }
    if !intent.dash || cooldown.remaining > 0.0 {
        return;
    }
    dash.direction = intent.movement.try_normalize().unwrap_or(facing.0);
    dash.remaining = DASH_SECONDS;
    cooldown.remaining = cooldown.total;
    // Never cut short a longer grace period the hero already enjoys.
    if invulnerable.is_none_or(|i| i.0.remaining_secs() < DASH_INVULNERABLE_SECONDS) {
        commands.entity(*entity).insert(Invulnerable::seconds(DASH_INVULNERABLE_SECONDS));
    }
}

/// Turns intent into velocity and facing. A dash overrides steering; a dead
/// hero stands still.
fn steer_hero(
    intent: Res<PlayerIntent>,
    hero: Res<HeroState>,
    mut player: Single<(&Health, &Dash, &mut Velocity, &mut Facing), With<Player>>,
) {
    let (health, dash, velocity, facing) = &mut *player;
    let speed = hero.stats.speed(health.fraction());
    velocity.0 = if health.is_dead() {
        Vec2::ZERO
    } else if dash.remaining > 0.0 {
        dash.direction * speed * DASH_SPEED_FACTOR
    } else {
        intent.movement.clamp_length_max(1.0) * speed
    };
    // Standing still keeps the last direction, so attacks still have an aim.
    if let Some(direction) = velocity.0.try_normalize() {
        facing.0 = direction;
    }
}

/// Mirrors the sprite to look where the hero faces (or swings) and picks the idle or
/// walk cycle.
fn animate_hero(
    library: Res<SpriteLibrary>,
    mut player: Single<(&Velocity, &Facing, Option<&Aim>, &mut Sprite, &mut SpriteAnim), With<Player>>,
) {
    let (velocity, facing, aim, sprite, anim) = &mut *player;
    // Mid-swing the hero looks where the sword goes, so it never seems to
    // strike out of their back.
    let look = match aim {
        Some(aim) if aim.swinging > 0.0 => aim.direction,
        _ => facing.0,
    };
    // Straight up or down keeps the previous side.
    if look.x != 0.0 {
        let flip = look.x < 0.0;
        if sprite.flip_x != flip {
            sprite.flip_x = flip;
        }
    }
    let cycle = if velocity.0 == Vec2::ZERO { SpriteId::HeroIdle } else { SpriteId::HeroWalk };
    library.switch(sprite, anim, cycle);
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(OnEnter(GameState::InRun), spawn_hero.in_set(RunSetup::Spawn))
        .add_systems(Update, (update_dash, steer_hero, animate_hero).chain().in_set(GameSet::Input));
}
