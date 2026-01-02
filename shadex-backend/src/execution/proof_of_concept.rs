use std::collections::HashMap;

use crate::{
    nodegraph::{
        FallibleNodeTypeRc, NodeAnnotation, NodeAnnotationHas, NodeGraph, NodeRef,
        NodeTypeAnnotation, ValueRef,
    },
    typechecking::{NodeGraphFormalTypeAnalysis, typetypes::TypeError},
};

#[derive(Clone)]
pub struct ShaderProgram {
    pub text: String,
    pub name: String,
}

pub struct NameGenerator {
    next_id: usize,
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self {
            next_id: Default::default(),
        }
    }
}

impl NameGenerator {
    pub fn generate_name(&mut self) -> String {
        let st = format!("id{}", self.next_id);
        self.next_id += 1;
        st
    }

    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

pub struct Executor {
    pub namer: NameGenerator,
}

#[derive(Debug, Clone)]
pub enum ExecutionInformation {
    Add,
    Exp,
    Constant(f32),
    Attr(String),
    Out,
    ERR,
    Vector3,
}

impl NodeTypeAnnotation for ExecutionInformation {}

impl Default for Executor {
    fn default() -> Self {
        Self {
            namer: Default::default(),
        }
    }
}

impl Executor {
    fn make_prog<T: NodeAnnotationHas<FallibleNodeTypeRc>>(
        &mut self,
        cached: &mut HashMap<NodeRef, Result<ShaderProgram, TypeError>>,
        port: ValueRef,
        graph: &NodeGraph<T>,
        types: &NodeGraphFormalTypeAnalysis,
    ) -> Result<ShaderProgram, TypeError> {
        let n = graph.get_node(port.node).ok_or(TypeError {
            message: "Node not found".to_string(),
        })?;
        let exec = n.annotation.get_t().clone().map(|f| f.annotation.clone())?;

        let inps: Option<Result<Vec<ShaderProgram>, TypeError>> = n
            .inputs
            .iter()
            .map(|f| f.map(|g| self.make_prog(cached, g, graph, types)))
            .collect();

        let res = match exec {
            ExecutionInformation::Add => {
                let inps = inps.ok_or(TypeError {
                    message: "No inputs".to_string(),
                })??;

                let (inp_texts, inp_names): (Vec<String>, Vec<String>) =
                    inps.into_iter().map(|a| (a.text, a.name)).collect();

                let name = self.namer.generate_name();

                let result_text = format!(
                    "{}\n{}\nfn {}(x: f32, y: f32, component: u32) -> f32 {{ return {}(x,y,component) + {}(x,y,component); }}",
                    inp_texts[0], inp_texts[1], name, inp_names[0], inp_names[1]
                );

                Ok(ShaderProgram {
                    text: result_text,
                    name,
                })
            }
            ExecutionInformation::Vector3 => {
                let inps = inps.ok_or(TypeError {
                    message: "No inputs".to_string(),
                })??;

                let (inp_texts, inp_names): (Vec<String>, Vec<String>) =
                    inps.into_iter().map(|a| (a.text, a.name)).collect();

                let name = self.namer.generate_name();

                let result_text = format!(
                    "{}\n{}\n{}\nfn {}(x: f32, y: f32, component: u32) -> f32 {{ if component == 0 {{ return {}(x,y,component); }} if component == 1 {{ return {}(x,y,component); }} return {}(x,y,component); }}",
                    inp_texts[0],
                    inp_texts[1],
                    inp_texts[2],
                    name,
                    inp_names[0],
                    inp_names[1],
                    inp_names[2]
                );

                Ok(ShaderProgram {
                    text: result_text,
                    name,
                })
            }
            ExecutionInformation::Exp => todo!(),
            ExecutionInformation::Constant(val) => {
                let name = self.namer.generate_name();
                Ok(ShaderProgram {
                    text: format!(
                        "fn {}(x: f32, y: f32, component: u32) -> f32 {{ return {}f; }}",
                        name, val
                    ),
                    name,
                })
            }
            ExecutionInformation::Attr(attr_name) => {
                let name = self.namer.generate_name();
                Ok(ShaderProgram {
                    text: format!(
                        "fn {}(x: f32, y: f32, component: u32) -> f32 {{ return {}; }}",
                        name, attr_name
                    ),
                    name,
                })
            }
            ExecutionInformation::Out => todo!(),
            ExecutionInformation::ERR => Err(TypeError {
                message: "No execution information".to_string(),
            }),
        };

        cached.insert(port.node, res.clone());
        res
    }

    pub fn reset_names(&mut self) {
        self.namer.reset();
    }

    pub fn run<T: NodeAnnotationHas<FallibleNodeTypeRc>>(
        &mut self,
        graph: &NodeGraph<T>,
        types: &NodeGraphFormalTypeAnalysis,
    ) -> Result<ShaderProgram, TypeError> {
        let mut results = HashMap::new();
        for n in graph.iter_nodes() {
            if let Ok(typ) = n.1.annotation.get_t() {
                if let ExecutionInformation::Out = typ.annotation {
                    if let Some(inp) = n.1.inputs[0] {
                        return self.make_prog(&mut results, inp, graph, types);
                    }
                }
            }
        }
        Err(TypeError {
            message: "No output found".to_string(),
        })
    }
}
