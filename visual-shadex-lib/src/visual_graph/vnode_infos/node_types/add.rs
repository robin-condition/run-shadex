use shadex_backend::{
    execution::typechecking::typetypes::{PrimitiveType, ValueType},
    nodegraph::{InputInfo, NodeTypeInfo, OutputInfo},
};

use crate::visual_graph::VisualNodeInfo;

pub struct AddInfo {}
impl AddInfo {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisualNodeInfo for AddInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        false
    }

    fn get_shadex_type(&self) -> NodeTypeInfo {
        NodeTypeInfo {
            name: "add".to_string(),
            inputs: vec![
                InputInfo {
                    name: "a".to_string(),
                    value_type: Box::new(ValueType::primitive(PrimitiveType::F32)),
                },
                InputInfo {
                    name: "b".to_string(),
                    value_type: Box::new(ValueType::primitive(PrimitiveType::F32)),
                },
            ],
            outputs: vec![OutputInfo {
                name: "value".to_string(),
                value_type: Box::new(ValueType::primitive(PrimitiveType::F32)),
            }],
        }
    }

    fn get_name(&self) -> &str {
        "AddF32"
    }
}
