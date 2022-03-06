use super::*;
use crate::game::shooting::{BulletImpactEvent, ImpactSurface};
use crate::game::GameAssets;
use benimator::{Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy::utils::Duration;

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
