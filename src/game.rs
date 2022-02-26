mod crosshair;
mod main_camera;
mod player;
mod shooting;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use iyes_bevy_util::BevyState;
use iyes_bevy_util::despawn_with;

use crate::game::crosshair::*;
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::shooting::*; 

pub mod sc1;
pub use sc1::Scenario1Plugin;

pub mod dev;
pub use dev::DevPlaygroundPlugin;

/// This plugin should add all common game systems used in all levels
pub struct GamePlugin<S: BevyState> {
    pub state: S,
}

impl<S: BevyState> Plugin for GamePlugin<S> {
    fn build(&self, app: &mut App) {
        // add systems to `self.state`
        app.add_system_set(
            SystemSet::on_enter(self.state.clone())
                .with_system(init_main_camera)
                .with_system(setup_crosshair)
                .with_system(init_player)
        );
        app.add_system_set(
            SystemSet::on_update(self.state.clone())
                .with_system(crosshair_positon_update_system)
                .with_system(mouse_pos_to_wspace_system)
                .with_system(recalculate_camera_desination_system)
                .with_system(refresh_camera_position_system)
                .with_system(transfer_input_to_player_system)
                .with_system(player_shoot)
                .with_system(bullets_despawn)
                .with_system(bullets_collision)
        );
        app.add_system_set(
            SystemSet::on_exit(self.state.clone())
                .with_system(despawn_with::<Crosshair>)
                .with_system(despawn_with::<Player>)
                .with_system(despawn_with::<Bullet>)
        );
    }
}

#[derive(AssetCollection)]
pub struct GameAssets {
}
