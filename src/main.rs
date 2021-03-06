// disable windows console in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};
use game::shooting::*;

use enum_iterator::IntoEnumIterator;
use iyes_bevy_util::remove_resource;

const GAMENAME: &str = "Absconder";

mod editor;
mod game;
mod ui;
mod util;
mod scene_exporter;

/// Each level/map in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
#[derive(IntoEnumIterator)]
pub enum GameMode {
    Scenario1,
    DevPlayground,
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum AppState {
    MainAssetLoading,
    MainMenu,
    GameAssetLoading(GameMode),
    InGame(GameMode),
    GameOver,
    Credits,
    DevEditor,
}

/// Cant we have stageless already! :yeet:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(StageLabel)]
enum FuckStages {
    Pre,
    Post,
}

fn main() {
    let mut app = App::new();

    // configure bevy
    app.insert_resource(bevy::log::LogSettings {
        level: bevy::log::Level::DEBUG,
        ..Default::default()
    });
    app.insert_resource(WindowDescriptor {
        title: GAMENAME.into(),
        vsync: true,
        resizable: false,
        width: 1280.,
        height: 960.,
        ..Default::default()
    });
    app.insert_resource(ClearColor(Color::BLACK));
    app.add_plugins(DefaultPlugins);

    // extra stages, cuz bevy is fucked like that
    app.add_stage_after(CoreStage::Update, FuckStages::Post, SystemStage::parallel());
    app.add_stage_before(CoreStage::Update, FuckStages::Pre, SystemStage::parallel());

    // our state enum
    app.add_state(AppState::MainAssetLoading);

    // 3rdparty plugins
    app.add_plugin(bevy_tweening::TweeningPlugin);
    app.add_plugin(benimator::AnimationPlugin::default());
    app.add_plugin(bevy_kira_audio::AudioPlugin);
    app.add_plugin(heron::PhysicsPlugin::default());
    app.add_plugin(bevy_prototype_debug_lines::DebugLinesPlugin::default());

    // assets loader
    AssetLoader::new(AppState::MainAssetLoading)
        .continue_to_state(AppState::MainMenu)
        .with_asset_collection_file("meta/ui.assets")
        .with_asset_collection_file("meta/game.assets")
        .with_collection::<ui::UiAssets>()
        .with_collection::<game::GameAssets>()
        .build(&mut app);

    // our game stuff
    app.add_plugin(ui::UiSetupPlugin);

    app.add_system_set(
        SystemSet::on_exit(AppState::GameOver)
            .with_system(remove_resource::<game::GameResult>)
    );
    app.add_system_set(
        SystemSet::on_enter(AppState::MainMenu)
            .with_system(remove_resource::<GameMode>)
    );

    for mode in GameMode::into_enum_iter() {
        app.add_plugin(game::GamePlugin { state: AppState::InGame(mode) });
        app.add_plugin(ui::hud::HudPlugin {state: AppState::InGame(mode)});
    }

    app.add_plugin(game::DevPlaygroundPlugin {
        loading_state: AppState::GameAssetLoading(GameMode::DevPlayground),
        state: AppState::InGame(GameMode::DevPlayground),
    });
    app.add_plugin(game::Scenario1Plugin {
        loading_state: AppState::GameAssetLoading(GameMode::Scenario1),
        state: AppState::InGame(GameMode::Scenario1),
    });

    app.add_plugin(util::UtilPlugin);
    app.add_plugin(editor::DevEditorPlugin);
    app.add_plugin(game::blueprints::BlueprintsPlugin);
    app.add_plugin(scene_exporter::SerializePlugin);

    // debug systems; uncomment if needed
    // app.add_system(debug_state);

    app.run();
}

#[allow(dead_code)]
fn debug_state(
    state: Res<State<AppState>>,
) {
    debug!("Current AppState: {:?}", state.current());
}
