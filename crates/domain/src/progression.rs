//! Progression context: experience, levels and the hero's stats.
//! The boons that change those stats live in [`crate::boons`].

use crate::boons::Boon;
use crate::combat::AttackProfile;

/// Experience and level of the hero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    level: u32,
    xp: u32,
}

impl Default for Progress {
    fn default() -> Self {
        Self { level: 1, xp: 0 }
    }
}

impl Progress {
    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn xp(&self) -> u32 {
        self.xp
    }

    /// Experience needed to go from `level` to `level + 1`.
    pub fn xp_required(level: u32) -> u32 {
        6 + level * 5 + level * level
    }

    pub fn xp_to_next(&self) -> u32 {
        Self::xp_required(self.level)
    }

    /// Progress towards the next level in `0.0..=1.0`.
    pub fn fraction(&self) -> f32 {
        self.xp as f32 / self.xp_to_next() as f32
    }

    /// Adds experience and returns how many levels were gained.
    pub fn add_xp(&mut self, amount: u32) -> u32 {
        self.xp += amount;
        let mut gained = 0;
        while self.xp >= self.xp_to_next() {
            self.xp -= self.xp_to_next();
            self.level += 1;
            gained += 1;
        }
        gained
    }
}

/// Below this fraction of health, [`HeroStats::last_stand`] kicks in.
pub const LAST_STAND_BELOW: f32 = 0.35;
const LAST_STAND_DAMAGE: f32 = 1.5;
const LAST_STAND_SPEED: f32 = 1.15;

/// Everything about the hero that boons can change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeroStats {
    pub max_health: f32,
    /// World units per second.
    pub move_speed: f32,
    pub attack: AttackProfile,
    /// Seconds between sword slashes.
    pub attack_cooldown: f32,
    /// Radius of the sword arc in world units.
    pub attack_reach: f32,
    pub armor: f32,
    /// Seconds between dashes.
    pub dash_cooldown: f32,
    /// Distance at which embers fly to the hero.
    pub pickup_radius: f32,
    pub regen_per_second: f32,
    /// Damage of one fire bolt of the dragon companion.
    pub companion_damage: f32,
    /// Seconds between companion volleys.
    pub companion_cooldown: f32,
    /// Extra enemies each fire bolt passes through.
    pub companion_pierce: u32,
    /// Damage per burn tick of the flames a dash leaves behind (0 = none).
    pub dash_trail_damage: f32,
    /// Damage dealt back to anything that hurts the hero by touch.
    pub thorns: f32,
    /// Health restored whenever the sword lands a critical hit.
    pub crit_heal: f32,
    /// Every n-th swing also hurls a crescent of light (0 = never).
    pub wave_every: u32,
    /// Harder and faster below [`LAST_STAND_BELOW`] health.
    pub last_stand: bool,
    /// Times per run a fatal blow is survived instead.
    pub rebirths: u32,
    /// Multiplier on the experience of collected embers.
    pub xp_multiplier: f32,
    /// Chance that a slain monster bursts into fire bolts.
    pub corpse_burst_chance: f32,
    /// Chance that a slain monster drops a heart.
    pub heart_chance: f32,
    /// Health restored by a heart.
    pub heart_heal: f32,
    /// How many times each boon was taken, indexed by `Boon as usize`.
    stacks: [u8; Boon::ALL.len()],
}

impl Default for HeroStats {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            move_speed: 78.0,
            attack: AttackProfile { damage: 12.0, crit_chance: 0.05, crit_multiplier: 2.0, knockback: 120.0 },
            attack_cooldown: 0.42,
            attack_reach: 26.0,
            armor: 0.0,
            dash_cooldown: 1.6,
            pickup_radius: 34.0,
            regen_per_second: 0.0,
            companion_damage: 6.0,
            companion_cooldown: 1.5,
            companion_pierce: 0,
            dash_trail_damage: 0.0,
            thorns: 0.0,
            crit_heal: 0.0,
            wave_every: 0,
            last_stand: false,
            rebirths: 0,
            xp_multiplier: 1.0,
            corpse_burst_chance: 0.0,
            heart_chance: 0.06,
            heart_heal: 20.0,
            stacks: [0; Boon::ALL.len()],
        }
    }
}

impl HeroStats {
    /// How many times `boon` has been taken this run.
    pub fn stacks(&self, boon: Boon) -> u8 {
        self.stacks[boon as usize]
    }

    /// Whether `boon` may still be offered.
    pub fn can_take(&self, boon: Boon) -> bool {
        boon.max_stacks().is_none_or(|max| self.stacks(boon) < max)
    }

    /// Applies a boon permanently.
    pub fn apply(&mut self, boon: Boon) {
        self.stacks[boon as usize] = self.stacks[boon as usize].saturating_add(1);
        boon.bless(self);
    }

    fn in_last_stand(&self, health_fraction: f32) -> bool {
        self.last_stand && health_fraction < LAST_STAND_BELOW
    }

    /// The sword blow at the given health.
    pub fn sword_attack(&self, health_fraction: f32) -> AttackProfile {
        let mut attack = self.attack;
        if self.in_last_stand(health_fraction) {
            attack.damage *= LAST_STAND_DAMAGE;
        }
        attack
    }

    /// Walking speed at the given health.
    pub fn speed(&self, health_fraction: f32) -> f32 {
        if self.in_last_stand(health_fraction) { self.move_speed * LAST_STAND_SPEED } else { self.move_speed }
    }

    /// Experience granted for embers worth `amount`.
    pub fn xp_for(&self, amount: u32) -> u32 {
        (amount as f32 * self.xp_multiplier).round() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xp_carries_over_and_can_grant_several_levels() {
        let mut progress = Progress::default();
        assert_eq!(progress.add_xp(Progress::xp_required(1) - 1), 0);
        assert_eq!(progress.add_xp(1), 1);
        assert_eq!((progress.level(), progress.xp()), (2, 0));
        let two_levels = Progress::xp_required(2) + Progress::xp_required(3) + 3;
        assert_eq!(progress.add_xp(two_levels), 2);
        assert_eq!((progress.level(), progress.xp()), (4, 3));
    }

    #[test]
    fn levels_get_more_expensive() {
        assert!((1..30).all(|l| Progress::xp_required(l + 1) > Progress::xp_required(l)));
    }

    #[test]
    fn last_stand_only_helps_when_low() {
        let mut stats = HeroStats::default();
        assert_eq!(stats.sword_attack(0.1), stats.attack);
        stats.last_stand = true;
        assert_eq!(stats.sword_attack(0.9), stats.attack);
        assert!(stats.sword_attack(0.1).damage > stats.attack.damage);
        assert!(stats.speed(0.1) > stats.speed(0.9));
    }

    #[test]
    fn embers_are_worth_more_with_a_multiplier() {
        let mut stats = HeroStats::default();
        assert_eq!(stats.xp_for(4), 4);
        stats.xp_multiplier = 1.25;
        assert_eq!(stats.xp_for(4), 5);
    }
}
