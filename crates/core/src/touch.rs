//! Contract for playing with fingers: which kind of device the player is
//! using, and where the on-screen controls are and what they are doing.
//!
//! The app's input adapter writes these; the UI draws from them. Both use
//! [`PadLayout`] so a button is exactly where it is drawn. All positions are
//! in logical window pixels with the origin at the top-left, like
//! `Touch::position`.

use bevy::prelude::*;

/// The kind of device the player is using right now. The UI adapts its
/// prompts and shows the on-screen controls only in [`InputMode::Touch`].
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum InputMode {
    #[default]
    Keyboard,
    Touch,
}

/// The virtual thumb-stick while a finger is on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stick {
    /// Where the stick is centred (where the finger first landed).
    pub origin: Vec2,
    /// Where the knob is drawn: the finger, kept within the stick radius.
    pub knob: Vec2,
}

/// What the on-screen controls are doing this frame.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct TouchPad {
    pub stick: Option<Stick>,
    pub attack_held: bool,
    pub dash_held: bool,
}

/// Where the on-screen controls sit for a given window size. The stick
/// floats: it appears wherever a thumb lands on the left part of the
/// screen, so [`PadLayout::stick_home`] is only where its hint is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PadLayout {
    pub stick_radius: f32,
    pub stick_home: Vec2,
    pub attack_center: Vec2,
    pub attack_radius: f32,
    pub dash_center: Vec2,
    pub dash_radius: f32,
    /// Touches starting left of this x drive the stick; the rest of the
    /// screen is the sword, except the dash button.
    pub stick_zone_right: f32,
}

impl PadLayout {
    pub fn for_window(size: Vec2) -> Self {
        let short = size.x.min(size.y);
        let attack_radius = (short * 0.12).clamp(34.0, 64.0);
        let stick_radius = (short * 0.13).clamp(40.0, 72.0);
        let margin = attack_radius * 0.6;
        let attack_center = size - Vec2::splat(margin + attack_radius);
        Self {
            stick_radius,
            stick_home: Vec2::new(margin + stick_radius * 1.4, size.y - margin - stick_radius * 1.4),
            attack_center,
            attack_radius,
            dash_center: attack_center + Vec2::new(-1.8, -1.2) * attack_radius,
            dash_radius: attack_radius * 0.72,
            stick_zone_right: size.x * 0.45,
        }
    }

    /// Whether a touch landing at `position` presses the dash button. The
    /// target is a little larger than the drawn button, for thumbs.
    pub fn hits_dash(&self, position: Vec2) -> bool {
        position.distance(self.dash_center) <= self.dash_radius * 1.3
    }
}

pub(crate) fn register(app: &mut App) {
    app.init_resource::<InputMode>().init_resource::<TouchPad>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_fit_a_landscape_phone_without_overlapping() {
        for size in [Vec2::new(844.0, 390.0), Vec2::new(667.0, 375.0), Vec2::new(1920.0, 1080.0)] {
            let pad = PadLayout::for_window(size);
            let inside =
                |c: Vec2, r: f32| c.x - r >= 0.0 && c.y - r >= 0.0 && c.x + r <= size.x && c.y + r <= size.y;
            assert!(inside(pad.attack_center, pad.attack_radius), "{size}");
            assert!(inside(pad.dash_center, pad.dash_radius), "{size}");
            assert!(inside(pad.stick_home, pad.stick_radius), "{size}");
            assert!(pad.attack_center.distance(pad.dash_center) > pad.attack_radius + pad.dash_radius);
            assert!(pad.stick_home.x + pad.stick_radius < pad.stick_zone_right);
            assert!(pad.dash_center.x - pad.dash_radius * 1.3 > pad.stick_zone_right);
        }
    }
}
