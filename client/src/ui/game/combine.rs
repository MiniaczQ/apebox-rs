use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::{networking::CombineData, ui::widgets::root_element};

pub fn update(mut ctx: Query<&mut EguiContext>) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Combine");
        egui::Grid::new("nav-buttons")
            .num_columns(3)
            .show(ui, |ui| {
                _ = ui.button("<");
                //ui.image(egui::load::SizedTexture::new(
                //    image_id,
                //    egui::vec2(300., 300.),
                //));
                _ = ui.button(">");
                ui.end_row();

                _ = ui.button("<");
                ui.set_enabled(false);
                _ = ui.button("lorem ipsum");
                ui.set_enabled(true);
                _ = ui.button(">");
                ui.end_row();
            });
        _ = ui.button("Submit");
    });
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<CombineData>();
}
