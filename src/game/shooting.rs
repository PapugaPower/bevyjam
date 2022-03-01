use bevy::asset::HandleId;
use crate::game::crosshair::Crosshair;
use crate::game::damage::{DamageAreaShape, DamageEvent, DamageSource, Pulsing};
use crate::game::phys_layers::PhysLayer;
use crate::game::player::Player;
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioSource};
use heron::{prelude::*, rapier_plugin::PhysicsWorld};
use crate::game::{GameAssets, GameAudioChannel};
use rand::Rng;

#[derive(Component)]
pub struct LastShootTime {
    pub time: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum AmmoType {
    // not physical objects
    Projectile,
    // physics based
    Throwable,
    // just nonphysical static objects
    Static,
}

#[derive(Component)]
pub struct Weapon {
    pub ammo_type: AmmoType,
    pub damage: f32,
    // this is 1 / real_fire_rate
    pub fire_rate: f32,
    // speed of projectile or pulse rate of static
    pub projectile_speed: f32,
    // in seconds
    pub projectile_life_time: f32,
    // in degrees
    pub spread: f32,
    // bullets will be spread equally over `spread`
    pub projectiles_per_shot: u32,
    // how far projectiles should spawn from player
    pub projectile_spawn_offset: f32,
    // for now used to define radius for Throwable and Static
    // does nothing for Projectile
    pub radius_of_effect: f32,
}

#[derive(Component)]
pub struct Armament {
    life_time: Timer,
}

#[derive(Component)]
pub struct Projectile {
    damage: f32,
    direction: Vec3,
    speed: f32,
}

pub fn player_shoot(
    mut commands: Commands,
    mut ev_fired: EventWriter<PlayerFiredEvent>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut query_player: Query<(Entity, &Transform, &Weapon, &mut LastShootTime), With<Player>>,
    mut query_cross: Query<&Transform, With<Crosshair>>,
) {
    // TODO handle input
    if keys.pressed(KeyCode::Space) {
        let (e, player_transform, weapon, mut last_shoot) = query_player.single_mut();
        let cross_transform = query_cross.single_mut();
        let shoot_dir = (cross_transform.translation - player_transform.translation).normalize();
        let spawn_transform = {
            let mut pt = *player_transform;
            pt.translation += shoot_dir * weapon.projectile_spawn_offset;;
            pt
        };

        let now = time.time_since_startup().as_secs_f32();
        if last_shoot.time + weapon.fire_rate <= now {
            // if only one bullet we don't care about spread
            let (spread_edge, spread_step) = if weapon.projectiles_per_shot <= 1 {
                (0.0, 0.0)
            } else {
                (
                    -weapon.spread / 2.0,
                    weapon.spread / (weapon.projectiles_per_shot - 1) as f32,
                )
            };
            
            ev_fired.send(PlayerFiredEvent{entity:e, ammo_type:weapon.ammo_type});
            
            for i in 0..weapon.projectiles_per_shot {
                let shoot_dir = Quat::from_rotation_z((spread_step * i as f32).to_radians())
                    * Quat::from_rotation_z(spread_edge.to_radians() as f32)
                    * shoot_dir;
                match weapon.ammo_type {
                    AmmoType::Projectile => {
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.4, 0.06)),
                                    color: Color::rgba(1.0, 0.8, 0.8, 0.5),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Armament {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                            })
                            .insert(Projectile {
                                damage: weapon.damage,
                                direction: shoot_dir,
                                speed: weapon.projectile_speed,
                            });
                    }
                    AmmoType::Throwable => {
                        let throw_dir = shoot_dir * weapon.projectile_speed;
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.4, 0.4)),
                                    color: Color::rgb(0.9, 0.9, 0.2),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Armament {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                            })
                            .insert(Pulsing {
                                pulse_time: Timer::from_seconds(weapon.projectile_life_time, false),
                                damage: weapon.damage,
                            })
                            .insert(DamageAreaShape::Sphere {
                                radius: weapon.radius_of_effect,
                            })
                            .insert(RigidBody::Dynamic)
                            .insert(CollisionShape::Sphere { radius: 0.2 })
                            .insert(Velocity::from_linear(throw_dir))
                            .insert(PhysicMaterial {
                                friction: 1.0,
                                density: 10.0,
                                ..Default::default()
                            })
                            .insert(
                                CollisionLayers::none()
                                    .with_group(PhysLayer::Bullets)
                                    .with_masks(&[PhysLayer::World, PhysLayer::Enemies]),
                            );
                    }
                    AmmoType::Static => {
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.5, 0.5)),
                                    color: Color::rgb(0.2, 0.5, 0.9),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Armament {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                            })
                            .insert(Pulsing {
                                pulse_time: Timer::from_seconds(weapon.projectile_speed, true),
                                damage: weapon.damage,
                            })
                            .insert(DamageAreaShape::Sphere {
                                radius: weapon.radius_of_effect,
                            });
                    }
                }
            }
            last_shoot.time = now;
        }
    }
}

pub fn projectiles_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    mut impact_event: EventWriter<BulletImpactEvent>,
    physics_world: PhysicsWorld,
    query_player: Query<Entity, With<Player>>,
    mut query_projectiles: Query<(Entity, &Projectile, &mut Transform)>,
) {
    let player_entity = query_player.single();
    for (entity, projectile, mut transform) in query_projectiles.iter_mut() {
        let ray_cast = physics_world.ray_cast(transform.translation, projectile.direction, true);
        let bullet_travel = projectile.speed * time.delta_seconds();
        let mut surface = ImpactSurface::World;
        if let Some(collision) = ray_cast {
            // we need to manually check for collision with different entities
            // TODO refactor maybe
            if collision.entity == player_entity {
                surface = ImpactSurface::Player;
            } else if (collision.collision_point - transform.translation).length() <= bullet_travel {
                damage_event.send(DamageEvent {
                    entity: collision.entity,
                    source: DamageSource::Weapon,
                    damage: projectile.damage,
                });
                surface = ImpactSurface::Monster;
                commands.entity(entity).despawn();
            } else {
                surface = ImpactSurface::World;
            }
            impact_event.send(BulletImpactEvent{pos: collision.collision_point, surface });
            
            // debug collision point
            // commands.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::GREEN,
            //         custom_size: Some(Vec2::new(10., 10.)),
            //         ..Default::default()
            //     },
            //     transform: Transform::from_translation(collision.collision_point),
            //     ..Default::default()
            // });
        }
        transform.translation += projectile.direction * bullet_travel;
    }
}

pub fn armaments_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query_armaments: Query<(Entity, &mut Armament)>,
) {
    for (entity, mut armament) in query_armaments.iter_mut() {
        armament.life_time.tick(time.delta());
        if armament.life_time.finished() {
            commands.entity(entity).despawn();
            continue;
        }
    }
}

pub fn handle_shot_audio(audio: Res<Audio>,
                         channel: Res<GameAudioChannel>, 
                         assets: Res<GameAssets>,
                         mut ev_fired: EventReader<PlayerFiredEvent>){
    for ev in ev_fired.iter() {
        match ev.ammo_type {
            AmmoType::Projectile => {
                audio.play_in_channel(
                    assets.smg_shot_audio.clone(),
                    &channel.0,
                );
            },
            AmmoType::Throwable => {
            },
            AmmoType::Static => {
            }
        }}
}

pub fn handle_impact_audio(assets: Res<GameAssets>,
                           audio: Res<Audio>,
                           channel: Res<GameAudioChannel>,
                           mut commands: Commands,
                           mut event: EventReader<BulletImpactEvent>){
    for ev in event.iter() {
        
        let mut untyped_audio:HandleUntyped;
        let mut rng = rand::thread_rng();
        match ev.surface {
            ImpactSurface::World => {
                let max = assets.world_impacts.len();
                let idx = rng.gen_range(0..max);
                //let mut audio_vec = assets.world_impacts.clone();
                untyped_audio = assets.world_impacts[idx].clone();
            },
            ImpactSurface::Monster => {
                let max = assets.monster_impacts.len();
                let idx = rng.gen_range(0..max);
                //let mut audio_vec = assets.monster_impacts.copy();
                untyped_audio = assets.monster_impacts[idx].clone();
            },
            ImpactSurface::Player => { // TODO: change to proper sounds
                let max = assets.monster_impacts.len();
                let idx = rng.gen_range(0..max);
                //let mut audio_vec = assets.monster_impacts.copy();
                untyped_audio = assets.monster_impacts[idx].clone();
            }
        }
        
        let clip:Handle<AudioSource> = Handle::weak(untyped_audio.id);
        audio.play_in_channel(
            clip,
            &channel.0,
        );
        
        /*commands
            .spawn()
            .insert(Transform::from_translation(ev.pos))
            .insert({
                let mut a = super::SpatialAudio::default();
                a.source = clip.clone();
                a.set_looping(false);
                a.playback_rate = 1.0;
                a.attenuation = super::Attenuation::InverseSquareDistance(40.0);
                a.max_volume = 0.85;
                a
            });*/
        return; // exit after first iteration to reduce audio spam
    }
    
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerFiredEvent {
    pub entity:Entity,
    pub ammo_type:AmmoType
}

pub struct BulletImpactEvent {
    pub pos:Vec3,
    pub surface:ImpactSurface,
}

pub enum ImpactSurface {
    World,
    Monster,
    Player
}

// pub fn debug_damage_event_reader(mut events: EventReader<DamageEvent>) {
//     for e in events.iter() {
//         debug!("damage event: {:?}", e);
//     }
// }
