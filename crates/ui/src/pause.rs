//! Pause overlay: shown while [`RunPhase::Paused`]. Resuming is the app's
//! job; this screen only says how.

use bevy::prelude::*;
use emberbound_core::{GameState, RunPhase};

use crate::theme::{self, font};
use crate::widgets::{Pulse, device_label, label, text_shadow};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(RunPhase::Paused), spawn_overlay);
}

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        // The phase exit removes it; the run-scoped tag is a safety net.
        DespawnOnExit(RunPhase::Paused),
        DespawnOnExit(GameState::InRun),
        GlobalZIndex(theme::z::BOONS),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(18),
            ..default()
        },
        BackgroundColor(theme::NIGHT.with_alpha(0.72)),
        children![
            (
                label("PAUSED", font::HEADLINE, theme::EMBER),
                TextShadow { color: theme::FLAME, ..text_shadow(font::HEADLINE) }
            ),
            (
                device_label("PRESS ESC OR ENTER TO RESUME", "TAP TO RESUME", font::TITLE, theme::PARCHMENT),
                Pulse
            ),
        ],
    ));
}
