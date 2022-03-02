use crate::game::damage::{DamageEvent, DamageSource, Health};
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;
use bevy::prelude::*;
use heron::rapier_plugin::{PhysicsWorld, ShapeCastCollisionType};
use heron::{CollisionLayers, CollisionShape, RigidBody};

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyAttack {
    pub range: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct EnemyWave {
    pub timer: Timer,
    pub number: u32,
    pub radius: f32,
    pub despawn_radius: f32,
}

/// Bevy Bundle for easy spawning of entities
#[derive(Bundle)]
pub struct EnemyBundle {
    // include base bundle for rendering
    #[bundle]
    sprite: SpriteBundle,
    // cleanup marker
    cleanup: super::GameCleanup,
    // our game behaviors
    enemy: Enemy,
    attack: EnemyAttack,
    health: Health,
    // physics
    rigidbody: RigidBody,
    phys_layers: CollisionLayers,
    phys_shape: CollisionShape,
}

impl Default for EnemyBundle {
    fn default() -> EnemyBundle {
        EnemyBundle {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    color: Color::rgb(1.0, 0.0, 0.1),
                    ..Default::default()
                },
                ..Default::default()
            },
            cleanup: super::GameCleanup,
            enemy: Enemy,
            attack: EnemyAttack {
                range: 3.0,
                damage: 5.0,
            },
            health: Health {
                max: 69.0,
                current: 69.0,
            },
            rigidbody: RigidBody::KinematicPositionBased,
            phys_layers: CollisionLayers::none()
                .with_group(PhysLayer::Enemies)
                .with_masks(&[PhysLayer::World, PhysLayer::Player, PhysLayer::Enemies]),
            phys_shape: CollisionShape::Sphere { radius: 25.0 }
        }
    }
}

pub fn enemy_controller(
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    physics_world: PhysicsWorld,
    mut query: QuerySet<(
        QueryState<(Entity, &Transform), With<Player>>,
        QueryState<(&EnemyAttack, &CollisionShape, &mut Transform), With<Enemy>>,
    )>,
) {
    let (player, player_position) = {
        let (p, t) = query.q0().single();
        (p, t.translation)
    };

    for (attack, collision_shape, mut transform) in query.q1().iter_mut() {
        // damage check
        let direction = player_position - transform.translation;
        let distance = direction.length();
        if distance <= attack.range {
            damage_event.send(DamageEvent {
                entity: player,
                source: DamageSource::Enemy,
                damage: attack.damage,
            });
        }

        // movement
        let direction = direction.normalize();
        let angle = direction.y.atan2(direction.x);
        transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

        let mut move_vector = direction * 100.0 * time.delta_seconds();

        for iter in 0..4 {
            let hit = physics_world.shape_cast_with_filter(
                collision_shape,
                transform.translation,
                transform.rotation,
                move_vector,
                CollisionLayers::none()
                    .with_group(PhysLayer::Enemies)
                    .with_masks(&[PhysLayer::World]),
                |_entitity| true,
            );

            if let Some(collision) = hit {
                if let ShapeCastCollisionType::Collided(info) = collision.collision_type {
                    if iter == 3 {
                        move_vector = Vec3::ZERO;
                        break;
                    }
                    let cross = info.self_normal.cross(Vec3::Z);
                    move_vector = cross * cross.dot(move_vector);
                } else if let ShapeCastCollisionType::AlreadyPenetrating = collision.collision_type
                {
                    // TODO: If there are collision artifacts resulting in this being called,
                    // implement a "last trusted position" local resource to be reverted to.
                }
            }
        }
        transform.translation += move_vector;
    }
}

pub fn enemy_spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut EnemyWave), With<Player>>,
) {
    let (transform, mut wave) = query.single_mut();
    wave.timer.tick(time.delta());
    if wave.timer.finished() {
        for i in 0..wave.number {
            let pos = transform.translation
                + Quat::from_rotation_z(360.0 / wave.number as f32 * i as f32)
                    .mul_vec3(Vec3::Y * wave.radius);
            commands.spawn_bundle(EnemyBundle::default())
                .insert(Transform::from_translation(pos));
        }
    }
}

pub fn enemy_despawn(
    mut commands: Commands,
    query_enemy_health: Query<(Entity, &Health, &Transform), With<Enemy>>,
    query_player: Query<(&Transform, &EnemyWave), With<Player>>,
) {
    let (player_transform, wave) = query_player.single();
    for (enemy, health, transform) in query_enemy_health.iter() {
        if (player_transform.translation - transform.translation).length() > wave.despawn_radius {
            commands.entity(enemy).despawn();
        }
        if health.current <= 0.0 {
            debug!("enemy {:?} despawn", enemy);
            commands.entity(enemy).despawn();
        }
    }
}
