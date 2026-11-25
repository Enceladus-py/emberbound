//! In-run HUD: hero vitals with the boons taken beneath them (top-left),
//! run facts with the minimap under them (top-right), the dragon lure /
//! boss bar (top-centre), a fading controls hint (bottom-centre) and the
//! low-health vignette.

use bevy::prelude::*;
use emberbound_core::{DashCooldown, Dragon, GameState, Health, HeroState, Player, RunSetup, RunStats};
use emberbound_domain::bestiary::{DragonKind, DragonPhase};

use crate::boon_strip::BoonStrip;
use crate::minimap::MinimapSlot;
use crate::theme::{self, font};
use crate::widgets::{Bar, bar, clock, device_label, label, panel, set_text, shadowed_label, text_shadow};

/// Seconds of run time after which the controls hint starts to fade.
const HINT_VISIBLE_SECONDS: f32 = 12.0;
const HINT_FADE_SECONDS: f32 = 1.5;
/// Health fraction below which the screen edge pulses red.
const LOW_HEALTH: f32 = 0.3;
/// Width of the vitals and run facts panels: room for a full health bar
/// and "HP 100/100" side by side.
const HUD_PANEL_WIDTH: f32 = 280.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InRun), spawn_hud.in_set(RunSetup::Spawn)).add_systems(
        Update,
        (update_vitals, update_run_facts, update_dragon_bars, fade_hint, pulse_vignette)
            .run_if(in_state(GameState::InRun)),
    );
}

#[derive(Component)]
struct HealthFill;
#[derive(Component)]
struct HealthText;
#[derive(Component)]
struct XpFill;
#[derive(Component)]
struct LevelText;
#[derive(Component)]
struct DashFill;
#[derive(Component)]
struct TimeText;
#[derive(Component)]
struct KillsText;
#[derive(Component)]
struct DragonsText;
/// The slim "next dragon" progress box.
#[derive(Component)]
struct LureBox;
#[derive(Component)]
struct LureFill;
#[derive(Component)]
struct LureText;
/// The big boss health box.
#[derive(Component)]
struct BossBox;
#[derive(Component)]
struct BossFill;
#[derive(Component)]
struct BossName;
#[derive(Component)]
struct BossEpithet;
#[derive(Component)]
struct HintText;
/// Border ring of the low-health vignette; `strength` scales its opacity.
#[derive(Component)]
struct Vignette {
    strength: f32,
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::InRun),
        GlobalZIndex(theme::z::HUD),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            padding: px(14).all(),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            column_gap: px(12),
            ..default()
        },
        children![
            (column(8.0), children![vitals(), (BoonStrip, boon_strip_node(HUD_PANEL_WIDTH))]),
            dragon_bars(),
            (
                Node { align_items: AlignItems::FlexEnd, ..column(8.0) },
                children![run_facts(), (MinimapSlot, Node::default())]
            ),
            hint(),
            vignette(18.0, 0.5),
            vignette(42.0, 0.22)
        ],
    ));
}

fn row(gap: f32) -> Node {
    Node { align_items: AlignItems::Center, column_gap: px(gap), ..default() }
}

fn column(gap: f32) -> Node {
    Node { flex_direction: FlexDirection::Column, row_gap: px(gap), ..default() }
}

/// The row the [`BoonStrip`] fills, wrapping under the vitals panel.
fn boon_strip_node(width: f32) -> Node {
    Node { width: px(width), flex_wrap: FlexWrap::Wrap, column_gap: px(4), row_gap: px(4), ..default() }
}

fn vitals() -> impl Bundle {
    (
        panel(Node { padding: px(10).all(), width: px(HUD_PANEL_WIDTH), ..column(7.0) }),
        children![
            (
                row(8.0),
                children![
                    bar(HealthFill, px(128), px(16), theme::HEALTH),
                    (label("HP", font::SMALL, theme::PARCHMENT), HealthText),
                ],
            ),
            (
                row(8.0),
                children![
                    bar(XpFill, px(128), px(10), theme::XP),
                    (label("LV 1", font::SMALL, theme::XP), LevelText),
                ],
            ),
            (
                row(8.0),
                children![
                    bar(DashFill, px(56), px(8), theme::FROST),
                    label("DASH", font::SMALL, theme::MUTED),
                ],
            ),
        ],
    )
}

fn run_facts() -> impl Bundle {
    (
        panel(Node {
            padding: px(10).all(),
            width: px(HUD_PANEL_WIDTH),
            align_items: AlignItems::FlexEnd,
            ..column(4.0)
        }),
        children![
            (label("00:00", font::TITLE, theme::PARCHMENT), TimeText),
            (label("KILLS 0", font::SMALL, theme::MUTED), KillsText),
            (label("DRAGONS 0/3", font::SMALL, theme::EMBER), DragonsText),
        ],
    )
}

fn dragon_bars() -> impl Bundle {
    (
        Node { flex_grow: 1.0, max_width: px(520), align_items: AlignItems::Center, ..column(0.0) },
        children![
            (
                LureBox,
                Node { width: percent(70), align_items: AlignItems::Center, ..column(4.0) },
                children![
                    (shadowed_label("", font::SMALL, theme::PARCHMENT), LureText),
                    bar(LureFill, percent(100), px(8), theme::EMBER),
                ],
            ),
            (
                BossBox,
                Node {
                    width: percent(100),
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..column(4.0)
                },
                children![
                    (shadowed_label("", font::TITLE, theme::PARCHMENT), BossName),
                    bar(BossFill, percent(100), px(20), theme::FLAME),
                    (shadowed_label("", font::SMALL, theme::MUTED), BossEpithet),
                ],
            ),
        ],
    )
}

fn hint() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            bottom: px(18),
            left: px(0),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            device_label(
                "MOVE: WASD / ARROWS  -  SWORD: SPACE / J / CLICK  -  DASH: SHIFT / K  -  PAUSE: ESC",
                "MOVE: LEFT THUMB  -  SWORD: HOLD RIGHT SIDE, SLIDE TO AIM  -  DASH: DASH BUTTON",
                font::SMALL,
                theme::PARCHMENT,
            ),
            text_shadow(font::SMALL),
            HintText,
        )],
    )
}

/// One ring of the vignette: a transparent full-screen node with a thick
/// border. Two rings of different width fake a soft falloff.
fn vignette(width: f32, strength: f32) -> impl Bundle {
    (
        Vignette { strength },
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            border: px(width).all(),
            ..default()
        },
        BorderColor::all(Color::NONE),
    )
}

fn update_vitals(
    player: Single<(&Health, &DashCooldown), With<Player>>,
    hero: Res<HeroState>,
    mut health_fill: Single<&mut Bar, (With<HealthFill>, Without<XpFill>, Without<DashFill>)>,
    mut xp_fill: Single<&mut Bar, (With<XpFill>, Without<HealthFill>, Without<DashFill>)>,
    mut dash_fill: Single<
        (&mut Bar, &mut BackgroundColor),
        (With<DashFill>, Without<HealthFill>, Without<XpFill>),
    >,
    mut health_text: Single<&mut Text, (With<HealthText>, Without<LevelText>)>,
    mut level_text: Single<&mut Text, (With<LevelText>, Without<HealthText>)>,
) {
    let (health, dash) = *player;
    health_fill.target = health.fraction();
    set_text(&mut health_text, format!("HP {:.0}/{:.0}", health.current().ceil(), health.max()));

    xp_fill.target = hero.progress.fraction();
    set_text(&mut level_text, format!("LV {}", hero.progress.level()));

    let ready = dash.remaining <= 0.0;
    dash_fill.0.target = if ready || dash.total <= 0.0 { 1.0 } else { 1.0 - dash.remaining / dash.total };
    let color = if ready { theme::FROST } else { theme::MUTED.with_alpha(0.7) };
    if dash_fill.1.0 != color {
        dash_fill.1.0 = color;
    }
}

fn update_run_facts(
    stats: Res<RunStats>,
    mut time: Single<&mut Text, (With<TimeText>, Without<KillsText>, Without<DragonsText>)>,
    mut kills: Single<&mut Text, (With<KillsText>, Without<TimeText>, Without<DragonsText>)>,
    mut dragons: Single<&mut Text, (With<DragonsText>, Without<TimeText>, Without<KillsText>)>,
) {
    set_text(&mut time, clock(stats.elapsed));
    set_text(&mut kills, format!("KILLS {}", stats.kills));
    set_text(&mut dragons, format!("DRAGONS {}/{}", stats.dragons_slain, DragonKind::ALL.len()));
}

fn dragon_color(kind: DragonKind) -> Color {
    match kind {
        DragonKind::MossWyrm => theme::MOSS,
        DragonKind::FrostDrake => theme::FROST,
        DragonKind::AshenTyrant => theme::FLAME,
    }
}

fn show(node: &mut Mut<Node>, visible: bool) {
    let display = if visible { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}

fn update_dragon_bars(
    time: Res<Time>,
    stats: Res<RunStats>,
    dragons: Query<(&Dragon, &Health)>,
    mut lure_box: Single<&mut Node, (With<LureBox>, Without<BossBox>)>,
    mut boss_box: Single<&mut Node, (With<BossBox>, Without<LureBox>)>,
    mut lure_fill: Single<&mut Bar, (With<LureFill>, Without<BossFill>)>,
    mut boss_fill: Single<(&mut Bar, &mut BackgroundColor), (With<BossFill>, Without<LureFill>)>,
    mut lure_text: Single<&mut Text, (With<LureText>, Without<BossName>, Without<BossEpithet>)>,
    mut boss_name: Single<&mut Text, (With<BossName>, Without<LureText>, Without<BossEpithet>)>,
    mut boss_epithet: Single<
        (&mut Text, &mut TextColor),
        (With<BossEpithet>, Without<LureText>, Without<BossName>),
    >,
) {
    let boss = dragons.iter().next();
    show(&mut boss_box, boss.is_some());
    show(&mut lure_box, boss.is_none() && stats.next_dragon.is_some());

    let Some((dragon, health)) = boss else {
        if let Some(next) = stats.next_dragon {
            lure_fill.target = stats.lure_progress;
            set_text(&mut lure_text, format!("{} STIRS", next.spec().epithet.to_uppercase()));
        }
        return;
    };

    let spec = dragon.kind.spec();
    let fraction = health.fraction();
    let enraged = spec.phase(fraction) == DragonPhase::Enraged;
    boss_fill.0.target = fraction;
    set_text(&mut boss_name, spec.name.to_uppercase());

    let base = dragon_color(dragon.kind);
    let (color, epithet, epithet_color) = if enraged {
        // An enraged dragon's bar burns hotter and throbs.
        let throb = 0.5 + 0.5 * (time.elapsed_secs() * 9.0).sin();
        let hot = base.mix(&theme::FLAME, 0.45).lighter(0.05 + 0.12 * throb);
        (hot, format!("{} - ENRAGED", spec.epithet.to_uppercase()), theme::FLAME)
    } else {
        (base, spec.epithet.to_uppercase(), theme::MUTED)
    };
    boss_fill.1.0 = color;
    set_text(&mut boss_epithet.0, epithet);
    if boss_epithet.1.0 != epithet_color {
        boss_epithet.1.0 = epithet_color;
    }
}

fn fade_hint(stats: Res<RunStats>, mut hint: Single<(&mut TextColor, &mut TextShadow), With<HintText>>) {
    let alpha = (1.0 - (stats.elapsed - HINT_VISIBLE_SECONDS) / HINT_FADE_SECONDS).clamp(0.0, 1.0);
    if hint.0.0.alpha() != alpha {
        hint.0.0.set_alpha(alpha);
        hint.1.color.set_alpha(alpha * 0.8);
    }
}

fn pulse_vignette(
    time: Res<Time>,
    player: Single<&Health, With<Player>>,
    mut rings: Query<(&Vignette, &mut BorderColor)>,
) {
    let fraction = player.fraction();
    let danger = if fraction < LOW_HEALTH && !player.is_dead() {
        // Grows more insistent the closer the hero is to death.
        let urgency = 1.0 - fraction / LOW_HEALTH;
        (0.45 + 0.55 * urgency) * (0.55 + 0.45 * (time.elapsed_secs() * 5.0).sin())
    } else {
        0.0
    };
    for (ring, mut border) in &mut rings {
        let color = theme::HEALTH.with_alpha(danger * ring.strength);
        if border.top != color {
            border.set_all(color);
        }
    }
}
