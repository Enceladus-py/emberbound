//! Combat context: health, attacks and hit resolution.

use crate::ports::RandomSource;

/// Hit points of anything that can be hurt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    current: f32,
    max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        let max = max.max(1.0);
        Self { current: max, max }
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    /// Remaining health in `0.0..=1.0`.
    pub fn fraction(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Applies damage and reports whether this blow was fatal.
    /// Damaging something already dead is never fatal again.
    pub fn damage(&mut self, amount: f32) -> bool {
        if self.is_dead() {
            return false;
        }
        self.current = (self.current - amount.max(0.0)).max(0.0);
        self.is_dead()
    }

    pub fn heal(&mut self, amount: f32) {
        if !self.is_dead() {
            self.current = (self.current + amount.max(0.0)).min(self.max);
        }
    }

    /// Brings the fallen back with `fraction` of their maximum health.
    pub fn revive(&mut self, fraction: f32) {
        self.current = (self.max * fraction.clamp(0.0, 1.0)).max(1.0);
    }

    /// Changes the maximum; any gained capacity is also healed.
    pub fn set_max(&mut self, max: f32) {
        let max = max.max(1.0);
        let gained = (max - self.max).max(0.0);
        self.max = max;
        self.current = (self.current + gained).min(max);
    }
}

/// Everything needed to describe one offensive blow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackProfile {
    pub damage: f32,
    /// Probability in `0.0..=1.0`.
    pub crit_chance: f32,
    pub crit_multiplier: f32,
    /// Impulse (world units / second) applied to the victim.
    pub knockback: f32,
}

impl AttackProfile {
    /// A plain blow that never crits.
    pub const fn flat(damage: f32, knockback: f32) -> Self {
        Self { damage, crit_chance: 0.0, crit_multiplier: 1.0, knockback }
    }
}

/// Result of resolving an attack against a defender.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitOutcome {
    pub amount: f32,
    pub critical: bool,
}

/// Armor is a flat reduction, but a hit always deals at least 1 damage.
pub fn resolve_hit(attack: &AttackProfile, armor: f32, rng: &mut dyn RandomSource) -> HitOutcome {
    let critical = attack.crit_chance > 0.0 && rng.chance(attack.crit_chance);
    let raw = if critical { attack.damage * attack.crit_multiplier } else { attack.damage };
    HitOutcome { amount: (raw - armor.max(0.0)).max(1.0).round(), critical }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::testing::Fixed;

    #[test]
    fn damage_is_fatal_exactly_once() {
        let mut hp = Health::full(10.0);
        assert!(!hp.damage(4.0));
        assert!(hp.damage(100.0));
        assert!(!hp.damage(1.0));
        assert_eq!(hp.current(), 0.0);
    }

    #[test]
    fn healing_never_exceeds_max_or_revives() {
        let mut hp = Health::full(10.0);
        hp.damage(3.0);
        hp.heal(50.0);
        assert_eq!(hp.current(), 10.0);
        hp.damage(10.0);
        hp.heal(5.0);
        assert!(hp.is_dead());
    }

    #[test]
    fn revival_restores_a_share_of_health() {
        let mut hp = Health::full(80.0);
        hp.damage(500.0);
        hp.revive(0.5);
        assert!(!hp.is_dead());
        assert_eq!(hp.current(), 40.0);
    }

    #[test]
    fn raising_max_heals_the_difference() {
        let mut hp = Health::full(10.0);
        hp.damage(5.0);
        hp.set_max(15.0);
        assert_eq!((hp.current(), hp.max()), (10.0, 15.0));
    }

    #[test]
    fn armor_reduces_but_never_nullifies() {
        let hit = resolve_hit(&AttackProfile::flat(3.0, 0.0), 10.0, &mut Fixed(0.9));
        assert_eq!(hit.amount, 1.0);
        assert!(!hit.critical);
    }

    #[test]
    fn crits_multiply_damage() {
        let attack = AttackProfile { damage: 10.0, crit_chance: 0.5, crit_multiplier: 2.0, knockback: 0.0 };
        let hit = resolve_hit(&attack, 0.0, &mut Fixed(0.1));
        assert!(hit.critical);
        assert_eq!(hit.amount, 20.0);
    }
}
