//! Adapter around the domain's encounter [`Director`]: feeds it time and
//! kills, executes its spawn orders and mirrors its state into [`RunStats`].

use bevy::prelude::*;
use emberbound_core::{ActorDied, ActorKind, Announce, GameState, Monster, RunSetup, RunStats, ScreenShake};
use emberbound_domain::encounter::{Director, SpawnOrder};

use crate::AiStep;
use crate::dragon::spawn_dragon;
use crate::hero::{HeroPos, hero_alive};
use crate::spawning::Spawner;

/// The pacing of the current run.
#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub(crate) struct RunDirector(Director);

fn reset_director(mut director: ResMut<RunDirector>) {
    *director = RunDirector::default();
}

/// Tells the director about kills. Runs in every run phase so that a death
/// reported on the frame a boon screen opens is not lost.
fn observe_deaths(mut deaths: MessageReader<ActorDied>, mut director: ResMut<RunDirector>) {
    for death in deaths.read() {
        match death.kind {
            ActorKind::Monster(_) => director.monster_killed(),
            ActorKind::Dragon(_) => {
                director.dragon_slain();
            }
            ActorKind::Hero => {}
        }
    }
}

/// Advances the director and spawns whatever it orders.
fn direct(
    time: Res<Time>,
    mut director: ResMut<RunDirector>,
    hero: HeroPos,
    monsters: Query<(), With<Monster>>,
    mut spawner: Spawner,
    mut announce: MessageWriter<Announce>,
    mut shake: ResMut<ScreenShake>,
) {
    let hero_pos = hero.translation.truncate();
    let orders = director.tick(time.delta_secs(), monsters.iter().len(), &mut *spawner.rng);
    for order in orders {
        match order {
            SpawnOrder::Monster { kind, difficulty } => {
                spawner.spawn_monster_on_ring(&kind.spec(difficulty), hero_pos);
            }
            SpawnOrder::Dragon(kind) => {
                spawn_dragon(&mut spawner, kind, hero_pos, &mut announce, &mut shake);
            }
        }
    }
}

fn sync_run_stats(director: Res<RunDirector>, mut stats: ResMut<RunStats>) {
    stats.lure_progress = director.lure_progress();
    stats.next_dragon = director.next_dragon();
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<RunDirector>()
        .add_systems(OnEnter(GameState::InRun), reset_director.in_set(RunSetup::Reset))
        .add_systems(
            Update,
            (
                (observe_deaths, sync_run_stats).chain().run_if(in_state(GameState::InRun)),
                direct.run_if(hero_alive).in_set(AiStep::Direct),
            ),
        );
}
