use std::rc::Rc;

use serde::{Deserialize, Serialize};
use shadex_backend::{
    nodegraph::{FallibleNodeTypeRc, InputInfo, NodeTypeInfo, OutputInfo},
    typechecking::typetypes::{PrimitiveType, TypeError, ValueType},
};

use crate::visual_graph::VisualNodeInfo;

#[derive(Serialize, Deserialize)]
pub struct AttrInfoData {
    pub name: String,
    pub type_str: String,
}

pub struct AttrInfo {
    pub data: AttrInfoData,
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
                name: None,
                value_type: typ,
            }],
            annotation: shadex_backend::execution::ExecutionInformation::Attr(n.clone()),
        }))
    }

    pub fn new(name: String, type_str: String) -> Self {
        let ftype = Self::build_type(&name, &type_str);
        Self {
            data: AttrInfoData { name, type_str },
            prev_valid_type: ftype,
        }
    }
}

impl Serialize for AttrInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AttrInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        AttrInfoData::deserialize(deserializer).map(AttrInfoData::into)
    }
}

impl From<AttrInfoData> for AttrInfo {
    fn from(value: AttrInfoData) -> Self {
        Self::new(value.name, value.type_str)
    }
}

#[typetag::serde]
impl VisualNodeInfo for AttrInfo {
    fn show(&mut self, ui: &mut egui::Ui) -> bool {
        ui.set_max_width(50f32);

        let changed = ui.text_edit_singleline(&mut self.data.name).changed()
            | ui.text_edit_singleline(&mut self.data.type_str).changed();
        if changed {
            self.prev_valid_type = Self::build_type(&self.data.name, &self.data.type_str);
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
