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
    game::{Drawing, Index, Prompt, Vote},
    protocol::ClientMsgComm,
};

use crate::{states::GameState, ui::widgets::root_element, GameSystemOdering};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiAction>();
        app.add_reentrant_statebound(
            GameState::Vote,
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
    pub combination1: (Index, Drawing, Prompt),
    pub combination2: (Index, Drawing, Prompt),
}

#[derive(Resource)]
pub struct Context {
    pub duration: Duration,
    pub combination1: (Index, Handle<Image>, String),
    pub combination2: (Index, Handle<Image>, String),
}

#[derive(Event)]
enum UiAction {
    Vote1,
    Vote2,
}

fn setup(
    mut commands: Commands,
    mut actions: ResMut<Events<UiAction>>,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    data: Res<Data>,
) {
    actions.clear();
    let data = data.clone();

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
        data: data.combination1.1.data,
        ..default()
    };
    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone_weak());
    let combination1 = (data.combination1.0, image_handle, data.combination1.2.data);

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
        data: data.combination2.1.data,
        ..default()
    };
    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone_weak());
    let combination2 = (data.combination2.0, image_handle, data.combination2.2.data);

    commands.insert_resource(Context {
        duration: data.duration,
        combination1,
        combination2,
    });
}

fn show_ui(
    mut ui_ctx: Query<&mut EguiContext>,
    mut actions: EventWriter<UiAction>,
    images: Res<EguiUserTextures>,
    ctx: ResMut<Context>,
) {
    let mut ui_ctx = ui_ctx.single_mut();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.label("Vote");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let image_id = images.image_id(&ctx.combination1.1).unwrap();
                ui.image(egui::load::SizedTexture::new(
                    image_id,
                    egui::vec2(300., 300.),
                ));
                ui.label(&ctx.combination1.2);
                if ui.button("Vote").clicked() {
                    actions.send(UiAction::Vote1);
                }
            });
            ui.vertical(|ui| {
                let image_id = images.image_id(&ctx.combination2.1).unwrap();
                ui.image(egui::load::SizedTexture::new(
                    image_id,
                    egui::vec2(300., 300.),
                ));
                ui.label(&ctx.combination2.2);
                if ui.button("Vote").clicked() {
                    actions.send(UiAction::Vote2);
                }
            });
        });
    });
}

fn execute_actions(
    mut actions: ResMut<Events<UiAction>>,
    mut client: ResMut<QuinnetClient>,
    ctx: ResMut<Context>,
) {
    for action in actions.drain() {
        match action {
            UiAction::Vote1 => {
                client
                    .connection_mut()
                    .send_message(
                        ClientMsgComm::SubmitVote(Vote {
                            combination: ctx.combination2.0,
                        })
                        .root(),
                    )
                    .ok();
            }
            UiAction::Vote2 => {
                client
                    .connection_mut()
                    .send_message(
                        ClientMsgComm::SubmitVote(Vote {
                            combination: ctx.combination2.0,
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
