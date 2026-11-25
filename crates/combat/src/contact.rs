//! Touching the horde hurts.

use bevy::prelude::*;
use emberbound_core::{Body, ContactDamage, DamageRequest, Health, HeroState, Invulnerable, Player, Team};
use emberbound_domain::combat::AttackProfile;

/// Impulse that shoves the hero away from whatever touched them.
const CONTACT_KNOCKBACK: f32 = 170.0;

/// Needs no cooldown of its own: a hurt hero is briefly invulnerable, and
/// invulnerable targets ignore damage requests. Thornhide strikes back only
/// when the touch can actually hurt, so it shares that rhythm: only the
/// first toucher of an unshielded hero is struck back, because `resolve`
/// lets just one hit on the hero land per frame.
pub fn touch(
    hero: Single<(Entity, &Transform, &Body, &Health, Has<Invulnerable>), With<Player>>,
    hero_state: Res<HeroState>,
    horde: Query<(Entity, &Transform, &Team, &ContactDamage, &Body, &Health), Without<Player>>,
    mut damage: MessageWriter<DamageRequest>,
) {
    let (target, hero_transform, hero_body, hero_health, shielded) = *hero;
    let thorns = hero_state.stats.thorns;
    if hero_health.is_dead() {
        return;
    }
    let hero_position = hero_transform.translation.truncate();
    let mut retaliated = false;
    for (attacker, transform, team, contact, body, health) in &horde {
        if *team != Team::Horde || health.is_dead() {
            continue;
        }
        let origin = transform.translation.truncate();
        if origin.distance(hero_position) < body.radius + hero_body.radius {
            damage.write(DamageRequest {
                target,
                attack: AttackProfile::flat(contact.0, CONTACT_KNOCKBACK),
                origin,
            });
            if thorns > 0.0 && !shielded && !retaliated {
                retaliated = true;
                damage.write(DamageRequest {
                    target: attacker,
                    attack: AttackProfile::flat(thorns, CONTACT_KNOCKBACK * 0.5),
                    origin: hero_position,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use emberbound_core::{ActorDied, Announce, DamageApplied, FxRequest, GameRng, Monster, ScreenShake};
    use emberbound_domain::bestiary::MonsterKind;

    use super::*;
    use crate::damage::{Rebirths, bury, resolve};

    const TOUCH: f32 = 10.0;
    const THORNS: f32 = 4.0;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageRequest>()
            .add_message::<DamageApplied>()
            .add_message::<ActorDied>()
            .add_message::<Announce>()
            .add_message::<FxRequest>()
            .init_resource::<HeroState>()
            .init_resource::<Rebirths>()
            .init_resource::<GameRng>()
            .init_resource::<ScreenShake>()
            .add_systems(Update, (touch, bury, resolve).chain());
        app.world_mut().resource_mut::<HeroState>().stats.thorns = THORNS;
        app
    }

    fn spawn_hero(app: &mut App) -> Entity {
        let body = Body { radius: 6.0, walks: true };
        app.world_mut().spawn((Player, Transform::default(), body, Health::full(100.0))).id()
    }

    fn spawn_slime(app: &mut App, x: f32) -> Entity {
        let monster = Monster { kind: MonsterKind::Slime, xp_reward: 1 };
        let body = Body { radius: 6.0, walks: true };
        let bundle = (
            monster,
            Team::Horde,
            ContactDamage(TOUCH),
            body,
            Transform::from_xyz(x, 0.0, 0.0),
            Health::full(20.0),
        );
        app.world_mut().spawn(bundle).id()
    }

    fn health(app: &App, entity: Entity) -> f32 {
        app.world().get::<Health>(entity).unwrap().current()
    }

    fn thorned(app: &App, slimes: &[Entity]) -> usize {
        slimes.iter().filter(|&&slime| health(app, slime) < 20.0).count()
    }

    #[test]
    fn a_single_toucher_hurts_the_hero_and_takes_thorns() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        let slime = spawn_slime(&mut app, 5.0);
        app.update();

        assert_eq!(health(&app, hero), 100.0 - TOUCH);
        assert_eq!(health(&app, slime), 20.0 - THORNS);
    }

    #[test]
    fn thorns_strike_back_once_however_many_monsters_touch() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        let slimes: Vec<_> = [5.0, -5.0, 3.0, -3.0].into_iter().map(|x| spawn_slime(&mut app, x)).collect();
        app.update();

        assert_eq!(health(&app, hero), 100.0 - TOUCH, "the hero takes one hit");
        assert_eq!(thorned(&app, &slimes), 1, "exactly one toucher is struck back");
    }

    #[test]
    fn a_shielded_hero_deals_no_thorns() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        app.world_mut().entity_mut(hero).insert(Invulnerable::seconds(5.0));
        let slimes: Vec<_> = [5.0, -5.0].into_iter().map(|x| spawn_slime(&mut app, x)).collect();
        app.update();

        assert_eq!(health(&app, hero), 100.0);
        assert_eq!(thorned(&app, &slimes), 0);
    }

    #[test]
    fn monsters_out_of_reach_do_not_touch() {
        let mut app = app();
        let hero = spawn_hero(&mut app);
        let slime = spawn_slime(&mut app, 40.0);
        app.update();

        assert_eq!(health(&app, hero), 100.0);
        assert_eq!(health(&app, slime), 20.0);
    }
}
