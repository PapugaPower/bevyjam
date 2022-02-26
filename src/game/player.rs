use crate::game::shooting::{LastShootTime, Weapon};
use bevy::prelude::*;
use crate::game::crosshair::Crosshair;

#[derive(Component)]
pub struct Player {
    // to be expanded
}

pub fn init_player(mut commands: Commands) {
    let player_tform = Transform::from_scale(Vec3::new(32.0, 32.0, 1.0));

    commands
        .spawn_bundle(SpriteBundle {
            transform: player_tform,
            sprite: Sprite {
                color: Color::rgb(0.1, 0.4, 0.1),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player {})
        .insert(Weapon {
            fire_rate: 1.0 / 5.0,
            bullet_speed: 1000.0,
            spread: 90.0,
            num_bullets_per_shot: 5,
        })
        .insert(LastShootTime { time: 0.0 });
    
    // Spawn a rectangle gizmo for movement reference
    commands.spawn_bundle(SpriteBundle{
        transform: player_tform,
        sprite: Sprite {
            color: Color::rgb(0.1, 0.4, 0.1),
            ..Default::default()
        },
        ..Default::default()
    })
        .insert(Player{});
    
    let mut reference_tform = Transform::from_scale(Vec3::new(128.0, 32.0, 1.0));
    reference_tform.translation = Vec3::new(50.0, 50.0, 1.0);
    commands.spawn_bundle(SpriteBundle {
        transform: reference_tform,
        sprite: Sprite {
            color: Color::rgb(0.1, 0.1, 0.1),
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn tear_down_player(mut commands: Commands, q: Query<Entity, With<Player>>) {
    let player_entity = q.single();
    commands.entity(player_entity).despawn();
}

pub fn transfer_input_to_player_system(mut player_tform_q: Query<&mut Transform, With<Player>>,
                                       xhair_q: Query<&Crosshair>, 
                                       keys: Res<Input<KeyCode>>, 
                                       time: Res<Time>
) {
    let mut player_tform = player_tform_q.single_mut();
    let xhair = xhair_q.single();
    let mut mouse_pos_level = xhair.mouse_pos;
    mouse_pos_level.z = 0.0;
    
    let mut direction = mouse_pos_level - player_tform.translation;
    let angle = direction.y.atan2(direction.x);
    player_tform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
    
    let mut movement_vec = Vec3::ZERO;
    if keys.pressed(KeyCode::W) {
        movement_vec += Vec3::Y;
    }

    if keys.pressed(KeyCode::S) {
        movement_vec -= Vec3::Y;
    }

    if keys.pressed(KeyCode::A) {
        movement_vec -= Vec3::X;
    }

    if keys.pressed(KeyCode::D) {
        movement_vec += Vec3::X;
    }
    
    player_tform.translation = player_tform.translation + (movement_vec * 140.0 * time.delta_seconds()); 
}