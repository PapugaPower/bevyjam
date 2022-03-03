use super::{GameAssets, SpatialAudioReceptor};
use crate::game::crosshair::Crosshair;
use crate::game::damage::Health;
use crate::game::enemies::EnemyWave;
use crate::game::phys_layers::PhysLayer;
use crate::game::shooting::{AmmoType, LastShootTime, Weapon};
use crate::util::WorldCursor;
use crate::AppState;
use bevy::prelude::*;
use heron::rapier_plugin::{PhysicsWorld, ShapeCastCollisionType};
use heron::{CollisionLayers, CollisionShape, RigidBody};

#[derive(Component)]
pub struct Player {
    // to be expanded
}

#[derive(Component)]
pub struct PlayerMovementSpeed {
    pub value: f32,
}

#[derive(Component)]
pub enum ShootingAnimationState {
    NotShooting,
    Shooting,
}

#[derive(Component)]
pub struct ShootingAnimationTimer {
    pub animation_time: f32,
    pub timer: Option<Timer>,
}

pub fn init_player(mut commands: Commands, assets: Option<Res<GameAssets>>) {
    if let Some(assets) = assets {
        let player_transform = Transform::default();
        let _x = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(48.0, 48.0)),
                    ..Default::default()
                },

                texture: assets.player_idle.clone(),
                transform: player_transform,
                ..Default::default()
            })
            .insert(super::GameCleanup)
            .insert(Player {})
            .insert(Health {
                current: 200.,
                max: 200.,
            })
            .insert(PlayerMovementSpeed { value: 320.0 })
            .insert(Weapon {
                ammo_type: AmmoType::Projectile,
                damage: 26.0,
                fire_rate: 1.0 / 10.0,
                projectile_speed: 2000.0,
                projectile_life_time: 1.0,
                spread: 90.0,
                projectiles_per_shot: 1,
                projectile_spawn_offset: 50.0,
                radius_of_effect: 100.0,
            })
            .insert(LastShootTime { time: 0.0 })
            .insert(ShootingAnimationState::NotShooting)
            .insert(ShootingAnimationTimer {
                animation_time: 0.05,
                timer: None,
            })
            .insert(EnemyWave {
                timer: Timer::from_seconds(5.0, true),
                number: 10,
                radius: 1000.0,
                despawn_radius: 1500.0,
            })
            .insert(RigidBody::KinematicPositionBased)
            .insert(
                CollisionLayers::none()
                    .with_group(PhysLayer::Player)
                    .with_masks(&[PhysLayer::World, PhysLayer::PlayerTriggers]),
            )
            .insert(CollisionShape::Sphere { radius: 24.0 })
            .insert(SpatialAudioReceptor);
    }
}

pub fn player_shooting_animation(
    time: Res<Time>,
    assets: Option<Res<GameAssets>>,
    mut query_player: Query<(
        &mut Handle<Image>,
        &mut ShootingAnimationState,
        &mut ShootingAnimationTimer,
    )>,
) {
    if let Some(assets) = assets {
        let (mut texture, mut state, mut timer) = query_player.single_mut();
        match state.as_mut() {
            ShootingAnimationState::NotShooting => {
                if let Some(t) = &mut timer.timer {
                    t.tick(time.delta());
                    if t.finished() {
                        *texture = assets.player_idle.clone();
                        timer.timer = None;
                    }
                }
            }
            ShootingAnimationState::Shooting => {
                if let Some(t) = &mut timer.timer {
                    t.reset();
                } else {
                    timer.timer = Some(Timer::from_seconds(timer.animation_time, false));
                    *texture = assets.player_shooting.clone();
                }
                *state = ShootingAnimationState::NotShooting;
            }
        }
    }
}

pub fn transfer_input_to_player_system(
    mut player_tform_q: Query<
        (&mut Transform, &CollisionShape, &PlayerMovementSpeed),
        With<Player>,
    >,
    xhair_q: Query<&Crosshair>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    crs: Res<WorldCursor>,
    physics_world: PhysicsWorld,
) {
    let (mut player_tform, player_col, speed) = player_tform_q.single_mut();
    let xhair = xhair_q.single();
    let mut mouse_pos_level = crs.0.extend(0.0);

    let direction = mouse_pos_level - player_tform.translation;
    let angle = direction.y.atan2(direction.x);
    player_tform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

    let mut kb_inupt_vector = Vec3::ZERO;
    if keys.pressed(KeyCode::W) {
        kb_inupt_vector += Vec3::Y;
    }

    if keys.pressed(KeyCode::S) {
        kb_inupt_vector -= Vec3::Y;
    }

    if keys.pressed(KeyCode::A) {
        kb_inupt_vector -= Vec3::X;
    }

    if keys.pressed(KeyCode::D) {
        kb_inupt_vector += Vec3::X;
    }

    let mut final_movement_vector = kb_inupt_vector * speed.value * time.delta_seconds();

    // We re-check for collisions and calulate movement deflection three times,
    // and discard inputs on the fourth - better make the player stand still
    // if the geometry is too restrictive.
    for iter in 0..4 {
        let hit = physics_world.shape_cast_with_filter(
            player_col,
            player_tform.translation,
            player_tform.rotation,
            final_movement_vector * 1.1,
            CollisionLayers::none()
                .with_group(PhysLayer::Player)
                .with_mask(PhysLayer::World),
            |_entitity| true,
        );

        if let Some(collision) = hit {
            if let ShapeCastCollisionType::Collided(info) = collision.collision_type {
                if iter == 3 {
                    final_movement_vector = Vec3::ZERO;
                    break;
                }
                let cross = info.self_normal.cross(Vec3::Z);
                final_movement_vector = cross * cross.dot(final_movement_vector);
            } else if let ShapeCastCollisionType::AlreadyPenetrating = collision.collision_type {
                // TODO: If there are collision artifacts resulting in this being called,
                // implement a "last trusted position" local resource to be reverted to.
            }
        }
    }

    player_tform.translation += final_movement_vector;
}

pub fn check_player_dead(
    health_q: Query<&Health, With<Player>>,
    mut state: ResMut<State<AppState>>,
    mut commands: Commands,
) {
    let health = health_q.single();
    if health.current <= 0.0 {
        state.push(AppState::GameOver).unwrap();
        commands.insert_resource(crate::game::GameResult::LoseHealth);
    }
}
