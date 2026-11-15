use bevy::prelude::*;

/// Top-level screens.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    /// A run is in progress. Entering it starts a fresh run ([`RunSetup`]).
    /// Tag run-scoped entities with `DespawnOnExit(GameState::InRun)`.
    InRun,
    GameOver,
    Victory,
}

/// What is happening inside a run.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::InRun)]
pub enum RunPhase {
    /// Normal gameplay; all [`GameSet`]s run.
    #[default]
    Fighting,
    /// Gameplay is frozen while the player picks a boon.
    ChoosingBoon,
}

/// Ordered phases of a gameplay frame in `Update`.
/// They only run while in [`RunPhase::Fighting`].
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    /// Turn [`crate::PlayerIntent`] into hero velocity, facing and attacks.
    Input,
    /// Monsters, dragons and the companion decide what to do.
    Ai,
    /// Velocities are integrated and terrain collisions resolved.
    Movement,
    /// Projectiles fly, hits are detected, damage is requested.
    Combat,
    /// Damage requests are applied; deaths, pickups and XP are settled.
    Resolve,
    /// Dead and expired entities are despawned.
    Cleanup,
}

/// Ordered phases of `OnEnter(GameState::InRun)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunSetup {
    /// Run resources are reset (terrain generated, stats and progress zeroed).
    Reset,
    /// Entities are spawned from the freshly reset resources.
    Spawn,
}

pub(crate) fn register(app: &mut App) {
    app.init_state::<GameState>()
        .add_sub_state::<RunPhase>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Ai,
                GameSet::Movement,
                GameSet::Combat,
                GameSet::Resolve,
                GameSet::Cleanup,
            )
                .chain()
                .run_if(in_state(RunPhase::Fighting)),
        )
        .configure_sets(OnEnter(GameState::InRun), (RunSetup::Reset, RunSetup::Spawn).chain());
}
