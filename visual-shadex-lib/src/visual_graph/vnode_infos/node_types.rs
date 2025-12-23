use crate::visual_graph::{ConstantInfo, VisualNodeInfo, vnode_infos::out::OutInfo};

pub mod add;
pub mod constant;
pub mod out;

pub const INITIALIZATIONS: [(&str, fn() -> Box<dyn VisualNodeInfo>); 2] = [
    ("Constant", || Box::new(ConstantInfo::new(2.0f32))),
    ("Out", || Box::new(OutInfo::new())),
];
