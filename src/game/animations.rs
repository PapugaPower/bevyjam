use crate::game::environment::barrel::ExplosiveObjectState;
use crate::game::player::{PlayerState, PlayerStateEnum};
use crate::game::shooting::{BulletImpactEvent, ImpactSurface};
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

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
    pub cleanup: super::GameCleanup,
}

impl AnimationBundle {
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
            cleanup: super::GameCleanup,
        }
    }
}

#[derive(Component)]
pub struct BulletsImpactAnimations {
    pub world: Animation,
    pub monsters: Animation,
}

impl BulletsImpactAnimations {
    pub fn from_game_assets(
        assets: &GameAssets,
        textures: &mut Assets<TextureAtlas>,
        animations: &mut Assets<SpriteSheetAnimation>,
    ) -> Self {
        Self {
            world: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.hit_0.clone(),
                    Vec2::new(80.0, 80.0),
                    4,
                    4,
                )),
                animation: animations.add(
                    SpriteSheetAnimation::from_range(0..=15, Duration::from_millis(20)).once(),
                ),
            },
            monsters: Animation {
                texture_atlas: textures.add(TextureAtlas::from_grid(
                    assets.blood_splash.clone(),
                    Vec2::new(100.0, 100.0),
                    5,
                    5,
                )),
                animation: animations.add(
                    SpriteSheetAnimation::from_range(0..=24, Duration::from_millis(20)).once(),
                ),
            },
        }
    }
}

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
                    Duration::from_millis(50),
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

pub fn animations_removal(mut commands: Commands, animations: RemovedComponents<Play>) {
    for entity in animations.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn animation_player(
    mut query_player: Query<(
        &PlayerAnimations,
        &mut PlayerState,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
        &mut Handle<SpriteSheetAnimation>,
    )>,
) {
    let (animations, mut state, mut texture_atlas, mut texture_atlas_sprite, mut animation) =
        query_player.single_mut();
    if state.changed() {
        println!("animation change: {:?}", state);
        match state.new {
            PlayerStateEnum::Idle => {
                *texture_atlas = animations.idle.texture_atlas.clone();
                *animation = animations.idle.animation.clone();
            }
            PlayerStateEnum::Running => {
                *texture_atlas = animations.running.texture_atlas.clone();
                *animation = animations.running.animation.clone();
            }
            PlayerStateEnum::Reloading => {
                *texture_atlas = animations.reloading.texture_atlas.clone();
                *animation = animations.reloading.animation.clone();
            }
            PlayerStateEnum::Shooting => {
                *texture_atlas = animations.shooting.texture_atlas.clone();
                *animation = animations.shooting.animation.clone();
            }
        }
        // resetting index
        texture_atlas_sprite.index = 0;
        state.current = state.new;
    }
}

pub fn animation_player_impact(
    mut commands: Commands,
    mut events: EventReader<BulletImpactEvent>,
    impact_animations: Query<&BulletsImpactAnimations>,
) {
    let animations = impact_animations.single();
    for event in events.iter() {
        let rotation = event.direction.y.atan2(event.direction.x) - std::f32::consts::FRAC_PI_2;
        let transform = Transform::from_translation(event.position)
            .with_rotation(Quat::from_rotation_z(rotation));
        match event.surface {
            ImpactSurface::Player => {}
            ImpactSurface::World => {
                commands.spawn_bundle(AnimationBundle::from_animation_transform_size(
                    &animations.world,
                    transform,
                    None,
                ));
            }
            ImpactSurface::Monster => {
                commands.spawn_bundle(AnimationBundle::from_animation_transform_size(
                    &animations.monsters,
                    transform,
                    None,
                ));
            }
        }
    }
}

pub fn animation_explosive_objects(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    query: Query<(Entity, &Transform, &ExplosiveObjectState), Without<TextureAtlasSprite>>,
) {
    let animation_handle = animations.add(SpriteSheetAnimation::from_range(
        0..=29,
        Duration::from_millis(50),
    ));
    for (entity, transform, state) in query.iter() {
        if let ExplosiveObjectState::Exploding(_) = state {
            commands
                .entity(entity)
                .remove_bundle::<SpriteBundle>()
                .insert_bundle(SpriteSheetBundle {
                    texture_atlas: textures.add(TextureAtlas::from_grid(
                        assets.explosion.clone(),
                        Vec2::new(124.0, 119.0),
                        6,
                        5,
                    )),
                    transform: *transform,
                    ..Default::default()
                })
                // Insert the asset handle of the animation
                .insert(animation_handle.clone())
                .insert(Play)
                .insert(super::GameCleanup);
        }
    }
}
