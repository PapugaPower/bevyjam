use super::animations::AnimationBundle;
use super::shooting::WeaponryBundle;
use super::SpatialAudioReceptor;
use crate::game::damage::Health;
use crate::game::phys_layers::PhysLayer;
use crate::util::WorldCursor;
use crate::AppState;
use bevy::prelude::*;
use heron::rapier_plugin::{PhysicsWorld, ShapeCastCollisionType};
use heron::{CollisionLayers, CollisionShape, RigidBody};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerLegs {
    pub initialized: bool,
}

#[derive(Component)]
pub struct PlayerMovementSpeed {
    pub value: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum PlayerMoveState {
    Idle,
    Running,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum PlayerShootState {
    Nothing,
    Reloading,
    Shooting,
}

#[derive(Component, Debug)]
pub struct PlayerState {
    pub current: (PlayerMoveState, PlayerShootState),
    pub new: (PlayerMoveState, PlayerShootState),
}

impl PlayerState {
    pub fn changed(&self) -> bool {
        self.current != self.new
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub movement_speed: PlayerMovementSpeed,
    pub health: Health,
    // cleanup marker
    pub state: PlayerState,
    pub cleanup: super::GameCleanup,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            movement_speed: PlayerMovementSpeed { value: 320.0 },
            health: Health {
                current: 200.,
                max: 200.,
            },
            state: PlayerState {
                // different states to trigger animation
                current: (PlayerMoveState::Running, PlayerShootState::Nothing),
                new: (PlayerMoveState::Idle, PlayerShootState::Nothing),
            },
            cleanup: super::GameCleanup,
        }
    }
}

pub fn init_player(mut commands: Commands) {
    let player_transform = Transform::from_translation(Vec3::new(-2982.9265, 1052.7454, 1.0));
    let _x = commands
        .spawn_bundle(AnimationBundle::from_default_with_transform_size(
            player_transform,
            Some(Vec2::new(64.0, 64.0)),
        ))
        .insert_bundle(PlayerBundle::default())
        .insert_bundle(WeaponryBundle::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(
            CollisionLayers::none()
                .with_group(PhysLayer::Player)
                .with_masks(&[
                    PhysLayer::World,
                    PhysLayer::PlayerTriggers,
                    PhysLayer::Enemies,
                ]),
        )
        .insert(CollisionShape::Sphere { radius: 24.0 })
        .insert(SpatialAudioReceptor)
        .with_children(|commands| {
            commands.spawn_bundle(AnimationBundle::from_default_with_transform_size(
                Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                Some(Vec2::new(32.0, 32.0)),
            ))
            .insert(PlayerLegs {
                initialized: false,
            });
        });
}

pub fn print_player_position(q: Query<&Transform, With<Player>>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::P) {
        let t = q.single();
        println!("Current player position: {:?}", t.translation);
    }
}

pub fn transfer_input_to_player_system(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    crs: Res<WorldCursor>,
    physics_world: PhysicsWorld,
    mut player_tform_q: Query<
        (
            &CollisionShape,
            &PlayerMovementSpeed,
            &mut Transform,
            &mut PlayerState,
        ),
        With<Player>,
    >,
) {
    let (player_col, speed, mut player_tform, mut state) = player_tform_q.single_mut();
    let mouse_pos_level = crs.0.extend(0.0);

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

    // legs direction


    // animations
    if kb_inupt_vector == Vec3::ZERO {
        state.new.0 = PlayerMoveState::Idle;
    } else {
        state.new.0 = PlayerMoveState::Running;
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
