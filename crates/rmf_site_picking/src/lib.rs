use bevy_state::state::States;

pub mod visual_cue;
pub mod category_visibility;


#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, States)]
pub enum InteractionState {
    Enable,
    #[default]
    Disable,
}
