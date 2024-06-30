use std::{ops::Range, time::Duration};

use bevy::prelude::*;
use bevy::{ecs::prelude::Resource, utils::hashbrown::HashSet};
use bevy_quinnet::{server::QuinnetServer, shared::ClientId};
use common::{
    app::AppExt,
    game::{Combination, Combined, Drawing, Index, Indexer, Prompt},
    protocol::ServerMsgRoot,
};
use rand::prelude::SliceRandom;

use crate::{
    game::{CombineConfig, GameConfig},
    networking::Submission,
    states::{GameState, RoomState},
    GameSystemOdering, ProgressGame, Users,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Submission<Combination>>();
        app.add_reentrant_statebound(
            GameState::Combine,
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
    config: Res<CombineConfig>,
    drawings: Query<(Entity, &Index, &Drawing), Without<Prompt>>,
    prompts: Query<(Entity, &Index, &Prompt), Without<Drawing>>,
) {
    info!("Setup combine");
    commands.insert_resource(Context {
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

        let message = ServerMsgRoot::Combine {
            duration: config.duration,
            drawings,
            prompts,
        };

        endpoint.send_message(id, &message).ok();
    }
}

fn update(
    mut commands: Commands,
    mut submissions: ResMut<Events<Submission<Combination>>>,
    mut progress: EventWriter<ProgressGame>,
    mut indexer: ResMut<Indexer>,
    mut context: ResMut<Context>,
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
    }
    let out_of_time = context.started + config.duration + game_config.extra_time < time.elapsed();
    let everyone_submitted = context.submited.len() >= users.iter_active().count();
    if out_of_time || everyone_submitted {
        progress.send(ProgressGame);
    }
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<CombineConfig>();
    commands.remove_resource::<Context>();
}
