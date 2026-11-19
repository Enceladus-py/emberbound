//! Play-testing shortcuts, compiled only with `--features cheats`.
//!
//! * `G` - grant a pile of embers (levels up)
//! * `H` - smite every lesser monster on the map
//! * `N` - smite the elder dragon
//! * `T` - fill the hero's health
//! * `B` - take the next boon in catalogue order (announced on screen)

use bevy::prelude::*;
use emberbound_core::{
    Announce, Armor, DamageRequest, Dragon, GameSet, Health, HeroState, Monster, Player, XpCollected,
};
use emberbound_domain::boons::Boon;
use emberbound_domain::combat::AttackProfile;

pub struct CheatsPlugin;

impl Plugin for CheatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (cheat, bless).in_set(GameSet::Input));
    }
}

fn cheat(
    keys: Res<ButtonInput<KeyCode>>,
    monsters: Query<(Entity, &Transform), With<Monster>>,
    dragons: Query<(Entity, &Transform), With<Dragon>>,
    mut hero: Query<&mut Health, With<Player>>,
    mut xp: MessageWriter<XpCollected>,
    mut damage: MessageWriter<DamageRequest>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        xp.write(XpCollected { amount: 40 });
    }
    let smite = AttackProfile::flat(99_999.0, 0.0);
    if keys.just_pressed(KeyCode::KeyH) {
        for (target, transform) in &monsters {
            damage.write(DamageRequest { target, attack: smite, origin: transform.translation.truncate() });
        }
    }
    if keys.just_pressed(KeyCode::KeyN) {
        for (target, transform) in &dragons {
            damage.write(DamageRequest { target, attack: smite, origin: transform.translation.truncate() });
        }
    }
    if keys.just_pressed(KeyCode::KeyT)
        && let Ok(mut health) = hero.single_mut()
    {
        health.heal(99_999.0);
    }
}

/// Walks through the boon catalogue one key press at a time.
fn bless(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: Local<usize>,
    mut hero: ResMut<HeroState>,
    mut player: Query<(&mut Health, &mut Armor), With<Player>>,
    mut announce: MessageWriter<Announce>,
) {
    if !keys.just_pressed(KeyCode::KeyB) {
        return;
    }
    let boon = Boon::ALL[*next % Boon::ALL.len()];
    *next += 1;
    hero.stats.apply(boon);
    if let Ok((mut health, mut armor)) = player.single_mut() {
        health.set_max(hero.stats.max_health);
        armor.0 = hero.stats.armor;
    }
    announce.write(Announce { title: boon.title().to_uppercase(), subtitle: boon.description().into() });
}
