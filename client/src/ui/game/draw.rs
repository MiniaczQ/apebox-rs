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

use crate::{networking::DrawData, states::GameState, ui::widgets::root_element};

#[derive(Resource)]
pub struct DrawContext {
    pub image_handle: Handle<Image>,
}

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
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
    commands.insert_resource(DrawContext { image_handle });
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<DrawContext>();
    commands.remove_resource::<DrawData>();
}

pub fn update(
    mut ctx: Query<&mut EguiContext>,
    images: Res<EguiUserTextures>,
    draw_ctx: Res<DrawContext>,
    mut gizmos: Gizmos,
    mut cursor_events: EventReader<CursorMoved>,
) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        let image_id = images.image_id(&draw_ctx.image_handle).unwrap();
        ui.label("Draw");
        let origin = ui
            .allocate_exact_size(
                egui::Vec2::ZERO,
                egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                },
            )
            .0
            .left_top();
        let origin = Vec2::new(origin.x, origin.y);

        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(512., 512.), egui::Sense::click_and_drag());
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.image(egui::load::SizedTexture::new(
                image_id,
                egui::vec2(512., 512.),
            ))
            .is_pointer_button_down_on();
        });
        let paint = resp.is_pointer_button_down_on();

        // TODO: keep position between frames
        // TODO: allow press without drag
        if paint {
            let line = cursor_events
                .read()
                .map(|e| {
                    let pos = e.position - Vec2::new(origin.x, -origin.y) - Vec2::new(0.0, 300.0);
                    let pos = Vec2::new(pos.x, -pos.y);
                    info!("{} || {} || {}", e.position, origin, pos);
                    pos
                })
                .collect::<Vec<_>>();

            if !line.is_empty() {
                gizmos.circle_2d(*line.first().unwrap(), 5.0, WHITE);
                gizmos.circle_2d(*line.last().unwrap(), 5.0, WHITE);
                gizmos.linestrip_2d(line, WHITE);
            }
        }

        _ = ui.button("Submit");
    });
}
