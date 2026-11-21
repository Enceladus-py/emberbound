//! Where and how lesser monsters enter the valley.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use emberbound_core::{
    Armor, Body, ContactDamage, Dragon, Facing, GameRng, GameState, Health, Knockback, Monster, Player,
    SpriteId, SpriteLibrary, Team, Terrain, Velocity, YSort, layer,
};
use emberbound_domain::bestiary::{MonsterKind, MonsterSpec};
use emberbound_domain::ports::RandomSource;
use emberbound_domain::world::TILE_SIZE;

use crate::AiStep;
use crate::ai::attach_brain;
use crate::hero::{HeroPos, hero_alive};

/// Monsters appear on a ring just outside the ~384x216 view.
const RING_MIN: f32 = 235.0;
const RING_MAX: f32 = 280.0;
/// Random spots tried before a spawn is given up.
const SPOT_ATTEMPTS: usize = 8;
/// Beyond this distance from the hero a creature is moved back to the ring.
const LEASH: f32 = 420.0;
/// The outermost tiles are solid forest; nothing spawns in there.
const MAP_MARGIN: f32 = TILE_SIZE * 2.0;
/// How far above their body position flyers are drawn.
const FLYER_HOVER: f32 = 5.0;
/// Shadow z relative to its owner: under every y-sorted actor, above pickups.
const SHADOW_Z: f32 = -4.0;
const WALK_FPS: f32 = 5.0;
const FLAP_FPS: f32 = 10.0;

/// A soft ground shadow to attach as a child, `drop` units below its owner.
pub(crate) fn shadow(sprites: &SpriteLibrary, drop: f32, scale: f32) -> impl Bundle {
    (
        sprites.sprite(SpriteId::Shadow),
        Transform::from_xyz(0.0, -drop, SHADOW_Z).with_scale(Vec3::new(scale, scale, 1.0)),
    )
}

/// Transform of a freshly spawned actor (`world` keeps the z sorted later).
pub(crate) fn actor_transform(position: Vec2) -> Transform {
    Transform::from_translation(position.extend(layer::ACTORS - position.y * 0.001))
}

fn sprite_of(kind: MonsterKind) -> SpriteId {
    match kind {
        MonsterKind::Slime => SpriteId::Slime,
        MonsterKind::Bat => SpriteId::Bat,
        MonsterKind::Goblin => SpriteId::Goblin,
        MonsterKind::Wyrmling => SpriteId::Wyrmling,
    }
}

/// Everything needed to place and build creatures.
#[derive(SystemParam)]
pub(crate) struct Spawner<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub sprites: Res<'w, SpriteLibrary>,
    pub terrain: Res<'w, Terrain>,
    pub rng: ResMut<'w, GameRng>,
}

impl Spawner<'_, '_> {
    /// Whether a creature fits at `position`: inside the valley, and on
    /// open ground if it walks.
    fn fits(&self, position: Vec2, radius: f32, walks: bool) -> bool {
        if walks {
            return self.terrain.0.is_walkable_at(position.x, position.y, radius);
        }
        let (width, height) = self.terrain.0.world_size();
        let margin = MAP_MARGIN + radius;
        (margin..width - margin).contains(&position.x) && (margin..height - margin).contains(&position.y)
    }

    /// A random fitting spot between `min` and `max` units from `centre`.
    pub fn spot_around(
        &mut self,
        centre: Vec2,
        (min, max): (f32, f32),
        radius: f32,
        walks: bool,
    ) -> Option<Vec2> {
        (0..SPOT_ATTEMPTS).find_map(|_| {
            let spot = centre + self.rng.direction() * self.rng.range(min, max);
            self.fits(spot, radius, walks).then_some(spot)
        })
    }

    /// A fitting spot on the off-screen ring around the hero.
    pub fn ring_spot(&mut self, hero: Vec2, radius: f32, walks: bool) -> Option<Vec2> {
        self.spot_around(hero, (RING_MIN, RING_MAX), radius, walks)
    }

    /// Like [`Self::ring_spot`] for a flyer that must appear no matter what:
    /// falls back to the nearest point inside the valley.
    pub fn ring_spot_or_nearest(&mut self, hero: Vec2, radius: f32) -> Vec2 {
        self.ring_spot(hero, radius, false).unwrap_or_else(|| {
            let (width, height) = self.terrain.0.world_size();
            let margin = Vec2::splat(MAP_MARGIN + radius);
            let spot = hero + self.rng.direction() * RING_MIN;
            spot.clamp(margin, Vec2::new(width, height) - margin)
        })
    }

    /// Builds one lesser monster at `position`.
    pub fn spawn_monster(&mut self, spec: &MonsterSpec, position: Vec2) {
        let id = sprite_of(spec.kind);
        let size = self.sprites.size(id);
        let (hover, fps) = if spec.flies { (FLYER_HOVER, FLAP_FPS) } else { (0.0, WALK_FPS) };
        // Flyers: the sprite is lifted off the body and the shadow sinks.
        let shadow_drop = size.y * 0.5 - 1.0 + hover * 0.5;
        let mut entity = self.commands.spawn((
            (
                Monster { kind: spec.kind, xp_reward: spec.xp_reward },
                Health::full(spec.max_health),
                Armor(spec.armor),
                Team::Horde,
                ContactDamage(spec.contact_damage),
                Body { radius: spec.radius, walks: !spec.flies },
            ),
            (Velocity::default(), Knockback::default(), Facing::default(), YSort),
            self.sprites.animated(id, fps),
            Anchor(Vec2::new(0.0, -hover / size.y)),
            actor_transform(position),
            DespawnOnExit(GameState::InRun),
            children![shadow(&self.sprites, shadow_drop, 1.0)],
        ));
        attach_brain(&mut entity, spec, &mut self.rng);
    }

    /// Spawns a monster just off-screen; skipped when no spot fits.
    pub fn spawn_monster_on_ring(&mut self, spec: &MonsterSpec, hero: Vec2) {
        if let Some(spot) = self.ring_spot(hero, spec.radius, !spec.flies) {
            self.spawn_monster(spec, spot);
        }
    }
}

/// Creatures the hero has left far behind quietly re-enter from the ring
/// instead of being lost somewhere in the valley.
fn recall_strays(
    hero: HeroPos,
    mut strays: Query<(&mut Transform, &Body), (Or<(With<Monster>, With<Dragon>)>, Without<Player>)>,
    mut spawner: Spawner,
) {
    let hero_pos = hero.translation.truncate();
    for (mut transform, body) in &mut strays {
        if transform.translation.truncate().distance_squared(hero_pos) <= LEASH * LEASH {
            continue;
        }
        if let Some(spot) = spawner.ring_spot(hero_pos, body.radius, body.walks) {
            transform.translation.x = spot.x;
            transform.translation.y = spot.y;
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, recall_strays.run_if(hero_alive).in_set(AiStep::Direct));
}
