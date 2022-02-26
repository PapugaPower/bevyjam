use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

pub mod mainmenu;

pub struct UiSetupPlugin;

impl Plugin for UiSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_ui_camera);
        app.add_system(button_interact_visual);
    }
}

#[derive(AssetCollection)]
pub struct UiAssets {
    #[asset(key = "ui.font_menu_bold")]
    font_menu_bold: Handle<Font>,
    #[asset(key = "ui.font_menu_regular")]
    font_menu_regular: Handle<Font>,
}

fn button_connector<B: Component + Clone>(
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
    mut query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *color = UiColor(Color::rgb(0.75, 0.75, 0.75));
            }
            Interaction::Hovered => {
                *color = UiColor(Color::rgb(0.8, 0.8, 0.8));
            },
            Interaction::None => {
                *color = UiColor(Color::rgb(1.0, 1.0, 1.0));
            },
        }
    }
}

fn init_ui_camera(mut cmd: Commands) {
    cmd.spawn_bundle(UiCameraBundle::default());
}
