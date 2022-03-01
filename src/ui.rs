use bevy::{prelude::*, reflect::TypeUuid};
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::*;
use bevy_ninepatch::*;

use crate::{AppState, FuckStages};

mod ninepatches;

pub mod gameover;
pub mod mainmenu;

pub struct UiSetupPlugin;

impl Plugin for UiSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NinePatchPlugin::<ContentId>::default());
        app.insert_resource(UiAudioChannel(AudioChannel::new("ui".into())));
        // FIXME: use actual deserializable assets
        app.add_startup_system(crate::ui::ninepatches::setup_ninepatches);
        //app.add_plugin(bevy_asset_ron::RonAssetPlugin::<Ninepatches>::new(&["np"]));
        app.add_startup_system(init_ui_camera);
        app.add_system_to_stage(FuckStages::Pre, button_interact_visual);
        app.add_system(button_sounds);
        app.add_system_set(
            SystemSet::on_exit(AppState::MainAssetLoading)
                .with_system(init_uicfg)
        );

        app.add_plugin(mainmenu::MainMenuPlugin);
        app.add_plugin(gameover::GameoverUiPlugin);
    }
}

#[derive(AssetCollection)]
pub struct UiAssets {
    #[asset(key = "ui.font_menu_bold")]
    font_menu_bold: Handle<Font>,
    #[asset(key = "ui.font_menu_regular")]
    pub font_menu_regular: Handle<Font>,
    #[asset(key = "ui.npimg.button")]
    npimg_button: Handle<Image>,
    #[asset(key = "ui.npimg.button.hovered")]
    npimg_button_hovered: Handle<Image>,
    #[asset(key = "ui.npimg.button.clicked")]
    npimg_button_clicked: Handle<Image>,
    #[asset(key = "ui.snd.button.on")]
    snd_button_on: Handle<AudioSource>,
    #[asset(key = "ui.snd.button.off")]
    snd_button_off: Handle<AudioSource>,
    #[asset(key = "ui.snd.button.hover")]
    snd_button_hover: Handle<AudioSource>,
}

struct UiAudioChannel(AudioChannel);

/// FIXME: move this into UiAssets
struct UiNinepatches {
    npmeta_button: Handle<NinePatchBuilder<ContentId>>,
}

pub struct UiConfig {
    pub btn_style: Style,
    pub btn_style_text: TextStyle,
    pub heading_style_text: TextStyle,
    pub dialog_style_text: TextStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
enum ContentId {
    ButtonText,
}

#[derive(Component)]
struct GameBtn;

pub trait Btn: Component + Clone + Copy {
    fn fill_content(&self) -> String;
    fn register_handler(sset: SystemSet) -> SystemSet;
}

pub fn button_connector<B: Component + Clone>(
    mut query: Query<
        (&Interaction, &B),
        (Changed<Interaction>, With<Button>),
    >,
) -> Option<B> {
    let mut clicked = None;

    for (interaction, val) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                clicked = Some(val.clone());
            }
            _ => (),
        }
    }

    clicked
}

fn button_interact_visual(
    mut cmd: Commands,
    uicfg: Option<Res<UiAssets>>,
    mut query: Query<
        (Entity, &Interaction, &mut NinePatchData<ContentId>),
        (/*Changed<Interaction>, */With<GameBtn>),
    >,
) {
    // PERF: change detection not working with ninepatch
    if let Some(uicfg) = uicfg {
        for (e, interaction, mut npdata) in query.iter_mut() {
            match interaction {
                Interaction::None => {
                    npdata.texture = uicfg.npimg_button.clone();
                },
                Interaction::Hovered => {
                    npdata.texture = uicfg.npimg_button_hovered.clone();
                },
                Interaction::Clicked => {
                    npdata.texture = uicfg.npimg_button_clicked.clone();
                }
            }
            npdata.loaded = false;
            cmd.entity(e).despawn_descendants();
        }
    }
}

fn button_sounds(
    audio: Res<Audio>,
    channel: Res<UiAudioChannel>,
    assets: Option<Res<UiAssets>>,
    query: Query<
        &Interaction,
        (Changed<Interaction>, With<GameBtn>),
    >,
) {
    if let Some(assets) = assets {
        for interaction in query.iter() {
            match interaction {
                Interaction::None => {
                    audio.play_in_channel(
                        assets.snd_button_hover.clone(),
                        &channel.0
                    );
                },
                Interaction::Hovered => {
                    audio.play_in_channel(
                        assets.snd_button_on.clone(),
                        &channel.0
                    );
                },
                Interaction::Clicked => {
                    audio.play_in_channel(
                        assets.snd_button_off.clone(),
                        &channel.0
                    );
                }
            }
        }
    }
}

fn init_ui_camera(mut cmd: Commands) {
    cmd.spawn_bundle(UiCameraBundle::default());
}

fn init_uicfg(
    mut cmd: Commands,
    assets: Res<UiAssets>,
) {
    cmd.insert_resource(UiConfig {
        btn_style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: Rect::all(Val::Px(4.0)),
            margin: Rect::all(Val::Px(4.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        btn_style_text: TextStyle {
            font: assets.font_menu_regular.clone(),
            font_size: 24.0,
            color: Color::BLACK,
        },
        heading_style_text: TextStyle {
            font: assets.font_menu_bold.clone(),
            font_size: 24.0,
            color: Color::BLACK,
        },
        dialog_style_text: TextStyle {
            font: assets.font_menu_regular.clone(),
            font_size: 20.0,
            color: Color::BLACK,
        },
    });
}

fn fill_btn<B: Btn>(
    mut cmd: Commands,
    mut patch_content: Query<(Entity, &mut NinePatchContent<ContentId>, &mut Style)>,
    ui_element_query: Query<&B>,
    uicfg: Option<Res<UiConfig>>,
) {
    if let Some(uicfg) = uicfg {
        for (e, mut content, mut style) in patch_content.iter_mut() {
            if !content.loaded {
                if let Ok(b) = ui_element_query.get(content.parent) {
                    let s = b.fill_content();
                    let child = cmd.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            s,
                            uicfg.btn_style_text.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    }).id();
                    cmd.entity(e).push_children(&[child]);
                    style.justify_content = JustifyContent::Center;
                    style.align_items = AlignItems::Center;
                    content.loaded = true;
                }
            }
        }
    }
}

fn spawn_button<B: Btn>(
    commands: &mut Commands,
    style: Style,
    img: Handle<Image>,
    np: Handle<NinePatchBuilder<ContentId>>,
    parent: Entity,
    btn: B,
) {
    let child = commands.spawn_bundle(
        ButtonBundle::default()
    ).insert_bundle(
        NinePatchBundle {
            style: style,
            nine_patch_data: NinePatchData {
                nine_patch: np,
                texture: img,
                ..Default::default()
            },
            ..Default::default()
        }
    )
    .insert(GameBtn)
    .insert(btn)
    .id();

    commands.entity(parent).push_children(&[child]);
}
