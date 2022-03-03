use bevy::{prelude::*, app::AppExit};

use iyes_bevy_util::{despawn_with_recursive};

use crate::{GameMode, AppState, FuckStages, game::GameResult};

use super::{UiAssets, UiNinepatches, ContentId, UiConfig, Btn, fill_btn, spawn_button};

mod btn {
    use bevy::prelude::*;

    #[derive(Component, Clone, Copy)]
    pub struct ReplayGame;
    #[derive(Component, Clone, Copy)]
    pub struct GotoMenu;
}

#[derive(Component)]
struct GameoverUiCleanup;

pub struct GameoverUiPlugin;

impl Plugin for GameoverUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::GameOver)
                .with_system(init_ui)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::GameOver)
                .with_system(despawn_with_recursive::<GameoverUiCleanup>)
        );

        let update = SystemSet::on_update(AppState::GameOver);
        let update = btn::GotoMenu::register_handler(update);
        let update = btn::ReplayGame::register_handler( update);
        app.add_system_set(update);

        app.add_system_to_stage(FuckStages::Post, fill_btn::<btn::ReplayGame>);
        app.add_system_to_stage(FuckStages::Post, fill_btn::<btn::GotoMenu>);
    }
}

fn init_ui(
    mut cmd: Commands,
    assets: Res<UiAssets>,
    uicfg: Res<UiConfig>,
    nps: Res<UiNinepatches>,
    gres: Res<GameResult>,
) {
    let dialog = cmd.spawn_bundle(NodeBundle {
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
    }).insert(GameoverUiCleanup).id();

    let heading = cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            padding: Rect::all(Val::Px(4.0)),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let heading_text = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            match *gres {
                GameResult::LoseHealth => "GAME OVER",
                GameResult::LoseTime => "GAME OVER",
                GameResult::Win => "GREAT SUCCESS!",
            },
            uicfg.heading_style_text.clone(),
            Default::default()
        ),
        ..Default::default()
    }).id();

    cmd.entity(heading).push_children(&[heading_text]);

    let middle = cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            padding: Rect::all(Val::Px(4.0)),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let middle_text = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            match *gres {
                GameResult::LoseHealth => "You died. :(",
                GameResult::LoseTime => "You ran out of time. It's all over now...",
                GameResult::Win => "You really showed 'em how it's done! Good job!",
            },
            uicfg.dialog_style_text.clone(),
            Default::default()
        ),
        ..Default::default()
    }).id();

    cmd.entity(middle).push_children(&[middle_text]);

    let btnrow = cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::rgb(0.4, 0.4, 0.4)),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    spawn_button(
        &mut cmd,
        uicfg.btn_style.clone(),
        assets.npimg_button.clone(),
        nps.npmeta_button.clone(),
        btnrow, btn::ReplayGame
    );

    spawn_button(
        &mut cmd,
        uicfg.btn_style.clone(),
        assets.npimg_button.clone(),
        nps.npmeta_button.clone(),
        btnrow, btn::GotoMenu
    );

    cmd.entity(dialog).push_children(&[heading, middle, btnrow]);
}

impl Btn for btn::ReplayGame {
    fn fill_content(&self) -> String {
        "Replay Scenario".into()
    }
    fn register_handler(sset: SystemSet) -> SystemSet {
        fn handler(
            In(clicked): In<Option<btn::ReplayGame>>,
            mut state: ResMut<State<AppState>>,
            gm: Res<GameMode>,
        ) {
            if clicked.is_some() {
                state.replace(AppState::InGame(*gm)).ok();
            }
        }
        sset.with_system(
            crate::ui::button_connector::<Self>.system()
                .chain(handler)
        )
    }
}

impl Btn for btn::GotoMenu {
    fn fill_content(&self) -> String {
        "Exit to Menu".into()
    }
    fn register_handler(sset: SystemSet) -> SystemSet {
        fn handler(
            In(clicked): In<Option<btn::GotoMenu>>,
            mut state: ResMut<State<AppState>>,
        ) {
            if clicked.is_some() {
                state.replace(AppState::MainMenu).ok();
            }
        }
        sset.with_system(
            crate::ui::button_connector::<Self>.system()
                .chain(handler)
        )
    }
}


