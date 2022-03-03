use crate::game::phys_layers::PhysLayer;
use bevy::prelude::*;
use heron::{rapier_plugin::PhysicsWorld, CollisionLayers, CollisionShape};

#[derive(Debug, Component)]
pub struct Health {
    pub max: f32,
    pub current: f32,
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
        debug!("damage event: {:?}", e);
        if let Ok(mut health) = query_health.get_mut(e.entity) {
            health.current -= e.damage;
        }
    }
}

#[derive(Debug, Component)]
pub struct Pulsing {
    pub pulse_time: Timer,
    pub damage: f32,
}

#[derive(Component)]
pub enum DamageAreaShape {
    Cuboid { half_extends: Vec3 },
    Sphere { radius: f32 },
}

impl From<&DamageAreaShape> for CollisionShape {
    fn from(shape: &DamageAreaShape) -> Self {
        match shape {
            DamageAreaShape::Cuboid { half_extends } => CollisionShape::Cuboid {
                half_extends: *half_extends,
                border_radius: None,
            },
            DamageAreaShape::Sphere { radius } => CollisionShape::Sphere { radius: *radius },
        }
    }
}

#[derive(Bundle)]
pub struct PulsingBundle {
    pub pulsing: Pulsing,
    pub damage_area_shape: DamageAreaShape,
    // cleanup marker
    pub cleanup: super::GameCleanup,
}

impl Default for PulsingBundle {
    fn default() -> Self {
        Self {
            pulsing: Pulsing {
                pulse_time: Timer::from_seconds(1.0, true),
                damage: 10.0,
            },
            damage_area_shape: DamageAreaShape::Sphere { radius: 10.0 },
            cleanup: super::GameCleanup,
        }
    }
}

pub fn pulsation_controller(
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    physics_world: PhysicsWorld,
    mut query_pulsing: Query<(&Transform, &DamageAreaShape, &mut Pulsing)>,
) {
    for (transform, shape, mut pulsating) in query_pulsing.iter_mut() {
        // collision check
        pulsating.pulse_time.tick(time.delta());
        if pulsating.pulse_time.finished() {
            if !pulsating.pulse_time.repeating() && !pulsating.pulse_time.just_finished() {
                continue;
            }
            physics_world.intersections_with_shape(
                &shape.into(),
                transform.translation,
                transform.rotation,
                CollisionLayers::all::<PhysLayer>(),
                &mut |e| {
                    damage_event.send(DamageEvent {
                        entity: e,
                        source: DamageSource::Weapon,
                        damage: pulsating.damage,
                    });
                    true
                },
            );
        }
    }
}
