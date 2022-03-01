#![allow(unused_imports)]

/// # HOW TO ADD A NEW BLUEPRINT TYPE
///
/// - have a unique marker component type
/// - `impl Blueprint for MyMarker {}`
/// - register it in `BlueprintsPlugin`
/// - insert it in `add_blueprint_meta`
/// - create new init function
///   (you can copypaste `init_bp_medkit` as a template)
///   - use your new marker, in the `BlueprintQuery` param
///   - in the body, insert whatever components you want
///   - be sure to preserve the transform
///

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashSet;
use heron::*;

use crate::FuckStages;

use super::GameAssets;

use crate::game::audio2d::*;
use crate::game::crosshair::*;
use crate::game::damage::*;
use crate::game::doors::*;
use crate::game::enemies::*;
use crate::game::environment::*;
use crate::game::main_camera::*;
use crate::game::player::*;
use crate::game::phys_layers::*;
use crate::game::player_triggers::*;
use crate::game::shooting::*;
use crate::game::timer::*;
use crate::game::world_interaction::*;

pub struct BlueprintsPlugin;

impl Plugin for BlueprintsPlugin {
    fn build(&self, app: &mut App) {
        // registration: add our own types that should be exported to scenes:
        app.register_type::<Medkit>();
        app.register_type::<MultiUse>();
        app.add_startup_system(add_blueprint_meta);
        //
        app.add_system_set_to_stage(
            FuckStages::Post,
            SystemSet::new()
                .with_system(init_bp_medkit)
        );
    }
}

pub trait Blueprint: Component + Reflect + Default + Clone {
    const EDITOR_ID: &'static str;
    const DEFAULT_Z: f32;
}

#[derive(SystemParam)]
struct BlueprintQuery<'w, 's, T: Blueprint> {
    query: Query<'w, 's, (Entity, &'static Transform), Added<T>>,
}

/// List of types that may be serialized by the scene exporter
pub struct ExportableTypes {
    pub names: HashSet<&'static str>,
}

fn add_blueprint_meta(mut commands: Commands) {
    let mut names = HashSet::default();
    // add everything that might be used in a blueprint here:
    names.insert("Transform");
    names.insert("Medkit");
    names.insert("MultiUse");
    commands.insert_resource(ExportableTypes { names });
}

#[derive(Bundle, Default)]
pub struct BlueprintBundle<T: Blueprint> {
    pub transform: Transform,
    pub marker: T,
}

// MEDKITS

#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Medkit {
    pub healing: f32,
}

impl Blueprint for Medkit {
    const EDITOR_ID: &'static str = "Medkit";
    const DEFAULT_Z: f32 = 1.0;
}

fn init_bp_medkit(
    mut commands: Commands,
    q_bp: BlueprintQuery<Medkit>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for (e, xf) in q_bp.query.iter() {
            commands.entity(e)
                // scene export support
                .insert(crate::scene_exporter::SaveSceneMarker)
                // editor support
                .insert(crate::editor::controls::EditableSprite)
                // sprite stuff
                .insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(32., 32.)),
                        color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                        ..Default::default()
                    },
                    // preserve the transform
                    transform: *xf,
                    texture: assets.medkit.clone(),
                    ..Default::default()
                })
                .insert(PlayerPresenceDetector { detected: false })
                .insert(Interactive::default())
                .insert(RigidBody::Sensor)
                .insert(CollisionLayers::none()
                    .with_group(PhysLayer::PlayerTriggers)
                    .with_masks(&[PhysLayer::Player]))
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(20., 20., 1.),
                    border_radius: None,
                });
        }
    }
}

