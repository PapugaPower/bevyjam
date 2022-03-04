//! # HOW TO ADD A NEW BLUEPRINT TYPE
//!
//! - have a unique marker component type
//! - `impl Blueprint for MyMarker {}`
//!   - fill it out with the info for the editor
//! - register it in `BlueprintsPlugin`
//! - insert it in `add_blueprint_meta`
//! - create new init function
//!   (you can copypaste `init_bp_collider` as a template)
//!   - use your new marker, in the `BlueprintQuery` param
//!   - in the body, insert whatever components you want
//!   - be sure to preserve the transform
//!

#![allow(unused_imports)]

use bevy::ecs::system::EntityCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashSet;
use heron::*;

use crate::FuckStages;
use crate::editor::Editable;
use crate::editor::NewlySpawned;
use crate::editor::collider::ColliderEditorVisColor;
use crate::editor::collider::EditableCollider;

use super::GameAssets;
use super::GameCleanup;
use super::collider;
use super::collider::ColliderKind;

use crate::game::audio2d::*;
use crate::game::crosshair::*;
use crate::game::damage::*;
use crate::game::enemies::*;
use crate::game::environment::*;
use crate::game::main_camera::*;
use crate::game::phys_layers::*;
use crate::game::player::*;
use crate::game::shooting::*;
use crate::game::timer::*;

pub struct BlueprintsPlugin;

impl Plugin for BlueprintsPlugin {
    fn build(&self, app: &mut App) {
        // registration: add our own types that should be exported to scenes:
        app.register_type::<Medkit>();
        app.register_type::<MultiUse>();
        app.register_type::<EditableCollider>();
        app.register_type::<collider::Wall>();
        app.register_type::<collider::HurtZone>();
        app.register_type::<collider::WinZone>();
        app.register_type::<collider::SpawnZone>();
        app.add_startup_system(add_blueprint_meta);
        //
        app.add_system_set_to_stage(
            FuckStages::Post,
            SystemSet::new()
                .with_system(init_bp_medkit)
                .with_system(init_bp_collider::<collider::Wall>)
                .with_system(init_bp_collider::<collider::HurtZone>)
                .with_system(init_bp_collider::<collider::WinZone>)
                .with_system(init_bp_collider::<collider::SpawnZone>)
        );
    }
}

/// impl this Trait for a marker type to enable using it with the editor
pub trait Blueprint: Component + Reflect + Default + Clone {
    /// Text for the spawn button in the editor ui
    const EDITOR_ID: &'static str;
    /// Z coordinate to spawn at
    const DEFAULT_Z: f32;

    /// The bundle to use when spawning from the editor
    /// The editor will spawn the new entity with a default instance of this bundle
    type BlueprintBundle: Bundle + Default;
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
    names.insert("EditableCollider");
    names.insert("Wall");
    names.insert("HurtZone");
    names.insert("SpawnZone");
    names.insert("WinZone");
    commands.insert_resource(ExportableTypes { names });
}

/// Simple generic blueprint bundle, if you only want to initialize with a transform and marker
#[derive(Bundle, Default)]
pub struct BasicBlueprintBundle<T: Blueprint> {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub marker: T,
}

#[derive(SystemParam)]
struct BlueprintQuery<'w, 's, T: Blueprint> {
    query: Query<'w, 's, (Entity, &'static T, &'static Transform), Added<T>>,
}

// MEDKITS

#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Medkit {
    pub healing: f32,
}

#[derive(Bundle, Default)]
pub struct MedkitBlueprintBundle {
    pub transform: Transform,
    pub medkit: Medkit,
    pub multi_use: MultiUse,
}

impl Blueprint for Medkit {
    const EDITOR_ID: &'static str = "Medkit";
    const DEFAULT_Z: f32 = 1.0;
    type BlueprintBundle = MedkitBlueprintBundle;
}

fn init_bp_medkit(
    mut commands: Commands,
    q_bp: BlueprintQuery<Medkit>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for (e, _medkit, xf) in q_bp.query.iter() {
            // The editor spawns the entity with a `NewlySpawned` component.
            // This is used to enable positioning it in the scene with the mouse.
            // Since we are reparenting the medkit under the trigger,
            // we have to remove NewlySpawned from the medkit entity
            // and add it to the toplevel trigger entity.

            // trigger for medkit
            commands
                .spawn()
                .insert(GameCleanup)
                .insert(Editable)
                .insert(GlobalTransform::default())
                .insert(*xf)
                .insert(Trigger {
                    player_detected: false,
                    entities: vec![e],
                })
                .insert(RigidBody::Sensor)
                .insert(
                    CollisionLayers::none()
                        .with_group(PhysLayer::PlayerTriggers)
                        .with_masks(&[PhysLayer::Player]),
                )
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(20., 20., 1.),
                    border_radius: None,
                })
                // hack to make spawning from editor work
                .insert(NewlySpawned)
                // medkit is child of sensor
                .add_child(e);

            // medkit
            commands
                .entity(e)
                .insert(crate::scene_exporter::SaveSceneMarker)
                // hack to make spawning from editor work
                .remove::<NewlySpawned>()
                // sprite stuff
                .insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(32., 32.)),
                        color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                        ..Default::default()
                    },
                    texture: assets.medkit.clone(),
                    ..Default::default()
                });
        }
    }
}

// COLLIDERS

#[derive(Bundle, Default)]
pub struct ColliderBlueprintBundle<T: ColliderBehavior> {
    #[bundle]
    basic: BasicBlueprintBundle<T>,
    collider: EditableCollider,
}

pub trait ColliderBehavior: Blueprint {
    const EDITOR_COLOR: Color;
    const KINDENUM: ColliderKind;

    fn fill_blueprint(&self, cmd: &mut EntityCommands);
}

impl Blueprint for collider::Wall {
    const EDITOR_ID: &'static str = "Wall";
    const DEFAULT_Z: f32 = 0.0;
    type BlueprintBundle = ColliderBlueprintBundle<Self>;
}

impl ColliderBehavior for collider::Wall {
    const KINDENUM: ColliderKind = ColliderKind::Wall;
    const EDITOR_COLOR: Color = Color::rgba(1.0, 0.75, 0.5, 0.25);
    fn fill_blueprint(&self, cmd: &mut EntityCommands) {
        cmd
            .insert(GlobalTransform::default())
            .insert(RigidBody::Static)
            .insert(CollisionLayers::none()
                .with_group(PhysLayer::World)
                .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]));
    }
}

impl Blueprint for collider::HurtZone {
    const EDITOR_ID: &'static str = "HurtZone";
    const DEFAULT_Z: f32 = 0.0;
    type BlueprintBundle = ColliderBlueprintBundle<Self>;
}

impl ColliderBehavior for collider::HurtZone {
    const KINDENUM: ColliderKind = ColliderKind::HurtZone;
    const EDITOR_COLOR: Color = Color::rgba(1.0, 0.25, 0.0, 0.25);
    fn fill_blueprint(&self, cmd: &mut EntityCommands) {
        cmd
            .insert(GlobalTransform::default())
            .insert(Pulsing::from(self));
    }
}

impl Blueprint for collider::WinZone {
    const EDITOR_ID: &'static str = "WinZone";
    const DEFAULT_Z: f32 = 0.0;
    type BlueprintBundle = ColliderBlueprintBundle<Self>;
}

impl ColliderBehavior for collider::WinZone {
    const KINDENUM: ColliderKind = ColliderKind::WinZone;
    const EDITOR_COLOR: Color = Color::rgba(0.25, 1.0, 0.5, 0.25);
    fn fill_blueprint(&self, cmd: &mut EntityCommands) {
        cmd
            // the CollisionShape will be automatically inserted by `game::collider::collider_apply_sync`
            // TODO: add any other stuff needed
            .insert(GlobalTransform::default());
    }
}

impl Blueprint for collider::SpawnZone {
    const EDITOR_ID: &'static str = "SpawnZone";
    const DEFAULT_Z: f32 = 0.0;
    type BlueprintBundle = ColliderBlueprintBundle<Self>;
}

impl ColliderBehavior for collider::SpawnZone {
    const KINDENUM: ColliderKind = ColliderKind::SpawnZone;
    const EDITOR_COLOR: Color = Color::rgba(0.25, 0.5, 1.0, 0.25);
    fn fill_blueprint(&self, cmd: &mut EntityCommands) {
        cmd
            // the CollisionShape will be automatically inserted by `game::collider::collider_apply_sync`
            // TODO: add any other stuff needed
            .insert(GlobalTransform::default());
    }
}

fn init_bp_collider<T: ColliderBehavior>(
    mut commands: Commands,
    q_bp: BlueprintQuery<T>,
) {
    for (e, coll, _) in q_bp.query.iter() {
        commands.entity(e)
            .insert(GameCleanup)
            // editor integration
            .insert(Editable)
            .insert(crate::scene_exporter::SaveSceneMarker)
            .insert(T::KINDENUM)
            .insert(ColliderEditorVisColor(T::EDITOR_COLOR));
        coll.fill_blueprint(&mut commands.entity(e));
    }
}
