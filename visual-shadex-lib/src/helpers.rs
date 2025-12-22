use egui::{Align, Color32, FontId, Painter, Pos2, epaint::TextShape, text::LayoutJob, vec2};

pub fn draw_text(
    painter: &Painter,
    text: String,
    pos: Pos2,
    font_size: f32,
    halign: Align,
    valign: Align,
) -> TextShape {
    let mut job =
        LayoutJob::simple_singleline(text, FontId::proportional(font_size), Color32::WHITE);
    job.halign = halign;
    let galley = painter.layout_job(job);
    let rect = galley.rect;
    TextShape::new(
        pos - vec2(0f32, rect.bottom() * valign.to_factor()),
        galley,
        Color32::WHITE,
    )
}
