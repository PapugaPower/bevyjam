use crate::game::damage::{DamageAreaShape, Health, Pulsing};
use crate::game::phys_layers::PhysLayer;
use bevy::prelude::*;
use heron::prelude::*;

#[derive(Debug, Component)]
pub enum ExplosiveObjectState {
    NotDetonated,
    Exploding,
    Detonated,
}

#[derive(Component)]
pub struct ExplosiveObject {
    pub state: ExplosiveObjectState,
}

pub fn explosive_objects_controller(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, &mut Pulsing, &mut ExplosiveObjectState)>,
) {
    for (entity, health, mut pulsing, mut state) in query.iter_mut() {
        match *state {
            ExplosiveObjectState::NotDetonated => {
                if health.current <= 0.0 {
                    *state = ExplosiveObjectState::Exploding;
                    pulsing.pulse_time.unpause();
                }
            }
            ExplosiveObjectState::Exploding => {
                if pulsing.pulse_time.finished() {
                    *state = ExplosiveObjectState::Detonated;
                }
            }
            ExplosiveObjectState::Detonated => {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn debug_environment(mut commands: Commands) {
    // damage zone 1
    let pos = Vec3::new(250.0, 0.0, 0.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                color: Color::rgba(0.4, 0.1, 0.1, 0.3),
                ..Default::default()
            },
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
        .insert(Pulsing {
            pulse_time: Timer::from_seconds(0.5, true),
            damage: 15.0,
        })
        .insert(DamageAreaShape::Sphere { radius: 50.0 });

    // damage zone 2
    let pos = Vec3::new(150.0, 250.0, 0.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(200.0, 100.0)),
                color: Color::rgba(0.4, 0.1, 0.1, 0.3),
                ..Default::default()
            },
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
        .insert(Pulsing {
            pulse_time: Timer::from_seconds(0.2, true),
            damage: 1.0,
        })
        .insert(DamageAreaShape::Cuboid {
            half_extends: Vec3::new(100., 50., 1.),
        });

    // explosive barrel
    let pos = Vec3::new(-150.0, 250.0, 0.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                color: Color::rgb(0.1, 0.1, 0.8),
                ..Default::default()
            },
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
        .insert(ExplosiveObjectState::NotDetonated)
        .insert(Health {
            current: 100.0,
            max: 100.0,
        })
        .insert(Pulsing {
            pulse_time: {
                let mut t = Timer::from_seconds(0.5, true);
                t.pause();
                t
            },
            damage: 100.0,
        })
        .insert(DamageAreaShape::Sphere { radius: 300.0 })
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Sphere { radius: 50.0 })
        .insert(
            CollisionLayers::none()
                .with_group(PhysLayer::Bullets)
                .with_masks(&[PhysLayer::World, PhysLayer::Enemies]),
        );
}
