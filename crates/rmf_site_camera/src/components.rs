use std::any::type_name;

use bevy_ecs::{component::{ComponentHooks, Immutable, Mutable, StorageType}, prelude::*};
use bevy_reflect::Reflect;
use bevy_math::prelude::*;
use tracing::warn;

use crate::{cursor::CursorCommand, keyboard::KeyboardCommand, CameraCommandType};

#[derive(Component)]
pub struct CameraSelectionMarker;

#[derive(Component)]
pub struct PerspectiveHeadlightTarget;

#[derive(Component)]
pub struct OrthographicHeadlightTarget;

#[derive(Component)]
pub struct PerspectiveCameraRoot;


#[derive(Component)]
pub struct OrthographicCameraRoot;

#[derive(PartialEq, Debug, Copy, Clone, Reflect, Resource)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

// impl Component for ProjectionMode {
//     const STORAGE_TYPE: StorageType = StorageType::Table;     

//     type Mutability = Immutable;

//     fn register_component_hooks(_hooks: &mut ComponentHooks) {
//         _hooks.on_add(|mut world, hook| {
//             let mode = match world.entity(hook.entity).get::<Self>() {
//                 Some(val) => val.clone(),
//                 None => {
//                     warn!(
//                         "could not get {:#} on: {:#}",
//                         type_name::<Self>(),
//                         hook.entity
//                     );
//                     return;
//                 }
//             };     
//             match mode {
//                 ProjectionMode::Perspective => world.commands().entity(hook.entity).insert(PerspectiveModeRequest),
//                 ProjectionMode::Orthographic => world.commands().entity(hook.entity).insert(OrthographicModeRequest),
//             };
//         });

//     }
// }


// pub struct ProjectionModeRequest(ProjectionMode);


// impl Component for ProjectionModeRequest {

// }