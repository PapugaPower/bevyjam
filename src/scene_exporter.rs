use std::path::PathBuf;

use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::system::{CommandQueue, SystemParam, SystemState};
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::reflect::TypeRegistryArc;
use bevy::scene::DynamicEntity;
use bevy::ecs::event::Events;

use crate::game::blueprints::ExportableTypes;
use crate::{AppState, FuckStages};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SaveSceneMarker;
pub struct SaveScenePath(pub PathBuf);
pub struct SaveSceneEvent;

fn has_save_event(e: Res<Events<SaveSceneEvent>>) -> ShouldRun {
    let mut result = ShouldRun::No;
    if !e.is_empty() {
        result = ShouldRun::Yes;
    }
    result
}

fn save_scene(world: &mut World) {
    let path = world.get_resource::<SaveScenePath>().unwrap().0.clone();
    world.get_resource_mut::<Events<SaveSceneEvent>>().unwrap().clear();

    let mut ss = SystemState::<(
        Res<ExportableTypes>,
        Query<Entity, With<SaveSceneMarker>>
    )>::new(world);
    let (et, q) = ss.get(world);
    let entities = q.iter().collect();

    let type_registry = world.get_resource::<TypeRegistry>().unwrap();
    let scene = scene_from_entities(world, type_registry, entities, &*et);
    let scene = scene.serialize_ron(type_registry).unwrap();

    match std::fs::write(&path, scene) {
        Ok(()) => info!("Scene Saved to {:?}", path),
        Err(e) => error!("Could not save scene to {:?}: {}", path, e),
    }
}

pub fn scene_from_entities(
    world: &World,
    type_registry: &TypeRegistryArc,
    entities: Vec<Entity>,
    typenames: &ExportableTypes,
) -> DynamicScene {
    let mut scene = DynamicScene::default();
    let type_registry = type_registry.read();

    for archetype in world.archetypes().iter() {
        let entities_offset = scene.entities.len();

        // Create a new dynamic entity for each entity of the given archetype
        // and insert it into the dynamic scene.
        for entity in archetype.entities().iter().filter(|e| entities.contains(e)) {
            scene.entities.push(DynamicEntity {
                entity: entity.id(),
                components: Vec::new(),
            });
        }

        // Add each reflection-powered component to the entity it belongs to.
        for component_id in archetype.components() {
            let type_registration = world
                .components()
                .get_info(component_id)
                .and_then(|info| type_registry.get(info.type_id().unwrap()));
            if let Some(name) = type_registration.map(|r| r.short_name()) {
                if !typenames.names.contains(name) {
                    continue;
                }
            }
            let reflect_component = type_registration.and_then(|registration| registration.data::<ReflectComponent>());
            if let Some(reflect_component) = reflect_component {
                for (i, entity) in archetype
                    .entities()
                    .iter()
                    .filter(|e| entities.contains(e))
                    .enumerate()
                {
                    if let Some(component) = reflect_component.reflect_component(world, *entity) {
                        scene.entities[entities_offset + i]
                            .components
                            .push(component.clone_value());
                    }
                }
            }
        }
    }

    scene
}

fn save_scene_on_key(
    input: Res<Input<KeyCode>>,
    mut evw: EventWriter<SaveSceneEvent>,
) {
    if input.just_pressed(KeyCode::F10) {
        evw.send(SaveSceneEvent);
    }
}

/*
fn load_scene(
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut state: ResMut<State<GameState>>,
) {
    let scene_handle = asset_server.load("levels/level1.scn.ron");
    scene_spawner.spawn_dynamic(scene_handle);
    state.replace(GameState::PostLoadLevel).unwrap();
}
*/

pub struct SerializePlugin;
impl Plugin for SerializePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(FuckStages::Post,
            save_scene
                .exclusive_system()
                .with_run_criteria(has_save_event),
        );
        app.add_event::<SaveSceneEvent>();
        app.insert_resource(SaveScenePath("test.ron".into()));
        app.add_system(save_scene_on_key);
    }
}
