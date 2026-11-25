//! The hero's sword: a short arc swung along the hero's [`Aim`] (see
//! [`crate::aim`]), or along facing where there is no aim.

use bevy::prelude::*;
use emberbound_core::{
    Aim, Body, DamageRequest, Facing, GameState, Health, HeroState, Player, PlayerIntent, SpawnProjectile,
    SpriteId, SpriteLibrary, Team, layer,
};
use emberbound_domain::combat::AttackProfile;

/// Half-angle of the arc that a swing covers, in degrees.
const ARC_HALF_ANGLE_DEG: f32 = 75.0;
/// Anything this close to the hero is hit regardless of angle.
const POINT_BLANK: f32 = 10.0;
const SLASH_FPS: f32 = 20.0;
/// Size of the slash sprite and how far out it is drawn, in reaches.
const ARC_SPAN: f32 = 1.6;
const ARC_OFFSET: f32 = 0.6;

/// Crescent Wave: share of the sword's damage the flying crescent deals,
/// and how it flies.
const WAVE_DAMAGE: f32 = 0.7;
const WAVE_SPEED: f32 = 175.0;
const WAVE_RADIUS: f32 = 7.0;
const WAVE_LIFETIME: f32 = 0.75;
const WAVE_PIERCE: u32 = 4;

/// Seconds until the next swing is allowed. Kept as a plain countdown because
/// boons shorten `attack_cooldown` in the middle of a run. Also counts swings
/// for boons that act on every n-th one.
#[derive(Resource, Default)]
pub struct SwingCooldown {
    remaining: f32,
    swings: u32,
}

impl SwingCooldown {
    /// Whether the next swing may happen now.
    pub fn is_ready(&self) -> bool {
        self.remaining <= 0.0
    }
}

pub fn reset(mut cooldown: ResMut<SwingCooldown>) {
    *cooldown = SwingCooldown::default();
}

/// Whether a body of `radius` whose centre sits at `offset` from the hero
/// overlaps the arc swung along the unit vector `facing`. The arc is widened
/// by the angle the body subtends, so a big monster whose centre lies just
/// outside the arc is still hit when its flank is inside it.
fn in_arc(offset: Vec2, facing: Vec2, reach: f32, radius: f32) -> bool {
    let distance = offset.length();
    if distance > reach + radius {
        return false;
    }
    if distance < POINT_BLANK + radius {
        return true;
    }
    let angle = (offset.dot(facing) / distance).clamp(-1.0, 1.0).acos();
    let subtended = (radius / distance).min(1.0).asin();
    angle <= ARC_HALF_ANGLE_DEG.to_radians() + subtended
}

pub fn swing(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    hero_state: Res<HeroState>,
    library: Res<SpriteLibrary>,
    mut cooldown: ResMut<SwingCooldown>,
    mut hero: Single<(&Transform, &Facing, &Health, Option<&mut Aim>), With<Player>>,
    targets: Query<(Entity, &Transform, &Team, &Body), With<Health>>,
    mut commands: Commands,
    mut damage: MessageWriter<DamageRequest>,
    mut projectiles: MessageWriter<SpawnProjectile>,
) {
    cooldown.remaining = (cooldown.remaining - time.delta_secs()).max(0.0);
    let (hero_transform, facing, health, aim) = &mut *hero;
    if !intent.attack || health.is_dead() || cooldown.remaining > 0.0 {
        return;
    }
    let stats = &hero_state.stats;
    cooldown.remaining = stats.attack_cooldown;
    cooldown.swings += 1;
    let attack = stats.sword_attack(health.fraction());

    let origin = hero_transform.translation.truncate();
    let facing = match aim {
        Some(aim) => {
            aim.swinging = crate::aim::SWING_LOOK_SECONDS;
            aim.direction
        }
        None => facing.0,
    }
    .normalize_or(Vec2::X);
    let reach = stats.attack_reach;

    // The crescent sits near the middle of its frame: drawn ARC_SPAN reaches
    // wide and ARC_OFFSET reaches out, its outer edge lands at the reach.
    let scale = reach * ARC_SPAN / library.size(SpriteId::SlashArc).x;
    commands.spawn((
        library.one_shot(SpriteId::SlashArc, SLASH_FPS),
        Transform::from_translation((origin + facing * reach * ARC_OFFSET).extend(layer::FX))
            .with_rotation(Quat::from_rotation_z(facing.to_angle()))
            .with_scale(Vec3::new(scale, scale, 1.0)),
        DespawnOnExit(GameState::InRun),
    ));

    for (target, transform, team, body) in &targets {
        if *team == Team::Horde
            && in_arc(transform.translation.truncate() - origin, facing, reach, body.radius)
        {
            damage.write(DamageRequest { target, attack, origin });
        }
    }

    if stats.wave_every > 0 && cooldown.swings.is_multiple_of(stats.wave_every) {
        projectiles.write(SpawnProjectile {
            team: Team::Hero,
            sprite: SpriteId::SlashWave,
            position: origin + facing * reach * 0.5,
            velocity: facing * WAVE_SPEED,
            attack: AttackProfile { damage: attack.damage * WAVE_DAMAGE, ..attack },
            radius: WAVE_RADIUS,
            lifetime: WAVE_LIFETIME,
            pierce: WAVE_PIERCE,
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use emberbound_core::CorePlugin;

    use super::*;

    const REACH: f32 = 26.0;
    const SMALL: f32 = 5.0;
    /// The Ashen Tyrant's size.
    const DRAGON: f32 = 22.0;

    fn at(degrees: f32, distance: f32) -> Vec2 {
        Vec2::from_angle(degrees.to_radians()) * distance
    }

    #[test]
    fn a_centre_inside_the_arc_is_hit() {
        assert!(in_arc(at(0.0, 20.0), Vec2::X, REACH, SMALL));
        assert!(in_arc(at(70.0, 20.0), Vec2::X, REACH, SMALL));
        assert!(in_arc(at(-70.0, 20.0), Vec2::X, REACH, SMALL));
    }

    #[test]
    fn a_big_body_overlapping_the_arc_edge_is_hit() {
        // Centre 100 degrees off the facing, but the flank reaches into the arc.
        assert!(in_arc(at(100.0, 35.0), Vec2::X, REACH, DRAGON));
        assert!(in_arc(at(-100.0, 35.0), Vec2::X, REACH, DRAGON));
    }

    #[test]
    fn a_small_body_just_outside_the_arc_edge_is_missed() {
        assert!(!in_arc(at(110.0, 20.0), Vec2::X, REACH, SMALL));
    }

    #[test]
    fn a_target_behind_the_hero_is_missed() {
        assert!(!in_arc(at(180.0, 20.0), Vec2::X, REACH, SMALL));
        // A dragon behind the hero whose body stays clear of the arc.
        assert!(!in_arc(at(180.0, 40.0), Vec2::X, REACH, DRAGON));
    }

    #[test]
    fn anything_at_point_blank_is_hit() {
        assert!(in_arc(at(180.0, POINT_BLANK - 1.0), Vec2::X, REACH, SMALL));
        // A big body pressed against the hero's back.
        assert!(in_arc(at(180.0, POINT_BLANK + DRAGON - 1.0), Vec2::X, REACH, DRAGON));
    }

    #[test]
    fn a_target_out_of_reach_is_missed() {
        assert!(!in_arc(at(0.0, REACH + SMALL + 1.0), Vec2::X, REACH, SMALL));
        assert!(in_arc(at(0.0, REACH + SMALL - 1.0), Vec2::X, REACH, SMALL));
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            // The sprite library uploads into these; no renderer is needed.
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins(CorePlugin)
            .init_resource::<SwingCooldown>()
            .add_systems(Update, swing);
        app.world_mut().resource_mut::<HeroState>().stats.attack_reach = REACH;
        app.world_mut().resource_mut::<PlayerIntent>().attack = true;
        app.world_mut().spawn((Player, Transform::default(), Facing(Vec2::X), Health::full(100.0)));
        app
    }

    fn spawn_target(app: &mut App, team: Team, offset: Vec2, radius: f32) -> Entity {
        let body = Body { radius, walks: true };
        let transform = Transform::from_translation(offset.extend(0.0));
        app.world_mut().spawn((team, body, transform, Health::full(50.0))).id()
    }

    fn struck(app: &mut App) -> Vec<Entity> {
        let requests = app.world_mut().resource_mut::<Messages<DamageRequest>>().drain().collect::<Vec<_>>();
        requests.into_iter().map(|request| request.target).collect()
    }

    #[test]
    fn a_swing_hits_only_the_horde() {
        let mut app = app();
        let monster = spawn_target(&mut app, Team::Horde, at(0.0, 20.0), SMALL);
        spawn_target(&mut app, Team::Hero, at(10.0, 20.0), SMALL);
        app.update();

        assert_eq!(struck(&mut app), vec![monster]);
    }

    #[test]
    fn a_swing_hits_a_dragon_whose_flank_overlaps_the_arc() {
        let mut app = app();
        let dragon = spawn_target(&mut app, Team::Horde, at(100.0, 35.0), DRAGON);
        spawn_target(&mut app, Team::Horde, at(180.0, 20.0), SMALL);
        spawn_target(&mut app, Team::Horde, at(0.0, REACH + SMALL + 1.0), SMALL);
        app.update();

        assert_eq!(struck(&mut app), vec![dragon]);
    }

    #[test]
    fn a_swing_follows_the_aim_and_turns_the_hero_that_way() {
        let mut app = app();
        let hero = app.world_mut().query_filtered::<Entity, With<Player>>().single(app.world()).unwrap();
        // Facing right, aiming left.
        app.world_mut().entity_mut(hero).insert(Aim { direction: Vec2::NEG_X, ..Aim::default() });
        let behind = spawn_target(&mut app, Team::Horde, at(180.0, 20.0), SMALL);
        spawn_target(&mut app, Team::Horde, at(0.0, 20.0), SMALL);
        app.update();

        assert_eq!(struck(&mut app), vec![behind]);
        assert!(app.world().get::<Aim>(hero).unwrap().swinging > 0.0);
    }
}
