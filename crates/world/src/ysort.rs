//! Depth sorting: things lower on the screen are drawn in front.
//!
//! Depth is measured from a [`DepthOrigin`] near the hero rather than from
//! world zero, so however far the hero walks the y-sorted band stays
//! between [`layer::PICKUP`] and [`layer::PROJECTILE`]. When the hero gets
//! far from the origin it jumps to the hero and everything is re-sorted.

use bevy::prelude::*;
use emberbound_core::{GameSet, GameState, Player, RunSetup, YSort, layer};

/// How much z one world unit of y is worth.
const Z_PER_UNIT: f32 = 0.001;
/// How far (in y) the hero may get from the depth origin before it moves.
/// Everything on screen then stays within about `±4.3` of [`layer::ACTORS`].
const RECENTRE_DISTANCE: f32 = 4_000.0;

/// The world y that is drawn at exactly [`layer::ACTORS`].
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct DepthOrigin(f32);

impl DepthOrigin {
    /// The z a y-sorted thing standing at height `y` is drawn at.
    pub(crate) fn depth(self, y: f32) -> f32 {
        layer::ACTORS - (y - self.0) * Z_PER_UNIT
    }
}

/// A prop that never moves (tree, rock, ruin): sorted when spawned and
/// again only when the [`DepthOrigin`] moves.
#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct StaticDepth;

fn reset_origin(mut origin: ResMut<DepthOrigin>) {
    *origin = DepthOrigin::default();
}

/// Moves the origin to the hero once they have wandered far from it, and
/// re-sorts everything, moving or not.
fn recentre(
    mut origin: ResMut<DepthOrigin>,
    mut sorted: Query<(&mut Transform, Has<Player>), Or<(With<YSort>, With<StaticDepth>)>>,
) {
    let Some(hero_y) = sorted.iter().find(|(_, hero)| *hero).map(|(transform, _)| transform.translation.y)
    else {
        return;
    };
    if (hero_y - origin.0).abs() <= RECENTRE_DISTANCE {
        return;
    }
    *origin = DepthOrigin(hero_y.round());
    for (mut transform, _) in &mut sorted {
        transform.translation.z = origin.depth(transform.translation.y);
    }
}

/// Re-sorts everything that moved (or was just spawned). Static props are
/// given their depth once at spawn and carry no [`YSort`] marker, so this
/// only ever visits the handful of moving actors.
fn sort_by_y(origin: Res<DepthOrigin>, mut sorted: Query<&mut Transform, (With<YSort>, Changed<Transform>)>) {
    for mut transform in &mut sorted {
        let z = origin.depth(transform.translation.y);
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
    app.init_resource::<DepthOrigin>()
        .add_systems(OnEnter(GameState::InRun), reset_origin.in_set(RunSetup::Reset))
        .add_systems(Update, (recentre, sort_by_y).chain().after(GameSet::Movement));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_is_measured_from_the_origin() {
        let origin = DepthOrigin(10_000.0);
        assert_eq!(origin.depth(10_000.0), layer::ACTORS);
        assert!(origin.depth(9_990.0) > origin.depth(10_010.0), "lower is in front");
    }

    #[test]
    fn far_from_home_everything_is_re_sorted_around_the_hero() {
        let mut app = App::new();
        app.init_resource::<DepthOrigin>().add_systems(Update, (recentre, sort_by_y).chain());
        let hero = app.world_mut().spawn((Player, YSort, Transform::from_xyz(0.0, 50_000.0, 0.0))).id();
        let tree = app.world_mut().spawn((StaticDepth, Transform::from_xyz(30.0, 49_900.0, 0.0))).id();
        app.update();
        let z = |app: &App, entity| app.world().get::<Transform>(entity).unwrap().translation.z;
        assert_eq!(*app.world().resource::<DepthOrigin>(), DepthOrigin(50_000.0));
        assert_eq!(z(&app, hero), layer::ACTORS);
        assert!(z(&app, tree) > z(&app, hero), "the tree lower down is drawn in front");
        assert!(z(&app, tree) < layer::PROJECTILE && z(&app, tree) > layer::PICKUP);
    }
}
