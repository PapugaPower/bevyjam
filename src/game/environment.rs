use crate::game::damage::{DamageAreaShape, Health, Pulsing};
use crate::game::player::Player;
use crate::game::phys_layers::PhysLayer;
use bevy::prelude::*;
use heron::prelude::*;

pub mod barrel;
pub mod door;
pub mod medkit;

pub struct InterationEvent {
    entity: Entity,
}

#[derive(Component)]
pub struct Trigger {
    pub player_detected: bool,
    pub entities: Vec<Entity>,
}

#[derive(Component)]
pub struct TriggerTimeout {
    pub timeout: Timer,
}

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

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct MultiUse {
    pub remaining: i32,
}

pub fn trigger_player_detection(
    mut collision_events: EventReader<CollisionEvent>,
    query_player: Query<Entity, With<Player>>,
    mut query_triggers: Query<&mut Trigger>,
) {
    let player = query_player.single();
    for event in collision_events.iter() {
            match event {
            CollisionEvent::Started(e1, e2) => {
                let trigger = if e1.rigid_body_entity() == player {
                    e2
                } else {
                    e1
                };
                if let Ok(mut trigger) = query_triggers.get_mut(trigger.rigid_body_entity()) {
                    trigger.player_detected = true;
                }
                // println!("Collision started between {:?} and {:?}", e1, e2)
            }
            CollisionEvent::Stopped(e1, e2) => {
                let trigger = if e1.rigid_body_entity() == player {
                    e2
                } else {
                    e1
                };
                if let Ok(mut trigger) = query_triggers.get_mut(trigger.rigid_body_entity()) {
                    trigger.player_detected = false;
                }
                // println!("Collision stopped between {:?} and {:?}", e1, e2)
            }
        }
    }
}

pub fn trigger_interaction(
    input: Res<Input<KeyCode>>,
    mut interation_events: EventWriter<InterationEvent>,
    query_triggers: Query<(&Trigger, Option<&TriggerTimeout>)>,
) {
    if input.just_pressed(KeyCode::E) {
        for (trigger, timeout) in query_triggers.iter() {
            if let Some(timeout) = timeout {
                if !timeout.timeout.finished() {
                    continue;
                }
            }
            if trigger.player_detected {
                for entity in trigger.entities.iter() {
                    interation_events.send(InterationEvent { entity: *entity });
                }
            }
        }
    };
}

pub fn triggir_timeout_process(time: Res<Time>, mut query_triggers: Query<&mut TriggerTimeout>) {
    for mut timeout in query_triggers.iter_mut() {
        timeout.timeout.tick(time.delta());
    }
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

pub fn debug_environment_damage_zones(mut commands: Commands) {
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
                .with_group(PhysLayer::World)
                .with_masks(&[PhysLayer::Bullets, PhysLayer::Player, PhysLayer::Enemies]),
        );
}
