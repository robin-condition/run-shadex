use std::collections::HashMap;

pub struct NodeRef {
    id: usize,
}

pub struct ValueRef {
    pub node: NodeRef,
    pub output_index: usize,
}

#[derive(Clone, Copy)]
pub enum NodeTypeRef {
    Constant,
    Vec3,
    Out,
}

pub enum PrimitiveType {
    F32,
    I32,
}

pub struct ValueType {
    pub inputs: HashMap<String, Box<ValueType>>,
    pub output: PrimitiveType,
}

pub struct InputInfo {
    pub name: String,
    pub value_type: Box<ValueType>,
}

pub struct OutputInfo {
    pub name: String,
    pub primitive_type: PrimitiveType,
}

pub struct NodeTypeInfo {
    pub name: &'static str,
    pub inputs: Vec<InputInfo>,
    pub outputs: Vec<OutputInfo>,
}

pub struct Node {
    pub node_type: NodeTypeRef,
    pub inputs: Vec<Option<ValueRef>>,
}

pub struct TypeUniverse {
    node_types: HashMap<NodeTypeRef, NodeTypeInfo>,
}

pub struct NodeGraph {
    types: TypeUniverse,
    nodes: HashMap<usize, Node>,
    next_id: usize,
}