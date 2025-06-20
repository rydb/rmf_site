use bevy_app::prelude::*;
use bevy_picking::mesh_picking::MeshPickingPlugin;

use crate::{picking::*, select::SelectionBasePlugin};

/// default picking configuration for this project.
pub struct ProjectPickingPlugin;

impl Plugin for ProjectPickingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(MeshPickingPlugin)
        .add_plugins(SelectionBasePlugin)
        .init_resource::<Picked>()
        .init_resource::<PickingBlockers>()
        
        ;
    }
}