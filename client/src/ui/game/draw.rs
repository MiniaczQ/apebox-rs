use bevy::{
    color::palettes::css::WHITE,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_egui::{EguiContext, EguiUserTextures};
use egui::Stroke;

use crate::{networking::DrawData, states::GameState, ui::widgets::root_element};

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

#[derive(Resource)]
pub struct DrawContext {
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
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let mut image = Image {
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
        ..default()
    };
    image.resize(size);
    image.data.fill(255);
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
        image_handle,
        last_pos: None,
        brush_size: BRUSH_SIZES[2],
        brush_color: BRUSH_COLORS[0],
    });
    resize.send(UpdateBrush::Size(BRUSH_SIZES[2]));
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<DrawContext>();
    commands.remove_resource::<DrawData>();
}

pub fn update(
    mut ctx: Query<&mut EguiContext>,
    images: Res<EguiUserTextures>,
    mut draw_ctx: ResMut<DrawContext>,
    gizmos: Gizmos,
    window: Query<&Window>,
    mut update_brush: EventWriter<UpdateBrush>,
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

        show_brushes(ui, draw_ctx, update_brush);

        _ = ui.button("Submit");
    });
}

fn show_brushes(
    ui: &mut egui::Ui,
    draw_ctx: ResMut<DrawContext>,
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
