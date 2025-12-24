use crate::visual_graph::{
    AddInfo, ConstantInfo, VisualNodeInfo,
    vnode_infos::{attr::AttrInfo, out::OutInfo},
};

pub mod add;
pub mod attr;
pub mod constant;
pub mod out;

pub const INITIALIZATIONS: [(&str, fn() -> Box<dyn VisualNodeInfo>); 4] = [
    ("Constant", || Box::new(ConstantInfo::new(2.0f32))),
    ("Out", || Box::new(OutInfo::new())),
    ("Attr", || {
        Box::new(AttrInfo::new("x".to_string(), "f32".to_string()))
    }),
    ("Add", || Box::new(AddInfo::new())),
];
