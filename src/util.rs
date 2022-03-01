use bevy::prelude::*;

use crate::FuckStages;

pub struct WorldCursor(pub Vec2);

#[derive(Component)]
pub struct MainCamera;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldCursor(Vec2::ZERO));
        app.add_system_to_stage(FuckStages::Pre, world_cursor_system);
    }
}

fn world_cursor_system(
    mut crs: ResMut<WorldCursor>,
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    if let Ok((camera, camera_transform)) = q_camera.get_single() {
        // get the window that the camera is displaying to
        let wnd = wnds.get(camera.window).unwrap();

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = wnd.cursor_position() {
            // get the size of the window
            let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            let world_pos: Vec2 = world_pos.truncate();

            crs.0 = world_pos;
        }
    }
}
