use std::collections::HashMap;

use rpds::HashTrieMap;

use crate::{
    execution::value_reprs::{InputSet, ParameterValueSet},
    nodegraph::{NodeGraph, NodeRef, NodeTypeRef, ValueRef},
};

pub mod computation_requests;
pub mod typechecking;
pub mod value_reprs;

pub struct Executor<'a> {
    pub graph: &'a NodeGraph,
}

pub struct SinglePrimitiveComputationResult<T: Copy> {
    val: T,
}

struct FunctionRepresentation {
    // Some kind of type information

    // And association computation
    computation: ComputationRepresentation,
}

struct ConstantPrimitiveFunction<T> {
    pub val: T,
}

struct CPUNodeClosure {
    pub output_value: ValueRef,
    pub context: HashTrieMap<String, FunctionRepresentation>,
}

struct GPUFragShader {
    pub source: String, // CHANGE to something using rspirv
    pub bindings: (),   // Buh
}

enum ComputationRepresentation {
    // Constant -- need? distinction between uniform buffer and CPU-side variable
    // - Vector
    //  (CPU-side "eager")
    ConstantF32(ConstantPrimitiveFunction<f32>),
    ConstantI32(ConstantPrimitiveFunction<i32>),

    // CPU-side "lazy" (closure)
    CPUDeferred(CPUNodeClosure),
    // Shader (GPU-side "lazy")

    // Texture (GPU-side "eager")
    // Buffer ? -- for later
}

impl<'a> Executor<'a> {
    fn request_computation(&mut self, val: ValueRef, params: InputSet) -> FunctionRepresentation {
        panic!()
    }

    fn get_output_valueref(&self) -> Option<ValueRef> {
        self.graph.iter_nodes().find_map(|(_, nod)| {
            if nod.node_type == NodeTypeRef::Out {
                nod.inputs[0]
            } else {
                None
            }
        })
    }
    pub fn execute(&mut self) -> Option<()> {
        let outval = self.get_output_valueref()?;
        panic!()
        /*
        let outp = self.request_computation(
            outval,
            InputSet {
                params: [
                    ("x".to_string(), ParameterValueSet::I32Range([0, 256])),
                    ("y".to_string(), ParameterValueSet::I32Range([0, 256])),
                    ("component".to_string(), ParameterValueSet::I32Range([0, 3])),
                ]
                .into(),
            },
        );
        Some(())
        */
    }
}
