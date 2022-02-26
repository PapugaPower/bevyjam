use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};

/// Each level/map in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameMode {
    Scenario1,
    DevPlayground,
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
    MainAssetLoading,
    MainMenu,
    Loading(GameMode),
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

    // assets loader
    AssetLoader::new(AppState::MainAssetLoading)
        .continue_to_state(AppState::MainMenu)
        .with_asset_collection_file("meta/ui.assets")
        .with_collection::<UiAssets>()
        .build(&mut app);

    // our game stuff
    //app.add_plugin(ui::mainmenu::MainMenuPlugin(AppState::MainMenu));

    app.run();
}

#[derive(AssetCollection)]
struct UiAssets {
    #[asset(key = "ui.font_menu_bold")]
    font_menu_bold: Handle<Font>,
    #[asset(key = "ui.font_menu_regular")]
    font_menu_regular: Handle<Font>,
}
