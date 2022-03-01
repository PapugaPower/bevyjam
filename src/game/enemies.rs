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
                    .with_mask(PhysLayer::World),
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

pub fn enemy_despawn(
    mut commands: Commands,
    query_enemy_health: Query<(Entity, &Health), With<Enemy>>,
) {
    for (enemy, health) in query_enemy_health.iter() {
        if health.current <= 0.0 {
            debug!("enemy {:?} despawn", enemy);
            commands.entity(enemy).despawn();
        }
    }
}

pub fn debug_spawn_enemy(mut commands: Commands) {
    let start_x = 0.0;
    let start_y = 500.0;
    let rows = 4;
    let cols = 25;

    for r in 0..rows {
        for c in 0..cols {
            let enemy_pos = Vec3::new(start_x + 50.0 * c as f32, start_y + 100.0 * r as f32, 0.0);
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(50.0, 50.0)),
                        color: Color::rgb(1.0, 0.0, 0.1),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(enemy_pos),
                    ..Default::default()
                })
                .insert(Enemy)
                .insert(Health {
                    max: 69.0,
                    current: 69.0,
                })
                .insert(EnemyAttack {
                    range: 3.0,
                    damage: 5.0,
                })
                .insert(RigidBody::KinematicPositionBased)
                .insert(
                    CollisionLayers::none()
                        .with_group(PhysLayer::Enemies)
                        .with_masks(&[PhysLayer::World, PhysLayer::Player, PhysLayer::Enemies]),
                )
                .insert(CollisionShape::Sphere { radius: 25.0 });
        }
    }
}
