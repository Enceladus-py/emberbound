//! Application layer: the life cycle of a run. Resets run resources, turns
//! collected embers into levels and boons, and decides when a run is lost
//! or won. It orchestrates the domain; it holds no rules of its own.

use bevy::prelude::*;
use emberbound_core::{
    ActorDied, ActorKind, Armor, BoonChosen, BoonOffer, GameRng, GameSet, GameState, Health, HeroState,
    LeveledUp, Player, PlayerIntent, RunPhase, RunSetup, RunStats, Terrain, XpCollected,
};
use emberbound_domain::progression::offer_boons;
use emberbound_domain::world;

/// Size of the valley in tiles.
const MAP_SIZE: (u32, u32) = (72, 72);
const BOONS_PER_OFFER: usize = 3;
/// Seconds the world lingers after the hero falls or the last dragon dies.
const AFTERMATH_SECONDS: f32 = 1.4;

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingBoons>()
            .init_resource::<Aftermath>()
            .add_systems(OnEnter(GameState::InRun), reset_run.in_set(RunSetup::Reset))
            .add_systems(
                Update,
                (track_run, grant_levels, offer_next_boon).chain().in_set(GameSet::Resolve),
            )
            .add_systems(Update, apply_chosen_boon.run_if(in_state(RunPhase::ChoosingBoon)))
            .add_systems(Update, finish_run.run_if(in_state(GameState::InRun)))
            .add_systems(Update, start_run.run_if(not(in_state(GameState::InRun))));
    }
}

/// Level-ups that still owe the player a boon choice.
#[derive(Resource, Default)]
struct PendingBoons(u32);

/// Set when the run has been decided; the outcome screen follows shortly.
#[derive(Resource, Default)]
struct Aftermath(Option<(Timer, GameState)>);

fn start_run(intent: Res<PlayerIntent>, mut next: ResMut<NextState<GameState>>) {
    if intent.confirm {
        next.set(GameState::InRun);
    }
}

fn reset_run(
    time: Res<Time<Real>>,
    mut rng: ResMut<GameRng>,
    mut terrain: ResMut<Terrain>,
    mut hero: ResMut<HeroState>,
    mut stats: ResMut<RunStats>,
    mut offer: ResMut<BoonOffer>,
    mut pending: ResMut<PendingBoons>,
    mut aftermath: ResMut<Aftermath>,
) {
    // The moment the player pressed start is as good a seed as any.
    *rng = GameRng::new(time.elapsed().as_nanos() as u64);
    terrain.0 = world::generate(MAP_SIZE.0, MAP_SIZE.1, &mut *rng);
    *hero = HeroState::default();
    *stats = RunStats::default();
    offer.0.clear();
    pending.0 = 0;
    aftermath.0 = None;
}

fn track_run(
    time: Res<Time>,
    mut stats: ResMut<RunStats>,
    mut deaths: MessageReader<ActorDied>,
    mut aftermath: ResMut<Aftermath>,
) {
    stats.elapsed += time.delta_secs();
    for death in deaths.read() {
        let outcome = match death.kind {
            ActorKind::Monster(_) => {
                stats.kills += 1;
                None
            }
            ActorKind::Dragon(kind) => {
                stats.dragons_slain += 1;
                kind.next().is_none().then_some(GameState::Victory)
            }
            ActorKind::Hero => Some(GameState::GameOver),
        };
        if let (Some(outcome), None) = (outcome, &aftermath.0) {
            aftermath.0 = Some((Timer::from_seconds(AFTERMATH_SECONDS, TimerMode::Once), outcome));
        }
    }
}

fn grant_levels(
    mut collected: MessageReader<XpCollected>,
    mut hero: ResMut<HeroState>,
    mut pending: ResMut<PendingBoons>,
    mut leveled: MessageWriter<LeveledUp>,
) {
    for xp in collected.read() {
        let gained = hero.progress.add_xp(xp.amount);
        if gained > 0 {
            pending.0 += gained;
            leveled.write(LeveledUp { level: hero.progress.level() });
        }
    }
}

fn offer_next_boon(
    pending: Res<PendingBoons>,
    aftermath: Res<Aftermath>,
    mut rng: ResMut<GameRng>,
    mut offer: ResMut<BoonOffer>,
    mut phase: ResMut<NextState<RunPhase>>,
) {
    if pending.0 > 0 && aftermath.0.is_none() {
        offer.0 = offer_boons(BOONS_PER_OFFER, &mut *rng);
        phase.set(RunPhase::ChoosingBoon);
    }
}

fn apply_chosen_boon(
    mut chosen: MessageReader<BoonChosen>,
    mut hero: ResMut<HeroState>,
    mut pending: ResMut<PendingBoons>,
    mut rng: ResMut<GameRng>,
    mut offer: ResMut<BoonOffer>,
    mut phase: ResMut<NextState<RunPhase>>,
    mut player: Query<(&mut Health, &mut Armor), With<Player>>,
) {
    for BoonChosen(boon) in chosen.read() {
        if pending.0 == 0 {
            continue;
        }
        hero.stats.apply(*boon);
        if let Ok((mut health, mut armor)) = player.single_mut() {
            health.set_max(hero.stats.max_health);
            armor.0 = hero.stats.armor;
        }
        pending.0 -= 1;
        if pending.0 > 0 {
            offer.0 = offer_boons(BOONS_PER_OFFER, &mut *rng);
        } else {
            offer.0.clear();
            phase.set(RunPhase::Fighting);
        }
    }
}

fn finish_run(time: Res<Time>, mut aftermath: ResMut<Aftermath>, mut next: ResMut<NextState<GameState>>) {
    if let Some((timer, outcome)) = aftermath.0.as_mut()
        && timer.tick(time.delta()).is_finished()
    {
        next.set(*outcome);
        aftermath.0 = None;
    }
}
