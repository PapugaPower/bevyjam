use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};
use iyes_bevy_util::BevyState;

use crate::game::timer::GameTimer;

/// This plugin should add all Scenario1 specific stuff
pub struct Scenario1Plugin<S: BevyState + Copy> {
    pub loading_state: S,
    pub state: S,
}

impl<S: BevyState + Copy> Plugin for Scenario1Plugin<S> {
    fn build(&self, app: &mut App) {
        // asset loader
        AssetLoader::new(self.loading_state)
            .continue_to_state(self.state)
            .with_asset_collection_file("meta/sc1.assets")
            .with_collection::<Sc1Assets>()
            .build(app);

        // add systems to `self.state`
        app.add_system_set(
            SystemSet::on_enter(self.state)
                .with_system(init_game_timer)
        );
        app.add_system_set(
            SystemSet::on_update(self.state)
        );
        app.add_system_set(
            SystemSet::on_exit(self.state)
        );
    }
}

#[derive(AssetCollection)]
struct Sc1Assets {
}

fn init_game_timer(
    mut commands: Commands,
) {
    let timer = Timer::from_seconds(3.0 * 60.0, false);
    commands.insert_resource(GameTimer(timer));
}
