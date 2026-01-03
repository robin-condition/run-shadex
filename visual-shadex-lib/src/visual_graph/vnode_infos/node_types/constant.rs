use std::rc::Rc;

use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

/*
thread_local! {
    static CONST_TYPE: FallibleNodeTypeRc = Ok(Rc::new(NodeTypeInfo {
            inputs: Vec::new(),
            outputs: vec![OutputInfo {
                name: "value".to_string(),
                value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
            }],
        }));
}
*/

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
        ui.style_mut().spacing.slider_width = 75f32;
        ui.add(
            egui::Slider::new(&mut self.val, 0f32..=1f32)
                .clamping(egui::SliderClamping::Never)
                .logarithmic(false),
        )
        .changed()
    }

    fn get_shadex_type(&self) -> FallibleNodeTypeRc {
        //CONST_TYPE.with(FallibleNodeTypeRc::clone)
        Ok(Rc::new(NodeTypeInfo {
            inputs: Vec::new(),
            outputs: vec![OutputInfo {
                name: None,
                value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
            }],
            annotation: shadex_backend::execution::ExecutionInformation::Constant(self.val),
        }))
    }

    fn get_name(&self) -> &str {
        "Constant"
    }
}
