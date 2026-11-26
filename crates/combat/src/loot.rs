//! Loot dropped by the slain, and the pickups the hero gathers.

use bevy::prelude::*;
use emberbound_core::{
    ActorDied, Dragon, GameRng, GameState, Health, HeroState, Monster, Player, SpriteId, SpriteLibrary,
    XpCollected, layer,
};
use emberbound_domain::ports::RandomSource;

const PICKUP_FPS: f32 = 8.0;
const MONSTER_ORBS_MAX: u32 = 3;
const DRAGON_ORBS: (u32, u32) = (10, 14);
const DRAGON_HEARTS: usize = 2;
/// Pop-out speed range of ordinary drops and of a dragon's shower.
const SCATTER_SPEED: (f32, f32) = (25.0, 70.0);
const SHOWER_SPEED: (f32, f32) = (60.0, 190.0);
/// How quickly the pop-out motion dies down (per second).
const SETTLE_RATE: f32 = 5.0;
/// Top speed of a magnetised pickup; well above the hero's so it always arrives.
const MAGNET_SPEED: f32 = 210.0;
/// How quickly a magnetised pickup turns towards the hero (per second).
const MAGNET_RATE: f32 = 7.0;
const COLLECT_RADIUS: f32 = 7.0;
const BOB_HEIGHT: f32 = 1.5;
/// Most ember orbs allowed on the ground at once; beyond it the farthest are
/// folded into one so a long run cannot pile up loot (and frame time).
const MAX_EMBER_ORBS: usize = 250;
/// A folded orb is drawn a little larger, so it reads as a richer prize.
const FOLDED_SCALE: f32 = 1.5;
const BOB_SPEED: f32 = 4.0;

#[derive(Clone, Copy)]
enum Reward {
    Embers(u32),
    Heart(f32),
}

/// Something lying on the ground for the hero to gather. It moves itself
/// (no `Velocity`/`Body`): terrain never blocks loot.
#[derive(Component)]
pub struct Pickup {
    reward: Reward,
    /// Resting position; the sprite bobs around it.
    position: Vec2,
    velocity: Vec2,
    /// Offset into the bob cycle so a pile of loot does not move in unison.
    phase: f32,
}

/// Splits `total` into at most `parts` positive shares that add up to it.
fn shares(total: u32, parts: u32) -> impl Iterator<Item = u32> {
    let parts = parts.clamp(1, total.max(1));
    (0..parts).map(move |i| total / parts + u32::from(i < total % parts)).filter(|share| *share > 0)
}

/// Spawns one pickup that pops out of `origin` in a random direction.
fn spawn_pickup(
    commands: &mut Commands,
    library: &SpriteLibrary,
    rng: &mut GameRng,
    reward: Reward,
    origin: Vec2,
    speed: (f32, f32),
) {
    let sprite = match reward {
        Reward::Embers(_) => SpriteId::EmberOrb,
        Reward::Heart(_) => SpriteId::Heart,
    };
    commands.spawn((
        library.animated(sprite, PICKUP_FPS),
        Transform::from_translation(origin.extend(layer::PICKUP)),
        Pickup {
            reward,
            position: origin,
            velocity: rng.direction() * rng.range(speed.0, speed.1),
            phase: rng.range(0.0, std::f32::consts::TAU),
        },
        DespawnOnExit(GameState::InRun),
    ));
}

/// Turns this frame's deaths into pickups. Runs before the dead are buried,
/// so their reward components are still there to read.
pub fn drop_loot(
    mut deaths: MessageReader<ActorDied>,
    monsters: Query<&Monster>,
    dragons: Query<&Dragon>,
    library: Res<SpriteLibrary>,
    hero_state: Res<HeroState>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    let (heart_chance, heart_heal) = (hero_state.stats.heart_chance, hero_state.stats.heart_heal);
    for death in deaths.read() {
        let at = death.position;
        if let Ok(monster) = monsters.get(death.entity) {
            let orbs = 1 + rng.index(MONSTER_ORBS_MAX as usize) as u32;
            for share in shares(monster.xp_reward, orbs) {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Embers(share), at, SCATTER_SPEED);
            }
            if rng.chance(heart_chance) {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Heart(heart_heal), at, SCATTER_SPEED);
            }
        } else if let Ok(dragon) = dragons.get(death.entity) {
            let orbs = DRAGON_ORBS.0 + rng.index((DRAGON_ORBS.1 - DRAGON_ORBS.0 + 1) as usize) as u32;
            for share in shares(dragon.xp_reward, orbs) {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Embers(share), at, SHOWER_SPEED);
            }
            for _ in 0..DRAGON_HEARTS {
                spawn_pickup(&mut commands, &library, &mut rng, Reward::Heart(heart_heal), at, SHOWER_SPEED);
            }
        }
    }
}

/// Moves pickups: the pop-out settles, the living hero's magnet pulls, and the
/// sprite bobs gently around the resting position.
pub fn drift(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    hero: Single<(&Transform, &Health), (With<Player>, Without<Pickup>)>,
    mut pickups: Query<(&mut Transform, &mut Pickup)>,
) {
    let dt = time.delta_secs();
    let (hero_transform, hero_health) = *hero;
    let hero_position = hero_transform.translation.truncate();
    let magnet_radius = if hero_health.is_dead() { 0.0 } else { hero_state.stats.pickup_radius };

    for (mut transform, mut pickup) in &mut pickups {
        let to_hero = hero_position - pickup.position;
        if to_hero.length() < magnet_radius {
            let pull = to_hero.normalize_or_zero() * MAGNET_SPEED;
            pickup.velocity = pickup.velocity.lerp(pull, 1.0 - (-MAGNET_RATE * dt).exp());
        } else {
            pickup.velocity *= (-SETTLE_RATE * dt).exp();
        }
        let step = pickup.velocity * dt;
        pickup.position += step;
        let bob = (time.elapsed_secs() * BOB_SPEED + pickup.phase).sin() * BOB_HEIGHT;
        transform.translation = (pickup.position + Vec2::Y * bob).extend(layer::PICKUP);
    }
}

/// Ember orbs to fold together: `absorb` are removed and their embers go to
/// `keep`, which then carries `amount`.
#[derive(Debug, PartialEq)]
struct Fold {
    keep: usize,
    absorb: Vec<usize>,
    amount: u32,
}

/// Chooses how to bring `orbs` (position and embers each) down to `cap` by
/// folding the ones farthest from the hero into one. Orbs within the
/// magnet's reach are never touched, so it may fold fewer than needed, and
/// does nothing when fewer than two orbs could be folded.
fn plan_fold(orbs: &[(Vec2, u32)], hero: Vec2, magnet_radius: f32, cap: usize) -> Option<Fold> {
    let excess = orbs.len().checked_sub(cap).filter(|excess| *excess > 0)?;
    let mut far: Vec<(usize, f32)> = orbs
        .iter()
        .enumerate()
        .map(|(index, (position, _))| (index, position.distance(hero)))
        .filter(|(_, distance)| *distance >= magnet_radius)
        .collect();
    // Farthest first; folding n + 1 orbs into one removes n.
    far.sort_by(|a, b| b.1.total_cmp(&a.1));
    far.truncate(excess + 1);
    if far.len() < 2 {
        return None;
    }
    // The nearest of the chosen survives, so the prize stays as close as it was.
    let (keep, _) = far.pop()?;
    let absorb: Vec<usize> = far.into_iter().map(|(index, _)| index).collect();
    let amount = orbs[keep].1 + absorb.iter().map(|index| orbs[*index].1).sum::<u32>();
    Some(Fold { keep, absorb, amount })
}

/// Keeps the number of ember orbs bounded without losing any embers.
pub fn fold_far_orbs(
    hero_state: Res<HeroState>,
    hero: Single<(&Transform, &Health), (With<Player>, Without<Pickup>)>,
    mut pickups: Query<(Entity, &mut Pickup, &mut Transform)>,
    mut commands: Commands,
) {
    let orbs: Vec<(Entity, Vec2, u32)> = pickups
        .iter()
        .filter_map(|(entity, pickup, _)| match pickup.reward {
            Reward::Embers(amount) => Some((entity, pickup.position, amount)),
            Reward::Heart(_) => None,
        })
        .collect();
    if orbs.len() <= MAX_EMBER_ORBS {
        return;
    }
    let (hero_transform, hero_health) = *hero;
    let magnet_radius = if hero_health.is_dead() { 0.0 } else { hero_state.stats.pickup_radius };
    let layout: Vec<(Vec2, u32)> = orbs.iter().map(|(_, position, amount)| (*position, *amount)).collect();
    let Some(fold) = plan_fold(&layout, hero_transform.translation.truncate(), magnet_radius, MAX_EMBER_ORBS)
    else {
        return;
    };
    for index in fold.absorb {
        commands.entity(orbs[index].0).try_despawn();
    }
    if let Ok((_, mut pickup, mut transform)) = pickups.get_mut(orbs[fold.keep].0) {
        pickup.reward = Reward::Embers(fold.amount);
        transform.scale = Vec3::new(FOLDED_SCALE, FOLDED_SCALE, 1.0);
    }
}

/// Hands pickups that reached the living hero over to them.
pub fn collect(
    mut hero: Single<(&Transform, &mut Health), With<Player>>,
    hero_state: Res<HeroState>,
    pickups: Query<(Entity, &Pickup)>,
    mut commands: Commands,
    mut collected: MessageWriter<XpCollected>,
) {
    let (hero_transform, hero_health) = &mut *hero;
    if hero_health.is_dead() {
        return;
    }
    let hero_position = hero_transform.translation.truncate();
    for (entity, pickup) in &pickups {
        if pickup.position.distance(hero_position) > COLLECT_RADIUS {
            continue;
        }
        match pickup.reward {
            Reward::Embers(amount) => {
                collected.write(XpCollected { amount: hero_state.stats.xp_for(amount) });
            }
            Reward::Heart(amount) => hero_health.heal(amount),
        }
        commands.entity(entity).try_despawn();
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use emberbound_core::{ActorKind, CorePlugin};
    use emberbound_domain::bestiary::{DragonKind, MonsterKind};

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            // The sprite library uploads into these; no renderer is needed.
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins(CorePlugin)
            .insert_resource(GameRng::new(7))
            .add_systems(Update, (drop_loot, collect));
        // Hearts from ordinary monsters are left to chance; keep them out.
        app.world_mut().resource_mut::<HeroState>().stats.heart_chance = 0.0;
        app
    }

    fn spawn_hero(app: &mut App) -> Entity {
        app.world_mut().spawn((Player, Transform::default(), Health::full(100.0))).id()
    }

    fn spawn_pickup_at(app: &mut App, reward: Reward, position: Vec2) -> Entity {
        let pickup = Pickup { reward, position, velocity: Vec2::ZERO, phase: 0.0 };
        app.world_mut().spawn((pickup, Transform::from_translation(position.extend(layer::PICKUP)))).id()
    }

    fn kill(app: &mut App, entity: Entity, kind: ActorKind) {
        app.world_mut().write_message(ActorDied { entity, kind, position: Vec2::new(500.0, 500.0) });
    }

    fn rewards(app: &mut App) -> Vec<Reward> {
        let world = app.world_mut();
        world.query::<&Pickup>().iter(world).map(|pickup| pickup.reward).collect()
    }

    fn embers(rewards: &[Reward]) -> Vec<u32> {
        rewards.iter().filter_map(|r| if let Reward::Embers(n) = r { Some(*n) } else { None }).collect()
    }

    fn hearts(rewards: &[Reward]) -> usize {
        rewards.iter().filter(|r| matches!(r, Reward::Heart(_))).count()
    }

    fn drain<M: Message>(app: &mut App) -> Vec<M> {
        app.world_mut().resource_mut::<Messages<M>>().drain().collect()
    }

    fn total_embers(app: &mut App) -> u32 {
        embers(&rewards(app)).iter().sum()
    }

    #[test]
    fn nothing_is_folded_at_or_under_the_cap() {
        let orbs = vec![(Vec2::new(500.0, 0.0), 1); 5];
        assert_eq!(plan_fold(&orbs, Vec2::ZERO, 10.0, 5), None);
        assert_eq!(plan_fold(&orbs, Vec2::ZERO, 10.0, 9), None);
        assert_eq!(plan_fold(&[], Vec2::ZERO, 10.0, 0), None);
    }

    #[test]
    fn the_farthest_orbs_fold_into_the_nearest_of_them() {
        // Index = distance rank: orb i lies 100 * (i + 1) units away.
        let orbs: Vec<(Vec2, u32)> =
            (0..6).map(|i| (Vec2::new(100.0 * (i + 1) as f32, 0.0), i + 1)).collect();
        let fold = plan_fold(&orbs, Vec2::ZERO, 10.0, 4).unwrap();
        // Two too many: the three farthest become one.
        assert_eq!(fold.keep, 3);
        let mut absorb = fold.absorb.clone();
        absorb.sort();
        assert_eq!(absorb, vec![4, 5]);
        assert_eq!(fold.amount, 4 + 5 + 6);
        assert_eq!(orbs.len() - fold.absorb.len(), 4);
    }

    #[test]
    fn folding_keeps_every_ember() {
        for (count, cap) in [(10, 3), (40, 39), (300, 250), (7, 1)] {
            let orbs: Vec<(Vec2, u32)> = (0..count)
                .map(|i| (Vec2::from_angle(i as f32) * (50.0 + i as f32 * 3.0), 1 + i % 7))
                .collect();
            let before: u32 = orbs.iter().map(|(_, amount)| amount).sum();
            let fold = plan_fold(&orbs, Vec2::ZERO, 20.0, cap as usize).unwrap();
            let after: u32 = orbs
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != fold.keep && !fold.absorb.contains(index))
                .map(|(_, (_, amount))| amount)
                .sum::<u32>()
                + fold.amount;
            assert_eq!(after, before, "{count} orbs, cap {cap}");
            assert_eq!(count as usize - fold.absorb.len(), cap as usize, "{count} orbs, cap {cap}");
        }
    }

    #[test]
    fn orbs_within_the_magnet_are_never_folded() {
        let near = (Vec2::new(5.0, 0.0), 9);
        let mut orbs = vec![near; 6];
        orbs.push((Vec2::new(400.0, 0.0), 1));
        orbs.push((Vec2::new(300.0, 0.0), 1));
        orbs.push((Vec2::new(200.0, 0.0), 1));
        // Six too many, but only three lie outside the magnet.
        let fold = plan_fold(&orbs, Vec2::ZERO, 34.0, 3).unwrap();
        let touched: Vec<usize> = fold.absorb.iter().copied().chain([fold.keep]).collect();
        assert!(touched.iter().all(|index| *index >= 6), "{touched:?}");
        assert_eq!(fold.amount, 3);

        // With a single far orb there is nothing to fold it into.
        assert_eq!(plan_fold(&orbs[..7], Vec2::ZERO, 34.0, 3), None);
    }

    #[test]
    fn the_ground_never_holds_more_than_the_cap_of_ember_orbs() {
        let mut app = app();
        app.add_systems(Update, fold_far_orbs.after(collect));
        spawn_hero(&mut app);
        let radius = app.world().resource::<HeroState>().stats.pickup_radius;
        let near = spawn_pickup_at(&mut app, Reward::Embers(2), Vec2::new(radius - 5.0, 0.0));
        let heart = spawn_pickup_at(&mut app, Reward::Heart(10.0), Vec2::new(900.0, 900.0));
        for i in 0..MAX_EMBER_ORBS + 30 {
            spawn_pickup_at(&mut app, Reward::Embers(3), Vec2::from_angle(i as f32) * (100.0 + i as f32));
        }
        let before = total_embers(&mut app);
        app.update();

        let after = rewards(&mut app);
        assert_eq!(embers(&after).len(), MAX_EMBER_ORBS);
        assert_eq!(total_embers(&mut app), before, "no embers are lost");
        assert_eq!(hearts(&after), 1);
        assert!(app.world().get_entity(heart).is_ok(), "hearts are never folded");
        let near_pickup = app.world().get::<Pickup>(near).expect("an orb in reach is never folded");
        assert!(matches!(near_pickup.reward, Reward::Embers(2)));

        let world = app.world_mut();
        let folded: Vec<u32> = world
            .query::<(&Pickup, &Transform)>()
            .iter(world)
            .filter(|(_, transform)| transform.scale.x == FOLDED_SCALE)
            .filter_map(|(pickup, _)| if let Reward::Embers(n) = pickup.reward { Some(n) } else { None })
            .collect();
        // 31 too many (the near orb counts too): the 32 farthest become one larger orb.
        assert_eq!(folded, vec![3 * 32]);
    }

    #[test]
    fn shares_add_up_and_respect_the_part_count() {
        for total in 0..=40 {
            for parts in 0..=15 {
                let split: Vec<u32> = shares(total, parts).collect();
                assert_eq!(split.iter().sum::<u32>(), total, "total {total}, parts {parts}");
                assert!(split.iter().all(|share| *share > 0), "total {total}, parts {parts}");
                assert!(split.len() as u32 <= parts.max(1), "total {total}, parts {parts}");
            }
        }
    }

    #[test]
    fn nothing_to_share_yields_nothing() {
        assert_eq!(shares(0, 0).count(), 0);
        assert_eq!(shares(0, 3).count(), 0);
        assert_eq!(shares(2, 5).collect::<Vec<_>>(), vec![1, 1]);
    }

    #[test]
    fn a_slain_monster_drops_embers_worth_its_reward() {
        let mut app = app();
        spawn_hero(&mut app);
        for xp_reward in [1, 2, 7, 30] {
            let monster = app.world_mut().spawn(Monster { kind: MonsterKind::Slime, xp_reward }).id();
            kill(&mut app, monster, ActorKind::Monster(MonsterKind::Slime));
            app.update();

            let dropped = rewards(&mut app);
            let orbs = embers(&dropped);
            assert!((1..=MONSTER_ORBS_MAX as usize).contains(&orbs.len()), "{} orbs", orbs.len());
            assert_eq!(orbs.iter().sum::<u32>(), xp_reward);
            assert_eq!(hearts(&dropped), 0);

            let world = app.world_mut();
            let pickups: Vec<Entity> = world.query_filtered::<Entity, With<Pickup>>().iter(world).collect();
            for pickup in pickups {
                world.despawn(pickup);
            }
        }
    }

    #[test]
    fn a_slain_dragon_always_drops_its_hearts() {
        let mut app = app();
        spawn_hero(&mut app);
        let dragon = app.world_mut().spawn(Dragon { kind: DragonKind::MossWyrm, xp_reward: 120 }).id();
        kill(&mut app, dragon, ActorKind::Dragon(DragonKind::MossWyrm));
        app.update();

        let dropped = rewards(&mut app);
        assert_eq!(hearts(&dropped), DRAGON_HEARTS);
        let orbs = embers(&dropped);
        assert!(
            (DRAGON_ORBS.0 as usize..=DRAGON_ORBS.1 as usize).contains(&orbs.len()),
            "{} orbs",
            orbs.len()
        );
        assert_eq!(orbs.iter().sum::<u32>(), 120);
    }

    #[test]
    fn the_living_hero_collects_embers_within_reach() {
        let mut app = app();
        spawn_hero(&mut app);
        app.world_mut().resource_mut::<HeroState>().stats.xp_multiplier = 1.5;
        let near = spawn_pickup_at(&mut app, Reward::Embers(4), Vec2::new(COLLECT_RADIUS - 1.0, 0.0));
        let far = spawn_pickup_at(&mut app, Reward::Embers(4), Vec2::new(COLLECT_RADIUS + 1.0, 0.0));
        app.update();

        assert!(app.world().get_entity(near).is_err(), "a reached pickup is gathered");
        assert!(app.world().get_entity(far).is_ok(), "a distant pickup stays put");
        let collected: Vec<u32> = drain::<XpCollected>(&mut app).iter().map(|m| m.amount).collect();
        assert_eq!(collected, vec![6], "xp goes through the hero's multiplier");
    }

    #[test]
    fn hearts_heal_but_never_beyond_max() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        app.world_mut().get_mut::<Health>(hero).unwrap().damage(50.0);
        spawn_pickup_at(&mut app, Reward::Heart(20.0), Vec2::ZERO);
        app.update();
        assert_eq!(app.world().get::<Health>(hero).unwrap().current(), 70.0);

        spawn_pickup_at(&mut app, Reward::Heart(20.0), Vec2::ZERO);
        spawn_pickup_at(&mut app, Reward::Heart(20.0), Vec2::ZERO);
        app.update();
        assert_eq!(app.world().get::<Health>(hero).unwrap().current(), 100.0);
        assert_eq!(rewards(&mut app).len(), 0);
        assert!(drain::<XpCollected>(&mut app).is_empty(), "hearts grant no xp");
    }

    #[test]
    fn a_dead_hero_collects_nothing() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        app.world_mut().get_mut::<Health>(hero).unwrap().damage(1_000.0);
        let orb = spawn_pickup_at(&mut app, Reward::Embers(3), Vec2::ZERO);
        let heart = spawn_pickup_at(&mut app, Reward::Heart(20.0), Vec2::ZERO);
        app.update();

        assert!(app.world().get_entity(orb).is_ok());
        assert!(app.world().get_entity(heart).is_ok());
        assert!(drain::<XpCollected>(&mut app).is_empty());
        assert!(app.world().get::<Health>(hero).unwrap().is_dead());
    }
}
