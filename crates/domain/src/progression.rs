//! Progression context: experience, levels, hero stats and boons.

use crate::combat::AttackProfile;
use crate::ports::RandomSource;

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
        }
    }
}

/// A permanent upgrade offered on level up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Boon {
    WhettedEdge,
    WardensVigor,
    Fleetfoot,
    KeenEye,
    EmberHeart,
    Wingbeat,
    Lodestone,
    Scalemail,
    SweepingArc,
    Kindling,
    Fervor,
    Stokedflame,
}

impl Boon {
    pub const ALL: [Boon; 12] = [
        Boon::WhettedEdge,
        Boon::WardensVigor,
        Boon::Fleetfoot,
        Boon::KeenEye,
        Boon::EmberHeart,
        Boon::Wingbeat,
        Boon::Lodestone,
        Boon::Scalemail,
        Boon::SweepingArc,
        Boon::Kindling,
        Boon::Fervor,
        Boon::Stokedflame,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Boon::WhettedEdge => "Whetted Edge",
            Boon::WardensVigor => "Warden's Vigor",
            Boon::Fleetfoot => "Fleetfoot",
            Boon::KeenEye => "Keen Eye",
            Boon::EmberHeart => "Ember Heart",
            Boon::Wingbeat => "Wingbeat",
            Boon::Lodestone => "Lodestone",
            Boon::Scalemail => "Scalemail",
            Boon::SweepingArc => "Sweeping Arc",
            Boon::Kindling => "Kindling",
            Boon::Fervor => "Fervor",
            Boon::Stokedflame => "Stoked Flame",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Boon::WhettedEdge => "+25% sword damage",
            Boon::WardensVigor => "+25 max health, and heal it",
            Boon::Fleetfoot => "+12% movement speed",
            Boon::KeenEye => "+10% critical chance",
            Boon::EmberHeart => "+40% dragon bolt damage",
            Boon::Wingbeat => "Dash recharges 25% faster",
            Boon::Lodestone => "+50% ember pickup range",
            Boon::Scalemail => "+2 armor",
            Boon::SweepingArc => "+20% sword reach",
            Boon::Kindling => "Regenerate 1 health per second",
            Boon::Fervor => "Swing 15% faster",
            Boon::Stokedflame => "Your dragon attacks 20% faster",
        }
    }
}

impl HeroStats {
    /// Applies a boon permanently.
    pub fn apply(&mut self, boon: Boon) {
        match boon {
            Boon::WhettedEdge => self.attack.damage *= 1.25,
            Boon::WardensVigor => self.max_health += 25.0,
            Boon::Fleetfoot => self.move_speed *= 1.12,
            Boon::KeenEye => self.attack.crit_chance = (self.attack.crit_chance + 0.10).min(1.0),
            Boon::EmberHeart => self.companion_damage *= 1.4,
            Boon::Wingbeat => self.dash_cooldown = (self.dash_cooldown * 0.75).max(0.3),
            Boon::Lodestone => self.pickup_radius *= 1.5,
            Boon::Scalemail => self.armor += 2.0,
            Boon::SweepingArc => self.attack_reach *= 1.2,
            Boon::Kindling => self.regen_per_second += 1.0,
            Boon::Fervor => self.attack_cooldown = (self.attack_cooldown * 0.85).max(0.12),
            Boon::Stokedflame => self.companion_cooldown = (self.companion_cooldown * 0.8).max(0.25),
        }
    }
}

/// Draws `count` distinct boons to offer the player.
pub fn offer_boons(count: usize, rng: &mut dyn RandomSource) -> Vec<Boon> {
    let mut pool = Boon::ALL.to_vec();
    let mut offer = Vec::with_capacity(count);
    while offer.len() < count && !pool.is_empty() {
        offer.push(pool.swap_remove(rng.index(pool.len())));
    }
    offer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::testing::TestRng;

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
    fn offers_are_distinct() {
        let mut rng = TestRng(42);
        for _ in 0..50 {
            let offer = offer_boons(3, &mut rng);
            assert_eq!(offer.len(), 3);
            assert!(offer[0] != offer[1] && offer[1] != offer[2] && offer[0] != offer[2]);
        }
        assert_eq!(offer_boons(99, &mut rng).len(), Boon::ALL.len());
    }

    #[test]
    fn every_boon_changes_the_hero() {
        for boon in Boon::ALL {
            let mut stats = HeroStats::default();
            stats.apply(boon);
            assert_ne!(stats, HeroStats::default(), "{boon:?} had no effect");
            assert!(!boon.title().is_empty() && !boon.description().is_empty());
        }
    }
}
