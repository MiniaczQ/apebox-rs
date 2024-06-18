mod game;
mod menu;
mod widgets;

use bevy::{prelude::*, render::view::RenderLayers};
use game::draw::UpdateBrush;

use crate::states::{ClientState, GameState};

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateBrush>();
        app.insert_gizmo_config(
            DefaultGizmoConfigGroup,
            GizmoConfig {
                render_layers: RenderLayers::layer(1),
                line_joints: GizmoLineJoint::Round(32),
                ..default()
            },
        );

        // Menu
        app.add_systems(Update, menu::show.run_if(in_state(ClientState::Menu)));

        // Wait
        app.add_systems(Update, game::wait::update.run_if(in_state(GameState::Wait)));

        // Draw
        app.add_systems(
            Update,
            (game::draw::resize_brush, game::draw::update)
                .chain()
                .run_if(in_state(GameState::Draw)),
        );
        app.add_systems(OnEnter(GameState::Draw), game::draw::setup);
        app.add_systems(OnExit(GameState::Draw), game::draw::teardown);

        // Prompt
        app.add_systems(
            Update,
            game::prompt::update.run_if(in_state(GameState::Prompt)),
        );
        app.add_systems(OnEnter(GameState::Prompt), game::prompt::setup);
        app.add_systems(OnExit(GameState::Prompt), game::prompt::teardown);

        // Combine
        app.add_systems(
            Update,
            game::combine::update.run_if(in_state(GameState::Combine)),
        );
        app.add_systems(OnExit(GameState::Combine), game::combine::teardown);

        // Vote
        app.add_systems(Update, game::vote::update.run_if(in_state(GameState::Vote)));
        app.add_systems(OnExit(GameState::Vote), game::vote::teardown);
    }
}
