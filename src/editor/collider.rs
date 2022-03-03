use bevy::{prelude::*, math::const_vec2};
use heron::CollisionShape;

use crate::util::{WorldCursor, WorldCursorPrev};

use super::select::Selection;

#[derive(Component)]
pub struct EditableCollider {
    pub half_extends: Vec2,
}

#[derive(Component)]
pub struct ColliderVisualized;

pub fn visualize_spriteless_colliders(
    mut cmd: Commands,
    q: Query<(Entity, &EditableCollider), (Without<Sprite>, Without<ColliderVisualized>)>
) {
    for (e, edit) in q.iter() {
        let bundle = SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 0.75, 0.5, 0.25),
                custom_size: Some(edit.half_extends * 2.0),
                ..Default::default()
            },
            ..Default::default()
        };
        cmd.entity(e)
            .insert(ColliderVisualized)
            .insert(bundle.sprite)
            .insert(bundle.texture)
            .insert(bundle.visibility);
    }
}

pub fn update_collider_visualization(
    mut q: Query<(&mut Sprite, &EditableCollider), With<ColliderVisualized>>
) {
    for (mut spr, edit) in q.iter_mut() {
        spr.custom_size = Some(edit.half_extends * 2.0);
    }
}

pub fn cleanup_collider_visualizations(
    mut cmd: Commands,
    q: Query<Entity, With<ColliderVisualized>>,
) {
    for e in q.iter() {
        cmd.entity(e)
            .remove::<ColliderVisualized>()
            .remove::<Sprite>()
            .remove::<Handle<Image>>();
    }
}

pub fn collider_apply_sync(
    mut q: Query<(Entity, &EditableCollider, Option<&mut CollisionShape>)>,
    mut cmd: Commands,
) {
    for (e, edit, shape) in q.iter_mut() {
        if let Some(mut shape) = shape {
            match &mut *shape {
                CollisionShape::Cuboid { half_extends, border_radius: _ } => {
                    *half_extends = edit.half_extends.extend(half_extends.z);
                }
                _ => {
                    cmd.entity(e).remove::<EditableCollider>();
                }
            }
        } else {
            cmd.entity(e).insert(CollisionShape::Cuboid {
                half_extends: edit.half_extends.extend(100.0),
                border_radius: None,
            });
        }
    }
}

const DRAGHANDLE_RADIUS: f32 = 8.0;

#[derive(Debug, Component, Clone, Copy)]
pub struct DragHandle {
    drag: Vec2,
    target: Entity,
}

pub fn mouse_edit_collider(
    crs: Res<WorldCursor>,
    crs_old: Res<WorldCursorPrev>,
    btn: Res<Input<MouseButton>>,
    q_draghandle: Query<(&GlobalTransform, &DragHandle)>,
    mut q_tgt: Query<(&mut Transform, &mut EditableCollider)>,
) {
    if btn.pressed(MouseButton::Left) {
        let mut best = None;
        for (xf, dh) in q_draghandle.iter() {
            let minv = xf.compute_matrix().inverse();
            let pos_model = minv.transform_point3(crs.0.extend(xf.translation.z)).truncate();
            let distance = pos_model.distance(Vec2::ZERO);
            if distance > DRAGHANDLE_RADIUS {
                continue;
            }
            if let Some((d, _)) = best {
                if distance < d {
                    best = Some((distance, *dh));
                }
            } else {
                best = Some((distance, *dh));
            }
        }
        if let Some((_, dh)) = best {
            if let Ok((mut xf, mut edit)) = q_tgt.get_mut(dh.target) {
                let delta = (crs.0 - crs_old.0) * dh.drag / 2.0;
                xf.translation += delta.extend(0.0);
                edit.half_extends += delta;
            }
        }
    }
}

pub fn spawn_draghandles(
    mut cmd: Commands,
    q_sel: Query<(Entity, &Selection)>,
    q_tgt: Query<&EditableCollider>,
) {
    const DRAGS: [Vec2; 4] = [
        const_vec2!([1.0, 1.0]),
        const_vec2!([-1.0, 1.0]),
        const_vec2!([1.0, -1.0]),
        const_vec2!([-1.0, -1.0]),
    ];
    for (e, sel) in q_sel.iter() {
        if let Ok(edit) = q_tgt.get(sel.0) {
            for drag in DRAGS {
                let h = cmd.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::splat(DRAGHANDLE_RADIUS)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation((edit.half_extends * drag).extend(99.0)),
                    ..Default::default()
                }).insert(DragHandle {
                    drag,
                    target: sel.0,
                }).id();
                cmd.entity(e).push_children(&[h]);
            }
        }
    }
}

pub fn draghandles_track_collider(
    mut q_dh: Query<(&mut Transform, &DragHandle)>,
    q_tgt: Query<&EditableCollider>,
) {
    for (mut xf, dh) in q_dh.iter_mut() {
        if let Ok(edit) = q_tgt.get(dh.target) {
            xf.translation = (edit.half_extends * dh.drag).extend(99.0);
        }
    }
}
