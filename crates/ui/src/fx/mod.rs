//! World-space feedback during a run: damage numbers, hit flashes and
//! particle effects. Purely cosmetic; everything is run-scoped.

mod damage_numbers;
mod flash;
mod particles;

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((damage_numbers::plugin, flash::plugin, particles::plugin));
}
