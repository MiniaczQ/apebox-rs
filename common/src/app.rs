use bevy::prelude::*;

use crate::transitions::OnReenter;

pub trait AppExt {
    fn add_statebound<S: States, A, B, C>(
        &mut self,
        state: S,
        setup: impl IntoSystemConfigs<A>,
        teardown: impl IntoSystemConfigs<C>,
        update: impl IntoSystemConfigs<B>,
    );

    fn add_reentrant_statebound<S: States, A, B, C>(
        &mut self,
        state: S,
        setup: impl IntoSystemConfigs<A>,
        teardown: impl IntoSystemConfigs<C>,
        update: impl IntoSystemConfigs<B>,
    );
}

impl AppExt for App {
    fn add_statebound<S: States, A, B, C>(
        &mut self,
        state: S,
        setup: impl IntoSystemConfigs<A>,
        teardown: impl IntoSystemConfigs<C>,
        update: impl IntoSystemConfigs<B>,
    ) {
        self.add_systems(OnEnter(state.clone()), setup);
        self.add_systems(OnExit(state.clone()), teardown);
        self.add_systems(Update, update.run_if(in_state(state.clone())));
    }

    fn add_reentrant_statebound<S: States, A, B, C>(
        &mut self,
        state: S,
        setup: impl IntoSystemConfigs<A>,
        teardown: impl IntoSystemConfigs<C>,
        update: impl IntoSystemConfigs<B>,
    ) {
        self.add_systems(OnReenter(state.clone()), setup);
        self.add_systems(OnExit(state.clone()), teardown);
        self.add_systems(Update, update.run_if(in_state(state.clone())));
    }
}
