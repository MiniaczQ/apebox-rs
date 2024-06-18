use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::{networking::VoteData, ui::widgets::root_element};

pub fn update(mut ctx: Query<&mut EguiContext>) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Vote");
        _ = ui.button("Submit");
    });
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<VoteData>();
}
