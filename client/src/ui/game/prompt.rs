use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_quinnet::client::QuinnetClient;
use common::{app::AppExt, game::Prompt, protocol::ClientMsgComm};

use crate::{states::GameState, ui::widgets::root_element, GameSystemOdering};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiAction>();
        app.add_reentrant_statebound(
            GameState::Prompt,
            setup,
            teardown,
            (show_ui, execute_actions)
                .chain()
                .in_set(GameSystemOdering::StateLogic),
        );
    }
}

#[derive(Resource, Clone)]
pub struct Data {
    pub duration: Duration,
}

#[derive(Resource)]
pub struct Context {
    pub prompt: String,
}

#[derive(Event)]
enum UiAction {
    Submit,
}

fn setup(mut commands: Commands, mut actions: ResMut<Events<UiAction>>) {
    actions.clear();

    commands.insert_resource(Context {
        prompt: String::new(),
    });
}

fn show_ui(
    mut ui_ctx: Query<&mut EguiContext>,
    mut ctx: ResMut<Context>,
    mut actions: EventWriter<UiAction>,
) {
    let mut ui_ctx = ui_ctx.single_mut();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.label("Prompt");
        ui.text_edit_singleline(&mut ctx.prompt);

        if ui.button("Submit").clicked() {
            actions.send(UiAction::Submit);
        }
    });
}

fn execute_actions(
    mut actions: ResMut<Events<UiAction>>,
    mut ctx: ResMut<Context>,
    mut client: ResMut<QuinnetClient>,
) {
    for action in actions.drain() {
        match action {
            UiAction::Submit => {
                let data = std::mem::take(&mut ctx.prompt);
                client
                    .connection_mut()
                    .send_message(ClientMsgComm::SubmitPrompt(Prompt { data }).root())
                    .ok();
            }
        }
    }
}

fn teardown(mut commands: Commands, mut actions: ResMut<Events<UiAction>>) {
    commands.remove_resource::<Data>();
    commands.remove_resource::<Context>();
    actions.clear();
}
