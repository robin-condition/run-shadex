use egui::{
    Color32, FontId, InnerResponse, Label, Rect, RichText, Sense, Separator, Shape, Stroke, Vec2,
    epaint::RectShape, layers::PaintList, vec2,
};
use shadex_backend::nodegraph::{
    NodeTypeInfo, NodeTypeRef, OutputInfo, PrimitiveType, TypeUniverse, ValueType,
};

use crate::visual_graph::VNodeOutputRef;

pub trait VisualNodeInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool;
    fn get_shadex_type(&self) -> NodeTypeInfo;
    fn get_name(&self) -> &str;
}

pub struct VisualNode {
    pub data: Box<dyn VisualNodeInfo>,
    pub position: Vec2,
    pub input_sources: Vec<Option<VNodeOutputRef>>
}

impl VisualNode {
    pub fn show_box(&mut self, ui: &mut egui::Ui) -> InnerResponse<bool> {
        let idx = ui.painter().add(Shape::Noop);
        let changed = &mut false;

        let inner_resp = ui.scope_builder(
            egui::UiBuilder::new()
                .sense(Sense::drag())
                .max_rect(Rect::from_min_size(self.position.to_pos2(), Vec2::INFINITY)),
            |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5f32);
                    ui.horizontal(|ui| {
                        ui.add_space(10f32);
                        // Find out how to make title label centered and/or distinguishable.

                        ui.add(
                            Label::new(
                                RichText::new(self.data.get_name())
                                    .font(FontId::proportional(15f32))
                                    .underline(),
                            )
                            .selectable(false),
                        );
                    });

                    ui.horizontal_top(|ui| {
                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            // Do input ports
                            // TODO: Ports selectable.
                            ui.label("Port 1 test");
                        });
                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            *changed = self.data.show(ui);
                        });
                        ui.add_space(5f32);
                        ui.vertical(|ui| {
                            // Do output ports
                            // TODO: Ports selectable.
                            ui.horizontal(|ui| {
                                let label_resp = ui.label("Output port 1 test");
                                let (resp, painter) = ui.allocate_painter(
                                    vec2(label_resp.rect.height(), label_resp.rect.height()),
                                    Sense::hover(),
                                );
                                painter.circle(
                                    resp.rect.center(),
                                    label_resp.rect.height() / 2.0f32,
                                    Color32::RED,
                                    Stroke::default(),
                                );
                            });
                        });
                    });
                });
            },
        );

        let initial_rect = inner_resp.response.rect;

        ui.painter().set(
            idx,
            RectShape::filled(initial_rect, 10.0f32, Color32::BLACK),
        );

        let response = inner_resp.response;

        if response.dragged() {
            let delt = response.drag_delta();
            self.position += delt;
        }

        //helpers::draw_text(painter, text, pos, font_size, halign, valign).galley.rect.width();

        InnerResponse {
            inner: *changed,
            response: response,
        }
    }
}

pub struct ConstantInfo {
    pub val: f32,
}
impl ConstantInfo {
    pub fn new(val: f32) -> Self {
        Self { val }
    }
}

impl VisualNodeInfo for ConstantInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        ui.add(
            egui::Slider::new(&mut self.val, 0.01f32..=100f32)
                .clamping(egui::SliderClamping::Never)
                .logarithmic(true),
        )
        .changed()
    }

    fn get_shadex_type(&self) -> NodeTypeInfo {
        NodeTypeInfo {
            name: "test".to_string(),
            inputs: Vec::new(),
            outputs: vec![OutputInfo {
                name: "value".to_string(),
                value_type: Box::new(ValueType::primitive(PrimitiveType::F32)),
            }],
        }
    }

    fn get_name(&self) -> &str {
        "Constant"
    }
}
