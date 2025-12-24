use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::typechecking::typetypes::{MaybeValueType, TypeError, ValueType};

pub trait NodeAnnotation: Clone + std::fmt::Debug {}

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

pub trait PortTypeAnnotation: Clone {}

#[derive(Debug)]
pub struct InputInfo<T: PortTypeAnnotation> {
    pub name: String,
    pub value_type: T,
}

#[derive(Debug)]
pub struct OutputInfo<T: PortTypeAnnotation> {
    pub name: String,
    pub value_type: T,
}

#[derive(Debug)]
pub struct NodeTypeInfo<T: PortTypeAnnotation> {
    pub inputs: Vec<InputInfo<T>>,
    pub outputs: Vec<OutputInfo<T>>,
}

pub type FallibleNodeTypeRc = Result<Rc<NodeTypeInfo<MaybeValueType>>, TypeError>;

impl PortTypeAnnotation for Box<ValueType> {}
impl PortTypeAnnotation for MaybeValueType {}
pub type NodeTypeRc = Rc<NodeTypeInfo<Box<ValueType>>>;
impl NodeAnnotation for NodeTypeRc {}

pub type TypedNode = Node<NodeTypeRc>;
pub type TypedNodeGraph = NodeGraph<NodeTypeRc>;

#[derive(Debug)]
pub struct Node<T: NodeAnnotation> {
    pub annotation: T,
    pub inputs: Vec<Option<ValueRef>>,
    pub extra_data: Option<String>,
}

#[derive(Debug)]
pub struct NodeGraph<T: NodeAnnotation> {
    nodes: HashMap<usize, Node<T>>,
    next_id: usize,
}

impl<T: NodeAnnotation> NodeGraph<T> {
    pub fn add_node(&mut self, node: Node<T>) -> NodeRef {
        let node_id = self.next_id;
        self.nodes.insert(node_id, node);
        self.next_id += 1;
        NodeRef { id: node_id }
    }

    pub fn get_node(&self, node_ref: NodeRef) -> Option<&Node<T>> {
        self.nodes.get(&node_ref.id)
    }

    pub fn new() -> Self {
        NodeGraph {
            nodes: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeRef, &Node<T>)> {
        self.nodes.iter().map(|f| (NodeRef { id: *f.0 }, f.1))
    }
}
