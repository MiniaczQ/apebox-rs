use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_quinnet::client::QuinnetClient;
use common::{game::Prompt, protocol::ClientMsgComm};

use crate::ui::widgets::root_element;

#[derive(Event)]
pub struct PromptData {
    pub duration: Duration,
}

#[derive(Resource)]
pub struct PromptContext {
    pub prompt: String,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(PromptContext {
        prompt: String::new(),
    });
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<PromptContext>();
}

pub fn update(
    mut ctx: Query<&mut EguiContext>,
    mut prompt_ctx: ResMut<PromptContext>,
    mut client: ResMut<QuinnetClient>,
) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Prompt");
        ui.text_edit_singleline(&mut prompt_ctx.prompt);

        let submit = ui.button("Submit").clicked();
        if submit {
            let data = std::mem::take(&mut prompt_ctx.prompt);
            client
                .connection_mut()
                .send_message(ClientMsgComm::SubmitPrompt(Prompt { data }).root())
                .ok();
        }
    });
}
