use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use common::game::{Drawing, Prompt};

use crate::ui::widgets::root_element;

#[derive(Event)]
pub struct VoteData {
    pub duration: Duration,
    pub combination1: (Drawing, Prompt),
    pub combination2: (Drawing, Prompt),
}

pub fn update(mut ctx: Query<&mut EguiContext>) {
    let mut ctx = ctx.single_mut();

    root_element(ctx.get_mut(), |ui| {
        ui.label("Vote");
        _ = ui.button("Submit");
    });
}

pub fn teardown(mut commands: Commands) {}
