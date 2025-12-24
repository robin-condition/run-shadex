use std::{collections::HashMap, rc::Rc};

use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, U32Boundedness, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

thread_local! {
    static OUT_TYPE: FallibleNodeTypeRc = Ok(Rc::new(NodeTypeInfo {
            inputs: vec![InputInfo {
                name: "val".to_string(),
                value_type: Ok(ValueType {
                    inputs: [
                        (
                            "x".to_string(),
                            Box::new(ValueType::primitive(PrimitiveType::U32(
                                U32Boundedness::Bounded(1024),
                            ))),
                        ),
                        (
                            "y".to_string(),
                            Box::new(ValueType::primitive(PrimitiveType::U32(
                                U32Boundedness::Bounded(1024),
                            ))),
                        ),
                        (
                            "component".to_string(),
                            Box::new(ValueType::primitive(PrimitiveType::U32(
                                U32Boundedness::Bounded(3),
                            ))),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    output: PrimitiveType::F32,
                })
            }],
            outputs: Vec::new(),
        }));
}

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

    fn get_shadex_type(&self) -> FallibleNodeTypeRc {
        OUT_TYPE.with(FallibleNodeTypeRc::clone)
    }

    fn get_name(&self) -> &str {
        "Out"
    }
}
