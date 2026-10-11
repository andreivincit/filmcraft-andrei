//! Plain-text identity for the personal fork; no upstream trademark artwork is embedded.

/// Draw the fork name and return its rectangle, preserving the upstream layout contract.
pub fn paint_wordmark(ui: &egui::Ui, left_center: egui::Pos2, height: f32, _dark: bool) -> Option<egui::Rect> {
    Some(ui.painter().text(left_center, egui::Align2::LEFT_CENTER, "FilmCraft Andrei", egui::FontId::proportional(height), ui.visuals().text_color()))
}
