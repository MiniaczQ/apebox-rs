use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*, state::state::FreelyMutableState};

/// Trait for barrier markers.
pub trait BarrierMarker: Send + Sync + 'static {
    /// Type of the state this barrier uses.
    type State: States + FreelyMutableState;

    /// State in which the barrier runs.
    fn barrier_state() -> Self::State;

    /// State to go to after the barrier completes.
    fn next_state() -> Self::State;
}

/// App extension that defines barrier methods.
pub trait AppExtBarrier {
    /// Add a new barrier to the application.
    fn register_barrier<M: BarrierMarker>(&mut self) -> &mut Self;

    /// Add a system to a specific barrier.
    fn add_barrier_system<M: BarrierMarker, A>(
        &mut self,
        system: impl IntoSystemConfigs<A>,
    ) -> &mut Self;
}

impl AppExtBarrier for SubApp {
    fn register_barrier<M: BarrierMarker>(&mut self) -> &mut Self {
        self.add_systems(OnEnter(M::barrier_state()), setup_barrier::<M>);
        self.add_systems(
            Last,
            update_barrier::<M>().run_if(in_state(M::barrier_state())),
        );
        self.add_systems(OnExit(M::barrier_state()), teardown_barrier::<M>);
        self
    }

    fn add_barrier_system<M: BarrierMarker, A>(
        &mut self,
        system: impl IntoSystemConfigs<A>,
    ) -> &mut Self {
        self.add_systems(Update, system.run_if(in_state(M::barrier_state())));
        self
    }
}

impl AppExtBarrier for App {
    fn register_barrier<M: BarrierMarker>(&mut self) -> &mut Self {
        self.main_mut().register_barrier::<M>();
        self
    }

    fn add_barrier_system<M: BarrierMarker, A>(
        &mut self,
        system: impl IntoSystemConfigs<A>,
    ) -> &mut Self {
        self.main_mut().add_barrier_system::<M, _>(system);
        self
    }
}

/// Creates the barrier data resource.
fn setup_barrier<M: BarrierMarker>(mut commands: Commands) {
    commands.init_resource::<BarrierData<M>>();
}

/// Checks if all barrier systems have finished.
fn update_barrier<M: BarrierMarker>() -> impl Fn(ResMut<BarrierData<M>>, ResMut<NextState<M::State>>)
{
    move |barrier: ResMut<BarrierData<M>>, mut next: ResMut<NextState<M::State>>| {
        if barrier.completed_count >= barrier.registered_count {
            next.set(M::next_state());
        }
    }
}

/// Removes the barrier data resource.
fn teardown_barrier<M: BarrierMarker>(mut commands: Commands) {
    commands.remove_resource::<BarrierData<M>>();
}

/// Internal representation of a barrier.
#[derive(Resource)]
struct BarrierData<B: BarrierMarker> {
    marker: PhantomData<B>,
    registered_count: usize,
    completed_count: usize,
}

impl<M: BarrierMarker> Default for BarrierData<M> {
    fn default() -> Self {
        Self {
            marker: PhantomData::<M>,
            registered_count: Default::default(),
            completed_count: Default::default(),
        }
    }
}

/// [`SystemParam`] for barrier systems.
#[derive(SystemParam)]
pub struct Barrier<'w, 's, M: BarrierMarker> {
    data: ResMut<'w, BarrierData<M>>,
    is_registered: Local<'s, bool>,
    is_completed: Local<'s, bool>,
}

impl<'w, 's, M: BarrierMarker> Barrier<'w, 's, M> {
    pub fn is_completed(&mut self) -> bool {
        if !*self.is_registered {
            self.data.registered_count += 1;
            *self.is_registered = true;
        }
        *self.is_completed
    }

    pub fn complete(&mut self) {
        self.data.completed_count += 1;
        *self.is_completed = true;
    }
}
