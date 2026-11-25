//! The on-screen controls for touch players: a thumb-stick that appears
//! under the left thumb, a sword stick that appears under the right finger
//! (hold to swing, slide past its inner ring to aim) and the dash button. Drawn from
//! core's [`TouchPad`] and [`PadLayout`], the same layout the input adapter
//! hit-tests against, so every button is exactly where it is drawn.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use emberbound_core::{
    AIM_DEAD_ZONE, DashCooldown, GameState, InputMode, PadLayout, Player, RunPhase, RunSetup, TouchPad,
};

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
    /// The sword: a resting button, or the base of the sword stick while a
    /// finger is down.
    Sword,
    /// The sword stick's dead zone: inside it the sword swings unaimed.
    AimRing,
    AimKnob,
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
            circle(PadPart::AimRing, ""),
            circle(PadPart::AimKnob, ""),
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
            PadPart::Sword => match pad.aim {
                Some(aim) => (aim.origin, layout.stick_radius, (0.14, 0.6), theme::EMBER),
                None => {
                    let alpha = if pad.attack_held { (0.45, 0.9) } else { (0.16, 0.5) };
                    (layout.attack_center, layout.attack_radius, alpha, theme::EMBER)
                }
            },
            PadPart::AimRing => match pad.aim {
                Some(aim) => (aim.origin, layout.stick_radius * AIM_DEAD_ZONE, (0.0, 0.35), theme::EMBER),
                None => (layout.attack_center, 0.0, (0.0, 0.0), theme::EMBER),
            },
            PadPart::AimKnob => match pad.aim {
                Some(aim) => {
                    // Brighter once it aims, i.e. past the dead zone ring.
                    let aiming = aim.knob.distance(aim.origin) > layout.stick_radius * AIM_DEAD_ZONE;
                    let alpha = if aiming { (0.5, 0.95) } else { (0.25, 0.6) };
                    (aim.knob, layout.stick_radius * 0.45, alpha, theme::EMBER)
                }
                None => (layout.attack_center, 0.0, (0.0, 0.0), theme::EMBER),
            },
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

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use bevy::window::{PrimaryWindow, WindowResolution};
    use emberbound_core::{CorePlugin, Stick};

    use super::*;

    const WINDOW: Vec2 = Vec2::new(844.0, 390.0);

    fn app(pad: TouchPad) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins(CorePlugin)
            .insert_resource(UiScale(1.0))
            .insert_resource(pad)
            .add_systems(Update, place_pad);
        app.world_mut().spawn((
            Window { resolution: WindowResolution::new(WINDOW.x as u32, WINDOW.y as u32), ..default() },
            PrimaryWindow,
        ));
        for part in [PadPart::Sword, PadPart::AimRing, PadPart::AimKnob] {
            app.world_mut().spawn((
                part,
                Node::default(),
                BackgroundColor::default(),
                BorderColor::default(),
            ));
        }
        app.update();
        app
    }

    /// Centre and radius a part was placed at, in window pixels.
    fn placed(app: &mut App, wanted: PadPart) -> (Vec2, f32) {
        let mut parts = app.world_mut().query::<(&PadPart, &Node)>();
        let (_, node) = parts.iter(app.world()).find(|(part, _)| **part == wanted).unwrap();
        let value = |val: Val| match val {
            Val::Px(px) => px,
            other => panic!("{other:?}"),
        };
        let radius = value(node.width) / 2.0;
        (Vec2::new(value(node.left), value(node.top)) + radius, radius)
    }

    #[test]
    fn the_sword_stick_is_drawn_where_the_finger_is() {
        let layout = PadLayout::for_window(WINDOW);
        let origin = Vec2::new(650.0, 250.0);
        let knob = origin + Vec2::new(30.0, -10.0);
        let mut app = app(TouchPad { aim: Some(Stick { origin, knob }), attack_held: true, ..default() });

        let (centre, radius) = placed(&mut app, PadPart::Sword);
        assert!(centre.distance(origin) < 0.01 && (radius - layout.stick_radius).abs() < 0.01);
        let (centre, radius) = placed(&mut app, PadPart::AimRing);
        assert!(centre.distance(origin) < 0.01);
        assert!((radius - layout.stick_radius * AIM_DEAD_ZONE).abs() < 0.01);
        assert!(placed(&mut app, PadPart::AimKnob).0.distance(knob) < 0.01);
    }

    #[test]
    fn at_rest_the_sword_is_its_button_and_the_stick_parts_vanish() {
        let layout = PadLayout::for_window(WINDOW);
        let mut app = app(TouchPad::default());
        let (centre, radius) = placed(&mut app, PadPart::Sword);
        assert!(centre.distance(layout.attack_center) < 0.01);
        assert!((radius - layout.attack_radius).abs() < 0.01);
        assert_eq!(placed(&mut app, PadPart::AimKnob).1, 0.0);
        assert_eq!(placed(&mut app, PadPart::AimRing).1, 0.0);
    }
}
