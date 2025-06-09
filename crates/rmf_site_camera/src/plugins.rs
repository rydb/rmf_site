use bevy_app::prelude::*;

use crate::{*, systems::*};

pub struct CameraControlsPlugin;

impl Plugin for CameraControlsPlugin {
    fn build(&self, app: &mut App) {
        //app
        //.add_systems(schedule, systems)
    }
}

pub struct CameraControlsBasePlugin;

impl Plugin for CameraControlsBasePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<CursorCommand>()
        .init_resource::<KeyboardCommand>()
        .init_resource::<HeadlightToggle>()
        .init_resource::<CameraControls>()
        .add_systems(PostStartup, init_cameras)
        .add_systems(Update, toggle_headlights.run_if(resource_changed::<HeadlightToggle>))
        .add_systems(Update, change_projection_mode.run_if(resource_changed::<ProjectionMode>))
        .add_systems(
            Update,
            (
                update_cursor_command,
                update_keyboard_command,
                camera_controls,
                update_orbit_center_marker,
            )
                .chain(),
        )
        ;
    }
}