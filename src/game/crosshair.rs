use bevy::prelude::*;
use bevy::math::Vec3;
use crate::game::main_camera::MainCamera;
use iyes_bevy_util::BevyState;

#[derive(Component)]
pub struct Crosshair {
    mouse_pos: Vec3,
}

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
        .insert(Crosshair{mouse_pos: Vec3::ZERO});
    
    println!("Crosshair initialized.");
}

pub fn tear_down_crosshair(mut commands: Commands, q: Query<(Entity), With<Crosshair>>){
    let crosshair_entity = q.single();
    
    commands.entity(crosshair_entity).despawn();
}

pub fn crosshair_positon_update_system(mut q: Query<(&mut Transform, &Crosshair)>){
    let (mut xhair_tform, xhair) = q.single_mut();
    xhair_tform.translation = xhair.mouse_pos;
}

pub fn mouse_pos_to_wspace_system(mut q: Query<&mut Crosshair>,
                              cam_q: Query<&Transform, With<MainCamera>>,
                              mut windows: ResMut<Windows>
){
    let mut window = windows.get_primary_mut().unwrap();
    let mut xhair = q.single_mut();

    if let Some(_position) = window.cursor_position() {
        window.set_cursor_visibility(false);
        let cam_tform = cam_q.single();
        let pos = _position;
        let wspace_pos = window_to_world(window, cam_tform, &pos);
        xhair.mouse_pos = wspace_pos;
    } else {
        window.set_cursor_visibility(true);
    }
}

fn window_to_world(
    window: &Window,
    camera: &Transform,
    position: &Vec2,
) -> Vec3 {
    let center = camera.translation.truncate();
    let half_width = (window.width() / 2.0) * camera.scale.x;
    let half_height = (window.height() / 2.0) * camera.scale.y;
    let left = center.x - half_width;
    let bottom = center.y - half_height;
    Vec3::new(
        left + position.x * camera.scale.x,
        bottom + position.y * camera.scale.y,
        10.0, 
    )
}