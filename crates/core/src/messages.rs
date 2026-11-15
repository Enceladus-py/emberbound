use bevy::prelude::*;
use emberbound_domain::bestiary::{DragonKind, MonsterKind};
use emberbound_domain::combat::AttackProfile;
use emberbound_domain::progression::Boon;
use emberbound_pixelart::SpriteId;

use crate::Team;

/// What kind of actor something is, for messages that outlive the entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorKind {
    Hero,
    Monster(MonsterKind),
    Dragon(DragonKind),
}

/// Ask `combat` to hit `target`. Anyone may write this.
/// Ignored if the target is dead, [`crate::Invulnerable`] or despawned.
#[derive(Message, Debug, Clone, Copy)]
pub struct DamageRequest {
    pub target: Entity,
    pub attack: AttackProfile,
    /// Where the blow came from; knockback pushes away from here.
    pub origin: Vec2,
}

/// Written by `combat` after a hit actually landed.
#[derive(Message, Debug, Clone, Copy)]
pub struct DamageApplied {
    pub target: Entity,
    pub victim: ActorKind,
    pub amount: f32,
    pub critical: bool,
    pub position: Vec2,
    pub fatal: bool,
}

/// Written by `combat` when something dies. Monsters and dragons are
/// despawned by `combat` in the same frame; the hero entity stays.
#[derive(Message, Debug, Clone, Copy)]
pub struct ActorDied {
    pub entity: Entity,
    pub kind: ActorKind,
    pub position: Vec2,
}

/// Ask `combat` to launch a projectile. Anyone may write this.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnProjectile {
    /// The side that fired it; it only hurts the other side.
    pub team: Team,
    pub sprite: SpriteId,
    pub position: Vec2,
    /// World units per second. The sprite is rotated to face along it.
    pub velocity: Vec2,
    pub attack: AttackProfile,
    pub radius: f32,
    /// Seconds before it fizzles out.
    pub lifetime: f32,
    /// How many extra targets it passes through (0 = dies on first hit).
    pub pierce: u32,
}

/// Written by `combat` when the hero picks up embers.
#[derive(Message, Debug, Clone, Copy)]
pub struct XpCollected {
    pub amount: u32,
}

/// Written by the app each time the hero gains a level. The app also fills
/// [`crate::BoonOffer`] and switches to [`crate::RunPhase::ChoosingBoon`].
#[derive(Message, Debug, Clone, Copy)]
pub struct LeveledUp {
    pub level: u32,
}

/// Written by `ui` when the player picks from [`crate::BoonOffer`]. The app
/// applies it and resumes [`crate::RunPhase::Fighting`].
#[derive(Message, Debug, Clone, Copy)]
pub struct BoonChosen(pub Boon);

/// Ask `ui` to show a big banner (dragon arrivals, growth of the companion...).
#[derive(Message, Debug, Clone)]
pub struct Announce {
    pub title: String,
    pub subtitle: String,
}

/// One-shot visual effects, rendered by `ui`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FxKind {
    /// Small spark burst where a hit landed.
    HitSpark,
    /// Smoke puff where something died.
    DeathPuff,
    /// Burst of embers (dragon death, level up).
    EmberBurst,
}

/// Ask `ui` to play an effect at a world position. Anyone may write this.
#[derive(Message, Debug, Clone, Copy)]
pub struct FxRequest {
    pub kind: FxKind,
    pub position: Vec2,
}

pub(crate) fn register(app: &mut App) {
    app.add_message::<DamageRequest>()
        .add_message::<DamageApplied>()
        .add_message::<ActorDied>()
        .add_message::<SpawnProjectile>()
        .add_message::<XpCollected>()
        .add_message::<LeveledUp>()
        .add_message::<BoonChosen>()
        .add_message::<Announce>()
        .add_message::<FxRequest>();
}
