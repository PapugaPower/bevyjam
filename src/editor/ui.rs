use std::marker::PhantomData;

use bevy::prelude::*;
use enum_iterator::IntoEnumIterator;

use crate::{ui::{UiAssets, UiConfig}, game::blueprints::{Blueprint, BlueprintBundle, Medkit}, util::WorldCursor};

use super::{UsingTool, EditorHideCleanup, NewlySpawned};

#[derive(Component)]
struct EditorBtn;

#[derive(Default)]
pub struct SpawnBtnParent(Option<Entity>);

#[derive(Component, Clone, Copy)]
pub(super) struct ToolBtn(UsingTool);

#[derive(Component, Clone, Copy)]
pub(super) struct SpawnBtn<T: Blueprint>(PhantomData<T>);

impl ToolBtn {
    fn label(&self) -> &'static str {
        match self.0 {
            UsingTool::Select => "Select",
            UsingTool::Move => "Move",
            UsingTool::Rotate => "Rotate",
        }
    }
}

pub(super) fn tool_btn_handler(
    In(clicked): In<Option<ToolBtn>>,
    mut using: ResMut<UsingTool>,
) {
    if let Some(btn) = clicked {
        *using = btn.0;
    }
}

pub(super) fn spawn_btn_handler<T: Blueprint>(
    In(clicked): In<Option<SpawnBtn<T>>>,
    mut commands: Commands,
    crs: Res<WorldCursor>,
) {
    if clicked.is_some() {
        commands.spawn_bundle(BlueprintBundle {
            transform: Transform::from_translation(crs.0.extend(T::DEFAULT_Z)),
            marker: T::default(),
        }).insert(NewlySpawned);
    }
}

pub(super) fn tool_btn_visual(
    mut query: Query<
        (&Interaction, &ToolBtn, &mut UiColor),
        /*Changed<Interaction>,*/
    >,
    using: Res<UsingTool>,
) {
    const COLOR_ACTIVE: UiColor = UiColor(Color::rgb(0.5, 0.5, 0.5));
    const COLOR_IDLE: UiColor = UiColor(Color::rgb(0.9, 0.9, 0.9));
    const COLOR_HOVER: UiColor = UiColor(Color::rgb(0.8, 0.8, 0.8));
    const COLOR_CLICK: UiColor = UiColor(Color::rgb(0.75, 0.75, 0.75));
    for (interaction, btn, mut color) in query.iter_mut() {
        if *using == btn.0 {
            *color = COLOR_ACTIVE;
            continue;
        }
        match interaction {
            Interaction::None => {
                *color = COLOR_IDLE;
            },
            Interaction::Hovered => {
                *color = COLOR_HOVER;
            },
            Interaction::Clicked => {
                *color = COLOR_CLICK;
            }
        }
    }
}

pub(super) fn add_spawn_button<T: Blueprint>(
    mut cmd: Commands,
    uicfg: Res<UiConfig>,
    uiassets: Res<UiAssets>,
    btnrow: Res<SpawnBtnParent>,
) {
    let textstyle_btn = TextStyle {
        color: Color::BLACK,
        font_size: 16.0,
        font: uiassets.font_menu_regular.clone(),
    };

    let spbtn = SpawnBtn::<T>(PhantomData);
    let btntext = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            Medkit::EDITOR_ID,
            textstyle_btn.clone(),
            Default::default()
        ),
        ..Default::default()
    }).id();

    let btn = cmd.spawn_bundle(ButtonBundle {
        color: UiColor(Color::rgb(0.75, 0.75, 0.75)),
        style: uicfg.btn_style.clone(),
        ..Default::default()
    }).insert(EditorBtn).insert(spbtn).id();
    cmd.entity(btn).push_children(&[btntext]);
    cmd.entity(btnrow.0.unwrap()).push_children(&[btn]);
}

pub(super) fn spawn_ui(
    mut cmd: Commands,
    uiassets: Res<UiAssets>,
    uicfg: Res<UiConfig>,
    mut r_btnrow: ResMut<SpawnBtnParent>,
) {
    let textstyle_btn = TextStyle {
        color: Color::BLACK,
        font_size: 16.0,
        font: uiassets.font_menu_regular.clone(),
    };

    let top = cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::rgb(1.0, 1.0, 1.0)),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(100.0),
                top: Val::Auto,
                left: Val::Px(100.0),
                right: Val::Auto,
            },
            //margin: Rect::all(Val::Auto),
            //align_self: AlignSelf::Center,
            flex_direction: FlexDirection::ColumnReverse,
            //align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).insert(EditorHideCleanup).id();

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
            "EDITOR TOOLS:",
            uicfg.heading_style_text.clone(),
            Default::default()
        ),
        ..Default::default()
    }).id();

    let dialog = cmd.spawn_bundle(NodeBundle {
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

    let dialog_text = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            "(left click)",
        uicfg.dialog_style_text.clone(),
            Default::default()
        ),
        ..Default::default()
    }).id();

    cmd.entity(heading).push_children(&[heading_text]);
    cmd.entity(dialog).push_children(&[dialog_text]);
    cmd.entity(top).push_children(&[heading, dialog]);

    for tool in UsingTool::into_enum_iter() {
        let toolbtn = ToolBtn(tool);
        let btntext = cmd.spawn_bundle(TextBundle {
            text: Text::with_section(
                toolbtn.label(),
                textstyle_btn.clone(),
                Default::default()
            ),
            ..Default::default()
        }).id();

        let btn = cmd.spawn_bundle(ButtonBundle {
            style: uicfg.btn_style.clone(),
            ..Default::default()
        }).insert(EditorBtn).insert(toolbtn).id();
        cmd.entity(btn).push_children(&[btntext]);
        cmd.entity(top).push_children(&[btn]);
    }

    let top2 = cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::rgb(1.0, 1.0, 1.0)),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(100.0),
                top: Val::Auto,
                right: Val::Px(100.0),
                left: Val::Auto,
            },
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).insert(EditorHideCleanup).id();

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
            "SPAWN NEW:",
            uicfg.heading_style_text.clone(),
            Default::default()
        ),
        ..Default::default()
    }).id();

    cmd.entity(heading).push_children(&[heading_text]);

    let btnrow = cmd.spawn_bundle(NodeBundle {
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

    cmd.entity(top2).push_children(&[heading, btnrow]);

    r_btnrow.0 = Some(btnrow);
}
