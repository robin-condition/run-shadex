use std::collections::HashMap;

use nom::{
    Or, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        complete::{alpha1, alphanumeric0, space0},
        digit1, multispace0,
    },
    combinator::{all_consuming, opt},
    error::{Error, ParseError},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
};

type InputSpan<'a> = &'a str;
type MyError<'a> = Error<InputSpan<'a>>;

use crate::nodedef::ast::{
    AnnotatedExpression, ArithmeticOp, AssignmentStatement, CallExpression,
    FourArithmeticExpression, LambdaExpression, LiteralExpression, LiteralExpressionNumber,
    MemberExpression, StructExpression,
    full_untyped::{
        GlobalUntypedExprDefs, ScopedIdentifier, UntypedBody, UntypedExpression, UntypedStatement,
    },
    mathy_ast::ArithmeticOrLiteralOrId,
};

fn space_or_comment<'a, E: ParseError<InputSpan<'a>>>()
-> impl Parser<InputSpan<'a>, Output = (), Error = E> {
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
fn ws<'a, O, E: ParseError<InputSpan<'a>>, F: Parser<InputSpan<'a>, Output = O, Error = E>>(
    f: F,
) -> impl Parser<InputSpan<'a>, Output = O, Error = E> {
    delimited(space_or_comment(), f, space_or_comment())
}

fn parse_identifier<'a>() -> impl Parser<InputSpan<'a>, Output = String, Error = MyError<'a>> {
    ws((alpha1, alphanumeric0)
        // .map(|b| String::from_utf8_lossy(b.0).to_string() + String::from_utf8_lossy(b.1).as_ref()))
        .map(|b: (&str, &str)| b.0.to_string() + b.1))
}

fn parse_u32<'a>() -> impl Parser<InputSpan<'a>, Output = u32, Error = MyError<'a>> {
    ws(nom::character::complete::u32)
}

fn parse_opt_sign<'a>() -> impl Parser<InputSpan<'a>, Output = i32, Error = MyError<'a>> {
    opt(ws(alt((tag("-").map(|_| -1), tag("+").map(|_| 1))))).map(|a| match a {
        Some(s) => s,
        None => 1,
    })
}

fn parse_i32<'a>() -> impl Parser<InputSpan<'a>, Output = i32, Error = MyError<'a>> {
    (parse_opt_sign(), parse_u32()).map(|(a, b)| a * b as i32)
}

fn parse_f32<'a>() -> impl Parser<InputSpan<'a>, Output = f32, Error = MyError<'a>> {
    ws(nom::number::float())
}

/*fn parse_assignment<'a>()
-> impl Parser<InputSpan<'a>, Output = UntypedStatement, Error = MyError<'a>> {
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
}*/

fn parse_decl_assign<'a>()
-> impl Parser<InputSpan<'a>, Output = UntypedStatement, Error = MyError<'a>> {
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

fn parse_stmt<'a>() -> impl Parser<InputSpan<'a>, Output = UntypedStatement, Error = MyError<'a>> {
    //alt((parse_assignment(), parse_decl_assign()))
    parse_decl_assign()
}

fn parse_body<'a>() -> impl Parser<InputSpan<'a>, Output = UntypedBody, Error = MyError<'a>> {
    delimited(
        ws(tag("{")),
        (many0(parse_stmt()), ws(opt(parse_expr()))),
        ws(tag("}")),
    )
    .map(|g| UntypedBody {
        stmts: g.0,
        end_expr: g.1.map(Box::new),
    })
}

fn parse_lambda_decl<'a>()
-> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>> {
    let parse_arg = parse_identifier();
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

fn parse_struct_ctor<'a>()
-> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>> {
    delimited(
        ws(tag("(")),
        separated_list1(
            ws(tag(",")),
            separated_pair(parse_identifier(), ws(tag(":")), parse_expr()),
        ),
        ws(tag(")")),
    )
    .map(|flds| UntypedExpression::StructConstructor(StructExpression { fields: flds }))
}

pub fn parse_expr() -> ExprParser {
    ExprParser {}
}

fn parse_scoped_ident<'a>()
-> impl Parser<InputSpan<'a>, Output = ScopedIdentifier, Error = MyError<'a>> {
    separated_list1(ws(tag("::")), parse_identifier()).map(|idents| {
        let mut res = ScopedIdentifier::Scopeless(idents[0].clone());
        for i in idents.into_iter().skip(1) {
            res = ScopedIdentifier::InScope(Box::new(res), i)
        }
        res
    })
}

fn parse_atom<'a>() -> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>> {
    alt((
        terminated(parse_f32(), tag("f32"))
            .map(|f| {
                ArithmeticOrLiteralOrId::Literal(LiteralExpressionNumber::LiteralF32(
                    LiteralExpression { v: f },
                ))
            })
            .map(UntypedExpression::Arithmetic),
        terminated(parse_u32(), tag("u32"))
            .map(|v| {
                ArithmeticOrLiteralOrId::Literal(LiteralExpressionNumber::LiteralU32(
                    LiteralExpression { v },
                ))
            })
            .map(UntypedExpression::Arithmetic),
        parse_i32()
            .map(|v| {
                ArithmeticOrLiteralOrId::Literal(LiteralExpressionNumber::LiteralI32(
                    LiteralExpression { v },
                ))
            })
            .map(UntypedExpression::Arithmetic),
        parse_scoped_ident()
            .map(ArithmeticOrLiteralOrId::Id)
            .map(UntypedExpression::Arithmetic),
        parse_lambda_decl(),
        delimited(ws(tag("(")), parse_expr(), ws(tag(")"))),
        parse_struct_ctor(),
    ))
}

enum FactorSuffix {
    MemberAccess(String),
    FnCall(Vec<(String, UntypedExpression)>),
    Annotation(Vec<String>),
}

fn parse_member_access_suffix<'a>()
-> impl Parser<InputSpan<'a>, Output = String, Error = MyError<'a>> {
    preceded(ws(tag(".")), parse_identifier())
}

fn parse_annotation_suffix<'a>()
-> impl Parser<InputSpan<'a>, Output = Vec<String>, Error = MyError<'a>> {
    delimited(
        ws(tag("<")),
        separated_list0(ws(tag(",")), parse_identifier()),
        ws(tag(">")),
    )
}

fn parse_fn_call_suffix<'a>()
-> impl Parser<InputSpan<'a>, Output = Vec<(String, UntypedExpression)>, Error = MyError<'a>> {
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
-> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>> {
    (
        parse_atom(),
        many0(alt((
            parse_member_access_suffix().map(FactorSuffix::MemberAccess),
            parse_fn_call_suffix().map(FactorSuffix::FnCall),
            parse_annotation_suffix().map(FactorSuffix::Annotation),
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
                    FactorSuffix::Annotation(annotations) => {
                        UntypedExpression::AnnotatedExpression(AnnotatedExpression {
                            src: Box::new(res),
                            annotations,
                        })
                    }
                };
            }
            res
        })
}

fn parse_factor<'a>() -> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>>
{
    parse_atom_with_suffices()
}

pub fn parse_term<'a>()
-> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>> {
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
                res = UntypedExpression::Arithmetic(ArithmeticOrLiteralOrId::Arithmetic(
                    FourArithmeticExpression {
                        op: op.0,
                        left: Box::new(res),
                        right: Box::new(op.1),
                    },
                ));
            }
            res
        })
}

pub fn parse_sum<'a>() -> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>>
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
                res = UntypedExpression::Arithmetic(ArithmeticOrLiteralOrId::Arithmetic(
                    FourArithmeticExpression {
                        op: op.0,
                        left: Box::new(res),
                        right: Box::new(op.1),
                    },
                ));
            }
            res
        })
}

pub fn parse_comparator_level<'a>()
-> impl Parser<InputSpan<'a>, Output = UntypedExpression, Error = MyError<'a>> {
    (
        parse_sum(),
        opt((
            alt((
                ws(tag("==")).map(|_| ArithmeticOp::Eq),
                ws(tag(">=")).map(|_| ArithmeticOp::Geq),
                ws(tag("<=")).map(|_| ArithmeticOp::Leq),
            )),
            parse_sum(),
        )),
    )
        .map(|(strt, ops)| {
            let mut res = strt;
            if let Some(op) = ops {
                res = UntypedExpression::Arithmetic(ArithmeticOrLiteralOrId::Arithmetic(
                    FourArithmeticExpression {
                        op: op.0,
                        left: Box::new(res),
                        right: Box::new(op.1),
                    },
                ));
            }
            res
        })
}

pub struct ExprParser;

impl<'a> Parser<InputSpan<'a>> for ExprParser {
    type Output = UntypedExpression;

    type Error = MyError<'a>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: InputSpan<'a>,
    ) -> nom::PResult<OM, InputSpan<'a>, Self::Output, Self::Error> {
        parse_comparator_level().process::<OM>(input)
    }
}

fn parse_global_def<'a>()
-> impl Parser<InputSpan<'a>, Error = MyError<'a>, Output = (String, UntypedExpression)> {
    preceded(
        ws(tag("DEF")),
        separated_pair(parse_identifier(), ws(tag(":")), parse_expr()),
    )
}

fn parse_global_def_file<'a>()
-> impl Parser<InputSpan<'a>, Error = MyError<'a>, Output = GlobalUntypedExprDefs> {
    all_consuming(many0(parse_global_def()).map(|v| {
        let name_vec = v.iter().map(|a| a.0.clone()).collect();
        GlobalUntypedExprDefs {
            map: v.into_iter().collect(),
            names: name_vec,
        }
    }))
}

pub fn parse_global_def_file_specific<'a>(
    inp: InputSpan<'a>,
) -> Result<GlobalUntypedExprDefs, nom::Err<MyError<'a>>> {
    parse_global_def_file().parse_complete(inp).map(|f| f.1)
}
