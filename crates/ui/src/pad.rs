//! The on-screen controls for touch players: a thumb-stick that appears
//! under the left thumb, the sword button and the dash button. Drawn from
//! core's [`TouchPad`] and [`PadLayout`], the same layout the input adapter
//! hit-tests against, so every button is exactly where it is drawn.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use emberbound_core::{DashCooldown, GameState, InputMode, PadLayout, Player, RunPhase, RunSetup, TouchPad};

use crate::theme::{self, font};
use crate::widgets::label;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InRun), spawn_pad.in_set(RunSetup::Spawn))
        .add_systems(Update, (show_pad, place_pad).chain().run_if(in_state(GameState::InRun)));
}

/// Which part of the pad a node draws.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum PadPart {
    StickBase,
    StickKnob,
    Sword,
    Dash,
}

#[derive(Component)]
struct PadRoot;

fn spawn_pad(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::InRun),
        PadRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(theme::z::PAD),
        Pickable::IGNORE,
        Visibility::Hidden,
        children![
            circle(PadPart::StickBase, ""),
            circle(PadPart::StickKnob, ""),
            circle(PadPart::Sword, "SWORD"),
            circle(PadPart::Dash, "DASH"),
        ],
    ));
}

/// A round, translucent control, positioned every frame by [`place_pad`].
fn circle(part: PadPart, caption: &'static str) -> impl Bundle {
    (
        part,
        Node {
            position_type: PositionType::Absolute,
            border: px(3).all(),
            border_radius: BorderRadius::MAX,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        Pickable::IGNORE,
        children![label(caption, font::SMALL, theme::PARCHMENT.with_alpha(0.85))],
    )
}

/// The pad shows only to touch players and only while fighting.
fn show_pad(
    mode: Res<InputMode>,
    phase: Option<Res<State<RunPhase>>>,
    mut root: Single<&mut Visibility, With<PadRoot>>,
) {
    let fighting = phase.is_some_and(|phase| *phase.get() == RunPhase::Fighting);
    let wanted =
        if *mode == InputMode::Touch && fighting { Visibility::Inherited } else { Visibility::Hidden };
    root.set_if_neq(wanted);
}

fn place_pad(
    pad: Res<TouchPad>,
    scale: Res<UiScale>,
    window: Single<&Window, With<PrimaryWindow>>,
    dash: Query<&DashCooldown, With<Player>>,
    mut parts: Query<(&PadPart, &mut Node, &mut BackgroundColor, &mut BorderColor)>,
) {
    let layout = PadLayout::for_window(window.size());
    let dash_ready = dash.single().is_ok_and(|cooldown| cooldown.remaining <= 0.0);
    for (part, mut node, mut fill, mut border) in &mut parts {
        let (center, radius, (fill_alpha, border_alpha), color) = match part {
            PadPart::StickBase => match pad.stick {
                Some(stick) => (stick.origin, layout.stick_radius, (0.10, 0.45), theme::PARCHMENT),
                None => (layout.stick_home, layout.stick_radius, (0.04, 0.18), theme::PARCHMENT),
            },
            PadPart::StickKnob => match pad.stick {
                Some(stick) => (stick.knob, layout.stick_radius * 0.45, (0.35, 0.7), theme::PARCHMENT),
                None => (layout.stick_home, layout.stick_radius * 0.45, (0.08, 0.2), theme::PARCHMENT),
            },
            PadPart::Sword => {
                let alpha = if pad.attack_held { (0.45, 0.9) } else { (0.16, 0.5) };
                (layout.attack_center, layout.attack_radius, alpha, theme::EMBER)
            }
            PadPart::Dash => {
                let alpha = match (pad.dash_held, dash_ready) {
                    (true, _) => (0.45, 0.9),
                    (false, true) => (0.16, 0.55),
                    (false, false) => (0.05, 0.2),
                };
                (layout.dash_center, layout.dash_radius, alpha, theme::FROST)
            }
        };
        // Layout is in window pixels; UI values are scaled by `UiScale`.
        let (left, top, size) =
            ((center.x - radius) / scale.0, (center.y - radius) / scale.0, radius * 2.0 / scale.0);
        let wanted = (px(left), px(top), px(size));
        if (node.left, node.top, node.width) != wanted {
            node.left = wanted.0;
            node.top = wanted.1;
            node.width = wanted.2;
            node.height = wanted.2;
        }
        fill.set_if_neq(BackgroundColor(color.with_alpha(fill_alpha)));
        border.set_if_neq(BorderColor::all(color.with_alpha(border_alpha)));
    }
}
