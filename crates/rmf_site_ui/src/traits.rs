use bevy_ecs::{prelude::*, system::{SystemParam, SystemState}};
use bevy_egui::egui::Ui;

use crate::*;

pub type ShowResult<T = ()> = Result<T, ShowError>;

/// Do not implement this widget directly. Instead create a struct that derives
/// [`SystemParam`] and then implement [`WidgetSystem`] for that struct.
pub trait ExecuteWidget<Input, Output> {
    fn show(&mut self, input: Input, ui: &mut Ui, world: &mut World) -> Output;
}

/// Trait implemented on [`World`] to let it render child widgets. Note that
/// this is not able to render widgets recursively, so you should make sure not
/// to have circular dependencies in your widget structure.
pub trait TryShowWidgetWorld {
    /// Try to show a widget that has `()` for input and output belonging to the
    /// specified entity.
    fn try_show(&mut self, entity: Entity, ui: &mut Ui) -> ShowResult<()> {
        self.try_show_out(entity, (), ui)
    }

    /// Same as [`Self::try_show`] but takes an input that will be fed to the widget.
    fn try_show_in<Input>(&mut self, entity: Entity, input: Input, ui: &mut Ui) -> ShowResult<()>
    where
        Input: 'static + Send + Sync,
    {
        self.try_show_out(entity, input, ui)
    }

    /// Same as [`Self::try_show`] but takes an input for the widget and provides
    /// an output from the widget.
    fn try_show_out<Output, Input>(
        &mut self,
        entity: Entity,
        input: Input,
        ui: &mut Ui,
    ) -> ShowResult<Output>
    where
        Input: 'static + Send + Sync,
        Output: 'static + Send + Sync;
}

impl TryShowWidgetWorld for World {
    fn try_show_out<Output, Input>(
        &mut self,
        entity: Entity,
        input: Input,
        ui: &mut Ui,
    ) -> ShowResult<Output>
    where
        Input: 'static + Send + Sync,
        Output: 'static + Send + Sync,
    {
        let Ok(mut entity_mut) = self.get_entity_mut(entity) else {
            return Err(ShowError::EntityMissing);
        };
        entity_mut.try_show_out(input, ui)
    }
}

/// Same as [`TryShowWidgetWorld`] but is implemented for [`EntityWorldMut`] so
/// you do not need to specify the target entity.
pub trait TryShowWidgetEntity {
    /// Try to show a widget that has `()` for input and output
    fn try_show(&mut self, ui: &mut Ui) -> ShowResult<()> {
        self.try_show_out((), ui)
    }

    fn try_show_in<Input>(&mut self, input: Input, ui: &mut Ui) -> ShowResult<()>
    where
        Input: 'static + Send + Sync,
    {
        self.try_show_out(input, ui)
    }

    fn try_show_out<Output, Input>(&mut self, input: Input, ui: &mut Ui) -> ShowResult<Output>
    where
        Input: 'static + Send + Sync,
        Output: 'static + Send + Sync;
}

impl<'w> TryShowWidgetEntity for EntityWorldMut<'w> {
    fn try_show_out<Output, Input>(&mut self, input: Input, ui: &mut Ui) -> ShowResult<Output>
    where
        Input: 'static + Send + Sync,
        Output: 'static + Send + Sync,
    {
        let Some(mut widget) = self.get_mut::<Widget<Input, Output>>() else {
            return Err(ShowError::WidgetMissing);
        };

        let Some(mut inner) = widget.inner.take() else {
            return Err(ShowError::Recursion);
        };

        let output = self.world_scope(|world| inner.show(input, ui, world));

        if let Some(mut widget) = self.get_mut::<Widget<Input, Output>>() {
            widget.inner = Some(inner);
        }

        Ok(output)
    }
}

/// Implement this on a [`SystemParam`] struct to make it a widget that can be
/// plugged into the site editor UI.
///
/// See documentation of [`PropertiesTilePlugin`] or [`InspectionPlugin`] to see
/// examples of using this.
pub trait WidgetSystem<Input = (), Output = ()>: SystemParam {
    fn show(input: Input, ui: &mut Ui, state: &mut SystemState<Self>, world: &mut World) -> Output;
}

/// This is a marker trait to indicate that the system state of a widget can be
/// safely shared across multiple renders of the widget. For example, the system
/// parameters do not use the [`Changed`] filter. It is the responsibility of
/// the user to ensure that sharing this widget will not have any bad side
/// effects.
///
/// [`ShareableWidget`]s can be used by the [`ShowSharedWidget`] trait which is
/// implemented for the [`World`] struct.
pub trait ShareableWidget {}

/// This gives a convenient function for rendering a widget using a world.
pub trait ShowSharedWidget {
    fn show<W, Output, Input>(&mut self, input: Input, ui: &mut Ui) -> Output
    where
        W: ShareableWidget + WidgetSystem<Input, Output> + 'static;
}

impl ShowSharedWidget for World {
    fn show<W, Output, Input>(&mut self, input: Input, ui: &mut Ui) -> Output
    where
        W: ShareableWidget + WidgetSystem<Input, Output> + 'static,
    {
        if !self.contains_resource::<SharedWidget<W>>() {
            let widget = SharedWidget::<W> {
                state: SystemState::new(self),
            };
            self.insert_resource(widget);
        }

        self.resource_scope::<SharedWidget<W>, Output>(|world, mut widget| {
            let u = W::show(input, ui, &mut widget.state, world);
            widget.state.apply(world);
            u
        })
    }
}