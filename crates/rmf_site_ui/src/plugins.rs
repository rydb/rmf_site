
use bevy_ecs::prelude::*;
use bevy_app::prelude::*;

use crate::{panel::PanelSide, properties_panel::PropertiesPanelPlugin};


#[derive(Default)]
pub struct StandardPropertiesPanelPluginBase;

impl Plugin for StandardPropertiesPanelPluginBase {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PropertiesPanelPlugin::new(PanelSide::Right),
        ));
    }
}