use crate::game::damage::{Health, Pulsing};
use bevy::prelude::*;

#[derive(Debug, Component)]
pub enum ExplosiveObjectState {
    NotDetonated,
    Exploding,
    Detonated,
}

#[derive(Component)]
pub struct ExplosiveObject {
    pub state: ExplosiveObjectState,
}

pub fn explosive_objects_controller(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, &mut Pulsing, &mut ExplosiveObjectState)>,
) {
    for (entity, health, mut pulsing, mut state) in query.iter_mut() {
        match *state {
            ExplosiveObjectState::NotDetonated => {
                if health.current <= 0.0 {
                    *state = ExplosiveObjectState::Exploding;
                    pulsing.pulse_time.unpause();
                }
            }
            ExplosiveObjectState::Exploding => {
                if pulsing.pulse_time.finished() {
                    *state = ExplosiveObjectState::Detonated;
                }
            }
            ExplosiveObjectState::Detonated => {
                commands.entity(entity).despawn();
            }
        }
    }
}
