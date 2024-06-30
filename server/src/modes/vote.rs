use std::time::Duration;

use bevy::prelude::*;
use bevy::{ecs::prelude::Resource, utils::hashbrown::HashSet};
use bevy_quinnet::{server::QuinnetServer, shared::ClientId};
use common::{
    app::AppExt,
    game::{Combination, Drawing, Index, Prompt, Vote, VotedOut},
    protocol::ServerMsgRoot,
};
use rand::prelude::SliceRandom;

use crate::{
    game::{GameConfig, VoteConfig},
    networking::Submission,
    states::VoteState,
    GameSystemOdering, ProgressGame, Users,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Submission<Vote>>();
        app.add_reentrant_statebound(
            VoteState,
            setup,
            teardown,
            update.in_set(GameSystemOdering::StateLogic),
        );
    }
}

#[derive(Resource, Debug)]
pub struct Context {
    started: Duration,
    submited: HashSet<ClientId>,
    combination1: (Entity, Index),
    combination1_votes: usize,
    combination2: (Entity, Index),
    combination2_votes: usize,
}

fn setup(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<VoteConfig>,
    combinations: Query<
        (Entity, &Index, &Combination),
        (
            With<Combination>,
            Without<VotedOut>,
            Without<Drawing>,
            Without<Prompt>,
        ),
    >,
    drawings: Query<(&Index, &Drawing), (Without<Combination>, Without<Prompt>)>,
    prompts: Query<(&Index, &Prompt), (Without<Combination>, Without<Drawing>)>,
) {
    info!("Setup vote");
    let mut rng = rand::thread_rng();
    let mut combinations = combinations.iter().collect::<Vec<_>>();
    combinations.shuffle(&mut rng);
    let combination1 = combinations[0];
    let combination2 = combinations[1];
    commands.insert_resource(Context {
        started: time.elapsed(),
        submited: HashSet::new(),
        combination1: (combination1.0, *combination1.1),
        combination2: (combination2.0, *combination2.1),
        combination1_votes: 0,
        combination2_votes: 0,
    });
    let drawing1 = drawings
        .iter()
        .find(|d| *d.0 == combination1.2.drawing)
        .unwrap()
        .1
        .clone();
    let prompt1 = prompts
        .iter()
        .find(|d| *d.0 == combination1.2.prompt)
        .unwrap()
        .1
        .clone();
    let drawing2 = drawings
        .iter()
        .find(|d| *d.0 == combination2.2.drawing)
        .unwrap()
        .1
        .clone();
    let prompt2 = prompts
        .iter()
        .find(|d| *d.0 == combination2.2.prompt)
        .unwrap()
        .1
        .clone();
    let endpoint = server.endpoint_mut();
    let message = ServerMsgRoot::Vote {
        duration: config.duration,
        combination1: (*combination1.1, drawing1, prompt1),
        combination2: (*combination2.1, drawing2, prompt2),
    };
    for (id, _) in users.iter_active() {
        endpoint.send_message(*id, &message).ok();
    }
}

fn update(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Vote>>>,
    mut progress: EventWriter<ProgressGame>,
    mut next: ResMut<NextState<VoteState>>,
    mut context: ResMut<Context>,
    game_config: Res<GameConfig>,
    config: Res<VoteConfig>,
    time: Res<Time>,
    combinations: Query<(&Index, &Combination), Without<VotedOut>>,
    users: Res<Users>,
) {
    let combinations_left = combinations.iter().len();
    for submission in submissions.drain() {
        if context.submited.contains(&submission.author.id) {
            warn!("User submitting vote multiple times!");
            continue;
        }
        info!("{:?}", submission);
        context.submited.insert(submission.author.id);
        if submission.data.combination != context.combination1.1
            && submission.data.combination != context.combination2.1
        {
            warn!("User submitting invalid vote!");
            continue;
        }
        if submission.data.combination == context.combination1.1 {
            context.combination1_votes += 1;
        } else {
            context.combination2_votes += 1;
        }
    }
    let out_of_time = context.started + config.duration + game_config.extra_time < time.elapsed();
    let everyone_submitted = context.submited.len() >= users.iter_active().count();
    if out_of_time || everyone_submitted {
        if context.combination1_votes >= context.combination2_votes {
            commands.entity(context.combination2.0).insert(VotedOut);
        } else {
            commands.entity(context.combination1.0).insert(VotedOut);
        }
        if combinations_left > 2 {
            next.set(VoteState);
        } else {
            progress.send(ProgressGame);
        }
    }
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<VoteConfig>();
    commands.remove_resource::<Context>();
}
