#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro_rules::rules;
use quote::{quote, quote_spanned};
//use readme_code_extractor_lib::types::Config;
use core::str::FromStr;

#[doc(hidden)]
const _ASSERT_README_CODE_EXTRACTOR_LIB_VERSION: () = {
    if !readme_code_extractor_lib::is_exact_version(env!("CARGO_PKG_VERSION")) {
        panic!("prudent-rs/readme-code-extractor-lib is of different version than prudent-rs/readme-code-extractor-proc. Please report this as an issue, along with both versions.");
    }
};

#[proc_macro]
pub fn all(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $readme_file_path_literal:literal ) => {

            let span = readme_file_path_literal.span();
            // @TODO:
            let _ = span.local_file();

            let file_content = "content";
            let _ts = TokenStream::from_str(file_content).unwrap();

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

#[proc_macro]
pub fn nth(_input: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro]
pub fn create_nth_extractor_macro(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $name_of_new_extractor_macro:ident, $config_file_path:literal ) => {

            let span = config_file_path.span();
            quote_spanned! {span=>
                #[macro_export]
                macro_rules! #name_of_new_extractor_macro {
                    ($n:literal) => {
                        ::readme_code_extractor_proc::nth!($config_file_path);
                        let _ = $$a;
                    };
                }
            }
        }
    }).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn version(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        () => {
            let version = env!("CARGO_PKG_VERSION");
            quote! {
                #version
            }
        }
    })
    .into()
}
