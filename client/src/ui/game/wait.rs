use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::ui::widgets::root_element;

pub fn update(mut ctx: Query<&mut EguiContext>) {
    let mut ctx = ctx.single_mut();
    root_element(ctx.get_mut(), |ui| ui.label("Waiting for game to start."));
}
