//! Projectiles of both sides: launched on request, flown by `world`'s
//! movement system, and tested here against the opposing team.

use bevy::prelude::*;
use emberbound_core::{
    Body, DamageRequest, FxKind, FxRequest, GameState, Health, Invulnerable, SpawnProjectile, SpriteLibrary,
    Team, Velocity, layer,
};
use emberbound_domain::combat::AttackProfile;

const PROJECTILE_FPS: f32 = 10.0;
/// Knockback originates this far behind the projectile, so victims are pushed
/// along its flight path rather than away from an arbitrary overlap point.
const IMPACT_SETBACK: f32 = 6.0;

/// A shot in flight.
#[derive(Component)]
pub struct Projectile {
    team: Team,
    attack: AttackProfile,
    radius: f32,
    lifetime: Timer,
    /// Extra targets it may still pass through.
    pierce: u32,
    /// Targets it already hurt; a piercing shot hits each only once.
    already_hit: Vec<Entity>,
    /// It hit its last target and waits for clean-up.
    spent: bool,
}

pub fn launch(
    mut requests: MessageReader<SpawnProjectile>,
    library: Res<SpriteLibrary>,
    mut commands: Commands,
) {
    for request in requests.read() {
        let heading = request.velocity.normalize_or(Vec2::X);
        commands.spawn((
            library.animated(request.sprite, PROJECTILE_FPS),
            Transform::from_translation(request.position.extend(layer::PROJECTILE))
                .with_rotation(Quat::from_rotation_z(heading.to_angle())),
            Velocity(request.velocity),
            Projectile {
                team: request.team,
                attack: request.attack,
                radius: request.radius,
                lifetime: Timer::from_seconds(request.lifetime, TimerMode::Once),
                pierce: request.pierce,
                already_hit: Vec::new(),
                spent: false,
            },
            DespawnOnExit(GameState::InRun),
        ));
    }
}

/// Ages every projectile and lets it hit at most one new target per frame.
pub fn strike(
    time: Res<Time>,
    mut projectiles: Query<(&Transform, &Velocity, &mut Projectile)>,
    targets: Query<(Entity, &Transform, &Team, &Health, &Body, Has<Invulnerable>)>,
    mut damage: MessageWriter<DamageRequest>,
    mut fx: MessageWriter<FxRequest>,
) {
    for (transform, velocity, mut projectile) in &mut projectiles {
        projectile.lifetime.tick(time.delta());
        if projectile.spent || projectile.lifetime.is_finished() {
            continue;
        }
        let position = transform.translation.truncate();
        let victim = targets.iter().find(|(entity, target, team, health, body, invulnerable)| {
            **team != projectile.team
                && !health.is_dead()
                && !invulnerable
                && !projectile.already_hit.contains(entity)
                && target.translation.truncate().distance(position) < projectile.radius + body.radius
        });
        let Some((target, ..)) = victim else { continue };

        damage.write(DamageRequest {
            target,
            attack: projectile.attack,
            origin: position - velocity.normalize_or_zero() * IMPACT_SETBACK,
        });
        fx.write(FxRequest { kind: FxKind::HitSpark, position });
        if projectile.pierce > 0 {
            projectile.pierce -= 1;
            projectile.already_hit.push(target);
        } else {
            projectile.spent = true;
        }
    }
}

/// Removes projectiles that hit their last target or ran out of time.
pub fn fizzle(projectiles: Query<(Entity, &Projectile)>, mut commands: Commands) {
    for (entity, projectile) in &projectiles {
        if projectile.spent || projectile.lifetime.is_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use emberbound_core::{CorePlugin, Monster, Player, SpriteId};
    use emberbound_domain::bestiary::MonsterKind;

    use super::*;

    const FRAME: Duration = Duration::from_millis(50);
    const SHOT: AttackProfile = AttackProfile::flat(5.0, 0.0);
    const BODY: Body = Body { radius: 6.0, walks: true };

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            // The sprite library uploads into these; no renderer is needed.
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins(CorePlugin)
            .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME))
            .add_systems(Update, (launch, strike, fizzle).chain());
        app
    }

    fn fire(app: &mut App, team: Team, position: Vec2, pierce: u32, lifetime: f32) {
        app.world_mut().write_message(SpawnProjectile {
            team,
            sprite: SpriteId::Firebolt,
            position,
            // Nothing moves projectiles in these tests; the heading only sets knockback.
            velocity: Vec2::X * 100.0,
            attack: SHOT,
            radius: 4.0,
            lifetime,
            pierce,
        });
    }

    fn spawn_monster(app: &mut App, position: Vec2) -> Entity {
        let monster = Monster { kind: MonsterKind::Slime, xp_reward: 1 };
        let transform = Transform::from_translation(position.extend(0.0));
        app.world_mut().spawn((monster, transform, Team::Horde, Health::full(20.0), BODY)).id()
    }

    fn spawn_hero(app: &mut App, position: Vec2) -> Entity {
        let transform = Transform::from_translation(position.extend(0.0));
        app.world_mut().spawn((Player, transform, Team::Hero, Health::full(100.0), BODY)).id()
    }

    fn hits(app: &mut App) -> Vec<Entity> {
        let mut messages = app.world_mut().resource_mut::<Messages<DamageRequest>>();
        messages.drain().map(|request| request.target).collect()
    }

    fn projectiles(app: &mut App) -> usize {
        let world = app.world_mut();
        world.query::<&Projectile>().iter(world).count()
    }

    #[test]
    fn a_shot_hurts_only_the_opposing_team() {
        let mut app = app();
        let hero = spawn_hero(&mut app, Vec2::ZERO);
        let monster = spawn_monster(&mut app, Vec2::new(100.0, 0.0));

        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 1.0);
        app.update();
        assert!(hits(&mut app).is_empty(), "a hero shot never targets the hero");
        assert_eq!(projectiles(&mut app), 1, "it keeps flying");

        fire(&mut app, Team::Horde, Vec2::new(100.0, 0.0), 0, 1.0);
        app.update();
        assert!(hits(&mut app).is_empty(), "a horde shot never targets a monster");

        fire(&mut app, Team::Horde, Vec2::ZERO, 0, 1.0);
        fire(&mut app, Team::Hero, Vec2::new(100.0, 0.0), 0, 1.0);
        app.update();
        let mut landed = hits(&mut app);
        landed.sort();
        let mut expected = vec![hero, monster];
        expected.sort();
        assert_eq!(landed, expected);
    }

    #[test]
    fn dead_and_invulnerable_targets_are_skipped() {
        let mut app = app();
        let dead = spawn_monster(&mut app, Vec2::ZERO);
        app.world_mut().get_mut::<Health>(dead).unwrap().damage(1_000.0);
        let shielded = spawn_monster(&mut app, Vec2::ZERO);
        app.world_mut().entity_mut(shielded).insert(Invulnerable::seconds(10.0));

        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 1.0);
        app.update();
        assert!(hits(&mut app).is_empty());
        assert_eq!(projectiles(&mut app), 1, "a shot that hit nothing flies on");
    }

    #[test]
    fn a_shot_misses_targets_out_of_reach() {
        let mut app = app();
        // Just beyond projectile radius + body radius.
        spawn_monster(&mut app, Vec2::new(4.0 + BODY.radius + 0.5, 0.0));
        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 1.0);
        app.update();
        assert!(hits(&mut app).is_empty());
    }

    #[test]
    fn a_plain_shot_overlapping_two_monsters_hits_one_and_is_spent() {
        let mut app = app();
        spawn_monster(&mut app, Vec2::ZERO);
        spawn_monster(&mut app, Vec2::new(1.0, 0.0));

        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 1.0);
        app.update();
        assert_eq!(hits(&mut app).len(), 1);
        assert_eq!(projectiles(&mut app), 0, "a spent shot is cleaned up");

        app.update();
        assert!(hits(&mut app).is_empty());
    }

    #[test]
    fn a_piercing_shot_hits_one_new_target_per_frame_never_twice() {
        let mut app = app();
        let first = spawn_monster(&mut app, Vec2::ZERO);
        let second = spawn_monster(&mut app, Vec2::new(1.0, 0.0));
        let third = spawn_monster(&mut app, Vec2::new(-1.0, 0.0));

        fire(&mut app, Team::Hero, Vec2::ZERO, 1, 1.0);
        app.update();
        let frame_one = hits(&mut app);
        assert_eq!(frame_one.len(), 1, "at most one new target per frame");
        assert_eq!(projectiles(&mut app), 1, "it still has a target to pass through");

        app.update();
        let frame_two = hits(&mut app);
        assert_eq!(frame_two.len(), 1);
        assert_ne!(frame_one[0], frame_two[0], "never the same target twice");
        assert!([first, second, third].contains(&frame_two[0]));
        assert_eq!(projectiles(&mut app), 0, "pierce 1 means two targets, then it is gone");

        app.update();
        assert!(hits(&mut app).is_empty());
    }

    #[test]
    fn a_piercing_shot_does_not_hit_its_lone_target_again() {
        let mut app = app();
        spawn_monster(&mut app, Vec2::ZERO);
        fire(&mut app, Team::Hero, Vec2::ZERO, 2, 1.0);
        let mut landed = 0;
        for _ in 0..4 {
            app.update();
            landed += hits(&mut app).len();
        }
        assert_eq!(landed, 1);
        assert_eq!(projectiles(&mut app), 1, "it still flies with pierce to spare");
    }

    #[test]
    fn an_expired_shot_hits_nothing_and_fizzles() {
        let mut app = app();
        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 0.12);
        // Age it past its lifetime while nothing is in reach.
        for _ in 0..4 {
            app.update();
        }
        assert_eq!(projectiles(&mut app), 0, "an expired shot is cleaned up");

        // A shot that runs out on the frame a target comes into reach hits nothing.
        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 0.12);
        app.update();
        app.update();
        assert_eq!(projectiles(&mut app), 1, "still in flight before its lifetime is up");
        spawn_monster(&mut app, Vec2::ZERO);
        app.update();
        assert!(hits(&mut app).is_empty());
        assert_eq!(projectiles(&mut app), 0);
    }

    #[test]
    fn knockback_originates_behind_the_shot() {
        let mut app = app();
        spawn_monster(&mut app, Vec2::new(2.0, 0.0));
        fire(&mut app, Team::Hero, Vec2::ZERO, 0, 1.0);
        app.update();
        let requests: Vec<DamageRequest> =
            app.world_mut().resource_mut::<Messages<DamageRequest>>().drain().collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].origin, Vec2::new(-IMPACT_SETBACK, 0.0));
        assert_eq!(requests[0].attack.damage, SHOT.damage);
    }
}
