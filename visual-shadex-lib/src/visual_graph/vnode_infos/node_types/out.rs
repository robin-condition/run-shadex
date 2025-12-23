use std::collections::HashMap;

use shadex_backend::nodegraph::{InputInfo, NodeTypeInfo, OutputInfo, PrimitiveType, ValueType};

use crate::visual_graph::VisualNodeInfo;

pub struct OutInfo {}
impl OutInfo {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisualNodeInfo for OutInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        false
    }

    fn get_shadex_type(&self) -> NodeTypeInfo {
        NodeTypeInfo {
            name: "test".to_string(),
            inputs: vec![InputInfo {
                name: "val".to_string(),
                value_type: Box::new(ValueType {
                    inputs: [
                        (
                            "x".to_string(),
                            Box::new(ValueType::primitive(PrimitiveType::U32(
                                shadex_backend::nodegraph::U32Boundedness::Bounded(1024),
                            ))),
                        ),
                        (
                            "y".to_string(),
                            Box::new(ValueType::primitive(PrimitiveType::U32(
                                shadex_backend::nodegraph::U32Boundedness::Bounded(1024),
                            ))),
                        ),
                        (
                            "component".to_string(),
                            Box::new(ValueType::primitive(PrimitiveType::U32(
                                shadex_backend::nodegraph::U32Boundedness::Bounded(3),
                            ))),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    output: PrimitiveType::F32,
                }),
            }],
            outputs: Vec::new(),
        }
    }

    fn get_name(&self) -> &str {
        "Out"
    }
}
