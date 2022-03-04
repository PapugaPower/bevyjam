use super::InterationEvent;
use crate::game::blueprints::AmmoBox;
use crate::game::player::Player;
use bevy::prelude::*;
use crate::game::environment::ReadyToDespawn;
use crate::SpareAmmo;

pub fn ammo_box_interaction(
    mut interaction_events: EventReader<InterationEvent>,
    query_ammo: Query<&AmmoBox>,
    mut query_player_ammo: Query<&mut SpareAmmo, With<Player>>, 
    mut commands: Commands
) {
    let mut player_ammo = query_player_ammo.single_mut();
    for interaction in interaction_events.iter() {
        if let Ok(ammo_box) = query_ammo.get(interaction.entity) {
            println!("Picked up {} rounds.", ammo_box.amount);
            player_ammo.current = (player_ammo.current + ammo_box.amount);
            commands.entity(interaction.entity).insert(ReadyToDespawn);
        }
    }
}
