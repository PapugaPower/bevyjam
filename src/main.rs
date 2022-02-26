use bevy::prelude::*;

/// Each level/map in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameMode {
    Scenario1,
    DevPlayground,
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
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

    // 3rdparty plugins
    app.add_plugin(bevy_tweening::TweeningPlugin);
    app.add_plugin(benimator::AnimationPlugin::default());
    app.add_plugin(bevy_kira_audio::AudioPlugin);

    // our state enum and game stuff
    app.add_state(AppState::MainMenu);

    app.run();
}
