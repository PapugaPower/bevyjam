use crate::game::player_triggers::PlayerPresenceDetector;
use crate::game::world_interaction::{InteractionDirty, Interactive};
use bevy::prelude::*;
use heron::{CollisionShape, RigidBody};

pub struct DoorUseEvent {
    entity: Entity,
}

pub enum DoorOpenStatus {
    Opened,
    Closed,
}

pub enum DoorLockStatus {
    Locked,
    Unlocked,
}

pub enum DoorRotationSide {
    Right,
    Left,
}

#[derive(Component)]
pub struct Door {
    pub front: Vec3,
    pub open_status: DoorOpenStatus,
    pub lock_status: DoorLockStatus,
    pub rotation_side: DoorRotationSide,
}

#[derive(Component)]
pub struct DoorSensor {
    doors: Vec<Entity>,
}

pub fn door_interaction(
    mut commands: Commands,
    mut door_events: EventWriter<DoorUseEvent>,
    doors_sensors: Query<(Entity, &DoorSensor), With<InteractionDirty>>,
) {
    for (e, sensor) in doors_sensors.iter() {

        // TODO refactor to use events
        commands.entity(e).remove::<InteractionDirty>();

        for door in sensor.doors.iter() {
            debug!("using door: {:?}", door);
            door_events.send(DoorUseEvent { entity: *door });
        }
    }
}

pub fn door_event_processor(
    mut door_events: EventReader<DoorUseEvent>,
    mut doors: Query<(&CollisionShape, &mut Door, &mut Transform)>,
) {
    for door in door_events.iter() {
        if let Ok((collision_shape, mut door, mut transform)) = doors.get_mut(door.entity) {
            let half_door_len = match collision_shape {
                CollisionShape::Cuboid { half_extends, .. } => half_extends[1],
                _ => {
                    debug!("not rectangular door???");
                    0.0
                }
            };
            match door.lock_status {
                DoorLockStatus::Locked => {
                    debug!("door is locked");
                }
                DoorLockStatus::Unlocked => {
                    let (axis_rotation, door_rotation) = match door.open_status {
                        DoorOpenStatus::Opened => {
                            door.open_status = DoorOpenStatus::Closed;
                            match door.rotation_side {
                                DoorRotationSide::Left => (1.0, -1.0),
                                DoorRotationSide::Right => (-1.0, 1.0),
                            }
                        }
                        DoorOpenStatus::Closed => {
                            door.open_status = DoorOpenStatus::Opened;
                            match door.rotation_side {
                                DoorRotationSide::Left => (-1.0, -1.0),
                                DoorRotationSide::Right => (1.0, 1.0),
                            }
                        }
                    };
                    let to_side = Quat::from_rotation_z(axis_rotation * 90.0_f32.to_radians())
                        .mul_vec3(door.front);
                    let axis_pos = transform.translation + to_side * half_door_len;
                    let to_new_pos = Quat::from_rotation_z(-door_rotation * 90.0_f32.to_radians())
                        .mul_vec3(-to_side);
                    transform.translation = axis_pos + to_new_pos * half_door_len;
                    transform.rotate(Quat::from_rotation_z(door_rotation * 90.0_f32.to_radians()));
                }
            }
        }
    }
}

pub fn debug_spawn_door(mut commands: Commands) {
    let door_pos = Vec3::new(437.0, 136.0, 0.0);
    // door
    let door = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 100.0)),
                color: Color::rgb(1.0, 0.2, 1.0),
                ..Default::default()
            },
            transform: Transform::from_translation(door_pos),
            ..Default::default()
        })
        .insert(Door {
            front: -Vec3::X,
            open_status: DoorOpenStatus::Closed,
            lock_status: DoorLockStatus::Unlocked,
            rotation_side: DoorRotationSide::Left,
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(5.0, 50.0, 0.1),
            border_radius: None,
        })
        .id();

    // sensors
    let sensor1_pos = Vec3::new(400.0, 136.0, 0.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                color: Color::rgb(1.0, 0.2, 1.0),
                ..Default::default()
            },
            transform: Transform::from_translation(sensor1_pos),
            ..Default::default()
        })
        .insert(DoorSensor { doors: vec![door] })
        .insert(PlayerPresenceDetector { detected: false })
        .insert(Interactive::default())
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(10.0, 10.0, 0.1),
            border_radius: None,
        });

    let sensor2_pos = Vec3::new(480.0, 136.0, 0.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                color: Color::rgb(1.0, 0.2, 1.0),
                ..Default::default()
            },
            transform: Transform::from_translation(sensor2_pos),
            ..Default::default()
        })
        .insert(DoorSensor { doors: vec![door] })
        .insert(PlayerPresenceDetector { detected: false })
        .insert(Interactive::default())
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(10.0, 10.0, 0.1),
            border_radius: None,
        });
}
