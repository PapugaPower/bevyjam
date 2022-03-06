use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

pub mod enemies;
pub mod environment;
pub mod player;
pub mod shooting;

pub use enemies::*;
pub use environment::*;
pub use player::*;
pub use shooting::*;

#[derive(Component)]
pub struct AnimationTag;

#[derive(Component)]
pub struct PausableAnimationTag;

pub struct Animation {
    pub texture_atlas: Handle<TextureAtlas>,
    pub animation: Handle<SpriteSheetAnimation>,
}

#[derive(Bundle)]
pub struct AnimationBundle {
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub animation_handle: Handle<SpriteSheetAnimation>,
    pub play: Play,
    pub animation_tag: AnimationTag,
    pub cleanup: super::GameCleanup,
}

impl AnimationBundle {
    pub fn default_with_transform_size(transform: Transform, size: Option<Vec2>) -> Self {
        AnimationBundle {
            sprite_sheet_bundle: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    custom_size: size,
                    ..Default::default()
                },
                transform,
                ..Default::default()
            },
            animation_handle: Handle::default(),
            play: Play,
            animation_tag: AnimationTag,
            cleanup: super::GameCleanup,
        }
    }

    pub fn from_animation_transform_size(
        animation: &Animation,
        transform: Transform,
        size: Option<Vec2>,
    ) -> Self {
        AnimationBundle {
            sprite_sheet_bundle: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    custom_size: size,
                    ..Default::default()
                },
                texture_atlas: animation.texture_atlas.clone(),
                transform,
                ..Default::default()
            },
            animation_handle: animation.animation.clone(),
            play: Play,
            animation_tag: AnimationTag,
            cleanup: super::GameCleanup,
        }
    }
}

#[derive(Bundle)]
pub struct AnimationPausableBundle {
    #[bundle]
    pub animation_bundle: AnimationBundle,
    pub pausable_tag: PausableAnimationTag,
}

impl AnimationPausableBundle {
    pub fn default_with_transform_size(transform: Transform, size: Option<Vec2>) -> Self {
        Self {
            animation_bundle: AnimationBundle::default_with_transform_size(transform, size),
            pausable_tag: PausableAnimationTag,
        }
    }

    pub fn from_animation_transform_size(
        animation: &Animation,
        transform: Transform,
        size: Option<Vec2>,
    ) -> Self {
        Self {
            animation_bundle: AnimationBundle::from_animation_transform_size(
                animation, transform, size,
            ),
            pausable_tag: PausableAnimationTag,
        }
    }
}

pub fn animations_init(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    commands.insert_resource(PlayerAnimations::from_game_assets(
        &assets,
        &mut textures,
        &mut animations,
    ));
    commands.insert_resource(BulletsImpactAnimations::from_game_assets(
        &assets,
        &mut textures,
        &mut animations,
    ));
    commands.insert_resource(ExplosionAnimations::from_game_assets(
        &assets,
        &mut textures,
        &mut animations,
    ));
    commands.insert_resource(EnemyAnimations::from_game_assets(
        &assets,
        &mut textures,
        &mut animations,
    ));
}

#[allow(clippy::complexity)]
pub fn animations_removal(
    mut commands: Commands,
    animations: Query<
        Entity,
        (
            Without<Play>,
            With<AnimationTag>,
            Without<PausableAnimationTag>,
        ),
    >,
) {
    for entity in animations.iter() {
        commands.entity(entity).despawn();
    }
}
