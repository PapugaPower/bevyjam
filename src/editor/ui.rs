use bevy::prelude::*;
use enum_iterator::IntoEnumIterator;

use crate::ui::{UiAssets, UiConfig};

use super::{UsingTool, EditorHideCleanup};

#[derive(Component)]
struct EditorBtn;

#[derive(Component, Clone, Copy)]
pub(super) struct ToolBtn(UsingTool);

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

pub(super) fn spawn_ui(
    mut cmd: Commands,
    uiassets: Res<UiAssets>,
    uicfg: Res<UiConfig>,
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

    cmd.entity(heading).push_children(&[heading_text]);
    cmd.entity(top).push_children(&[heading]);

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
}
