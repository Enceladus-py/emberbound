//! Where and how lesser monsters (and dragons) enter play: on a ring just
//! outside the view, wherever the hero is.

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

/// Transform of a freshly spawned actor. `world` sorts its z by y (from a
/// depth origin near the hero) before it is first drawn.
pub(crate) fn actor_transform(position: Vec2) -> Transform {
    Transform::from_translation(position.extend(layer::ACTORS))
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
        let tiles = self.terrain.tiles();
        if walks {
            return tiles.is_walkable_at(position.x, position.y, radius);
        }
        let Some((width, height)) = tiles.bounds() else { return true };
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
            let spot = hero + self.rng.direction() * RING_MIN;
            let Some((width, height)) = self.terrain.tiles().bounds() else { return spot };
            let margin = Vec2::splat(MAP_MARGIN + radius);
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

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::state::app::StatesPlugin;
    use emberbound_core::CorePlugin;
    use emberbound_domain::bestiary::DragonKind;
    use emberbound_domain::world::{self, WorldMap};

    use super::*;

    const TRIES: usize = 300;

    fn app(seed: u64) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            // The sprite library uploads into these; no renderer is needed.
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins(CorePlugin)
            .insert_resource(Terrain::Valley(world::generate(72, 72, &mut GameRng::new(seed))))
            .insert_resource(GameRng::new(seed + 1))
            .add_systems(Update, recall_strays);
        app
    }

    fn map(app: &App) -> WorldMap {
        let Terrain::Valley(map) = app.world().resource::<Terrain>() else { panic!("tests use a valley") };
        map.clone()
    }

    fn centre(app: &App) -> Vec2 {
        Vec2::from(map(app).spawn_point())
    }

    /// Runs `pick` with a fresh [`Spawner`] `TRIES` times and returns every answer.
    fn sample<T: Send + 'static>(app: &mut App, pick: fn(&mut Spawner) -> T) -> Vec<T> {
        app.world_mut()
            .run_system_once(move |mut spawner: Spawner| {
                (0..TRIES).map(|_| pick(&mut spawner)).collect::<Vec<T>>()
            })
            .unwrap()
    }

    fn in_ring(spot: Vec2, hero: Vec2) -> bool {
        // A little slack for float rounding.
        (RING_MIN - 0.01..=RING_MAX + 0.01).contains(&spot.distance(hero))
    }

    fn inside_margin(map: &WorldMap, spot: Vec2, radius: f32) -> bool {
        let (width, height) = map.world_size();
        let margin = MAP_MARGIN + radius;
        (margin..=width - margin).contains(&spot.x) && (margin..=height - margin).contains(&spot.y)
    }

    #[test]
    fn ring_spots_lie_on_the_ring_and_fit() {
        for seed in [1, 2, 3] {
            let mut app = app(seed);
            let hero = centre(&app);
            for walks in [true, false] {
                let spots = app
                    .world_mut()
                    .run_system_once(move |mut spawner: Spawner| {
                        let mut spots = Vec::new();
                        for _ in 0..TRIES {
                            if let Some(spot) = spawner.ring_spot(hero, 6.0, walks) {
                                spots.push((spot, spawner.fits(spot, 6.0, walks)));
                            }
                        }
                        spots
                    })
                    .unwrap();
                assert!(spots.len() > TRIES / 2, "seed {seed}: most tries find a spot");
                for (spot, fits) in spots {
                    assert!(in_ring(spot, hero), "seed {seed}: {spot} is off the ring around {hero}");
                    assert!(fits, "seed {seed}: {spot} does not fit");
                }
            }
        }
    }

    #[test]
    fn walkers_are_never_placed_on_blocked_ground() {
        for seed in [4, 5, 6] {
            let mut app = app(seed);
            let map = map(&app);
            let spots = sample(&mut app, |spawner| {
                let hero = Vec2::from(spawner.terrain.tiles().spawn_point());
                spawner.ring_spot(hero, 7.0, true)
            });
            let found: Vec<Vec2> = spots.into_iter().flatten().collect();
            assert!(!found.is_empty(), "seed {seed}: no walker spot at all");
            for spot in found {
                assert!(map.is_walkable_at(spot.x, spot.y, 7.0), "seed {seed}: walker at {spot}");
                let (tx, ty) = WorldMap::tile_coords(spot.x, spot.y);
                assert!(map.tile(tx, ty).is_walkable());
            }
        }
    }

    #[test]
    fn flyers_fit_anywhere_inside_the_margin_but_not_beyond() {
        let mut app = app(7);
        let map = map(&app);
        let (width, height) = map.world_size();
        let verdicts = app
            .world_mut()
            .run_system_once(move |spawner: Spawner| {
                let margin = MAP_MARGIN + 4.0;
                [
                    spawner.fits(Vec2::new(margin + 1.0, margin + 1.0), 4.0, false),
                    spawner.fits(Vec2::new(width - margin - 1.0, height / 2.0), 4.0, false),
                    spawner.fits(Vec2::new(margin - 1.0, height / 2.0), 4.0, false),
                    spawner.fits(Vec2::new(width / 2.0, height - margin + 1.0), 4.0, false),
                    // The forest border blocks walkers, though a flyer may hover near it.
                    spawner.fits(Vec2::new(1.0, 1.0), 4.0, true),
                ]
            })
            .unwrap();
        assert_eq!(verdicts, [true, true, false, false, false]);
    }

    #[test]
    fn a_spot_is_given_up_when_nothing_fits() {
        let mut app = app(8);
        // Far outside the valley: no flyer spot on that ring can fit.
        let misses = sample(&mut app, |spawner| spawner.ring_spot(Vec2::new(-5_000.0, -5_000.0), 4.0, false));
        assert!(misses.iter().all(Option::is_none));
    }

    #[test]
    fn the_nearest_fallback_stays_inside_the_valley_from_any_corner() {
        let mut app = app(9);
        let map = map(&app);
        let (width, height) = map.world_size();
        // The last hero stands so far out that no ring spot fits: only the fallback answers.
        let corners = [
            Vec2::ZERO,
            Vec2::new(width, 0.0),
            Vec2::new(0.0, height),
            Vec2::new(width, height),
            Vec2::splat(-2_000.0),
        ];
        for corner in corners {
            let spots = app
                .world_mut()
                .run_system_once(move |mut spawner: Spawner| {
                    (0..TRIES).map(|_| spawner.ring_spot_or_nearest(corner, 10.0)).collect::<Vec<_>>()
                })
                .unwrap();
            for spot in spots {
                assert!(inside_margin(&map, spot, 10.0), "{spot} from corner {corner}");
            }
        }
    }

    #[test]
    fn strays_beyond_the_leash_return_to_the_ring_and_near_ones_stay() {
        let mut app = app(10);
        let hero = centre(&app);
        app.world_mut().spawn((Player, Transform::from_translation(hero.extend(0.0)), Health::full(100.0)));
        let far = hero + Vec2::new(LEASH + 40.0, 0.0);
        let near = hero + Vec2::new(0.0, LEASH - 10.0);
        let flyer = Body { radius: 5.0, walks: false };
        let stray = app
            .world_mut()
            .spawn((Monster { kind: MonsterKind::Bat, xp_reward: 1 }, actor_transform(far), flyer))
            .id();
        let lost_dragon = app
            .world_mut()
            .spawn((Dragon { kind: DragonKind::FrostDrake, xp_reward: 1 }, actor_transform(far), flyer))
            .id();
        let nearby = app
            .world_mut()
            .spawn((Monster { kind: MonsterKind::Bat, xp_reward: 1 }, actor_transform(near), flyer))
            .id();
        app.update();

        let position = |entity: Entity| app.world().get::<Transform>(entity).unwrap().translation.truncate();
        for recalled in [stray, lost_dragon] {
            assert!(in_ring(position(recalled), hero), "{} was not recalled", position(recalled));
        }
        assert_eq!(position(nearby), near, "a creature within the leash is left alone");
    }

    #[test]
    fn on_the_endless_world_spots_ring_the_hero_wherever_they_roam() {
        let mut app = app(11);
        let hero = Vec2::new(23_456.0, -17_890.0);
        let mut terrain = Terrain::endless(77);
        terrain.load_around(hero);
        app.insert_resource(terrain);
        let spots = app
            .world_mut()
            .run_system_once(move |mut spawner: Spawner| {
                (0..TRIES)
                    .map(|i| {
                        let walks = i % 2 == 0;
                        (spawner.ring_spot(hero, 6.0, walks), walks, spawner.ring_spot_or_nearest(hero, 10.0))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();
        let found = spots.iter().filter(|(spot, _, _)| spot.is_some()).count();
        assert!(found > TRIES / 2, "only {found} spots found");
        let terrain = app.world().resource::<Terrain>();
        for (spot, walks, dragon) in spots {
            if let Some(spot) = spot {
                assert!(in_ring(spot, hero), "{spot} is off the ring");
                assert!(!walks || terrain.tiles().is_walkable_at(spot.x, spot.y, 6.0), "walker at {spot}");
            }
            // No map edge to pull a dragon back to: it always arrives on the ring.
            assert!(in_ring(dragon, hero), "dragon at {dragon}");
        }
    }
}
