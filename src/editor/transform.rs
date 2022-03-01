use bevy::{prelude::*, math::Vec3Swizzles};

use crate::util::{WorldCursor, WorldCursorPrev};

use super::{select::Selection, UsingTool, controls::EditableSprite, NewlySpawned};

pub fn mouse_move_selections(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    q_sel: Query<&Selection>,
    mut q_tgt: Query<&mut Transform, With<EditableSprite>>,
    tool: Res<UsingTool>,
    btn: Res<Input<MouseButton>>,
) {
    if *tool != UsingTool::Move {
        return;
    }

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

pub fn mouse_move_newlyspawned(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    mut q_tgt: Query<(Entity, &mut Transform), With<NewlySpawned>>,
    btn: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    for (e, mut xf) in q_tgt.iter_mut() {
        let delta = crs.0 - crs_old.0;
        xf.translation.x += delta.x;
        xf.translation.y += delta.y;
        if btn.just_pressed(MouseButton::Left) {
            commands.entity(e).remove::<NewlySpawned>();
        }
    }
}

pub fn mouse_rotate_selections(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    q_sel: Query<&Selection>,
    mut q_tgt: Query<&mut Transform, With<EditableSprite>>,
    tool: Res<UsingTool>,
    btn: Res<Input<MouseButton>>,
) {
    if *tool != UsingTool::Rotate {
        return;
    }

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
