//! Game-over and victory screens with the summary of the finished run.
//! The world is already despawned, so both are opaque.

use bevy::prelude::*;
use emberbound_core::{GameState, HeroState, RunStats};
use emberbound_domain::bestiary::DragonKind;

use crate::boon_strip::{BoonStrip, ICON_SIZE};
use crate::theme::{self, font};
use crate::widgets::{Pulse, centered_label, clock, device_label, label, panel, screen_root, text_shadow};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GameOver), spawn_game_over)
        .add_systems(OnEnter(GameState::Victory), spawn_victory);
}

fn spawn_game_over(mut commands: Commands, stats: Res<RunStats>, hero: Res<HeroState>) {
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        end_screen(
            "THE EMBER FADES",
            "THE WYRMWARDEN HAS FALLEN AND THE DRAGONS RULE THE VALLEY",
            theme::HEALTH,
            &stats,
            &hero,
        ),
    ));
}

fn spawn_victory(mut commands: Commands, stats: Res<RunStats>, hero: Res<HeroState>) {
    commands.spawn((
        DespawnOnExit(GameState::Victory),
        end_screen(
            "WYRMWARDEN",
            "THE ELDER DRAGONS ARE SLAIN AND THE VALLEY IS FREE",
            theme::XP,
            &stats,
            &hero,
        ),
    ));
}

fn end_screen(
    headline: &str,
    verdict: &str,
    accent: Color,
    stats: &RunStats,
    hero: &HeroState,
) -> impl Bundle {
    (
        screen_root(),
        children![
            (
                centered_label(headline, font::HEADLINE * 1.25, accent),
                TextShadow { color: accent.darker(0.35), ..text_shadow(font::HEADLINE) },
            ),
            centered_label(verdict, font::BODY, theme::MUTED),
            summary(stats, hero),
            taken_boons(),
            (
                device_label("PRESS ENTER TO PLAY AGAIN", "TAP TO PLAY AGAIN", font::TITLE, theme::PARCHMENT),
                Pulse
            ),
        ],
    )
}

/// The boons of the finished run, filled in by the strip.
fn taken_boons() -> impl Bundle {
    (
        BoonStrip,
        Node {
            max_width: px(12.0 * (ICON_SIZE + 6.0)),
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            row_gap: px(6),
            ..default()
        },
    )
}

fn summary(stats: &RunStats, hero: &HeroState) -> impl Bundle {
    let rows = [
        ("TIME", clock(stats.elapsed)),
        ("LEVEL REACHED", hero.progress.level().to_string()),
        ("MONSTERS SLAIN", stats.kills.to_string()),
        ("DRAGONS SLAIN", format!("{}/{}", stats.dragons_slain, DragonKind::ALL.len())),
    ];
    (
        panel(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            padding: UiRect::axes(px(36), px(22)),
            margin: px(10).vertical(),
            min_width: px(420),
            ..default()
        }),
        Children::spawn(SpawnIter(rows.into_iter().map(|(name, value)| {
            (
                Node { justify_content: JustifyContent::SpaceBetween, column_gap: px(48), ..default() },
                children![label(name, font::BODY, theme::MUTED), label(value, font::BODY, theme::PARCHMENT)],
            )
        }))),
    )
}
