use crate::game::shooting::{LastShootTime, Weapon};
use bevy::prelude::*;

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
}

pub fn tear_down_player(mut commands: Commands, q: Query<Entity, With<Player>>) {
    let player_entity = q.single();
    commands.entity(player_entity).despawn();
}
