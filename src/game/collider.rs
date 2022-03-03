use bevy::{prelude::*, ecs::system::EntityCommands};

#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Wall;
#[derive(Component, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct HurtZone;
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
            ColliderKind::HurtZone => cmd.insert(HurtZone),
            ColliderKind::WinZone => cmd.insert(WinZone),
            ColliderKind::SpawnZone => cmd.insert(SpawnZone),
        };
    }
}
