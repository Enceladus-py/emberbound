//! Title screen: logotype, the cast, the controls and "press enter".
//! Starting the run is the app's job; this screen only shows itself.

use bevy::prelude::*;
use emberbound_core::{GameState, SpriteId, SpriteLibrary};

use crate::theme::{self, font};
use crate::widgets::{
    Pulse, centered_label, device_label, label, panel, screen_root, sprite_image, text_shadow,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), spawn_title);
}

/// Action, keyboard binding, touch binding.
const CONTROLS: [(&str, &str, &str); 4] = [
    ("MOVE", "WASD / ARROWS", "LEFT THUMB"),
    ("SWORD", "SPACE / J / CLICK", "HOLD + SLIDE RIGHT"),
    ("DASH", "SHIFT / K", "DASH BUTTON"),
    ("BOONS", "1  2  3", "TAP A CARD"),
];

fn spawn_title(mut commands: Commands, library: Res<SpriteLibrary>) {
    commands.spawn((
        DespawnOnExit(GameState::Title),
        screen_root(),
        children![
            (
                label("EMBERBOUND", font::LOGO, theme::EMBER),
                TextShadow { color: theme::FLAME, ..text_shadow(font::LOGO) }
            ),
            centered_label(
                "THE LAST WYRMWARDEN AND HER HATCHLING AGAINST THREE ELDER DRAGONS",
                font::BODY,
                theme::MUTED,
            ),
            cast(&library),
            controls(),
            (device_label("PRESS ENTER", "TAP TO START", font::TITLE, theme::PARCHMENT), Pulse),
        ],
    ));
}

/// The hero and her hatchling facing the three elder dragons.
fn cast(library: &SpriteLibrary) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            column_gap: px(18),
            margin: px(8).vertical(),
            ..default()
        },
        children![
            sprite_image(library, SpriteId::HeroIdle, 5.0, 5.0),
            sprite_image(library, SpriteId::CompanionHatchling, 5.0, 6.0),
            (Node { width: px(56), ..default() },),
            dragon(library, SpriteId::DragonMoss),
            dragon(library, SpriteId::DragonFrost),
            dragon(library, SpriteId::DragonAshen),
        ],
    )
}

/// Dragons are drawn facing right; mirror them to face the hero.
fn dragon(library: &SpriteLibrary, id: SpriteId) -> impl Bundle {
    (sprite_image(library, id, 3.0, 6.0), UiTransform::from_scale(Vec2::new(-1.0, 1.0)))
}

fn controls() -> impl Bundle {
    (
        panel(Node { column_gap: px(34), padding: UiRect::axes(px(28), px(14)), ..default() }),
        Children::spawn(SpawnIter(CONTROLS.into_iter().map(|(action, keys, touch)| {
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(4),
                    ..default()
                },
                children![
                    label(action, font::SMALL, theme::EMBER),
                    device_label(keys, touch, font::BODY, theme::PARCHMENT)
                ],
            )
        }))),
    )
}
