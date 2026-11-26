//! Cinder, the hero's bonded dragon: hovers at their side, spits fire bolts
//! at the horde and grows as the hero levels up.

use bevy::prelude::*;
use emberbound_core::{
    Announce, Companion, Facing, FxKind, FxRequest, GameState, Health, HeroState, Player, SpawnProjectile,
    SpriteAnim, SpriteId, SpriteLibrary, Team, Terrain, layer,
};
use emberbound_domain::combat::AttackProfile;
use emberbound_domain::companion::GrowthStage;

const COMPANION_FPS: f32 = 6.0;
/// Drawn above every y-sorted actor because it flies.
const COMPANION_Z: f32 = layer::PROJECTILE - 1.0;
/// Where it starts relative to the hero's spawn point.
const SPAWN_OFFSET: Vec2 = Vec2::new(-14.0, 6.0);
/// Resting place relative to the hero: behind their back, slightly up.
const SIDE_DISTANCE: f32 = 16.0;
const SIDE_HEIGHT: f32 = 5.0;
/// Size of the idle figure-eight flown around the resting place.
const WANDER: Vec2 = Vec2::new(7.0, 3.0);
const WANDER_SPEED: f32 = 1.3;
/// Spring towards the resting place; slightly under-damped so it overshoots
/// a little and feels alive.
const SPRING_STIFFNESS: f32 = 42.0;
const SPRING_DAMPING: f32 = 8.5;
/// Height of the hover above the ground point, and how much it breathes.
const HOVER_HEIGHT: f32 = 11.0;
const HOVER_BOB: f32 = 1.5;
const HOVER_BOB_SPEED: f32 = 3.1;
/// Below this horizontal speed the travel direction does not flip the sprite.
const TURN_SPEED: f32 = 6.0;
const SHADOW_SCALE: f32 = 0.7;

const BOLT_SPEED: f32 = 150.0;
const BOLT_KNOCKBACK: f32 = 40.0;
const BOLT_RADIUS: f32 = 4.0;
const BOLT_LIFETIME: f32 = 1.2;
/// Angle between neighbouring bolts of one volley, in radians.
const BOLT_SPREAD: f32 = 0.16;

/// Private flight state of the companion.
#[derive(Component)]
pub struct Flight {
    stage: GrowthStage,
    /// Point on the ground it hovers above; the sprite is drawn higher.
    ground: Vec2,
    velocity: Vec2,
    /// Seconds until the next volley.
    cooldown: f32,
}

/// The companion's shadow, kept on the ground while the body bobs.
#[derive(Component)]
pub struct CompanionShadow;

fn sprite_for(stage: GrowthStage) -> SpriteId {
    match stage {
        GrowthStage::Hatchling => SpriteId::CompanionHatchling,
        GrowthStage::Fledgling => SpriteId::CompanionFledgling,
        GrowthStage::Drake => SpriteId::CompanionDrake,
    }
}

pub fn spawn(
    terrain: Res<Terrain>,
    hero_state: Res<HeroState>,
    library: Res<SpriteLibrary>,
    mut commands: Commands,
) {
    let stage = GrowthStage::for_level(hero_state.progress.level());
    let ground = Vec2::from(terrain.tiles().spawn_point()) + SPAWN_OFFSET;
    commands.spawn((
        Companion,
        library.animated(sprite_for(stage), COMPANION_FPS),
        Transform::from_translation((ground + Vec2::Y * HOVER_HEIGHT).extend(COMPANION_Z)),
        Flight { stage, ground, velocity: Vec2::ZERO, cooldown: hero_state.stats.companion_cooldown },
        DespawnOnExit(GameState::InRun),
        children![(
            CompanionShadow,
            library.sprite(SpriteId::Shadow),
            // Local to the parent: back down on the ground, in the shadow layer.
            Transform::from_xyz(0.0, -HOVER_HEIGHT, layer::SHADOW - COMPANION_Z).with_scale(Vec3::new(
                SHADOW_SCALE,
                SHADOW_SCALE,
                1.0
            )),
        )],
    ));
}

/// Switches to the next growth stage once the hero's level calls for it.
pub fn grow(
    hero_state: Res<HeroState>,
    library: Res<SpriteLibrary>,
    mut companion: Single<(&Transform, &mut Sprite, &mut SpriteAnim, &mut Flight), With<Companion>>,
    mut announce: MessageWriter<Announce>,
    mut fx: MessageWriter<FxRequest>,
) {
    let stage = GrowthStage::for_level(hero_state.progress.level());
    let (transform, sprite, anim, flight) = &mut *companion;
    if flight.stage == stage {
        return;
    }
    flight.stage = stage;
    library.switch(sprite, anim, sprite_for(stage));
    announce.write(Announce {
        title: "Cinder grows!".into(),
        subtitle: format!("{} - spits {} fire bolts per volley", stage.name(), stage.bolts_per_volley()),
    });
    fx.write(FxRequest { kind: FxKind::EmberBurst, position: transform.translation.truncate() });
}

/// Springs towards a wandering point behind the hero and hovers above it.
pub fn follow(
    time: Res<Time>,
    hero: Single<(&Transform, &Facing), (With<Player>, Without<Companion>)>,
    mut companion: Single<(&mut Transform, &mut Flight), With<Companion>>,
    mut shadow: Single<&mut Transform, (With<CompanionShadow>, Without<Companion>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let (hero_transform, facing) = *hero;
    let (transform, flight) = &mut *companion;

    // Stay behind the hero's back; a figure-eight (x at half the frequency
    // of y) keeps it from ever sitting still.
    let side = if facing.x >= 0.0 { -1.0 } else { 1.0 };
    let wander = Vec2::new((t * WANDER_SPEED).sin(), (t * WANDER_SPEED * 2.0).sin()) * WANDER;
    let goal = hero_transform.translation.truncate() + Vec2::new(side * SIDE_DISTANCE, SIDE_HEIGHT) + wander;

    let acceleration = (goal - flight.ground) * SPRING_STIFFNESS - flight.velocity * SPRING_DAMPING;
    flight.velocity += acceleration * dt;
    let step = flight.velocity * dt;
    flight.ground += step;

    let hover = HOVER_HEIGHT + (t * HOVER_BOB_SPEED).sin() * HOVER_BOB;
    transform.translation = (flight.ground + Vec2::Y * hover).extend(COMPANION_Z);
    shadow.translation.y = -hover;
}

/// Looks at the nearest enemy in sight (or along its travel) and spits a
/// volley at it whenever the cooldown allows.
pub fn aim_and_fire(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    mut companion: Single<(&Transform, &mut Sprite, &mut Flight), With<Companion>>,
    horde: Query<(&Transform, &Team, &Health), Without<Companion>>,
    mut shots: MessageWriter<SpawnProjectile>,
) {
    let (transform, sprite, flight) = &mut *companion;
    flight.cooldown = (flight.cooldown - time.delta_secs()).max(0.0);

    let muzzle = transform.translation.truncate();
    let sight = flight.stage.sight_range();
    let nearest = horde
        .iter()
        .filter(|(_, team, health)| **team == Team::Horde && !health.is_dead())
        .map(|(target, ..)| target.translation.truncate() - muzzle)
        .filter(|offset| offset.length() <= sight)
        .min_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));

    let Some(offset) = nearest else {
        if flight.velocity.x.abs() > TURN_SPEED {
            sprite.flip_x = flight.velocity.x < 0.0;
        }
        return;
    };
    sprite.flip_x = offset.x < 0.0;
    if flight.cooldown > 0.0 {
        return;
    }

    let stats = &hero_state.stats;
    flight.cooldown = stats.companion_cooldown;
    let aim = offset.normalize_or(Vec2::X);
    let bolts = flight.stage.bolts_per_volley();
    let attack =
        AttackProfile::flat(stats.companion_damage * flight.stage.damage_multiplier(), BOLT_KNOCKBACK);
    for bolt in 0..bolts {
        let fan = (bolt as f32 - (bolts - 1) as f32 * 0.5) * BOLT_SPREAD;
        shots.write(SpawnProjectile {
            team: Team::Hero,
            sprite: SpriteId::Firebolt,
            position: muzzle,
            velocity: Vec2::from_angle(fan).rotate(aim) * BOLT_SPEED,
            attack,
            radius: BOLT_RADIUS,
            lifetime: BOLT_LIFETIME,
            pierce: stats.companion_pierce,
        });
    }
}
