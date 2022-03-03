use crate::game::environment::barrel::ExplosiveObjectState;
use crate::game::shooting::{BulletImpactEvent, ImpactSurface};
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Component)]
pub enum ShootingAnimationState {
    NotShooting,
    Shooting,
}

#[derive(Component)]
pub struct ShootingAnimationTimer {
    pub animation_time: f32,
    pub timer: Option<Timer>,
}

#[derive(Bundle)]
pub struct ShootingAnimationBundle {
    pub state: ShootingAnimationState,
    pub timer: ShootingAnimationTimer,
}

impl Default for ShootingAnimationBundle {
    fn default() -> Self {
        Self {
            state: ShootingAnimationState::NotShooting,
            timer: ShootingAnimationTimer {
                animation_time: 0.05,
                timer: None,
            },
        }
    }
}

pub fn animations_removal(mut commands: Commands, animations: RemovedComponents<Play>) {
    for entity in animations.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn animation_player_shooting(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut query_player: Query<(
        &mut Handle<Image>,
        &mut ShootingAnimationState,
        &mut ShootingAnimationTimer,
    )>,
) {
    let (mut texture, mut state, mut timer) = query_player.single_mut();
    match state.as_mut() {
        ShootingAnimationState::NotShooting => {
            if let Some(t) = &mut timer.timer {
                t.tick(time.delta());
                if t.finished() {
                    *texture = assets.player_idle.clone();
                    timer.timer = None;
                }
            }
        }
        ShootingAnimationState::Shooting => {
            if let Some(t) = &mut timer.timer {
                t.reset();
            } else {
                timer.timer = Some(Timer::from_seconds(timer.animation_time, false));
                *texture = assets.player_shooting.clone();
            }
            *state = ShootingAnimationState::NotShooting;
        }
    }
}

pub fn animation_player_impact(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    mut events: EventReader<BulletImpactEvent>,
) {
    for event in events.iter() {
        let rotation = event.direction.y.atan2(event.direction.x) - std::f32::consts::FRAC_PI_2;
        match event.surface {
            ImpactSurface::Player => {}
            ImpactSurface::World => {
                let animation_handle = animations.add(
                    SpriteSheetAnimation::from_range(0..=15, Duration::from_millis(20)).once(),
                );
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: textures.add(TextureAtlas::from_grid(
                            assets.hit_0.clone(),
                            Vec2::new(80.0, 80.0),
                            4,
                            4,
                        )),
                        transform: Transform::from_translation(event.position)
                            .with_rotation(Quat::from_rotation_z(rotation)),
                        ..Default::default()
                    })
                    // Insert the asset handle of the animation
                    .insert(animation_handle.clone())
                    .insert(Play)
                    .insert(super::GameCleanup);
            }
            ImpactSurface::Monster => {
                let animation_handle = animations.add(
                    SpriteSheetAnimation::from_range(0..=24, Duration::from_millis(20)).once(),
                );
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: textures.add(TextureAtlas::from_grid(
                            assets.blood_splash.clone(),
                            Vec2::new(100.0, 100.0),
                            5,
                            5,
                        )),
                        transform: Transform::from_translation(event.position)
                            .with_rotation(Quat::from_rotation_z(rotation)),
                        ..Default::default()
                    })
                    // Insert the asset handle of the animation
                    .insert(animation_handle.clone())
                    .insert(Play)
                    .insert(super::GameCleanup);
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
