//! Announcement banners: big centred titles for dragon arrivals and the
//! like. Simultaneous announcements queue up and play one after another.

use std::collections::VecDeque;

use bevy::prelude::*;
use emberbound_core::{Announce, GameState, RunPhase};

use crate::theme::{self, font};
use crate::widgets::{centered_label, text_shadow};

const FADE_IN: f32 = 0.35;
const HOLD: f32 = 2.2;
const FADE_OUT: f32 = 0.5;
/// Design pixels the banner slides down while fading in.
const SLIDE: f32 = 26.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Pending>()
        .add_systems(OnExit(GameState::InRun), clear_pending)
        .add_systems(
            Update,
            (
                collect.run_if(in_state(GameState::InRun)),
                // Banners wait while a boon is chosen or the run is paused, so
                // they never cover the cards or the pause screen.
                (show_next, animate).run_if(in_state(RunPhase::Fighting)),
            )
                .chain(),
        )
        .add_systems(OnEnter(RunPhase::ChoosingBoon), hide_banners)
        .add_systems(OnExit(RunPhase::ChoosingBoon), show_banners)
        .add_systems(OnEnter(RunPhase::Paused), hide_banners)
        .add_systems(OnExit(RunPhase::Paused), show_banners);
}

/// Announcements waiting for the current banner to finish.
#[derive(Resource, Default)]
struct Pending(VecDeque<Announce>);

/// The banner on screen and how long it has been there.
#[derive(Component)]
struct Banner {
    age: f32,
}

/// A text of the banner and its colour at full opacity.
#[derive(Component)]
struct BannerText(Color);

fn clear_pending(mut pending: ResMut<Pending>, mut announcements: MessageReader<Announce>) {
    pending.0.clear();
    announcements.clear();
}

fn collect(mut announcements: MessageReader<Announce>, mut pending: ResMut<Pending>) {
    pending.0.extend(announcements.read().cloned());
}

fn show_next(mut commands: Commands, mut pending: ResMut<Pending>, banners: Query<(), With<Banner>>) {
    if !banners.is_empty() {
        return;
    }
    let Some(announce) = pending.0.pop_front() else { return };
    commands.spawn((
        DespawnOnExit(GameState::InRun),
        Banner { age: 0.0 },
        GlobalZIndex(theme::z::BANNER),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            top: percent(17),
            left: px(0),
            width: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6),
            padding: UiRect::axes(px(24), px(14)),
            border: px(3).vertical(),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        UiTransform::IDENTITY,
        children![
            banner_text(announce.title.to_uppercase(), font::HEADLINE, theme::EMBER),
            banner_text(announce.subtitle.to_uppercase(), font::BODY, theme::PARCHMENT),
        ],
    ));
}

fn banner_text(text: String, size: f32, color: Color) -> impl Bundle {
    (centered_label(text, size, color.with_alpha(0.0)), text_shadow(size), BannerText(color))
}

fn animate(
    time: Res<Time>,
    mut commands: Commands,
    mut banners: Query<(Entity, &mut Banner, &mut UiTransform, &mut BackgroundColor, &mut BorderColor)>,
    mut texts: Query<(&BannerText, &mut TextColor, &mut TextShadow)>,
) {
    let Ok((entity, mut banner, mut transform, mut background, mut border)) = banners.single_mut() else {
        return;
    };
    banner.age += time.delta_secs();
    if banner.age >= FADE_IN + HOLD + FADE_OUT {
        commands.entity(entity).despawn();
        return;
    }

    let fade_in = (banner.age / FADE_IN).clamp(0.0, 1.0);
    let fade_out = ((FADE_IN + HOLD + FADE_OUT - banner.age) / FADE_OUT).clamp(0.0, 1.0);
    let alpha = fade_in.min(fade_out);
    // Ease out: fast at first, settling into place.
    let settle = 1.0 - (1.0 - fade_in).powi(3);
    transform.translation.y = px(-SLIDE * (1.0 - settle));
    background.0 = theme::NIGHT.with_alpha(0.6 * alpha);
    border.set_all(theme::EMBER.with_alpha(0.7 * alpha));
    for (base, mut color, mut shadow) in &mut texts {
        color.0 = base.0.with_alpha(alpha);
        shadow.color.set_alpha(0.8 * alpha);
    }
}

fn hide_banners(mut banners: Query<&mut Visibility, With<Banner>>) {
    for mut visibility in &mut banners {
        *visibility = Visibility::Hidden;
    }
}

fn show_banners(mut banners: Query<&mut Visibility, With<Banner>>) {
    for mut visibility in &mut banners {
        *visibility = Visibility::Inherited;
    }
}
