//! Path-finding for walkers: a flow field that always leads to the hero,
//! over a window of tiles around them so the cost does not depend on how
//! big (or endless) the world is.

use bevy::prelude::*;
use emberbound_core::{GameSet, GameState, RunSetup, Terrain};
use emberbound_domain::world::{FlowField, WorldMap};

use crate::AiStep;
use crate::hero::HeroPos;

/// Closer than this a walker just heads straight for the hero.
const DIRECT_DISTANCE: f32 = 26.0;
/// Tiles on each side of the hero the route covers: 32 tiles is 512 units,
/// beyond the 420-unit leash, so every creature in play has a route and
/// the whole window lies in the terrain kept loaded around the hero.
const ROUTE_RADIUS: i32 = 32;

/// The walking route to the hero from anywhere near them. Rebuilt only
/// when the hero steps onto another tile.
#[derive(Resource, Default)]
pub(crate) struct Trail(Option<FlowField>);

impl Trail {
    /// Unit heading a walker at `position` should take to reach `hero`,
    /// going around obstacles. Falls back to a straight line when no route
    /// exists (e.g. the walker was shoved somewhere cut off).
    pub fn heading(&self, position: Vec2, hero: Vec2) -> Vec2 {
        let direct = (hero - position).normalize_or_zero();
        if position.distance(hero) < DIRECT_DISTANCE {
            return direct;
        }
        self.0
            .as_ref()
            .and_then(|field| field.direction(position.x, position.y))
            .map_or(direct, |(x, y)| Vec2::new(x, y))
    }
}

fn forget(mut trail: ResMut<Trail>) {
    trail.0 = None;
}

fn track_hero(hero: HeroPos, terrain: Res<Terrain>, mut trail: ResMut<Trail>) {
    let tile = WorldMap::tile_coords(hero.translation.x, hero.translation.y);
    if trail.0.as_ref().is_none_or(|field| field.target() != tile) {
        trail.0 = Some(FlowField::around(terrain.tiles(), tile, ROUTE_RADIUS));
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Trail>()
        .add_systems(OnEnter(GameState::InRun), forget.in_set(RunSetup::Reset))
        .add_systems(Update, track_hero.in_set(AiStep::Direct).in_set(GameSet::Ai));
}
