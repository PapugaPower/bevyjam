use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};

use enum_iterator::IntoEnumIterator;

mod game;
mod ui;

/// Each level/map in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
#[derive(IntoEnumIterator)]
pub enum GameMode {
    Scenario1,
    DevPlayground,
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
enum AppState {
    MainAssetLoading,
    MainMenu,
    InGame(GameMode),
    GameOverWin,
    GameOverLose,
    Credits,
}

fn main() {
    let mut app = App::new();

    // configure bevy
    app.insert_resource(bevy::log::LogSettings {
        level: bevy::log::Level::DEBUG,
        ..Default::default()
    });
    app.insert_resource(WindowDescriptor {
        title: "Jam Game".into(),
        vsync: true,
        resizable: true,
        width: 1280.,
        height: 720.,
        ..Default::default()
    });
    app.insert_resource(ClearColor(Color::BLACK));
    app.add_plugins(DefaultPlugins);

    // our state enum
    app.add_state(AppState::MainAssetLoading);

    // 3rdparty plugins
    app.add_plugin(bevy_tweening::TweeningPlugin);
    app.add_plugin(benimator::AnimationPlugin::default());
    app.add_plugin(bevy_kira_audio::AudioPlugin);
    app.add_plugin(heron::PhysicsPlugin::default());

    // assets loader
    AssetLoader::new(AppState::MainAssetLoading)
        .continue_to_state(AppState::MainMenu)
        .with_asset_collection_file("meta/ui.assets")
        .with_asset_collection_file("meta/game.assets")
        .with_collection::<UiAssets>()
        .with_collection::<GameAssets>()
        .build(&mut app);

    // our game stuff
    app.add_plugin(ui::UiSetupPlugin);
    app.add_plugin(ui::mainmenu::MainMenuPlugin);

    for mode in GameMode::into_enum_iter() {
        app.add_plugin(game::GamePlugin { state: AppState::InGame(mode) });
    }

    app.add_plugin(game::DevPlaygroundPlugin { state: AppState::InGame(GameMode::DevPlayground) });
    app.add_plugin(game::Scenario1Plugin { state: AppState::InGame(GameMode::Scenario1) });

    // debug systems; uncomment if needed
    //app.add_system(debug_state);

    app.run();
}

#[allow(dead_code)]
fn debug_state(
    state: Res<State<AppState>>,
) {
    debug!("Current AppState: {:?}", state.current());
}

#[derive(AssetCollection)]
struct UiAssets {
    #[asset(key = "ui.font_menu_bold")]
    font_menu_bold: Handle<Font>,
    #[asset(key = "ui.font_menu_regular")]
    font_menu_regular: Handle<Font>,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(key = "enviro.map_prototype")]
    map_prototype: Handle<Image>,
}
