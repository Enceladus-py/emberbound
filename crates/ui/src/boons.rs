//! Boon choice overlay: shown while [`RunPhase::ChoosingBoon`]. Pick with
//! the number keys (via [`PlayerIntent`]), a tap or a click.
//!
//! Each card wears its rarity: a coloured frame and ribbon, a medallion
//! with the boon's pixel-art icon, its aspect, any curse in flame red and
//! how many times it has been taken. Cards rise in one after another, lift
//! under the pointer, and legendary frames smoulder.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use emberbound_core::{
    BoonChosen, BoonGlyph, BoonOffer, GameState, HeroState, PlayerIntent, RunPhase, SpriteId, SpriteLibrary,
};
use emberbound_domain::boons::{Boon, Rarity};

use crate::theme::{self, font};
use crate::widgets::{DeviceText, centered_label, label, text_shadow};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(RunPhase::ChoosingBoon), spawn_overlay)
        .add_systems(
            Update,
            (choose_by_key, choose_by_tap, rebuild_cards, animate_cards)
                .run_if(in_state(RunPhase::ChoosingBoon)),
        )
        .add_observer(choose_by_click)
        .add_observer(hover_card)
        .add_observer(leave_card);
}

const CARD_WIDTH: f32 = 236.0;
const CARD_PADDING: f32 = 14.0;
const ICON_SCALE: f32 = 3.0;
/// Seconds between the entrances of neighbouring cards, and of one entrance.
const STAGGER: f32 = 0.09;
const ENTRANCE: f32 = 0.32;
/// How far a card rises from as it enters, and lifts when hovered.
const RISE: f32 = 46.0;
const HOVER_LIFT: f32 = 8.0;
/// Keyboard hints shown on each card; touch players tap instead.
const KEY_HINTS: [&str; 3] = ["[1]", "[2]", "[3]"];

/// The row that holds the cards; its children are rebuilt per offer.
#[derive(Component)]
struct CardRow;

/// A card offering one boon, and its animation state.
#[derive(Component)]
struct BoonCard {
    boon: Boon,
    /// Seconds since the card started entering (negative = still waiting).
    age: f32,
    hovered: bool,
    /// Eased hover amount, `0..=1`.
    lift: f32,
}

pub(crate) fn rarity_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => theme::MUTED,
        Rarity::Rare => theme::FROST,
        Rarity::Legendary => theme::XP,
    }
}

/// The picture on each boon's card.
pub(crate) fn glyph(boon: Boon) -> BoonGlyph {
    match boon {
        Boon::WhettedEdge => BoonGlyph::Whetstone,
        Boon::WardensVigor => BoonGlyph::ShieldHeart,
        Boon::Fleetfoot => BoonGlyph::WingedBoot,
        Boon::KeenEye => BoonGlyph::Eye,
        Boon::EmberHeart => BoonGlyph::FlameHeart,
        Boon::Wingbeat => BoonGlyph::Wing,
        Boon::Lodestone => BoonGlyph::Magnet,
        Boon::Scalemail => BoonGlyph::Scales,
        Boon::SweepingArc => BoonGlyph::Crescent,
        Boon::Kindling => BoonGlyph::Campfire,
        Boon::Fervor => BoonGlyph::QuickSword,
        Boon::Stokedflame => BoonGlyph::Bellows,
        Boon::EmberWake => BoonGlyph::FlameSteps,
        Boon::Thornhide => BoonGlyph::Thorns,
        Boon::SanguineEdge => BoonGlyph::BloodBlade,
        Boon::CrescentWave => BoonGlyph::Wave,
        Boon::TwinFlame => BoonGlyph::TwinFlames,
        Boon::LastStand => BoonGlyph::CrackedShield,
        Boon::MoltenHeart => BoonGlyph::MoltenHeart,
        Boon::PhoenixFeather => BoonGlyph::Feather,
        Boon::EmberTithe => BoonGlyph::EmberPile,
        Boon::Wildfire => BoonGlyph::BurningSkull,
        Boon::Hearthkeeper => BoonGlyph::Hearth,
    }
}

fn spawn_overlay(
    mut commands: Commands,
    offer: Res<BoonOffer>,
    hero: Res<HeroState>,
    library: Res<SpriteLibrary>,
) {
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
            row_gap: px(10),
            padding: px(20).all(),
            ..default()
        },
        BackgroundColor(theme::NIGHT.with_alpha(0.82)),
        children![
            (
                label("LEVEL UP", font::HEADLINE, theme::XP),
                TextShadow { color: theme::FLAME, ..text_shadow(font::HEADLINE) }
            ),
            label(format!("LEVEL {}  -  CHOOSE A BOON", hero.progress.level()), font::BODY, theme::MUTED),
            (
                CardRow,
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: px(20),
                    margin: px(12).top(),
                    ..default()
                },
                cards(&offer, &hero, &library),
            ),
        ],
    ));
}

fn cards(offer: &BoonOffer, hero: &HeroState, library: &SpriteLibrary) -> impl Bundle + use<> {
    let built: Vec<_> =
        offer.0.iter().enumerate().map(|(index, boon)| card(index, *boon, hero, library)).collect();
    Children::spawn(SpawnIter(built.into_iter()))
}

fn card(index: usize, boon: Boon, hero: &HeroState, library: &SpriteLibrary) -> impl Bundle + use<> {
    let rarity = boon.rarity();
    let color = rarity_color(rarity);
    let border = if rarity == Rarity::Legendary { 4.0 } else { 3.0 };
    let owned = hero.stats.stacks(boon);
    let stacks = match (boon.max_stacks(), owned) {
        (Some(1), _) => "ONCE PER RUN".to_owned(),
        (Some(max), 0) => format!("NEW - UP TO {max}"),
        (Some(max), _) => format!("OWNED {owned}/{max}"),
        (None, 0) => "NEW".to_owned(),
        (None, _) => format!("OWNED {owned}"),
    };
    (
        BoonCard { boon, age: -(index as f32) * STAGGER, hovered: false, lift: 0.0 },
        Node {
            width: px(CARD_WIDTH),
            min_height: px(300),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(8),
            padding: UiRect::new(px(CARD_PADDING), px(CARD_PADDING), px(0), px(12)),
            border: px(border).all(),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        BorderColor::all(color),
        UiTransform::from_translation(Val2::px(0.0, RISE)),
        // Hidden until its turn to enter comes.
        Visibility::Hidden,
        Children::spawn((
            Spawn(ribbon(rarity, color)),
            Spawn(medallion(boon, color, library)),
            Spawn(centered_label(boon.title().to_uppercase(), font::BODY * 1.1, theme::PARCHMENT)),
            Spawn(aspect_tag(boon)),
            Spawn(centered_label(
                boon.description().to_uppercase(),
                font::SMALL,
                theme::PARCHMENT.with_alpha(0.8),
            )),
            Spawn(curse(boon)),
            Spawn(footer(index, stacks)),
        )),
    )
}

/// The rarity name on a coloured band across the top of the card.
fn ribbon(rarity: Rarity, color: Color) -> impl Bundle {
    (
        Node {
            // Bleed through the card's side padding to span it edge to edge.
            align_self: AlignSelf::Stretch,
            justify_content: JustifyContent::Center,
            padding: UiRect::vertical(px(5)),
            margin: UiRect::new(px(-CARD_PADDING), px(-CARD_PADDING), px(0), px(4)),
            ..default()
        },
        BackgroundColor(color.with_alpha(0.18)),
        children![label(rarity.name().to_uppercase(), font::SMALL, color)],
    )
}

/// The boon's icon on a round plate ringed in the rarity colour.
fn medallion(boon: Boon, color: Color, library: &SpriteLibrary) -> impl Bundle + use<> {
    let icon = library.size(SpriteId::BoonGlyphs) * ICON_SCALE;
    let plate = icon.x + 22.0;
    (
        Node {
            width: px(plate),
            height: px(plate),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: px(3).all(),
            border_radius: BorderRadius::MAX,
            flex_shrink: 0.0,
            ..default()
        },
        // A plate tinted by the rarity, light enough for the icon's dark
        // outline to read against it.
        BackgroundColor(theme::PANEL_BORDER.mix(&color, 0.22)),
        BorderColor::all(color.with_alpha(0.7)),
        children![(
            ImageNode::from_atlas_image(
                library.image(SpriteId::BoonGlyphs),
                TextureAtlas { layout: library.layout(SpriteId::BoonGlyphs), index: glyph(boon) as usize },
            ),
            Node { width: px(icon.x), height: px(icon.y), ..default() },
            Pickable::IGNORE,
        )],
    )
}

fn aspect_tag(boon: Boon) -> impl Bundle {
    (
        Node {
            padding: UiRect::axes(px(8), px(2)),
            border: px(1).all(),
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        BorderColor::all(theme::PANEL_BORDER),
        children![label(boon.aspect().name().to_uppercase(), font::SMALL * 0.85, theme::EMBER)],
    )
}

/// A cursed boon's price, or nothing.
fn curse(boon: Boon) -> impl Bundle {
    let text = boon.curse().map(|cost| format!("CURSE: {}", cost.to_uppercase())).unwrap_or_default();
    centered_label(text, font::SMALL, theme::FLAME)
}

/// How often it has been taken, and the key that picks it.
fn footer(index: usize, stacks: String) -> impl Bundle {
    let key = KEY_HINTS.get(index).copied().unwrap_or("");
    (
        Node {
            width: percent(100),
            margin: UiRect::top(Val::Auto),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            label(stacks, font::SMALL * 0.85, theme::MUTED),
            (label(key, font::BODY, theme::EMBER), DeviceText { keyboard: key, touch: "" }),
        ],
    )
}

/// The app may refill the offer without leaving the phase (several levels
/// gained at once): swap the cards for the new offer.
fn rebuild_cards(
    mut commands: Commands,
    offer: Res<BoonOffer>,
    hero: Res<HeroState>,
    library: Res<SpriteLibrary>,
    row: Single<Entity, With<CardRow>>,
) {
    if offer.is_changed() {
        commands.entity(*row).despawn_related::<Children>().insert(cards(&offer, &hero, &library));
    }
}

/// Entrance, hover lift and the smoulder of legendary frames.
fn animate_cards(
    time: Res<Time>,
    mut cards: Query<(&mut BoonCard, &mut UiTransform, &mut Visibility, &mut BorderColor)>,
) {
    let dt = time.delta_secs();
    let ease = 1.0 - (-14.0 * dt).exp();
    for (mut card, mut transform, mut visibility, mut border) in &mut cards {
        card.age += dt;
        if card.age < 0.0 {
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        let entered = (card.age / ENTRANCE).min(1.0);
        // Ease-out-back: overshoots a touch, then settles.
        let t = entered - 1.0;
        let enter = 1.0 + 2.7 * t * t * t + 1.7 * t * t;
        let target_lift = if card.hovered { 1.0 } else { 0.0 };
        card.lift += (target_lift - card.lift) * ease;

        let rise = RISE * (1.0 - enter) - HOVER_LIFT * card.lift;
        let scale = 0.9 + 0.1 * enter + 0.04 * card.lift;
        transform.translation = Val2::px(0.0, rise);
        transform.scale = Vec2::splat(scale);

        let rarity = card.boon.rarity();
        let base = rarity_color(rarity);
        let color = if rarity == Rarity::Legendary {
            let glow = 0.5 + 0.5 * (time.elapsed_secs() * 4.0).sin();
            base.mix(&theme::FLAME, glow * 0.6)
        } else if card.hovered {
            theme::EMBER
        } else {
            base
        };
        border.set_if_neq(BorderColor::all(color));
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
        cards.iter().find(|(card, node, transform)| card.age > 0.0 && node.contains_point(**transform, point))
    {
        chosen.write(BoonChosen(card.boon));
    }
}

/// Mouse clicks; touches are handled by [`choose_by_tap`].
fn choose_by_click(click: On<PointerClick>, cards: Query<&BoonCard>, mut chosen: MessageWriter<BoonChosen>) {
    if click.pointer.id.is_touch() {
        return;
    }
    if let Ok(card) = cards.get(click.event_target()) {
        chosen.write(BoonChosen(card.boon));
    }
}

fn hover_card(over: On<PointerOver>, mut cards: Query<&mut BoonCard>) {
    if let Ok(mut card) = cards.get_mut(over.event_target()) {
        card.hovered = true;
    }
}

fn leave_card(out: On<PointerOut>, mut cards: Query<&mut BoonCard>) {
    if let Ok(mut card) = cards.get_mut(out.event_target()) {
        card.hovered = false;
    }
}
