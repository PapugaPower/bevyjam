use crate::game::crosshair::Crosshair;
use crate::game::damage::{DamageAreaShape, DamageEvent, DamageSource, Pulsing, PulsingBundle};
use crate::game::enemies::Enemy;
use crate::game::phys_layers::PhysLayer;
use crate::game::player::{Player, PlayerState, PlayerShootState};
use crate::game::{GameAssets, GameAudioChannel};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioSource};
use heron::{prelude::*, rapier_plugin::PhysicsWorld};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct PlayerFiredEvent {
    pub entity: Entity,
    pub ammo_type: AmmoType,
}

#[derive(Debug, Clone, Copy)]
pub struct BulletImpactEvent {
    pub position: Vec3,
    pub direction: Vec3,
    pub surface: ImpactSurface,
}

#[derive(Debug, Clone, Copy)]
pub enum ImpactSurface {
    World,
    Monster,
    Player,
}

#[derive(Component)]
pub struct LastShootTime {
    pub time: f32,
    ///// Set to true at the start of the game so that no bullets are fired when the player is still
    ///// clicking (just after pressing the "start scenario" button).
    prevent_accidental_fire: bool,
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
    pub projectile_spawn_offset: Vec3,
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
#[derive(Component)]
pub struct WeaponMagazine {
    pub current: i32,
    pub max: i32,
    pub reload_time: f32,
    pub current_reload: f32,
}

#[derive(Component)]
pub struct SpareAmmo {
    pub current: i32,
}

#[derive(Bundle)]
pub struct WeaponryBundle {
    pub weapon: Weapon,
    pub weapon_magazine: WeaponMagazine,
    pub spare_ammo: SpareAmmo,
    pub last_shoot_time: LastShootTime,
}

impl Default for WeaponryBundle {
    fn default() -> Self {
        Self {
            weapon: Weapon {
                ammo_type: AmmoType::Projectile,
                damage: 40.0,
                fire_rate: 1.0 / 10.0,
                projectile_speed: 2000.0,
                projectile_life_time: 1.0,
                spread: 90.0,
                projectiles_per_shot: 1,
                projectile_spawn_offset: Vec3::new(33.0, -15.0, 0.0),
                radius_of_effect: 100.0,
            },
            spare_ammo: SpareAmmo { current: 40 },
            weapon_magazine: WeaponMagazine {
                current: 30,
                max: 30,
                reload_time: 1.1,
                current_reload: 0.0,
            },
            last_shoot_time: LastShootTime {
                time: 0.0,
                prevent_accidental_fire: true,
            },
        }
    }
}

pub fn player_shoot(
    mut commands: Commands,
    mut ev_fired: EventWriter<PlayerFiredEvent>,
    time: Res<Time>,
    keys: Res<Input<MouseButton>>,
    mut query_player: Query<
        (
            Entity,
            &Transform,
            &Weapon,
            &mut LastShootTime,
            &mut PlayerState,
            &mut WeaponMagazine,
        ),
        With<Player>,
    >,
    mut query_cross: Query<&Transform, With<Crosshair>>,
) {
    let (e, player_transform, weapon, mut last_shoot, mut player_state, mut mag) =
        query_player.single_mut();

    if keys.pressed(MouseButton::Left) {
        // Don't shoot if the player most likely didn't intend to shoot.
        if last_shoot.prevent_accidental_fire {
            return;
        }

        // return if out of ammo or reloading
        if mag.current < 1 || mag.current_reload > 0.0 {
            return;
        }

        // animation
        player_state.new.1 = PlayerShootState::Shooting;

        let cross_transform = query_cross.single_mut();
        let shoot_offset = player_transform
            .rotation
            .mul_vec3(weapon.projectile_spawn_offset);
        let spawn_transform = {
            let mut pt = *player_transform;
            pt.translation += shoot_offset;
            pt
        };

        let shoot_dir = (cross_transform.translation - spawn_transform.translation).normalize();


        let now = time.time_since_startup().as_secs_f32();
        if last_shoot.time + weapon.fire_rate <= now {
            last_shoot.time = now;

            // sound
            ev_fired.send(PlayerFiredEvent {
                entity: e,
                ammo_type: weapon.ammo_type,
            });

            // firing
            // if only one bullet we don't care about spread
            let (spread_edge, spread_step) = if weapon.projectiles_per_shot <= 1 {
                (0.0, 0.0)
            } else {
                (
                    -weapon.spread / 2.0,
                    weapon.spread / (weapon.projectiles_per_shot - 1) as f32,
                )
            };

            // reduce current ammo in mag
            mag.current -= 1;

            for i in 0..weapon.projectiles_per_shot {
                let shoot_dir = Quat::from_rotation_z((spread_step * i as f32).to_radians())
                    * Quat::from_rotation_z(spread_edge.to_radians() as f32)
                    * shoot_dir;
                match weapon.ammo_type {
                    AmmoType::Projectile => {
                        commands
                            .spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(20.0, 3.0)),
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
                                    custom_size: Some(Vec2::new(20.0, 20.0)),
                                    color: Color::rgb(0.9, 0.9, 0.2),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Armament {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                            })
                            .insert_bundle(PulsingBundle {
                                pulsing: Pulsing {
                                    pulse_time: Timer::from_seconds(
                                        weapon.projectile_life_time,
                                        false,
                                    ),
                                    damage: weapon.damage,
                                },
                                damage_area_shape: DamageAreaShape::Sphere {
                                    radius: weapon.radius_of_effect,
                                },
                                ..Default::default()
                            })
                            .insert(RigidBody::Dynamic)
                            .insert(CollisionShape::Sphere { radius: 10.0 })
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
                                    custom_size: Some(Vec2::new(25.0, 25.0)),
                                    color: Color::rgb(0.2, 0.5, 0.9),
                                    ..Default::default()
                                },
                                transform: spawn_transform,
                                ..Default::default()
                            })
                            .insert(Armament {
                                life_time: Timer::from_seconds(weapon.projectile_life_time, false),
                            })
                            .insert_bundle(PulsingBundle {
                                pulsing: Pulsing {
                                    pulse_time: Timer::from_seconds(weapon.projectile_speed, false),
                                    damage: weapon.damage,
                                },
                                damage_area_shape: DamageAreaShape::Sphere {
                                    radius: weapon.radius_of_effect,
                                },
                                ..Default::default()
                            });
                    }
                }
            }
        }
    } else {
        // If the button is no longer pressed, then any clicks after are most likely intended for
        // shooting
        last_shoot.prevent_accidental_fire = false;
        player_state.new.1 = PlayerShootState::Nothing;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn projectiles_controller(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_event: EventWriter<DamageEvent>,
    mut impact_event: EventWriter<BulletImpactEvent>,
    physics_world: PhysicsWorld,
    query_player: Query<Entity, With<Player>>,
    query_enemy: Query<Entity, With<Enemy>>,
    mut query_projectiles: Query<(Entity, &Projectile, &mut Transform)>,
) {
    let player_entity = query_player.single();
    for (entity, projectile, mut transform) in query_projectiles.iter_mut() {
        let ray_cast = physics_world.ray_cast(transform.translation, projectile.direction, true);
        let bullet_travel = projectile.speed * time.delta_seconds();
        if let Some(collision) = ray_cast {
            let surface = if collision.entity == player_entity {
                ImpactSurface::Player
            } else if (collision.collision_point - transform.translation).length() <= bullet_travel
                && query_enemy.get(collision.entity).is_ok()
            {
                ImpactSurface::Monster
            } else {
                ImpactSurface::World
            };
            damage_event.send(DamageEvent {
                entity: collision.entity,
                source: DamageSource::Weapon,
                damage: projectile.damage,
            });
            commands.entity(entity).despawn();

            impact_event.send(BulletImpactEvent {
                position: collision.collision_point,
                direction: projectile.direction,
                surface,
            });
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

pub fn gun_reload(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut q: Query<(&mut WeaponMagazine, &mut SpareAmmo, &mut PlayerState), With<Player>>,
) {
    let (mut mag, mut spare_ammo, mut player_state) = q.single_mut();

    // If a reload is in progress, try to complete it.
    if mag.current_reload > 0.0 {
        // animation
        player_state.new.1 = PlayerShootState::Reloading;

        mag.current_reload += time.delta_seconds();
        if mag.current_reload > mag.reload_time {
            mag.current_reload = 0.0;
            let deficit = mag.max - mag.current;
            let fill = if spare_ammo.current >= deficit {
                deficit
            } else {
                spare_ammo.current % deficit
            };
            mag.current += fill;
            spare_ammo.current -= fill;
        }
        return;
    } 

    if keys.just_pressed(KeyCode::R) {
        if spare_ammo.current < 1 || mag.current == mag.max {
            return;
        }
        mag.current_reload += time.delta_seconds(); // add delta early, acts as a flog
    }
}

pub fn handle_shot_audio(
    audio: Res<Audio>,
    channel: Res<GameAudioChannel>,
    assets: Res<GameAssets>,
    mut ev_fired: EventReader<PlayerFiredEvent>,
) {
    for ev in ev_fired.iter() {
        match ev.ammo_type {
            AmmoType::Projectile => {
                audio.play_in_channel(assets.smg_shot_audio.clone(), &channel.0);
            }
            AmmoType::Throwable => {}
            AmmoType::Static => {}
        }
    }
}

pub fn handle_impact_audio(
    assets: Res<GameAssets>,
    audio: Res<Audio>,
    channel: Res<GameAudioChannel>,
    mut commands: Commands,
    mut event: EventReader<BulletImpactEvent>,
) {
    for ev in event.iter() {
        let mut untyped_audio: HandleUntyped;
        let mut rng = rand::thread_rng();
        match ev.surface {
            ImpactSurface::World => {
                let max = assets.world_impacts.len();
                let idx = rng.gen_range(0..max);
                //let mut audio_vec = assets.world_impacts.clone();
                untyped_audio = assets.world_impacts[idx].clone();
            }
            ImpactSurface::Monster => {
                let max = assets.monster_impacts.len();
                let idx = rng.gen_range(0..max);
                //let mut audio_vec = assets.monster_impacts.copy();
                untyped_audio = assets.monster_impacts[idx].clone();
            }
            ImpactSurface::Player => {
                // TODO: change to proper sounds
                let max = assets.monster_impacts.len();
                let idx = rng.gen_range(0..max);
                //let mut audio_vec = assets.monster_impacts.copy();
                untyped_audio = assets.monster_impacts[idx].clone();
            }
        }

        let clip: Handle<AudioSource> = Handle::weak(untyped_audio.id);
        audio.play_in_channel(clip, &channel.0);

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

// pub fn debug_damage_event_reader(mut events: EventReader<DamageEvent>) {
//     for e in events.iter() {
//         debug!("damage event: {:?}", e);
//     }
// }
