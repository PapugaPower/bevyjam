use crate::game::crosshair::Crosshair;
use crate::game::player::Player;
use bevy::prelude::*;
use heron::prelude::*;

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
}

// Physical layers of the game
#[derive(PhysicsLayer)]
pub enum Layer {
    World,
    Player,
    Enemies,
    Bullets,
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
                let bullet_vel = bullet_dir * weapon.bullet_speed;

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
                        life_time: Timer::from_seconds(1.0, false),
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(CollisionShape::Sphere { radius: 0.1 })
                    .insert(Velocity::from_linear(bullet_vel))
                    .insert(PhysicMaterial {
                        friction: 1.0,
                        density: 10.0,
                        ..Default::default()
                    })
                    .insert(
                        CollisionLayers::none()
                            .with_group(Layer::Bullets)
                            .with_mask(Layer::World),
                    );
            }
            last_shoot.time = now;
        }
    }
}

pub fn bullets_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in query.iter_mut() {
        bullet.life_time.tick(time.delta());
        if bullet.life_time.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn tear_down_bullets(mut commands: Commands, query: Query<Entity, With<Bullet>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
