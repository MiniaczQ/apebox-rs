pub fn root_element<R>(
    ui: &mut egui::Context,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<egui::InnerResponse<egui::InnerResponse<R>>> {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.centered_and_justified(|ui| ui.vertical_centered(add_contents))
    })
}

pub fn validated_singleline_textbox(
    ui: &mut egui::Ui,
    is_valid: bool,
    buffer: &mut dyn egui::TextBuffer,
) -> egui::Response {
    ui.add(egui::TextEdit::singleline(buffer).text_color(if is_valid {
        egui::Color32::WHITE
    } else {
        egui::Color32::RED
    }))
}
