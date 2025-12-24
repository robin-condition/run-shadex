use std::rc::Rc;

use shadex_backend::{
    nodegraph::{InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

pub struct AttrInfo {
    pub name: String,
    pub type_str: String,
}
impl AttrInfo {
    pub fn new() -> Self {
        Self {
            name: "x".to_string(),
            type_str: "f32".to_string(),
        }
    }
}

impl VisualNodeInfo for AttrInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        ui.text_edit_singleline(&mut self.name).changed()
            | ui.text_edit_singleline(&mut self.type_str).changed()
    }

    fn get_shadex_type(&self) -> Rc<NodeTypeInfo> {
        Rc::new(NodeTypeInfo {
            inputs: vec![InputInfo {
                name: self.name.clone(),
                value_type: Box::new(ValueType::primitive(PrimitiveType::F32)),
            }],
            outputs: vec![OutputInfo {
                name: self.name.clone(),
                value_type: Box::new(ValueType::primitive(PrimitiveType::F32)),
            }],
        })
    }

    fn get_name(&self) -> &str {
        "Attr"
    }
}
