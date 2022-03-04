use crate::game::collider::WinZone;
use crate::game::player::Player;
use crate::game::GameResult;
use crate::AppState;
use bevy::prelude::*;
use heron::prelude::*;

pub mod barrel;
pub mod door;
pub mod medkit;

pub struct InterationEvent {
    entity: Entity,
}

#[derive(Component, Default)]
pub struct Trigger {
    pub player_detected: bool,
    pub entities: Vec<Entity>,
}

#[derive(Component)]
pub struct TriggerTimeout {
    pub timeout: Timer,
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct MultiUse {
    pub remaining: i32,
}

pub fn trigger_player_detection(
    mut collision_events: EventReader<CollisionEvent>,
    query_player: Query<Entity, With<Player>>,
    mut query_triggers: Query<&mut Trigger>,
) {
    let player = query_player.single();
    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(e1, e2) => {
                let trigger = if e1.rigid_body_entity() == player {
                    e2
                } else {
                    e1
                };
                if let Ok(mut trigger) = query_triggers.get_mut(trigger.rigid_body_entity()) {
                    trigger.player_detected = true;
                }
                // println!("Collision started between {:?} and {:?}", e1, e2)
            }
            CollisionEvent::Stopped(e1, e2) => {
                let trigger = if e1.rigid_body_entity() == player {
                    e2
                } else {
                    e1
                };
                if let Ok(mut trigger) = query_triggers.get_mut(trigger.rigid_body_entity()) {
                    trigger.player_detected = false;
                }
                // println!("Collision stopped between {:?} and {:?}", e1, e2)
            }
        }
    }
}

pub fn trigger_interaction(
    input: Res<Input<KeyCode>>,
    mut interation_events: EventWriter<InterationEvent>,
    query_triggers: Query<(&Trigger, Option<&TriggerTimeout>)>,
) {
    if input.just_pressed(KeyCode::E) {
        for (trigger, timeout) in query_triggers.iter() {
            if let Some(timeout) = timeout {
                if !timeout.timeout.finished() {
                    continue;
                }
            }
            if trigger.player_detected {
                for entity in trigger.entities.iter() {
                    interation_events.send(InterationEvent { entity: *entity });
                }
            }
        }
    };
}

pub fn triggir_timeout_process(time: Res<Time>, mut query_triggers: Query<&mut TriggerTimeout>) {
    for mut timeout in query_triggers.iter_mut() {
        timeout.timeout.tick(time.delta());
    }
}

pub fn check_game_win(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    query_triggers: Query<&Trigger, With<WinZone>>,
) {
    for trigger in query_triggers.iter() {
        if trigger.player_detected {
            state.push(AppState::GameOver).unwrap();
            commands.insert_resource(GameResult::Win);
        }
    }
}
