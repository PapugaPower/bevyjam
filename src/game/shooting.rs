use crate::game::crosshair::Crosshair;
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;
use bevy::prelude::*;
use heron::{prelude::*, rapier_plugin::PhysicsWorld};

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
    Periodic,
}

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct Throwable;

#[derive(Component)]
pub struct Periodic;

#[derive(Component)]
pub struct Weapon {
    pub ammo_type: AmmoType,
    // this is 1 / real_fire_rate
    pub fire_rate: f32,
    pub projectile_speed: f32,
    // in seconds
    pub projectile_life_time: f32,
    // in degrees
    pub spread: f32,
    // bullets will be spread equally over `spread`
    pub num_bullets_per_shot: u32,
    // how far projectiles should spawn from player
    pub projectile_spawn_offset: f32,
}

#[derive(Component)]
pub struct Ammo {
    life_time: Timer,
    direction: Vec3,
    speed: f32,
}

pub fn player_shoot(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut query_player: Query<(&Transform, &Weapon, &mut LastShootTime), With<Player>>,
    mut query_cross: Query<&Transform, With<Crosshair>>,
) {
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
            let spread_step = weapon.spread / (weapon.num_bullets_per_shot - 1) as f32;
            for i in 0..weapon.num_bullets_per_shot {
                let bullet_dir = Quat::from_rotation_z((spread_step * i as f32).to_radians())
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
                            .insert(Ammo {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                                direction: bullet_dir,
                                speed: weapon.projectile_speed,
                            });
                    }
                    AmmoType::Throwable => {
                        let bullet_vel = bullet_dir * weapon.projectile_speed;
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
                            .insert(Ammo {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                                direction: bullet_dir,
                                speed: weapon.projectile_speed,
                            })
                            .insert(RigidBody::Dynamic)
                            .insert(CollisionShape::Sphere { radius: 0.2 })
                            .insert(Velocity::from_linear(bullet_vel))
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
                    AmmoType::Periodic => {
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
                            .insert(Ammo {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                                direction: bullet_dir,
                                speed: weapon.projectile_speed,
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

pub fn projectiles_collision(
    mut commands: Commands,
    time: Res<Time>,
    physics_world: PhysicsWorld,
    query_player: Query<Entity, With<Player>>,
    mut query_bullets: Query<(Entity, &mut Transform, &Ammo), Without<CollisionShape>>,
) {
    let player_entity = query_player.single();
    for (entity, mut transform, bullet) in query_bullets.iter_mut() {
        let ray_cast = physics_world.ray_cast(transform.translation, bullet.direction, true);
        let bullet_travel = bullet.speed * time.delta_seconds();
        if let Some(collision) = ray_cast {
            // we need to manually check for collision with different entities
            // TODO refactor maybe
            if collision.entity == player_entity {
            } else if (collision.collision_point - transform.translation).length() <= bullet_travel
            {
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
        transform.translation += bullet.direction * bullet_travel;
    }
}

pub fn ammo_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query_projectiles: Query<(Entity, &mut Ammo)>,
) {
    for (entity, mut ammo) in query_projectiles.iter_mut() {
        ammo.life_time.tick(time.delta());
        if ammo.life_time.finished() {
            commands.entity(entity).despawn();
        }
    }
}
