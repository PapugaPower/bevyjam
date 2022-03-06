use super::*;
use crate::game::environment::barrel::ExplosiveObjectState;
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Component)]
pub struct ExplosionAnimations {
    pub explosion: Animation,
}

impl ExplosionAnimations {
    pub fn from_game_assets(
        assets: &GameAssets,
        textures: &mut Assets<TextureAtlas>,
        animations: &mut Assets<SpriteSheetAnimation>,
    ) -> Self {
        Self {
            explosion: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.explosion.clone(),
                    Vec2::new(124.0, 119.0),
                    6,
                    5,
                )),
                animation: animations.add(
                    SpriteSheetAnimation::from_range(0..=29, Duration::from_millis(50)).once(),
                ),
            },
        }
    }
}

pub fn animation_explosive_objects(
    mut commands: Commands,
    explosion_animations: Res<ExplosionAnimations>,
    query: Query<(Entity, &Transform, &ExplosiveObjectState), Without<TextureAtlasSprite>>,
) {
    for (entity, transform, state) in query.iter() {
        if let ExplosiveObjectState::Exploding(_) = state {
            commands
                .entity(entity)
                .remove_bundle::<SpriteBundle>()
                .insert_bundle(AnimationBundle::from_animation_transform_size(
                    &explosion_animations.explosion,
                    *transform,
                    None,
                ));
        }
    }
}
