//! Application layer: the life cycle of a run. Resets run resources, turns
//! collected embers into levels and boons, and decides when a run is lost
//! or won. It orchestrates the domain; it holds no rules of its own.

use bevy::prelude::*;
use bevy::window::{WindowFocused, WindowOccluded};
use emberbound_core::{
    ActorDied, ActorKind, Armor, BoonChosen, BoonOffer, GameRng, GameSet, GameState, Health, HeroState,
    LeveledUp, Player, PlayerIntent, RunPhase, RunSetup, RunStats, Terrain, XpCollected,
};
use emberbound_domain::boons::offer_boons;
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
            .add_systems(Update, (track_run, grant_levels, offer_next_boon).chain().in_set(GameSet::Resolve))
            .add_systems(Update, apply_chosen_boon.run_if(in_state(RunPhase::ChoosingBoon)))
            .add_systems(Update, toggle_pause.run_if(in_state(GameState::InRun)))
            .add_systems(
                Update,
                finish_run.run_if(in_state(GameState::InRun)).run_if(not(in_state(RunPhase::Paused))),
            )
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
    hero: Res<HeroState>,
    aftermath: Res<Aftermath>,
    mut rng: ResMut<GameRng>,
    mut offer: ResMut<BoonOffer>,
    mut phase: ResMut<NextState<RunPhase>>,
) {
    if pending.0 > 0 && aftermath.0.is_none() {
        offer.0 = offer_boons(BOONS_PER_OFFER, &hero.stats, hero.progress.level(), &mut *rng);
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
            offer.0 = offer_boons(BOONS_PER_OFFER, &hero.stats, hero.progress.level(), &mut *rng);
        } else {
            offer.0.clear();
            phase.set(RunPhase::Fighting);
        }
    }
}

/// Esc / P pause and resume a fight; Enter or a tap also resume. Losing
/// window focus (another tab, app or notification) pauses too, so the hero
/// is never left to die unattended. Boon choice is never paused.
fn toggle_pause(
    intent: Res<PlayerIntent>,
    phase: Res<State<RunPhase>>,
    mut focus: MessageReader<WindowFocused>,
    mut occlusion: MessageReader<WindowOccluded>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    let lost_focus = focus.read().any(|event| !event.focused);
    let hidden = occlusion.read().any(|event| event.occluded);
    match phase.get() {
        RunPhase::Fighting if intent.pause || lost_focus || hidden => next.set(RunPhase::Paused),
        RunPhase::Paused if intent.pause || intent.confirm || intent.tap.is_some() => {
            next.set(RunPhase::Fighting)
        }
        _ => {}
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

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use emberbound_core::{CorePlugin, PlayerIntent};
    use emberbound_domain::bestiary::{DragonKind, MonsterKind};
    use emberbound_domain::boons::Boon;
    use emberbound_domain::progression::Progress;

    use super::*;

    /// A run in progress, fighting, with a deterministic rng.
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            // The sprite library uploads into these; no renderer is needed.
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_message::<WindowFocused>()
            .add_message::<WindowOccluded>()
            .add_plugins((CorePlugin, SessionPlugin));
        app.world_mut().resource_mut::<PlayerIntent>().confirm = true;
        app.update(); // start_run asks for the run...
        app.world_mut().resource_mut::<PlayerIntent>().confirm = false;
        app.update(); // ...which begins (and resets) here.
        assert_eq!(*app.world().resource::<State<RunPhase>>().get(), RunPhase::Fighting);
        app.insert_resource(GameRng::new(7));
        app
    }

    fn spawn_hero(app: &mut App) -> Entity {
        app.world_mut().spawn((Player, Health::full(100.0), Armor(0.0))).id()
    }

    fn phase(app: &App) -> RunPhase {
        *app.world().resource::<State<RunPhase>>().get()
    }

    fn set_phase(app: &mut App, phase: RunPhase) {
        app.world_mut().resource_mut::<NextState<RunPhase>>().set(phase);
        app.update();
    }

    fn pending(app: &App) -> u32 {
        app.world().resource::<PendingBoons>().0
    }

    fn offer(app: &App) -> Vec<Boon> {
        app.world().resource::<BoonOffer>().0.clone()
    }

    fn outcome(app: &App) -> Option<GameState> {
        app.world().resource::<Aftermath>().0.as_ref().map(|(_, outcome)| *outcome)
    }

    fn drain<M: Message>(app: &mut App) -> Vec<M> {
        app.world_mut().resource_mut::<Messages<M>>().drain().collect()
    }

    fn die(app: &mut App, kind: ActorKind) {
        app.world_mut().write_message(ActorDied { entity: Entity::PLACEHOLDER, kind, position: Vec2::ZERO });
    }

    /// Embers that take a fresh hero up exactly `levels` levels.
    fn xp_for_levels(levels: u32) -> (u32, u32) {
        let mut progress = Progress::default();
        let mut xp = 0;
        for _ in 0..levels {
            let needed = progress.xp_to_next() - progress.xp();
            xp += needed;
            progress.add_xp(needed);
        }
        (xp, progress.level())
    }

    #[test]
    fn one_big_haul_grants_several_levels_and_a_single_level_up() {
        let mut app = app();
        let (xp, level) = xp_for_levels(3);
        app.world_mut().write_message(XpCollected { amount: xp });
        app.update();

        assert_eq!(pending(&app), 3);
        let leveled: Vec<u32> = drain::<LeveledUp>(&mut app).iter().map(|m| m.level).collect();
        assert_eq!(leveled, vec![level]);
        assert_eq!(offer(&app).len(), BOONS_PER_OFFER);
        app.update();
        assert_eq!(phase(&app), RunPhase::ChoosingBoon);
    }

    #[test]
    fn no_boon_screen_once_the_run_is_decided() {
        let mut app = app();
        die(&mut app, ActorKind::Hero);
        app.world_mut().write_message(XpCollected { amount: xp_for_levels(1).0 });
        app.update();
        app.update();

        assert_eq!(outcome(&app), Some(GameState::GameOver));
        assert_eq!(pending(&app), 1);
        assert!(offer(&app).is_empty());
        assert_eq!(phase(&app), RunPhase::Fighting);
    }

    #[test]
    fn the_boon_queue_drains_back_to_fighting_and_syncs_the_hero() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        app.world_mut().get_mut::<Health>(hero).unwrap().damage(40.0);
        app.world_mut().write_message(XpCollected { amount: xp_for_levels(2).0 });
        app.update();
        app.update();
        assert_eq!(phase(&app), RunPhase::ChoosingBoon);

        app.world_mut().write_message(BoonChosen(Boon::Scalemail));
        app.update();
        assert_eq!(pending(&app), 1);
        assert_eq!(offer(&app).len(), BOONS_PER_OFFER, "a fresh offer for the next level");
        assert_eq!(app.world().get::<Armor>(hero).unwrap().0, 2.0);
        assert_eq!(phase(&app), RunPhase::ChoosingBoon);

        app.world_mut().write_message(BoonChosen(Boon::WardensVigor));
        app.update();
        assert_eq!(pending(&app), 0);
        assert!(offer(&app).is_empty());
        let health = app.world().get::<Health>(hero).unwrap();
        assert_eq!((health.current(), health.max()), (85.0, 125.0));
        let stats = app.world().resource::<HeroState>().stats;
        assert_eq!((stats.stacks(Boon::Scalemail), stats.stacks(Boon::WardensVigor)), (1, 1));

        app.update();
        assert_eq!(phase(&app), RunPhase::Fighting);
        assert!(offer(&app).is_empty(), "no new offer once the queue is empty");
    }

    #[test]
    fn a_stray_boon_choice_is_ignored() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        set_phase(&mut app, RunPhase::ChoosingBoon);
        app.world_mut().write_message(BoonChosen(Boon::Scalemail));
        app.update();
        app.update();

        assert_eq!(app.world().resource::<HeroState>().stats.stacks(Boon::Scalemail), 0);
        assert_eq!(app.world().get::<Armor>(hero).unwrap().0, 0.0);
        assert_eq!(pending(&app), 0);
        assert_eq!(phase(&app), RunPhase::ChoosingBoon);
    }

    #[test]
    fn kills_are_counted_and_the_last_dragon_wins_the_run() {
        let mut app = app();
        die(&mut app, ActorKind::Monster(MonsterKind::Slime));
        die(&mut app, ActorKind::Monster(MonsterKind::Slime));
        die(&mut app, ActorKind::Dragon(DragonKind::MossWyrm));
        app.update();
        let stats = *app.world().resource::<RunStats>();
        assert_eq!((stats.kills, stats.dragons_slain), (2, 1));
        assert_eq!(outcome(&app), None, "an earlier dragon does not end the run");

        die(&mut app, ActorKind::Dragon(DragonKind::AshenTyrant));
        die(&mut app, ActorKind::Hero);
        app.update();
        die(&mut app, ActorKind::Hero);
        app.update();
        assert_eq!(outcome(&app), Some(GameState::Victory), "the first outcome wins");
    }

    #[test]
    fn the_heros_death_loses_the_run() {
        let mut app = app();
        die(&mut app, ActorKind::Hero);
        app.update();
        assert_eq!(outcome(&app), Some(GameState::GameOver));
    }

    #[test]
    fn a_new_run_starts_clean() {
        let mut app = app();
        app.world_mut().write_message(XpCollected { amount: xp_for_levels(2).0 });
        die(&mut app, ActorKind::Monster(MonsterKind::Slime));
        app.update();
        // Boon choice freezes the run, so decide it directly.
        app.world_mut().resource_mut::<Aftermath>().0 =
            Some((Timer::from_seconds(60.0, TimerMode::Once), GameState::GameOver));
        assert!(pending(&app) > 0 && !offer(&app).is_empty() && outcome(&app).is_some());

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Title);
        app.update();
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::InRun);
        app.update();

        assert_eq!(pending(&app), 0);
        assert!(offer(&app).is_empty());
        assert_eq!(outcome(&app), None);
        let stats = *app.world().resource::<RunStats>();
        assert_eq!((stats.kills, stats.dragons_slain), (0, 0));
        assert_eq!(app.world().resource::<HeroState>().progress.level(), Progress::default().level());
        assert_eq!(phase(&app), RunPhase::Fighting);
    }

    fn press(app: &mut App, edit: impl FnOnce(&mut PlayerIntent)) {
        edit(&mut app.world_mut().resource_mut::<PlayerIntent>());
        app.update();
        *app.world_mut().resource_mut::<PlayerIntent>() = PlayerIntent::default();
        app.update();
    }

    #[test]
    fn pause_toggles_and_freezes_the_run() {
        let mut app = app();
        press(&mut app, |intent| intent.pause = true);
        assert_eq!(phase(&app), RunPhase::Paused);

        let elapsed = app.world().resource::<RunStats>().elapsed;
        app.world_mut().write_message(XpCollected { amount: xp_for_levels(1).0 });
        app.update();
        app.update();
        assert_eq!(app.world().resource::<RunStats>().elapsed, elapsed, "the clock stops");
        assert_eq!(pending(&app), 0, "nothing is settled while paused");

        press(&mut app, |intent| intent.pause = true);
        assert_eq!(phase(&app), RunPhase::Fighting);
    }

    #[test]
    fn enter_or_a_tap_resumes() {
        let mut app = app();
        press(&mut app, |intent| intent.pause = true);
        press(&mut app, |intent| intent.confirm = true);
        assert_eq!(phase(&app), RunPhase::Fighting);

        press(&mut app, |intent| intent.pause = true);
        press(&mut app, |intent| intent.tap = Some(Vec2::new(10.0, 10.0)));
        assert_eq!(phase(&app), RunPhase::Fighting);
    }

    #[test]
    fn losing_focus_pauses_but_regaining_it_does_not_resume() {
        let mut app = app();
        app.world_mut().write_message(WindowFocused { window: Entity::PLACEHOLDER, focused: false });
        app.update();
        app.update();
        assert_eq!(phase(&app), RunPhase::Paused);

        app.world_mut().write_message(WindowFocused { window: Entity::PLACEHOLDER, focused: true });
        app.update();
        app.update();
        assert_eq!(phase(&app), RunPhase::Paused, "the player resumes when ready");
    }

    #[test]
    fn a_boon_choice_cannot_be_paused() {
        let mut app = app();
        set_phase(&mut app, RunPhase::ChoosingBoon);
        press(&mut app, |intent| intent.pause = true);
        app.world_mut().write_message(WindowFocused { window: Entity::PLACEHOLDER, focused: false });
        app.update();
        app.update();
        assert_eq!(phase(&app), RunPhase::ChoosingBoon);
    }

    #[test]
    fn a_decided_run_waits_while_paused() {
        let mut app = app();
        die(&mut app, ActorKind::Hero);
        app.update();
        press(&mut app, |intent| intent.pause = true);
        app.world_mut()
            .resource_mut::<Aftermath>()
            .0
            .as_mut()
            .unwrap()
            .0
            .set_elapsed(std::time::Duration::from_secs_f32(AFTERMATH_SECONDS));
        app.update();
        app.update();
        assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::InRun);
        assert_eq!(phase(&app), RunPhase::Paused);
    }
}
