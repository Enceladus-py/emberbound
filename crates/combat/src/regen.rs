//! Passive regeneration of the hero.

use bevy::prelude::*;
use emberbound_core::{Health, HeroState, Player};

pub fn regenerate(
    time: Res<Time>,
    hero_state: Res<HeroState>,
    mut health: Single<&mut Health, With<Player>>,
) {
    let rate = hero_state.stats.regen_per_second;
    // `heal` never revives, so a fallen hero stays down.
    if rate > 0.0 {
        health.heal(rate * time.delta_secs());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;

    fn app(rate: f32) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<HeroState>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(100)))
            .add_systems(Update, regenerate);
        app.world_mut().resource_mut::<HeroState>().stats.regen_per_second = rate;
        let hero = app.world_mut().spawn((Player, Health::full(100.0))).id();
        // The first update only starts the clock; wound the hero after it.
        app.update();
        (app, hero)
    }

    fn health(app: &App, hero: Entity) -> f32 {
        app.world().get::<Health>(hero).unwrap().current()
    }

    #[test]
    fn health_rises_at_the_regen_rate_up_to_max() {
        let (mut app, hero) = app(5.0);
        app.world_mut().get_mut::<Health>(hero).unwrap().damage(10.0);
        app.update();
        assert!((health(&app, hero) - 90.5).abs() < 1e-4, "{}", health(&app, hero));
        app.update();
        assert!((health(&app, hero) - 91.0).abs() < 1e-4, "{}", health(&app, hero));
        for _ in 0..50 {
            app.update();
        }
        assert_eq!(health(&app, hero), 100.0);
    }

    #[test]
    fn no_regen_without_the_boon() {
        let (mut app, hero) = app(0.0);
        app.world_mut().get_mut::<Health>(hero).unwrap().damage(10.0);
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(health(&app, hero), 90.0);
    }

    #[test]
    fn a_fallen_hero_stays_down() {
        let (mut app, hero) = app(50.0);
        app.world_mut().get_mut::<Health>(hero).unwrap().damage(1_000.0);
        for _ in 0..10 {
            app.update();
        }
        assert!(app.world().get::<Health>(hero).unwrap().is_dead());
        assert_eq!(health(&app, hero), 0.0);
    }
}
