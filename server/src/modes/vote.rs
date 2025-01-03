use std::time::Duration;

use bevy::prelude::*;
use bevy::{ecs::prelude::Resource, utils::hashbrown::HashSet};
use bevy_quinnet::{server::QuinnetServer, shared::ClientId};
use common::{
    game::{Combination, Drawing, Index, Prompt, Vote, VotedOut},
    protocol::ServerMsgRoot,
};
use rand::prelude::SliceRandom;

use crate::{
    game::{GameConfig, VoteConfig},
    networking::Submission,
    states::{GameState, VoteState},
    GameSystemOdering, ProgressGame, Users,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Submission<Vote>>();
        app.add_systems(OnExit(GameState::Vote), teardown_vote);

        app.add_event::<Winner>();
        app.add_systems(OnEnter(VoteState::Voting), setup_voting);
        app.add_systems(
            Update,
            update_voting
                .run_if(in_state(VoteState::Voting))
                .in_set(GameSystemOdering::StateLogic),
        );
        app.add_systems(OnExit(VoteState::Voting), teardown_voting);

        app.add_systems(OnEnter(VoteState::Winner), setup_winner);
        app.add_systems(
            Update,
            update_winner
                .run_if(in_state(VoteState::Winner))
                .in_set(GameSystemOdering::StateLogic),
        );
        app.add_systems(OnExit(VoteState::Winner), teardown_winner);
    }
}

#[derive(Resource, Debug)]
pub struct VotingContext {
    started: Duration,
    submited: HashSet<ClientId>,
    combination1: (Entity, Index),
    combination1_votes: usize,
    combination2: (Entity, Index),
    combination2_votes: usize,
}

#[derive(Resource, Debug)]
pub struct WinnerContext {
    started: Duration,
}

#[derive(Event)]
pub struct Winner(Entity);

fn setup_voting(
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
    commands.insert_resource(VotingContext {
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
        duration: config.voting_duration,
        combination1: (*combination1.1, drawing1, prompt1),
        combination2: (*combination2.1, drawing2, prompt2),
    };
    for (id, _) in users.iter_active() {
        endpoint.send_message(*id, &message).ok();
    }
}

fn update_voting(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Vote>>>,
    mut next: ResMut<NextState<VoteState>>,
    mut context: ResMut<VotingContext>,
    mut winner: EventWriter<Winner>,
    game_config: Res<GameConfig>,
    config: Res<VoteConfig>,
    time: Res<Time>,
    users: Res<Users>,
) {
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
    let out_of_time =
        context.started + config.voting_duration + game_config.extra_time < time.elapsed();
    let everyone_submitted = context.submited.len() >= users.iter_active().count();
    if out_of_time || everyone_submitted {
        if context.combination1_votes >= context.combination2_votes {
            commands.entity(context.combination2.0).insert(VotedOut);
            winner.send(Winner(context.combination1.0));
        } else {
            commands.entity(context.combination1.0).insert(VotedOut);
            winner.send(Winner(context.combination2.0));
        }
        next.set(VoteState::Winner);
    }
}

fn setup_winner(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<VoteConfig>,
    combinations: Query<
        &Combination,
        (
            With<Combination>,
            Without<VotedOut>,
            Without<Drawing>,
            Without<Prompt>,
        ),
    >,
    drawings: Query<(&Index, &Drawing), (Without<Combination>, Without<Prompt>)>,
    prompts: Query<(&Index, &Prompt), (Without<Combination>, Without<Drawing>)>,
    mut winner: ResMut<Events<Winner>>,
) {
    let winner = winner.drain().last().unwrap();
    commands.insert_resource(WinnerContext {
        started: time.elapsed(),
    });

    let combination = combinations.get(winner.0).unwrap();
    let drawing = drawings
        .iter()
        .find(|d| *d.0 == combination.drawing)
        .unwrap()
        .1
        .clone();
    let prompt = prompts
        .iter()
        .find(|d| *d.0 == combination.prompt)
        .unwrap()
        .1
        .clone();

    let endpoint = server.endpoint_mut();
    let message = ServerMsgRoot::Winner {
        duration: config.winner_duration,
        drawing,
        prompt,
    };
    for (id, _) in users.iter_active() {
        endpoint.send_message(*id, &message).ok();
    }
}

fn update_winner(
    mut progress: EventWriter<ProgressGame>,
    mut next: ResMut<NextState<VoteState>>,
    combinations: Query<(&Index, &Combination), Without<VotedOut>>,
    context: Res<WinnerContext>,
    config: Res<VoteConfig>,
    time: Res<Time>,
) {
    let out_of_time = context.started + config.voting_duration < time.elapsed();
    if out_of_time {
        let combinations_left = combinations.iter().len();
        if combinations_left > 2 {
            next.set(VoteState::Voting);
        } else {
            progress.send(ProgressGame);
        }
    }
}

fn teardown_vote(mut commands: Commands) {
    commands.remove_resource::<VoteConfig>();
}

fn teardown_voting(mut commands: Commands) {
    commands.remove_resource::<VotingContext>();
}

fn teardown_winner(mut commands: Commands) {
    commands.remove_resource::<WinnerContext>();
}
