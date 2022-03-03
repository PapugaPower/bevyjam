use std::fmt::Alignment;
use bevy::{prelude::*, app::AppExit};

use iyes_bevy_util::{BevyState, despawn_with_recursive};

use crate::{GameMode, AppState, FuckStages, WeaponMagazine, SpareAmmo};
use crate::game::damage::Health;
use crate::game::player::Player;

use super::{UiAssets, UiNinepatches, ContentId, UiConfig, Btn, fill_btn, spawn_button};


#[derive(Component)]
struct HudCleanup;

pub struct HudPlugin<S: BevyState> {
    pub state: S,
}

#[derive(Component)]
pub struct AmmoCounter;

#[derive(Component)]
pub struct HealthCounter;

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
        .id();

    let health_txt = cmd.spawn_bundle(TextBundle {
        text: Text::with_section(
            "HEALTH: ---",
            uicfg.hud_resource_counter_style_text.clone(),
            TextAlignment { vertical: VerticalAlign::Bottom, horizontal: HorizontalAlign::Left },
        ),
        ..Default::default()
    }).insert(HealthCounter)
        .id();
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


