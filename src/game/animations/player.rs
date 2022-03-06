use super::*;
use crate::game::player::{PlayerLegs, PlayerMoveState, PlayerShootState, PlayerState};
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Component)]
pub struct PlayerAnimations {
    pub legs: Animation,
    pub idle: Animation,
    pub running: Animation,
    pub reloading: Animation,
    pub shooting: Animation,
}

impl PlayerAnimations {
    pub fn from_game_assets(
        assets: &GameAssets,
        textures: &mut Assets<TextureAtlas>,
        animations: &mut Assets<SpriteSheetAnimation>,
    ) -> Self {
        Self {
            legs: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.player_legs.clone(),
                    Vec2::new(204.0, 124.0),
                    5,
                    4,
                )),
                animation: animations.add(SpriteSheetAnimation::from_range(
                    0..=19,
                    Duration::from_millis(30),
                )),
            },
            idle: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.player_idle.clone(),
                    Vec2::new(313.0, 207.0),
                    3,
                    7,
                )),
                animation: animations.add(SpriteSheetAnimation::from_range(
                    0..=19,
                    Duration::from_millis(50),
                )),
            },
            running: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.player_move.clone(),
                    Vec2::new(313.0, 207.0),
                    3,
                    7,
                )),
                animation: animations.add(SpriteSheetAnimation::from_range(
                    0..=19,
                    Duration::from_millis(50),
                )),
            },
            reloading: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.player_reload.clone(),
                    Vec2::new(322.0, 217.0),
                    3,
                    7,
                )),
                animation: animations.add(SpriteSheetAnimation::from_range(
                    0..=19,
                    Duration::from_millis(90),
                )),
            },
            shooting: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.player_shoot.clone(),
                    Vec2::new(312.0, 206.0),
                    3,
                    1,
                )),
                animation: animations.add(SpriteSheetAnimation::from_range(
                    0..=2,
                    Duration::from_millis(50),
                )),
            },
        }
    }
}

#[allow(clippy::complexity)]
pub fn animation_player(
    mut commands: Commands,
    animations: Res<PlayerAnimations>,
    mut query_set: QuerySet<(
        QueryState<(
            &mut PlayerState,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
            &mut Handle<SpriteSheetAnimation>,
        )>,
        QueryState<(
            Entity,
            &mut PlayerLegs,
            &mut Handle<TextureAtlas>,
            &mut Handle<SpriteSheetAnimation>,
        )>,
    )>,
) {
    let mut query_playr_legs = query_set.q1();
    let (legs, mut player_legs, mut legs_texture_atlas, mut legs_animation) =
        query_playr_legs.single_mut();
    if !player_legs.initialized {
        *legs_texture_atlas = animations.legs.texture_atlas.clone();
        *legs_animation = animations.legs.animation.clone();
        player_legs.initialized = true;
    }

    let mut query_player = query_set.q0();
    let (mut state, mut texture_atlas, mut texture_atlas_sprite, mut animation) =
        query_player.single_mut();
    if state.changed() {
        println!("animation change: {:?}", state);
        match state.new.0 {
            PlayerMoveState::Idle => {
                *texture_atlas = animations.idle.texture_atlas.clone();
                *animation = animations.idle.animation.clone();
                commands.entity(legs).remove::<Play>();
            }
            PlayerMoveState::Running => {
                *texture_atlas = animations.running.texture_atlas.clone();
                *animation = animations.running.animation.clone();
                commands.entity(legs).insert(Play);
            }
        }
        match state.new.1 {
            PlayerShootState::Nothing => {}
            PlayerShootState::Reloading => {
                *texture_atlas = animations.reloading.texture_atlas.clone();
                *animation = animations.reloading.animation.clone();
            }
            PlayerShootState::Shooting => {
                *texture_atlas = animations.shooting.texture_atlas.clone();
                *animation = animations.shooting.animation.clone();
            }
        }
        // resetting index
        texture_atlas_sprite.index = 0;
        state.current = state.new;
    }
}
