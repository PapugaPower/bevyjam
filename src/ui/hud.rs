use std::fmt::Alignment;
use bevy::{prelude::*, app::AppExit};
use bevy_kira_audio::Audio;

use iyes_bevy_util::{BevyState, despawn_with_recursive};

use crate::{GameMode, AppState, FuckStages, WeaponMagazine, SpareAmmo};
use crate::game::damage::Health;
use crate::game::player::Player;
use crate::game;

use super::{UiAudioChannel, UiAssets, UiNinepatches, ContentId, UiConfig, Btn, fill_btn, spawn_button};


#[derive(Component)]
struct HudCleanup;

pub struct HudPlugin<S: BevyState> {
    pub state: S,
}

#[derive(Component)]
pub struct AmmoCounter;

#[derive(Component)]
pub struct HealthCounter;

#[derive(Component)]
pub struct GameTimer(u64);

impl<S: BevyState> Plugin for HudPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(self.state.clone())
                .with_system(init_mainmenu)
        );
        app.add_system_set(
            SystemSet::on_exit(self.state.clone())
                .with_system(despawn_with_recursive::<HudCleanup>)
        );
        
        app.add_system_set(
            SystemSet::on_update(self.state.clone())
                .with_system(update_ammo)
                .with_system(update_health)
                .with_system(update_timer)
        );
    }
}

fn init_mainmenu(
    mut cmd: Commands,
    assets: Res<UiAssets>,
    uicfg: Res<UiConfig>,
    nps: Res<UiNinepatches>,
) {
    let layout = cmd.spawn_bundle(NodeBundle {
        color: UiColor(Color::rgba(0.5, 0.5, 0.5, 0.0)),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            margin: Rect::all(Val::Auto),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).insert(HudCleanup).id();

    let ammo_txt = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            "AMMO: ---",
            uicfg.hud_resource_counter_style_text.clone(),
            TextAlignment { vertical: VerticalAlign::Bottom, horizontal: HorizontalAlign::Right },

        ),
        ..Default::default()
    }).insert(AmmoCounter)
    .insert(HudCleanup)
    .id();

    let health_txt = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            "HEALTH: ---",
            uicfg.hud_resource_counter_style_text.clone(),
            TextAlignment { vertical: VerticalAlign::Bottom, horizontal: HorizontalAlign::Left },
        ),
        ..Default::default()
    }).insert(HealthCounter)
    .insert(HudCleanup)
    .id();

	let mut timer_style = uicfg.hud_resource_counter_style_text.clone();
	timer_style.font_size *= 3.0;
	cmd.spawn_bundle(TextBundle {
		text: Text::with_section(
			"00:00",
			timer_style,
			TextAlignment {
				vertical: VerticalAlign::Top,
				horizontal: HorizontalAlign::Center,
			}
		),
		// It's supposed to go at the center-top but other elements interfere, so just leave as is
		// for now
		/*
		style: Style {
			margin: Rect {
				top: Val::Px(2.0),
				bottom: Val::Auto,
				left: Val::Auto,
				right: Val::Auto,
			},
			..Default::default()
		},
		*/
		..Default::default()
	}).insert(GameTimer(0)).insert(HudCleanup);
}


fn update_ammo(
    mut q: Query<&mut Text, With<AmmoCounter>>,
    mut ammo_q: Query<(&WeaponMagazine, &SpareAmmo)>
) {
    let mut text = q.single_mut();
    let (mag, spare) = ammo_q.single();
    let str = format!("Ammo: {} | {}",mag.current.to_string(), spare.current.to_string());
    text.sections[0].value = str;
}

fn update_health (
    mut q: Query<&mut Text, With<HealthCounter>>,
    hp_q: Query<&Health, With<Player>>
) {
    let mut text = q.single_mut();
    let hp = hp_q.single();
    let str = format!("Health: {}", hp.current.to_string());
    text.sections[0].value = str;
}

fn update_timer(
	mut ui: Query<(&mut Text, &mut GameTimer)>,
	time: Res<game::GameTimer>,
	audio: Res<Audio>,
	channel: Res<UiAudioChannel>,
    assets: Res<UiAssets>,
) {
	let (mut text, mut gt) = ui.single_mut();
	// Directly use seconds to round up
	let t = time.0.duration().as_secs() - time.0.elapsed().as_secs();
	if gt.0 != t {
		gt.0 = t;
		text.sections[0].value = format!("{:02}:{:02}", t / 60, t % 60);

		if true {
			audio.play_in_channel(assets.snd_button_on.clone(), &channel.0);
		}
	}
}
