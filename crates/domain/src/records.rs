//! Records context: the best run, kept between runs so there is a score to
//! chase.

use crate::bestiary::DragonKind;

/// How a finished run went, as far as records are concerned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BestRun {
    pub dragons_slain: u32,
    pub level: u32,
    /// Lesser monsters slain.
    pub kills: u32,
    /// Seconds spent fighting.
    pub elapsed: f32,
}

impl BestRun {
    /// Whether the run slew every elder dragon.
    pub fn is_victory(&self) -> bool {
        self.dragons_slain as usize >= DragonKind::ALL.len()
    }

    /// Whether this run ranks above `other`: more dragons slain, then a
    /// higher level, then more monsters slain, then (between two victories)
    /// the quicker one. A tie never beats the record.
    pub fn beats(&self, other: &BestRun) -> bool {
        let rank = |run: &BestRun| (run.dragons_slain, run.level, run.kills);
        match rank(self).cmp(&rank(other)) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                self.is_victory() && other.is_victory() && self.elapsed < other.elapsed
            }
        }
    }

    /// The record after `run`, and whether `run` set it. With no record yet,
    /// any run is the best.
    pub fn record(previous: Option<BestRun>, run: BestRun) -> (BestRun, bool) {
        match previous {
            Some(best) if !run.beats(&best) => (best, false),
            _ => (run, true),
        }
    }

    /// Plain `key=value` lines, read back by [`BestRun::from_text`].
    pub fn to_text(&self) -> String {
        format!(
            "dragons_slain={}\nlevel={}\nkills={}\nelapsed={}\n",
            self.dragons_slain, self.level, self.kills, self.elapsed
        )
    }

    /// Reads what [`BestRun::to_text`] wrote. Anything missing, unknown or
    /// malformed means there is no usable record.
    pub fn from_text(text: &str) -> Option<BestRun> {
        let (mut dragons_slain, mut level, mut kills, mut elapsed) = (None, None, None, None);
        for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            let (key, value) = line.split_once('=')?;
            let value = value.trim();
            match key.trim() {
                "dragons_slain" => dragons_slain = Some(value.parse().ok()?),
                "level" => level = Some(value.parse().ok()?),
                "kills" => kills = Some(value.parse().ok()?),
                "elapsed" => {
                    elapsed = Some(value.parse::<f32>().ok().filter(|s| s.is_finite() && *s >= 0.0)?)
                }
                _ => return None,
            }
        }
        Some(BestRun { dragons_slain: dragons_slain?, level: level?, kills: kills?, elapsed: elapsed? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: u32 = DragonKind::ALL.len() as u32;

    fn run(dragons_slain: u32, level: u32, kills: u32, elapsed: f32) -> BestRun {
        BestRun { dragons_slain, level, kills, elapsed }
    }

    #[test]
    fn dragons_outrank_levels_which_outrank_kills() {
        assert!(run(1, 2, 3, 100.0).beats(&run(0, 20, 900, 100.0)));
        assert!(run(1, 5, 0, 100.0).beats(&run(1, 4, 900, 100.0)));
        assert!(run(1, 5, 10, 100.0).beats(&run(1, 5, 9, 1.0)));
        assert!(!run(0, 20, 900, 100.0).beats(&run(1, 2, 3, 100.0)));
    }

    #[test]
    fn time_only_breaks_ties_between_victories() {
        assert!(run(ALL, 9, 50, 300.0).beats(&run(ALL, 9, 50, 400.0)));
        assert!(!run(ALL, 9, 50, 400.0).beats(&run(ALL, 9, 50, 300.0)));
        // Losing quickly or slowly is all the same.
        assert!(!run(1, 9, 50, 300.0).beats(&run(1, 9, 50, 400.0)));
        assert!(!run(1, 9, 50, 400.0).beats(&run(1, 9, 50, 300.0)));
    }

    #[test]
    fn a_tie_keeps_the_old_record() {
        let best = run(2, 8, 40, 250.0);
        assert_eq!(BestRun::record(Some(best), best), (best, false));
    }

    #[test]
    fn the_first_run_is_always_the_best() {
        let first = run(0, 1, 0, 3.0);
        assert_eq!(BestRun::record(None, first), (first, true));
    }

    #[test]
    fn a_better_run_takes_the_record() {
        let (old, new) = (run(0, 4, 30, 100.0), run(1, 3, 10, 200.0));
        assert_eq!(BestRun::record(Some(old), new), (new, true));
        assert_eq!(BestRun::record(Some(new), old), (new, false));
    }

    #[test]
    fn a_record_survives_a_round_trip_through_text() {
        for best in [run(0, 1, 0, 0.0), run(ALL, 17, 1234, 905.25), run(2, 9, 77, 61.5)] {
            assert_eq!(BestRun::from_text(&best.to_text()), Some(best));
        }
    }

    #[test]
    fn missing_or_garbled_text_is_no_record() {
        let full = "dragons_slain=1\nlevel=2\nkills=3\nelapsed=4.5\n";
        assert!(BestRun::from_text(full).is_some());
        for broken in [
            "",
            "garbage",
            "dragons_slain=1\nlevel=2\nkills=3\n",
            "dragons_slain=one\nlevel=2\nkills=3\nelapsed=4.5",
            "dragons_slain=-1\nlevel=2\nkills=3\nelapsed=4.5",
            "dragons_slain=1\nlevel=2\nkills=3\nelapsed=NaN",
            "dragons_slain=1\nlevel=2\nkills=3\nelapsed=-4",
            "dragons_slain=1\nlevel=2\nkills=3\nelapsed=4.5\nscore=9000",
            "\u{0}\u{1}\u{2}",
        ] {
            assert_eq!(BestRun::from_text(broken), None, "{broken:?}");
        }
    }

    #[test]
    fn stray_whitespace_is_forgiven() {
        let text = "  level = 2 \r\n\r\nkills=3\r\ndragons_slain= 1\r\nelapsed =4.5\r\n";
        assert_eq!(BestRun::from_text(text), Some(run(1, 2, 3, 4.5)));
    }
}
