
use std::collections::HashSet;

use bevy_derive::{Deref, DerefMut};
pub use bevy_ecs::prelude::*;
use bevy_impulse::Stream;

#[derive(Component)]
pub struct SelectorInput<T>(pub T);

#[derive(Component, Debug, PartialEq, Eq)]
pub struct Selected {
    /// This object has been selected
    pub is_selected: bool,
    /// Another object is selected but wants this entity to be highlighted
    pub support_selected: HashSet<Entity>,
}

impl Selected {
    pub fn cue(&self) -> bool {
        self.is_selected || !self.support_selected.is_empty()
    }
}

impl Default for Selected {
    fn default() -> Self {
        Self {
            is_selected: false,
            support_selected: Default::default(),
        }
    }
}

/// Component to track whether an element should be viewed in the Hovered state
/// for the selection tool.
#[derive(Component, Debug, PartialEq, Eq)]
pub struct Hovered {
    /// The cursor is hovering on this object specifically
    pub is_hovered: bool,
    /// The cursor is hovering on a different object which wants this entity
    /// to be highlighted.
    pub support_hovering: HashSet<Entity>,
}


impl Hovered {
    pub fn cue(&self) -> bool {
        self.is_hovered || !self.support_hovering.is_empty()
    }
}

impl Default for Hovered {
    fn default() -> Self {
        Self {
            is_hovered: false,
            support_hovering: Default::default(),
        }
    }
}

/// A resource to track what kind of blockers are preventing the selection
/// behavior from being active
#[derive(Resource)]
pub struct SelectionBlockers {
    /// An entity is being dragged
    pub dragging: bool,
}

impl SelectionBlockers {
    pub fn blocking(&self) -> bool {
        self.dragging
    }
}

impl Default for SelectionBlockers {
    fn default() -> Self {
        SelectionBlockers { dragging: false }
    }
}

/// Used as a resource to keep track of which entity is currently selected.
#[derive(Default, Debug, Clone, Copy, Deref, DerefMut, Resource)]
pub struct Selection(pub Option<Entity>);

/// Used as a resource to keep track of which entity is currently hovered.
#[derive(Default, Debug, Clone, Copy, Deref, DerefMut, Resource)]
pub struct Hovering(pub Option<Entity>);

/// Used as an event to command a change in the selected entity.
#[derive(Default, Debug, Clone, Copy, Deref, DerefMut, Event, Stream)]
pub struct Select(pub Option<SelectionCandidate>);


#[derive(Debug, Clone, Copy)]
pub struct SelectionCandidate {
    /// The entity that's being requested as a selection
    pub candidate: Entity,
    /// The entity was created specifically to be selected, so if it ends up
    /// going unused by the workflow then it should be despawned.
    pub provisional: bool,
}


impl Select {
    pub fn new(candidate: Option<Entity>) -> Select {
        Select(candidate.map(|c| SelectionCandidate::new(c)))
    }

    pub fn provisional(candidate: Entity) -> Select {
        Select(Some(SelectionCandidate::provisional(candidate)))
    }
}

impl SelectionCandidate {
    pub fn new(candidate: Entity) -> SelectionCandidate {
        SelectionCandidate {
            candidate,
            provisional: false,
        }
    }

    pub fn provisional(candidate: Entity) -> SelectionCandidate {
        SelectionCandidate {
            candidate,
            provisional: true,
        }
    }
}

/// Used as an event to command a change in the hovered entity.
#[derive(Default, Debug, Clone, Copy, Deref, DerefMut, Event, Stream)]
pub struct Hover(pub Option<Entity>);
