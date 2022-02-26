use bevy::prelude::*;
use heron::prelude::*;
use iyes_bevy_util::BevyState;
use crate::GameAssets;

/// This plugin should add all Scenario1 specific stuff
pub struct DevPlaygroundPlugin<S: BevyState> {
    pub state: S,
}

impl<S: BevyState> Plugin for DevPlaygroundPlugin<S> {
    fn build(&self, app: &mut App) {
        // add systems to `self.state`
        app.add_system_set(SystemSet::on_enter(self.state.clone()).with_system(setup_scene));
    }
}

fn setup_scene(mut commands: Commands, assets: Res<GameAssets>) {
    // enemy
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                color: Color::rgb(0.9, 0.2, 0.2),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Sphere { radius: 20.0 });
    
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(500.0, 10.0)),
                color: Color::rgb(0.9, 0.9, 0.9),
                ..Default::default()
            },
            transform: Transform::from_xyz(100.0, 300.0, 0.0),
            ..Default::default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(250.0, 5.0, 0.1),
            border_radius: None,
        });
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(500.0, 10.0)),
                color: Color::rgb(0.9, 0.9, 0.9),
                ..Default::default()
            },
            transform: Transform::from_xyz(100.0, -300.0, 0.0),
            ..Default::default()
        })
        .insert(RigidBody::Static)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(250.0, 5.0, 0.1),
            border_radius: None,
        });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            global_transform: Default::default(),
            texture: assets.map_prototype.clone(),
            visibility: Default::default()
        });
}
