use bevy::prelude::*;
use heron::{CollisionLayers, CollisionShape, RigidBody};
use crate::editor::controls::EditableSpriteBundle;
use crate::game::dev::DevAssets;
use crate::game::phys_layers::PhysLayer;
use crate::game::damage::Health;
use crate::game::player::Player;
use crate::game::player_triggers::PlayerPresenceDetector;

use super::{GameAssets, blueprints};
use super::blueprints::BlueprintMarker;

#[derive(Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct MultiUse {
    pub remaining: u32,
}

#[derive(Component)]
pub struct Interactive{
    pub timeout: f32,
}

#[derive(Component)]
pub struct InteractionDirty;

#[derive(Component)]
pub struct DespawnAfterInteraction;

#[derive(Component)]
pub struct ReadyToDespawn;

#[derive(Component)]
pub struct InteractionTimeout{
    pub time: f32,
    pub elapsed: f32
}


impl Default for Interactive {
    fn default() -> Self {
        Self  {
            timeout: 0.5
        }
    }
}

impl Default for InteractionTimeout{
    fn default() -> Self {
        Self {
            time: 1.0,
            elapsed: 0.0
        }
    }
}

#[derive(Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Healing {
    pub healing: f32,
}

/// Bevy Bundle for easy spawning of entities
#[derive(Bundle)]
pub struct MedkitBundle {
    // include base bundle for rendering
    #[bundle]
    sprite: SpriteBundle,
    // cleanup marker
    cleanup: super::GameCleanup,
    // editor enablement
    #[bundle]
    editor: EditableSpriteBundle<blueprints::Medkit>,
    // our game behaviors
    healing: Healing,
    multi_use: MultiUse,
    presence: PlayerPresenceDetector,
    interactive: Interactive,
    // physics
    rigidbody: RigidBody,
    phys_layers: CollisionLayers,
    phys_shape: CollisionShape,
}

impl MedkitBundle {
    pub fn from_blueprint(
        assets: &GameAssets,
        transform: &Transform,
        healing: Option<&Healing>,
        multi_use: Option<&MultiUse>,
    ) -> Self {
        MedkitBundle {
            sprite: SpriteBundle {
                texture: assets.medkit.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(32., 32.)),
                    color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                    ..Default::default()
                },
                transform: *transform,
                ..Default::default()
            },
            cleanup: super::GameCleanup,
            editor: EditableSpriteBundle::default(),
            healing: healing.cloned().unwrap_or_default(),
            multi_use: multi_use.cloned().unwrap_or_default(),
            presence: PlayerPresenceDetector::default(),
            interactive: Interactive::default(),
            rigidbody: RigidBody::Sensor,
            phys_layers: CollisionLayers::none()
                .with_group(PhysLayer::PlayerTriggers)
                .with_masks(&[PhysLayer::Player]),
            phys_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(20., 20., 1.),
                border_radius: None,
            },
        }
    }
}

pub fn process_world_medkit_use(kit_q: Query<(Entity, &Healing, Option<&DespawnAfterInteraction>), With<InteractionDirty>>,
                                mut health_q: Query<&mut Health, With<Player>>, mut commands: Commands
){
    let mut player_hp = health_q.single_mut();
    for (e, medkit, despawn) in kit_q.iter(){
        let new_hp = (player_hp.current + medkit.healing).clamp(0.0, player_hp.max);
        player_hp.current = new_hp;
        println!("Player used a world medkit. Healing: {}", medkit.healing);
        
        commands.entity(e).remove::<InteractionDirty>();
        
        if let Some(despawn) = despawn {
            commands.entity(e).remove::<DespawnAfterInteraction>().
                insert(ReadyToDespawn);
            println!("Item depleted, despawning.");
        }
    }
    
}

// Tags used world items with 'cooldown', 'destruction' and 'dirty' components
pub fn process_new_interactions(mut inter_q: Query<(Entity, &Interactive, &PlayerPresenceDetector, Option<&mut MultiUse>, Option<&mut InteractionTimeout>), Without<InteractionDirty>>,
                                input: Res<Input<KeyCode>>,
                                mut commands: Commands
){
    if !input.just_pressed(KeyCode::E) { return };
    for (e, interactive, trigger, multi_use, timeout) in inter_q.iter_mut() {
        if !trigger.detected { continue }
        commands.entity(e)
            .insert(InteractionDirty)
            .remove::<Interactive>(); 
        
        if let Some(mut multi_use) = multi_use {
            if multi_use.remaining <= 1 {
                commands.entity(e).insert(DespawnAfterInteraction)
                    .remove::<MultiUse>();
            } else {
                multi_use.remaining -= 1;
            }
        } else {
            commands.entity(e).insert(DespawnAfterInteraction);
        }
        
        if let Some(mut timeout) = timeout {
            timeout.elapsed = 0.0;
        } else {
            commands.entity(e).insert(InteractionTimeout::default());
        }
    }
}

pub fn process_interaction_timeouts(mut inter_q: Query<(Entity, &mut InteractionTimeout)>, 
                                    time: Res<Time>, 
                                    mut commands: Commands
){
    for (e, mut timeout) in inter_q.iter_mut(){
        timeout.elapsed += time.delta_seconds();
        if timeout.elapsed >= timeout.time {
            commands.entity(e).insert(Interactive {timeout: timeout.time })
                .remove::<InteractionTimeout>();
        }
    }
}

pub fn process_interactable_despawn(q: Query<Entity, With<ReadyToDespawn>>, 
                                    mut commands: Commands
){
    for entity in q.iter(){
        commands.entity(entity).despawn();
    }
}
