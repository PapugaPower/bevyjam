use bevy::prelude::*;
use iyes_bevy_util::BevyState;

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
    }
}
