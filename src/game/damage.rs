use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max: f32,
    pub current: f32
}

#[derive(Debug, Clone, Copy)]
pub enum DamageSource {
    Weapon,
    Enemy,
    Environment,
}

#[derive(Debug, Clone, Copy)]
pub struct DamageEvent {
    pub entity: Entity,
    pub source: DamageSource,
    pub damage: f32,
}

pub fn process_damage(mut events: EventReader<DamageEvent>, mut query_health: Query<&mut Health>) {
    for e in events.iter() {
        if let Ok(mut health) = query_health.get_mut(e.entity) {
            health.current -= e.damage;
        }
    }
}
