use std::time::Duration;

use bevy::ecs::prelude::Resource;
use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServer;
use common::{
    app::AppExt,
    game::{Indexer, Prompt},
    protocol::ServerMsgRoot,
};

use crate::{
    game::{GameConfig, PromptConfig},
    networking::Submission,
    states::{GameState, RoomState},
    GameSystemOdering, ProgressGame, Users,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Submission<Prompt>>();
        app.add_reentrant_statebound(
            GameState::Prompt,
            setup,
            teardown,
            update.in_set(GameSystemOdering::StateLogic),
        );
    }
}

#[derive(Resource, Debug)]
pub struct Context {
    started: Duration,
    submited: usize,
}

fn setup(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<PromptConfig>,
) {
    info!("Setup prompt");
    commands.insert_resource(Context {
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

fn update(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Prompt>>>,
    mut progress: EventWriter<ProgressGame>,
    mut indexer: ResMut<Indexer>,
    mut context: ResMut<Context>,
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
    }
    let out_of_time = context.started + config.duration + game_config.extra_time < time.elapsed();
    let everyone_submitted = max_submissions <= context.submited;
    if out_of_time || everyone_submitted {
        progress.send(ProgressGame);
    }
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<PromptConfig>();
    commands.remove_resource::<Context>();
}
