use bevy::prelude::*;

use crate::{AppState, FuckStages};

mod select;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsingTool {
    Select,
}

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
                .with_system(select::mouse_select_sprite)
        );
        app.add_system_set(
            SystemSet::on_enter(AppState::DevEditor)
                .with_system(select::set_selection_visibility::<true>)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::DevEditor)
                .with_system(select::set_selection_visibility::<false>)
        );
    }
}

pub mod controls {
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct Selectable;
    #[derive(Component)]
    pub struct TransformEdit;
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
