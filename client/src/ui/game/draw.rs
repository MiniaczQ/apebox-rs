use std::time::Duration;

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_egui::{EguiContext, EguiUserTextures};
use bevy_quinnet::client::QuinnetClient;
use common::{app::AppExt, game::Drawing, protocol::ClientMsgComm};
use egui::Stroke;

use crate::{
    states::GameState,
    ui::{util::Scaler, widgets::root_element},
    GameSystemOdering,
};

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiAction>();
        app.add_plugins(save_image::SaveDrawingPlugin);
        app.insert_gizmo_config(
            DefaultGizmoConfigGroup,
            GizmoConfig {
                render_layers: RenderLayers::layer(1),
                line_joints: GizmoLineJoint::Round(32),
                ..default()
            },
        );
        app.add_reentrant_statebound(
            GameState::Draw,
            setup,
            teardown,
            (execute_actions, show_ui, send_image)
                .chain()
                .in_set(GameSystemOdering::StateLogic),
        );
    }
}

const BRUSH_SIZES: [f32; 5] = [3.0, 5.0, 13.0, 21.0, 43.0];
const BRUSH_COLORS: [egui::Color32; 10] = [
    egui::Color32::BLACK,
    egui::Color32::GRAY,
    egui::Color32::WHITE,
    egui::Color32::RED,
    egui::Color32::DARK_RED,
    egui::Color32::GREEN,
    egui::Color32::DARK_GREEN,
    egui::Color32::BLUE,
    egui::Color32::DARK_BLUE,
    egui::Color32::GOLD,
];
const IMG_HALF_SIZE: f32 = 256.0;
const IMG_SIZE: f32 = 2.0 * IMG_HALF_SIZE;
const IMG_PADDING_HALF_SIZE: f32 = 8.0;
const IMG_PADDING: f32 = 2.0 * IMG_PADDING_HALF_SIZE;

#[derive(Resource, Clone)]
pub struct Data {
    pub duration: Duration,
}

#[derive(Resource)]
pub struct Context {
    pub duration: Duration,
    pub image_handle: Handle<Image>,
    pub last_pos: Option<Vec2>,
    pub brush_size: f32,
    pub brush_color: egui::Color32,
    pub bg_color: egui::Color32,
}

#[derive(Event)]
pub enum UiAction {
    BrushSize(f32),
    BrushColor(egui::Color32),
    ShirtColor(egui::Color32),
    Submit,
}

fn setup(
    mut commands: Commands,
    mut actions: ResMut<Events<UiAction>>,
    mut images: ResMut<Assets<Image>>,
    mut textures: ResMut<EguiUserTextures>,
    data: Res<Data>,
) {
    actions.clear();
    let data = data.clone();

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("drawing"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);
    textures.add_image(image_handle.clone_weak());
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                target: image_handle.clone().into(),
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
        StateScoped(GameState::Draw),
    ));
    commands.insert_resource(Context {
        duration: data.duration,
        image_handle,
        last_pos: None,
        brush_size: BRUSH_SIZES[2],
        brush_color: BRUSH_COLORS[0],
        bg_color: BRUSH_COLORS[2],
    });
    actions.send(UiAction::BrushSize(BRUSH_SIZES[2]));
}

fn teardown(mut commands: Commands, mut actions: ResMut<Events<UiAction>>) {
    commands.remove_resource::<Data>();
    commands.remove_resource::<Context>();
    actions.clear();
}

fn show_ui(
    mut ui_ctx: Query<&mut EguiContext>,
    mut ctx: ResMut<Context>,
    mut actions: EventWriter<UiAction>,
    mut gizmos: Gizmos,
    window: Query<&Window>,
    images: Res<EguiUserTextures>,
) {
    let mut ui_ctx = ui_ctx.single_mut();
    let window = window.single();

    root_element(ui_ctx.get_mut(), |ui| {
        ui.label("Draw");

        ui.horizontal(|ui| {
            show_brush_colors(ui, &mut actions);
            show_canvas(ui, &mut ctx, &mut gizmos, &images, window);
            show_shirt_colors(ui, &mut actions);
        });

        show_brushes(ui, &mut ctx, &mut actions);

        if ui.button("Submit").clicked() {
            actions.send(UiAction::Submit);
        }
    });
}

fn show_brush_colors(ui: &mut egui::Ui, actions: &mut EventWriter<UiAction>) {
    ui.vertical(|ui| {
        let eraser = ui.button("Eraser").clicked();
        if eraser {
            actions.send(UiAction::BrushColor(egui::Color32::TRANSPARENT));
        }
        egui::Grid::new("brush-colors")
            .num_columns(2)
            .show(ui, |ui| {
                for (i, color) in BRUSH_COLORS.into_iter().enumerate() {
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::Vec2::splat(10.0), egui::Sense::click());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, color);
                    if resp.clicked() {
                        actions.send(UiAction::BrushColor(color));
                    }
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    });
}

fn show_shirt_colors(ui: &mut egui::Ui, actions: &mut EventWriter<UiAction>) {
    ui.vertical(|ui| {
        ui.label("Shirt color");
        egui::Grid::new("shirt-colors")
            .num_columns(2)
            .show(ui, |ui| {
                for (i, color) in BRUSH_COLORS.into_iter().enumerate() {
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::Vec2::splat(10.0), egui::Sense::click());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, color);
                    if resp.clicked() {
                        actions.send(UiAction::ShirtColor(color));
                    }
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    });
}

fn send_image(
    mut client: ResMut<QuinnetClient>,
    comm: Res<save_image::MainWorldComm>,
    ctx: Res<Context>,
) {
    let Some(drawing) = comm.receiver.try_recv().ok() else {
        return;
    };
    let bg_color = [ctx.bg_color.r(), ctx.bg_color.g(), ctx.bg_color.b()];

    client
        .connection_mut()
        .send_message(ClientMsgComm::SubmitDrawing(Drawing { drawing, bg_color }).root())
        .ok();
}

fn show_brushes(
    ui: &mut egui::Ui,
    draw_ctx: &mut Context,
    update_brush: &mut EventWriter<UiAction>,
) {
    egui::Grid::new("brush-sizes")
        .num_columns(BRUSH_SIZES.len())
        .show(ui, |ui| {
            for size in BRUSH_SIZES {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::Vec2::splat(43.0), egui::Sense::click());
                let painter = ui.painter_at(rect);
                let color = if draw_ctx.brush_size == size {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::DARK_GRAY
                };
                painter.circle_filled(rect.center(), size / 2.0, color);
                if resp.clicked() {
                    update_brush.send(UiAction::BrushSize(size));
                }
            }
        });
}

fn show_canvas(
    ui: &mut egui::Ui,
    ctx: &mut Context,
    gizmos: &mut Gizmos<DefaultGizmoConfigGroup, ()>,
    images: &EguiUserTextures,
    window: &Window,
) {
    let (padded_rect, resp) = ui.allocate_exact_size(
        egui::Vec2::splat(IMG_SIZE + 2.0 * IMG_PADDING),
        egui::Sense::click_and_drag(),
    );

    let image_id = images.image_id(&ctx.image_handle).unwrap();
    let painter = ui.painter_at(padded_rect);
    painter.rect_filled(padded_rect, 0.0, ctx.bg_color);
    let paint = resp.is_pointer_button_down_on();

    let img_rect = padded_rect.shrink(IMG_PADDING);
    ui.allocate_ui_at_rect(img_rect, |ui| {
        ui.image(egui::load::SizedTexture::new(
            image_id,
            egui::Vec2::splat(IMG_SIZE),
        ))
        .is_pointer_button_down_on();
    });

    let rescaler = Scaler::new(
        img_rect,
        egui::Rect::from_min_max(
            egui::pos2(-IMG_HALF_SIZE, IMG_HALF_SIZE),
            egui::pos2(IMG_HALF_SIZE, -IMG_HALF_SIZE),
        ),
    );

    if let Some(current_pos) = window.cursor_position() {
        let color = ctx.brush_color;
        let color = Color::srgba_u8(color.r(), color.g(), color.b(), color.a());
        if paint {
            let current_pos = rescaler.scale(current_pos);
            gizmos.ellipse_2d(current_pos, 0.0, Vec2::ONE, color);
            gizmos.ellipse_2d(current_pos, std::f32::consts::PI, Vec2::ONE, color);

            if let Some(last_pos) = ctx.last_pos {
                if let Some(dir) = (current_pos - last_pos).try_normalize() {
                    let side_offset = Vec2::new(dir.y, -dir.x);
                    gizmos.line_2d(last_pos - side_offset, current_pos - side_offset, color);
                    gizmos.line_2d(last_pos + side_offset, current_pos + side_offset, color);
                    ctx.last_pos = Some(current_pos);
                }
            } else {
                ctx.last_pos = Some(current_pos);
            }
        } else if ctx.last_pos.is_some() {
            ctx.last_pos = None;
        }

        let painter = ui.painter_at(padded_rect);
        let current_pos = egui::pos2(current_pos.x, current_pos.y);
        let mut color = ctx.brush_color;
        if color.a() == 0 {
            color = egui::Color32::WHITE;
        }
        painter.circle_stroke(
            current_pos,
            ctx.brush_size / 2.0 + 3.0,
            Stroke::new(1.0, color),
        );
    }

    ui.advance_cursor_after_rect(padded_rect);
}

fn execute_actions(
    mut ctx: ResMut<Context>,
    mut actions: EventReader<UiAction>,
    mut gizmo_configs: ResMut<GizmoConfigStore>,
    comm: ResMut<save_image::MainWorldComm>,
) {
    for action in actions.read() {
        match action {
            UiAction::BrushSize(size) => {
                let config = gizmo_configs.config_mut::<DefaultGizmoConfigGroup>().0;
                ctx.brush_size = *size;
                config.line_width = size - 1.0;
            }
            UiAction::BrushColor(color) => ctx.brush_color = *color,
            UiAction::ShirtColor(color) => ctx.bg_color = *color,
            UiAction::Submit => {
                comm.sender.try_send(ctx.image_handle.clone()).ok();
            }
        }
    }
}

mod save_image {
    use super::IMG_SIZE;

    use bevy::{
        prelude::*,
        render::{
            {
                render_asset::RenderAssets,
                render_resource::{
                    BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d,
                    ImageCopyBuffer, ImageDataLayout, MapMode,
                },
            },
            {
                renderer::{RenderDevice, RenderQueue},
                texture::GpuImage,
            },
            {Render, RenderApp},
        },
        tasks::AsyncComputeTaskPool,
    };
    use crossbeam_channel::{Receiver, Sender};

    pub struct SaveDrawingPlugin;

    impl Plugin for SaveDrawingPlugin {
        fn build(&self, app: &mut App) {
            let (m_s, r_r) = crossbeam_channel::unbounded();
            let (r_s, m_r) = crossbeam_channel::unbounded();

            app.insert_resource(MainWorldComm {
                receiver: m_r,
                sender: m_s,
            });

            let render_app = app.sub_app_mut(RenderApp);
            render_app.insert_resource(RenderWorldComm {
                sender: r_s,
                receiver: r_r,
            });
            render_app.add_systems(Render, save_drawing);
        }
    }

    #[derive(Resource)]
    pub struct MainWorldComm {
        pub receiver: Receiver<Vec<u8>>,
        pub sender: Sender<Handle<Image>>,
    }

    #[derive(Resource)]
    pub struct RenderWorldComm {
        pub sender: Sender<Vec<u8>>,
        pub receiver: Receiver<Handle<Image>>,
    }

    pub(crate) fn save_drawing(
        device: Res<RenderDevice>,
        queue: Res<RenderQueue>,
        assets: Res<RenderAssets<GpuImage>>,
        comm: Res<RenderWorldComm>,
    ) {
        // Wait for request
        let Some(handle) = comm.receiver.try_recv().ok() else {
            return;
        };

        // Prepare destination buffer, hardcoded image size of 512x512 in Bgra8UnormSrgb
        let image = assets.get(&handle).expect("Missing asset");
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("drawing-transfer-buffer"),
            size: IMG_SIZE as u64 * IMG_SIZE as u64 * 4,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Request copy
        encoder.copy_texture_to_buffer(
            image.texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(IMG_SIZE as u32 * 4),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: image.size.x,
                height: image.size.y,
                ..default()
            },
        );
        queue.submit([encoder.finish()]);

        // Wait for copy to finish
        let sender = comm.sender.clone();
        let finish = async move {
            let (tx, rx) = async_channel::bounded(1);
            // Copy data from GPU to CPU
            let buffer_slice = buffer.slice(..);
            buffer_slice.map_async(MapMode::Read, move |result| {
                let err = result.err();
                if err.is_some() {
                    panic!("{}", err.unwrap().to_string());
                }
                tx.try_send(()).unwrap();
            });
            rx.recv().await.unwrap();
            let data = buffer_slice.get_mapped_range();
            let result = Vec::from(&*data);
            drop(data);
            drop(buffer);
            sender.send(result).ok();
        };

        AsyncComputeTaskPool::get().spawn(finish).detach();
    }
}
