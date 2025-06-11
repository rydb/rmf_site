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