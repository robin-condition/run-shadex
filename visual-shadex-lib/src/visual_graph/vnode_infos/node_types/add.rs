use std::rc::Rc;

use shadex_backend::{
    nodegraph::{InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

thread_local! {
    static ADD_TYPE: Rc<NodeTypeInfo> =
        Rc::new(NodeTypeInfo {
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
            });
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

    fn get_shadex_type(&self) -> Rc<NodeTypeInfo> {
        ADD_TYPE.with(Rc::<NodeTypeInfo>::clone)
    }

    fn get_name(&self) -> &str {
        "AddF32"
    }
}
