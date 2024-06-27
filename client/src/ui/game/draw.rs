use std::time::Duration;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer,
            ImageDataLayout, MapMode, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::RenderLayers,
        Render, RenderApp,
    },
    tasks::AsyncComputeTaskPool,
};
use bevy_egui::{EguiContext, EguiUserTextures};
use bevy_quinnet::client::QuinnetClient;
use common::{game::Drawing, protocol::ClientMsgComm};
use crossbeam_channel::{Receiver, Sender};
use egui::Stroke;

use crate::{states::GameState, ui::widgets::root_element};

const BRUSH_SIZES: [f32; 5] = [3.0, 5.0, 13.0, 21.0, 43.0];
const BRUSH_COLORS: [egui::Color32; 10] = [
    egui::Color32::WHITE,
    egui::Color32::RED,
    egui::Color32::GREEN,
    egui::Color32::BLACK,
    egui::Color32::WHITE,
    egui::Color32::GOLD,
    egui::Color32::GRAY,
    egui::Color32::WHITE,
    egui::Color32::WHITE,
    egui::Color32::WHITE,
];
const IMG_HALF_SIZE: f32 = 256.0;
const IMG_SIZE: f32 = 2.0 * IMG_HALF_SIZE;
const IMG_PADDING_HALF_SIZE: f32 = 8.0;
const IMG_PADDING: f32 = 2.0 * IMG_PADDING_HALF_SIZE;

#[derive(Event)]
pub struct DrawData {
    pub duration: Duration,
}

#[derive(Resource)]
pub struct DrawContext {
    pub duration: Duration,
    pub image_handle: Handle<Image>,
    pub last_pos: Option<Vec2>,
    pub brush_size: f32,
    pub brush_color: egui::Color32,
}

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut resize: EventWriter<UpdateBrush>,
    mut data: ResMut<Events<DrawData>>,
) {
    let data = data.drain().last().unwrap();
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
    egui_user_textures.add_image(image_handle.clone_weak());
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
    commands.insert_resource(DrawContext {
        duration: data.duration,
        image_handle,
        last_pos: None,
        brush_size: BRUSH_SIZES[2],
        brush_color: BRUSH_COLORS[0],
    });
    resize.send(UpdateBrush::Size(BRUSH_SIZES[2]));
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<DrawContext>();
}

pub fn update(
    mut ctx: Query<&mut EguiContext>,
    images: Res<EguiUserTextures>,
    mut draw_ctx: ResMut<DrawContext>,
    gizmos: Gizmos,
    window: Query<&Window>,
    mut update_brush: EventWriter<UpdateBrush>,
    comm: ResMut<MainWorldComm>,
) {
    let mut ctx = ctx.single_mut();
    let window = window.single();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Draw");

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let eraser = ui.button("Eraser").clicked();
                if eraser {
                    update_brush.send(UpdateBrush::Color(egui::Color32::TRANSPARENT));
                }
                egui::Grid::new("colors").num_columns(2).show(ui, |ui| {
                    for (i, color) in BRUSH_COLORS.into_iter().enumerate() {
                        let (rect, resp) =
                            ui.allocate_exact_size(egui::Vec2::splat(10.0), egui::Sense::click());
                        let painter = ui.painter_at(rect);
                        painter.rect_filled(rect, 0.0, color);
                        if resp.clicked() {
                            update_brush.send(UpdateBrush::Color(color));
                        }
                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
            });
            show_canvas(ui, &images, &mut draw_ctx, window, gizmos);
        });

        show_brushes(ui, &mut draw_ctx, update_brush);

        let submit = ui.button("Submit").clicked();
        if submit {
            comm.sender.try_send(draw_ctx.image_handle.clone()).ok();
        }
    });
}

pub fn send_image(mut client: ResMut<QuinnetClient>, comm: ResMut<MainWorldComm>) {
    let Some(data) = comm.receiver.try_recv().ok() else {
        return;
    };

    client
        .connection_mut()
        .send_message(ClientMsgComm::SubmitDrawing(Drawing { data }).root())
        .ok();
}

fn show_brushes(
    ui: &mut egui::Ui,
    draw_ctx: &mut DrawContext,
    mut update_brush: EventWriter<UpdateBrush>,
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
                    update_brush.send(UpdateBrush::Size(size));
                }
            }
        });
}

fn show_canvas(
    ui: &mut egui::Ui,
    images: &EguiUserTextures,
    draw_ctx: &mut DrawContext,
    window: &Window,
    mut gizmos: Gizmos<DefaultGizmoConfigGroup, ()>,
) {
    let (padded_rect, resp) = ui.allocate_exact_size(
        egui::Vec2::splat(IMG_SIZE + 2.0 * IMG_PADDING),
        egui::Sense::click_and_drag(),
    );

    let image_id = images.image_id(&draw_ctx.image_handle).unwrap();
    let painter = ui.painter_at(padded_rect);
    painter.rect_stroke(
        padded_rect.shrink(IMG_PADDING_HALF_SIZE),
        0.0,
        egui::Stroke::new(IMG_PADDING, egui::Color32::BLACK),
    );
    let paint = resp.is_pointer_button_down_on();

    let img_rect = padded_rect.shrink(IMG_PADDING);
    ui.allocate_ui_at_rect(img_rect, |ui| {
        ui.image(egui::load::SizedTexture::new(
            image_id,
            egui::Vec2::splat(IMG_SIZE),
        ))
        .is_pointer_button_down_on();
    });

    let rescaler = Rescaler::new(
        img_rect,
        egui::Rect::from_min_max(
            egui::pos2(-IMG_HALF_SIZE, IMG_HALF_SIZE),
            egui::pos2(IMG_HALF_SIZE, -IMG_HALF_SIZE),
        ),
    );

    if let Some(current_pos) = window.cursor_position() {
        let color = draw_ctx.brush_color;
        let color = Color::srgba_u8(color.r(), color.g(), color.b(), color.a());
        if paint {
            let current_pos = rescaler.rescale(current_pos);
            gizmos.ellipse_2d(current_pos, 0.0, Vec2::ONE, color);
            gizmos.ellipse_2d(current_pos, std::f32::consts::PI, Vec2::ONE, color);

            if let Some(last_pos) = draw_ctx.last_pos {
                if let Some(dir) = (current_pos - last_pos).try_normalize() {
                    let side_offset = Vec2::new(dir.y, -dir.x);
                    gizmos.line_2d(last_pos - side_offset, current_pos - side_offset, color);
                    gizmos.line_2d(last_pos + side_offset, current_pos + side_offset, color);
                    draw_ctx.last_pos = Some(current_pos);
                }
            } else {
                draw_ctx.last_pos = Some(current_pos);
            }
        } else if draw_ctx.last_pos.is_some() {
            draw_ctx.last_pos = None;
        }

        let painter = ui.painter_at(padded_rect);
        let current_pos = egui::pos2(current_pos.x, current_pos.y);
        let mut color = draw_ctx.brush_color;
        if color.a() == 0 {
            color = egui::Color32::WHITE;
        }
        painter.circle_stroke(
            current_pos,
            draw_ctx.brush_size / 2.0 + 3.0,
            Stroke::new(1.0, color),
        );
    }

    ui.advance_cursor_after_rect(padded_rect);
}

#[derive(Event)]
pub enum UpdateBrush {
    Size(f32),
    Color(egui::Color32),
}

pub fn resize_brush(
    mut events: EventReader<UpdateBrush>,
    mut gizmo_configs: ResMut<GizmoConfigStore>,
    mut draw_ctx: ResMut<DrawContext>,
) {
    for event in events.read() {
        match event {
            UpdateBrush::Size(size) => {
                let config = gizmo_configs.config_mut::<DefaultGizmoConfigGroup>().0;
                draw_ctx.brush_size = *size;
                config.line_width = size - 1.0;
            }
            UpdateBrush::Color(color) => draw_ctx.brush_color = *color,
        }
    }
}

/// Scales a point from one rectangular space to another.
struct Rescaler {
    source_size: egui::Vec2,
    source_min: egui::Pos2,
    destination_size: egui::Vec2,
    destination_min: egui::Pos2,
}

impl Rescaler {
    pub fn new(source: egui::Rect, destination: egui::Rect) -> Self {
        Self {
            source_size: source.size(),
            source_min: source.min,
            destination_size: destination.size(),
            destination_min: destination.min,
        }
    }

    fn rescale(&self, position: Vec2) -> Vec2 {
        let normalized_position = (
            (position.x - self.source_min.x) / self.source_size.x,
            (position.y - self.source_min.y) / self.source_size.y,
        );

        Vec2::new(
            normalized_position.0 * self.destination_size.x + self.destination_min.x,
            normalized_position.1 * self.destination_size.y + self.destination_min.y,
        )
    }
}

pub struct SaveDrawingPlugin;

impl Plugin for SaveDrawingPlugin {
    fn build(&self, app: &mut App) {
        let (m_s, r_r) = crossbeam_channel::unbounded();
        let (r_s, m_r) = crossbeam_channel::unbounded();

        app.insert_resource(MainWorldComm {
            receiver: m_r,
            sender: m_s,
        });

        let rapp = app.sub_app_mut(RenderApp);
        rapp.insert_resource(RenderWorldComm {
            sender: r_s,
            receiver: r_r,
        });
        rapp.add_systems(Render, save_drawing);
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

pub fn save_drawing(
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
        size: 512 * 512 * 4,
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
                bytes_per_row: Some(512 * 4),
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
        // Send data to main app
        let empty = result.iter().filter(|x| **x == 0).count();
        info!("{}", empty as f32 / (512. * 512. * 4.));
        sender.send(result).ok();
    };

    AsyncComputeTaskPool::get().spawn(finish).detach();
}
