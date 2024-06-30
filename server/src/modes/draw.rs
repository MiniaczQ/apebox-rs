use std::time::Duration;

use bevy::prelude::*;
use bevy::{ecs::prelude::Resource, utils::hashbrown::HashSet};
use bevy_quinnet::{server::QuinnetServer, shared::ClientId};
use common::{
    app::AppExt,
    game::{Drawing, Indexer},
    protocol::ServerMsgRoot,
};

use crate::{
    game::{DrawConfig, GameConfig},
    networking::Submission,
    states::{GameState, RoomState},
    GameSystemOdering, ProgressGame, Users,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Submission<Drawing>>();
        app.add_reentrant_statebound(
            GameState::Draw,
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
}

fn setup(
    mut commands: Commands,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
    users: Res<Users>,
    config: Res<DrawConfig>,
) {
    info!("Setup draw");
    commands.insert_resource(Context {
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

fn update(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Drawing>>>,
    mut progress: EventWriter<ProgressGame>,
    mut indexer: ResMut<Indexer>,
    mut context: ResMut<Context>,
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
    }
    let out_of_time = context.started + config.duration + game_config.extra_time < time.elapsed();
    let everyone_submitted = context.submited.len() >= users.iter_active().count();
    if out_of_time || everyone_submitted {
        progress.send(ProgressGame);
    }
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<DrawConfig>();
    commands.remove_resource::<Context>();
}
