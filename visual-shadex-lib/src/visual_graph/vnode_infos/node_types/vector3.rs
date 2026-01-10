use std::rc::Rc;

use serde::{Deserialize, Serialize};
use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

thread_local! {
    static VEC3_TYPE: FallibleNodeTypeRc =
        Ok(Rc::new(NodeTypeInfo {
                inputs: vec![
                    InputInfo {
                        name: "x".to_string(),
                        value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
                    },
                    InputInfo {
                        name: "y".to_string(),
                        value_type: Ok(ValueType::primitive(PrimitiveType::F32)),
                    },
                    InputInfo {
                        name: "z".to_string(),
                        value_type: Ok(ValueType::primitive(PrimitiveType::F32))
                    }
                ],
                outputs: vec![OutputInfo {
                    name: None,
                    value_type: Ok(ValueType { inputs: [("component".to_string(), Box::new(ValueType::primitive(PrimitiveType::U32(shadex_backend::typechecking::typetypes::U32Boundedness::Bounded(3)))))].into(), output: PrimitiveType::F32 }),
                }],
                annotation: shadex_backend::execution::ExecutionInformation::Vector3
            }));
}

#[derive(Serialize, Deserialize)]
pub struct Vector3Info {}
impl Vector3Info {
    pub fn new() -> Self {
        Self {}
    }
}

#[typetag::serde]
impl VisualNodeInfo for Vector3Info {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        false
    }

    fn get_shadex_type(&self) -> FallibleNodeTypeRc {
        VEC3_TYPE.with(FallibleNodeTypeRc::clone)
    }

    fn get_name(&self) -> &str {
        "Vec3"
    }
}
