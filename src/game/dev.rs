use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_kira_audio::AudioSource;
use heron::prelude::*;
use iyes_bevy_util::BevyState;

use crate::editor::NoEditor;
use crate::editor::collider::EditableCollider;
use crate::game::environment::debug_environment_damage_zones;
use crate::{AppState, GameMode};
use crate::game::phys_layers::PhysLayer;
use crate::game::timer::GameTimer;
use crate::game::environment::door::debug_spawn_door;

/// This plugin should add all DevPlayground specific stuff
pub struct DevPlaygroundPlugin<S: BevyState + Copy> {
    pub loading_state: S,
    pub state: S,
}

impl<S: BevyState + Copy> Plugin for DevPlaygroundPlugin<S> {
    fn build(&self, app: &mut App) {
        // asset loader
        AssetLoader::new(self.loading_state)
            .continue_to_state(self.state)
            .with_asset_collection_file("meta/dev.assets")
            .with_collection::<DevAssets>()
            .build(app);

        // add systems to `self.state`
        app.add_system_set(
            SystemSet::on_enter(self.state)
                .with_system(spawn_dynamic_scene)
                .with_system(init_game_timer)
                .with_system(setup_scene)
                .with_system(debug_spawn_door)
                .with_system(debug_environment_damage_zones)
        );
        app.add_system_set(
            SystemSet::on_update(self.state)
				.with_system(fake_dev_hint)
        );
        app.add_system_set(
            SystemSet::on_exit(self.state)
        );
    }
}

#[derive(AssetCollection)]
pub struct DevAssets {
    #[asset(key = "enviro.map_prototype")]
    pub map_prototype: Handle<Image>,
    #[asset(key = "scene.dev")]
    pub scene: Handle<DynamicScene>,
    #[asset(key = "enviro.map_level_0")]
    pub map_level_0: Handle<Image>,
    #[asset(key = "enviro.generator")]
	pub generator: Handle<AudioSource>,
}

fn init_game_timer(
    mut commands: Commands,
) {
    let mut timer = Timer::from_seconds(99.9 * 60.0, false);
    timer.pause();
    commands.insert_resource(GameTimer(timer));
}

fn spawn_dynamic_scene(
    mut scene_spawner: ResMut<SceneSpawner>,
    assets: Res<DevAssets>,
) {
    scene_spawner.spawn_dynamic(assets.scene.clone());
}

fn setup_scene(mut commands: Commands, assets: Res<DevAssets>) {
    // enemy
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                color: Color::rgb(0.9, 0.2, 0.2),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            ..Default::default()
        })
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::Enemies)
            .with_masks(&[PhysLayer::World, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Sphere { radius: 20.0 });

    // top
    let width = 945.0;
    let height = 25.0;
    commands.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_xyz(-25.5, 355.0, 0.0))
        .insert(RigidBody::Static)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::World)
            .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(EditableCollider {
            half_extends: Vec2::new(width / 2.0, height / 2.0),
        });
    // bot
    commands.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_xyz(-25.5, -85.0, 0.0))
        .insert(RigidBody::Static)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::World)
            .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(EditableCollider {
            half_extends: Vec2::new(width / 2.0, height / 2.0),
        });
    // letf top
    let width = 20.0;
    let height = 130.0;
    commands.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_xyz(-488.0, 280.0, 0.0))
        .insert(RigidBody::Static)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::World)
            .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(EditableCollider {
            half_extends: Vec2::new(width / 2.0, height / 2.0),
        });
    // letf bot
    commands.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_xyz(-488.0, -7.5, 0.0))
        .insert(RigidBody::Static)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::World)
            .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(EditableCollider {
            half_extends: Vec2::new(width / 2.0, height / 2.0),
        });
    // right top
    let width = 20.0;
    let height = 130.0;
    commands.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_xyz(437.0, 280.0, 0.0))
        .insert(RigidBody::Static)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::World)
            .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(EditableCollider {
            half_extends: Vec2::new(width / 2.0, height / 2.0),
        });
    // right bot
    commands.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_xyz(437.0, -7.5, 0.0))
        .insert(RigidBody::Static)
        .insert(CollisionLayers::none()
            .with_group(PhysLayer::World)
            .with_masks(&[PhysLayer::Player, PhysLayer::Enemies, PhysLayer::Bullets]))
        .insert(EditableCollider {
            half_extends: Vec2::new(width / 2.0, height / 2.0),
        });

    // background
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        global_transform: Default::default(),
        texture: assets.map_prototype.clone(),
        visibility: Default::default(),
    }).insert(NoEditor);

    // big level
    let mut level_tform = Transform::from_xyz(3596.0, 0.0, -1.5);
    level_tform.scale = Vec3::new(1.25, 1.25,1.0);
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(8192.0, 8192.0)),
            ..Default::default()
        },
        transform: level_tform,
        global_transform: Default::default(),
        texture: assets.map_level_0.clone(),
        visibility: Default::default(),
    });
    return;
	// "generator"
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(50.0, 80.0)),
			color: Color::RED,
            ..Default::default()
        },
        transform: Transform::from_xyz(-900.0, 550.0, 0.08),
		..Default::default()
    }).insert({
		let mut a = super::SpatialAudio::default();
		a.source = assets.generator.clone();
		a.set_looping(true);
		a.playback_rate = 0.07;
		a.attenuation = super::Attenuation::InverseSquareDistance(10.0);
		a
	});

	// "generator" that disappears
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(50.0, 80.0)),
			color: Color::RED,
            ..Default::default()
        },
        transform: Transform::from_xyz(-90.0, 50.0, 0.08),
		..Default::default()
    }).insert({
		let mut a = super::SpatialAudio::default();
		a.source = assets.generator.clone();
		// FIXME multiple looping audio sources breaks things.
		//a.set_looping(true);
		a.playback_rate = 0.4;
		a.attenuation = super::Attenuation::InverseSquareDistance(40.0);
		a
	});
}

fn fake_dev_hint(mut hints: Query<&mut super::Hints, Added<super::Hints>>) {
	let _ = hints.get_single_mut().map(|mut h| h.push("this is the dev scene"));
}
