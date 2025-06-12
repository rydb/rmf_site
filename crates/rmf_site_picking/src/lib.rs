use bevy_impulse::{Service, StreamOf};
use bevy_input::keyboard::KeyCode;
use bevy_state::state::States;

pub mod visual_cue;
pub mod category_visibility;
pub mod picking;
pub mod select;
pub mod plugins;

use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States)]
pub enum InteractionState {
    Enable,
    #[default]
    Disable,
}

/// This component is put on entities with meshes to mark them as items that can
/// be interacted with to
#[derive(Component, Clone, Copy, Debug)]
pub struct Selectable {
    /// Toggle whether this entity is selectable
    pub is_selectable: bool,
    /// What element of the site is being selected when this entity is clicked
    pub element: Entity,
}

impl Selectable {
    pub fn new(element: Entity) -> Self {
        Selectable {
            is_selectable: true,
            element,
        }
    }
}

/// (rydb) #TODO: move this into an rmf_site_controls crate. This is only here for the rmf_site_picking peel-off
#[derive(Resource)]
pub struct KeyboardServices {
    pub keyboard_just_pressed: Service<(), (), StreamOf<KeyCode>>,
}