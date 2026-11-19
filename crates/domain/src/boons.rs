//! Boons context: the permanent upgrades offered on level up, how rare they
//! are, how often they can be taken and what they do to the hero.

use crate::ports::RandomSource;
use crate::progression::HeroStats;

/// How rare a boon is. Rarer boons change how a run plays, not just its
/// numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Rare,
    Legendary,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Legendary => "Legendary",
        }
    }

    /// Relative chance of being drawn at a given hero level. Legendaries
    /// grow likelier as the run goes on.
    fn weight(self, level: u32) -> f32 {
        match self {
            Rarity::Common => 60.0,
            Rarity::Rare => 26.0,
            Rarity::Legendary => 2.0 + level as f32 * 1.2,
        }
    }
}

/// What part of the hero's craft a boon belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    Blade,
    Vigor,
    Flight,
    Cinder,
    Embers,
}

impl Aspect {
    pub fn name(self) -> &'static str {
        match self {
            Aspect::Blade => "Blade",
            Aspect::Vigor => "Vigor",
            Aspect::Flight => "Flight",
            Aspect::Cinder => "Cinder",
            Aspect::Embers => "Embers",
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
    EmberWake,
    Thornhide,
    SanguineEdge,
    CrescentWave,
    TwinFlame,
    LastStand,
    MoltenHeart,
    PhoenixFeather,
    EmberTithe,
    Wildfire,
    Hearthkeeper,
}

impl Boon {
    pub const ALL: [Boon; 23] = [
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
        Boon::EmberWake,
        Boon::Thornhide,
        Boon::SanguineEdge,
        Boon::CrescentWave,
        Boon::TwinFlame,
        Boon::LastStand,
        Boon::MoltenHeart,
        Boon::PhoenixFeather,
        Boon::EmberTithe,
        Boon::Wildfire,
        Boon::Hearthkeeper,
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
            Boon::EmberWake => "Ember Wake",
            Boon::Thornhide => "Thornhide",
            Boon::SanguineEdge => "Sanguine Edge",
            Boon::CrescentWave => "Crescent Wave",
            Boon::TwinFlame => "Twin Flame",
            Boon::LastStand => "Last Stand",
            Boon::MoltenHeart => "Molten Heart",
            Boon::PhoenixFeather => "Phoenix Feather",
            Boon::EmberTithe => "Ember Tithe",
            Boon::Wildfire => "Wildfire",
            Boon::Hearthkeeper => "Hearthkeeper",
        }
    }

    /// What the boon grants.
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
            Boon::EmberWake => "Your dash leaves a trail of fire that burns the horde",
            Boon::Thornhide => "Monsters that touch you take 8 damage",
            Boon::SanguineEdge => "Critical hits heal you for 2",
            Boon::CrescentWave => "Every third swing hurls a crescent of light that cuts through foes",
            Boon::TwinFlame => "Cinder's bolts pierce one more foe",
            Boon::LastStand => "Below 35% health: +50% sword damage and +15% speed",
            Boon::MoltenHeart => "+60% sword damage and +30% dragon bolt damage",
            Boon::PhoenixFeather => {
                "Once, a fatal blow instead rekindles you at half health in a burst of flame"
            }
            Boon::EmberTithe => "Embers grant 25% more experience",
            Boon::Wildfire => "Slain monsters have a 20% chance to burst into fire bolts",
            Boon::Hearthkeeper => "Hearts drop more often and heal 10 more",
        }
    }

    /// The price of a cursed boon, if it has one.
    pub fn curse(self) -> Option<&'static str> {
        match self {
            Boon::MoltenHeart => Some("-30 max health"),
            _ => None,
        }
    }

    pub fn rarity(self) -> Rarity {
        match self {
            Boon::Kindling
            | Boon::Stokedflame
            | Boon::EmberWake
            | Boon::SanguineEdge
            | Boon::TwinFlame
            | Boon::LastStand
            | Boon::Wildfire => Rarity::Rare,
            Boon::CrescentWave | Boon::MoltenHeart | Boon::PhoenixFeather => Rarity::Legendary,
            _ => Rarity::Common,
        }
    }

    pub fn aspect(self) -> Aspect {
        match self {
            Boon::WhettedEdge
            | Boon::KeenEye
            | Boon::SweepingArc
            | Boon::Fervor
            | Boon::SanguineEdge
            | Boon::CrescentWave
            | Boon::MoltenHeart => Aspect::Blade,
            Boon::WardensVigor
            | Boon::Scalemail
            | Boon::Kindling
            | Boon::Thornhide
            | Boon::LastStand
            | Boon::PhoenixFeather
            | Boon::Hearthkeeper => Aspect::Vigor,
            Boon::Fleetfoot | Boon::Wingbeat | Boon::EmberWake => Aspect::Flight,
            Boon::EmberHeart | Boon::Stokedflame | Boon::TwinFlame | Boon::Wildfire => Aspect::Cinder,
            Boon::Lodestone | Boon::EmberTithe => Aspect::Embers,
        }
    }

    /// How many times it can be taken in one run (`None` = without limit).
    pub fn max_stacks(self) -> Option<u8> {
        match self {
            Boon::CrescentWave | Boon::LastStand | Boon::MoltenHeart | Boon::PhoenixFeather => Some(1),
            Boon::EmberWake | Boon::TwinFlame | Boon::Hearthkeeper => Some(2),
            Boon::Thornhide | Boon::SanguineEdge | Boon::EmberTithe | Boon::Wildfire => Some(3),
            Boon::Wingbeat | Boon::Stokedflame => Some(4),
            Boon::KeenEye | Boon::Fervor => Some(5),
            _ => None,
        }
    }

    /// Changes the hero's stats. Called once per stack by [`HeroStats::apply`].
    pub(crate) fn bless(self, stats: &mut HeroStats) {
        match self {
            Boon::WhettedEdge => stats.attack.damage *= 1.25,
            Boon::WardensVigor => stats.max_health += 25.0,
            Boon::Fleetfoot => stats.move_speed *= 1.12,
            Boon::KeenEye => stats.attack.crit_chance = (stats.attack.crit_chance + 0.10).min(1.0),
            Boon::EmberHeart => stats.companion_damage *= 1.4,
            Boon::Wingbeat => stats.dash_cooldown = (stats.dash_cooldown * 0.75).max(0.3),
            Boon::Lodestone => stats.pickup_radius *= 1.5,
            Boon::Scalemail => stats.armor += 2.0,
            Boon::SweepingArc => stats.attack_reach *= 1.2,
            Boon::Kindling => stats.regen_per_second += 1.0,
            Boon::Fervor => stats.attack_cooldown = (stats.attack_cooldown * 0.85).max(0.12),
            Boon::Stokedflame => stats.companion_cooldown = (stats.companion_cooldown * 0.8).max(0.25),
            Boon::EmberWake => stats.dash_trail_damage += 7.0,
            Boon::Thornhide => stats.thorns += 8.0,
            Boon::SanguineEdge => stats.crit_heal += 2.0,
            Boon::CrescentWave => stats.wave_every = 3,
            Boon::TwinFlame => stats.companion_pierce += 1,
            Boon::LastStand => stats.last_stand = true,
            Boon::MoltenHeart => {
                stats.attack.damage *= 1.6;
                stats.companion_damage *= 1.3;
                stats.max_health = (stats.max_health - 30.0).max(30.0);
            }
            Boon::PhoenixFeather => stats.rebirths += 1,
            Boon::EmberTithe => stats.xp_multiplier += 0.25,
            Boon::Wildfire => stats.corpse_burst_chance = (stats.corpse_burst_chance + 0.2).min(1.0),
            Boon::Hearthkeeper => {
                stats.heart_chance += 0.04;
                stats.heart_heal += 10.0;
            }
        }
    }
}

/// Draws up to `count` distinct boons the hero can still take, weighted by
/// rarity at the hero's `level`.
pub fn offer_boons(count: usize, stats: &HeroStats, level: u32, rng: &mut dyn RandomSource) -> Vec<Boon> {
    let mut pool: Vec<Boon> = Boon::ALL.into_iter().filter(|boon| stats.can_take(*boon)).collect();
    let mut offer = Vec::with_capacity(count);
    while offer.len() < count && !pool.is_empty() {
        let total: f32 = pool.iter().map(|boon| boon.rarity().weight(level)).sum();
        let mut roll = rng.range(0.0, total);
        let mut picked = pool.len() - 1;
        for (index, boon) in pool.iter().enumerate() {
            let weight = boon.rarity().weight(level);
            if roll < weight {
                picked = index;
                break;
            }
            roll -= weight;
        }
        offer.push(pool.swap_remove(picked));
    }
    offer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::testing::TestRng;

    #[test]
    fn offers_are_distinct() {
        let (stats, mut rng) = (HeroStats::default(), TestRng(42));
        for _ in 0..50 {
            let offer = offer_boons(3, &stats, 5, &mut rng);
            assert_eq!(offer.len(), 3);
            assert!(offer[0] != offer[1] && offer[1] != offer[2] && offer[0] != offer[2]);
        }
        assert_eq!(offer_boons(99, &stats, 5, &mut rng).len(), Boon::ALL.len());
    }

    #[test]
    fn every_boon_changes_the_hero_and_explains_itself() {
        for boon in Boon::ALL {
            // `bless` skips the stack counter, so any difference is the effect.
            let mut blessed = HeroStats::default();
            boon.bless(&mut blessed);
            assert_ne!(blessed, HeroStats::default(), "{boon:?} had no effect");
            assert!(!boon.title().is_empty() && !boon.description().is_empty());
        }
    }

    #[test]
    fn capped_boons_stop_being_offered() {
        let mut stats = HeroStats::default();
        let mut rng = TestRng(9);
        stats.apply(Boon::PhoenixFeather);
        assert!(!stats.can_take(Boon::PhoenixFeather));
        for _ in 0..200 {
            assert!(!offer_boons(3, &stats, 20, &mut rng).contains(&Boon::PhoenixFeather));
        }
        stats.apply(Boon::Thornhide);
        stats.apply(Boon::Thornhide);
        assert!(stats.can_take(Boon::Thornhide));
        stats.apply(Boon::Thornhide);
        assert!(!stats.can_take(Boon::Thornhide));
        assert_eq!(stats.stacks(Boon::Thornhide), 3);
    }

    #[test]
    fn legendaries_are_rare_early_and_commoner_late() {
        let stats = HeroStats::default();
        let count = |level, rng: &mut TestRng| {
            (0..2000)
                .flat_map(|_| offer_boons(3, &stats, level, rng))
                .filter(|boon| boon.rarity() == Rarity::Legendary)
                .count()
        };
        let mut rng = TestRng(77);
        let (early, late) = (count(1, &mut rng), count(15, &mut rng));
        assert!(early > 0, "legendaries must be possible from the start");
        assert!(late > early * 2, "early {early}, late {late}");
    }

    #[test]
    fn molten_heart_costs_health_but_never_all_of_it() {
        let mut stats = HeroStats::default();
        stats.apply(Boon::MoltenHeart);
        assert_eq!(stats.max_health, 70.0);
        assert!(stats.attack.damage > HeroStats::default().attack.damage);
        assert_eq!(Boon::MoltenHeart.curse(), Some("-30 max health"));
        let mut frail = HeroStats::default();
        frail.max_health = 40.0;
        Boon::MoltenHeart.bless(&mut frail);
        assert_eq!(frail.max_health, 30.0);
    }
}
