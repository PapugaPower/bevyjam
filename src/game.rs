mod audio2d;
mod crosshair;
mod damage;
mod doors;
mod enemies;
mod hints;
mod main_camera;
mod player;
mod shooting;
mod timer;
mod environment;
mod phys_layers;
mod player_triggers;
mod world_interaction;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::{AudioChannel, AudioSource};
use iyes_bevy_util::*;

use crate::game::audio2d::*;
use crate::game::crosshair::*;
use crate::game::damage::*;
use crate::game::doors::*;
use crate::game::enemies::*;
use crate::game::environment::*;
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::player_triggers::*;
use crate::game::shooting::*;
use crate::game::timer::*;
use crate::game::world_interaction::*;
use crate::util::MainCamera;
use hints::*;

pub mod sc1;
pub use sc1::Scenario1Plugin;

pub mod dev;

pub use dev::DevPlaygroundPlugin;

pub mod blueprints;

/// This plugin should add all common game systems used in all levels
pub struct GamePlugin<S: BevyState> {
    pub state: S,
}

impl<S: BevyState> Plugin for GamePlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioChannelPool::default());
        app.insert_resource(GameAudioChannel(AudioChannel::new("game".into())));
        // add event types
        app.add_event::<DamageEvent>();
        app.add_event::<DoorUseEvent>();
        app.add_event::<PlayerFiredEvent>();
        app.add_event::<BulletImpactEvent>();
        // add systems to `self.state`
        app.add_system_set(
            SystemSet::on_enter(self.state.clone())
                .with_system(init_main_camera)
                .with_system(setup_crosshair)
                .with_system(init_player)
                .with_system(init_hints)
                .with_system(set_cursor_visibility::<false>),
        );
        let _x = app.add_system_set(
            SystemSet::on_update(self.state.clone())
                // player movement
                .with_system(crosshair_position_update_system)
                .with_system(recalculate_camera_desination_system)
                .with_system(refresh_camera_position_system)
                .with_system(transfer_input_to_player_system)
                // enemies
                .with_system(enemy_controller.label("enemy_controller"))
                .with_system(enemy_spawn)
                .with_system(enemy_despawn)
                // shooting
                .with_system(player_shoot)
                .with_system(projectiles_controller.label("projectiles"))
                .with_system(armaments_despawn)
                .with_system(handle_shot_audio)
                .with_system(handle_impact_audio)
                // damage 
                .with_system(pulsation_controller.label("pulses"))
                .with_system(explosive_objects_controller)
                .with_system(
                    process_damage
                        .after("projectiles")
                        .after("pulses")
                        .after("enemy_controller"),
                )
                // general gameplay
                .with_system(tick_game_timer)
                .with_system(check_game_over)
                .with_system(check_player_dead)
                .with_system(evaluate_player_detection_triggers_system)
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
                .with_system(spatial_audio_removed),
        );
        app.add_system_set(
            SystemSet::on_exit(self.state.clone())
                .with_system(despawn_with::<Crosshair>)
                .with_system(despawn_with::<Player>)
                .with_system(despawn_with::<Projectile>)
                .with_system(despawn_with::<MainCamera>)
                .with_system(remove_resource::<GameTimer>)
                .with_system(set_cursor_visibility::<true>),
        );
        app.add_system_set(
            SystemSet::on_pause(self.state.clone()).with_system(set_cursor_visibility::<true>),
        );
        app.add_system_set(
            SystemSet::on_resume(self.state.clone()).with_system(set_cursor_visibility::<false>),
        );
    }
}

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(key = "item.medkit")]
    pub medkit: Handle<Image>,
    #[asset(key = "audio.smg_shot")]
    pub smg_shot_audio: Handle<AudioSource>,
    #[asset(path = "audio/world_impacts", folder)]
    pub world_impacts: Vec<HandleUntyped>,
    #[asset(path = "audio/monster_impacts", folder)]
    pub monster_impacts: Vec<HandleUntyped>,
}
pub struct GameAudioChannel(AudioChannel);

/// Insert as resource on game over, to indicate status
pub enum GameResult {
    /// Scenario Completed
    Win,
    /// Player died
    LoseHealth,
    /// Player ran out of time
    LoseTime,
}

fn set_cursor_visibility<const VIS: bool>(mut wnds: ResMut<Windows>) {
    let wnd = wnds.get_primary_mut().unwrap();
    wnd.set_cursor_visibility(VIS);
}
