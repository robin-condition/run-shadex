use egui::{
    Align, Color32, FontId, Painter, Pos2, Shape,
    epaint::{PathShape, PathStroke, TextShape},
    text::LayoutJob,
    vec2,
};

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

// From earlier nodes-test

// Derived myself :)
// From:
// f(0)=0, f(1)=1
// f'(0)=f'(1)=0
// f''(0)=f''(1)=0
// Then I googled it and found Knuth proposed it apparently
fn smoother_step(t: f32) -> f32 {
    ((6f32 * t - 15f32) * t + 10f32) * t.powi(3)
}

pub(crate) fn draw_line(start_pt: Pos2, end_pt: Pos2, steps: usize) -> Shape {
    let dist = end_pt - start_pt;
    let steps = steps + 2;
    let pts: Vec<Pos2> = (0..=steps)
        .into_iter()
        .map(|p| {
            let t = p as f32 / steps as f32;
            let smooth_t = smoother_step(t);

            // fancy interpolations I s/like very much :)/SPENT WAY TOO LONG ON

            // Handpicked coefficients
            let k = if dist.x < 0f32 {
                -2.6f32 * dist.x
            } else {
                1.3 * dist.x
            };
            let v = Pos2 {
                x: smooth_t * (dist.x - k) + k * t + start_pt.x,
                y: smooth_t * dist.y + start_pt.y,
            };
            v
        })
        .collect();

    let path = PathShape {
        points: pts,
        closed: false,
        fill: Color32::TRANSPARENT,
        stroke: PathStroke {
            width: 3f32,
            color: egui::epaint::ColorMode::Solid(Color32::WHITE),
            kind: egui::StrokeKind::Middle,
        },
    };

    path.into()
}
