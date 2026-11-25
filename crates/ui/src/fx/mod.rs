//! World-space feedback during a run: the aim pointer, damage numbers, hit
//! flashes and particle effects. Purely cosmetic; everything is run-scoped.

mod aim;
mod damage_numbers;
mod flash;
mod particles;

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((aim::plugin, damage_numbers::plugin, flash::plugin, particles::plugin));
}
