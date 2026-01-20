use nom::Parser;
use proc_macro::{Literal, TokenStream, TokenTree};
use syn::{DeriveInput, Expr, LitStr, parse_macro_input};

#[proc_macro]
pub fn nodedef(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as LitStr);
    let inp_str = inp.value();
    let mut parse = shadex_computation_definitions::nodedef::parsing::parse_expr().whole_file();
    let expr = parse.parse_complete(&inp_str.as_str()).unwrap().1;

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
}
