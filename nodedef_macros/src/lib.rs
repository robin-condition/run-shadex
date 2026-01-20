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

#[proc_macro]
pub fn include_nodedef(input: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro]
pub fn runtime_parse(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as LitStr);
    let inp_str = inp.value();

    let expanded = quote::quote! {
    {let mut parse = shadex_computation_definitions::nodedef::parsing::parse_expr().whole_file();
    let expr = parse.parse_complete(#inp_str).unwrap().1;
    expr
    }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
}
