use std::f32::consts::PI;
use std::vec;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::Collision;
use bevy_prototype_debug_lines::DebugLines;
use heron::{CollisionLayers, CollisionShape};
use heron::rapier_plugin::nalgebra::Quaternion;
use heron::rapier_plugin::PhysicsWorld;
use crate::Commands;
use crate::game::collider::Wall;
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;

#[derive(Default)]
pub struct NavGrid {
    pub positions: Vec<Vec2>,
    pub costs: Vec<f32>,
    pub links: Vec<Vec<i32>>,
    pub valid: Vec<bool>,
    pub complete: bool,
}

#[derive(Component)]
pub struct NodeDebugToken;

#[derive(Default)]
pub struct GridState{
    pub ran: bool,
    
}

const SIZE: f32 = 15000.0;
const RESOLUTION: f32 = 0.015; // points per unit
const SIDE_NODE_NO: i32 = (RESOLUTION * SIZE) as i32;

pub fn generate_grid(mut commands: Commands, 
                     q: Query<(&Transform, &CollisionShape), With<Wall>>, 
                     mut state: Local<GridState>, 
                     mut nav_grid: ResMut<NavGrid>){
    if state.ran || nav_grid.complete {return;}
    
    if q.iter().count() < 1 {return;}

    let offset = Vec2::new(SIZE /2.0 + 2000.0, SIZE /2.0 + 2000.0);
    nav_grid.costs.clear();
    nav_grid.positions.clear();
    nav_grid.valid.clear();
    nav_grid.links.clear();
    
    for x in 0..SIDE_NODE_NO {
        for y in 0..SIDE_NODE_NO {
            let x_coord = x as f32/RESOLUTION;
            let y_coord = y as f32/RESOLUTION;
            let pos = Vec2::new(x_coord - offset.x, y_coord - offset.y);
            nav_grid.valid.push(check_node_validity(&q, 30.0, pos));
            nav_grid.positions.push(pos);
        }
    }
    info!("Created {} nodes", nav_grid.positions.len().to_string());
    // Build links
    for idx in 0.. nav_grid.positions.len() {
        let pos = &nav_grid.positions[idx];
        let (grid_x, grid_y) = coord_from_1d(SIDE_NODE_NO, idx as i32);
        
        if !nav_grid.valid[idx] { // omit invalid cells
            nav_grid.links.push(vec![-1,-1,-1,-1,-1,-1,-1,-1]);
            continue;
        }
        
        let mut nbours = get_neighbours(SIDE_NODE_NO, grid_x, grid_y);
        for n in 0..nbours.len(){
            if nbours[n] < 0 { continue }
            if nav_grid.valid[nbours[n] as usize] == false {
                nbours[n] = -1;
            }
        }
        nav_grid.links.push(nbours);

    }

    // Distribute weights
    // currently only based on no. of neighbours - gives agents slight preference for open spaces
    for idx in 0.. nav_grid.positions.len() {
        let mut count = 0.0;
        let links = &nav_grid.links[idx];
        for n in links {
            if *n > 0 {count += 1.0}
        }
        let cost = 1.0 + (8.0-count)*0.15;
        nav_grid.costs.push(cost)
    }
    
    // draw markers (development only)
    let mut i = 0;
    for pos in &nav_grid.positions{
        let is_valid = nav_grid.valid[i];
        let mut color = Color::WHITE;
        if is_valid {
            let blue = nav_grid.costs[i] - 1.0;
            color = Color::rgba(0.0, 1.0 - blue, blue, 0.05);
        }
        else {
            color = Color::rgba(1.0, 0.0, 0.0, 0.05);
        }
        commands.spawn_bundle(SpriteBundle{
            sprite: Sprite {
                color: color,
                flip_x: false,
                flip_y: false,
                custom_size: Option::from(Vec2::new(25.0, 25.0))
            },
            transform: Transform::from_xyz(pos.x, pos.y, 1.0),
            ..Default::default()
        });
        i += 1
    }
    nav_grid.complete = true;
    state.ran = true;
}
pub fn draw_links(
    mut lines: ResMut<DebugLines>,
    nav_grid: Res<NavGrid>,
    player_q: Query<&Transform, With<Player>>
){
    //let visited = Vec::<usize>::new();
    if !nav_grid.complete {return;}
    
    let ppos = player_q.single().translation.truncate();
    
    for idx in 0..nav_grid.positions.len() {
        if !nav_grid.valid[idx] {continue};
        let this_node_pos = nav_grid.positions[idx];
        if (this_node_pos - ppos).length_squared() > 50000.0 {continue} 
        let links = &nav_grid.links[idx];
        for link in links{
            if *link < 0 { continue }
            if !nav_grid.valid[*link as usize] {continue};
            let linked_pos = nav_grid.positions[*link as usize];
            lines.line_colored(this_node_pos.extend(0.1), linked_pos.extend(0.1), 0.001, Color::rgba(0.3, 0.3, 0.3, 0.12));
        }
    }
    
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

/*
 0 1 2
  \|/
 3-x-4   <- neighbour vector layout
  /|\
 5 6 7 
 */
fn get_neighbours(side_dim: i32, curr_x: i32, curr_y:i32) -> Vec<i32> {
    let mut out:Vec<i32> = vec![-1,-1,-1,-1,-1,-1,-1,-1];
    let sdmo = side_dim-1;
    if curr_x != 0 && curr_y != 0           // -1 -1
        { out[0] = coord_to_1d(side_dim, curr_x-1, curr_y-1) }
    if curr_y != 0                          // 0 -1
        { out[1] = coord_to_1d(side_dim, curr_x, curr_y-1) }
    if curr_x != sdmo && curr_y != 0        // +1 -1 
        { out[2] = coord_to_1d(side_dim, curr_x+1, curr_y-1) }
    if curr_x != 0                          // -1 0
        { out[3] = coord_to_1d(side_dim, curr_x-1, curr_y) }
    if curr_x != sdmo                       // +1 0
        { out[4] = coord_to_1d(side_dim, curr_x+1, curr_y) }
    if curr_x != 0 && curr_y != sdmo        // -1 +1
        { out[5] = coord_to_1d(side_dim, curr_x+-1, curr_y+1) }
    if curr_y != sdmo                       // 0 +1
        { out[6] = coord_to_1d(side_dim, curr_x, curr_y+1) }
    if curr_x != sdmo && curr_y != sdmo     // +1 +1
        { out[7] = coord_to_1d(side_dim, curr_x+1, curr_y+1) }
    
    out
}

fn coord_from_1d(side_dim: i32, idx: i32) -> (i32,i32) {
    let x = idx % side_dim;
    let y = (idx - x) / side_dim;
    (x,y)
}

fn coord_to_1d(side_dim: i32, x: i32, y:i32) -> i32
{
    y * side_dim + x
}

fn rect_circle_overlap(rect_pos: Vec2, rect_dim: Vec2, rect_rot_z: f32, c_pos: Vec2, c_radius: f32)
    -> bool {
    let rel = c_pos - rect_pos;
    let local_x = f32::cos(rect_rot_z) * rel.x + f32::cos(rect_rot_z - PI * 0.5) * rel.y;
    let local_y = f32::sin(rect_rot_z) * rel.x + f32::sin(rect_rot_z - PI * 0.5) * rel.y;

    f32::abs(local_x) - c_radius < rect_dim.x && f32::abs(local_y) - c_radius < rect_dim.y
}

fn is_point_within_rect(point: Vec2, rect_pos: Vec2, rect_dim: Vec2, rect_rot_z: f32)
    -> bool {
    let rel = point - rect_pos;
    let local_x = f32::cos(rect_rot_z) * rel.x + f32::cos(rect_rot_z - PI * 0.5) * rel.y;
    let local_y = f32::sin(rect_rot_z) * rel.x + f32::sin(rect_rot_z - PI * 0.5) * rel.y;

    f32::abs(local_x) < rect_dim.x && f32::abs(local_y) < rect_dim.y

}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test] // Smaller circle fully inside the rect
    fn test_circle_overlap_a() {
        let mut x = rect_circle_overlap(Vec2::ZERO, Vec2::ONE, 0.0, Vec2::ZERO, 0.15);
        assert_eq!(x, true);
        x = rect_circle_overlap(Vec2::X * 2.01, Vec2::ONE, 45.0, Vec2::ZERO, 1.0);
        assert_eq!(x, true);
        x = rect_circle_overlap(Vec2::ZERO, Vec2::ONE, 0.0, Vec2::X, 1.2);
        assert_eq!(x, true);
        x = rect_circle_overlap(Vec2::X * 3.01, Vec2::ONE, 45.0, Vec2::ZERO, 1.0);
        assert_eq!(x, false);
    }
    
    
    #[test]
    fn test_point_within_rect_a() {
        let mut x = is_point_within_rect(Vec2::new(0.0, 1.1), Vec2::ZERO, Vec2::ONE, 45.0);
        assert_eq!(x, true);
        x = is_point_within_rect(Vec2::new(0.0, 1.1), Vec2::ZERO, Vec2::ONE, 0.0);
        assert_eq!(x, false);
    }
    
    #[test]
    fn test_point_within_rect_b() {

    }
    
    #[test]
    fn test_coord_to_1d() {
        let a = coord_to_1d(4, 2, 1);
        assert_eq!(a, 6);
        let b = coord_to_1d(4, 3, 2);
        assert_eq!(b, 11);
        let c = coord_to_1d(4, 3, 3);
        assert_eq!(c, 15);
        let d = coord_to_1d(4, 0, 0);
        assert_eq!(d, 0);
    }
    
    #[test]
    fn test_1d_to_coord(){
        let a = coord_from_1d(4, 0);
        let b = coord_from_1d(4, 15);
        let c = coord_from_1d(4, 3);
        let d = coord_from_1d(4, 10);
        assert_eq!(a, (0,0));
        assert_eq!(b, (3,3));
        assert_eq!(c, (3,0));
        assert_eq!(d, (2,2));
    }
    
    #[test]
    fn test_get_neighbours(){
        let mut a = get_neighbours(4, 1, 1);
        assert_eq!(a, vec![0,1,2,4,6,8,9,10]);
        a = get_neighbours(4, 0, 0);
        assert_eq!(a, vec![-1,-1,-1,-1,1,-1,4,5]);
        a = get_neighbours(4, 3, 3);
        assert_eq!(a, vec![10,11,-1,14,-1,-1,-1,-1]);
    }
}