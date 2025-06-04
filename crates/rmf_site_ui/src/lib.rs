use crate::traits::ExecuteWidget;
use bevy_ecs::{prelude::*, system::{SystemParam, SystemState}};

pub mod panel;
pub mod traits;
pub mod plugins;
pub mod panel_of_tiles;
pub mod header_panel;
pub mod properties_panel;
pub mod menu_bar;
pub mod canvas_tooltips;

use bevy_egui::egui::Ui;
use traits::*;

/// Errors that can happen while attempting to show a widget.
#[derive(Debug)]
pub enum ShowError {
    /// The entity whose widget you are trying to show is missing from the world
    EntityMissing,
    /// There is no [`Widget`] component for the entity
    WidgetMissing,
    /// The entity has a [`Widget`] component, but the widget is already in use,
    /// which implies that we are trying to render the widget recursively, and
    /// that is not supported due to soundness issues.
    Recursion,
}

/// This component should be given to an entity that needs to be rendered as a
/// nested widget in the UI.
///
/// For standard types of widgets you don't need to create this component yourself,
/// instead use one of the generic convenience plugins:
/// - [`InspectionPlugin`]
/// - [`PropertiesTilePlugin`]
#[derive(Component)]
pub struct Widget<Input = (), Output = ()> {
    inner: Option<Box<dyn ExecuteWidget<Input, Output> + 'static + Send + Sync>>,
    _ignore: std::marker::PhantomData<(Input, Output)>,
}

impl<Input, Output> Widget<Input, Output>
where
    Input: 'static + Send + Sync,
    Output: 'static + Send + Sync,
{
    pub fn new<W>(world: &mut World) -> Self
    where
        W: WidgetSystem<Input, Output> + 'static + Send + Sync,
    {
        let inner = InnerWidget::<Input, Output, W> {
            state: SystemState::new(world),
            _ignore: Default::default(),
        };

        Self {
            inner: Some(Box::new(inner)),
            _ignore: Default::default(),
        }
    }
}

struct InnerWidget<Input, Output, W: WidgetSystem<Input, Output> + 'static> {
    state: SystemState<W>,
    _ignore: std::marker::PhantomData<(Input, Output)>,
}

impl<Input, Output, W> ExecuteWidget<Input, Output> for InnerWidget<Input, Output, W>
where
    W: WidgetSystem<Input, Output>,
{
    fn show(&mut self, input: Input, ui: &mut Ui, world: &mut World) -> Output {
        let u = W::show(input, ui, &mut self.state, world);
        self.state.apply(world);
        u
    }
}

/// This set is for systems that impact rendering the UI using egui. The
/// [`UserCameraDisplay`] resource waits until after this set is finished before
/// computing the user camera area.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct RenderUiSet;

/// A resource to store a widget so that it can be reused multiple times in one
/// render pass.
#[derive(Resource)]
pub struct SharedWidget<W: SystemParam + ShareableWidget + 'static> {
    pub state: SystemState<W>,
}