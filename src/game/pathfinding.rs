use std::cmp::Ordering;
use std::collections::HashSet;
use binary_heap_plus::{BinaryHeap, MinComparator};
use std::f32::consts::PI;
use std::time::Instant;
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
use crate::game::nav_grid;
use crate::game::nav_grid::{NavGrid, OFFSET_X, OFFSET_Y, RESOLUTION, SIDE_NODE_NO};
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;

#[derive(Default)]
pub struct AStarBuffer {
    pub parents: Vec<i32>,
    pub h_cost: Vec<f64>,
    pub g_cost: Vec<f64>,
    pub f_cost: Vec<f64>,
    pub state: Vec<u8>, // 0 - unchecked, 1 - open, 2 - closed
    pub open_sorted_idxs: Vec<i32>,
}

#[derive(Component)]
pub struct NodeDebugToken;

#[derive(Default)]
pub struct GridState{
    pub ran: bool,
}

pub fn calculate_path(
    from: Vec2, 
    to: Vec2,
    grid: &NavGrid,
    buffer: &mut AStarBuffer
) -> Vec<Vec2> {
    let timer = Instant::now();
    let mut output = vec![from];
    let start_node_idx = nav_grid::closest_cell_1d(from, Vec2::new(OFFSET_X, OFFSET_Y), SIDE_NODE_NO, RESOLUTION);
    let end_node_idx = nav_grid::closest_cell_1d(to, Vec2::new(OFFSET_X, OFFSET_Y), SIDE_NODE_NO, RESOLUTION);
    let end_node_coord = nav_grid::coord_from_1d(SIDE_NODE_NO, end_node_idx);
    
    // ^ i used a flattened grid vector for each parameter,
    //   since it was easier to reason about, but it's not optimal

    buffer.open_sorted_idxs.clear();
    buffer.open_sorted_idxs.push(start_node_idx);
    buffer.g_cost[start_node_idx as usize] = 0.0;
    
    for n in 0..buffer.state.len(){
        buffer.state[n] = 0; // reset node search states
        buffer.parents[n] = -1; // clear parents
        buffer.f_cost[n] = 999.0;
        buffer.g_cost[n] = 999.0;
        buffer.h_cost[n] = 999.0;
    }

    let mut last_result = 1;
    let mut iters = 5000;

    
    while last_result == 1 {
        last_result = check_node(
            &mut buffer.open_sorted_idxs,
            &mut buffer.state,
            &mut buffer.parents,
            &mut buffer.h_cost,
            &mut buffer.g_cost,
            &mut buffer.f_cost,
            &grid,
            end_node_idx,
            end_node_coord);
        iters -= 1;
        if iters < 0 { 
            error!("A* exceeded 2500 iterations for a single path!");
            last_result = 0;
            let last_idx = buffer.open_sorted_idxs.pop().unwrap();
            output = build_path(&buffer.parents, last_idx as usize, &grid);
        }
    }

    if last_result == 0 {
        //error!("A* couldn't find path, returning start node.");
        //output.push(grid.positions[start_node_idx as usize]);
    } else {
        //info!("A* built a complete path.");
        output = build_path(&buffer.parents, end_node_idx as usize, &grid);

    }
    //info!("1st node x: {} y: {}", output[0].x.to_string(), output[0].y.to_string());
    let elapsed = timer.elapsed().as_nanos() as f64 / 1000000.0;
    info!("Pathfinding time: {}ms, iter: {}", elapsed.to_string(), (5000 - iters).to_string());
    output
}

// Note: A* using Manhattan heuristic
fn check_node(mut open_nodes: &mut Vec<i32>,
              mut node_states: &mut Vec<u8>,
              mut parents: &mut Vec<i32>,
              mut h_cost: &mut Vec<f64>,
              mut g_cost: &mut Vec<f64>,
              mut f_cost: &mut Vec<f64>,
              grid: &NavGrid,
              end_node_idx: i32,
              end_node_coord: (i32, i32)
) -> i32 {
    let lowest_cost_node = open_nodes.pop();
    if let Some(curr_node_idx) = lowest_cost_node {
        let cn_idx_us = curr_node_idx as usize;
        node_states[cn_idx_us] = 2;
        if end_node_idx == curr_node_idx {
            return 2; // found the target!
        }
        // we're not at target yet, check linked nodes
        for iter_idx in 0..grid.links[cn_idx_us].len(){
            let linked_idx = *&grid.links[cn_idx_us][iter_idx];
            let link_idx_us = linked_idx as usize;
            
            // ignore node if unwalkable or already closed
            if linked_idx == -1 || !grid.valid[link_idx_us] || node_states[link_idx_us] == 2 { continue; }
            
            // check if already in the open list
            let mut node_already_open = open_nodes.contains(&linked_idx);
            
            // if already open, check if it's the better path
            if node_already_open{
                let new_g_cost = g_cost[cn_idx_us] + cost_for_link_dir(iter_idx);
                if new_g_cost < g_cost[link_idx_us]{
                    g_cost[link_idx_us] = new_g_cost;
                    f_cost[link_idx_us] = new_g_cost + h_cost[link_idx_us];
                    parents[link_idx_us] = curr_node_idx as i32;
                    // Remove idx and re-insert with new values
                    pluck_from_open(&mut open_nodes, linked_idx);
                    add_to_open(&mut open_nodes,f_cost, linked_idx, f_cost[link_idx_us]);

                }
            } else { // if not, calculate and store H F G values, set parent, and add to open
                parents[link_idx_us] = curr_node_idx;
                g_cost[link_idx_us] = g_cost[cn_idx_us] + cost_for_link_dir(iter_idx);
                let grid_location = nav_grid::coord_from_1d(SIDE_NODE_NO, linked_idx);
                h_cost[link_idx_us] = (end_node_coord.0 as f64 - grid_location.0 as f64).abs() + 
                    (end_node_coord.1 as f64 - grid_location.1 as f64).abs();
                f_cost[link_idx_us] = g_cost[link_idx_us] + h_cost[link_idx_us];
                add_to_open(open_nodes, f_cost, linked_idx, f_cost[link_idx_us]);
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

fn add_to_open(open_vector: &mut Vec<i32>, f_cost_vector: &mut Vec<f64>, node_idx: i32, f_cost: f64) {
    if open_vector.len() == 0{
        open_vector.push(node_idx);
        return;
    }
    let search_result = open_vector.binary_search_by(|a:&i32|
        return if f_cost_vector[*a as usize] > f_cost {
            Ordering::Less
        } else if f64::abs(f_cost_vector[*a as usize] - f_cost) < 0.0001 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    );
    match search_result {
        Ok(idx) => {
            open_vector.insert(idx, node_idx);
        },
        Err(idx) => {
            open_vector.insert(idx, node_idx);
        }
    }
}

fn pluck_from_open(open_vector: &mut Vec<i32>, node_idx: i32){
    if open_vector.len() == 0 {return;}
    for item in 0..open_vector.len(){
        if open_vector[item] == node_idx{
            open_vector.remove(item);
            return;
        }
    }
    return;
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

pub fn draw_path_to_debug_point(
    mut lines: ResMut<DebugLines>,
    nav_grid: Option<Res<NavGrid>>,
    mut buffer: Option<ResMut<AStarBuffer>>,
    player_q: Query<&Transform, With<Player>>,
){
    if let Some(grid) = nav_grid{
        if !grid.complete { return; }
        let mut bfr = buffer.unwrap();
        let ppos = player_q.single().translation;
        let res = calculate_path(ppos.truncate(), Vec2::new(1418.4176, -1949.4218), &grid, &mut bfr);
        for pt in 1..res.len(){
            lines.line_colored(res[pt-1].extend(0.1), res[pt].extend(0.1), 0.01, Color::rgba(1.0, 0.1, 0.1, 0.52));
        }

    } else {
        return;
    }
}

pub fn draw_closed_nodes(
    mut cmd: Commands,
    q: Query<Entity, With<NodeDebugToken>>,
    b: Option<Res<AStarBuffer>>,
    grid: Option<Res<NavGrid>>,
) {
    if let Some(buffer) = b {
        if let Some(g) = grid {
            for e in q.iter() {
                cmd.entity(e).despawn();
            }

            for idx in 0..buffer.state.len() {
                let state = buffer.state[idx];
                if state == 1 {
                    let pos = g.positions[idx];
                    let color = Color::rgba(0.0, 1.0, 0.0, 0.3);
                    cmd.spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Option::from(Vec2::new(12.0, 12.0)),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(pos.extend(0.5)),
                        ..Default::default()
                    })
                        .insert(NodeDebugToken);
                } else if state == 2 {
                    let pos = g.positions[idx];
                    let color = Color::rgba(1.0, 0.0, 0.2, 0.3);
                    cmd.spawn_bundle(SpriteBundle {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::nav_grid::{closest_cell_2d, coord_from_1d, coord_to_1d, get_neighbours, is_point_within_rect, rect_circle_overlap};
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
    
    #[test]
    fn test_adding_to_open(){
        let mut vec_a = vec![3, 0, 2, 1];
        let mut vec_ref = vec![7.0, 1.0, 5.0, 12.0, 0.5];
        add_to_open(&mut vec_a, &mut vec_ref, 4, 0.5);
        assert_eq!(vec_a[4], 4);
        vec_ref.push(10.0);
        add_to_open(&mut vec_a, &mut vec_ref, 5, 0.5);
        assert_eq!(vec_a[5], 4);
        vec_ref.push(0.2);
        add_to_open(&mut vec_a, &mut vec_ref, 6, 0.2);
        assert_eq!(vec_a[6], 6);
        vec_ref[0] = 0.1;
        pluck_from_open(&mut vec_a, 0);
        add_to_open(&mut vec_a, &mut vec_ref, 0, 0.1);
        assert_eq!(vec_a[6], 0);
        vec_a.pop();
        assert_eq!(vec_a[5], 6);
    }

    #[test]
    fn test_plucking(){
        let mut vec_a = vec![1,2,3,4,5,6,78,97,123,515,621];
        pluck_from_open(&mut vec_a, 3);
        pluck_from_open(&mut vec_a, 1);
        pluck_from_open(&mut vec_a, 2);
        pluck_from_open(&mut vec_a, 4);
        pluck_from_open(&mut vec_a, 515);
        pluck_from_open(&mut vec_a, 97);
        pluck_from_open(&mut vec_a, 5);
        pluck_from_open(&mut vec_a, 6);
        pluck_from_open(&mut vec_a, 78);
        pluck_from_open(&mut vec_a, 123);
        assert_eq!(vec_a[0], 621);
    }
    
}