use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use std::marker::PhantomData;

#[derive(Default)]
pub struct IdentityTransitionsPlugin<S: States>(PhantomData<S>);
impl<S: States> Plugin for IdentityTransitionsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            StateTransition,
            last_transition::<S>
                .pipe(run_reenter::<S>)
                .in_set(EnterSchedules::<S>::default()),
        )
        .add_systems(
            StateTransition,
            last_transition::<S>
                .pipe(run_reexit::<S>)
                .in_set(ExitSchedules::<S>::default()),
        );
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnReenter<S: States>(pub S);

fn run_reenter<S: States>(transition: In<Option<StateTransitionEvent<S>>>, world: &mut World) {
    let Some(transition) = transition.0 else {
        return;
    };
    let Some(entered) = transition.entered else {
        return;
    };
    let _ = world.try_run_schedule(OnReenter(entered));
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OnReexit<S: States>(pub S);

fn run_reexit<S: States>(transition: In<Option<StateTransitionEvent<S>>>, world: &mut World) {
    let Some(transition) = transition.0 else {
        return;
    };
    let Some(exited) = transition.exited else {
        return;
    };
    let _ = world.try_run_schedule(OnReexit(exited));
}
