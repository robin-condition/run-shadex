use shadex_backend::nodegraph::{NodeTypeInfo, OutputInfo, PrimitiveType, ValueType};

use crate::visual_graph::VisualNodeInfo;

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
