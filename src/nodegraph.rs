use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct NodeRef {
    id: usize,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ValueRef {
    pub node: NodeRef,
    pub output_index: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NodeTypeRef {
    Custom(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum U32Boundedness {
    Unbounded,
    Bounded(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    F32,
    I32,
    U32(U32Boundedness),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueType {
    pub inputs: HashMap<String, Box<ValueType>>,
    pub output: PrimitiveType,
}

#[derive(Debug)]
pub struct InputInfo {
    pub name: String,
    pub value_type: Box<ValueType>,
}

#[derive(Debug)]
pub struct OutputInfo {
    pub name: String,
    pub value_type: Box<ValueType>,
}

#[derive(Debug)]
pub struct NodeTypeInfo {
    pub name: String,
    pub inputs: Vec<InputInfo>,
    pub outputs: Vec<OutputInfo>,
}

#[derive(Debug)]
pub struct Node {
    pub node_type: NodeTypeRef,
    pub inputs: Vec<Option<ValueRef>>,
    pub extra_data: Option<String>,
}

#[derive(Debug)]
pub struct TypeUniverse {
    pub node_types: HashMap<NodeTypeRef, NodeTypeInfo>,
    next_unclaimed_id: usize,
}

impl TypeUniverse {
    pub fn new() -> Self {
        Self {
            node_types: HashMap::new(),
            next_unclaimed_id: 0,
        }
    }

    pub fn create_new_type(&mut self, type_info: NodeTypeInfo) -> NodeTypeRef {
        let id = self.next_unclaimed_id;
        self.next_unclaimed_id = self.next_unclaimed_id + 1;
        let typeref = NodeTypeRef::Custom(id);
        self.node_types.insert(typeref.clone(), type_info);
        typeref
    }
}

#[derive(Debug)]
pub struct NodeGraph {
    pub types: TypeUniverse,
    nodes: HashMap<usize, Node>,
    next_id: usize,
}

impl NodeGraph {
    pub fn add_node(&mut self, node: Node) -> NodeRef {
        let node_id = self.next_id;
        self.nodes.insert(node_id, node);
        self.next_id += 1;
        NodeRef { id: node_id }
    }

    pub fn get_node(&self, node_ref: NodeRef) -> Option<&Node> {
        self.nodes.get(&node_ref.id)
    }

    pub fn new(types: TypeUniverse) -> Self {
        NodeGraph {
            types,
            nodes: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeRef, &Node)> {
        self.nodes.iter().map(|f| (NodeRef { id: *f.0 }, f.1))
    }
}
