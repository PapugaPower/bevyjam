use std::time::Duration;

use crate::game::damage::{DamageEvent, DamageSource, Health};
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;
use crate::util::WorldCursor;
use bevy::{prelude::*, transform};
use heron::rapier_plugin::{PhysicsWorld, ShapeCastCollisionType};
use heron::{CollisionLayers, CollisionShape, RigidBody};
use bevy_prototype_debug_lines::*;
use rand::prelude::*;

use super::collider::Wall;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyAttack {
    pub range: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct EnemyTargetPos(Vec2);

#[derive(Component)]
pub struct EnemyTargetEntity(Entity);

#[derive(Component)]
pub struct EnemyTargetLastSeen(Timer);

#[derive(Component)]
pub struct EnemyTargetScanning(f64, bool, bool);

#[derive(Component)]
pub struct EnemyWallAvoidance(f32);

impl EnemyTargetScanning {
    fn new(secs_since_startup: f64) -> EnemyTargetScanning {
        let mut rng = rand::thread_rng();
        EnemyTargetScanning(secs_since_startup, rng.gen(), rng.gen())
    }
}

// #[derive(Component)]
// pub struct EnemyWave {
//     pub timer: Timer,
//     pub number: u32,
//     pub radius: f32,
//     pub despawn_radius: f32,
// }

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
    target_pos: EnemyTargetPos,
    target_last_seen: EnemyTargetLastSeen,
    scanning: EnemyTargetScanning,
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
            target_pos: EnemyTargetPos(Vec2::new(200., 0.)),
            target_last_seen: EnemyTargetLastSeen(Timer::new(Duration::from_secs(1), false)),
            scanning: EnemyTargetScanning::new(0.0),
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

/*
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
*/

pub fn enemy_debug_lines(
    mut lines: ResMut<DebugLines>,
    q: Query<(&Transform, &EnemyTargetPos), With<Enemy>>,
) {
    for (xf, tgt) in q.iter() {
        lines.line(xf.translation, xf.translation + xf.local_x() * 200.0, 0.0);
        lines.line_colored(xf.translation, tgt.0.extend(xf.translation.z), 0.0, Color::GREEN);
    }
}

pub fn enemy_target_entity(
    mut q_tgt: Query<(&mut EnemyTargetPos, &EnemyTargetEntity)>,
    q_xf: Query<&Transform>,
) {
    for (mut pos, e) in q_tgt.iter_mut() {
        if let Ok(xf) = q_xf.get(e.0) {
            pos.0 = xf.translation.truncate();
        }
    }
}

pub fn enemy_target_scan(
    mut q: Query<(&Transform, &mut EnemyTargetPos, &EnemyTargetScanning)>,
    t: Res<Time>,
) {
    for (xf, mut pos, e) in q.iter_mut() {
        let direction = pos.0.extend(xf.translation.z) - xf.translation;
        let mut angle = (t.seconds_since_startup() - e.0).sin() as f32;
        if e.1 {
            angle -= angle;
        }
        if e.2 {
            angle += 0.1;
        } else {
            angle -= 0.1;
        }
        let rot = Quat::from_rotation_z(angle * t.delta_seconds());
        let direction = rot * direction;
        pos.0 = (xf.translation + direction).truncate();
    }
}

pub fn enemy_die(
    mut commands: Commands,
    query_enemy_health: Query<(Entity, &Health), With<Enemy>>,
) {
    for (e, health) in query_enemy_health.iter() {
        if health.current <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

pub fn enemy_rotation(
    mut q: Query<(&mut Transform, &EnemyTargetPos, Option<&EnemyWallAvoidance>)>,
    t: Res<Time>,
) {
    const P: f32 = 2.0;

    for (mut xf, target, avoid) in q.iter_mut() {
        let mut avoid = avoid.map(|x| x.0).unwrap_or(0.0);
        if avoid.is_nan() {
            avoid = 0.0;
        }
        let dir_fwd = xf.local_x().truncate();
        let dir_tgt = target.0 - xf.translation.truncate();
        let angle = dir_fwd.angle_between(dir_tgt);
        let rotation = Quat::from_rotation_z((dbg!(avoid) + angle * P) * t.delta_seconds());
        xf.rotate(rotation);
    }
}

pub fn enemy_line_of_sight(
    mut commands: Commands,
    mut q_enemy: Query<(Entity, &Transform, &mut EnemyTargetLastSeen, &mut EnemyTargetPos, Option<&EnemyTargetScanning>)>,
    q_player: Query<Entity, With<Player>>,
    q_wall: Query<Entity, With<Wall>>,
    physics_world: PhysicsWorld,
    t: Res<Time>,
    mut lines: ResMut<DebugLines>,
) {
    for (enemy, enemy_xf, mut lastseen, mut pos, scanning) in q_enemy.iter_mut() {
        lastseen.0.tick(t.delta());
        let raycast = physics_world.ray_cast_with_filter(
            enemy_xf.translation, enemy_xf.local_x() * 2000.0, true,
            CollisionLayers::none()
                .with_group(PhysLayer::Enemies)
                .with_masks(&[PhysLayer::World, PhysLayer::Player]),
            |_entitity| true,
        );
        if let Some(coll) = raycast {
            /*
            let distance = (coll.collision_point - enemy_xf.translation).length();
            if q_wall.get(coll.entity).is_ok() {
                let cross = coll.normal.cross(Vec3::Z);
                let dir = -(cross * cross.dot(enemy_xf.local_x() + Vec2::splat(0.001).extend(0.0))).normalize();
                let magn = coll.normal.dot(enemy_xf.local_x());
                let dfalloff = (-distance * 0.01).exp();
                // let dfalloff = 1.0 / (distance * 0.01).exp();
                // dbg!(dfalloff);
                // let angle = enemy_xf.local_x().truncate().angle_between(dir.truncate());
                lines.line_colored(coll.collision_point, coll.collision_point + dir * magn * dfalloff * 100.0, 0.0, Color::RED);
                // commands.entity(enemy).insert(EnemyWallAvoidance(dfalloff * magn * angle));
                if !magn.is_nan() && !dfalloff.is_nan() {
                    pos.0 += dir.truncate() * magn * dfalloff;
                }
            } else {
                // commands.entity(enemy).remove::<EnemyWallAvoidance>();
            }
            */
            if q_player.get(coll.entity).is_ok() {
                commands.entity(enemy)
                    .remove::<EnemyTargetScanning>()
                    .insert(EnemyTargetEntity(coll.entity));
                lastseen.0.reset();
            } else {
                if lastseen.0.finished() && scanning.is_none() {
                    commands.entity(enemy)
                        .insert(EnemyTargetScanning::new(t.seconds_since_startup()))
                        .remove::<EnemyTargetEntity>();
                }
            }
        } else {
            if lastseen.0.finished() && scanning.is_none() {
                commands.entity(enemy)
                    .insert(EnemyTargetScanning::new(t.seconds_since_startup()))
                    .remove::<EnemyTargetEntity>();
            }
        }

    }
}

pub fn enemy_walk(
    mut q: Query<(&mut Transform), With<Enemy>>,
    t: Res<Time>,
) {
    for mut xf in q.iter_mut() {
        let mv = xf.local_x() * 69.69 * t.delta_seconds();
        xf.translation += mv;
    }
}

pub fn enemy_target_player(
    mut q_tgt: Query<&mut EnemyTargetEntity>,
    q_player: Query<Entity, With<Player>>,
) {
    for mut enemy in q_tgt.iter_mut() {
        enemy.0 = q_player.single();
    }
}

pub fn debug_enemy_spawn(
    mut commands: Commands,
    crs: Res<WorldCursor>,
    mb: Res<Input<MouseButton>>,
) {
    if mb.just_pressed(MouseButton::Middle) {
        commands.spawn_bundle(EnemyBundle::default())
            .insert(Transform::from_translation(crs.0.extend(0.0)));
    }
}
