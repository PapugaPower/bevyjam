use bevy::prelude::*;
use core::default::Default;

#[derive(Component)]
pub struct MainCamera;

pub fn init_main_camera(mut commands: Commands){
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}