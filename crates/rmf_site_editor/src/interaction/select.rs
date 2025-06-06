/*
 * Copyright (C) 2022 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

use crate::{
    interaction::*,
    keyboard::KeyboardServices,
    site::{drawing_editor::CurrentEditDrawing, Anchor, AnchorBundle, DrawingMarker},
    CurrentWorkspace,
};
use anyhow::{anyhow, Error as Anyhow};
use bevy::{
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, StaticSystemParam, SystemParam},
    },
    math::Ray3d,
    picking::pointer::{PointerId, PointerInteraction},
    prelude::*,
};
use bevy_impulse::*;
use rmf_site_format::{
    Category, Door, Edge, Lane, LiftProperties, Measurement, NameOfSite, Pending, PixelsPerMeter,
    Pose, Side, Wall,
};
use std::{borrow::Borrow, collections::HashSet, error::Error};

pub mod create_edges;
use create_edges::*;

pub mod create_path;
use create_path::*;

pub mod create_point;
use create_point::*;

pub mod place_object;
pub use place_object::*;

pub mod place_object_2d;
pub use place_object_2d::*;

pub mod replace_point;
use replace_point::*;

pub mod replace_side;
use replace_side::*;

pub mod select_anchor;
pub use select_anchor::*;

pub const SELECT_ANCHOR_MODE_LABEL: &'static str = "select_anchor";

/// This allows an [`App`] to spawn a service that can stream Hover and
/// Select events that are managed by a filter. This can only be used with
/// [`App`] because some of the internal services are continuous, so they need
/// to be added to the schedule.
pub trait SpawnSelectionServiceExt {
    fn spawn_selection_service<F: SystemParam + 'static>(
        &mut self,
    ) -> Service<(), (), (Hover, Select)>
    where
        for<'w, 's> F::Item<'w, 's>: SelectionFilter;
}

impl SpawnSelectionServiceExt for App {
    fn spawn_selection_service<F: SystemParam + 'static>(
        &mut self,
    ) -> Service<(), (), (Hover, Select)>
    where
        for<'w, 's> F::Item<'w, 's>: SelectionFilter,
    {
        let picking_service = self.spawn_continuous_service(
            Update,
            picking_service::<F>.configure(|config: ScheduleConfigs<ScheduleSystem>| {
                config.in_set(SelectionServiceStages::Pick)
            }),
        );

        let hover_service = self.spawn_continuous_service(
            Update,
            hover_service::<F>.configure(|config: ScheduleConfigs<ScheduleSystem>| {
                config.in_set(SelectionServiceStages::Hover)
            }),
        );

        let select_service = self.spawn_continuous_service(
            Update,
            select_service::<F>.configure(|config: ScheduleConfigs<ScheduleSystem>| {
                config.in_set(SelectionServiceStages::Select)
            }),
        );

        self.world_mut()
            .spawn_workflow::<_, _, (Hover, Select), _>(|scope, builder| {
                let hover = builder.create_node(hover_service);
                builder.connect(hover.streams, scope.streams.0);
                builder.connect(hover.output, scope.terminate);

                let select = builder.create_node(select_service);
                builder.connect(select.streams, scope.streams.1);
                builder.connect(select.output, scope.terminate);

                // Activate all the services at the start
                scope.input.chain(builder).fork_clone((
                    |chain: Chain<_>| {
                        chain
                            .then(refresh_picked.into_blocking_callback())
                            .then(picking_service)
                            .connect(scope.terminate)
                    },
                    |chain: Chain<_>| chain.connect(hover.input),
                    |chain: Chain<_>| chain.connect(select.input),
                ));

                // This is just a dummy buffer to let us have a cleanup workflow
                let buffer = builder.create_buffer::<()>(BufferSettings::keep_all());
                builder.on_cleanup(buffer, |scope, builder| {
                    scope
                        .input
                        .chain(builder)
                        .trigger()
                        .then(clear_hover_select.into_blocking_callback())
                        .connect(scope.terminate);
                });
            })
    }
}

#[derive(Default)]
pub struct SelectionPlugin {}

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((
            InspectorServicePlugin::default(),
            AnchorSelectionPlugin::default(),
            ObjectPlacementPlugin::default(),
            SelectionBasePlugin,
        ));
    }
}

#[derive(Default)]
pub struct InspectorServicePlugin {}

impl Plugin for InspectorServicePlugin {
    fn build(&self, app: &mut App) {
        let inspector_select_service = app.spawn_selection_service::<InspectorFilter>();
        let inspector_cursor_transform = app.spawn_continuous_service(
            Update,
            inspector_cursor_transform.configure(|config: ScheduleConfigs<ScheduleSystem>| {
                config.in_set(SelectionServiceStages::Pick)
            }),
        );
        let selection_update = app.spawn_service(selection_update);
        let keyboard_just_pressed = app
            .world()
            .resource::<KeyboardServices>()
            .keyboard_just_pressed;

        let inspector_service = app.world_mut().spawn_workflow(|scope, builder| {
            let fork_input = scope.input.fork_clone(builder);
            fork_input
                .clone_chain(builder)
                .then(inspector_cursor_transform)
                .unused();
            fork_input
                .clone_chain(builder)
                .then_node(keyboard_just_pressed)
                .streams
                .chain(builder)
                .inner()
                .then(deselect_on_esc.into_blocking_callback())
                .unused();
            let selection = fork_input
                .clone_chain(builder)
                .then_node(inspector_select_service);
            selection
                .streams
                .1
                .chain(builder)
                .then(selection_update)
                .unused();
            builder.connect(selection.output, scope.terminate);
        });

        app.world_mut().insert_resource(InspectorService {
            inspector_service,
            inspector_select_service,
            inspector_cursor_transform,
            selection_update,
        });
    }
}


#[derive(SystemParam)]
pub struct InspectorFilter<'w, 's> {
    selectables: Query<'w, 's, &'static Selectable, (Without<Preview>, Without<Pending>)>,
}

impl<'w, 's> SelectionFilter for InspectorFilter<'w, 's> {
    fn filter_pick(&mut self, select: Entity) -> Option<Entity> {
        self.selectables
            .get(select)
            .ok()
            .map(|selectable| selectable.element)
    }
    fn filter_select(&mut self, target: Entity) -> Option<Entity> {
        Some(target)
    }
    fn on_click(&mut self, hovered: Hover) -> Option<Select> {
        Some(Select::new(hovered.0))
    }
}

/// Update the virtual cursor (dagger and circle) transform while in inspector mode
pub fn inspector_cursor_transform(
    In(ContinuousService { key }): ContinuousServiceInput<(), ()>,
    orders: ContinuousQuery<(), ()>,
    cursor: Res<Cursor>,
    camera_controls: Res<CameraControls>,
    pointers: Query<(&PointerId, &PointerInteraction)>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(orders) = orders.view(&key) else {
        return;
    };

    if orders.is_empty() {
        return;
    }

    let Some((_, interactions)) = pointers.single().ok() else {
        return;
    };
    let active_camera = camera_controls.active_camera();
    let Some((position, normal)) = interactions
        .iter()
        .find(|(_, hit_data)| hit_data.camera == active_camera)
        .and_then(|(_, hit_data)| {
            hit_data
                .position
                .zip(hit_data.normal.and_then(|n| Dir3::new(n).ok()))
        })
    else {
        return;
    };

    let mut transform = match transforms.get_mut(cursor.frame) {
        Ok(transform) => transform,
        Err(_) => {
            return;
        }
    };

    let ray = Ray3d::new(position, normal);
    *transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_rotation_arc(Vec3::new(0., 0., 1.), *ray.direction),
        ray.origin,
    ));
}
