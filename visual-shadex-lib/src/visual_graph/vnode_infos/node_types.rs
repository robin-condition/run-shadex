use crate::visual_graph::{
    AddInfo, ConstantInfo, VisualNodeInfo,
    vnode_infos::{attr::AttrInfo, out::OutInfo, vector3::Vector3Info},
};

pub mod add;
pub mod attr;
pub mod constant;
pub mod out;
pub mod vector3;

pub const INITIALIZATIONS: [(&str, fn() -> Box<dyn VisualNodeInfo>); 5] = [
    ("Constant", || Box::new(ConstantInfo::new(0.5f32))),
    ("Out", || Box::new(OutInfo::new())),
    ("Attr", || {
        Box::new(AttrInfo::new("x".to_string(), "f32".to_string()))
    }),
    ("Add", || Box::new(AddInfo::new())),
    ("Vector", || Box::new(Vector3Info::new())),
];
