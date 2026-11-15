//! Hit flash and invulnerability blink. This module is the only writer of
//! actors' `Sprite.color`.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use emberbound_core::{ActorKind, DamageApplied, GameState, Invulnerable, Player};

const FLASH_SECONDS: f32 = 0.09;
const MONSTER_TINT: Color = Color::srgb(1.0, 0.42, 0.22);
const HERO_TINT: Color = Color::srgb(1.0, 0.2, 0.22);
/// Blinks per second while the hero is invulnerable.
const BLINK_RATE: f32 = 12.0;
const BLINK_ALPHA: f32 = 0.35;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Flashes>().add_systems(OnExit(GameState::InRun), forget_flashes).add_systems(
        Update,
        (start_flashes, end_flashes, blink_hero).chain().run_if(in_state(GameState::InRun)),
    );
}

/// Seconds of tint left per flashing actor. Kept here rather than as a
/// component so nothing is ever inserted on an entity that may be gone.
#[derive(Resource, Default)]
struct Flashes(HashMap<Entity, f32>);

fn forget_flashes(mut flashes: ResMut<Flashes>) {
    flashes.0.clear();
}

fn start_flashes(
    mut hits: MessageReader<DamageApplied>,
    mut flashes: ResMut<Flashes>,
    mut sprites: Query<&mut Sprite>,
) {
    for hit in hits.read() {
        // Slain monsters are already despawned: nothing to flash.
        let Ok(mut sprite) = sprites.get_mut(hit.target) else { continue };
        sprite.color = if hit.victim == ActorKind::Hero { HERO_TINT } else { MONSTER_TINT };
        flashes.0.insert(hit.target, FLASH_SECONDS);
    }
}

fn end_flashes(time: Res<Time>, mut flashes: ResMut<Flashes>, mut sprites: Query<&mut Sprite>) {
    if flashes.0.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    flashes.0.retain(|&entity, left| {
        *left -= dt;
        if *left > 0.0 {
            return sprites.contains(entity);
        }
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            sprite.color = Color::WHITE;
        }
        false
    });
}

/// Runs after the flash systems so the blink keeps whatever tint is active.
fn blink_hero(time: Res<Time>, mut hero: Single<(&mut Sprite, Has<Invulnerable>), With<Player>>) {
    let blink_off = hero.1 && (time.elapsed_secs() * BLINK_RATE).fract() < 0.5;
    let alpha = if blink_off { BLINK_ALPHA } else { 1.0 };
    if hero.0.color.alpha() != alpha {
        hero.0.color.set_alpha(alpha);
    }
}
