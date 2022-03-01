use bevy::prelude::*;
use heron::{CollisionLayers, CollisionShape, RigidBody};
use crate::game::dev::DevAssets;
use crate::game::phys_layers::PhysLayer;
use crate::game::damage::Health;
use crate::game::player::Player;
use crate::game::player_triggers::PlayerPresenceDetector;

use super::blueprints::Medkit;

#[derive(Component)]
pub struct MultiUse {
    pub remaining: i32,
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

pub fn process_world_medkit_use(kit_q: Query<(Entity, &Medkit, Option<&DespawnAfterInteraction>), With<InteractionDirty>>,
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


// DEV SCENE
pub fn spawn_test_medkits(mut commands: Commands, assets: Res<DevAssets>) {
    let tform2 = Transform::from_xyz(-150., 100., -0.02);
    commands.spawn()
        .insert(tform2)
        .insert(Medkit { healing: 25.0 })
        .insert(MultiUse { remaining: 3 });

    let tform = Transform::from_xyz(-190., 100., -0.02);
    commands.spawn()
        .insert(tform)
        .insert(Medkit { healing: 60.0 });
}
