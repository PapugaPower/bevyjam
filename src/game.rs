mod audio2d;
mod crosshair;
mod hints;
mod main_camera;
mod player;
mod shooting;
mod timer;
mod doors;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use iyes_bevy_util::*;

use crate::game::audio2d::*; 
use crate::game::crosshair::*;
use hints::*;
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::shooting::*;
use crate::game::timer::*;
use crate::game::hurt_zones::*;
use crate::game::player_triggers::*;
use crate::game::world_interaction::*;
use crate::game::doors::*;

pub mod sc1;
pub use sc1::Scenario1Plugin;

pub mod dev;
mod phys_layers;
mod hurt_zones;
mod player_triggers;
mod world_interaction;

pub use dev::DevPlaygroundPlugin;

/// This plugin should add all common game systems used in all levels
pub struct GamePlugin<S: BevyState> {
    pub state: S,
}

impl<S: BevyState> Plugin for GamePlugin<S> {
    fn build(&self, app: &mut App) {
		app.insert_resource(AudioChannelPool::default());
        // add event types
        app.add_event::<DamageEvent>();
        app.add_event::<DoorUseEvent>();
        // add systems to `self.state`
        app.add_system_set(
            SystemSet::on_enter(self.state.clone())
                .with_system(init_main_camera)
                .with_system(setup_crosshair)
                .with_system(init_player)
				.with_system(init_hints)
        );
        let _x = app.add_system_set(
            SystemSet::on_update(self.state.clone())
                // player movement
                .with_system(crosshair_positon_update_system)
                .with_system(mouse_pos_to_wspace_system)
                .with_system(recalculate_camera_desination_system)
                .with_system(refresh_camera_position_system)
                .with_system(transfer_input_to_player_system)
                // shooting
                .with_system(player_shoot)
                .with_system(projectiles_controller.label("projectiles"))
                .with_system(pulsation_controller.label("pulses"))
                .with_system(armaments_despawn)
                // general gameplay
                .with_system(tick_game_timer)
                .with_system(check_game_over)
                .with_system(check_player_dead)
                .with_system(evaluate_player_detection_triggers_system)
                .with_system(evaluate_hurt_zones)
                .with_system(door_interaction.label("door_interaction"))
                .with_system(door_event_processor.after("door_interaction"))
                // interaction processing
                .with_system(process_new_interactions)
                .with_system(process_interaction_timeouts)
                .with_system(process_interactable_despawn)
                .with_system(process_world_medkit_use)
				// spatial sound
                .with_system(spatial_audio.after("spatial_audio_added"))
                .with_system(spatial_audio_changed.after("spatial_audio_added"))
                .with_system(spatial_audio_added.label("spatial_audio_added"))
                .with_system(spatial_audio_removed)
        );
        app.add_system_set(
            SystemSet::on_exit(self.state.clone())
                .with_system(despawn_with::<Crosshair>)
                .with_system(despawn_with::<Player>)
                .with_system(despawn_with::<Projectile>)
                .with_system(despawn_with::<MainCamera>)
                .with_system(remove_resource::<GameTimer>)
        );
    }
}

#[derive(AssetCollection)]
pub struct GameAssets {
}

/// Insert as resource on game over, to indicate status
pub enum GameResult {
    /// Scenario Completed
    Win,
    /// Player died
    LoseHealth,
    /// Player ran out of time
    LoseTime,
}
