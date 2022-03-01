#![allow(unused_imports)]

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
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
        app.add_system_set_to_stage(
            FuckStages::Post,
            SystemSet::new()
                .with_system(init_bp_medkit)
        );
    }
}

trait Blueprint: Component + Reflect {}

#[derive(SystemParam)]
struct BlueprintQuery<'w, 's, T: Blueprint> {
    query: Query<'w, 's, (Entity, &'static Transform), Added<T>>,
}

// MEDKITS

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct Medkit {
    pub healing: f32,
}

impl Blueprint for Medkit {}

fn init_bp_medkit(
    mut commands: Commands,
    q_bp: BlueprintQuery<Medkit>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for (e, xf) in q_bp.query.iter() {
            commands.entity(e)
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

