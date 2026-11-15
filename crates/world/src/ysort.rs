//! Depth sorting: things lower on the screen are drawn in front.

use bevy::prelude::*;
use emberbound_core::{GameSet, YSort, layer};

/// How much z one world unit of y is worth. Small enough that a whole map
/// stays between [`layer::PICKUP`] and [`layer::PROJECTILE`].
const Z_PER_UNIT: f32 = 0.001;

/// The z a y-sorted thing standing at height `y` is drawn at.
pub(crate) fn depth(y: f32) -> f32 {
    layer::ACTORS - y * Z_PER_UNIT
}

/// Re-sorts everything that moved (or was just spawned). Static props are
/// given their [`depth`] once at spawn and carry no [`YSort`] marker, so
/// this only ever visits the handful of moving actors.
fn sort_by_y(mut sorted: Query<&mut Transform, (With<YSort>, Changed<Transform>)>) {
    for mut transform in &mut sorted {
        let z = depth(transform.translation.y);
        // Compare first: writing unconditionally would mark the transform
        // as changed again and defeat the filter.
        if transform.translation.z != z {
            transform.translation.z = z;
        }
    }
}

pub(crate) fn register(app: &mut App) {
    // Not in a `GameSet`: actors spawned while gameplay is frozen still
    // need a depth.
    app.add_systems(Update, sort_by_y.after(GameSet::Movement));
}
