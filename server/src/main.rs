mod game;
mod networking;
mod states;

use std::{ops::Range, time::Duration};

use bevy::{
    dev_tools::states::log_transitions, ecs::prelude::Resource, log::LogPlugin,
    state::app::StatesPlugin, utils::hashbrown::HashSet,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::{
    server::{QuinnetServer, QuinnetServerPlugin},
    shared::ClientId,
};
use common::{
    app::AppExt,
    game::{Combination, Combined, Drawing, Index, Indexer, Prompt, Vote, VotedOut},
    protocol::{ClientMsgComm, ClientMsgRoot, NetMsg, ServerMsgRoot},
    transitions::IdentityTransitionsPlugin,
};
use game::{CombineConfig, DrawConfig, GameConfig, PromptConfig, StateData, VoteConfig};
use networking::Submission;
use rand::prelude::SliceRandom;
use states::{GameState, RoomState, ServerState, VoteState};

fn main() {
    let mut app = App::new();
    app.init_state::<ServerState>();
    app.add_sub_state::<RoomState>();
    app.add_sub_state::<GameState>();
    app.add_sub_state::<VoteState>();
    app.add_plugins((
        MinimalPlugins,
        LogPlugin::default(),
        StatesPlugin,
        QuinnetServerPlugin::default(),
        IdentityTransitionsPlugin::<GameState>::default(),
        IdentityTransitionsPlugin::<VoteState>::default(),
    ));
    app.init_resource::<Users>();
    app.configure_sets(
        Update,
        (
            GameSystemOdering::Networking,
            GameSystemOdering::StateLogic,
            GameSystemOdering::ChangeState,
        )
            .chain(),
    );

    // Debug
    app.add_systems(
        Update,
        (
            log_transitions::<ServerState>,
            log_transitions::<RoomState>,
            log_transitions::<GameState>,
        )
            .chain(),
    );

    // ServerState::Offline
    app.add_systems(OnEnter(ServerState::Offline), states::setup_server_offline);

    // ServerState::Running
    app.add_systems(OnEnter(ServerState::Running), states::setup_server_online);
    app.add_systems(OnExit(ServerState::Running), states::teardown_server_online);
    app.add_event::<NetMsg<ClientMsgRoot>>();
    app.add_event::<NetMsg<ClientMsgComm>>();
    app.add_event::<ProgressGame>();
    app.add_systems(
        PreUpdate,
        (
            networking::handle_server_events,
            networking::receive_messages,
            networking::handle_root,
            networking::handle_comm,
        )
            .chain()
            .in_set(GameSystemOdering::Networking)
            .run_if(in_state(ServerState::Running)),
    );
    app.enable_state_scoped_entities::<RoomState>();

    // RoomState::Waiting
    app.add_systems(
        Update,
        start_lobby
            .in_set(GameSystemOdering::ChangeState)
            .run_if(in_state(RoomState::Waiting)),
    );

    // RoomState::Running
    app.add_event::<ProgressGame>();
    app.add_statebound(
        RoomState::Running,
        states::setup_room_running,
        states::teardown_room_running,
        (progress_game, stop_lobby).in_set(GameSystemOdering::ChangeState),
    );

    // Draw
    app.add_event::<Submission<Drawing>>();
    app.add_reentrant_statebound(
        GameState::Draw,
        setup_draw,
        teardown_draw,
        update_draw.in_set(GameSystemOdering::StateLogic),
    );

    // Prompt
    app.add_event::<Submission<Prompt>>();
    app.add_reentrant_statebound(
        GameState::Prompt,
        setup_prompt,
        teardown_prompt,
        update_prompt.in_set(GameSystemOdering::StateLogic),
    );

    // Combine
    app.add_event::<Submission<Combination>>();
    app.add_reentrant_statebound(
        GameState::Combine,
        setup_combine,
        teardown_combine,
        update_combine.in_set(GameSystemOdering::StateLogic),
    );

    // Vote
    app.add_event::<Submission<Vote>>();
    app.add_reentrant_statebound(
        VoteState,
        setup_vote,
        teardown_vote,
        update_vote.in_set(GameSystemOdering::StateLogic),
    );

    app.run();
}

#[derive(SystemSet, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum GameSystemOdering {
    Networking,
    StateLogic,
    ChangeState,
}

/// Users and information about them.
#[derive(Resource, Debug, Clone, Default)]
struct Users {
    /// Clients that connected but didn't register a username.
    pending: HashMap<ClientId, Duration>,
    /// Clients that registered a username.
    registered: HashMap<ClientId, UserData>,
}

impl Users {
    /// Register a new pending user.
    fn add_pending(&mut self, id: ClientId, now: Duration) {
        self.pending.insert(id, now);
    }

    /// Turn a pending user into an active user by giving them a name.
    fn register(&mut self, id: ClientId, name: String) {
        self.pending.remove(&id).unwrap();
        self.registered.insert(
            id,
            UserData {
                name,
                playing: false,
            },
        );
    }

    /// Set all registered players to playing.
    fn set_playing(&mut self) {
        for user in self.registered.values_mut() {
            user.playing = true;
        }
    }

    /// Remove all trace of a user.
    fn remove(&mut self, id: &ClientId) -> Option<UserData> {
        self.pending.remove(id);
        self.registered.remove(id)
    }

    fn iter_active(&self) -> impl Iterator<Item = (&u64, &UserData)> {
        self.registered.iter().filter(|(_, u)| u.playing)
    }

    /// Take all pending users who didn't register a name for too long.
    fn drain_pending_too_long(&mut self, max_pending: Duration, now: Duration) -> Vec<ClientId> {
        let mut pending_too_long = vec![];
        for (id, joined) in self.pending.iter() {
            if *joined + max_pending < now {
                pending_too_long.push(*id);
            }
        }
        for id in pending_too_long.iter() {
            self.pending.remove(id);
        }
        pending_too_long
    }
}

/// Data of a single registered user.
#[derive(Resource, Debug, Clone)]
struct UserData {
    /// Username.
    name: String,
    /// Whether the user is waiting for a new game or already playing.
    playing: bool,
}

#[derive(Event)]
pub struct ProgressGame;

fn progress_game(
    mut progress: ResMut<Events<ProgressGame>>,
    mut commands: Commands,
    mut room_next: ResMut<NextState<RoomState>>,
    mut game_next: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameConfig>,
) {
    if progress.drain().last().is_none() {
        return;
    }

    let Some(next_state) = game_data.next_state() else {
        room_next.set(RoomState::Waiting);
        return;
    };

    match next_state {
        StateData::Draw(config) => {
            game_next.set(GameState::Draw);
            commands.insert_resource(config);
        }
        StateData::Prompt(config) => {
            game_next.set(GameState::Prompt);
            commands.insert_resource(config);
        }
        StateData::Combine(config) => {
            game_next.set(GameState::Combine);
            commands.insert_resource(config);
        }
        StateData::Vote(config) => {
            game_next.set(GameState::Vote);
            commands.insert_resource(config);
        }
    };
}

fn start_lobby(
    users: Res<Users>,
    mut room_next: ResMut<NextState<RoomState>>,
    mut progress: EventWriter<ProgressGame>,
) {
    // TODO: Wait for host start instead
    if users.registered.len() >= 3 {
        room_next.set(RoomState::Running);
        progress.send(ProgressGame);
    }
}

fn stop_lobby(users: Res<Users>, mut room_next: ResMut<NextState<RoomState>>) {
    if users.registered.len() < 1 {
        room_next.set(RoomState::Waiting);
    }
}

#[derive(Resource, Debug)]
pub struct DrawCtx {
    started: Duration,
    submited: HashSet<ClientId>,
}

fn setup_draw(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<DrawConfig>,
) {
    info!("Setup draw {:?}", time.elapsed());
    commands.insert_resource(DrawCtx {
        started: time.elapsed(),
        submited: HashSet::new(),
    });
    let endpoint = server.endpoint_mut();
    let message = ServerMsgRoot::Draw {
        duration: config.duration,
    };
    for (id, _) in users.iter_active() {
        endpoint.send_message(*id, &message).ok();
    }
}

fn update_draw(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Drawing>>>,
    mut progress: EventWriter<ProgressGame>,
    mut indexer: ResMut<Indexer>,
    mut context: ResMut<DrawCtx>,
    game_config: Res<GameConfig>,
    config: Res<DrawConfig>,
    time: Res<Time>,
    users: Res<Users>,
) {
    for submission in submissions.drain() {
        if context.submited.contains(&submission.author.id) {
            warn!("User submitting drawing multiple times!");
            continue;
        }
        info!("{:?}", submission);
        context.submited.insert(submission.author.id);
        commands.spawn((
            StateScoped(RoomState::Running),
            submission.author,
            submission.data,
            indexer.next(),
        ));
        if context.submited.len() >= users.iter_active().count() {
            progress.send(ProgressGame);
            return;
        }
    }
    if context.started + config.duration + game_config.extra_time < time.elapsed() {
        info!("{:?} {:?}", context.started, time.elapsed());
        progress.send(ProgressGame);
    }
}

fn teardown_draw(mut commands: Commands) {
    commands.remove_resource::<DrawConfig>();
    commands.remove_resource::<DrawCtx>();
}

#[derive(Resource, Debug)]
pub struct PromptCtx {
    started: Duration,
    submited: usize,
}

fn setup_prompt(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<PromptConfig>,
) {
    info!("Setup prompt");
    commands.insert_resource(PromptCtx {
        started: time.elapsed(),
        submited: 0,
    });
    let endpoint = server.endpoint_mut();
    let message = ServerMsgRoot::Prompt {
        duration: config.duration,
    };
    for (id, _) in users.iter_active() {
        endpoint.send_message(*id, &message).ok();
    }
}

fn update_prompt(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Prompt>>>,
    mut progress: EventWriter<ProgressGame>,
    mut indexer: ResMut<Indexer>,
    mut context: ResMut<PromptCtx>,
    game_config: Res<GameConfig>,
    config: Res<PromptConfig>,
    time: Res<Time>,
    users: Res<Users>,
) {
    let user_count = users.iter_active().count();
    let max_submissions = user_count * config.prompts_per_player;
    for submission in submissions.drain() {
        info!("{:?}", submission);
        context.submited += 1;
        commands.spawn((
            StateScoped(RoomState::Running),
            submission.author,
            submission.data,
            indexer.next(),
        ));
        if max_submissions <= context.submited {
            progress.send(ProgressGame);
            return;
        }
    }
    if context.started + config.duration + game_config.extra_time < time.elapsed() {
        progress.send(ProgressGame);
    }
}

fn teardown_prompt(mut commands: Commands) {
    commands.remove_resource::<PromptConfig>();
    commands.remove_resource::<PromptCtx>();
}

#[derive(Resource, Debug)]
pub struct CombineCtx {
    started: Duration,
    submited: HashSet<ClientId>,
}

fn setup_combine(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<CombineConfig>,
    drawings: Query<(Entity, &Index, &Drawing), Without<Prompt>>,
    prompts: Query<(Entity, &Index, &Prompt), Without<Drawing>>,
) {
    info!("Setup combine");
    commands.insert_resource(CombineCtx {
        started: time.elapsed(),
        submited: HashSet::new(),
    });

    let mut rng = rand::thread_rng();
    let mut drawing_ids = drawings.iter().map(|e| e.0).collect::<Vec<_>>();
    drawing_ids.shuffle(&mut rng);
    let mut prompt_ids = prompts.iter().map(|e| e.0).collect::<Vec<_>>();
    prompt_ids.shuffle(&mut rng);
    let mut user_ids = users.iter_active().map(|u| *u.0).collect::<Vec<_>>();
    user_ids.shuffle(&mut rng);

    let drawing_count = drawing_ids.len();
    let prompt_count = prompt_ids.len();
    let user_count = user_ids.len();

    if drawing_count < user_count || prompt_count < user_count {
        todo!("Handle not enough content!");
    }

    let min_drawings_per_user = drawing_count / user_count;
    let extra_drawing_for_first_n_users = drawing_count % user_count;
    let min_prompts_per_user = prompt_count / user_count;
    let extra_prompt_for_first_n_users = prompt_count % user_count;

    fn count_for_idx(min: usize, extra: usize, idx: usize) -> usize {
        idx * min + (idx < extra) as usize
    }

    fn range_for_idx(min: usize, extra: usize, idx: usize) -> Range<usize> {
        if idx == 0 {
            0..count_for_idx(min, extra, idx + 1)
        } else {
            count_for_idx(min, extra, idx)..count_for_idx(min, extra, idx + 1)
        }
    }

    let endpoint = server.endpoint_mut();
    for (idx, id) in user_ids.into_iter().enumerate() {
        let drawing_ids = &drawing_ids
            [range_for_idx(min_drawings_per_user, extra_drawing_for_first_n_users, idx)];
        let drawings = drawings
            .iter_many(drawing_ids)
            .map(|d| (*d.1, d.2.clone()))
            .collect::<Vec<_>>();

        let prompt_ids =
            &prompt_ids[range_for_idx(min_prompts_per_user, extra_prompt_for_first_n_users, idx)];
        let prompts = prompts
            .iter_many(prompt_ids)
            .map(|d| (*d.1, d.2.clone()))
            .collect::<Vec<_>>();

        info!(id = ?id, d = drawing_ids.len(), p = prompt_ids.len(), "Combinations sent to user");

        let message = ServerMsgRoot::Combine {
            duration: config.duration,
            drawings,
            prompts,
        };

        endpoint.send_message(id, &message).ok();
    }
}

fn update_combine(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Combination>>>,
    mut progress: EventWriter<ProgressGame>,
    mut indexer: ResMut<Indexer>,
    mut context: ResMut<CombineCtx>,
    game_config: Res<GameConfig>,
    config: Res<CombineConfig>,
    time: Res<Time>,
    drawings: Query<(Entity, &Index), (With<Drawing>, Without<Prompt>, Without<Combined>)>,
    prompts: Query<(Entity, &Index), (With<Prompt>, Without<Drawing>, Without<Combined>)>,
    users: Res<Users>,
) {
    for submission in submissions.drain() {
        if context.submited.contains(&submission.author.id) {
            warn!("User submitting combination multiple times!");
            continue;
        }

        let drawing_idx = submission.data.drawing;
        let drawing = drawings.iter().find(|d| *d.1 == drawing_idx);
        let prompt_idx = submission.data.prompt;
        let prompt = prompts.iter().find(|d| *d.1 == prompt_idx);

        let (drawing, prompt) = match (drawing, prompt) {
            (Some(drawing), Some(prompt)) => (drawing, prompt),
            _ => {
                warn!("Combination submission doesn't target a valid drawing and/or prompt.");
                continue;
            }
        };
        info!("{:?}", submission);

        context.submited.insert(submission.author.id);
        commands.spawn((
            StateScoped(RoomState::Running),
            submission.author,
            submission.data,
            indexer.next(),
        ));
        commands.entity(drawing.0).insert(Combined);
        commands.entity(prompt.0).insert(Combined);

        if context.submited.len() >= users.iter_active().count() {
            progress.send(ProgressGame);
            return;
        }
    }
    if context.started + config.duration + game_config.extra_time < time.elapsed() {
        progress.send(ProgressGame);
    }
}

fn teardown_combine(mut commands: Commands) {
    commands.remove_resource::<CombineConfig>();
    commands.remove_resource::<CombineCtx>();
}

#[derive(Resource, Debug)]
pub struct VoteCtx {
    started: Duration,
    submited: HashSet<ClientId>,
    combination1: (Entity, Index),
    combination1_votes: usize,
    combination2: (Entity, Index),
    combination2_votes: usize,
}

fn setup_vote(
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
    commands.insert_resource(VoteCtx {
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
        .find(|d| *d.0 == combination1.2.drawing)
        .unwrap()
        .1
        .clone();
    let prompt2 = prompts
        .iter()
        .find(|d| *d.0 == combination1.2.prompt)
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

fn update_vote(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Vote>>>,
    mut progress: EventWriter<ProgressGame>,
    mut next: ResMut<NextState<VoteState>>,
    mut context: ResMut<VoteCtx>,
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
    let everyone_submited = context.submited.len() >= users.iter_active().count();
    if out_of_time || everyone_submited {
        if combinations_left >= 2 {
            if context.combination1_votes >= context.combination2_votes {
                commands.entity(context.combination2.0).insert(VotedOut);
            } else {
                commands.entity(context.combination1.0).insert(VotedOut);
            }
            next.set(VoteState);
        } else {
            progress.send(ProgressGame);
        }
    }
}

fn teardown_vote(mut commands: Commands) {
    commands.remove_resource::<VoteConfig>();
    commands.remove_resource::<VoteCtx>();
}
