use bevy::{prelude::*, app::AppExit};

use iyes_bevy_util::{despawn_with_recursive};

use crate::{GameMode, AppState};
use super::UiAssets;

#[derive(Component)]
struct MainMenuCleanup;

mod btn {
    use bevy::prelude::*;

    #[derive(Component, Clone, Copy)]
    pub struct EnterGame(pub crate::GameMode);
    #[derive(Component, Clone, Copy)]
    pub struct ExitApp;
}

pub struct GameModeConfig {
}

pub struct MainMenuConfig {
    game_modes: Vec<GameModeConfig>,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::MainMenu)
                .with_system(init_mainmenu)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::MainMenu)
                .with_system(despawn_with_recursive::<MainMenuCleanup>)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(
                    super::button_connector::<btn::ExitApp>.system()
                        .chain(btn_impl_exitapp)
                )
                .with_system(
                    super::button_connector::<btn::EnterGame>.system()
                        .chain(btn_impl_entergame)
                )
        );
    }
}

fn init_mainmenu(
    mut cmd: Commands,
    assets: Res<UiAssets>,
) {
    let btn_style = Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: Rect::all(Val::Px(8.0)),
        margin: Rect::all(Val::Px(4.0)),
        flex_grow: 1.0,
        ..Default::default()
    };
    let btn_style_text = TextStyle {
        font: assets.font_menu_regular.clone(),
        font_size: 24.0,
        color: Color::BLACK,
    };
    let heading_text_style = TextStyle {
        font: assets.font_menu_bold.clone(),
        font_size: 24.0,
        color: Color::BLACK,
    };

    cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::rgb(0.5, 0.5, 0.5)),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            margin: Rect::all(Val::Auto),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::ColumnReverse,
            //align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).with_children(|p| {
        /*
        p.spawn_bundle(NodeBundle {
            material: assets.black.clone(),
            style: Style {
                flex_grow: 0.0,
                flex_shrink: 1.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        }).with_children(|p| {
            p.spawn_bundle(ImageBundle {
                style: Style {
                    flex_grow: 0.0,
                    flex_shrink: 1.0,
                    ..Default::default()
                },
                material: logo.0.clone(),
                ..Default::default()
            });
        });
        */
        p.spawn_bundle(NodeBundle {
            color: UiColor(Color::rgb(0.6, 0.6, 0.6)),
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        }).with_children(|r| {
            r.spawn_bundle(ButtonBundle {
                style: btn_style.clone(),
                ..Default::default()
            }).with_children(|btn| {
                btn.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Dev Playground",
                        btn_style_text.clone(),
                        Default::default(),
                    ),
                    ..Default::default()
                });
            }).insert(btn::EnterGame(GameMode::DevPlayground));
            r.spawn_bundle(ButtonBundle {
                style: btn_style.clone(),
                ..Default::default()
            }).with_children(|btn| {
                btn.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Play Game",
                        btn_style_text.clone(),
                        Default::default(),
                    ),
                    ..Default::default()
                });
            }).insert(btn::EnterGame(GameMode::Scenario1));
        });
        p.spawn_bundle(ButtonBundle {
            style: btn_style.clone(),
            ..Default::default()
        }).with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Exit Game",
                    btn_style_text.clone(),
                    Default::default(),
                ),
                ..Default::default()
            });
        }).insert(btn::ExitApp);
    }).insert(MainMenuCleanup);
}

fn btn_impl_exitapp(
    In(clicked): In<Option<btn::ExitApp>>,
    mut exit: EventWriter<AppExit>,
) {
    if clicked.is_some() {
        exit.send(AppExit);
    }
}

fn btn_impl_entergame(
    In(clicked): In<Option<btn::EnterGame>>,
    mut state: ResMut<State<AppState>>,
) {
    if let Some(gm) = clicked {
        state.set(AppState::GameAssetLoading(gm.0)).unwrap();
    }
}

