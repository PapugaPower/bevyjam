mod animations;
mod audio2d;
mod crosshair;
pub(crate) mod damage;
mod enemies;
mod environment;
mod hints;
mod main_camera;
mod phys_layers;
pub(crate) mod player;
pub(crate) mod shooting;
mod timer;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::{AudioChannel, AudioSource};
use heron::RigidBody;
use iyes_bevy_util::*;

use crate::game::animations::*;
use crate::game::audio2d::*;
use crate::game::crosshair::*;
use crate::game::damage::*;
use crate::game::enemies::*;
use crate::game::environment::{ammo_box::*, barrel::*, door::*, medkit::*, *};
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::shooting::*;
pub use crate::game::timer::*;
use crate::util::MainCamera;
use crate::AppState;
use crate::FuckStages;
use hints::*;

pub mod sc1;
pub use sc1::Scenario1Plugin;

pub mod dev;

pub use dev::DevPlaygroundPlugin;

pub mod blueprints;
pub mod collider;
mod pathfinding;

/// This plugin should add all common game systems used in all levels
pub struct GamePlugin<S: BevyState> {
    pub state: S,
}

/// Add this component to every in-game entity
/// that is initialized when starting a new game
/// and should be despawned when the game is over or restarted
#[derive(Component)]
pub struct GameCleanup;

impl<S: BevyState> Plugin for GamePlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioChannelPool::default());
        app.insert_resource(GameAudioChannel(AudioChannel::new("game".into())));
        app.init_resource::<EnemyConfig>();
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
                .with_system(animations_init)
                .with_system(init_player)
                .with_system(init_hints)
                .with_system(set_cursor_visibility::<false>),
        );
        let _x = app.add_system_set(
            SystemSet::on_update(self.state.clone())
                .with_system(exit_game_on_esc)
                // .with_system(debug_enemy_spawn)
                // .with_system(enemy_debug_lines)
                // player movement
                .with_system(crosshair_position_update_system.label("crosshair_update"))
                .with_system(transfer_input_to_player_system.label("player_movement"))
                .with_system(
                    recalculate_camera_desination_system
                        .label("recalculate_camera")
                        .after("crosshair_update")
                        .after("player_movement"),
                )
                .with_system(refresh_camera_position_system.after("recalculate_camera"))
                .with_system(print_player_position)
                // enemies
                //.with_system(enemy_controller.label("enemy_controller"))
                //.with_system(spawn_zones)
                .with_system(enemy_despawn_stuck)
                .with_system(enemy_despawn_far)
                .with_system(enemy_die.after("damage"))
                //.with_system(enemy_target_entity)
                //.with_system(enemy_rotation)
                //.with_system(enemy_player_search.before("damage"))
                //.with_system(enemy_target_scan.before("damage"))
                .with_system(enemy_walk)
                .with_system(enemy_damage.label("enemy_damage"))
                // .with_system(enemy_flock)
                // .with_system(enemy_spawn)
                // .with_system(enemy_despawn)
                // shooting
                .with_system(player_shoot.label("player_shoot").after("player_movement"))
                .with_system(projectiles_controller.label("projectiles"))
                .with_system(armaments_despawn)
                .with_system(gun_reload.label("gun_reload").after("player_shoot"))
                .with_system(handle_shot_audio.after("player_shoot"))
                .with_system(handle_impact_audio)
                // damage
                .with_system(pulsation_controller.label("pulses"))
                .with_system(explosive_objects_controller)
                .with_system(
                    process_damage
                        .label("damage")
                        .after("projectiles")
                        .after("pulses")
                        .after("enemy_damage"),
                )
                // animation
                .with_system(animations_removal)
                .with_system(
                    animation_player
                        .after("player_movement")
                        .after("player_shoot")
                        .after("gun_reload"),
                )
                .with_system(animation_player_impact.after("projectiles"))
                .with_system(animation_explosive_objects)
                // interaction processing
                .with_system(trigger_player_detection)
                .with_system(trigger_interaction.label("trigger_interaction"))
                .with_system(triggir_timeout_process)
                // general gameplay
                .with_system(tick_game_timer)
                .with_system(check_game_over)
                .with_system(check_player_dead)
                .with_system(check_game_win)
                .with_system(door_interaction.after("trigger_interaction"))
                .with_system(medkit_interaction.after("trigger_interaction"))
                .with_system(ammo_box_interaction.after("trigger_interaction"))
                .with_system(process_interactable_despawn)
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
    #[asset(key = "player.idle")]
    pub player_idle: Handle<Image>,
    #[asset(key = "player.legs")]
    pub player_legs: Handle<Image>,
    #[asset(key = "player.move")]
    pub player_move: Handle<Image>,
    #[asset(key = "player.reload")]
    pub player_reload: Handle<Image>,
    #[asset(key = "player.shoot")]
    pub player_shoot: Handle<Image>,
    #[asset(key = "item.medkit")]
    pub medkit: Handle<Image>,
    #[asset(key = "item.ammo")]
    pub ammo: Handle<Image>,
    #[asset(key = "animation.explosion")]
    pub explosion: Handle<Image>,
    #[asset(key = "animation.blood_splash")]
    pub blood_splash: Handle<Image>,
    #[asset(key = "animation.hit_0")]
    pub hit_0: Handle<Image>,
    #[asset(key = "animation.hit_1")]
    pub hit_1: Handle<Image>,
    #[asset(key = "enemy.move")]
    pub enemy_move: Handle<Image>,
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

/// exit to main menu on pressing Esc
/// FIXME: temporary until we have a menu
fn exit_game_on_esc(kbd: Res<Input<KeyCode>>, mut state: ResMut<State<AppState>>) {
    if kbd.just_pressed(KeyCode::Escape) {
        state.replace(AppState::MainMenu).ok();
    }
}

fn set_cursor_visibility<const VIS: bool>(mut wnds: ResMut<Windows>) {
    let wnd = wnds.get_primary_mut().unwrap();
    wnd.set_cursor_visibility(VIS);
}

/// Ensure our various game entities have a cleanup marker
fn add_missing_cleanup(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Without<GameCleanup>,
            Or<(
                With<RigidBody>,
                With<Player>,
                With<Projectile>,
                With<Enemy>,
                With<Trigger>,
            )>,
        ),
    >,
) {
    for e in query.iter() {
        commands.entity(e).insert(GameCleanup);
    }
}
