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

type Combination = (Index, (Handle<Image>, egui::Color32), Prompt);

#[derive(Resource)]
pub struct Context {
    pub duration: Duration,
    pub combination1: Combination,
    pub combination2: Combination,
    pub shirt: Handle<Image>,
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
    asset_server: Res<AssetServer>,
    data: Res<Data>,
) {
    actions.clear();
    let data = data.clone();

    let shirt: Handle<Image> = asset_server.load("textures/shirts/shirt1.png");
    egui_user_textures.add_image(shirt.clone_weak());

    let combination1 = prep_combination(&mut images, &mut egui_user_textures, data.combination1);
    let combination2 = prep_combination(&mut images, &mut egui_user_textures, data.combination2);

    commands.insert_resource(Context {
        duration: data.duration,
        combination1,
        combination2,
        shirt,
    });
}

fn prep_combination(
    images: &mut Assets<Image>,
    egui_user_textures: &mut EguiUserTextures,
    combination: (Index, Drawing, Prompt),
) -> (Index, (Handle<Image>, egui::Color32), Prompt) {
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
        data: combination.1.drawing,
        ..default()
    };
    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone_weak());
    let bg_color = combination.1.bg_color;
    let bg_color = egui::Color32::from_rgb(bg_color[0], bg_color[1], bg_color[2]);
    (combination.0, (image_handle, bg_color), combination.2)
}

fn show_ui(
    mut ui_ctx: Query<&mut EguiContext>,
    mut actions: EventWriter<UiAction>,
    images: Res<EguiUserTextures>,
    ctx: Res<Context>,
) {
    let mut ui_ctx = ui_ctx.single_mut();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.label("Vote");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                show_vote_option(
                    ui,
                    &mut actions,
                    &images,
                    &ctx.combination1,
                    &ctx.shirt,
                    ctx.combination1.1 .1,
                    UiAction::Vote1,
                );
            });
            ui.vertical(|ui| {
                show_vote_option(
                    ui,
                    &mut actions,
                    &images,
                    &ctx.combination2,
                    &ctx.shirt,
                    ctx.combination2.1 .1,
                    UiAction::Vote2,
                );
            });
        });
    });
}

fn show_vote_option(
    ui: &mut egui::Ui,
    actions: &mut EventWriter<UiAction>,
    images: &EguiUserTextures,
    combination: &Combination,
    shirt: &Handle<Image>,
    shirt_tint: egui::Color32,
    action: UiAction,
) {
    let shirt = images.image_id(shirt).unwrap();
    let drawing = images.image_id(&combination.1 .0).unwrap();

    let size = egui::vec2(512., 512.);
    let (rect, _) = ui.allocate_exact_size(
        size,
        egui::Sense {
            click: false,
            drag: false,
            focusable: false,
        },
    );
    ui.allocate_ui_at_rect(rect, |ui| {
        let image =
            egui::Image::from_texture(egui::load::SizedTexture::new(shirt, egui::vec2(512., 512.)))
                .tint(shirt_tint);
        ui.add(image);
    });
    ui.allocate_ui_at_rect(rect.shrink(128.0), |ui| {
        let image = egui::Image::from_texture(egui::load::SizedTexture::new(
            drawing,
            egui::vec2(256., 256.),
        ));
        ui.add(image);
    });
    ui.allocate_ui_at_rect(
        rect.shrink2(egui::vec2(128.0, 224.0) + egui::vec2(0.0, 160.0)),
        |ui| {
            ui.centered_and_justified(|ui| {
                let label = egui::Label::new(
                    RichText::new(&combination.2.text)
                        .font(combination.2.font.get_font_id())
                        .color(egui::Color32::WHITE),
                );
                ui.add(label);
            });
        },
    );
    ui.advance_cursor_after_rect(rect);
    if ui.button("Vote").clicked() {
        actions.send(action);
    }
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
