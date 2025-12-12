use core::panic;
use std::collections::HashMap;

use crate::nodegraph::{NodeGraph, NodeRef, NodeTypeRef, ValueRef, ValueType};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NodeInputReference {
    pub source_node: NodeRef,
    pub input_ind: usize,
}

#[derive(Clone, Debug)]
pub struct OutputTypeNotes {
    // Formal type has arguments. Each comes from at least one of two places.
    pub formal_type: ValueType,

    // If inputs were evaluated to spec types, what types would be left in the output type?
    pub step_computation_requires: HashMap<String, ValueType>,
    // What would we need to know just to evaluate the inputs?
    pub inputs_parameterized_by: HashMap<String, ValueType>,
}

#[derive(Clone, Debug)]
pub struct InputTypeNotes {
    // Formal type EITHER comes freeformed from free variable, or has arguments that each come from one of two places.
    pub formal_type: ValueType,

    pub type_source: InputValueTypeSource,
}

#[derive(Clone, Debug)]
pub enum InputValueTypeSource {
    FreeVariable(FreeVariableTypeSource),
    FromOutput(OutputPromotion),
}

#[derive(Clone, Debug)]
pub struct FreeVariableTypeSource {
    // Types in the actual argument
    pub types_from_fv: HashMap<String, ValueType>,
    // Type of the free variable itself
    pub itself: (String, ValueType),
}

#[derive(Clone, Debug)]
pub struct OutputPromotion {
    // Types that come from the value source.
    pub types_from_output: HashMap<String, ValueType>,
    // What additional arguments were added by the typechecker that are not used in computation?
    pub added_constant_wrt: HashMap<String, ValueType>,

    // Cast necessary? Leaving off for now.
    pub cast: (),
}

#[derive(Debug)]
pub struct NodeGraphFormalTypeAnalysis {
    output_type_notes: HashMap<ValueRef, OutputTypeNotes>,
    input_type_notes: HashMap<NodeInputReference, InputTypeNotes>,
}

impl NodeGraphFormalTypeAnalysis {
    fn analyze_single_input(
        &mut self,
        graph: &NodeGraph,
        inp_ref: NodeInputReference,
    ) -> InputTypeNotes {
        if let Some(res) = self.input_type_notes.get(&inp_ref) {
            return res.clone();
        }

        let node = graph.get_node(inp_ref.source_node).unwrap();
        let node_type = graph.types.node_types.get(&node.node_type).unwrap();

        let specd_input_type = &node_type.inputs[inp_ref.input_ind].value_type;

        let provided_output_type =
            node.inputs[inp_ref.input_ind].map(|f| self.analyze_single_output(graph, f));

        let inp_notes = match provided_output_type {
            Some(real_output) => {
                let mut op = OutputPromotion {
                    types_from_output: real_output
                        .formal_type
                        .inputs
                        .iter()
                        .map(|(a, b)| (a.clone(), (**b).clone()))
                        .collect(),
                    added_constant_wrt: HashMap::new(),
                    cast: (),
                };
                // The actual input type must have at least all the arguments of the source value.
                let mut result_args: HashMap<String, Box<ValueType>> =
                    real_output.formal_type.inputs.clone();
                for arg in &specd_input_type.inputs {
                    // Input must be a function with an argument not present in the actual value.
                    // That is, the input must be "upcasted" to a constant function with respect to those inputs.
                    if !real_output.formal_type.inputs.contains_key(arg.0) {
                        op.added_constant_wrt
                            .insert(arg.0.clone(), (**arg.1).clone());
                        result_args.insert(arg.0.clone(), arg.1.clone());
                    }
                    // Otherwise, it is present in both the source output and the input. Should be checked for equality.
                    else {
                        // I know this can be done more efficiently with some entry stuff that I did earlier, but I hate reading it.
                        // So I'm skipping it this time.

                        if **real_output.formal_type.inputs.get(arg.0).unwrap() != **arg.1 {
                            panic!("Input argument type is wrong")
                        }
                    }
                }

                if specd_input_type.output != real_output.formal_type.output {
                    // Should insert a cast. For now, scream.
                    panic!("Wrong primitive! Tell Robin to add casting.");
                }

                let src = InputValueTypeSource::FromOutput(op);

                InputTypeNotes {
                    formal_type: ValueType {
                        inputs: result_args,
                        output: specd_input_type.output,
                    },
                    type_source: src,
                }
            }
            None => {
                let mut formal_type = (**specd_input_type).clone();
                let itself = (
                    &node_type.inputs[inp_ref.input_ind].name,
                    formal_type.clone(),
                );
                formal_type
                    .inputs
                    .insert(itself.0.clone(), Box::new(itself.1.clone()));
                InputTypeNotes {
                    formal_type: formal_type,
                    type_source: InputValueTypeSource::FreeVariable(FreeVariableTypeSource {
                        types_from_fv: specd_input_type
                            .inputs
                            .iter()
                            .map(|(a, b)| (a.clone(), (**b).clone()))
                            .collect(),
                        itself: (itself.0.clone(), itself.1),
                    }),
                }
            }
        };

        self.input_type_notes.insert(inp_ref, inp_notes.clone());
        inp_notes
    }

    fn analyze_single_output(&mut self, graph: &NodeGraph, val_ref: ValueRef) -> OutputTypeNotes {
        // Check existing analysis.
        // If already analyzed:
        if let Some(typ) = self.output_type_notes.get(&val_ref) {
            return typ.clone();
        }

        let ValueRef {
            node: node_ref,
            output_index,
        } = val_ref;

        // Original node reference
        let node = graph.get_node(node_ref).unwrap();

        // Original node type
        let node_type = graph.types.node_types.get(&node.node_type).unwrap();

        // Original output type
        let output_type = &node_type.outputs[output_index];

        // Steps:
        // 1. Record types of inputs, real and missing.
        //      - For each one, note absent and excess arguments.
        //      - If type spec requires argument that is not present: Insert "ConstantWRT" correction / dummy argument.
        //      - If primitive is wrong, insert cast.
        //      - If arguments are specified that are not in type spec, absorb it into "excess" arguments that parameterize this computation.
        // 2. Create output type:
        //      - Primitive: primitive of desired output type.
        //      - Arguments: union of excess arguments of inputs, unioned with spec'd arguments of output
        //      -

        // Question: How to handle two excess input arguments with same name but different arguments?
        // For now: Just fail.
        // Possible solution: Intersect them / pick the 'narrowest' form / defer to constant functions basically.
        // Potential pain point: Inserting "constant wrt"s to allow the narrower inputs to extend to the broader uses.

        //let mut input_types: Vec<ValueType> = vec![];
        let mut excess_input_args: HashMap<String, ValueType> = HashMap::new();

        for i in 0..node_type.inputs.len() {
            let specd_input_type = &node_type.inputs[i].value_type;
            // This function call will insert "Constant WRT"s and casts as-needed, so that provided_input_type is always a superset of specd. ??? Maybe???
            let provided_input_type = self.analyze_single_input(
                graph,
                NodeInputReference {
                    source_node: node_ref,
                    input_ind: i,
                },
            );

            for (name, typ) in &provided_input_type.formal_type.inputs {
                if specd_input_type.inputs.contains_key(name) {
                    continue;
                }

                let slot = excess_input_args.entry(name.clone());
                slot.and_modify(|curr| {
                    if *curr != **typ {
                        panic!("Two arguments with same name and different inputs. Might change behavior later to be narrowing.")
                    }
                }).or_insert_with(|| *typ.clone());
            }
        }

        let inputs_parameterized_by = excess_input_args.clone();

        let mut output_formal_args = excess_input_args;

        let specd_output_args: HashMap<String, ValueType> = output_type
            .value_type
            .inputs
            .iter()
            .map(|f| (f.0.clone(), *f.1.clone()))
            .collect();

        for output_inp in &output_type.value_type.inputs {
            let slot = output_formal_args.entry(output_inp.0.clone());
            slot.and_modify(|curr| {
                if *curr != **output_inp.1 {
                    // This SHOULD be okay, because it's actually two evaluation steps (first to deparameterize the output and second to evaluate it.)
                    // I think?
                    panic!("Input and output arguments with same name and different inputs. WILL change behavior later.");
                }
            })
            .or_insert_with(|| *output_inp.1.clone());
        }

        let actual_output_type = ValueType {
            inputs: output_formal_args
                .into_iter()
                .map(|f| (f.0, Box::new(f.1)))
                .collect(),
            output: output_type.value_type.output,
        };

        let output_type_notes = OutputTypeNotes {
            formal_type: actual_output_type,
            step_computation_requires: specd_output_args,
            inputs_parameterized_by,
        };

        self.output_type_notes
            .insert(val_ref, output_type_notes.clone());

        output_type_notes
    }

    pub fn analyze(graph: &NodeGraph) -> NodeGraphFormalTypeAnalysis {
        let mut analysis = NodeGraphFormalTypeAnalysis {
            output_type_notes: HashMap::new(),
            input_type_notes: HashMap::new(),
        };
        let output: Vec<(NodeRef, &crate::nodegraph::Node)> = graph
            .iter_nodes()
            .filter(|(a, b)| graph.types.node_types.get(&b.node_type).unwrap().name == "Out")
            .collect();
        for i in output {
            analysis.analyze_single_input(
                graph,
                NodeInputReference {
                    source_node: i.0,
                    input_ind: 0,
                },
            );
        }
        analysis
    }
}

pub struct Expression {
    core: ExpressionCore,
    formal_type: ValueType,
}

pub enum ExpressionCore {
    TypeSystemStep(TypeCorrectionStep),
}

pub enum TypeCorrectionStep {
    ConstantWRTArgument(String, ValueType, Box<Expression>),
    AbsorbArgumentsLeftward(Box<Expression>),
}
