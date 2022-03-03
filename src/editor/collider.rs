use bevy::prelude::*;
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
