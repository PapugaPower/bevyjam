use crate::game::crosshair::Crosshair;
use crate::game::player::Player;
use bevy::prelude::*;
use heron::rapier_plugin::PhysicsWorld;

#[derive(Component)]
pub struct LastShootTime {
    pub time: f32,
}

#[derive(Component)]
pub struct Weapon {
    // this is 1 / real_fire_rate
    pub fire_rate: f32,
    pub bullet_speed: f32,
    // in degrees
    pub spread: f32,
    // bullets will be spread equally over `spread`
    pub num_bullets_per_shot: u32,
}

#[derive(Component)]
pub struct Bullet {
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

        let now = time.time_since_startup().as_secs_f32();
        if last_shoot.time + weapon.fire_rate <= now {
            let spread_step = weapon.spread / (weapon.num_bullets_per_shot - 1) as f32;
            for i in 0..weapon.num_bullets_per_shot {
                let bullet_dir = Quat::from_rotation_z((spread_step * i as f32).to_radians())
                    * Quat::from_rotation_z((-weapon.spread / 2.0).to_radians())
                    * shoot_dir;
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(0.2, 0.2)),
                            color: Color::rgb(0.8, 0.5, 0.5),
                            ..Default::default()
                        },
                        transform: *player_transform,
                        ..Default::default()
                    })
                    .insert(Bullet {
                        life_time: Timer::from_seconds(5.0, false),
                        direction: bullet_dir,
                        speed: weapon.bullet_speed,
                    });
            }
            last_shoot.time = now;
        }
    }
}

pub fn bullets_collision(
    mut commands: Commands,
    time: Res<Time>,
    physics_world: PhysicsWorld,
    mut query_bullets: Query<(Entity, &mut Transform, &Bullet)>,
) {
    for (entity, mut transform, bullet) in query_bullets.iter_mut() {
        let ray_cast = physics_world.ray_cast(transform.translation, bullet.direction, true);
        let bullet_travel = bullet.speed * time.delta_seconds();
        if let Some(collision) = ray_cast {
            if (collision.collision_point - transform.translation).length() <= bullet_travel {
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

pub fn bullets_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query_bullets: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in query_bullets.iter_mut() {
        bullet.life_time.tick(time.delta());
        if bullet.life_time.finished() {
            commands.entity(entity).despawn();
        }
    }
}
