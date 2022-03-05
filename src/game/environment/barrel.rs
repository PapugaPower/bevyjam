use crate::game::damage::{Health, Pulsing};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub enum ExplosiveObjectState {
    NotDetonated,
    Exploding(Timer),
    Detonated,
}

#[derive(Component)]
pub struct ExplosiveObject {
    pub state: ExplosiveObjectState,
}

pub fn explosive_objects_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Health, &mut Pulsing, &mut ExplosiveObjectState)>,
) {
    for (entity, health, mut pulsing, mut state) in query.iter_mut() {
        match state.as_mut() {
            ExplosiveObjectState::NotDetonated => {
                if health.current <= 0.0 {
                    *state = ExplosiveObjectState::Exploding(Timer::from_seconds(1.5, false));
                    pulsing.pulse_time.unpause();
                }
            }
            ExplosiveObjectState::Exploding(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    *state = ExplosiveObjectState::Detonated;
                }
            }
            ExplosiveObjectState::Detonated => {
                commands.entity(entity).despawn();
            }
        }
    }
}

