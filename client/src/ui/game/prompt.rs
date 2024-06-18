use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::{networking::PromptData, ui::widgets::root_element};

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
    commands.remove_resource::<PromptData>();
}

pub fn update(mut ctx: Query<&mut EguiContext>, mut prompt_ctx: ResMut<PromptContext>) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Prompt");
        ui.text_edit_singleline(&mut prompt_ctx.prompt);
        _ = ui.button("Submit");
    });
}
