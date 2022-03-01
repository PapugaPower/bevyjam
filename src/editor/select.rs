use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::util::WorldCursor;
use crate::editor::controls::EditableSprite;

use super::UsingTool;

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

pub fn mouse_select_sprite(
    crs: Res<WorldCursor>,
    btn: Res<Input<MouseButton>>,
    q: Query<(Entity, &GlobalTransform, &Sprite, &Handle<Image>), With<EditableSprite>>,
    imgs: Res<Assets<Image>>,
    mut cmd: Commands,
    mut sels: ResMut<Selections>,
    tool: Res<UsingTool>,
) {
    if *tool != UsingTool::Select {
        return;
    }

    if btn.just_pressed(MouseButton::Left) {
        let mut best = None;
        for (e, xf, spr, h_img) in q.iter() {
            let minv = xf.compute_matrix().inverse();
            let pos_model = minv.transform_point3(crs.0.extend(xf.translation.z));

            let spr_sz = spr.custom_size
                .or_else(|| {
                    imgs.get(h_img)
                        .map(|img| {
                            let isz = img.texture_descriptor.size;
                            Vec2::new(isz.width as f32, isz.height as f32)
                        })
                }).unwrap_or(Vec2::new(2.0, 2.0)) / 2.0;

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
                cmd.entity(sel).despawn();
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
