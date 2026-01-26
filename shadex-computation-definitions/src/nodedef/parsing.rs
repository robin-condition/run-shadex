use std::collections::HashMap;

use nom::{
    AsChar, IResult, Input, Or, Parser,
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
    AnnotatedExpression, ArithmeticOp, AssignmentStatement, BoolValuedOp, CallExpression,
    FourArithmeticExpression, LambdaExpression, LiteralExpression, LiteralExpressionNumber, MathOp,
    MemberExpression, StructExpression,
    full_maybetyped::{
        GlobalMaybetypedExprDefs, MaybetypedBody, MaybetypedExpression, MaybetypedExpressionShape,
        MaybetypedStatement, ScopedIdentifier,
    },
    mathy_ast::ArithmeticOrLiteralOrId,
    typing::Type,
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

fn identifier_start_chars<T, E: ParseError<T>>(inp: T) -> IResult<T, T, E>
where
    T: Input<Item = char>,
{
    inp.split_at_position1_complete(
        |c| !(c.is_alpha() || c == '_'),
        nom::error::ErrorKind::Alpha,
    )
}

fn identifier_rest_chars<T, E: ParseError<T>>(inp: T) -> IResult<T, T, E>
where
    T: Input<Item = char>,
{
    inp.split_at_position_complete(|c| !(c.is_alpha() || c.is_dec_digit() || c == '_'))
}

fn parse_identifier<'a>() -> impl Parser<InputSpan<'a>, Output = String, Error = MyError<'a>> {
    ws((identifier_start_chars, identifier_rest_chars)
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
-> impl Parser<InputSpan<'a>, Output = MaybetypedStatement, Error = MyError<'a>> {
    terminated(
        separated_pair(ws(parse_identifier()), ws(tag("=")), parse_expr()),
        ws(tag(";")),
    )
    .map(|(name, expr)| {
        MaybetypedStatement::Assignment(AssignmentStatement {
            id: name,
            rhs: expr,
        })
    })
}*/

fn parse_decl_assign<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedStatement, Error = MyError<'a>> {
    delimited(
        ws(tag("let")),
        separated_pair(ws(parse_identifier()), ws(tag("=")), parse_expr()),
        ws(tag(";")),
    )
    .map(|(name, expr)| {
        MaybetypedStatement::DeclAssignment(AssignmentStatement {
            id: name,
            rhs: expr,
        })
    })
}

fn parse_stmt<'a>() -> impl Parser<InputSpan<'a>, Output = MaybetypedStatement, Error = MyError<'a>>
{
    //alt((parse_assignment(), parse_decl_assign()))
    parse_decl_assign()
}

fn parse_body<'a>() -> impl Parser<InputSpan<'a>, Output = MaybetypedBody, Error = MyError<'a>> {
    delimited(
        ws(tag("{")),
        (many0(parse_stmt()), ws(opt(parse_expr()))),
        ws(tag("}")),
    )
    .map(|g| MaybetypedBody {
        stmts: g.0,
        end_expr: g.1.map(Box::new),
    })
}

fn parse_type_literal<'a>() -> impl Parser<InputSpan<'a>, Output = Type, Error = MyError<'a>> {
    ws(alt((
        tag("f32").map(|_| Type::F32),
        tag("u32").map(|_| Type::U32),
    )))
}

fn parse_lambda_decl<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
    let parse_arg = (
        parse_identifier(),
        opt(preceded(ws(tag(":")), parse_type_literal())),
    );
    let parse_args = delimited(
        ws(tag("(")),
        separated_list0(ws(tag(",")), parse_arg),
        ws(tag(")")),
    );
    separated_pair(parse_args, ws(tag("=>")), parse_body()).map(|(a, b)| MaybetypedExpression {
        shape: MaybetypedExpressionShape::Lambda(LambdaExpression {
            args: a,
            body: b,
            caps: None,
        }),
        typ: None,
    })
}

fn parse_struct_ctor<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
    delimited(
        ws(tag("(")),
        separated_list1(
            ws(tag(",")),
            separated_pair(parse_identifier(), ws(tag(":")), parse_expr()),
        ),
        ws(tag(")")),
    )
    .map(|flds| MaybetypedExpression {
        shape: MaybetypedExpressionShape::StructConstructor(StructExpression { fields: flds }),
        typ: None,
    })
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

fn parse_atom<'a>() -> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>>
{
    alt((
        terminated(parse_f32(), tag("f32"))
            .map(|f| {
                ArithmeticOrLiteralOrId::Literal(LiteralExpressionNumber::LiteralF32(
                    LiteralExpression { v: f },
                ))
            })
            .map(MaybetypedExpressionShape::Arithmetic)
            .map(MaybetypedExpression::typeless),
        terminated(parse_u32(), tag("u32"))
            .map(|v| {
                ArithmeticOrLiteralOrId::Literal(LiteralExpressionNumber::LiteralU32(
                    LiteralExpression { v },
                ))
            })
            .map(MaybetypedExpressionShape::Arithmetic)
            .map(MaybetypedExpression::typeless),
        parse_i32()
            .map(|v| {
                ArithmeticOrLiteralOrId::Literal(LiteralExpressionNumber::LiteralI32(
                    LiteralExpression { v },
                ))
            })
            .map(MaybetypedExpressionShape::Arithmetic)
            .map(MaybetypedExpression::typeless),
        parse_scoped_ident()
            .map(ArithmeticOrLiteralOrId::Id)
            .map(MaybetypedExpressionShape::Arithmetic)
            .map(MaybetypedExpression::typeless),
        parse_lambda_decl(),
        delimited(ws(tag("(")), parse_expr(), ws(tag(")"))),
        parse_struct_ctor(),
    ))
}

enum FactorSuffix {
    MemberAccess(String),
    FnCall(Vec<(String, MaybetypedExpression)>),
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
-> impl Parser<InputSpan<'a>, Output = Vec<(String, MaybetypedExpression)>, Error = MyError<'a>> {
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
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
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
                    FactorSuffix::MemberAccess(name) => MaybetypedExpression::typeless(
                        MaybetypedExpressionShape::MemberAccess(MemberExpression {
                            owner: Box::new(res),
                            name,
                        }),
                    ),
                    FactorSuffix::FnCall(items) => MaybetypedExpression::typeless(
                        MaybetypedExpressionShape::Call(CallExpression {
                            fn_expr: Box::new(res),
                            args: items,
                        }),
                    ),
                    FactorSuffix::Annotation(annotations) => MaybetypedExpression::typeless(
                        MaybetypedExpressionShape::AnnotatedExpression(AnnotatedExpression {
                            src: Box::new(res),
                            annotations,
                        }),
                    ),
                };
            }
            res
        })
}

fn parse_factor<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
    parse_atom_with_suffices()
}

pub fn parse_term<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
    (
        parse_factor(),
        many0((
            alt((
                ws(tag("*")).map(|_| MathOp::Arith(ArithmeticOp::Mult)),
                ws(tag("/")).map(|_| MathOp::Arith(ArithmeticOp::Div)),
            )),
            parse_factor(),
        )),
    )
        .map(|(strt, ops)| {
            let mut res = strt;
            for op in ops {
                res = MaybetypedExpression::typeless(MaybetypedExpressionShape::Arithmetic(
                    ArithmeticOrLiteralOrId::Arithmetic(FourArithmeticExpression {
                        op: op.0,
                        left: Box::new(res),
                        right: Box::new(op.1),
                    }),
                ));
            }
            res
        })
}

pub fn parse_sum<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
    (
        parse_term(),
        many0((
            alt((
                ws(tag("+")).map(|_| MathOp::Arith(ArithmeticOp::Add)),
                ws(tag("-")).map(|_| MathOp::Arith(ArithmeticOp::Sub)),
            )),
            parse_term(),
        )),
    )
        .map(|(strt, ops)| {
            let mut res = strt;
            for op in ops {
                res = MaybetypedExpression::typeless(MaybetypedExpressionShape::Arithmetic(
                    ArithmeticOrLiteralOrId::Arithmetic(FourArithmeticExpression {
                        op: op.0,
                        left: Box::new(res),
                        right: Box::new(op.1),
                    }),
                ));
            }
            res
        })
}

pub fn parse_comparator_level<'a>()
-> impl Parser<InputSpan<'a>, Output = MaybetypedExpression, Error = MyError<'a>> {
    (
        parse_sum(),
        opt((
            alt((
                ws(tag("==")).map(|_| MathOp::Comp(BoolValuedOp::Eq)),
                ws(tag(">=")).map(|_| MathOp::Comp(BoolValuedOp::Geq)),
                ws(tag("<=")).map(|_| MathOp::Comp(BoolValuedOp::Leq)),
            )),
            parse_sum(),
        )),
    )
        .map(|(strt, ops)| {
            let mut res = strt;
            if let Some(op) = ops {
                res = MaybetypedExpression::typeless(MaybetypedExpressionShape::Arithmetic(
                    ArithmeticOrLiteralOrId::Arithmetic(FourArithmeticExpression {
                        op: op.0,
                        left: Box::new(res),
                        right: Box::new(op.1),
                    }),
                ));
            }
            res
        })
}

pub struct ExprParser;

impl<'a> Parser<InputSpan<'a>> for ExprParser {
    type Output = MaybetypedExpression;

    type Error = MyError<'a>;

    fn process<OM: nom::OutputMode>(
        &mut self,
        input: InputSpan<'a>,
    ) -> nom::PResult<OM, InputSpan<'a>, Self::Output, Self::Error> {
        parse_comparator_level().process::<OM>(input)
    }
}

impl ExprParser {
    pub fn whole_file<'a>(
        self,
    ) -> impl Parser<InputSpan<'a>, Error = MyError<'a>, Output = MaybetypedExpression> {
        all_consuming(self)
    }
}

fn parse_global_def<'a>()
-> impl Parser<InputSpan<'a>, Error = MyError<'a>, Output = (String, MaybetypedExpression)> {
    preceded(
        ws(tag("DEF")),
        separated_pair(parse_identifier(), ws(tag(":")), parse_expr()),
    )
}

fn parse_global_def_file<'a>()
-> impl Parser<InputSpan<'a>, Error = MyError<'a>, Output = GlobalMaybetypedExprDefs> {
    all_consuming(many0(parse_global_def()).map(|v| {
        let name_vec = v.iter().map(|a| a.0.clone()).collect();
        GlobalMaybetypedExprDefs {
            map: v.into_iter().collect(),
            names: name_vec,
        }
    }))
}

pub fn parse_global_def_file_specific<'a>(
    inp: InputSpan<'a>,
) -> Result<GlobalMaybetypedExprDefs, nom::Err<MyError<'a>>> {
    parse_global_def_file().parse_complete(inp).map(|f| f.1)
}
