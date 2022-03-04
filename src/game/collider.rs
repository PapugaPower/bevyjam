use std::time::Duration;

use bevy::{prelude::*, ecs::system::EntityCommands};
use heron::CollisionShape;

use crate::editor::collider::EditableCollider;

use super::damage::{Pulsing, DamageAreaShape};

#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Wall;
#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct HurtZone {
    interval_secs: f32,
    damage: f32,
}
#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct WinZone;
#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct SpawnZone;

#[derive(Component, Clone, Copy)]
pub enum ColliderKind {
    Wall,
    HurtZone,
    WinZone,
    SpawnZone,
}

impl ColliderKind {
    pub fn insert(&self, cmd: &mut EntityCommands) {
        match self {
            ColliderKind::Wall => cmd.insert(Wall),
            ColliderKind::HurtZone => cmd.insert(HurtZone::default()), // FIXME editor cloning
            ColliderKind::WinZone => cmd.insert(WinZone),
            ColliderKind::SpawnZone => cmd.insert(SpawnZone),
        };
    }
}

impl From<&HurtZone> for Pulsing {
    fn from(hz: &HurtZone) -> Self {
        Pulsing {
            pulse_time: Timer::new(Duration::from_secs_f32(hz.interval_secs), true),
            damage: hz.damage,
        }
    }
}

impl Default for HurtZone {
    fn default() -> Self {
        HurtZone {
            interval_secs: 0.5,
            damage: 5.0,
        }
    }
}

pub fn collider_apply_sync(
    mut q: Query<(
        Entity,
        &EditableCollider,
        Option<&mut CollisionShape>
    ), (
        Changed<EditableCollider>,
        Without<HurtZone>,
    )>,
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

pub fn collider_apply_sync_hurtzone(
    mut q: Query<(
        Entity,
        &EditableCollider,
        Option<&mut DamageAreaShape>
    ), (
        Changed<EditableCollider>,
        With<HurtZone>,
    )>,
    mut cmd: Commands,
) {
    for (e, edit, shape) in q.iter_mut() {
        if let Some(mut shape) = shape {
            match &mut *shape {
                DamageAreaShape::Cuboid { half_extends } => {
                    *half_extends = edit.half_extends.extend(half_extends.z);
                }
                _ => {
                    cmd.entity(e).remove::<EditableCollider>();
                }
            }
        } else {
            cmd.entity(e).insert(DamageAreaShape::Cuboid {
                half_extends: edit.half_extends.extend(100.0),
            });
        }
    }
}
