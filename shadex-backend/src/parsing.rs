use std::collections::HashMap;
pub mod type_parsing;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::take_until, tag},
    character::{
        complete::{alpha1, alphanumeric0},
        multispace0,
    },
    combinator::{complete, eof, fail, map, opt, value},
    error::{Error, ParseError},
    multi::{many0, separated_list0},
    number::float,
    sequence::{self, delimited, preceded, terminated},
};

use crate::nodegraph::{
    InputInfo, Node, NodeAnnotation, NodeGraph, NodeRef, NodeTypeInfo, NodeTypeRc, NodeTypeRef,
    OutputInfo, TypedNodeGraph, ValueRef,
};

pub struct SimpleTypeWorld {
    pub node_types: HashMap<String, NodeTypeRc>,
}

impl SimpleTypeWorld {
    pub fn new() -> Self {
        Self {
            node_types: HashMap::new(),
        }
    }
}

#[derive(Clone)]
struct ParseState {
    named_vars: HashMap<String, Value>,
}

#[derive(Debug)]
pub enum NodeExpression {
    Identifier(String),
    FloatLiteral(f32),
    IntLiteral(i32),
    Assignment(String, Box<NodeExpression>),
    Construction(String, Option<String>, Vec<NodeExpression>),
    FreeVariable,
    Output(Box<NodeExpression>, String),
}

// https://github.com/rust-bakery/nom/blob/main/examples/json2.rs
fn ws<'a, O, E: ParseError<&'a [u8]>, F: Parser<&'a [u8], Output = O, Error = E>>(
    f: F,
) -> impl Parser<&'a [u8], Output = O, Error = E> {
    delimited(multispace0(), f, multispace0())
}

fn parse_identifier<'a>() -> impl Parser<&'a [u8], Output = String, Error = Error<&'a [u8]>> {
    ws((alpha1, alphanumeric0)
        .map(|b| String::from_utf8_lossy(b.0).to_string() + String::from_utf8_lossy(b.1).as_ref()))
}

struct ExprParser;

impl<'a> Parser<&'a [u8]> for ExprParser {
    type Output = NodeExpression;

    type Error = Error<&'a [u8]>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: &'a [u8],
    ) -> nom::PResult<OM, &'a [u8], Self::Output, Self::Error> {
        let mut parser = (
            alt((
                // Construction
                (
                    parse_identifier(),
                    opt(preceded(ws(tag(":")), take_until("("))),
                    ws(tag("(")),
                    separated_list0(ws(tag(",")), parse_expr()),
                    ws(tag(")")),
                )
                    .map(|(name, info, _, args, _)| {
                        NodeExpression::Construction(
                            name,
                            info.map(|b| String::from_utf8_lossy(b).to_string()),
                            args,
                        )
                    }),
                // Assignment
                (parse_identifier(), ws(tag("=")), ws(parse_expr()))
                    .map(|(name, _, expr)| NodeExpression::Assignment(name, Box::new(expr))),
                // Null
                ws(tag("NULL")).map(|_| NodeExpression::FreeVariable),
                // Identifier
                parse_identifier().map(NodeExpression::Identifier),
                // Float literal
                ws(float()).map(NodeExpression::FloatLiteral),
            )),
            // Output
            opt((ws(tag(".")), parse_identifier()).map(|(_, field)| field)),
        )
            .map(|(expr, sub)| match sub {
                Some(field) => NodeExpression::Output(Box::new(expr), field),
                None => expr,
            });

        parser.process::<OM>(input)
    }
}

fn parse_expr() -> ExprParser {
    ExprParser
}

pub fn parse_whole_input(input: &[u8]) -> IResult<&[u8], Vec<NodeExpression>> {
    terminated(many0(ws(parse_expr())), eof).parse_complete(input)
}

#[derive(Clone, Copy)]
enum Value {
    Float(f32),
    Int(i32),
    NodeRef(NodeRef),
    ValueRef(Option<ValueRef>),
}

fn process_node_expr(
    expr: NodeExpression,
    state: &mut ParseState,
    graph: &mut TypedNodeGraph,
    types: &SimpleTypeWorld,
) -> Result<Value, ()> {
    match expr {
        NodeExpression::Identifier(name) => state.named_vars.get(&name).cloned().ok_or(()),
        NodeExpression::FloatLiteral(v) => Ok(Value::Float(v)),
        NodeExpression::IntLiteral(i) => Ok(Value::Int(i)),
        NodeExpression::Assignment(name, node_expression) => {
            let rhs = process_node_expr(*node_expression, state, graph, types)?;
            state.named_vars.insert(name, rhs);
            Ok(rhs)
        }
        NodeExpression::Construction(typename, data, node_expressions) => {
            let args: Result<Vec<Option<ValueRef>>, ()> = node_expressions
                .into_iter()
                .map(|expr| {
                    let _arg = process_node_expr(expr, state, graph, types)?;
                    match _arg {
                        Value::ValueRef(vr) => Ok(vr),
                        _ => return Err(()),
                    }
                })
                .collect();

            let type_ref = types.node_types.get(&typename).ok_or(())?;

            let node = Node {
                annotation: type_ref.clone(),
                inputs: args?,
                extra_data: data,
            };

            let node_id = graph.add_node(node);

            Ok(Value::NodeRef(node_id))
        }
        NodeExpression::Output(node_expression, output_name) => {
            let node_value = process_node_expr(*node_expression, state, graph, types)?;
            let node_ref = match node_value {
                Value::NodeRef(nr) => nr,
                _ => return Err(()),
            };

            let node_info = graph.get_node(node_ref).ok_or(())?;

            let type_info = &node_info.annotation;

            let output_ind = type_info
                .outputs
                .iter()
                .position(|outp| outp.name == output_name)
                .ok_or(())?;

            Ok(Value::ValueRef(Some(ValueRef {
                node: node_ref,
                output_index: output_ind,
            })))
        }
        NodeExpression::FreeVariable => Ok(Value::ValueRef(None)),
    }
}

pub fn construct_node_graph(
    types: &SimpleTypeWorld,
    exprs: Vec<NodeExpression>,
) -> Result<TypedNodeGraph, ()> {
    let mut parse_state = ParseState {
        named_vars: HashMap::new(),
    };

    let mut graph = TypedNodeGraph::new();

    for expr in exprs {
        process_node_expr(expr, &mut parse_state, &mut graph, types)?;
    }

    Ok(graph)
}
