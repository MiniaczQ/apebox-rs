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
    game::{Combination, Drawing, Index, Prompt},
    protocol::ClientMsgComm,
};

use crate::ui::widgets::root_element;

#[derive(Event)]
pub struct CombineData {
    pub duration: Duration,
    pub drawings: Vec<(Index, Drawing)>,
    pub prompts: Vec<(Index, Prompt)>,
}

#[derive(Resource)]
pub struct CombineContext {
    pub duration: Duration,
    pub drawings: Vec<(Index, Handle<Image>)>,
    pub drawing_ptr: usize,
    pub prompts: Vec<(Index, Prompt)>,
    pub prompt_ptr: usize,
}

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut data: ResMut<Events<CombineData>>,
) {
    let data = data.drain().last().unwrap();

    let mut drawings = Vec::with_capacity(data.drawings.len());
    for drawing in data.drawings {
        let size = Extent3d {
            width: 512,
            height: 512,
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
            data: drawing.1.data,
            ..default()
        };
        let image_handle = images.add(image);
        egui_user_textures.add_image(image_handle.clone_weak());
        drawings.push((drawing.0, image_handle));
    }

    let prompts = data.prompts;

    commands.insert_resource(CombineContext {
        duration: data.duration,
        drawings,
        prompts,
        drawing_ptr: 0,
        prompt_ptr: 0,
    });
}

pub fn update(
    mut ctx: Query<&mut EguiContext>,
    images: Res<EguiUserTextures>,
    mut combine_ctx: ResMut<CombineContext>,
    mut client: ResMut<QuinnetClient>,
) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Combine");
        egui::Grid::new("nav-buttons")
            .num_columns(3)
            .show(ui, |ui| {
                let image_id = images
                    .image_id(&combine_ctx.drawings[combine_ctx.drawing_ptr].1)
                    .unwrap();
                let prev_drawing = ui.button("<").clicked();
                ui.image(egui::load::SizedTexture::new(
                    image_id,
                    egui::vec2(300., 300.),
                ));
                let next_drawing = ui.button(">").clicked();
                ui.end_row();

                let drawing_count = combine_ctx.drawings.len();
                if next_drawing {
                    combine_ctx.drawing_ptr = (combine_ctx.drawing_ptr + 1) % drawing_count;
                } else if prev_drawing {
                    combine_ctx.drawing_ptr =
                        (combine_ctx.drawing_ptr + drawing_count - 1) % drawing_count;
                }

                let prompt = &combine_ctx.prompts[combine_ctx.prompt_ptr];
                let prev_prompt = ui.button("<").clicked();
                ui.label(&prompt.1.data);
                let next_prompt = ui.button(">").clicked();
                ui.end_row();

                let prompt_count = combine_ctx.prompts.len();
                if next_prompt {
                    combine_ctx.prompt_ptr = (combine_ctx.prompt_ptr + 1) % prompt_count;
                } else if prev_prompt {
                    combine_ctx.prompt_ptr =
                        (combine_ctx.prompt_ptr + prompt_count - 1) % prompt_count;
                }
            });
        let submit = ui.button("Submit").clicked();
        if submit {
            client
                .connection_mut()
                .send_message(
                    ClientMsgComm::SubmitCombination(Combination {
                        drawing: combine_ctx.drawings[combine_ctx.drawing_ptr].0,
                        prompt: combine_ctx.prompts[combine_ctx.prompt_ptr].0,
                    })
                    .root(),
                )
                .ok();
        }
    });
}

pub fn teardown(mut commands: Commands) {}
