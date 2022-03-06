use std::f32::consts::PI;
use std::vec;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::Collision;
use heron::{CollisionLayers, CollisionShape};
use heron::rapier_plugin::nalgebra::Quaternion;
use heron::rapier_plugin::PhysicsWorld;
use crate::Commands;
use crate::game::collider::Wall;
use crate::game::phys_layers::PhysLayer;

pub struct NavGrid {
    pub positions: Vec<Vec2>,
    pub costs: Vec<f32>,
    pub links: Vec<Vec<i32>>,
    pub valid: Vec<bool>,
}

#[derive(Component)]
pub struct NodeDebugToken;

#[derive(Default)]
pub struct GridState{
    pub ran: bool,
    
}

pub fn generate_grid(mut commands: Commands, 
                     q: Query<(&Transform, &CollisionShape), With<Wall>>, 
                     mut state: Local<GridState> ){
    if state.ran {return;}
    
    if q.iter().count() < 1 {return;}
    
    const SIZE: f32 = 8200.0;
    const RESOLUTION: f32 = 40.0;
    let step = (SIZE / RESOLUTION) as i32;
    let offset = Vec2::new(SIZE /2.0, SIZE /2.0);
    let mut grid = NavGrid {
        positions: Vec::new(),
        costs: Vec::new(),
        links: Vec::new(),
        valid: Vec::new()
    };
    
    
    for x in 0..step {
        for y in 0..step {
            let x_coord = (x*step) as f32;
            let y_coord = (y*step) as f32;
            let pos = Vec2::new(x_coord - offset.x, y_coord - offset.y);
            grid.valid.push(check_node_validity(&q, 40.0, pos));
            grid.positions.push(pos);
        }
    }
    info!("Created {} spots", grid.positions.len().to_string());
    let mut idx = 0;
    for pos in grid.positions{
        commands.spawn_bundle(SpriteBundle{
            sprite: Sprite {
                color: if grid.valid[idx] {Color::GREEN} else {Color::RED},
                flip_x: false,
                flip_y: false,
                custom_size: Option::from(Vec2::new(25.0, 25.0))
            },
            transform: Transform::from_xyz(pos.x, pos.y, 5.0),
            ..Default::default()
        });
        idx += 1
    }
    
    state.ran = true;
}

fn check_node_validity(q: &Query<(&Transform, &CollisionShape), With<Wall>>, radius: f32, pos: Vec2) 
    -> bool {
    
    let mut c = 0;
    for (xf, shape) in q.iter(){
        let col_pos = xf.translation;
        let col_rot = xf.rotation.to_euler(EulerRot::XYZ).2;
        if let CollisionShape::Cuboid {half_extends, border_radius} = shape {
            let overlap = rect_circle_overlap(
                Vec2::new(col_pos.x, col_pos.y),
                Vec2::new(half_extends.x, half_extends.y),
                col_rot,
                Vec2::new(pos.x, pos.y),
                radius);

            if overlap { return false; }
        }
        c += 1;

    }
    info!("Checked {} colliders, no collision found", c.to_string());

    return true;
}

fn rect_circle_overlap(rect_pos: Vec2, rect_dim: Vec2, rect_rot_z: f32, c_pos: Vec2, c_radius: f32)
    -> bool {
    let rel = c_pos - rect_pos;
    let local_x = f32::cos(rect_rot_z) * rel.x + f32::cos(rect_rot_z - PI * 0.5) * rel.y;
    let local_y = f32::sin(rect_rot_z) * rel.x + f32::sin(rect_rot_z - PI * 0.5) * rel.y;

    f32::abs(local_x) - c_radius < rect_dim.x && f32::abs(local_y) - c_radius < rect_dim.y
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test] // Smaller circle fully inside the rect
    fn test_circle_overlap_a() {
        let x = rect_circle_overlap(Vec2::ZERO, Vec2::ONE, 0.0, Vec2::ZERO, 0.15);
        assert_eq!(x, true);
    }

    // Larger circle fully encompassing the rect
    #[test]
    fn test_circle_overlap_b() {
        let x = rect_circle_overlap(Vec2::ZERO, Vec2::ONE, 0.0, Vec2::ZERO, 3.0);
        assert_eq!(x, true);
    }
    
    // Larger circle slightly overlapping the rect
    #[test]
    fn test_circle_overlap_c() {
        let x = rect_circle_overlap(Vec2::ZERO, Vec2::ONE, 0.0, Vec2::X, 1.2);
        assert_eq!(x, true);
    }

    #[test] // Offset test
    fn test_circle_overlap_d() {
        let x = rect_circle_overlap(Vec2::X * 2.01, Vec2::ONE, 0.0, Vec2::ZERO, 1.0);
        assert_eq!(x, false);
    }

    #[test] // Rotation test
    fn test_circle_overlap_e() {
        let x = rect_circle_overlap(Vec2::X * 2.01, Vec2::ONE, 45.0, Vec2::ZERO, 1.0);
        assert_eq!(x, true);
    }

    #[test] // Rotation + offset test
    fn test_circle_overlap_f() {
        let x = rect_circle_overlap(Vec2::X * 3.01, Vec2::ONE, 45.0, Vec2::ZERO, 1.0);
        assert_eq!(x, false);
    }
}