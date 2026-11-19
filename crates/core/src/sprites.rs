//! Adapter from the engine-free pixel art crate to GPU sprites, plus a
//! minimal frame animator.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use emberbound_pixelart::{self as pixelart, SpriteId};

struct Entry {
    /// CPU copy of the frames, for baking sprites into bigger images.
    sheet: pixelart::SpriteSheet,
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frames: usize,
    size: Vec2,
}

/// Every sprite of the game, uploaded and ready to spawn.
#[derive(Resource)]
pub struct SpriteLibrary {
    entries: HashMap<SpriteId, Entry>,
}

impl SpriteLibrary {
    fn entry(&self, id: SpriteId) -> &Entry {
        &self.entries[&id]
    }

    /// A `Sprite` showing the first frame of `id`.
    pub fn sprite(&self, id: SpriteId) -> Sprite {
        self.sprite_frame(id, 0)
    }

    /// A `Sprite` showing a specific frame (wraps around).
    pub fn sprite_frame(&self, id: SpriteId, frame: usize) -> Sprite {
        let entry = self.entry(id);
        Sprite::from_atlas_image(
            entry.image.clone(),
            TextureAtlas { layout: entry.layout.clone(), index: frame % entry.frames },
        )
    }

    /// A looping animated sprite: spawn both components together.
    pub fn animated(&self, id: SpriteId, fps: f32) -> (Sprite, SpriteAnim) {
        (self.sprite(id), SpriteAnim::looping(self.frames(id), fps))
    }

    /// A sprite that plays once and then despawns its entity.
    pub fn one_shot(&self, id: SpriteId, fps: f32) -> (Sprite, SpriteAnim) {
        (self.sprite(id), SpriteAnim::once(self.frames(id), fps))
    }

    /// Switches an existing animated sprite to another sheet, keeping flips,
    /// colour and timing. Does nothing if it already shows `id`.
    pub fn switch(&self, sprite: &mut Sprite, anim: &mut SpriteAnim, id: SpriteId) {
        let entry = self.entry(id);
        if sprite.image == entry.image {
            return;
        }
        sprite.image = entry.image.clone();
        sprite.texture_atlas = Some(TextureAtlas { layout: entry.layout.clone(), index: 0 });
        anim.frames = entry.frames;
    }

    pub fn frames(&self, id: SpriteId) -> usize {
        self.entry(id).frames
    }

    /// Size of one frame in pixels (= world units).
    pub fn size(&self, id: SpriteId) -> Vec2 {
        self.entry(id).size
    }

    /// The whole strip image, e.g. for UI `ImageNode`s (see [`Self::layout`]).
    pub fn image(&self, id: SpriteId) -> Handle<Image> {
        self.entry(id).image.clone()
    }

    pub fn layout(&self, id: SpriteId) -> Handle<TextureAtlasLayout> {
        self.entry(id).layout.clone()
    }

    /// RGBA8 pixels of one frame (wraps around), row-major from the top
    /// row, [`Self::size`] large. For baking many static sprites into one
    /// image instead of spawning thousands of entities.
    pub fn pixels(&self, id: SpriteId, frame: usize) -> &[u8] {
        let frames = &self.entry(id).sheet.frames;
        &frames[frame % frames.len()]
    }
}

/// Steps through the frames of an atlas sprite.
#[derive(Component, Debug, Clone)]
pub struct SpriteAnim {
    pub frames: usize,
    pub timer: Timer,
    /// Despawn the entity after the last frame instead of looping.
    pub despawn_when_done: bool,
}

impl SpriteAnim {
    pub fn looping(frames: usize, fps: f32) -> Self {
        Self {
            frames,
            timer: Timer::from_seconds(1.0 / fps.max(0.1), TimerMode::Repeating),
            despawn_when_done: false,
        }
    }

    pub fn once(frames: usize, fps: f32) -> Self {
        Self { despawn_when_done: true, ..Self::looping(frames, fps) }
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut commands: Commands,
    mut sprites: Query<(Entity, &mut Sprite, &mut SpriteAnim)>,
) {
    for (entity, mut sprite, mut anim) in &mut sprites {
        if !anim.timer.tick(time.delta()).just_finished() {
            continue;
        }
        let Some(atlas) = sprite.texture_atlas.as_mut() else { continue };
        if atlas.index + 1 < anim.frames {
            atlas.index += 1;
        } else if anim.despawn_when_done {
            commands.entity(entity).try_despawn();
        } else {
            atlas.index = 0;
        }
    }
}

fn build_library(world: &mut World) -> SpriteLibrary {
    let mut entries = HashMap::new();
    for id in SpriteId::ALL {
        let sheet = pixelart::sheet(id);
        let (width, height, pixels) = sheet.to_strip();
        let image = Image::new(
            Extent3d { width, height, depth_or_array_layers: 1 },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(sheet.frame_width, sheet.frame_height),
            sheet.frames.len() as u32,
            1,
            None,
            None,
        );
        let entry = Entry {
            image: world.resource_mut::<Assets<Image>>().add(image),
            layout: world.resource_mut::<Assets<TextureAtlasLayout>>().add(layout),
            frames: sheet.frames.len(),
            size: Vec2::new(sheet.frame_width as f32, sheet.frame_height as f32),
            sheet,
        };
        entries.insert(id, entry);
    }
    SpriteLibrary { entries }
}

pub(crate) fn register(app: &mut App) {
    // Built eagerly so the library exists before any schedule runs.
    let library = build_library(app.world_mut());
    app.insert_resource(library).add_systems(Update, animate_sprites);
}
