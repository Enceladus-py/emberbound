//! Where the hero's sword goes. A deliberate aim (mouse or a dragging sword
//! finger) wins. Otherwise aim assist turns the sword to an enemy within
//! reach: while moving, only to ones roughly ahead, so walking past a crowd
//! does not spin the hero around; standing still, to the nearest one in any
//! direction (ahead wins near-ties), which is what frees a surrounded hero.
//! With nobody near, the sword follows facing.

use bevy::prelude::*;
use emberbound_core::{Aim, Body, Facing, Health, HeroState, Player, PlayerIntent, Team};

use crate::sword::SwingCooldown;

/// Assist reaches this much (world units) past the sword, so the sword is
/// already turning as an enemy steps into range.
const ASSIST_MARGIN: f32 = 12.0;
/// Half-angle of the cone ahead that assist keeps to while moving.
const ASSIST_CONE_DEG: f32 = 60.0;
/// Standing still, an enemy outside that cone counts as this much (world
/// units) further away: ahead wins near-ties, but a monster hugging the
/// hero's back still beats one at the edge of reach in front.
const BEHIND_PENALTY: f32 = 8.0;
/// Seconds the hero keeps looking along the aim after a swing.
pub(crate) const SWING_LOOK_SECONDS: f32 = 0.3;

/// An enemy aim assist may pick: who, where relative to the hero, how big.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Candidate {
    pub entity: Entity,
    pub offset: Vec2,
    pub radius: f32,
}

/// The direction of the next swing and the enemy assist locked onto, if any.
pub(crate) fn resolve_aim(
    explicit: Option<Vec2>,
    facing: Vec2,
    moving: bool,
    reach: f32,
    candidates: impl IntoIterator<Item = Candidate>,
) -> (Vec2, Option<Entity>) {
    let facing = facing.normalize_or(Vec2::X);
    if let Some(direction) = explicit.and_then(Vec2::try_normalize) {
        return (direction, None);
    }
    let cone = ASSIST_CONE_DEG.to_radians();
    let mut best: Option<(f32, Candidate)> = None;
    for candidate in candidates {
        let distance = candidate.offset.length();
        // Measured to the body's edge, so big monsters count when their flank is near.
        let gap = distance - candidate.radius;
        if gap > reach + ASSIST_MARGIN || distance <= f32::EPSILON {
            continue;
        }
        let ahead = facing.angle_to(candidate.offset).abs() <= cone;
        let score = match (ahead, moving) {
            (true, _) => gap,
            (false, true) => continue,
            (false, false) => gap + BEHIND_PENALTY,
        };
        if best.is_none_or(|(best, _)| score < best) {
            best = Some((score, candidate));
        }
    }
    match best {
        Some((_, candidate)) => (candidate.offset.normalize(), Some(candidate.entity)),
        None => (facing, None),
    }
}

/// Settles the hero's [`Aim`] for this frame, before the sword swings.
pub fn aim(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    hero_state: Res<HeroState>,
    cooldown: Res<SwingCooldown>,
    mut hero: Single<(&Transform, &Facing, &mut Aim), With<Player>>,
    enemies: Query<(Entity, &Transform, &Team, &Body, &Health), Without<Player>>,
) {
    let (transform, facing, aim) = &mut *hero;
    let origin = transform.translation.truncate();
    let candidates = enemies
        .iter()
        .filter(|(.., team, _, health)| **team == Team::Horde && !health.is_dead())
        .map(|(entity, enemy, _, body, _)| Candidate {
            entity,
            offset: enemy.translation.truncate() - origin,
            radius: body.radius,
        });
    let moving = intent.movement != Vec2::ZERO;
    let (direction, target) =
        resolve_aim(intent.aim, facing.0, moving, hero_state.stats.attack_reach, candidates);
    aim.direction = direction;
    aim.target = target;
    aim.ready = cooldown.is_ready();
    aim.swinging = (aim.swinging - time.delta_secs()).max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    const REACH: f32 = 26.0;

    fn enemy(index: u32, degrees: f32, distance: f32) -> Candidate {
        Candidate {
            entity: Entity::from_raw_u32(index + 1).unwrap(),
            offset: Vec2::from_angle(degrees.to_radians()) * distance,
            radius: 5.0,
        }
    }

    fn close(a: Vec2, b: Vec2) -> bool {
        a.distance(b) < 1e-4
    }

    #[test]
    fn a_deliberate_aim_always_wins() {
        let up = Vec2::Y;
        let (direction, target) = resolve_aim(Some(up), Vec2::X, false, REACH, [enemy(0, 0.0, 10.0)]);
        assert!(close(direction, up));
        assert_eq!(target, None);
    }

    #[test]
    fn with_nobody_near_the_sword_follows_facing() {
        let (direction, target) = resolve_aim(None, Vec2::NEG_X, false, REACH, [enemy(0, 0.0, 200.0)]);
        assert!(close(direction, Vec2::NEG_X));
        assert_eq!(target, None);
        assert_eq!(resolve_aim(None, Vec2::Y, true, REACH, []).0, Vec2::Y);
    }

    #[test]
    fn assist_picks_the_nearest_enemy_ahead() {
        let near = enemy(0, 30.0, 20.0);
        let far = enemy(1, -10.0, 30.0);
        let (direction, target) = resolve_aim(None, Vec2::X, true, REACH, [far, near]);
        assert_eq!(target, Some(near.entity));
        assert!(close(direction, near.offset.normalize()));
    }

    #[test]
    fn walking_past_an_enemy_behind_does_not_turn_the_sword() {
        let behind = enemy(0, 180.0, 12.0);
        let (direction, target) = resolve_aim(None, Vec2::X, true, REACH, [behind]);
        assert!(close(direction, Vec2::X));
        assert_eq!(target, None);
    }

    #[test]
    fn standing_still_the_nearest_enemy_anywhere_is_struck() {
        let behind = enemy(0, 180.0, 12.0);
        let side = enemy(1, 90.0, 25.0);
        let (direction, target) = resolve_aim(None, Vec2::X, false, REACH, [side, behind]);
        assert_eq!(target, Some(behind.entity));
        assert!(close(direction, Vec2::NEG_X));
    }

    #[test]
    fn a_surrounded_hero_strikes_what_hugs_them_not_what_is_at_the_edge_ahead() {
        let hugging = enemy(0, 180.0, 8.0);
        let edge_ahead = enemy(1, -55.0, 30.0);
        let (_, target) = resolve_aim(None, Vec2::X, false, REACH, [edge_ahead, hugging]);
        assert_eq!(target, Some(hugging.entity));
    }

    #[test]
    fn standing_still_ahead_wins_a_near_tie() {
        let behind = enemy(0, 180.0, 18.0);
        let ahead = enemy(1, 20.0, 22.0);
        let (_, target) = resolve_aim(None, Vec2::X, false, REACH, [behind, ahead]);
        assert_eq!(target, Some(ahead.entity));
    }

    #[test]
    fn big_enemies_count_from_their_edge() {
        let mut dragon = enemy(0, 0.0, REACH + ASSIST_MARGIN + 15.0);
        dragon.radius = 22.0;
        assert_eq!(resolve_aim(None, Vec2::X, true, REACH, [dragon]).1, Some(dragon.entity));
    }
}
