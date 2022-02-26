mod crosshair;
mod main_camera;

use bevy::prelude::*;
use iyes_bevy_util::BevyState;

use crate::game::crosshair::*;
use crate::game::main_camera::*;

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
        )
            .add_system_set(
                SystemSet::on_update(self.state.clone())
                    .with_system(crosshair_positon_update_system)
                    .with_system(mouse_pos_to_wspace_system)
            )
            .add_system_set(
                SystemSet::on_exit(self.state.clone())
                    .with_system(tear_down_crosshair)
            );
    }
}
