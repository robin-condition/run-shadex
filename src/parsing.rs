use std::collections::HashMap;

use nom::{IResult, Parser, branch::alt, bytes::tag, character::{complete::{alpha1, alphanumeric0}, multispace0}, combinator::{map, opt, value}, error::{Error, ParseError}, multi::{many0, separated_list0}, number::float, sequence::delimited};
use rpds::HashTrieMap;

use crate::nodegraph::{Node, NodeGraph, NodeRef, NodeTypeRef};


#[derive(Clone)]
struct ParseState {
    named_nodes: HashTrieMap<String, NodeRef>,
}

#[derive(Debug)]
pub enum NodeExpression {
    Identifier(String),
    FloatLiteral(f32),
    IntLiteral(i32),
    Assignment(String, Box<NodeExpression>),
    Construction(String, Vec<NodeExpression>),
    Output(Box<NodeExpression>, String),
}

// https://github.com/rust-bakery/nom/blob/main/examples/json2.rs
fn ws<'a, O, E: ParseError<&'a [u8]>, F: Parser<&'a [u8], Output = O, Error = E>>(
  f: F,
) -> impl Parser<&'a [u8], Output = O, Error = E> {
  delimited(multispace0(), f, multispace0())
}

fn parse_identifier<'a>() -> impl Parser<&'a [u8], Output = String, Error = Error<&'a [u8]>> {
    ws((alpha1, alphanumeric0).map(|b| String::from_utf8_lossy(b.0).to_string() + String::from_utf8_lossy(b.1).as_ref()))
}

struct ExprParser;

impl<'a> Parser<&'a [u8]> for ExprParser {
    type Output = NodeExpression;

    type Error = Error<&'a [u8]>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: &'a [u8],
      ) -> nom::PResult<OM, &'a [u8], Self::Output, Self::Error> {
        
let mut parser = (alt((
        // Construction
        (parse_identifier(), ws(tag("(")), separated_list0(ws(tag(",")), parse_expr()), ws(tag(")"))).map(|(name, _, args, _)| NodeExpression::Construction(name, args)),
        
        // Assignment
        (parse_identifier(), ws(tag("=")), ws(parse_expr())).map(|(name, _, expr)| NodeExpression::Assignment(name, Box::new(expr))),
        
        // Identifier
        parse_identifier().map(NodeExpression::Identifier),

        // Float literal
        ws(float()).map(NodeExpression::FloatLiteral),
    )),
    // Output
    opt((ws(tag(".")), parse_identifier()).map(|(_, field)| field)
    )).map(|(expr, sub)| match sub {
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
    many0(ws(parse_expr())).parse_complete(input)
}