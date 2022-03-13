use std::f32::consts::PI;
use bevy::log::info;
use heron::CollisionShape;
use bevy_prototype_debug_lines::DebugLines;
use crate::{Color, Commands, Entity, EulerRot, Local, Query, Res, ResMut, Sprite, SpriteBundle, Transform, Vec2, With};
use crate::game::collider::Wall;
use crate::game::pathfinding::{AStarBuffer, GridState, NodeDebugToken};
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

const SIZE: f32 = 11000.0;
pub const RESOLUTION: f32 = 0.015;
// points per unit
pub const SIDE_NODE_NO: i32 = (RESOLUTION * SIZE) as i32;
pub const OFFSET_X: f32= SIZE /2.0 + 2000.0;
pub const OFFSET_Y: f32= SIZE /2.0 + 2000.0;

/*
 0 1 2
  \|/
 3-x-4   <- neighbour vector layout
  /|\
 5 6 7 
 */
pub fn generate_grid(
    q: Query<(&Transform, &CollisionShape), With<Wall>>, 
    mut state: Local<GridState>, 
    mut nav_grid: ResMut<NavGrid>,
    mut astar_buffer: ResMut<AStarBuffer>
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

    // init astar buffer
    astar_buffer.parents = vec![-1; nav_grid.positions.len()];
    astar_buffer.state = vec![0; nav_grid.positions.len()];
    astar_buffer.h_cost = vec![-1.0; nav_grid.positions.len()];
    astar_buffer.g_cost = vec![-1.0; nav_grid.positions.len()];
    astar_buffer.f_cost = vec![-1.0; nav_grid.positions.len()];
    astar_buffer.open_sorted_idxs = Vec::<i32>::with_capacity(nav_grid.positions.len());
    
    nav_grid.complete = true;
    state.ran = true;
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

pub fn get_neighbours(side_dim: i32, curr_x: i32, curr_y:i32) -> Vec<i32> {
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

pub fn coord_from_1d(side_dim: i32, idx: i32) -> (i32, i32) {
    let x = idx % side_dim;
    let y = (idx - x) / side_dim;
    (x,y)
}

pub fn coord_to_1d(side_dim: i32, x: i32, y:i32) -> i32 {
    y * side_dim + x
}

pub fn closest_cell_1d(
    position: Vec2, 
    grid_offset: Vec2, 
    grid_size: i32,
    grid_resolution: f32
) -> i32 {
    let coord = closest_cell_2d(position, grid_offset, grid_size, grid_resolution);
    return coord_to_1d(SIDE_NODE_NO, coord.0, coord.1);
}

pub fn closest_cell_2d(
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

pub fn rect_circle_overlap(rect_pos: Vec2, rect_dim: Vec2, rect_rot_z: f32, c_pos: Vec2, c_radius: f32)
                           -> bool {
    let rel = c_pos - rect_pos;
    let local_x = f32::cos(rect_rot_z) * rel.x + f32::cos(rect_rot_z - PI * 0.5) * rel.y;
    let local_y = f32::sin(rect_rot_z) * rel.x + f32::sin(rect_rot_z - PI * 0.5) * rel.y;

    f32::abs(local_x) - c_radius < rect_dim.x && f32::abs(local_y) - c_radius < rect_dim.y
}

pub fn is_point_within_rect(point: Vec2, rect_pos: Vec2, rect_dim: Vec2, rect_rot_z: f32)
                            -> bool {
    let rel = point - rect_pos;
    let local_x = f32::cos(rect_rot_z) * rel.x + f32::cos(rect_rot_z - PI * 0.5) * rel.y;
    let local_y = f32::sin(rect_rot_z) * rel.x + f32::sin(rect_rot_z - PI * 0.5) * rel.y;

    f32::abs(local_x) < rect_dim.x && f32::abs(local_y) < rect_dim.y

}
