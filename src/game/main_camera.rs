use bevy::prelude::*;
use core::default::Default;
use crate::game::crosshair::Crosshair;
use crate::game::player::Player;
use crate::util::MainCamera;

#[derive(Component)]
pub struct CameraControl {
    destination: Vec3
}

pub fn init_main_camera(mut commands: Commands){
    let mut cam_bundle = OrthographicCameraBundle::new_2d();
    cam_bundle.orthographic_projection.scale = 1.0;
    commands.spawn_bundle(cam_bundle)
        .insert(MainCamera)
        .insert(CameraControl{destination: Vec3::ZERO});
}

// Takes player's avatar and crosshair position into account.
pub fn recalculate_camera_desination_system(cam_tform_q: Query<&Transform, With<CameraControl>>,
                                            xhair_q: Query<&Transform, With<Crosshair>>,
                                            player_q: Query<&Transform, With<Player>>,
                                            mut cam_q: Query<&mut CameraControl>
)
{
    let cam_tform = cam_tform_q.single();
    let xhair_tform = xhair_q.single();
    let player_tform = player_q.single();
    let mut cam = cam_q.single_mut();
    
    cam.destination = player_tform.translation;
    
    return;
    
    // TODO: solve the mouse follow problem

    let player_to_xhair = player_tform.translation - xhair_tform.translation;
    let to_xhair = player_to_xhair.clone().normalize();
    let max_dist_from_player: f32 = 150.0;

    let mut new_pos: Vec3 = player_tform.translation - player_to_xhair.clamp_length_max(max_dist_from_player);
    new_pos.z = 100.0;

    cam.destination = new_pos;
}

pub fn refresh_camera_position_system(mut cam_q: Query<(&mut Transform, &CameraControl)>){
    
    let (mut cam_tform, cam) = cam_q.single_mut();
    
    let mut cam_dest_leveled = cam.destination;
    cam_dest_leveled.z = 100.0;
    
    cam_tform.translation = cam_dest_leveled;
}
