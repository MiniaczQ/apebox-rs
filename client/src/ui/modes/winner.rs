use std::time::Duration;

use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::{EguiContext, EguiUserTextures};
use common::{
    app::AppExt,
    game::{Drawing, Prompt},
};

use crate::{states::GameState, ui::widgets::root_element, GameSystemOdering};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_reentrant_statebound(
            GameState::Winner,
            setup,
            teardown,
            show_ui.in_set(GameSystemOdering::StateLogic),
        );
    }
}

#[derive(Resource, Clone)]
pub struct Data {
    pub duration: Duration,
    pub drawing: Drawing,
    pub prompt: Prompt,
}

#[derive(Resource)]
pub struct Context {
    pub duration: Duration,
    pub drawing: (Handle<Image>, egui::Color32),
    pub prompt: Prompt,
    pub shirt: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    asset_server: Res<AssetServer>,
    data: Res<Data>,
) {
    let data = data.clone();

    let shirt: Handle<Image> = asset_server.load("textures/shirts/shirt1.png"); // TODO: cache loaded images
    egui_user_textures.add_image(shirt.clone_weak());

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
        data: data.drawing.drawing,
        ..default()
    };
    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone_weak());
    let bg_color = data.drawing.bg_color;
    let bg_color = egui::Color32::from_rgb(bg_color[0], bg_color[1], bg_color[2]);

    commands.insert_resource(Context {
        duration: data.duration,
        drawing: (image_handle, bg_color),
        prompt: data.prompt,
        shirt,
    });
}

fn show_ui(mut ui_ctx: Query<&mut EguiContext>, images: Res<EguiUserTextures>, ctx: Res<Context>) {
    let mut ui_ctx = ui_ctx.single_mut();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.label("Winner");

        ui.vertical(|ui| {
            super::vote::show_combination(ui, &images, &ctx.drawing, &ctx.prompt, &ctx.shirt);
        });
    });
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<Data>();
    commands.remove_resource::<Context>();
}
