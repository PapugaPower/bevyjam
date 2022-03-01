use bevy::prelude::*;
use enum_iterator::IntoEnumIterator;
use iyes_bevy_util::despawn_with_recursive;

use crate::{AppState, FuckStages, ui::button_connector};

mod ui;

mod select;
mod transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(IntoEnumIterator)]
pub enum UsingTool {
    Select,
    Move,
    Rotate,
}

#[derive(Component)]
struct EditorHideCleanup;

pub struct DevEditorPlugin;

impl Plugin for DevEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<select::Selections>();
        app.insert_resource(UsingTool::Select);
        app.add_system(enter_exit_editor);
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            select::selection_track_target
                .after(bevy::transform::TransformSystem::TransformPropagate)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::DevEditor)
                .with_system(button_connector::<ui::ToolBtn>.chain(ui::tool_btn_handler))
                .with_system(ui::tool_btn_visual)
                .with_system(select::mouse_select_sprite)
                .with_system(transform::mouse_move_selections)
                .with_system(transform::mouse_rotate_selections)
        );
        app.add_system_set(
            SystemSet::on_enter(AppState::DevEditor)
                .with_system(ui::spawn_ui)
                .with_system(select::set_selection_visibility::<true>)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::DevEditor)
                .with_system(despawn_with_recursive::<EditorHideCleanup>)
                .with_system(select::set_selection_visibility::<false>)
        );
    }
}

pub mod controls {
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct EditableSprite;
}

pub fn enter_exit_editor(
    kbd: Res<Input<KeyCode>>,
    mut state: ResMut<State<AppState>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if state.current() == &AppState::DevEditor {
            state.pop().unwrap();
        } else {
            state.push(AppState::DevEditor).unwrap();
        }
    }
}
