//! Data describing all the ninepatches
//!
//! FIXME: all of this should be moved into asset files
//! (need to figure out some bevy bugs with reflection first)

use bevy::prelude::*;
use bevy_ninepatch::*;

use super::ContentId;

pub(super) fn setup_ninepatches(
    mut assets: ResMut<Assets<NinePatchBuilder<ContentId>>>,
    mut cmd: Commands,
) {
    let npmeta_button = assets.add(NinePatchBuilder::by_margins_with_content(
        5,
        4,
        4,
        4,
        ContentId::ButtonText
    ));

    cmd.insert_resource(super::UiNinepatches {
        npmeta_button,
    });
}
