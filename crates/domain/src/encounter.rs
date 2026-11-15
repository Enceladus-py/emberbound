//! Encounter context: the director that paces a run.
//!
//! The director decides *what* should enter the valley and *when*. Where a
//! monster appears on screen is a presentation concern left to the adapter.

use crate::bestiary::{DragonKind, MonsterKind};
use crate::ports::RandomSource;

/// Something the director wants spawned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpawnOrder {
    Monster { kind: MonsterKind, difficulty: f32 },
    Dragon(DragonKind),
}

/// Kills needed (since the previous dragon fell) to lure out each dragon.
const KILLS_TO_LURE: [u32; 3] = [18, 30, 42];

#[derive(Debug, Clone, PartialEq)]
pub struct Director {
    elapsed: f32,
    spawn_timer: f32,
    kills_since_dragon: u32,
    next_dragon: Option<DragonKind>,
    dragon_active: bool,
    dragons_slain: u32,
}

impl Default for Director {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            spawn_timer: 1.5,
            kills_since_dragon: 0,
            next_dragon: Some(DragonKind::MossWyrm),
            dragon_active: false,
            dragons_slain: 0,
        }
    }
}

impl Director {
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn dragons_slain(&self) -> u32 {
        self.dragons_slain
    }

    pub fn dragon_active(&self) -> bool {
        self.dragon_active
    }

    /// The dragon that will appear next (or is being fought right now).
    pub fn next_dragon(&self) -> Option<DragonKind> {
        self.next_dragon
    }

    /// All three dragons are dead: the run is won.
    pub fn is_complete(&self) -> bool {
        self.next_dragon.is_none()
    }

    /// How close the hero is to luring out the next dragon, `0.0..=1.0`.
    pub fn lure_progress(&self) -> f32 {
        if self.dragon_active || self.is_complete() {
            return 1.0;
        }
        (self.kills_since_dragon as f32 / self.kills_needed() as f32).min(1.0)
    }

    /// Overall difficulty scalar fed into [`MonsterKind::spec`].
    pub fn difficulty(&self) -> f32 {
        1.0 + self.dragons_slain as f32 * 0.45 + self.elapsed / 300.0
    }

    /// Most lesser monsters allowed alive at once.
    pub fn monster_cap(&self) -> usize {
        if self.dragon_active {
            return 6 + self.dragons_slain as usize * 2;
        }
        (12.0 + self.dragons_slain as f32 * 8.0 + self.elapsed / 15.0).min(48.0) as usize
    }

    fn kills_needed(&self) -> u32 {
        KILLS_TO_LURE[(self.dragons_slain as usize).min(KILLS_TO_LURE.len() - 1)]
    }

    fn spawn_interval(&self) -> f32 {
        let base = (1.5 - self.dragons_slain as f32 * 0.3 - self.elapsed / 400.0).max(0.45);
        if self.dragon_active { base * 2.5 } else { base }
    }

    fn pick_monster(&self, rng: &mut dyn RandomSource) -> MonsterKind {
        // (kind, weight) tables grow nastier with every dragon slain.
        let table: &[(MonsterKind, f32)] = match self.dragons_slain {
            0 if self.elapsed < 25.0 => &[(MonsterKind::Slime, 1.0)],
            0 => &[(MonsterKind::Slime, 6.0), (MonsterKind::Bat, 3.0), (MonsterKind::Goblin, 1.0)],
            1 => &[
                (MonsterKind::Slime, 3.0),
                (MonsterKind::Bat, 3.0),
                (MonsterKind::Goblin, 3.0),
                (MonsterKind::Wyrmling, 1.0),
            ],
            _ => &[
                (MonsterKind::Slime, 1.0),
                (MonsterKind::Bat, 3.0),
                (MonsterKind::Goblin, 3.0),
                (MonsterKind::Wyrmling, 3.0),
            ],
        };
        let total: f32 = table.iter().map(|(_, w)| w).sum();
        let mut roll = rng.range(0.0, total);
        for (kind, weight) in table {
            if roll < *weight {
                return *kind;
            }
            roll -= weight;
        }
        table[table.len() - 1].0
    }

    /// Advances the run by `dt` seconds and returns what should spawn now.
    /// `alive_monsters` is the number of lesser monsters currently alive.
    pub fn tick(&mut self, dt: f32, alive_monsters: usize, rng: &mut dyn RandomSource) -> Vec<SpawnOrder> {
        let mut orders = Vec::new();
        if self.is_complete() {
            return orders;
        }
        self.elapsed += dt;

        if !self.dragon_active
            && self.kills_since_dragon >= self.kills_needed()
            && let Some(dragon) = self.next_dragon
        {
            self.dragon_active = true;
            orders.push(SpawnOrder::Dragon(dragon));
        }

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = self.spawn_interval();
            let room = self.monster_cap().saturating_sub(alive_monsters);
            // Occasionally a small pack arrives together.
            let pack = if rng.chance(0.25) { 3 } else { 1 };
            let kind = self.pick_monster(rng);
            for _ in 0..pack.min(room) {
                orders.push(SpawnOrder::Monster { kind, difficulty: self.difficulty() });
            }
        }
        orders
    }

    /// Call when a lesser monster dies.
    pub fn monster_killed(&mut self) {
        if !self.dragon_active {
            self.kills_since_dragon += 1;
        }
    }

    /// Call when the active dragon dies. Returns `true` if it was the last
    /// one, i.e. the run is won.
    pub fn dragon_slain(&mut self) -> bool {
        if !self.dragon_active {
            return self.is_complete();
        }
        self.dragon_active = false;
        self.dragons_slain += 1;
        self.kills_since_dragon = 0;
        self.next_dragon = self.next_dragon.and_then(DragonKind::next);
        self.is_complete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::testing::TestRng;

    fn lure(director: &mut Director, rng: &mut TestRng) -> DragonKind {
        for _ in 0..200 {
            director.monster_killed();
        }
        director
            .tick(0.1, 0, rng)
            .into_iter()
            .find_map(|o| if let SpawnOrder::Dragon(d) = o { Some(d) } else { None })
            .expect("dragon should be lured")
    }

    #[test]
    fn opens_with_slimes_only() {
        let (mut director, mut rng) = (Director::default(), TestRng(7));
        let mut seen = 0;
        for _ in 0..200 {
            for order in director.tick(0.1, 0, &mut rng) {
                assert!(matches!(order, SpawnOrder::Monster { kind: MonsterKind::Slime, .. }));
                seen += 1;
            }
        }
        assert!(seen > 5);
    }

    #[test]
    fn respects_the_monster_cap() {
        let (mut director, mut rng) = (Director::default(), TestRng(3));
        for _ in 0..100 {
            // The cap grows over time, so a full valley is checked against
            // the cap the director will use for this very tick.
            let cap = director.monster_cap() + 1;
            assert!(director.tick(0.5, cap, &mut rng).is_empty());
        }
    }

    #[test]
    fn three_dragons_in_order_then_victory() {
        let (mut director, mut rng) = (Director::default(), TestRng(11));
        assert_eq!(lure(&mut director, &mut rng), DragonKind::MossWyrm);
        assert!(director.dragon_active());
        // No second dragon while one is active, however many kills.
        for _ in 0..100 {
            director.monster_killed();
            assert!(!director.tick(0.1, 0, &mut rng).iter().any(|o| matches!(o, SpawnOrder::Dragon(_))));
        }
        assert!(!director.dragon_slain());
        assert_eq!(director.lure_progress(), 0.0);
        assert_eq!(lure(&mut director, &mut rng), DragonKind::FrostDrake);
        assert!(!director.dragon_slain());
        assert_eq!(lure(&mut director, &mut rng), DragonKind::AshenTyrant);
        assert!(director.dragon_slain());
        assert!(director.is_complete());
        assert!(director.tick(1.0, 0, &mut rng).is_empty());
    }

    #[test]
    fn difficulty_rises_with_dragons_slain() {
        let (mut director, mut rng) = (Director::default(), TestRng(5));
        let before = director.difficulty();
        lure(&mut director, &mut rng);
        director.dragon_slain();
        assert!(director.difficulty() > before);
    }
}
