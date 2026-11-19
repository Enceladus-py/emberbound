//! Fingers: a floating thumb-stick on the left, the sword on the right
//! with a dash button, and taps to get past the title and end screens.

use bevy::input::touch::Touch;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use emberbound_core::{GameState, PadLayout, RunPhase, Stick, TouchPad};

use super::TouchIntent;

/// Stick deflection (as a fraction of its radius) that is still ignored.
const DEAD_ZONE: f32 = 0.18;
/// A touch that slides further than this (logical pixels) is not a tap.
const TAP_SLOP: f32 = 24.0;

/// The screen a touch began on. Taps only count where they began, so
/// thumbs still down from the fight cannot pick a boon or restart a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Screen(GameState, Option<RunPhase>);

impl Screen {
    fn is_menu(self) -> bool {
        self.1 != Some(RunPhase::Fighting)
    }
}

/// Which finger is doing what.
#[derive(Resource, Default)]
pub(super) struct Fingers {
    /// The stick finger and where the stick is centred.
    stick: Option<(u64, Vec2)>,
    sword: Vec<u64>,
    dash: Option<u64>,
    /// Touches that began on a menu, and on which one.
    taps: Vec<(u64, Screen)>,
    pub(super) intent: TouchIntent,
}

pub(super) fn read_touches(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    state: Res<State<GameState>>,
    phase: Option<Res<State<RunPhase>>>,
    mut fingers: ResMut<Fingers>,
    mut pad: ResMut<TouchPad>,
) {
    let layout = PadLayout::for_window(window.size());
    let screen = Screen(*state.get(), phase.map(|phase| *phase.get()));
    let mut intent = TouchIntent::default();

    for touch in touches.iter_just_pressed() {
        let (id, position) = (touch.id(), touch.position());
        if screen.is_menu() {
            fingers.taps.push((id, screen));
        }
        if position.x < layout.stick_zone_right {
            if fingers.stick.is_none() {
                fingers.stick = Some((id, position));
            }
        } else if layout.hits_dash(position) {
            fingers.dash = Some(id);
            intent.dash = true;
        } else {
            fingers.sword.push(id);
        }
    }

    let lifted = touches.iter_just_released().map(|touch| (touch, true));
    let canceled = touches.iter_just_canceled().map(|touch| (touch, false));
    for (touch, lifted) in lifted.chain(canceled) {
        let id = touch.id();
        let began_here = fingers.taps.iter().any(|&(tap, on)| tap == id && on == screen);
        if lifted && began_here && touch.start_position().distance(touch.position()) <= TAP_SLOP {
            intent.tap = Some(touch.position());
        }
        fingers.taps.retain(|&(tap, _)| tap != id);
        fingers.sword.retain(|&sword| sword != id);
        if fingers.dash == Some(id) {
            fingers.dash = None;
        }
        if fingers.stick.is_some_and(|(stick, _)| stick == id) {
            fingers.stick = None;
        }
    }

    let stick = fingers.stick.and_then(|(id, origin)| {
        let touch = touches.get_pressed(id)?;
        Some(steer(touch, origin, layout.stick_radius))
    });
    if let Some((origin, _, movement)) = stick {
        // A finger that slides past the rim drags the stick along with it,
        // so reversing direction is instant.
        fingers.stick = fingers.stick.map(|(id, _)| (id, origin));
        intent.movement = movement;
    }

    intent.confirm = intent.tap.is_some() && screen.0 != GameState::InRun;
    intent.attack = !fingers.sword.is_empty();
    fingers.intent = intent;
    pad.set_if_neq(TouchPad {
        stick: stick.map(|(origin, knob, _)| Stick { origin, knob }),
        attack_held: intent.attack,
        dash_held: fingers.dash.is_some(),
    });
}

/// Stick origin (possibly dragged), knob position and the resulting world
/// movement for a finger. Screen y grows downwards, world y upwards.
fn steer(touch: &Touch, origin: Vec2, radius: f32) -> (Vec2, Vec2, Vec2) {
    let finger = touch.position();
    let offset = finger - origin;
    let origin = if offset.length() > radius { finger - offset.normalize() * radius } else { origin };
    let deflection = ((finger - origin) / radius).clamp_length_max(1.0);
    let movement =
        if deflection.length() < DEAD_ZONE { Vec2::ZERO } else { Vec2::new(deflection.x, -deflection.y) };
    (origin, finger, movement)
}
