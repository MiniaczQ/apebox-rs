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

pub struct VotePlugin;

impl Plugin for VotePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<VoteData>();
        app.add_reentrant_statebound(
            GameState::Vote,
            setup,
            teardown,
            update.in_set(GameSystemOdering::StateLogic),
        );
    }
}

#[derive(Event)]
pub struct VoteData {
    pub duration: Duration,
    pub combination1: (Index, Drawing, Prompt),
    pub combination2: (Index, Drawing, Prompt),
}

#[derive(Resource)]
pub struct VoteContext {
    pub duration: Duration,
    pub combination1: (Index, Handle<Image>, String),
    pub combination2: (Index, Handle<Image>, String),
}

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut data: ResMut<Events<VoteData>>,
) {
    let data = data.drain().last().unwrap();

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

    commands.insert_resource(VoteContext {
        duration: data.duration,
        combination1,
        combination2,
    });
}

pub fn update(
    mut ctx: Query<&mut EguiContext>,
    images: Res<EguiUserTextures>,
    vote_ctx: ResMut<VoteContext>,
    mut client: ResMut<QuinnetClient>,
) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Vote");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let image_id = images.image_id(&vote_ctx.combination1.1).unwrap();
                ui.image(egui::load::SizedTexture::new(
                    image_id,
                    egui::vec2(300., 300.),
                ));
                ui.label(&vote_ctx.combination1.2);
                let submit_vote = ui.button("Vote").clicked();
                if submit_vote {
                    client
                        .connection_mut()
                        .send_message(
                            ClientMsgComm::SubmitVote(Vote {
                                combination: vote_ctx.combination1.0,
                            })
                            .root(),
                        )
                        .ok();
                }
            });
            ui.vertical(|ui| {
                let image_id = images.image_id(&vote_ctx.combination2.1).unwrap();
                ui.image(egui::load::SizedTexture::new(
                    image_id,
                    egui::vec2(300., 300.),
                ));
                ui.label(&vote_ctx.combination2.2);
                let submit_vote = ui.button("Vote").clicked();
                if submit_vote {
                    client
                        .connection_mut()
                        .send_message(
                            ClientMsgComm::SubmitVote(Vote {
                                combination: vote_ctx.combination2.0,
                            })
                            .root(),
                        )
                        .ok();
                }
            });
        });
    });
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<VoteContext>();
}
