use std::time::Duration;

use bevy::{prelude::*, ecs::system::EntityCommands};
use heron::CollisionShape;

use crate::editor::collider::EditableCollider;

use super::{damage::{Pulsing, DamageAreaShape}, blueprints::ColliderBehavior};

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

