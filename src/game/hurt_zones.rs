use bevy::prelude::*;
use bevy::prelude::KeyCode::Period;
use heron::{CollisionEvent, CollisionLayers, CollisionShape, RigidBody};
use heron::rapier_plugin::{PhysicsWorld, ShapeCastCollisionType};
use crate::game::phys_layers::PhysLayer;
use crate::game::player::{Player, PlayerHealth};
use crate::game::player_triggers::{PeriodicActivation, PlayerPresenceDetector};

#[derive(Component)]
pub struct PlayerHurtZone {
    pub damage_per_tick: f32
}

pub fn setup_dev_hurt_zone(mut commands: Commands){
    let mut tform = Transform::from_scale(Vec3::new(2., 2., 1.));
    tform.translation = Vec3::new(250.,0.,-0.1);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                color: Color::rgba(0.4, 0.1, 0.1, 0.3),
                ..Default::default()
            },
            transform: tform,
            ..Default::default()
        })
        .insert(PlayerHurtZone { damage_per_tick: 15.0, })
        .insert(PlayerPresenceDetector { detected: false })
        .insert(PeriodicActivation { frequency: 0.5, last_update: 0.0})
        .insert(RigidBody::Sensor)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::PlayerTriggers)
            .with_masks(&[PhysLayer::Player]))
        .insert(CollisionShape::Sphere {radius: 50.0});

    let mut tform2 = Transform::from_xyz(50., 0., 0.);
    tform2.scale = Vec3::new(3., 1.2, 1.0);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 50.)),
                color: Color::rgba(0.4, 0.1, 0.1, 0.3),
                ..Default::default()
            },
            transform: tform2,
            ..Default::default()
        })
        .insert(PlayerHurtZone { damage_per_tick: 1.0, })
        .insert(PlayerPresenceDetector { detected: false })
        .insert(PeriodicActivation { frequency: 0.2, last_update: 0.0})
        .insert(RigidBody::Sensor)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::PlayerTriggers)
            .with_masks(&[PhysLayer::Player]))
        .insert(CollisionShape::Cuboid {           
            half_extends: Vec3::new(100., 50., 1.),
            border_radius: None,});
}

pub fn evaluate_hurt_zones(mut hp_q: Query<&mut PlayerHealth, With<Player>>, 
                           mut zone_q: Query<(&PlayerHurtZone, &PlayerPresenceDetector, &mut PeriodicActivation),>, 
                           time: Res<Time>){
    
    let mut player_hp = hp_q.single_mut();
    let delta = time.delta_seconds_f64();
    
    for (zone, detector, mut ticker) in zone_q.iter_mut(){
        ticker.last_update += delta;
        if ticker.frequency < ticker.last_update {
            ticker.last_update = 0.0;
            if detector.detected {
                player_hp.current -= zone.damage_per_tick;
                println!("Zone applied {} points of damage, current player HP is {}", zone.damage_per_tick, player_hp.current)
            }
        }
    }
}

