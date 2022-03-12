use binary_heap_plus::{BinaryHeap, MinComparator};
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
    pub paths: Vec<i32>,
    pub complete: bool,
}

#[derive(Component)]
pub struct NodeDebugToken;

#[derive(Default)]
pub struct GridState{
    pub ran: bool,
}

const SIZE: f32 = 11000.0;
const RESOLUTION: f32 = 0.015; // points per unit
const SIDE_NODE_NO: i32 = (RESOLUTION * SIZE) as i32;
const OFFSET_X: f32= SIZE /2.0 + 2000.0;
const OFFSET_Y: f32= SIZE /2.0 + 2000.0;


pub fn generate_grid(
    q: Query<(&Transform, &CollisionShape), With<Wall>>, 
    mut state: Local<GridState>, 
    mut nav_grid: ResMut<NavGrid>
){
    if state.ran {return;}
    if nav_grid.complete {return;}
    
    if q.iter().count() < 1 {return;}
    
    let offset = Vec2::new(OFFSET_X, OFFSET_Y);

    nav_grid.costs.clear();
    nav_grid.positions.clear();
    nav_grid.valid.clear();
    nav_grid.links.clear();
    
    for y in 0..SIDE_NODE_NO {
        for x in 0..SIDE_NODE_NO {
            let x_coord = x as f32/RESOLUTION;
            let y_coord = y as f32/RESOLUTION;
            let pos = Vec2::new(x_coord - offset.x, y_coord - offset.y);
            nav_grid.valid.push(check_node_validity(&q, 30.0, pos));
            nav_grid.positions.push(pos);
        }
    }
    info!("NavGrid created with {} nodes", nav_grid.positions.len().to_string());
    // Build links
    for idx in 0.. nav_grid.positions.len() {
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
    nav_grid.complete = true;
    state.ran = true;
}

pub fn calculate_path(
    from: Vec2, 
    to: Vec2,
    grid: &NavGrid
) -> Vec<Vec2> {
    let mut output = vec![from];
    let start_node_idx = closest_cell_1d(from, Vec2::new(OFFSET_X, OFFSET_Y), SIDE_NODE_NO, RESOLUTION);
    let end_node_idx = closest_cell_1d(to, Vec2::new(OFFSET_X, OFFSET_Y), SIDE_NODE_NO, RESOLUTION);
    let end_node_coord = coord_from_1d(SIDE_NODE_NO, end_node_idx);
    let mut parents = vec![-1; grid.positions.len()];
    let mut h_cost = vec![-1.0; grid.positions.len()];
    let mut g_cost = vec![-1.0; grid.positions.len()];
    let mut f_cost = vec![-1.0; grid.positions.len()];
    let mut open_nodes = Vec::<i32>::new(); // tried to use binary heap, but didn't know how to sort using another vector
    let mut closed_nodes = vec![0; grid.positions.len()];
    // ^ i used a flattened grid vector for each parameter,
    //   since it was easier to reason about, but it's not optimal
    
    open_nodes.push(start_node_idx);
    g_cost[start_node_idx as usize] = 0.0;
    let mut last_result = 1;
    let mut iters = 2500;
    while last_result == 1 {
        last_result = check_node(
            &mut open_nodes,
            &mut closed_nodes,
            &mut parents,
            &mut h_cost,
            &mut g_cost,
            &mut f_cost,
            &grid,
            end_node_idx,
            end_node_coord);
        iters -= 1;
        if iters < 0 { 
            error!("A* exceeded 2500 iterations for a single path!");
            last_result = 0
        }
    }
    
    if last_result == 0 {
        //error!("A* couldn't find path, returning start node.");
        output.push(grid.positions[start_node_idx as usize]);
    } else {
        //info!("A* built a complete path.");
        output = build_path(&parents, end_node_idx as usize, &grid)
    }
    //info!("1st node x: {} y: {}", output[0].x.to_string(), output[0].y.to_string());
    output
}

// Note: A* using Manhattan heuristic
fn check_node(open_nodes: &mut Vec<i32>,
              closed_nodes: &mut Vec<i32>,
              parents: &mut Vec<i32>,
              h_cost: &mut Vec<f64>,
              g_cost: &mut Vec<f64>,
              f_cost: &mut Vec<f64>,
              grid: &NavGrid,
              end_node_idx: i32,
              end_node_coord: (i32, i32)
) -> i32 {
    sort_open(open_nodes, f_cost);
    let lowest_cost_node = open_nodes.pop();
    if let Some(curr_node_idx) = lowest_cost_node {
        let cni_us = curr_node_idx as usize;
        closed_nodes[cni_us] = 1;
        if end_node_idx == curr_node_idx {
            return 2; // found the target!
        }
        // we're not at target yet, check linked nodes
        for iter_idx in 0..grid.links[cni_us].len(){
            let linked_idx = *&grid.links[cni_us][iter_idx];
            let link_idx_us = linked_idx as usize;
            
            // ignore node if unwalkable or already closed
            if linked_idx == -1 || !grid.valid[link_idx_us] || closed_nodes[link_idx_us] == 1 { continue; }
            
            // check if already in the open list
            let mut node_already_open = false;
            for n in open_nodes.iter(){
                if *n == link_idx_us as i32 {
                    node_already_open = true;
                    break;
                }
            }
            // if already open, check if it's the better path
            if node_already_open{
                let new_g_cost = g_cost[cni_us] + cost_for_link_dir(iter_idx);
                if new_g_cost < g_cost[link_idx_us]{
                    g_cost[link_idx_us] = new_g_cost;
                    f_cost[link_idx_us] = new_g_cost + h_cost[link_idx_us];
                    parents[link_idx_us] = curr_node_idx;
                }
            } else { // if not, calculate and store H F G values, set parent, and add to open
                parents[link_idx_us] = curr_node_idx;
                g_cost[link_idx_us] = g_cost[cni_us] + cost_for_link_dir(iter_idx);
                let grid_location = coord_from_1d(SIDE_NODE_NO, linked_idx);
                h_cost[link_idx_us] = (end_node_coord.0 as f64 - grid_location.0 as f64).abs() + 
                    (end_node_coord.1 as f64 - grid_location.1 as f64).abs();
                f_cost[link_idx_us] = g_cost[link_idx_us] + h_cost[link_idx_us];
                open_nodes.push(linked_idx);
            }
        }
        return 1;
    }
    else {
        error!("Pathfinding ran out of open nodes!");
        return 0;
    }
    
    // return values: 0 = path failed, no options remain
    //                1 = destination not reached, but more options remain
    //                2 = destination reached
}

fn build_path(
    parents: &Vec<i32>, 
    target_index: usize,
    grid: &NavGrid
) -> Vec<Vec2> {
    let mut output = Vec::<Vec2>::new();
    output.push(grid.positions[target_index]);
    let mut parent = parents[target_index];
    while parent != -1{
        output.push(grid.positions[parent as usize]);
        parent = parents[parent as usize];
    }
    
    output.reverse();
    output
}

fn cost_for_link_dir(link_dir: usize) -> f64 {
    if link_dir == 0 || link_dir == 2 || link_dir == 5 || link_dir == 7
        { return 1.4 } // diagonal links are proportionally higher cost, due to being longer
    1.0
}

fn sort_open(open_vec: &mut Vec<i32>, g_cost_vec: &Vec<f64>){
    open_vec.sort_unstable_by(|a,b| g_cost_vec[*b as usize].partial_cmp(&g_cost_vec[*a as usize]).unwrap())
}

pub fn draw_links(
    mut lines: ResMut<DebugLines>,
    nav_grid: Res<NavGrid>,
    player_q: Query<&Transform, With<Player>>
){
    if !nav_grid.complete {return;}
    
    let ppos = player_q.single().translation.truncate();
    
    for idx in 0..nav_grid.positions.len() {
        if !nav_grid.valid[idx] {continue};
        let this_node_pos = nav_grid.positions[idx];
        if (this_node_pos - ppos).length_squared() > 500000.0 {continue} 
        let links = &nav_grid.links[idx];
        for link in links{
            if *link < 0 { continue }
            if !nav_grid.valid[*link as usize] {continue};
            let linked_pos = nav_grid.positions[*link as usize];
            lines.line_colored(this_node_pos.extend(0.1), linked_pos.extend(0.1), 1.0/5.0, Color::rgba(0.3, 0.3, 0.3, 0.12));
        }
    }
}

pub fn draw_cells(
    mut cmd: Commands, 
    q: Query<Entity, With<NodeDebugToken>>, 
    nav_grid: Option<Res<NavGrid>>,
    q_player: Query<&Transform, With<Player>>
) {
    if let Some(grid) = nav_grid{
        if !grid.complete { return; }

        for e in q.iter(){
            cmd.entity(e).despawn();
        }
        const DISPLAY_DIR_NO :i32 = 10;
        let ppos = q_player.single().translation.truncate();
        let offset = Vec2::new(OFFSET_X + ppos.x, ppos.y + OFFSET_Y);
        let mut coords = offset * RESOLUTION;
        let rounded_x = f32::round(coords.x) as i32;
        let rounded_y = f32::round(coords.y) as i32;

        for x in -DISPLAY_DIR_NO..DISPLAY_DIR_NO{
            for y in -DISPLAY_DIR_NO..DISPLAY_DIR_NO{
                let idx = coord_to_1d(SIDE_NODE_NO, rounded_x+x, rounded_y+y) as usize;
                let pos = grid.positions[idx];

                let is_valid = grid.valid[idx];
                let mut color = Color::WHITE;
                if is_valid {
                    let blue = grid.costs[idx] - 1.0;
                    color = Color::rgba(0.0, 1.0 - blue, blue, 0.08);
                }
                else {
                    color = Color::rgba(1.0, 0.0, 0.0, 0.08);
                }

                cmd.spawn_bundle(SpriteBundle{
                    sprite: Sprite {
                        color,
                        custom_size: Option::from(Vec2::new(12.0, 12.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(pos.extend(0.5)),
                    ..Default::default()
                })
                    .insert(NodeDebugToken);
                
            }
        }
    } else { return; }
        
}

pub fn draw_path_to_debug_point(
    mut lines: ResMut<DebugLines>,
    nav_grid: Option<Res<NavGrid>>,
    player_q: Query<&Transform, With<Player>>,
){
    if let Some(grid) = nav_grid{
        if !grid.complete { return; }
        let ppos = player_q.single().translation;
        let res = calculate_path(ppos.truncate(), Vec2::new(1418.4176, -1949.4218), &grid);
        for pt in 1..res.len(){
            lines.line_colored(res[pt-1].extend(0.1), res[pt].extend(0.1), 1.0, Color::rgba(0.9, 0.3, 0.3, 0.12));
        }
    } else {
        return;
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
    //info!("Checked {} colliders, no collision found", c.to_string());

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

fn coord_to_1d(side_dim: i32, x: i32, y:i32) -> i32 {
    y * side_dim + x
}

fn closest_cell_1d(
    position: Vec2, 
    grid_offset: Vec2, 
    grid_size: i32,
    grid_resolution: f32
) -> i32 {
    let coord = closest_cell_2d(position, grid_offset, grid_size, grid_resolution);
    return coord_to_1d(SIDE_NODE_NO, coord.0, coord.1);
}

fn closest_cell_2d(
    position: Vec2, 
    grid_offset: Vec2, 
    grid_size: i32,
    grid_resolution: f32
) -> (i32,i32) {
    let pos_in_gridspace = Vec2::new(grid_offset.x + position.x, grid_offset.y + position.y);
    let mut coords = pos_in_gridspace * grid_resolution;
    // clamp result to grid size
    let mut rounded_x = i32::clamp(f32::round(coords.x) as i32, 0, grid_size-1);
    let mut rounded_y = i32::clamp(f32::round(coords.y) as i32, 0, grid_size-1);
    
    //debug!("Cell closest to {} is x:{} y:{}", position.to_string(), rounded_x.to_string(), rounded_y.to_string());
    return (rounded_x, rounded_y);
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
mod tests {
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
    fn test_1d_to_coord() {
        let a = coord_from_1d(4, 0);
        let b = coord_from_1d(4, 15);
        let c = coord_from_1d(4, 3);
        let d = coord_from_1d(4, 10);
        assert_eq!(a, (0, 0));
        assert_eq!(b, (3, 3));
        assert_eq!(c, (3, 0));
        assert_eq!(d, (2, 2));
    }

    #[test]
    fn test_get_neighbours() {
        let mut a = get_neighbours(4, 1, 1);
        assert_eq!(a, vec![0, 1, 2, 4, 6, 8, 9, 10]);
        a = get_neighbours(4, 0, 0);
        assert_eq!(a, vec![-1, -1, -1, -1, 1, -1, 4, 5]);
        a = get_neighbours(4, 3, 3);
        assert_eq!(a, vec![10, 11, -1, 14, -1, -1, -1, -1]);
    }

    #[test]
    fn test_sort_open_ordering() {
        let mut vec_a = vec![0, 1, 2, 3];
        let vec_ref = vec![7.0, 1.0, 5.0, 12.0];
        sort_open(&mut vec_a, &vec_ref);
        let pop1 = vec_a.pop().unwrap();
        let pop2 = vec_a.pop().unwrap();
        let pop3 = vec_a.pop().unwrap();
        assert_eq!(pop1, 1);
        assert_eq!(pop2, 2);
        assert_eq!(pop3, 0);
    }

    #[test]
    fn closest_node_search_clamps() {
        let side_size = 1000;
        let resolution = 0.01;
        let position = Vec2::new(-3000.0, 5000.0);
        let result = closest_cell_2d(position, Vec2::ZERO, side_size, resolution);
        println!("result x:{} y:{}", result.0.to_string(), result.1.to_string());
        assert_eq!(result.0, 0);
        assert_eq!(result.1, 9);
    }
}