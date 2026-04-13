#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro_rules::rules;
use quote::quote_spanned;

#[proc_macro]
pub fn black_box(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $file_path_literal:literal ) => {

            let span = file_path_literal.span();
            // @TODO:
            let _ = span.local_file();

            let s = "Hi";
            let mut q = quote_spanned! {span=>
                #s
            };
            let q2 = quote_spanned! {span=>
            };
            q.extend( q2);
            q
        }
        ( $file_path_literal:literal,

        ) => {

            let span = file_path_literal.span();

            let s = "Hi";
            let mut q = quote_spanned! {span=>
                #s
            };
            let q2 = quote_spanned! {span=>
            };
            q.extend( q2);
            q
        }
    })
    .into()
}
