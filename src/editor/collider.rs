use bevy::{prelude::*, math::{const_vec2, Mat2}, input::mouse::MouseMotion};
use heron::CollisionShape;

use crate::{util::{WorldCursor, WorldCursorPrev}};

use super::select::Selection;

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct EditableCollider {
    pub half_extends: Vec2,
}

impl Default for EditableCollider {
    fn default() -> Self {
        EditableCollider {
            half_extends: Vec2::new(40., 30.),
        }
    }
}

#[derive(Component)]
pub struct ColliderEditorVisColor(pub Color);

#[derive(Component)]
pub struct ColliderVisualized;

pub fn visualize_spriteless_colliders(
    mut cmd: Commands,
    q: Query<(
        Entity,
        &EditableCollider,
        &ColliderEditorVisColor
    ), (
        Without<Sprite>,
        Without<ColliderVisualized>
    )>
) {
    for (e, edit, viscolor) in q.iter() {
        let bundle = SpriteBundle {
            sprite: Sprite {
                color: viscolor.0,
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

const DRAGHANDLE_RADIUS: f32 = 8.0;

#[derive(Debug, Component, Clone, Copy)]
pub struct DragHandle {
    drag: Vec2,
    target: Entity,
}

#[derive(Default)]
pub struct ActiveDraghandle(Option<(DragHandle, f32)>);

pub fn mouse_select_draghandle(
    crs: Res<WorldCursor>,
    mut dh_active: ResMut<ActiveDraghandle>,
    btn: Res<Input<MouseButton>>,
    q_draghandle: Query<(&GlobalTransform, &DragHandle)>,
) {
    if btn.just_pressed(MouseButton::Left) {
        let mut best = None;
        for (xf, dh) in q_draghandle.iter() {
            let minv = xf.compute_matrix().inverse();
            let pos_model = minv.transform_point3(crs.0.extend(xf.translation.z)).truncate();
            let distance = pos_model.distance(Vec2::ZERO);
            if distance > DRAGHANDLE_RADIUS {
                continue;
            }
            let rot = xf.rotation.to_axis_angle().1;
            if let Some((d, _)) = best {
                if distance < d {
                    best = Some((distance, (*dh, rot)));
                }
            } else {
                best = Some((distance, (*dh, rot)));
            }
        }
        dh_active.0 = best.map(|(_, dh)| dh);
    }
    if btn.just_released(MouseButton::Left) {
        dh_active.0 = None;
    }
}

pub fn mouse_drag_handle(
    mut motion: EventReader<MouseMotion>,
    dh: Res<ActiveDraghandle>,
    mut q_tgt: Query<(&mut Transform, &mut EditableCollider)>,
) {
    const COLLIDER_MINSIZE: f32 = 4.0;
    if let Some((dh, rot)) = dh.0 {
        if let Ok((mut xf, mut edit)) = q_tgt.get_mut(dh.target) {
            let mut delta = Vec2::ZERO;
            for ev in motion.iter() {
                delta += ev.delta;
            }
            delta.y = -delta.y;
            delta /= 2.0;
            delta *= dh.drag;

            if edit.half_extends.x + delta.x < COLLIDER_MINSIZE {
                delta.x = 0.0;
            }
            if edit.half_extends.y + delta.y < COLLIDER_MINSIZE {
                delta.y = 0.0;
            }

            // let mrot = Mat2::from_angle(rot);

            // xf.translation += (mrot * (delta * dh.drag)).extend(0.0);
            // edit.half_extends += mrot * delta;
            xf.translation += (delta * dh.drag).extend(0.0);
            edit.half_extends += delta;
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
