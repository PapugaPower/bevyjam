use crate::game::environment::barrel::ExplosiveObjectState;
use crate::game::player::{PlayerLegs, PlayerMoveState, PlayerShootState, PlayerState};
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
    pub fn from_default_with_transform_size(transform: Transform, size: Option<Vec2>) -> Self {
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
                    3,
                    3,
                )),
                animation: animations
                    .add(SpriteSheetAnimation::from_range(0..=8, Duration::from_millis(20)).once()),
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

pub fn animations_removal(
    mut commands: Commands,
    animations: Query<
        Entity,
        (
            With<Handle<SpriteSheetAnimation>>,
            Without<Play>,
            Without<PlayerLegs>,
        ),
    >,
) {
    for entity in animations.iter() {
        commands.entity(entity).despawn();
    }
}

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

pub fn animation_player_impact(
    mut commands: Commands,
    mut events: EventReader<BulletImpactEvent>,
    impact_animations: Res<BulletsImpactAnimations>,
) {
    for event in events.iter() {
        let rotation = event.direction.y.atan2(event.direction.x) - std::f32::consts::FRAC_PI_2;
        let mut transform = Transform::from_translation(event.position)
            .with_rotation(Quat::from_rotation_z(rotation));
        match event.surface {
            ImpactSurface::Player => {}
            ImpactSurface::World => {
                commands.spawn_bundle(AnimationBundle::from_animation_transform_size(
                    &impact_animations.world,
                    transform,
                    None,
                ));
            }
            ImpactSurface::Monster => {
                transform.scale = Vec3::new(1.5, 1.5, 1.0);
                commands.spawn_bundle(AnimationBundle::from_animation_transform_size(
                    &impact_animations.monsters,
                    transform,
                    None,
                ));
            }
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
