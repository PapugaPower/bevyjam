use crate::game::crosshair::Crosshair;
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;
use bevy::prelude::*;
use heron::{prelude::*, rapier_plugin::PhysicsWorld};

#[derive(Debug, Clone, Copy)]
pub struct DamageEvent {
    pub entity: Entity,
    pub damage: f32,
}

#[derive(Component)]
pub struct LastShootTime {
    pub time: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum AmmoType {
    // pistols, rifles, shotguns
    Projectile,
    // granades, molotov
    Throwable,
    // flamethrower
    Static,
}

#[derive(Component)]
pub struct Weapon {
    pub ammo_type: AmmoType,
    pub damage: f32,
    // this is 1 / real_fire_rate
    pub fire_rate: f32,
    // speed of projectile or pulse rate of static
    pub projectile_speed: f32,
    // in seconds
    pub projectile_life_time: f32,
    // in degrees
    pub spread: f32,
    // bullets will be spread equally over `spread`
    pub num_bullets_per_shot: u32,
    // how far projectiles should spawn from player
    pub projectile_spawn_offset: f32,
    // for now used to define radius for Throwable and Static
    // does nothing for Projectile
    pub radius_of_effect: f32,
}

#[derive(Component)]
pub struct Projectile {
    damage: f32,
    life_time: Timer,
    direction: Vec3,
    speed: f32,
}

#[derive(Component)]
pub struct Throwable {
    damage: f32,
    radius: f32,
    life_time: Timer,
}

#[derive(Component)]
pub struct Static {
    damage: f32,
    radius: f32,
    life_time: Timer,
    pulse_time: Timer,
}

pub fn player_shoot(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut query_player: Query<(&Transform, &Weapon, &mut LastShootTime), With<Player>>,
    mut query_cross: Query<&Transform, With<Crosshair>>,
) {
    // TODO handle input
    if keys.pressed(KeyCode::Space) {
        let (player_transform, weapon, mut last_shoot) = query_player.single_mut();
        let cross_transform = query_cross.single_mut();
        let shoot_dir = (cross_transform.translation - player_transform.translation).normalize();
        let spawn_transform = {
            let mut pt = *player_transform;
            pt.translation += shoot_dir * weapon.projectile_spawn_offset;
            pt
        };

        let now = time.time_since_startup().as_secs_f32();
        if last_shoot.time + weapon.fire_rate <= now {
            // if only one bullet we don't care about spread
            let spread_step = if weapon.num_bullets_per_shot <= 1 {
                0.0
            } else {
                weapon.spread / (weapon.num_bullets_per_shot - 1) as f32
            };
            for i in 0..weapon.num_bullets_per_shot {
                let shoot_dir = Quat::from_rotation_z((spread_step * i as f32).to_radians())
                    * Quat::from_rotation_z((-weapon.spread / 2.0).to_radians())
                    * shoot_dir;
                match weapon.ammo_type {
                    AmmoType::Projectile => {
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.2, 0.2)),
                                    color: Color::rgb(0.8, 0.5, 0.5),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Projectile {
                                damage: weapon.damage,
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                                direction: shoot_dir,
                                speed: weapon.projectile_speed,
                            });
                    }
                    AmmoType::Throwable => {
                        let throw_dir = shoot_dir * weapon.projectile_speed;
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.4, 0.4)),
                                    color: Color::rgb(0.9, 0.9, 0.2),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Throwable {
                                damage: weapon.damage,
                                radius: weapon.radius_of_effect,
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                            })
                            .insert(RigidBody::Dynamic)
                            .insert(CollisionShape::Sphere { radius: 0.2 })
                            .insert(Velocity::from_linear(throw_dir))
                            .insert(PhysicMaterial {
                                friction: 1.0,
                                density: 10.0,
                                ..Default::default()
                            })
                            .insert(
                                CollisionLayers::none()
                                    .with_group(PhysLayer::Bullets)
                                    .with_masks(&[PhysLayer::World, PhysLayer::Enemies]),
                            );
                    }
                    AmmoType::Static => {
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.5, 0.5)),
                                    color: Color::rgb(0.2, 0.5, 0.9),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Static {
                                damage: weapon.damage,
                                radius: weapon.radius_of_effect,
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                                pulse_time: Timer::from_seconds(weapon.projectile_speed, true),
                            })
                            .insert(RigidBody::Sensor)
                            .insert(CollisionShape::Sphere { radius: 0.25 })
                            .insert(
                                CollisionLayers::none()
                                    .with_group(PhysLayer::Bullets)
                                    .with_masks(&[PhysLayer::World, PhysLayer::Enemies]),
                            );
                    }
                }
            }
            last_shoot.time = now;
        }
    }
}

pub fn projectiles_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    physics_world: PhysicsWorld,
    query_player: Query<Entity, With<Player>>,
    mut query_projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
    let player_entity = query_player.single();
    for (entity, mut transform, mut projectile) in query_projectiles.iter_mut() {
        // life time check
        projectile.life_time.tick(time.delta());
        if projectile.life_time.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        // collision check
        let ray_cast = physics_world.ray_cast(transform.translation, projectile.direction, true);
        let bullet_travel = projectile.speed * time.delta_seconds();
        if let Some(collision) = ray_cast {
            // we need to manually check for collision with different entities
            // TODO refactor maybe
            if collision.entity == player_entity {
            } else if (collision.collision_point - transform.translation).length() <= bullet_travel
            {
                damage_event.send(DamageEvent {
                    entity: collision.entity,
                    damage: projectile.damage,
                });
                commands.entity(entity).despawn();
            }
            // debug collision point
            // commands.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::GREEN,
            //         custom_size: Some(Vec2::new(10., 10.)),
            //         ..Default::default()
            //     },
            //     transform: Transform::from_translation(collision.collision_point),
            //     ..Default::default()
            // });
        }
        transform.translation += projectile.direction * bullet_travel;
    }
}

pub fn throwable_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    physics_world: PhysicsWorld,
    mut query_throwable: Query<(Entity, &Transform, &mut Throwable)>,
) {
    for (entity, transform, mut throwable) in query_throwable.iter_mut() {
        // life time check
        throwable.life_time.tick(time.delta());
        if throwable.life_time.finished() {
            // collision check
            physics_world.intersections_with_shape(
                &CollisionShape::Sphere { radius: throwable.radius },
                transform.translation,
                transform.rotation,
                CollisionLayers::all::<PhysLayer>(),
                &mut |e| {
                    damage_event.send(DamageEvent {
                        entity: e,
                        damage: throwable.damage,
                    });
                    true
                },
            );
            commands.entity(entity).despawn();
        }
    }
}

pub fn static_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    physics_world: PhysicsWorld,
    mut query_static: Query<(Entity, &Transform, &mut Static)>,
) {
    for (entity, transform, mut s) in query_static.iter_mut() {
        // collision check
        s.pulse_time.tick(time.delta());
        if s.pulse_time.finished() {
            physics_world.intersections_with_shape(
                &CollisionShape::Sphere { radius: s.radius },
                transform.translation,
                transform.rotation,
                CollisionLayers::all::<PhysLayer>(),
                &mut |e| {
                    damage_event.send(DamageEvent {
                        entity: e,
                        damage: s.damage,
                    });
                    true
                },
            );
        }
        // life time check
        s.life_time.tick(time.delta());
        if s.life_time.finished() {
            commands.entity(entity).despawn();
            continue;
        }
    }
}
