//! Ember Wake: a dash leaves burning patches behind that hurt the horde.

use bevy::prelude::*;
use emberbound_core::SpriteId;
use emberbound_core::{
    Body, DamageRequest, DashCooldown, GameState, Health, HeroState, Player, SpriteLibrary, Team, layer,
};
use emberbound_domain::combat::AttackProfile;

/// How long after a dash starts the trail keeps being laid (the dash
/// itself lasts a little less).
const TRAIL_SECONDS: f32 = 0.22;
/// Distance between two patches of one trail.
const PATCH_SPACING: f32 = 9.0;
const PATCH_LIFETIME: f32 = 2.5;
const BURN_INTERVAL: f32 = 0.4;
const BURN_RADIUS: f32 = 9.0;
const BURN_KNOCKBACK: f32 = 20.0;
const PATCH_FPS: f32 = 9.0;
/// Burning patches lie on the ground, under everything standing on it.
const PATCH_Z: f32 = layer::SHADOW + 0.5;

/// The trail being laid by the current dash.
#[derive(Resource, Default)]
pub struct Trail {
    /// Seconds of laying left (0 = not dashing).
    remaining: f32,
    last_patch: Option<Vec2>,
    /// Dash cooldown seen last frame; a jump up means a dash started.
    last_cooldown: f32,
}

pub fn reset(mut trail: ResMut<Trail>) {
    *trail = Trail::default();
}

/// One burning patch.
#[derive(Component)]
pub struct FirePatch {
    damage: f32,
    life: Timer,
    burn: Timer,
}

/// Starts a trail when the hero dashes and drops patches along it.
pub fn lay_trail(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    library: Res<SpriteLibrary>,
    hero: Single<(&Transform, &DashCooldown), With<Player>>,
    mut trail: ResMut<Trail>,
    mut commands: Commands,
) {
    let (transform, cooldown) = *hero;
    let dashed = cooldown.remaining > trail.last_cooldown + 0.05;
    trail.last_cooldown = cooldown.remaining;
    let damage = hero_state.stats.dash_trail_damage;
    if damage <= 0.0 {
        return;
    }
    if dashed {
        trail.remaining = TRAIL_SECONDS;
        trail.last_patch = None;
    }
    if trail.remaining <= 0.0 {
        return;
    }
    trail.remaining -= time.delta_secs();

    let here = transform.translation.truncate();
    if trail.last_patch.is_some_and(|last| last.distance(here) < PATCH_SPACING) {
        return;
    }
    trail.last_patch = Some(here);
    // Patches sit at the hero's feet.
    let feet = here - Vec2::Y * 4.0;
    commands.spawn((
        library.animated(SpriteId::FirePatch, PATCH_FPS),
        Transform::from_translation(feet.extend(PATCH_Z)),
        FirePatch {
            damage,
            life: Timer::from_seconds(PATCH_LIFETIME, TimerMode::Once),
            burn: Timer::from_seconds(BURN_INTERVAL, TimerMode::Repeating),
        },
        DespawnOnExit(GameState::InRun),
    ));
}

/// Patches burn whatever of the horde stands in them, then gutter out.
pub fn burn(
    time: Res<Time>,
    mut patches: Query<(Entity, &Transform, &mut FirePatch, &mut Sprite)>,
    horde: Query<(Entity, &Transform, &Team, &Body, &Health)>,
    mut commands: Commands,
    mut damage: MessageWriter<DamageRequest>,
) {
    for (entity, transform, mut patch, mut sprite) in &mut patches {
        if patch.life.tick(time.delta()).is_finished() {
            commands.entity(entity).try_despawn();
            continue;
        }
        // Fade over the last third of its life.
        let left = patch.life.remaining_secs() / PATCH_LIFETIME;
        sprite.color.set_alpha((left * 3.0).min(1.0));
        if !patch.burn.tick(time.delta()).just_finished() {
            continue;
        }
        let origin = transform.translation.truncate();
        for (target, victim, team, body, health) in &horde {
            if *team == Team::Horde
                && !health.is_dead()
                && victim.translation.truncate().distance(origin) <= BURN_RADIUS + body.radius
            {
                damage.write(DamageRequest {
                    target,
                    attack: AttackProfile::flat(patch.damage, BURN_KNOCKBACK),
                    origin,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use emberbound_core::{CorePlugin, Monster};
    use emberbound_domain::bestiary::MonsterKind;

    use super::*;

    const FRAME: Duration = Duration::from_millis(20);
    const BODY: Body = Body { radius: 6.0, walks: true };

    fn app(trail_damage: f32) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            // The sprite library uploads into these; no renderer is needed.
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins(CorePlugin)
            .init_resource::<Trail>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME))
            .add_systems(Update, (lay_trail, burn).chain());
        app.world_mut().resource_mut::<HeroState>().stats.dash_trail_damage = trail_damage;
        app
    }

    fn spawn_hero(app: &mut App) -> Entity {
        let cooldown = DashCooldown { remaining: 0.0, total: 1.0 };
        let hero = app.world_mut().spawn((Player, Transform::default(), cooldown, Team::Hero, BODY)).id();
        // The first update only starts the clock.
        app.update();
        hero
    }

    /// Moves the hero by `step` and lets the dash cooldown tick, for `frames` updates.
    fn run(app: &mut App, hero: Entity, step: Vec2, frames: usize) {
        for _ in 0..frames {
            let mut entity = app.world_mut().entity_mut(hero);
            entity.get_mut::<Transform>().unwrap().translation += step.extend(0.0);
            let mut cooldown = entity.get_mut::<DashCooldown>().unwrap();
            cooldown.remaining = (cooldown.remaining - FRAME.as_secs_f32()).max(0.0);
            app.update();
        }
    }

    fn dash(app: &mut App, hero: Entity) {
        app.world_mut().get_mut::<DashCooldown>(hero).unwrap().remaining = 1.0;
    }

    fn patches(app: &mut App) -> Vec<Vec2> {
        let world = app.world_mut();
        world
            .query_filtered::<&Transform, With<FirePatch>>()
            .iter(world)
            .map(|t| t.translation.truncate())
            .collect()
    }

    #[test]
    fn no_trail_without_the_boon() {
        let mut app = app(0.0);
        let hero = spawn_hero(&mut app);
        dash(&mut app, hero);
        run(&mut app, hero, Vec2::X * 5.0, 20);
        assert!(patches(&mut app).is_empty());
    }

    #[test]
    fn a_dash_lays_spaced_patches_for_a_short_while() {
        let mut app = app(4.0);
        let hero = spawn_hero(&mut app);
        dash(&mut app, hero);
        // 4 units a frame for the whole trail, and well beyond it.
        let frames = (TRAIL_SECONDS / FRAME.as_secs_f32()).ceil() as usize + 1;
        run(&mut app, hero, Vec2::X * 4.0, frames);
        let mut laid = patches(&mut app);
        assert!(laid.len() >= 3, "{} patches", laid.len());
        laid.sort_by(|a, b| a.x.total_cmp(&b.x));
        for pair in laid.windows(2) {
            assert!(pair[0].distance(pair[1]) >= PATCH_SPACING, "{} and {} too close", pair[0], pair[1]);
        }
        let reach = frames as f32 * 4.0;
        assert!(laid.iter().all(|patch| patch.x <= reach), "patches only along the dash");

        let count = laid.len();
        run(&mut app, hero, Vec2::X * 4.0, 30);
        assert_eq!(patches(&mut app).len(), count, "the trail ends after {TRAIL_SECONDS}s");
    }

    #[test]
    fn a_ticking_cooldown_is_not_a_dash() {
        let mut app = app(4.0);
        let hero = spawn_hero(&mut app);
        // Mid-cooldown, as last seen: the value only goes down from here.
        app.world_mut().get_mut::<DashCooldown>(hero).unwrap().remaining = 0.8;
        app.world_mut().resource_mut::<Trail>().last_cooldown = 0.8;
        run(&mut app, hero, Vec2::X * 5.0, 30);
        assert!(patches(&mut app).is_empty());

        // The next dash does start one.
        dash(&mut app, hero);
        run(&mut app, hero, Vec2::X * 5.0, 2);
        assert!(!patches(&mut app).is_empty());
    }

    fn spawn_patch(app: &mut App, at: Vec2) -> Entity {
        let patch = FirePatch {
            damage: 3.0,
            life: Timer::from_seconds(PATCH_LIFETIME, TimerMode::Once),
            burn: Timer::from_seconds(BURN_INTERVAL, TimerMode::Repeating),
        };
        app.world_mut()
            .spawn((patch, Sprite::default(), Transform::from_translation(at.extend(PATCH_Z))))
            .id()
    }

    fn spawn_actor(app: &mut App, team: Team, at: Vec2) -> Entity {
        let transform = Transform::from_translation(at.extend(0.0));
        let monster = Monster { kind: MonsterKind::Slime, xp_reward: 1 };
        app.world_mut().spawn((monster, transform, team, BODY, Health::full(20.0))).id()
    }

    fn burned(app: &mut App) -> Vec<Entity> {
        let mut messages = app.world_mut().resource_mut::<Messages<DamageRequest>>();
        messages.drain().map(|request| request.target).collect()
    }

    #[test]
    fn patches_burn_the_living_horde_in_reach_on_each_interval_then_gutter_out() {
        let mut app = app(0.0);
        spawn_hero(&mut app);
        let patch = spawn_patch(&mut app, Vec2::new(100.0, 0.0));
        let victim =
            spawn_actor(&mut app, Team::Horde, Vec2::new(100.0 + BURN_RADIUS + BODY.radius - 0.5, 0.0));
        spawn_actor(&mut app, Team::Horde, Vec2::new(100.0 + BURN_RADIUS + BODY.radius + 0.5, 0.0));
        spawn_actor(&mut app, Team::Hero, Vec2::new(100.0, 0.0));
        let corpse = spawn_actor(&mut app, Team::Horde, Vec2::new(100.0, 1.0));
        app.world_mut().get_mut::<Health>(corpse).unwrap().damage(1_000.0);

        let per_burn = (BURN_INTERVAL / FRAME.as_secs_f32()).round() as usize;
        let mut burns = Vec::new();
        // One spare frame: the clock may need a frame to get going.
        let frames = per_burn * 2 + 1;
        for frame in 1..=frames {
            app.update();
            let hit = burned(&mut app);
            if !hit.is_empty() {
                assert_eq!(hit, vec![victim], "only the living horde in reach burns");
                burns.push(frame);
            }
        }
        assert_eq!(burns.len(), 2, "burns on frames {burns:?}");
        assert!(burns[0] >= per_burn, "burned before {BURN_INTERVAL}s: {burns:?}");
        assert_eq!(burns[1] - burns[0], per_burn, "burns every {BURN_INTERVAL}s");

        let lifetime = (PATCH_LIFETIME / FRAME.as_secs_f32()).round() as usize;
        for _ in frames..lifetime - 1 {
            app.update();
        }
        assert!(app.world().get_entity(patch).is_ok(), "still burning just before its end");
        for _ in 0..3 {
            app.update();
        }
        assert!(app.world().get_entity(patch).is_err(), "gone after {PATCH_LIFETIME}s");
    }
}
