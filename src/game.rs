mod audio2d;
mod crosshair;
mod damage;
mod enemies;
mod hints;
mod main_camera;
mod player;
mod shooting;
mod timer;
mod environment;
mod phys_layers;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::{AudioChannel, AudioSource};
use heron::RigidBody;
use iyes_bevy_util::*;

use crate::game::audio2d::*;
use crate::game::crosshair::*;
use crate::game::damage::*;
use crate::game::enemies::*;
use crate::game::environment::{*, door::*, medkit::*};
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::shooting::*;
use crate::game::timer::*;
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

/// Add this component to every in-game entity
/// that is initialized when starting a new game
/// and should be despawned when the game is over or restarted
#[derive(Component)]
struct GameCleanup;

impl<S: BevyState> Plugin for GamePlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioChannelPool::default());
        app.insert_resource(GameAudioChannel(AudioChannel::new("game".into())));
        // add event types
        app.add_event::<DamageEvent>();
        app.add_event::<InterationEvent>();
        app.add_event::<PlayerFiredEvent>();
        app.add_event::<BulletImpactEvent>();
        app.add_system_to_stage(CoreStage::PostUpdate, add_missing_cleanup);
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
                .with_system(player_shoot.label("player_shoot"))
                .with_system(projectiles_controller.label("projectiles"))
                .with_system(armaments_despawn)
                .with_system(handle_shot_audio.after("player_shoot"))
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
                // interaction processing
                .with_system(trigger_player_detection)
                .with_system(trigger_interaction.label("trigger_interaction"))
                .with_system(triggir_timeout_process)
                // general gameplay
                .with_system(tick_game_timer)
                .with_system(check_game_over)
                .with_system(check_player_dead)
                .with_system(door_interaction.after("trigger_interaction"))
                .with_system(medkit_interaction.after("trigger_interaction"))
                .with_system(explosive_objects_controller)
                // spatial sound
                .with_system(spatial_audio.after("spatial_audio_added"))
                .with_system(spatial_audio_changed.after("spatial_audio_added"))
                .with_system(spatial_audio_added.label("spatial_audio_added"))
                .with_system(spatial_audio_removed),
        );
        app.add_system_set(
            SystemSet::on_exit(self.state.clone())
                .with_system(despawn_with_recursive::<GameCleanup>)
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

/// Ensure our various game entities have a cleanup marker
fn add_missing_cleanup(
    mut commands: Commands,
    query: Query<Entity, (
        Without<GameCleanup>,
        Or<(
            With<RigidBody>,
            With<Player>,
            With<Projectile>,
            With<Enemy>,
            With<Trigger>,
        )>,
    )>,
) {
    for e in query.iter() {
        commands.entity(e)
            .insert(GameCleanup);
    }
}
