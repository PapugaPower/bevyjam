use std::time::Duration;

use crate::editor::collider::EditableCollider;
use crate::game::animations::{Animation, AnimationBundle, EnemyAnimations};
use crate::game::damage::{DamageEvent, DamageSource, Health};
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;
use crate::util::WorldCursor;
use bevy::prelude::Transform;
use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use heron::rapier_plugin::PhysicsWorld;
use heron::{CollisionLayers, CollisionShape, RigidBody};
use rand::prelude::*;

use super::collider::{SpawnZone, Wall};

/// Parameters for controlling the spawning of enemies
pub struct EnemyConfig {
    /// no more enemies will be spawned if there are this many already
    pub max_count: u32,
    /// use the fast timer below this, slow timer above this
    pub min_count: u32,
    /// min distance from player
    pub min_distance: f32,
    /// despawn above this distance
    pub max_distance: f32,
    /// current count
    count: u32,
    pub timer_fast: Timer,
    pub timer_slow: Timer,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        EnemyConfig {
            max_count: 150,
            min_count: 8,
            count: 0,
            min_distance: 800.0,
            max_distance: 1600.0,
            timer_fast: Timer::new(Duration::from_secs_f32(0.8), true),
            timer_slow: Timer::new(Duration::from_secs_f32(1.0), true),
        }
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyAttack {
    pub range: f32,
    pub damage: f32,
    pub timer: Timer,
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
pub struct EnemyWallAvoidance(Quat);

#[derive(Component)]
/// Used to detect that an enemy keeps moving, and despawn if stuck in one place
pub struct EnemyStuckDetect {
    /// saved position
    pos: Vec2,
    /// if the enemy goes more than this far away from `pos` update `pos`
    radius: f32,
    /// reset when `pos` gets updated; despawn if timer elapsed
    timer: Timer,
}

impl Default for EnemyStuckDetect {
    fn default() -> Self {
        EnemyStuckDetect {
            pos: Vec2::ZERO,
            radius: 20.0,
            timer: Timer::new(Duration::from_secs_f32(1.0), false),
        }
    }
}

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
    animation: AnimationBundle,
    // our game behaviors
    enemy: Enemy,
    attack: EnemyAttack,
    target_pos: EnemyTargetPos,
    target_last_seen: EnemyTargetLastSeen,
    scanning: EnemyTargetScanning,
    stuck: EnemyStuckDetect,
    health: Health,
    // physics
    rigidbody: RigidBody,
    phys_layers: CollisionLayers,
    phys_shape: CollisionShape,
}

impl EnemyBundle {
    pub fn from_animation_transform_size(
        animation: &Animation,
        transform: Transform,
        size: Option<Vec2>,
    ) -> EnemyBundle {
        EnemyBundle {
            animation: AnimationBundle::from_animation_transform_size(animation, transform, size),
            enemy: Enemy,
            attack: EnemyAttack {
                range: 70.0,
                damage: 20.0,
                timer: Timer::from_seconds(0.5, true),
            },
            health: Health {
                max: 69.0,
                current: 69.0,
            },
            target_pos: EnemyTargetPos(Vec2::new(200., 0.)),
            target_last_seen: EnemyTargetLastSeen(Timer::new(Duration::from_secs(1), false)),
            scanning: EnemyTargetScanning::new(0.0),
            stuck: EnemyStuckDetect::default(),
            rigidbody: RigidBody::KinematicPositionBased,
            phys_layers: CollisionLayers::none()
                .with_group(PhysLayer::Enemies)
                .with_masks(&[PhysLayer::World, PhysLayer::Player, PhysLayer::Enemies]),
            phys_shape: CollisionShape::Sphere { radius: 25.0 },
        }
    }
}

/*
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
*/

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

#[allow(clippy::complexity)]
pub fn enemy_damage(
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    mut query: QuerySet<(
        QueryState<(Entity, &Transform), With<Player>>,
        QueryState<(&mut EnemyAttack, &Transform), With<Enemy>>,
    )>,
) {
    let (player, player_position) = {
        let (p, t) = query.q0().single();
        (p, t.translation)
    };

    for (mut attack, transform) in query.q1().iter_mut() {
        // damage check
        let direction = player_position - transform.translation;
        let distance = direction.length();
        if distance <= attack.range {
            attack.timer.tick(time.delta());
            if attack.timer.finished() {
                damage_event.send(DamageEvent {
                    entity: player,
                    source: DamageSource::Enemy,
                    damage: attack.damage,
                });
            }
        }
    }
}

pub fn enemy_debug_lines(
    mut lines: ResMut<DebugLines>,
    q: Query<(&Transform, &EnemyTargetPos), With<Enemy>>,
) {
    for (xf, tgt) in q.iter() {
        lines.line(xf.translation, xf.translation + xf.local_x() * 200.0, 0.0);
        lines.line_colored(
            xf.translation,
            tgt.0.extend(xf.translation.z),
            0.0,
            Color::GREEN,
        );
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
    mut cfg: ResMut<EnemyConfig>,
    query_enemy_health: Query<(Entity, &Health), With<Enemy>>,
) {
    for (e, health) in query_enemy_health.iter() {
        if health.current <= 0.0 {
            commands.entity(e).despawn();
            cfg.count -= 1;
        }
    }
}

pub fn enemy_despawn_far(
    mut commands: Commands,
    mut cfg: ResMut<EnemyConfig>,
    q_enemy: Query<(Entity, &GlobalTransform), With<Enemy>>,
    q_player: Query<&GlobalTransform, With<Player>>,
) {
    let player_pos = q_player.single().translation.truncate();
    for (e, enemy_xf) in q_enemy.iter() {
        let enemy_pos = enemy_xf.translation.truncate();
        if enemy_pos.distance(player_pos) > cfg.max_distance {
            commands.entity(e).despawn();
            cfg.count -= 1;
        }
    }
}

pub fn enemy_despawn_stuck(
    mut commands: Commands,
    mut cfg: ResMut<EnemyConfig>,
    mut q_enemy: Query<(Entity, &mut EnemyStuckDetect, &GlobalTransform)>,
    t: Res<Time>,
) {
    for (e, mut stuck, xf) in q_enemy.iter_mut() {
        stuck.timer.tick(t.delta());
        let enemy_pos = xf.translation.truncate();
        if enemy_pos.distance(stuck.pos) > stuck.radius {
            stuck.pos = enemy_pos;
            stuck.timer.reset();
        }
        if stuck.timer.finished() {
            commands.entity(e).despawn();
            cfg.count -= 1;
        }
    }
}

pub fn enemy_rotation(
    mut q: Query<(&mut Transform, &EnemyTargetPos)>,
    t: Res<Time>,
    mut commands: Commands,
) {
    const P: f32 = 2.0;

    for (mut xf, target) in q.iter_mut() {
        // let mut avoid = avoid.map(|x| x.0).unwrap_or(0.0);
        // if avoid.is_nan() {
        //     avoid = 0.0;
        // }
        let dir_fwd = xf.local_x().truncate();
        let dir_tgt = target.0 - xf.translation.truncate();
        let angle = dir_fwd.angle_between(dir_tgt);
        // let rotation = Quat::from_rotation_z((avoid + angle * P) * t.delta_seconds());
        let rotation = Quat::from_rotation_z((angle * P) * t.delta_seconds());
        xf.rotate(rotation);
    }
}

pub fn enemy_player_search(
    mut commands: Commands,

    mut q_enemy: Query<(
        Entity,
        &Transform,
        &mut EnemyTargetLastSeen,
        &mut EnemyTargetPos,
        Option<&EnemyTargetScanning>,
    )>,
    q_player: Query<Entity, With<Player>>,
    q_wall: Query<Entity, With<Wall>>,
    physics_world: PhysicsWorld,
    t: Res<Time>,
    mut lines: ResMut<DebugLines>,
) {
    for (enemy, enemy_xf, mut lastseen, mut pos, scanning) in q_enemy.iter_mut() {
        lastseen.0.tick(t.delta());
        let raycast = physics_world.ray_cast_with_filter(
            enemy_xf.translation,
            enemy_xf.local_x() * 200.0,
            true,
            CollisionLayers::none()
                .with_group(PhysLayer::Enemies)
                .with_masks(&[PhysLayer::World, PhysLayer::Player]),
            |_entitity| true,
        );
        if let Some(coll) = raycast {
            let direction = coll.collision_point - enemy_xf.translation;
            let distance = (coll.collision_point - enemy_xf.translation).length();
            if q_wall.get(coll.entity).is_ok() {
                let cross = coll.normal.cross(Vec3::Z);
                let deflection = cross * cross.dot(direction);
                let rot = Quat::from_rotation_arc(
                    direction.normalize(),
                    (deflection + direction).normalize(),
                );
                lines.line_colored(
                    coll.collision_point,
                    coll.collision_point + deflection,
                    0.0,
                    Color::RED,
                );
                commands.entity(enemy).insert(EnemyWallAvoidance(rot));
            }

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
                commands
                    .entity(enemy)
                    .remove::<EnemyTargetScanning>()
                    .insert(EnemyTargetEntity(coll.entity));
                lastseen.0.reset();
            } else {
                if lastseen.0.finished() && scanning.is_none() {
                    commands
                        .entity(enemy)
                        .insert(EnemyTargetScanning::new(t.seconds_since_startup()))
                        .remove::<EnemyTargetEntity>();
                }
            }
        } else {
            if lastseen.0.finished() && scanning.is_none() {
                commands
                    .entity(enemy)
                    .insert(EnemyTargetScanning::new(t.seconds_since_startup()))
                    .remove::<EnemyTargetEntity>();
            }
        }
    }
}

#[allow(clippy::complexity)]
pub fn enemy_walk(
    mut q_set: QuerySet<(
        QueryState<&mut Transform, With<Enemy>>,
        QueryState<&Transform, With<Player>>,
    )>,
    t: Res<Time>,
    physics_world: PhysicsWorld,
) {
    let mut player_pos = Vec3::ZERO;
    for p in q_set.q1().iter() {
        player_pos = p.translation;
    }

    for mut xf in q_set.q0().iter_mut() {
        let to_player: Vec3 = player_pos - xf.translation;
        if to_player.length_squared() < 45.0 * 45.0 {
            let direction = player_pos - xf.translation;
            let angle = direction.y.atan2(direction.x);
            xf.rotation = Quat::from_axis_angle(Vec3::Z, angle);
            continue;
        }
        let mut final_movement_vector = to_player.normalize() * 469.69 * t.delta_seconds();
        for iter in 0..4 {
            let hit = physics_world.ray_cast_with_filter(
                xf.translation,
                final_movement_vector * 5.0,
                true,
                CollisionLayers::none()
                    .with_group(PhysLayer::Enemies)
                    .with_mask(PhysLayer::World),
                |_entitity| true,
            );

            if let Some(collision) = hit {
                if iter == 3 {
                    final_movement_vector = Vec3::ZERO;
                    break;
                }
                let cross = collision.normal.cross(Vec3::Z);
                final_movement_vector = cross * cross.dot(final_movement_vector);
            }
        }
        let direction = player_pos - xf.translation;
        // fixes rotation
        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        xf.rotation = Quat::from_axis_angle(Vec3::Z, angle);
        xf.translation += final_movement_vector;
    }
}

/*
pub fn enemy_flock(
    mut q: Query<&mut Transform, With<Enemy>>,
    t: Res<Time>,
) {
    let enemypos: Vec<_> = q.iter().map(|xf| xf.translation.truncate()).collect();
    for mut xf in q.iter_mut() {
        let pos2 = xf.translation.truncate();
        let mut accum = Vec2::ZERO;
        for pos in enemypos.iter() {
            if pos2 == *pos { continue; }
            let distance = pos.distance(pos2);
            accum += (pos2 - *pos).normalize() * (-distance * 0.001).exp();
        }
        xf.translation += (accum * t.delta_seconds()).extend(0.0);
        dbg!(accum);
    }
}
*/

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
    animations: Res<EnemyAnimations>,
) {
    if mb.just_pressed(MouseButton::Middle) {
        commands.spawn_bundle(EnemyBundle::from_animation_transform_size(
            &animations.movement,
            Transform::from_translation(crs.0.extend(0.0)),
            None,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_zones(
    mut commands: Commands,
    q_player: Query<&GlobalTransform, With<Player>>,
    q_zone: Query<(&EditableCollider, &SpawnZone)>,
    q_zone2: Query<(Entity, &GlobalTransform), With<SpawnZone>>,
    mut cfg: ResMut<EnemyConfig>,
    t: Res<Time>,
    animations: Res<EnemyAnimations>,
    physics_world: PhysicsWorld,
) {
    use bevy::core::FloatOrd;

    cfg.timer_fast.tick(t.delta());
    cfg.timer_slow.tick(t.delta());

    let timer;

    if cfg.count < cfg.min_count {
        timer = cfg.timer_fast.finished();
    } else if cfg.count < cfg.max_count {
        timer = cfg.timer_slow.finished();
    } else {
        return;
    }

    if !timer {
        return;
    }

    debug!("Trying to spawn new enemy.");

    let mut rng = rand::thread_rng();

    let mindist2 = cfg.min_distance * cfg.min_distance;
    let playerpos = q_player.single().translation.truncate();

    // let mut zones: Vec<_> = q_zone2.iter().map(|(e, xf)| (e, xf.translation.truncate())).collect();
    // zones.sort_unstable_by_key(|(e, pos)| FloatOrd(pos.distance_squared(playerpos)));

    // get (Entity, Vec2) (position) of each zone above the min distance
    let mut zones: Vec<_> = q_zone2
        .iter()
        .map(|(e, xf)| (e, xf.translation.truncate()))
        .filter(|(_e, pos)| pos.distance_squared(playerpos) > mindist2)
        .collect();

    // sort zones by distance to the player
    zones.sort_unstable_by_key(|(_e, pos)| FloatOrd(pos.distance_squared(playerpos)));

    for (e, pos) in zones {
        let pos = pos.extend(0.);

        let shape = &CollisionShape::Sphere { radius: 100.0 };
        let hit = physics_world.shape_cast_with_filter(
            shape,
            pos,
            Quat::IDENTITY,
            Vec3::new(playerpos.x, playerpos.y, 0.0) - pos,
            CollisionLayers::none()
                .with_group(PhysLayer::Enemies)
                .with_mask(PhysLayer::World),
            |_entitity| true,
        );

        if hit.is_some() {
            // try next zone
            continue;
        }

        debug!("picked zone at {:?}", pos);
        let (area, _zone) = q_zone.get(e).unwrap();
        let x = rng.gen_range(-area.half_extends.x..area.half_extends.x);
        let y = rng.gen_range(-area.half_extends.y..area.half_extends.y);
        let spawnpos = Vec3::new(x, y, 0.0);

        let (_, xf) = q_zone2.get(e).unwrap();
        let mat = xf.compute_matrix();
        let spawnpos = mat.transform_point3(spawnpos);

        commands.spawn_bundle(EnemyBundle::from_animation_transform_size(
            &animations.movement,
            Transform::from_translation(spawnpos),
            None,
        ));

        cfg.count += 1;
        cfg.timer_fast.reset();
        cfg.timer_slow.reset();
        break;
    }
}
