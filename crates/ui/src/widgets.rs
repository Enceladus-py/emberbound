//! Small reusable UI building blocks: labels, panels, bars and a few
//! generic animations (smooth bars, pulsing text, animated sprite images).

use bevy::prelude::*;
use emberbound_core::{InputMode, SpriteId, SpriteLibrary};
use emberbound_pixelart::EM_PIXELS;

use crate::theme;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (animate_bars, pulse_text, animate_sprite_images, speak_to_device));
}

/// A line of text in the pixel font.
pub fn label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (Text::new(text), theme::text_font(size), TextColor(color), Pickable::IGNORE)
}

/// A [`label`] with a hard drop shadow, for text drawn over the world.
pub fn shadowed_label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (label(text, size, color), text_shadow(size))
}

/// A chunky drop shadow one font pixel down and right.
pub fn text_shadow(size: f32) -> TextShadow {
    TextShadow { offset: Vec2::splat(size / EM_PIXELS as f32), color: Color::BLACK.with_alpha(0.8) }
}

/// A [`label`] whose lines are centred and may wrap.
pub fn centered_label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (label(text, size, color), TextLayout::justify(Justify::Center))
}

/// Opaque full-screen column that centres its children: the base of the
/// title and end screens.
pub fn screen_root() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(22),
            padding: px(24).all(),
            ..default()
        },
        BackgroundColor(theme::NIGHT),
    )
}

/// Chunky bordered box. `node` supplies the layout; border width and
/// colours come from the theme.
pub fn panel(node: Node) -> impl Bundle {
    (
        Node { border: px(3).all(), ..node },
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::PANEL_BORDER),
    )
}

/// A text that reads differently for keyboard and touch players, e.g.
/// "PRESS ENTER" versus "TAP TO START". Spawn it with a [`Text`]; the
/// wording follows [`InputMode`].
#[derive(Component, Debug, Clone, Copy)]
pub struct DeviceText {
    pub keyboard: &'static str,
    pub touch: &'static str,
}

/// A [`label`] whose wording follows the player's device.
pub fn device_label(keyboard: &'static str, touch: &'static str, size: f32, color: Color) -> impl Bundle {
    (label(keyboard, size, color), DeviceText { keyboard, touch })
}

fn speak_to_device(mode: Res<InputMode>, mut texts: Query<(Ref<DeviceText>, &mut Text)>) {
    for (words, mut text) in &mut texts {
        if mode.is_changed() || words.is_added() {
            let wording = match *mode {
                InputMode::Keyboard => words.keyboard,
                InputMode::Touch => words.touch,
            };
            set_text(&mut text, wording.to_owned());
        }
    }
}

/// Changes a text only when it differs, sparing the text pipeline a
/// re-layout on every frame.
pub fn set_text(text: &mut Mut<Text>, value: String) {
    if text.0 != value {
        text.0 = value;
    }
}

/// The filled part of a [`bar`]. Set `target`; the shown width eases to it.
#[derive(Component, Debug, Default)]
pub struct Bar {
    /// Fill to show, `0..=1`.
    pub target: f32,
    shown: f32,
}

/// How quickly bars catch up with their target (higher = snappier).
const BAR_EASE_RATE: f32 = 10.0;

/// A bordered bar whose fill is found through `marker` (query
/// `(&mut Bar, &mut BackgroundColor)` with it).
pub fn bar(marker: impl Component, width: Val, height: Val, color: Color) -> impl Bundle {
    (
        Node { width, height, border: px(2).all(), ..default() },
        BackgroundColor(theme::NIGHT.with_alpha(0.85)),
        BorderColor::all(theme::PANEL_BORDER),
        children![(
            marker,
            Bar::default(),
            Node { width: percent(0), height: percent(100), ..default() },
            BackgroundColor(color),
        )],
    )
}

fn animate_bars(time: Res<Time>, mut bars: Query<(&mut Bar, &mut Node)>) {
    let ease = 1.0 - (-BAR_EASE_RATE * time.delta_secs()).exp();
    for (mut bar, mut node) in &mut bars {
        let target = bar.target.clamp(0.0, 1.0);
        if (bar.shown - target).abs() < 0.0005 {
            continue;
        }
        bar.shown += (target - bar.shown) * ease;
        if (bar.shown - target).abs() < 0.002 {
            bar.shown = target;
        }
        node.width = percent(bar.shown * 100.0);
    }
}

/// Makes a text breathe between dim and full brightness.
#[derive(Component, Debug)]
pub struct Pulse;

fn pulse_text(time: Res<Time>, mut texts: Query<&mut TextColor, With<Pulse>>) {
    let alpha = 0.35 + 0.65 * (0.5 + 0.5 * (time.elapsed_secs() * 3.2).sin());
    for mut color in &mut texts {
        color.0.set_alpha(alpha);
    }
}

/// Loops the frames of a [`sprite_image`].
#[derive(Component, Debug)]
pub struct SpriteImageAnim {
    frames: usize,
    timer: Timer,
}

/// A game sprite as a UI image, magnified `scale` times and animated at
/// `fps` (the app's nearest-neighbour sampling keeps the pixels crisp).
pub fn sprite_image(library: &SpriteLibrary, id: SpriteId, scale: f32, fps: f32) -> impl Bundle {
    let size = library.size(id) * scale;
    (
        ImageNode::from_atlas_image(library.image(id), TextureAtlas { layout: library.layout(id), index: 0 }),
        Node { width: px(size.x), height: px(size.y), flex_shrink: 0.0, ..default() },
        SpriteImageAnim {
            frames: library.frames(id),
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
        },
    )
}

fn animate_sprite_images(time: Res<Time>, mut images: Query<(&mut ImageNode, &mut SpriteImageAnim)>) {
    for (mut image, mut anim) in &mut images {
        if anim.timer.tick(time.delta()).just_finished()
            && let Some(atlas) = image.texture_atlas.as_mut()
        {
            atlas.index = (atlas.index + 1) % anim.frames;
        }
    }
}

/// `mm:ss` for a duration in seconds.
pub fn clock(seconds: f32) -> String {
    let total = seconds.max(0.0) as u32;
    format!("{:02}:{:02}", total / 60, total % 60)
}
