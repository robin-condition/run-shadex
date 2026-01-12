use nom::{
    Or, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        complete::{alpha1, alphanumeric0, space0},
        digit1, multispace0,
    },
    combinator::opt,
    error::{Error, ParseError},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
};

use crate::nodedef::ast::{
    ArithmeticOp, AssignmentStatement, CallExpression, FourArithmeticExpression, LambdaExpression,
    LiteralExpression, MemberExpression,
    full_untyped::{ArgName, ScopedIdentifier, UntypedBody, UntypedExpression, UntypedStatement},
};

fn space_or_comment<'a, E: ParseError<&'a [u8]>>() -> impl Parser<&'a [u8], Output = (), Error = E>
{
    many0(alt((
        tag(" "),
        tag("\r"),
        tag("\n"),
        tag("\t"),
        terminated(tag("//"), take_until("\n")),
    )))
    .map(|_| ())
}

// https://github.com/rust-bakery/nom/blob/main/examples/json2.rs
fn ws<'a, O, E: ParseError<&'a [u8]>, F: Parser<&'a [u8], Output = O, Error = E>>(
    f: F,
) -> impl Parser<&'a [u8], Output = O, Error = E> {
    delimited(space_or_comment(), f, space_or_comment())
}

fn parse_identifier<'a>() -> impl Parser<&'a [u8], Output = String, Error = Error<&'a [u8]>> {
    ws((alpha1, alphanumeric0)
        .map(|b| String::from_utf8_lossy(b.0).to_string() + String::from_utf8_lossy(b.1).as_ref()))
}

fn parse_u32<'a>() -> impl Parser<&'a [u8], Output = u32, Error = Error<&'a [u8]>> {
    ws(nom::character::complete::u32)
}

fn parse_opt_sign<'a>() -> impl Parser<&'a [u8], Output = i32, Error = Error<&'a [u8]>> {
    opt(ws(alt((tag("-").map(|_| -1), tag("+").map(|_| 1))))).map(|a| match a {
        Some(s) => s,
        None => 1,
    })
}

fn parse_i32<'a>() -> impl Parser<&'a [u8], Output = i32, Error = Error<&'a [u8]>> {
    (parse_opt_sign(), parse_u32()).map(|(a, b)| a * b as i32)
}

fn parse_f32<'a>() -> impl Parser<&'a [u8], Output = f32, Error = Error<&'a [u8]>> {
    ws(nom::number::float())
}

fn parse_assignment<'a>()
-> impl Parser<&'a [u8], Output = UntypedStatement, Error = Error<&'a [u8]>> {
    terminated(
        separated_pair(ws(parse_identifier()), ws(tag("=")), parse_expr()),
        ws(tag(";")),
    )
    .map(|(name, expr)| {
        UntypedStatement::Assignment(AssignmentStatement {
            id: name,
            rhs: expr,
        })
    })
}

fn parse_decl_assign<'a>()
-> impl Parser<&'a [u8], Output = UntypedStatement, Error = Error<&'a [u8]>> {
    delimited(
        ws(tag("let")),
        separated_pair(ws(parse_identifier()), ws(tag("=")), parse_expr()),
        ws(tag(";")),
    )
    .map(|(name, expr)| {
        UntypedStatement::DeclAssignment(AssignmentStatement {
            id: name,
            rhs: expr,
        })
    })
}

fn parse_stmt<'a>() -> impl Parser<&'a [u8], Output = UntypedStatement, Error = Error<&'a [u8]>> {
    alt((parse_assignment(), parse_decl_assign()))
}

fn parse_body<'a>() -> impl Parser<&'a [u8], Output = UntypedBody, Error = Error<&'a [u8]>> {
    delimited(ws(tag("{")), many0(parse_stmt()), ws(tag("}"))).map(|g| UntypedBody { stmts: g })
}

fn parse_lambda_decl<'a>()
-> impl Parser<&'a [u8], Output = UntypedExpression, Error = Error<&'a [u8]>> {
    let parse_arg = parse_identifier().map(|name| ArgName { name });
    let parse_args = delimited(
        ws(tag("(")),
        separated_list0(ws(tag(",")), parse_arg),
        ws(tag(")")),
    );
    separated_pair(parse_args, ws(tag("=>")), parse_body()).map(|(a, b)| {
        UntypedExpression::Lambda(LambdaExpression {
            args: a,
            body: b,
            caps: (),
        })
    })
}

pub fn parse_expr() -> ExprParser {
    ExprParser {}
}

fn parse_scoped_ident<'a>()
-> impl Parser<&'a [u8], Output = ScopedIdentifier, Error = Error<&'a [u8]>> {
    separated_list1(ws(tag("::")), parse_identifier()).map(|idents| {
        let mut res = ScopedIdentifier::Scopeless(idents[0].clone());
        for i in idents.into_iter().skip(1) {
            res = ScopedIdentifier::InScope(Box::new(res), i)
        }
        res
    })
}

fn parse_atom<'a>() -> impl Parser<&'a [u8], Output = UntypedExpression, Error = Error<&'a [u8]>> {
    alt((
        terminated(parse_f32(), tag("f32"))
            .map(|f| UntypedExpression::LiteralF32(LiteralExpression { v: f })),
        terminated(parse_u32(), tag("u32"))
            .map(|v| UntypedExpression::LiteralU32(LiteralExpression { v })),
        parse_i32().map(|v| UntypedExpression::LiteralI32(LiteralExpression { v })),
        parse_scoped_ident().map(UntypedExpression::ScopedIdentifier),
        parse_lambda_decl(),
        delimited(ws(tag("(")), parse_expr(), ws(tag(")"))),
    ))
}

enum FactorSuffix {
    MemberAccess(String),
    FnCall(Vec<(String, UntypedExpression)>),
}

fn parse_member_access_suffix<'a>()
-> impl Parser<&'a [u8], Output = String, Error = Error<&'a [u8]>> {
    preceded(ws(tag(".")), parse_identifier())
}

fn parse_fn_call_suffix<'a>()
-> impl Parser<&'a [u8], Output = Vec<(String, UntypedExpression)>, Error = Error<&'a [u8]>> {
    delimited(
        ws(tag("(")),
        separated_list0(
            ws(tag(",")),
            separated_pair(parse_identifier(), ws(tag(":")), parse_expr()),
        ),
        ws(tag(")")),
    )
}

fn parse_atom_with_suffices<'a>()
-> impl Parser<&'a [u8], Output = UntypedExpression, Error = Error<&'a [u8]>> {
    (
        parse_atom(),
        many0(alt((
            parse_member_access_suffix().map(FactorSuffix::MemberAccess),
            parse_fn_call_suffix().map(FactorSuffix::FnCall),
        ))),
    )
        .map(|(a, sfx)| {
            let mut res = a;
            for sf in sfx {
                res = match sf {
                    FactorSuffix::MemberAccess(name) => {
                        UntypedExpression::MemberAccess(MemberExpression {
                            owner: Box::new(res),
                            name,
                        })
                    }
                    FactorSuffix::FnCall(items) => UntypedExpression::Call(CallExpression {
                        fn_expr: Box::new(res),
                        args: items,
                    }),
                };
            }
            res
        })
}

fn parse_factor<'a>() -> impl Parser<&'a [u8], Output = UntypedExpression, Error = Error<&'a [u8]>>
{
    parse_atom_with_suffices()
}

pub fn parse_term<'a>() -> impl Parser<&'a [u8], Output = UntypedExpression, Error = Error<&'a [u8]>>
{
    (
        parse_factor(),
        many0((
            alt((
                ws(tag("*")).map(|_| ArithmeticOp::Mult),
                ws(tag("/")).map(|_| ArithmeticOp::Div),
            )),
            parse_factor(),
        )),
    )
        .map(|(strt, ops)| {
            let mut res = strt;
            for op in ops {
                res = UntypedExpression::Arithmetic(FourArithmeticExpression {
                    op: op.0,
                    left: Box::new(res),
                    right: Box::new(op.1),
                });
            }
            res
        })
}

pub fn parse_sum<'a>() -> impl Parser<&'a [u8], Output = UntypedExpression, Error = Error<&'a [u8]>>
{
    (
        parse_term(),
        many0((
            alt((
                ws(tag("+")).map(|_| ArithmeticOp::Add),
                ws(tag("-")).map(|_| ArithmeticOp::Sub),
            )),
            parse_term(),
        )),
    )
        .map(|(strt, ops)| {
            let mut res = strt;
            for op in ops {
                res = UntypedExpression::Arithmetic(FourArithmeticExpression {
                    op: op.0,
                    left: Box::new(res),
                    right: Box::new(op.1),
                });
            }
            res
        })
}

pub struct ExprParser;

impl<'a> Parser<&'a [u8]> for ExprParser {
    type Output = UntypedExpression;

    type Error = Error<&'a [u8]>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: &'a [u8],
    ) -> nom::PResult<OM, &'a [u8], Self::Output, Self::Error> {
        parse_sum().process::<OM>(input)
    }
}
