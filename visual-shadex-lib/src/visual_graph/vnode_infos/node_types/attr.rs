use std::rc::Rc;

use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, TypeError, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

pub struct AttrInfo {
    pub name: String,
    pub type_str: String,

    prev_valid_type: FallibleNodeTypeRc,
}
impl AttrInfo {
    fn build_type(n: &String, typstr: &str) -> FallibleNodeTypeRc {
        let typ = shadex_backend::parsing::type_parsing::parse_complete_value_type(typstr);
        Ok(Rc::new(NodeTypeInfo {
            inputs: vec![InputInfo {
                name: n.clone(),
                value_type: typ.clone(),
            }],
            outputs: vec![OutputInfo {
                name: Some(n.clone()),
                value_type: typ,
            }],
            annotation: shadex_backend::execution::ExecutionInformation::Attr(n.clone()),
        }))
    }

    pub fn new(name: String, type_str: String) -> Self {
        let ftype = Self::build_type(&name, &type_str);
        Self {
            name: name,
            type_str: type_str,
            prev_valid_type: ftype,
        }
    }
}

impl VisualNodeInfo for AttrInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        ui.set_max_width(50f32);

        let changed = ui.text_edit_singleline(&mut self.name).changed()
            | ui.text_edit_singleline(&mut self.type_str).changed();
        if changed {
            self.prev_valid_type = Self::build_type(&self.name, &self.type_str);
        }
        changed
    }

    fn get_shadex_type(&self) -> FallibleNodeTypeRc {
        self.prev_valid_type.clone()
    }

    fn get_name(&self) -> &str {
        "Attr"
    }
}
