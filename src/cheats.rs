//! Play-testing shortcuts, compiled only with `--features cheats`.
//!
//! * `G` - grant a pile of embers (levels up)
//! * `H` - smite every monster and dragon on the map
//! * `T` - fill the hero's health

use bevy::prelude::*;
use emberbound_core::{DamageRequest, GameSet, Health, Player, Team, XpCollected};
use emberbound_domain::combat::AttackProfile;

pub struct CheatsPlugin;

impl Plugin for CheatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cheat.in_set(GameSet::Input));
    }
}

fn cheat(
    keys: Res<ButtonInput<KeyCode>>,
    horde: Query<(Entity, &Team, &Transform)>,
    mut hero: Query<&mut Health, With<Player>>,
    mut xp: MessageWriter<XpCollected>,
    mut damage: MessageWriter<DamageRequest>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        xp.write(XpCollected { amount: 40 });
    }
    if keys.just_pressed(KeyCode::KeyH) {
        for (target, team, transform) in &horde {
            if *team == Team::Horde {
                let origin = transform.translation.truncate();
                damage.write(DamageRequest { target, attack: AttackProfile::flat(99_999.0, 0.0), origin });
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyT)
        && let Ok(mut health) = hero.single_mut()
    {
        health.heal(99_999.0);
    }
}
