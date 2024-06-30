use std::time::Duration;

use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::{EguiContext, EguiUserTextures};
use bevy_quinnet::client::QuinnetClient;
use common::{
    app::AppExt,
    game::{Combination, Drawing, Index, Prompt, IMG_SIZE},
    protocol::ClientMsgComm,
};
use egui::RichText;

use crate::{
    states::GameState,
    ui::{fonts::IntoFontFamily, widgets::root_element},
    GameSystemOdering,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiAction>();
        app.add_reentrant_statebound(
            GameState::Combine,
            setup,
            teardown,
            (draw_ui, execute_actions)
                .chain()
                .in_set(GameSystemOdering::StateLogic),
        );
    }
}

#[derive(Resource, Clone)]
pub struct Data {
    pub duration: Duration,
    pub drawings: Vec<(Index, Drawing)>,
    pub prompts: Vec<(Index, Prompt)>,
}

#[derive(Resource)]
pub struct Context {
    pub duration: Duration,
    pub drawings: Vec<(Index, (Handle<Image>, egui::Color32))>,
    pub drawing_ptr: usize,
    pub prompts: Vec<(Index, Prompt)>,
    pub prompt_ptr: usize,
}

#[derive(Event)]
enum UiAction {
    NextImage,
    PreviousImage,
    NextPrompt,
    PreviousPrompt,
    Submit,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut textures: ResMut<EguiUserTextures>,
    mut actions: ResMut<Events<UiAction>>,
    data: Res<Data>,
) {
    actions.clear();
    let data = data.clone();

    let mut drawings = Vec::with_capacity(data.drawings.len());
    for drawing in data.drawings {
        let size = Extent3d {
            width: IMG_SIZE as u32,
            height: IMG_SIZE as u32,
            ..default()
        };
        let image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            data: drawing.1.drawing,
            ..default()
        };
        let image_handle = images.add(image);
        textures.add_image(image_handle.clone_weak());
        let bg_color = drawing.1.bg_color;
        let bg_color = egui::Color32::from_rgb(bg_color[0], bg_color[1], bg_color[2]);
        drawings.push((drawing.0, (image_handle, bg_color)));
    }

    let prompts = data.prompts;

    commands.insert_resource(Context {
        duration: data.duration,
        drawings,
        prompts,
        drawing_ptr: 0,
        prompt_ptr: 0,
    });
}

fn draw_ui(
    mut ui_ctx: Query<&mut EguiContext>,
    mut actions: EventWriter<UiAction>,
    images: Res<EguiUserTextures>,
    ctx: Res<Context>,
) {
    let mut ui_ctx = ui_ctx.single_mut();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.label("Combine");
        egui::Grid::new("nav-buttons")
            .num_columns(3)
            .show(ui, |ui| {
                let drawing = &ctx.drawings[ctx.drawing_ptr];
                let image_id = images.image_id(&drawing.1 .0).unwrap();
                if ui.button("<--").clicked() {
                    actions.send(UiAction::PreviousImage);
                }
                ui.allocate_ui(egui::vec2(512., 512.), |ui| {
                    let rect = ui.max_rect();
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, drawing.1 .1);
                    let image = egui::Image::from_texture(egui::load::SizedTexture::new(
                        image_id,
                        egui::vec2(512., 512.),
                    ));
                    ui.add(image);
                });
                if ui.button("-->").clicked() {
                    actions.send(UiAction::NextImage);
                }
                ui.end_row();

                let prompt = &ctx.prompts[ctx.prompt_ptr];
                if ui.button("<--").clicked() {
                    actions.send(UiAction::PreviousPrompt);
                }
                ui.label(RichText::new(&prompt.1.text).font(prompt.1.font.into_font_id()));
                if ui.button("-->").clicked() {
                    actions.send(UiAction::NextPrompt);
                }
                ui.end_row();
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
        let drawing_count = ctx.drawings.len();
        let prompt_count = ctx.prompts.len();
        match action {
            UiAction::NextImage => {
                ctx.drawing_ptr = (ctx.drawing_ptr + 1) % drawing_count;
            }
            UiAction::PreviousImage => {
                ctx.drawing_ptr = (ctx.drawing_ptr + drawing_count - 1) % drawing_count;
            }
            UiAction::NextPrompt => {
                ctx.prompt_ptr = (ctx.prompt_ptr + 1) % prompt_count;
            }
            UiAction::PreviousPrompt => {
                ctx.prompt_ptr = (ctx.prompt_ptr + prompt_count - 1) % prompt_count;
            }
            UiAction::Submit => {
                client
                    .connection_mut()
                    .send_message(
                        ClientMsgComm::SubmitCombination(Combination {
                            drawing: ctx.drawings[ctx.drawing_ptr].0,
                            prompt: ctx.prompts[ctx.prompt_ptr].0,
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
