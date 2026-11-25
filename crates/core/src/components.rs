use bevy::prelude::*;
use emberbound_domain::bestiary::{DragonKind, MonsterKind};
use emberbound_domain::combat;

/// The hero. Exactly one exists during a run. Spawned by `world` with at
/// least: `Transform`, `Sprite`, [`Health`], [`Armor`], [`Team::Hero`],
/// [`Velocity`], [`Knockback`], [`Body`], [`Facing`], [`YSort`].
#[derive(Component, Debug, Default)]
pub struct Player;

/// A lesser monster. Spawned by `monsters`.
#[derive(Component, Debug)]
pub struct Monster {
    pub kind: MonsterKind,
    pub xp_reward: u32,
}

/// An elder dragon (boss). Spawned by `monsters`.
#[derive(Component, Debug)]
pub struct Dragon {
    pub kind: DragonKind,
    pub xp_reward: u32,
}

/// The hero's bonded dragon. Spawned and driven by `combat`.
#[derive(Component, Debug, Default)]
pub struct Companion;

/// The main 2D camera. Spawned once at start-up by `world`.
#[derive(Component, Debug, Default)]
pub struct MainCamera;

/// Hit points. Only `combat` may reduce them (via [`crate::DamageRequest`]).
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct Health(pub combat::Health);

impl Health {
    pub fn full(max: f32) -> Self {
        Self(combat::Health::full(max))
    }
}

/// Flat damage reduction.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Armor(pub f32);

/// Which side something fights for. Attacks only hurt the other side.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Hero,
    Horde,
}

/// Intended velocity in world units per second. Integrated by `world`.
#[derive(Component, Debug, Clone, Copy, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// Extra velocity from being hit; added to [`Velocity`] and decayed by `world`.
#[derive(Component, Debug, Clone, Copy, Default, Deref, DerefMut)]
pub struct Knockback(pub Vec2);

/// Physical presence used for hits and terrain collision.
#[derive(Component, Debug, Clone, Copy)]
pub struct Body {
    /// Collision / hurt radius in world units.
    pub radius: f32,
    /// `true` for things on foot (blocked by water, trees, rocks...).
    /// `false` for anything that flies over terrain.
    pub walks: bool,
}

/// Unit vector the entity looks along. Sprites face right; `world` flips the
/// hero, `monsters` flips its own sprites.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct Facing(pub Vec2);

impl Default for Facing {
    fn default() -> Self {
        Self(Vec2::X)
    }
}

/// Where the hero's next sword swing goes, settled every frame by `combat`
/// from the player's explicit aim, aim assist or [`Facing`]. Read by `world`
/// (the hero looks along it while swinging) and `ui` (the aim pointer).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    /// Unit vector.
    pub direction: Vec2,
    /// The enemy aim assist locked onto, if that is what chose the direction.
    pub target: Option<Entity>,
    /// The sword can swing right now.
    pub ready: bool,
    /// Seconds the hero keeps looking along the aim after a swing.
    pub swinging: f32,
}

impl Default for Aim {
    fn default() -> Self {
        Self { direction: Vec2::X, target: None, ready: true, swinging: 0.0 }
    }
}

/// Hurts the opposing team on touch (applied by `combat`).
#[derive(Component, Debug, Clone, Copy)]
pub struct ContactDamage(pub f32);

/// While present, damage requests against the entity are ignored.
/// `combat` ticks the timer and removes the component when it finishes.
#[derive(Component, Debug, Clone)]
pub struct Invulnerable(pub Timer);

impl Invulnerable {
    pub fn seconds(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

/// Marker: keep `Transform.translation.z` sorted by y so lower things draw
/// in front. `world` sets `z = layer::ACTORS - y * 0.001` every frame.
#[derive(Component, Debug, Default)]
pub struct YSort;

/// The hero's dash. Lives on the [`Player`]; owned and updated by `world`,
/// read by `ui` to draw the cooldown.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DashCooldown {
    /// Seconds until the dash is ready again (`0.0` = ready).
    pub remaining: f32,
    /// Full cooldown length in seconds.
    pub total: f32,
}
