use ::bevy::prelude::*;

/// Scales a point from one rectangular space to another.
pub struct Scaler {
    source_size: egui::Vec2,
    source_min: egui::Pos2,
    destination_size: egui::Vec2,
    destination_min: egui::Pos2,
}

impl Scaler {
    pub fn new(source: egui::Rect, destination: egui::Rect) -> Self {
        Self {
            source_size: source.size(),
            source_min: source.min,
            destination_size: destination.size(),
            destination_min: destination.min,
        }
    }

    pub fn scale(&self, position: Vec2) -> Vec2 {
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
