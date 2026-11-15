//! Companion context: the bonded dragon that grows with the hero.

/// The companion grows as the hero levels up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GrowthStage {
    Hatchling,
    Fledgling,
    Drake,
}

impl GrowthStage {
    pub fn for_level(level: u32) -> Self {
        match level {
            0..=3 => GrowthStage::Hatchling,
            4..=7 => GrowthStage::Fledgling,
            _ => GrowthStage::Drake,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            GrowthStage::Hatchling => "Hatchling",
            GrowthStage::Fledgling => "Fledgling",
            GrowthStage::Drake => "Drake",
        }
    }

    /// Fire bolts spat per volley.
    pub fn bolts_per_volley(self) -> u32 {
        match self {
            GrowthStage::Hatchling => 1,
            GrowthStage::Fledgling => 2,
            GrowthStage::Drake => 3,
        }
    }

    pub fn damage_multiplier(self) -> f32 {
        match self {
            GrowthStage::Hatchling => 1.0,
            GrowthStage::Fledgling => 1.3,
            GrowthStage::Drake => 1.7,
        }
    }

    /// How far the companion can see targets, in world units.
    pub fn sight_range(self) -> f32 {
        match self {
            GrowthStage::Hatchling => 90.0,
            GrowthStage::Fledgling => 110.0,
            GrowthStage::Drake => 130.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_with_level_and_never_shrinks() {
        assert_eq!(GrowthStage::for_level(1), GrowthStage::Hatchling);
        assert_eq!(GrowthStage::for_level(4), GrowthStage::Fledgling);
        assert_eq!(GrowthStage::for_level(8), GrowthStage::Drake);
        assert!((1..40).all(|l| GrowthStage::for_level(l + 1) >= GrowthStage::for_level(l)));
    }
}
