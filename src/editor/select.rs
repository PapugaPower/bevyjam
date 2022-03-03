use bevy::prelude::*;
use bevy::utils::HashMap;
use heron::CollisionShape;

use crate::util::{WorldCursor, WorldCursorPrev};

use super::{UsingTool, NoEditor, NewlySpawned, collider::EditableCollider};

const SELECTION_COLOR: Color = Color::rgba(1.0, 0.0, 1.0, 0.5);

/// Map of (Target Entity) -> (Selection Entity)
#[derive(Default)]
pub struct Selections(HashMap<Entity, Entity>);

#[derive(Component)]
pub struct Selection(pub Entity);

#[derive(Bundle)]
struct SelectionBundle {
    #[bundle]
    sprite: SpriteBundle,
    selection: Selection,
}

impl SelectionBundle {
    fn new(e: Entity, sz: Vec2) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: SELECTION_COLOR,
                    custom_size: Some(sz),
                    ..Default::default()
                },
                ..Default::default()
            },
            selection: Selection(e),
        }
    }
}

pub fn keyboard_despawn_selected(
    mut cmd: Commands,
    kbd: Res<Input<KeyCode>>,
    q_sel: Query<(Entity, &Selection)>,
) {
    if kbd.just_pressed(KeyCode::Delete) || kbd.just_pressed(KeyCode::Back) {
        for (e, sel) in q_sel.iter() {
            cmd.entity(sel.0).despawn_recursive();
            cmd.entity(e).despawn_recursive();
        }
    }
}

pub fn mouse_select(
    crs: Res<WorldCursor>,
    mut btn: ResMut<Input<MouseButton>>,
    q: Query<(
        Entity,
        &GlobalTransform,
        Option<&Sprite>,
        Option<&Handle<Image>>,
        Option<&EditableCollider>
    ), (
        Without<NoEditor>,
        Without<NewlySpawned>,
        Without<Parent>,
        Or<(With<Sprite>, With<EditableCollider>)>
    )>,
    imgs: Res<Assets<Image>>,
    mut cmd: Commands,
    mut sels: ResMut<Selections>,
) {
    if btn.just_pressed(MouseButton::Left) {
        btn.clear_just_pressed(MouseButton::Left);
        let mut best = None;
        for (e, xf, spr, h_img, edit) in q.iter() {
            //dbg!(best);
            let minv = xf.compute_matrix().inverse();
            let pos_model = minv.transform_point3(crs.0.extend(xf.translation.z));
            //dbg!(pos_model);

            let spr_sz = if let Some(edit) = edit {
                //dbg!(half_extends);
                edit.half_extends
            } else if let Some(spr) = spr {
                spr.custom_size
                    .or_else(|| {
                        h_img.and_then(|h_img|
                            imgs.get(h_img)
                                .map(|img| {
                                    let isz = img.texture_descriptor.size;
                                    Vec2::new(isz.width as f32, isz.height as f32)
                                }))
                    }).unwrap_or(Vec2::new(2.0, 2.0)) / 2.0
            } else {
                continue;
            };

            if pos_model.x > -spr_sz.x && pos_model.x < spr_sz.x &&
               pos_model.y > -spr_sz.y && pos_model.y < spr_sz.y
            {
                if let Some((z, _, _)) = best {
                    if xf.translation.z > z {
                        best = Some((xf.translation.z, e, spr_sz * 2.0));
                    }
                } else {
                    best = Some((xf.translation.z, e, spr_sz * 2.0));
                }
            }
        }

        if let Some((_, e, sz)) = best {
            if sels.0.contains_key(&e) {
                let sel = sels.0.remove(&e).unwrap();
                cmd.entity(sel).despawn_recursive();
            } else {
                let sel = cmd.spawn_bundle(SelectionBundle::new(e, sz)).id();
                sels.0.insert(e, sel);
            }
        }
    }
}

pub fn set_selection_visibility<const VIS: bool>(
    mut q_sel: Query<&mut Visibility, With<Selection>>,
) {
    for mut vis in q_sel.iter_mut() {
        vis.is_visible = VIS;
    }
}

pub fn selection_track_target(
    mut q_xf: Query<&mut GlobalTransform>,
    q_sel: Query<(Entity, &Selection)>,
    mut cmd: Commands,
    mut sels: ResMut<Selections>,
) {
    for (e, sel) in q_sel.iter() {
        if let Ok(xf_target) = q_xf.get(sel.0) {
            if let Ok(mut xf) = q_xf.get_mut(e) {
                *xf = *xf_target;
            } else {
                cmd.entity(e).despawn();
                sels.0.remove(&sel.0);
            }
        } else {
            cmd.entity(e).despawn();
            sels.0.remove(&sel.0);
        }
    }
}

pub fn selection_track_collider(
    mut q_sel: Query<(&mut Sprite, &Selection)>,
    q_tgt: Query<&EditableCollider>,
) {
    for (mut spr, sel) in q_sel.iter_mut() {
        if let Ok(edit) = q_tgt.get(sel.0) {
            spr.custom_size = Some(edit.half_extends * 2.0);
        }
    }
}
