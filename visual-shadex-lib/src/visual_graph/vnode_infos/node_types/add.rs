use std::rc::Rc;

use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

thread_local! {
    static ADD_TYPE: FallibleNodeTypeRc =
        Ok(Rc::new(NodeTypeInfo {
                inputs: vec![
                    InputInfo {
                        name: "a".to_string(),
                        value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
                    },
                    InputInfo {
                        name: "b".to_string(),
                        value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
                    },
                ],
                outputs: vec![OutputInfo {
                    name: "value".to_string(),
                    value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
                }],
                annotation: shadex_backend::execution::ExecutionInformation::Add
            }));
}

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

    fn get_shadex_type(&self) -> FallibleNodeTypeRc {
        ADD_TYPE.with(FallibleNodeTypeRc::clone)
    }

    fn get_name(&self) -> &str {
        "AddF32"
    }
}
