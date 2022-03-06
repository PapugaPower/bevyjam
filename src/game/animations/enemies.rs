use super::*;
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Component)]
pub struct EnemyAnimations {
    pub movement: Animation,
}

impl EnemyAnimations {
    pub fn from_game_assets(
        assets: &GameAssets,
        textures: &mut Assets<TextureAtlas>,
        animations: &mut Assets<SpriteSheetAnimation>,
    ) -> Self {
        Self {
            movement: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.enemy_move.clone(),
                    Vec2::new(64.0, 64.0),
                    8,
                    1,
                )),
                animation: animations.add(
                    SpriteSheetAnimation::from_range(0..=7, Duration::from_millis(50)),
                ),
            },
        }
    }
}
