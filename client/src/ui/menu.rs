use std::net::SocketAddr;

use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::{states::MenuState, ConnectionData};

use super::widgets::{root_element, validated_singleline_textbox};

pub fn show(
    mut egui_ctx: Query<&mut EguiContext>,
    mut data: ResMut<ConnectionData>,
    mut next: ResMut<NextState<MenuState>>,
    state: Res<State<MenuState>>,
) {
    let mut ctx = egui_ctx.single_mut();

    let address_is_valid = data.address.parse::<SocketAddr>().is_ok();

    root_element(ctx.get_mut(), |ui| {
        ui.set_enabled(*state.get() == MenuState::Configuring);

        egui::Grid::new("Main Menu Grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut data.name)
                    .on_hover_text("Your username.");
                ui.end_row();

                ui.label("Address:");
                validated_singleline_textbox(ui, address_is_valid, &mut data.address)
                    .on_hover_text("Valid IP address and port.");
                ui.end_row();
            });

        let connect = ui.button("Connect").clicked();
        if connect {
            next.set(MenuState::Connecting);
        }
    });
}
