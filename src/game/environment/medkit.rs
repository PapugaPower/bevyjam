use super::InterationEvent;
use crate::game::blueprints::Medkit;
use crate::game::damage::Health;
use crate::game::player::Player;
use bevy::prelude::*;
use crate::game::environment::ReadyToDespawn;

pub fn medkit_interaction(
    mut interaction_events: EventReader<InterationEvent>,
    query_medkits: Query<&Medkit>,
    mut query_player_health: Query<&mut Health, With<Player>>,
    mut commands: Commands
) {
    let mut player_hp = query_player_health.single_mut();
    for interaction in interaction_events.iter() {
        if let Ok(medkit) = query_medkits.get(interaction.entity) {
            println!("healing for {}", medkit.healing);
            player_hp.current = (player_hp.current + medkit.healing).clamp(0.0, player_hp.max);
            commands.entity(interaction.entity).insert(ReadyToDespawn);
        }
    }
}
