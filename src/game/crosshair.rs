use bevy::prelude::*;
use bevy::math::Vec3;
use iyes_bevy_util::BevyState;

use crate::util::{MainCamera, WorldCursor};

const CROSSHAIR_Z: f32 = 10.0;

#[derive(Component)]
pub struct Crosshair;

pub fn setup_crosshair(mut commands: Commands) {
    let mut xhair_tform= Transform::from_scale(Vec3::new(4.5, 4.5, 4.5));
    
    commands.spawn_bundle(SpriteBundle{
        transform: xhair_tform,
        sprite: Sprite {
            color: Color::rgb(1.0, 0.1, 0.1),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(super::GameCleanup)
    .insert(Crosshair);
    
    info!("Crosshair initialized.");
}

pub fn crosshair_position_update_system(
    crs: Res<WorldCursor>,
    mut q: Query<&mut Transform, With<Crosshair>>
) {
    let mut xhair_tform = q.single_mut();
    xhair_tform.translation = crs.0.extend(CROSSHAIR_Z);
}
