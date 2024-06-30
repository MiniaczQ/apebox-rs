use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_quinnet::client::QuinnetClient;
use common::{
    app::AppExt,
    game::{CustomFont, Prompt},
    protocol::ClientMsgComm,
};
use rand::Rng;

use crate::{
    states::GameState,
    ui::{
        fonts::{IntoFontFamily, FONTS},
        widgets::root_element,
    },
    GameSystemOdering,
};

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
    pub font: CustomFont,
    pub prompt: String,
}

impl Context {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let font = CustomFont(rng.gen_range(0..FONTS.len()));

        Context {
            font,
            prompt: String::new(),
        }
    }
}

#[derive(Event)]
enum UiAction {
    Submit,
}

fn setup(mut commands: Commands, mut actions: ResMut<Events<UiAction>>) {
    actions.clear();

    commands.insert_resource(Context::new());
}

fn show_ui(
    mut ui_ctx: Query<&mut EguiContext>,
    mut ctx: ResMut<Context>,
    mut actions: EventWriter<UiAction>,
) {
    let mut ui_ctx = ui_ctx.single_mut();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.horizontal(|ui| {
            let font_id = ctx.font.into_font_id();
            ui.label(egui::RichText::new("Prompt").font(font_id.clone()));
            ui.add(
                egui::TextEdit::singleline(&mut ctx.prompt)
                    .font(font_id)
                    .text_color(egui::Color32::WHITE),
            );
        });

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
                let mut old_ctx = Context::new();
                std::mem::swap(&mut *ctx, &mut old_ctx);
                client
                    .connection_mut()
                    .send_message(
                        ClientMsgComm::SubmitPrompt(Prompt {
                            text: old_ctx.prompt,
                            font: old_ctx.font,
                        })
                        .root(),
                    )
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
