//! # HOW TO ADD A NEW BLUEPRINT TYPE
//!
//! - have a unique marker component type
//! - `impl Blueprint for MyMarker {}`
//! - register it in `BlueprintsPlugin`
//! - insert it in `add_blueprint_meta`
//! - create new init function
//!   (you can copypaste `init_bp_medkit` as a template)
//!   - use your new marker, in the `BlueprintQuery` param
//!   - in the body, insert whatever components you want
//!   - be sure to preserve the transform
//!

#![allow(unused_imports)]

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
        app.register_type::<Healing>();
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

pub trait BlueprintMarker: Component + Reflect + Default + Clone {
    const EDITOR_ID: &'static str;
    const DEFAULT_Z: f32;
    type BlueprintBundle: Bundle + Default = BasicBlueprintBundle<Self>;
}

/// List of types that may be serialized by the scene exporter
pub struct ExportableTypes {
    pub names: HashSet<&'static str>,
}

fn add_blueprint_meta(mut commands: Commands) {
    let mut names = HashSet::default();
    // registration: add our own types that should be exported to scenes:
    names.insert("Transform");
    names.insert("Medkit");
    names.insert("Healing");
    names.insert("MultiUse");
    commands.insert_resource(ExportableTypes { names });
}

// PLAYER

// impl Blueprint for Player {
//     const EDITOR_ID: &'static str = "Player";
//     const DEFAULT_Z: f32 = 10.0;
// }

// MEDKITS

#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Medkit;

impl BlueprintMarker for Medkit {
    const EDITOR_ID: &'static str = "Medkit";
    const DEFAULT_Z: f32 = 1.0;
}

fn init_bp_medkit(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&Healing>, Option<&MultiUse>), Added<Medkit>>,
    extra: Option<Res<GameAssets>>,
)
{
    if let Some(extra) = extra {
        for (e, xf , mk, mu) in query.iter() {
            commands.entity(e).insert_bundle(
                MedkitBundle::from_blueprint(&*extra, xf, mk, mu)
            );
        }
    } else {
        for (e, _, _, _) in query.iter() {
            commands.entity(e).despawn();
        }
    }
}

#[derive(Bundle, Default)]
pub struct BasicBlueprintBundle<B: BlueprintMarker> {
    transform: Transform,
    marker: B,
}

