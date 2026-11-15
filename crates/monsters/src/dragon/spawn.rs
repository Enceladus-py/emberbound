//! Building an elder dragon and announcing its arrival.

use bevy::prelude::*;
use emberbound_core::{
    Announce, Armor, Body, ContactDamage, Dragon, Facing, GameState, Health, Knockback, ScreenShake,
    SpriteId, Team, Velocity, YSort,
};
use emberbound_domain::bestiary::DragonKind;

use super::DragonBrain;
use crate::spawning::{Spawner, actor_transform, shadow};

const WING_FPS: f32 = 4.0;
/// Width of the shadow relative to the body diameter.
const SHADOW_WIDTH_FACTOR: f32 = 1.3;
/// Width of the shadow sprite in pixels.
const SHADOW_SPRITE_WIDTH: f32 = 12.0;
const ARRIVAL_SHAKE: f32 = 0.6;

fn sprite_of(kind: DragonKind) -> SpriteId {
    match kind {
        DragonKind::MossWyrm => SpriteId::DragonMoss,
        DragonKind::FrostDrake => SpriteId::DragonFrost,
        DragonKind::AshenTyrant => SpriteId::DragonAshen,
    }
}

/// Brings a dragon in from just off-screen with a banner and a rumble.
pub(crate) fn spawn_dragon(
    spawner: &mut Spawner,
    kind: DragonKind,
    hero: Vec2,
    announce: &mut MessageWriter<Announce>,
    shake: &mut ScreenShake,
) {
    let spec = kind.spec();
    let id = sprite_of(kind);
    let position = spawner.ring_spot_or_nearest(hero, spec.radius);
    let shadow_scale = spec.radius * 2.0 * SHADOW_WIDTH_FACTOR / SHADOW_SPRITE_WIDTH;
    let shadow_drop = spawner.sprites.size(id).y * 0.5;
    let brain = DragonBrain::new(spec, &mut spawner.rng);
    spawner.commands.spawn((
        (
            Dragon { kind, xp_reward: spec.xp_reward },
            Health::full(spec.max_health),
            Armor(spec.armor),
            Team::Horde,
            ContactDamage(spec.contact_damage),
            // Dragons fly over terrain.
            Body { radius: spec.radius, walks: false },
        ),
        (Velocity::default(), Knockback::default(), Facing::default(), YSort),
        spawner.sprites.animated(id, WING_FPS),
        actor_transform(position),
        brain,
        DespawnOnExit(GameState::InRun),
        children![shadow(&spawner.sprites, shadow_drop, shadow_scale)],
    ));
    announce
        .write(Announce { title: spec.name.to_uppercase(), subtitle: format!("{} descends", spec.epithet) });
    shake.add(ARRIVAL_SHAKE);
}
