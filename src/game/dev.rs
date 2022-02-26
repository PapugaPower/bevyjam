use bevy::prelude::*;
use iyes_bevy_util::BevyState;

/// This plugin should add all Scenario1 specific stuff
pub struct DevPlaygroundPlugin<S: BevyState> {
    pub state: S,
}

impl<S: BevyState> Plugin for DevPlaygroundPlugin<S> {
    fn build(&self, app: &mut App) {
        // add systems to `self.state`
    }
}

