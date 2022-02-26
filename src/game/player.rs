use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    // to be expanded
}

pub fn init_player(mut commands: Commands) {
    let player_tform = Transform::from_scale(Vec3::new(32.0, 32.0, 1.0));
    
    commands.spawn_bundle(SpriteBundle{
        transform: player_tform,
        sprite: Sprite {
            color: Color::rgb(0.1, 0.4, 0.1),
            ..Default::default()
        },
        ..Default::default()
    })
        .insert(Player{});
}

pub fn tear_down_player(mut commands: Commands, q: Query<Entity, With<Player>>){
    let player_entity = q.single();
    commands.entity(player_entity).despawn();
}