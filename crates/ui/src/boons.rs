//! Boon choice overlay: shown while [`RunPhase::ChoosingBoon`]. Pick with
//! the number keys (via [`PlayerIntent`]) or by clicking a card.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use emberbound_core::{BoonChosen, BoonOffer, GameState, PlayerIntent, RunPhase};
use emberbound_domain::progression::Boon;

use crate::theme::{self, font};
use crate::widgets::{centered_label, label, panel, text_shadow};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(RunPhase::ChoosingBoon), spawn_overlay)
        .add_systems(
            Update,
            (choose_by_key, choose_by_tap, rebuild_cards).run_if(in_state(RunPhase::ChoosingBoon)),
        )
        .add_observer(choose_by_click)
        .add_observer(highlight_card)
        .add_observer(unhighlight_card);
}

/// The row that holds the cards; its children are rebuilt per offer.
#[derive(Component)]
struct CardRow;

/// A clickable card offering one boon.
#[derive(Component)]
struct BoonCard(Boon);

fn spawn_overlay(mut commands: Commands, offer: Res<BoonOffer>) {
    commands.spawn((
        // The phase exit removes it; the run-scoped tag is a safety net.
        DespawnOnExit(RunPhase::ChoosingBoon),
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
            padding: px(24).all(),
            ..default()
        },
        BackgroundColor(theme::NIGHT.with_alpha(0.78)),
        children![
            (
                label("LEVEL UP", font::HEADLINE, theme::XP),
                TextShadow { color: theme::FLAME, ..text_shadow(font::HEADLINE) }
            ),
            label("CHOOSE A BOON", font::BODY, theme::MUTED),
            (
                CardRow,
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: px(22),
                    margin: px(10).top(),
                    ..default()
                },
                cards(&offer),
            ),
        ],
    ));
}

fn cards(offer: &BoonOffer) -> impl Bundle + use<> {
    let boons = offer.0.clone();
    Children::spawn(SpawnIter(boons.into_iter().enumerate().map(|(index, boon)| card(index, boon))))
}

fn card(index: usize, boon: Boon) -> impl Bundle {
    (
        BoonCard(boon),
        panel(Node {
            width: px(250),
            min_height: px(210),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(14),
            padding: UiRect::axes(px(16), px(20)),
            ..default()
        }),
        children![
            label(format!("[{}]", index + 1), font::TITLE, theme::EMBER),
            centered_label(boon.title().to_uppercase(), font::BODY * 1.15, theme::PARCHMENT),
            centered_label(boon.description().to_uppercase(), font::SMALL, theme::MUTED),
        ],
    )
}

/// The app may refill the offer without leaving the phase (several levels
/// gained at once): swap the cards for the new offer.
fn rebuild_cards(mut commands: Commands, offer: Res<BoonOffer>, row: Single<Entity, With<CardRow>>) {
    if offer.is_changed() {
        commands.entity(*row).despawn_related::<Children>().insert(cards(&offer));
    }
}

fn choose_by_key(intent: Res<PlayerIntent>, offer: Res<BoonOffer>, mut chosen: MessageWriter<BoonChosen>) {
    if let Some(boon) = intent.choice.and_then(|index| offer.0.get(index)) {
        chosen.write(BoonChosen(*boon));
    }
}

/// Fingers choose through [`PlayerIntent::tap`]: picking can miss a quick
/// tap from a brand-new touch pointer, so the card under the tap is found
/// here instead.
fn choose_by_tap(
    intent: Res<PlayerIntent>,
    window: Single<&Window, With<PrimaryWindow>>,
    cards: Query<(&BoonCard, &ComputedNode, &UiGlobalTransform)>,
    mut chosen: MessageWriter<BoonChosen>,
) {
    let Some(tap) = intent.tap else { return };
    // Layout is in physical pixels; taps are logical.
    let point = tap * window.scale_factor();
    if let Some((card, ..)) =
        cards.iter().find(|(_, node, transform)| node.contains_point(**transform, point))
    {
        chosen.write(BoonChosen(card.0));
    }
}

/// Mouse clicks; touches are handled by [`choose_by_tap`].
fn choose_by_click(click: On<PointerClick>, cards: Query<&BoonCard>, mut chosen: MessageWriter<BoonChosen>) {
    if click.pointer.id.is_touch() {
        return;
    }
    if let Ok(card) = cards.get(click.event_target()) {
        chosen.write(BoonChosen(card.0));
    }
}

fn highlight_card(over: On<PointerOver>, mut cards: Query<&mut BorderColor, With<BoonCard>>) {
    if let Ok(mut border) = cards.get_mut(over.event_target()) {
        border.set_all(theme::EMBER);
    }
}

fn unhighlight_card(out: On<PointerOut>, mut cards: Query<&mut BorderColor, With<BoonCard>>) {
    if let Ok(mut border) = cards.get_mut(out.event_target()) {
        border.set_all(theme::PANEL_BORDER);
    }
}
