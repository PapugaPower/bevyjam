use bevy::{prelude::*, math::Vec3Swizzles, input::mouse::MouseMotion};

use crate::util::{WorldCursor, WorldCursorPrev, MainCamera};

use super::{select::Selection, UsingTool, NewlySpawned, ToolState};

pub fn editor_camera(
    mut q_cam: Query<&mut Transform, With<MainCamera>>,
    mut evr_motion: EventReader<MouseMotion>,
    btn: Res<Input<MouseButton>>,
) {
    if btn.pressed(MouseButton::Right) {
        let mut xf = q_cam.single_mut();
        let mut delta = Vec2::ZERO;
        for ev in evr_motion.iter() {
            delta += ev.delta;
        }
        xf.translation.x -= delta.x;
        xf.translation.y += delta.y;
    }
}

pub fn mouse_move_selections(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    q_sel: Query<&Selection>,
    mut q_tgt: Query<&mut Transform, Without<NewlySpawned>>,
    btn: Res<Input<MouseButton>>,
) {
    if btn.pressed(MouseButton::Left) {
        // TODO: does not work in children in hierarchy;
        // needs reverse transform propagation
        let delta = crs.0 - crs_old.0;
        for sel in q_sel.iter() {
            if let Ok(mut xf) = q_tgt.get_mut(sel.0) {
                xf.translation.x += delta.x;
                xf.translation.y += delta.y;
            }
        }
    }
}

pub(super) fn mouse_move_newlyspawned(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    mut q_tgt: Query<(Entity, &mut Transform), With<NewlySpawned>>,
    mut btn: ResMut<Input<MouseButton>>,
    mut commands: Commands,
    mut toolstate: ResMut<State<ToolState>>,
    tool: Res<UsingTool>,
) {
    for (e, mut xf) in q_tgt.iter_mut() {
        let delta = crs.0 - crs_old.0;
        xf.translation.x += delta.x;
        xf.translation.y += delta.y;
        if btn.just_pressed(MouseButton::Left) {
            btn.clear_just_pressed(MouseButton::Left);
            toolstate.push(ToolState::Using(*tool)).unwrap();
            commands.entity(e).remove::<NewlySpawned>();
        }
    }
}

pub fn mouse_rotate_selections(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    q_sel: Query<&Selection>,
    mut q_tgt: Query<&mut Transform, Without<NewlySpawned>>,
    btn: Res<Input<MouseButton>>,
) {
    if btn.pressed(MouseButton::Left) {
        // TODO: does not work in children in hierarchy;
        // needs reverse transform propagation
        for sel in q_sel.iter() {
            if let Ok(mut xf) = q_tgt.get_mut(sel.0) {
                let ray0 = crs_old.0 - xf.translation.xy();
                let ray1 = crs.0 - xf.translation.xy();
                let angle = ray0.angle_between(ray1);
                xf.rotate(Quat::from_rotation_z(angle));
            }
        }
    }
}
