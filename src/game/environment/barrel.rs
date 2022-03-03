use crate::game::damage::{Health, Pulsing};
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Debug, Component)]
pub enum ExplosiveObjectState {
    NotDetonated,
    Exploding(Timer),
    Detonated,
}

#[derive(Component)]
pub struct ExplosiveObject {
    pub state: ExplosiveObjectState,
}

pub fn explosive_objects_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Health, &mut Pulsing, &mut ExplosiveObjectState)>,
) {
    for (entity, health, mut pulsing, mut state) in query.iter_mut() {
        match state.as_mut() {
            ExplosiveObjectState::NotDetonated => {
                if health.current <= 0.0 {
                    *state = ExplosiveObjectState::Exploding(Timer::from_seconds(1.5, false));
                    pulsing.pulse_time.unpause();
                }
            }
            ExplosiveObjectState::Exploding(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    *state = ExplosiveObjectState::Detonated;
                }
            }
            ExplosiveObjectState::Detonated => {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn explosive_objects_animation(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    query: Query<(Entity, &Transform, &ExplosiveObjectState), Without<TextureAtlasSprite>>,
) {
    if let Some(assets) = assets {
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
                    .insert(Play);
            }
        }
    }
}
