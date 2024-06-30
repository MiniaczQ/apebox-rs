use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*, state::state::FreelyMutableState};

pub trait ResourceBarrierMarker: Send + Sync + 'static {
    type State: States + FreelyMutableState;

    fn loading_state() -> Self::State;
    fn next_state() -> Self::State;
}

pub trait ResourceBarrierExtApp {
    fn add_resource_barrier<M: ResourceBarrierMarker>(&mut self) -> &mut Self;
    fn add_resource_loader<M: ResourceBarrierMarker, A>(
        &mut self,
        system: impl IntoSystemConfigs<A>,
    ) -> &mut Self;
}

impl ResourceBarrierExtApp for App {
    fn add_resource_barrier<M: ResourceBarrierMarker>(&mut self) -> &mut Self {
        self.add_systems(OnEnter(M::loading_state()), add_barrier_resource::<M>);
        self.add_systems(OnExit(M::loading_state()), remove_barrier_resource::<M>);
        self.add_systems(
            Last,
            try_finish_barrier::<M>().run_if(in_state(M::loading_state())),
        );
        self
    }

    fn add_resource_loader<M: ResourceBarrierMarker, A>(
        &mut self,
        system: impl IntoSystemConfigs<A>,
    ) -> &mut Self {
        self.add_systems(Update, system.run_if(in_state(M::loading_state())));
        self
    }
}

fn add_barrier_resource<M: ResourceBarrierMarker>(mut commands: Commands) {
    commands.init_resource::<ResourceBarrierData<M>>();
}

fn remove_barrier_resource<M: ResourceBarrierMarker>(mut commands: Commands) {
    commands.remove_resource::<ResourceBarrierData<M>>();
}

fn try_finish_barrier<M: ResourceBarrierMarker>(
) -> impl Fn(ResMut<ResourceBarrierData<M>>, ResMut<NextState<M::State>>) {
    move |barrier: ResMut<ResourceBarrierData<M>>, mut next: ResMut<NextState<M::State>>| {
        if barrier.completed_loaders >= barrier.registered_loaders {
            next.set(M::next_state());
        }
    }
}

#[derive(Resource)]
struct ResourceBarrierData<M: ResourceBarrierMarker> {
    marker: PhantomData<M>,
    registered_loaders: usize,
    completed_loaders: usize,
}

impl<M: ResourceBarrierMarker> Default for ResourceBarrierData<M> {
    fn default() -> Self {
        Self {
            marker: PhantomData::<M>,
            registered_loaders: Default::default(),
            completed_loaders: Default::default(),
        }
    }
}

#[derive(SystemParam)]
pub struct ResourceBarrier<'w, 's, M: ResourceBarrierMarker> {
    awaiter: ResMut<'w, ResourceBarrierData<M>>,
    registered: Local<'s, bool>,
    completed: Local<'s, bool>,
}

impl<'w, 's, M: ResourceBarrierMarker> ResourceBarrier<'w, 's, M> {
    pub fn is_completed(&mut self) -> bool {
        if !*self.registered {
            self.awaiter.registered_loaders += 1;
            *self.registered = true;
        }
        *self.completed
    }

    pub fn complete(&mut self) {
        self.awaiter.completed_loaders += 1;
        *self.completed = true;
    }
}
