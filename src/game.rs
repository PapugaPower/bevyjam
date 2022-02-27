mod crosshair;
mod main_camera;
mod player;
mod shooting;
mod timer;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use iyes_bevy_util::*;

use crate::game::crosshair::*;
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::shooting::*;
use crate::game::timer::*;
use crate::game::hurt_zones::*;
use crate::game::player_triggers::*;

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
        // add event types
        app.add_event::<DamageEvent>();
        // add systems to `self.state`
        app.add_system_set(
            SystemSet::on_enter(self.state.clone())
                .with_system(init_main_camera)
                .with_system(setup_crosshair)
                .with_system(init_player)
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
                .with_system(explosions_controller.label("explosions"))
                .with_system(pulsation_controller.label("pulses"))
                .with_system(armaments_despawn)
                .with_system(debug_damage_event_reader
                    .after("projectiles")
                    .after("explosions")
                    .after("pulses"))
                // gameplay
                .with_system(tick_game_timer)
                .with_system(check_game_over)
                .with_system(check_player_dead)
                .with_system(evaluate_player_detection_triggers_system)
                .with_system(evaluate_hurt_zones)
        );
        app.add_system_set(
            SystemSet::on_exit(self.state.clone())
                .with_system(despawn_with::<Crosshair>)
                .with_system(despawn_with::<Player>)
                .with_system(despawn_with::<Projectile>)
                .with_system(remove_resource::<GameTimer>)
        );
    }
}

#[derive(AssetCollection)]
pub struct GameAssets {
}
