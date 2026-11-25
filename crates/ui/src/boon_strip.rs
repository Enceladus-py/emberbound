//! The boons taken this run, as a strip of small framed icons: under the
//! vitals in the HUD and on the end screens.
//!
//! Icons are ordered by when each boon was first taken. Each wears its
//! rarity's frame, shows `x3` once stacked, and cursed boons get a flame-red
//! outline. The strip is rebuilt only when what it shows changes.

use bevy::prelude::*;
use emberbound_core::{GameState, HeroState, RunSetup, SpriteId, SpriteLibrary};
use emberbound_domain::boons::Boon;

use crate::boons::{glyph, rarity_color};
use crate::theme::{self, font};
use crate::widgets::label;

/// Edge length of one framed icon, in design pixels.
pub(crate) const ICON_SIZE: f32 = 34.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<TakenBoons>()
        .add_systems(OnEnter(GameState::InRun), forget_boons.in_set(RunSetup::Reset))
        .add_systems(
            Update,
            (note_taken_boons.run_if(resource_exists_and_changed::<HeroState>), fill_strips).chain(),
        );
}

/// A node whose children are the icons of the boons taken this run.
#[derive(Component)]
pub(crate) struct BoonStrip;

/// Boons taken this run, in the order first taken, with their stacks.
#[derive(Resource, Debug, Default, PartialEq)]
pub(crate) struct TakenBoons(pub(crate) Vec<(Boon, u8)>);

fn forget_boons(mut taken: ResMut<TakenBoons>) {
    taken.0.clear();
}

/// Keeps [`TakenBoons`] in step with the hero, touching it only when a
/// boon was taken, so the strips are not rebuilt for every ember.
fn note_taken_boons(hero: Res<HeroState>, mut taken: ResMut<TakenBoons>) {
    let mut next = taken.0.clone();
    for entry in &mut next {
        entry.1 = hero.stats.stacks(entry.0);
    }
    next.retain(|&(_, stacks)| stacks > 0);
    for boon in Boon::ALL {
        let stacks = hero.stats.stacks(boon);
        if stacks > 0 && !next.iter().any(|&(taken, _)| taken == boon) {
            next.push((boon, stacks));
        }
    }
    taken.set_if_neq(TakenBoons(next));
}

fn fill_strips(
    mut commands: Commands,
    taken: Res<TakenBoons>,
    library: Res<SpriteLibrary>,
    strips: Query<Entity, With<BoonStrip>>,
    added: Query<(), Added<BoonStrip>>,
) {
    if !taken.is_changed() && added.is_empty() {
        return;
    }
    for strip in &strips {
        commands.entity(strip).despawn_related::<Children>();
        for &(boon, stacks) in &taken.0 {
            commands.spawn((boon_icon(boon, stacks, &library), ChildOf(strip)));
        }
    }
}

/// One boon: its card icon on a plate framed in its rarity colour, a stack
/// count once taken more than once, and a flame-red outline if cursed.
pub(crate) fn boon_icon(boon: Boon, stacks: u8, library: &SpriteLibrary) -> impl Bundle + use<> {
    let frame = rarity_color(boon.rarity());
    let cursed = boon.curse().is_some();
    let count = if stacks > 1 { format!("x{stacks}") } else { String::new() };
    (
        Node {
            width: px(ICON_SIZE),
            height: px(ICON_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: px(2).all(),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(theme::PANEL_BORDER.mix(&frame, 0.22).with_alpha(0.92)),
        BorderColor::all(frame),
        Outline::new(px(2), px(0), if cursed { theme::FLAME } else { Color::NONE }),
        Pickable::IGNORE,
        children![
            (
                ImageNode::from_atlas_image(
                    library.image(SpriteId::BoonGlyphs),
                    TextureAtlas {
                        layout: library.layout(SpriteId::BoonGlyphs),
                        index: glyph(boon) as usize
                    },
                ),
                Node { width: px(24), height: px(24), ..default() },
                Pickable::IGNORE,
            ),
            (
                // A dark badge in the bottom-right corner keeps the count
                // legible over the icon.
                Node {
                    position_type: PositionType::Absolute,
                    right: px(-4),
                    bottom: px(-4),
                    padding: UiRect::axes(px(2), px(1)),
                    display: if count.is_empty() { Display::None } else { Display::Flex },
                    ..default()
                },
                BackgroundColor(theme::NIGHT.with_alpha(0.85)),
                Pickable::IGNORE,
                children![label(count, font::SMALL, theme::PARCHMENT)],
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use emberbound_core::CorePlugin;

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>()
            .add_plugins((CorePlugin, plugin));
        app.update();
        app
    }

    fn take(app: &mut App, boon: Boon) {
        app.world_mut().resource_mut::<HeroState>().stats.apply(boon);
        app.update();
    }

    fn taken(app: &App) -> Vec<(Boon, u8)> {
        app.world().resource::<TakenBoons>().0.clone()
    }

    fn icons(app: &mut App) -> usize {
        let strip = app.world_mut().query_filtered::<Entity, With<BoonStrip>>().single(app.world()).unwrap();
        app.world().get::<Children>(strip).map_or(0, |children| children.len())
    }

    #[test]
    fn boons_line_up_in_the_order_first_taken_with_their_stacks() {
        let mut app = app();
        assert!(taken(&app).is_empty());
        take(&mut app, Boon::Thornhide);
        take(&mut app, Boon::WhettedEdge);
        take(&mut app, Boon::Thornhide);
        take(&mut app, Boon::Thornhide);
        assert_eq!(taken(&app), vec![(Boon::Thornhide, 3), (Boon::WhettedEdge, 1)]);
    }

    #[test]
    fn the_strip_shows_one_icon_per_boon_and_only_rebuilds_on_change() {
        let mut app = app();
        app.world_mut().spawn((BoonStrip, Node::default()));
        app.update();
        assert_eq!(icons(&mut app), 0, "empty at run start");

        take(&mut app, Boon::Scalemail);
        take(&mut app, Boon::Scalemail);
        take(&mut app, Boon::MoltenHeart);
        assert_eq!(icons(&mut app), 2);

        let tick = app.world().resource_ref::<TakenBoons>().last_changed();
        app.world_mut().resource_mut::<HeroState>().progress.add_xp(1);
        app.update();
        assert_eq!(
            app.world().resource_ref::<TakenBoons>().last_changed(),
            tick,
            "embers alone change nothing"
        );
    }

    #[test]
    fn a_new_run_starts_with_an_empty_strip() {
        let mut app = app();
        take(&mut app, Boon::Fleetfoot);
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::InRun);
        app.world_mut().insert_resource(HeroState::default());
        app.update();
        assert!(taken(&app).is_empty());
    }
}
