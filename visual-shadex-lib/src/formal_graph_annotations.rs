use std::collections::HashMap;

use shadex_backend::{
    nodegraph::{
        FallibleNodeTypeRc, NodeAnnotation, NodeAnnotationHas, NodeGraph, NodeRef, TypedNodeGraph,
        ValueRef,
    },
    typechecking::{
        NodeGraphFormalTypeAnalysis, NodeInputReference, typetypes::AccessibleFallibleType,
    },
};

use crate::visual_graph::{VNodeId, VNodeInputRef, VNodeOutputRef};

#[derive(Debug, Clone)]
pub struct MappedNodeAnnotation {
    pub type_info: FallibleNodeTypeRc,
    pub source_node: VNodeId,
}

impl NodeAnnotation for MappedNodeAnnotation {}

impl AccessibleFallibleType for MappedNodeAnnotation {
    fn fallible(&self) -> &FallibleNodeTypeRc {
        &self.type_info
    }
}

impl NodeAnnotationHas<FallibleNodeTypeRc> for MappedNodeAnnotation {
    fn get_t(&self) -> &FallibleNodeTypeRc {
        self.fallible()
    }
}

pub struct FormalGraph {
    pub formal_graph: NodeGraph<MappedNodeAnnotation>,
    pub typecheck: NodeGraphFormalTypeAnalysis,
    pub vnode_to_fnode: HashMap<VNodeId, NodeRef>,
}
