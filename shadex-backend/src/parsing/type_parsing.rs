use std::{collections::HashMap, process::Output, rc::Rc};

use nom::{
    Parser,
    branch::alt,
    bytes::tag,
    character::complete::alphanumeric1,
    combinator::{eof, not, opt, recognize},
    error::Error,
    multi::{many0, separated_list0, separated_list1},
    sequence::{self, delimited, preceded, separated_pair, terminated},
};

use crate::{
    nodegraph::{FallibleNodeTypeRc, InputInfo, NodeTypeInfo, NodeTypeRc, OutputInfo},
    parsing::SimpleTypeWorld,
    typechecking::typetypes::{
        MaybeValueType, PrimitiveType, TypeError, U32Boundedness, ValueType,
    },
};

use super::{parse_identifier, ws};

pub struct FnTypeParser;

impl<'a> Parser<&'a [u8]> for FnTypeParser {
    type Output = ValueType;

    type Error = Error<&'a [u8]>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: &'a [u8],
    ) -> nom::PResult<OM, &'a [u8], Self::Output, Self::Error> {
        let argset_parser = alt((
            (ws(tag("(")), ws(tag(")"))).map(|_| HashMap::<String, Box<ValueType>>::new()),
            separated_list1(ws(tag(",")), parse_named_arg_type())
                .map(|g| g.into_iter().map(|(a, b)| (a, Box::new(b))).collect()),
        ));

        let mut parser =
            separated_pair(argset_parser, ws(tag("->")), parse_primitive_type()).map(|(a, b)| {
                ValueType {
                    inputs: a,
                    output: b,
                }
            });

        parser.process::<OM>(input)
    }
}

fn parse_named_arg_type<'a>()
-> impl Parser<&'a [u8], Output = (String, ValueType), Error = Error<&'a [u8]>> {
    separated_pair(parse_identifier(), ws(tag(":")), parse_arg_type())
}

fn total_tag<'a>(s: &str) -> impl Parser<&'a [u8], Output = &'a [u8], Error = Error<&'a [u8]>> {
    terminated(tag(s), not(recognize(alphanumeric1)))
}

fn parse_u32_bound<'a>() -> impl Parser<&'a [u8], Output = u32, Error = Error<&'a [u8]>> {
    delimited(ws(tag("[")), nom::character::complete::u32, ws(tag("]")))
}

fn parse_primitive_type<'a>()
-> impl Parser<&'a [u8], Output = PrimitiveType, Error = Error<&'a [u8]>> {
    ws(alt((
        total_tag("i32").map(|_| PrimitiveType::I32),
        total_tag("f32").map(|_| PrimitiveType::F32),
        (total_tag("u32"), opt(parse_u32_bound())).map(|(_, bd)| {
            PrimitiveType::U32(bd.map_or(U32Boundedness::Unbounded, U32Boundedness::Bounded))
        }),
        parse_u32_bound().map(|num| PrimitiveType::U32(U32Boundedness::Bounded(num))),
    )))
}

fn parse_fn_type() -> FnTypeParser {
    FnTypeParser
}

fn parse_arg_type<'a>() -> impl Parser<&'a [u8], Output = ValueType, Error = Error<&'a [u8]>> {
    alt((
        parse_primitive_type().map(ValueType::primitive),
        delimited(ws(tag("(")), parse_fn_type(), ws(tag(")"))),
    ))
}

pub fn parse_sugar_fn_type<'a>()
-> impl Parser<&'a [u8], Output = ValueType, Error = Error<&'a [u8]>> {
    ws(alt((
        parse_primitive_type().map(ValueType::primitive),
        parse_fn_type(),
    )))
}

pub fn parse_complete_value_type(content: &str) -> MaybeValueType {
    let mut parser = terminated(parse_sugar_fn_type(), eof);
    match parser.parse_complete(content.as_bytes()) {
        Ok(typ) => Ok(typ.1),
        Err(_) => Err(TypeError {
            message: "Parsing failed".to_string(),
        }),
    }
}

// Node types

fn parse_named_value_type<'a>()
-> impl Parser<&'a [u8], Output = (String, ValueType), Error = Error<&'a [u8]>> {
    separated_pair(ws(parse_identifier()), ws(tag("@")), parse_sugar_fn_type())
}

fn parse_node_type_declaration<'a>()
-> impl Parser<&'a [u8], Output = (String, FallibleNodeTypeRc), Error = Error<&'a [u8]>> {
    let inputs_parser = separated_list0(
        ws(tag(";")),
        parse_named_value_type().map(|(n, content)| InputInfo {
            name: n,
            value_type: Ok(content),
        }),
    );
    let outputs_parser = separated_list0(
        ws(tag(";")),
        parse_named_value_type().map(|(n, content)| OutputInfo {
            name: Some(n),
            value_type: Ok(content),
        }),
    );

    let fn_name = ws(parse_identifier());

    let fn_details = separated_pair(inputs_parser, ws(tag("=>")), outputs_parser);

    let assignment = separated_pair(fn_name, ws(tag("=")), fn_details);

    assignment.map(|(name, (inputs, outputs))| {
        (
            name,
            Ok(Rc::new(NodeTypeInfo {
                inputs,
                outputs,
                annotation: crate::execution::ExecutionInformation::ERR,
            })),
        )
    })
}

pub fn parse_node_type_declarations<'a>()
-> impl Parser<&'a [u8], Output = Vec<(String, FallibleNodeTypeRc)>, Error = Error<&'a [u8]>> {
    many0(ws(parse_node_type_declaration()))
}

pub fn parse_type_world(content: &str) -> Result<SimpleTypeWorld<FallibleNodeTypeRc>, ()> {
    let mut parser = terminated(parse_node_type_declarations(), eof);
    let res = match parser.parse_complete(content.as_bytes()) {
        Ok((_, ok)) => ok,
        Err(a) => panic!("{a}"),
    };

    let mut uni = SimpleTypeWorld::<FallibleNodeTypeRc>::new();
    for i in res {
        uni.node_types.insert(i.0, i.1);
    }

    Ok(uni)
}
