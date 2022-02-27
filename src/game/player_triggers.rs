use bevy::prelude::*;
use heron::CollisionShape;
use crate::game::player::Player;

#[derive(Component)]
pub struct PeriodicActivation {
    pub frequency: f64,
    pub last_update: f64
}

#[derive(Component)]
pub struct PlayerPresenceDetector {
    pub detected: bool,
}

/* 
    WARNING
    Does NOT take player scale into account, if player ends up being scaled - extend this.
*/
pub fn evaluate_player_detection_triggers_system(q_player: Query<(&Transform, &CollisionShape), With<Player>>,
                                                 mut q_detectors: Query<(&Transform, &CollisionShape, &mut PlayerPresenceDetector)>){
    let (player_tform, player_col) = q_player.single();

    let mut player_radius = 0.0;
    if let CollisionShape::Sphere{radius} = player_col{
        player_radius = *radius;
    }

    for (d_tform, d_col, mut detector) in q_detectors.iter_mut(){
        let mut overlap = false;
        if let CollisionShape::Sphere{radius} = d_col {
            let col_radius = *radius;
            overlap = check_sphere_trigger_overlap(player_tform.translation,
                                                   player_radius,
                                                   d_tform,
                                                   col_radius);
        } else if let CollisionShape::Cuboid{half_extends, ..} = d_col {
            let col_extents = Vec3::new(half_extends.x, half_extends.y, 1.0);
            overlap = check_box_trigger_overlap(player_tform.translation,
                                                player_radius,
                                                d_tform,
                                                col_extents * d_tform.scale);
        }

        if overlap {
            detector.detected = true;
        } else {
            detector.detected = false;
        }
    }
}

fn check_box_trigger_overlap(player_pos: Vec3,
                             player_radius: f32,
                             trig_tform: &Transform,
                             trig_size: Vec3) -> bool {
    let diff = (trig_tform.translation - player_pos).abs();
    let trig_size_scaled = Vec2::new(trig_size.x, trig_size.y) * 0.5; // extents are automatically scaled to transform by heron
    return diff.x - player_radius < trig_size_scaled.x && diff.y - player_radius < trig_size_scaled.y;
}

fn check_sphere_trigger_overlap(player_pos: Vec3,
                                player_radius: f32,
                                trig_tform: &Transform,
                                trig_radius: f32) -> bool {
    let diff = (trig_tform.translation - player_pos).abs();
    let trig_radius_scaled = trig_radius * ((trig_tform.scale.x + trig_tform.scale.y)/2.); // Doesn't allow for oval colliders
    let radii_sum = trig_radius_scaled + player_radius;
    return diff.x < radii_sum && diff.y < radii_sum;
}