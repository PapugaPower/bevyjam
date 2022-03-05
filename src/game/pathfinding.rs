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
            let col_size_x = half_extends.x;
            let col_size_y = half_extends.y;
            let rel = pos.extend(0.) - col_pos;
            let local_x = f32::cos(col_rot) * rel.x + f32::cos(col_rot - PI * 0.5) * rel.y;
            let local_y = f32::sin(col_rot) * rel.x + f32::sin(col_rot - PI * 0.5) * rel.y; 
            
            //let delta_x = f32::min(local_x, col_size_x);
            //let delta_y = f32::min(local_y, col_size_y);
            
            if f32::abs(local_x) - radius < col_size_x && f32::abs(local_y) - radius < col_size_y  { return false; }
        }
        
        c += 1;
        
        /*
        rel_x = shape_x - rect_x;
        rel_y = shape_y - rect_y;
        angle = -rect_angle;
        local_x = cos(angle) * rel_x + cos(angle - pi / 2) * rel_y;
        local_y = sin(angle)  * rel_x + sin(angle - pi / 2) * rel_y;
        
        delta_x = min(local_x, rect_width);
        delta_y = min(local_y, rect_height);
        return delta_x * delta_x + delta_y * delta_y < circle_radius * circle_radius;
         */
    }
    info!("Checked {} colliders, no collision found", c.to_string());

    return true;
}