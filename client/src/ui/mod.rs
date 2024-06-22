pub mod game;
pub mod menu;
pub mod widgets;

use bevy::{prelude::*, render::view::RenderLayers};
use common::{
    app::AppExt,
    transitions::{OnReenter, OnReexit},
};
use game::{
    combine::CombineData,
    draw::{DrawData, UpdateBrush},
    prompt::PromptData,
    vote::VoteData,
};

use crate::{
    states::{ClientState, GameState},
    GameSystemOdering,
};

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        // Menu
        app.add_systems(Update, menu::show.run_if(in_state(ClientState::Menu)));

        // Wait
        app.add_systems(
            Update,
            game::wait::update
                .in_set(GameSystemOdering::StateLogic)
                .run_if(in_state(GameState::Wait)),
        );

        // Draw
        app.add_event::<DrawData>();
        app.add_event::<UpdateBrush>();
        app.insert_gizmo_config(
            DefaultGizmoConfigGroup,
            GizmoConfig {
                render_layers: RenderLayers::layer(1),
                line_joints: GizmoLineJoint::Round(32),
                ..default()
            },
        );
        app.add_reentrant_statebound(
            GameState::Draw,
            game::draw::setup,
            game::draw::teardown,
            (game::draw::resize_brush, game::draw::update)
                .chain()
                .in_set(GameSystemOdering::StateLogic),
        );

        // Prompt
        app.add_event::<PromptData>();
        app.add_reentrant_statebound(
            GameState::Prompt,
            game::prompt::setup,
            game::prompt::teardown,
            game::prompt::update.in_set(GameSystemOdering::StateLogic),
        );

        // Combine
        app.add_event::<CombineData>();
        app.add_reentrant_statebound(
            GameState::Combine,
            game::combine::setup,
            game::combine::teardown,
            game::combine::update.in_set(GameSystemOdering::StateLogic),
        );

        // Vote
        app.add_event::<VoteData>();
        app.add_systems(
            Update,
            game::vote::update
                .in_set(GameSystemOdering::StateLogic)
                .run_if(in_state(GameState::Vote)),
        );
        app.add_systems(OnReexit(GameState::Vote), game::vote::teardown);
    }
}
