use std::time::Duration;
use bevy::core::FixedTimestep;
use bevy::ecs::schedule::ShouldRun;

use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};
use heron::CollisionShape;
use iyes_bevy_util::BevyState;
use crate::game::collider::ColliderKind::Wall;
use crate::game::GameAssets;
use crate::game::pathfinding::generate_grid;

use crate::game::timer::GameTimer;

use super::GameCleanup;

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
                .with_system(spawn_dynamic_scene)
                .with_system(init_game_timer)
                .with_system(load_game_map)
        );
        app.add_system_set(
            SystemSet::on_enter(self.state).with_run_criteria(FixedTimestep::step(10.0))
        );
        app.add_system_set(
            SystemSet::on_update(self.state)
                .with_system(generate_grid)

        );
        app.add_system_set(
            SystemSet::on_exit(self.state)
        );
    }
}

#[derive(AssetCollection)]
struct Sc1Assets {
    #[asset(key = "scene.sc1")]
    pub scene: Handle<DynamicScene>,
    #[asset(key = "enviro.map_level_0")]
    pub map_level_0: Handle<Image>,
}

fn init_game_timer(
    mut commands: Commands,
) {
    let timer = Timer::from_seconds(2.0 * 60.0, false);
    commands.insert_resource(GameTimer(timer));
}

fn spawn_dynamic_scene(
    mut scene_spawner: ResMut<SceneSpawner>,
    assets: Res<Sc1Assets>,
) {
    scene_spawner.spawn_dynamic(assets.scene.clone());
}
fn load_game_map(
    mut commands: Commands,
    assets: Res<Sc1Assets>,
){
    let mut level_tform = Transform::from_xyz(0.0, 0.0, -1.5);
    level_tform.scale = Vec3::new(1.25, 1.25,1.0);
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(8192.0, 8192.0)),
            ..Default::default()
        },
        transform: level_tform,
        global_transform: Default::default(),
        texture: assets.map_level_0.clone(),
        visibility: Default::default(),
    }).insert(GameCleanup);
}