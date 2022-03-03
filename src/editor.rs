use bevy::prelude::*;
use enum_iterator::IntoEnumIterator;
use iyes_bevy_util::{despawn_with_recursive, despawn_with, remove_from_all};

use crate::{AppState, FuckStages, ui::button_connector, game::blueprints::Medkit};

use self::collider::{DragHandle, EditableCollider};

mod ui;

mod select;
mod transform;
pub mod collider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
#[derive(IntoEnumIterator)]
pub enum UsingTool {
    Select,
    Move,
    Rotate,
    EditCollider,
}

/// Add to entities that should not be selectable with the editor
#[derive(Component)]
pub struct NoEditor;

#[derive(Component)]
struct EditorHideCleanup;

#[derive(Component)]
pub struct NewlySpawned;

pub struct DevEditorPlugin;

impl Plugin for DevEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<select::Selections>();
        app.init_resource::<ui::SpawnBtnParent>();
        app.insert_resource(UsingTool::Select);
        app.add_system(enter_exit_editor);
        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(select::selection_track_collider)
                .with_system(select::selection_track_target
                    .after(bevy::transform::TransformSystem::TransformPropagate))
                .with_system(collider::draghandles_track_collider
                    .before(bevy::transform::TransformSystem::TransformPropagate))
        );
        app.add_system_to_stage(FuckStages::Pre, collider::collider_apply_sync);
        app.add_system_set(
            SystemSet::on_enter(AppState::DevEditor)
                .with_system(ui::spawn_ui.label("editorui"))
                .with_system(remove_from_all::<NewlySpawned>)
                .with_system(select::set_selection_visibility::<true>)
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::DevEditor)
                .with_system(despawn_with_recursive::<EditorHideCleanup>)
                .with_system(select::set_selection_visibility::<false>)
                .with_system(collider::cleanup_collider_visualizations)
        );
        app.add_stage_after(CoreStage::Update, ToolStage, SystemStage::single_threaded());
        app.add_state_to_stage(ToolStage, ToolState::Inactive);
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_update(ToolState::Using(UsingTool::Select))
                .with_system(select::mouse_select)
        );
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_update(ToolState::Using(UsingTool::Move))
                .with_system(transform::mouse_move_selections)
        );
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_update(ToolState::Using(UsingTool::Rotate))
                .with_system(transform::mouse_rotate_selections)
        );
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_enter(ToolState::Using(UsingTool::EditCollider))
                .with_system(collider::spawn_draghandles)
        );
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_update(ToolState::Using(UsingTool::EditCollider))
                .with_system(collider::mouse_edit_collider)
        );
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_exit(ToolState::Using(UsingTool::EditCollider))
                .with_system(despawn_with::<DragHandle>)
        );
        app.add_system_set_to_stage(
            ToolStage,
            SystemSet::on_update(ToolState::Spawning)
                .with_system(transform::mouse_move_newlyspawned)
        );
        app.add_system_set(
            SystemSet::on_update(AppState::DevEditor)
                .with_system(ui::tool_btn_visual)
                .with_system(select::keyboard_despawn_selected)
                .with_system(collider::visualize_spriteless_colliders)
                .with_system(collider::update_collider_visualization)
                .with_system(transform::editor_camera)
                .with_system(button_connector::<ui::ToolBtn>.chain(ui::tool_btn_handler))
                // handle spawn buttons for blueprints:
                .with_system(button_connector.chain(ui::spawn_btn_handler::<EditableCollider>))
                .with_system(button_connector.chain(ui::spawn_btn_handler::<Medkit>))
        );
        app.add_system_set(
            SystemSet::on_enter(AppState::DevEditor).after("editorui")
                // add spawn buttons for blueprints:
                .with_system(ui::add_spawn_button::<EditableCollider>)
                .with_system(ui::add_spawn_button::<Medkit>)
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
#[derive(StageLabel)]
struct ToolStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
enum ToolState {
    Using(UsingTool),
    Spawning,
    Inactive,
}

fn enter_exit_editor(
    kbd: Res<Input<KeyCode>>,
    mut state: ResMut<State<AppState>>,
    mut toolstate: ResMut<State<ToolState>>,
    tool: Res<UsingTool>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if state.current() == &AppState::DevEditor {
            state.pop().unwrap();
            toolstate.pop().unwrap();
        } else {
            state.push(AppState::DevEditor).unwrap();
            toolstate.push(ToolState::Using(*tool)).unwrap();
        }
    }
}
