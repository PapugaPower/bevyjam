use bevy::{prelude::*, app::AppExit};

use bevy_ninepatch::*;
use iyes_bevy_util::{despawn_with_recursive};

use crate::{GameMode, AppState, FuckStages};

use super::{UiAssets, UiNinepatches, ContentId, UiConfig, Btn, fill_btn, spawn_button};

mod btn {
    use bevy::prelude::*;

    #[derive(Component, Clone, Copy)]
    pub struct EnterGame(pub crate::GameMode);
    #[derive(Component, Clone, Copy)]
    pub struct ExitApp;
}

#[derive(Component)]
struct MainMenuCleanup;

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

        let update = SystemSet::on_update(AppState::MainMenu);
        let update = btn::ExitApp::register_handler( update);
        let update = btn::EnterGame::register_handler( update);
        app.add_system_set(update);

        app.add_system_to_stage(FuckStages::Post, fill_btn::<btn::EnterGame>);
        app.add_system_to_stage(FuckStages::Post, fill_btn::<btn::ExitApp>);
    }
}

fn init_mainmenu(
    mut cmd: Commands,
    assets: Res<UiAssets>,
    uicfg: Res<UiConfig>,
    nps: Res<UiNinepatches>,
) {
    let menu = cmd.spawn_bundle(NodeBundle {
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
    })//.with_children(|p| {
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
        /*
        p.spawn_bundle(NodeBundle {
            color: UiColor(Color::rgb(0.6, 0.6, 0.6)),
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })*/
    //})
    .insert(MainMenuCleanup).id();

    spawn_button(
        &mut cmd,
        uicfg.btn_style.clone(),
        assets.npimg_button.clone(),
        nps.npmeta_button.clone(),
        menu, btn::EnterGame(GameMode::Scenario1)
    );

    spawn_button(
        &mut cmd,
        uicfg.btn_style.clone(),
        assets.npimg_button.clone(),
        nps.npmeta_button.clone(),
        menu, btn::EnterGame(GameMode::DevPlayground)
    );

    spawn_button(
        &mut cmd,
        uicfg.btn_style.clone(),
        assets.npimg_button.clone(),
        nps.npmeta_button.clone(),
        menu, btn::ExitApp
    );
}

impl Btn for btn::EnterGame {
    fn fill_content(&self) -> String {
        match self.0 {
            GameMode::DevPlayground => "Dev Playground",
            GameMode::Scenario1 => "Main Scenario",
        }.into()
    }
    fn register_handler(sset: SystemSet) -> SystemSet {
        fn handler(
            In(clicked): In<Option<btn::EnterGame>>,
            mut state: ResMut<State<AppState>>,
        ) {
            if let Some(gm) = clicked {
                state.set(AppState::GameAssetLoading(gm.0)).unwrap();
            }
        }
        sset.with_system(
            crate::ui::button_connector::<Self>.system()
                .chain(handler)
        )
    }
}

impl Btn for btn::ExitApp {
    fn fill_content(&self) -> String {
        "Exit Game".into()
    }
    fn register_handler(sset: SystemSet) -> SystemSet {
        fn handler(
            In(clicked): In<Option<btn::ExitApp>>,
            mut exit: EventWriter<AppExit>,
        ) {
            if clicked.is_some() {
                exit.send(AppExit);
            }
        }
        sset.with_system(
            crate::ui::button_connector::<Self>.system()
                .chain(handler)
        )
    }
}

