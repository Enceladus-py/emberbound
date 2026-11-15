//! Bestiary context: what lives in the valley and how dangerous it is.

/// How a lesser monster moves and fights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Behavior {
    /// Walks straight at the hero.
    Chase,
    /// Flies over terrain, weaving erratically towards the hero.
    Swoop,
    /// Keeps its distance and throws projectiles.
    Skirmish { preferred_distance: f32, shot_cooldown: f32, shot_damage: f32, shot_speed: f32 },
    /// Pauses to wind up, then lunges in a straight line.
    Lunge { windup: f32, lunge_speed: f32, lunge_time: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonsterKind {
    Slime,
    Bat,
    Goblin,
    Wyrmling,
}

/// Concrete numbers for one monster at a given difficulty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonsterSpec {
    pub kind: MonsterKind,
    pub max_health: f32,
    /// World units per second.
    pub speed: f32,
    pub contact_damage: f32,
    pub armor: f32,
    pub xp_reward: u32,
    /// Collision radius in world units.
    pub radius: f32,
    pub flies: bool,
    pub behavior: Behavior,
}

impl MonsterKind {
    pub const ALL: [MonsterKind; 4] =
        [MonsterKind::Slime, MonsterKind::Bat, MonsterKind::Goblin, MonsterKind::Wyrmling];

    pub fn name(self) -> &'static str {
        match self {
            MonsterKind::Slime => "Bog Slime",
            MonsterKind::Bat => "Dusk Bat",
            MonsterKind::Goblin => "Goblin Slinger",
            MonsterKind::Wyrmling => "Wyrmling",
        }
    }

    /// Stats at `difficulty` (1.0 is the start of a run). Health and damage
    /// scale with difficulty; speed only creeps up so fights stay readable.
    pub fn spec(self, difficulty: f32) -> MonsterSpec {
        let d = difficulty.max(1.0);
        let speed_scale = 1.0 + (d - 1.0) * 0.12;
        let (hp, speed, dmg, armor, xp, radius, flies, behavior) = match self {
            MonsterKind::Slime => (18.0, 34.0, 6.0, 0.0, 2, 6.0, false, Behavior::Chase),
            MonsterKind::Bat => (10.0, 52.0, 5.0, 0.0, 2, 5.0, true, Behavior::Swoop),
            MonsterKind::Goblin => (
                24.0,
                34.0,
                7.0,
                1.0,
                4,
                6.0,
                false,
                Behavior::Skirmish {
                    preferred_distance: 72.0,
                    shot_cooldown: 2.2,
                    shot_damage: 7.0 * d,
                    shot_speed: 95.0,
                },
            ),
            MonsterKind::Wyrmling => (
                42.0,
                30.0,
                12.0,
                2.0,
                7,
                7.0,
                false,
                Behavior::Lunge { windup: 0.55, lunge_speed: 170.0, lunge_time: 0.45 },
            ),
        };
        MonsterSpec {
            kind: self,
            max_health: (hp * d).round(),
            speed: speed * speed_scale,
            contact_damage: (dmg * (1.0 + (d - 1.0) * 0.6)).round(),
            armor,
            xp_reward: xp,
            radius,
            flies,
            behavior,
        }
    }
}

/// The three elder dragons, in the order they must be slain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DragonKind {
    MossWyrm,
    FrostDrake,
    AshenTyrant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonPhase {
    /// Circles the hero and breathes cones of projectiles.
    Stalking,
    /// Below the enrage threshold: faster, and adds radial novas.
    Enraged,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonSpec {
    pub kind: DragonKind,
    pub name: &'static str,
    pub epithet: &'static str,
    pub max_health: f32,
    pub speed: f32,
    pub contact_damage: f32,
    pub armor: f32,
    pub xp_reward: u32,
    pub radius: f32,
    /// Distance the dragon tries to keep from the hero while stalking.
    pub orbit_distance: f32,
    pub breath_damage: f32,
    /// Projectiles per breath, fanned across `breath_spread_degrees`.
    pub breath_projectiles: u32,
    pub breath_spread_degrees: f32,
    pub breath_cooldown: f32,
    pub projectile_speed: f32,
    /// Projectiles in the radial burst used while enraged.
    pub nova_projectiles: u32,
    pub nova_cooldown: f32,
    /// Health fraction below which the dragon enrages.
    pub enrage_below: f32,
    /// Minions called in while enraged, if any.
    pub summons: Option<MonsterKind>,
}

impl DragonKind {
    pub const ALL: [DragonKind; 3] = [DragonKind::MossWyrm, DragonKind::FrostDrake, DragonKind::AshenTyrant];

    /// The dragon that follows this one, if any.
    pub fn next(self) -> Option<DragonKind> {
        match self {
            DragonKind::MossWyrm => Some(DragonKind::FrostDrake),
            DragonKind::FrostDrake => Some(DragonKind::AshenTyrant),
            DragonKind::AshenTyrant => None,
        }
    }

    pub fn spec(self) -> DragonSpec {
        match self {
            DragonKind::MossWyrm => DragonSpec {
                kind: self,
                name: "Vyrdmaw",
                epithet: "the Moss Wyrm",
                max_health: 420.0,
                speed: 34.0,
                contact_damage: 16.0,
                armor: 1.0,
                xp_reward: 40,
                radius: 16.0,
                orbit_distance: 80.0,
                breath_damage: 9.0,
                breath_projectiles: 5,
                breath_spread_degrees: 50.0,
                breath_cooldown: 2.6,
                projectile_speed: 80.0,
                nova_projectiles: 10,
                nova_cooldown: 5.0,
                enrage_below: 0.4,
                summons: Some(MonsterKind::Slime),
            },
            DragonKind::FrostDrake => DragonSpec {
                kind: self,
                name: "Hrimvald",
                epithet: "the Frost Drake",
                max_health: 800.0,
                speed: 44.0,
                contact_damage: 20.0,
                armor: 2.0,
                xp_reward: 70,
                radius: 16.0,
                orbit_distance: 90.0,
                breath_damage: 12.0,
                breath_projectiles: 7,
                breath_spread_degrees: 70.0,
                breath_cooldown: 2.2,
                projectile_speed: 95.0,
                nova_projectiles: 14,
                nova_cooldown: 4.2,
                enrage_below: 0.45,
                summons: Some(MonsterKind::Bat),
            },
            DragonKind::AshenTyrant => DragonSpec {
                kind: self,
                name: "Morkhaal",
                epithet: "the Ashen Tyrant",
                max_health: 1400.0,
                speed: 50.0,
                contact_damage: 26.0,
                armor: 3.0,
                xp_reward: 0,
                radius: 22.0,
                orbit_distance: 96.0,
                breath_damage: 15.0,
                breath_projectiles: 9,
                breath_spread_degrees: 90.0,
                breath_cooldown: 1.9,
                projectile_speed: 105.0,
                nova_projectiles: 18,
                nova_cooldown: 3.6,
                enrage_below: 0.5,
                summons: Some(MonsterKind::Wyrmling),
            },
        }
    }
}

impl DragonSpec {
    pub fn phase(&self, health_fraction: f32) -> DragonPhase {
        if health_fraction < self.enrage_below { DragonPhase::Enraged } else { DragonPhase::Stalking }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_makes_monsters_tougher() {
        for kind in MonsterKind::ALL {
            let (easy, hard) = (kind.spec(1.0), kind.spec(3.0));
            assert!(hard.max_health > easy.max_health);
            assert!(hard.contact_damage > easy.contact_damage);
            assert!(hard.speed < easy.speed * 1.5, "speed must not explode");
        }
    }

    #[test]
    fn dragons_escalate_and_end_with_the_tyrant() {
        let specs: Vec<_> = DragonKind::ALL.iter().map(|k| k.spec()).collect();
        assert!(specs.windows(2).all(|w| w[1].max_health > w[0].max_health));
        assert_eq!(DragonKind::MossWyrm.next(), Some(DragonKind::FrostDrake));
        assert_eq!(DragonKind::AshenTyrant.next(), None);
    }

    #[test]
    fn dragons_enrage_when_wounded() {
        let spec = DragonKind::MossWyrm.spec();
        assert_eq!(spec.phase(1.0), DragonPhase::Stalking);
        assert_eq!(spec.phase(0.1), DragonPhase::Enraged);
    }
}
