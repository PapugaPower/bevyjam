use crate::game::shooting::{LastShootTime, Weapon, AmmoType};
use crate::util::WorldCursor;
use bevy::prelude::*;
use heron::{CollisionLayers, CollisionShape, RigidBody};
use heron::rapier_plugin::{PhysicsWorld, ShapeCastCollisionType};
use crate::AppState;
use crate::game::crosshair::Crosshair;
use crate::game::phys_layers::PhysLayer;
use crate::game::damage::Health;
use super::SpatialAudioReceptor;

#[derive(Component)]
pub struct Player {
    // to be expanded
}

#[derive(Component)]
pub struct PlayerMovementSpeed {
    pub value: f32
}

pub fn init_player(mut commands: Commands) {
    let player_tform = Transform::from_scale(Vec3::new(48.0, 48.0, 1.0));

    let _x = commands
        .spawn_bundle(SpriteBundle {
            transform: player_tform,
            sprite: Sprite {
                color: Color::rgb(0.1, 0.4, 0.1),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player {})
        .insert(Health { current: 200., max: 200.})
        .insert(PlayerMovementSpeed{value: 320.0})
        .insert(Weapon {
            ammo_type: AmmoType::Throwable,
            damage: 69.0,
            fire_rate: 1.0 / 5.0,
            projectile_speed: 10.0,
            projectile_life_time: 2.0,
            spread: 90.0,
            projectiles_per_shot: 5,
            projectile_spawn_offset: 50.0,
            radius_of_effect: 100.0,
        })
        .insert(LastShootTime { time: 0.0 })
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::Player)
            .with_masks(&[PhysLayer::World, PhysLayer::Enemies]))
        .insert(CollisionShape::Sphere { radius: 24.0 })
		.insert(SpatialAudioReceptor);
}

pub fn transfer_input_to_player_system(
    mut player_tform_q: Query<(&mut Transform, &CollisionShape, &PlayerMovementSpeed), With<Player>>,
    xhair_q: Query<&Crosshair>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    crs: Res<WorldCursor>,
    physics_world: PhysicsWorld
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
    for iter in 0..4{
        let hit = physics_world.shape_cast_with_filter(
            player_col,
            player_tform.translation,
            player_tform.rotation,
            final_movement_vector * 1.1,
            CollisionLayers::none().with_group(PhysLayer::Player).with_mask(PhysLayer::World),
            |_entitity| true);

        if let Some(collision) = hit {
            if let ShapeCastCollisionType::Collided(info) = collision.collision_type{
                if iter == 3 {
                    final_movement_vector = Vec3::ZERO;
                    break;
                }
                let cross = info.self_normal.cross(Vec3::Z);
                final_movement_vector = cross * cross.dot(final_movement_vector);
            } else if let ShapeCastCollisionType::AlreadyPenetrating = collision.collision_type{
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
