use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;
use bevy_pbr::StandardMaterial;
use bevy_render::mesh::Mesh;

#[derive(Resource)]
pub struct CameraOrbitMat(pub Handle<StandardMaterial>);

#[derive(Resource)]
pub struct CameraControlMesh(pub Handle<Mesh>);