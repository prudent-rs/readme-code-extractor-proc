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
        // @TODO instead of readme_file_path_literal, accept a TOML config text:
        //
        // $config_toml_content
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

// Invoked by `readme_code_extractor::all_by_file`.
#[doc(hidden)]
#[proc_macro]
pub fn all_by_file(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $config_toml_file_path:literal ) => {

            let span = config_toml_file_path.span();
            
            let config_toml_file_path = config_toml_file_path.to_string();
            // @TODO error handling (or not)
            let config_toml_content = std::fs::read_to_string(config_toml_file_path).unwrap();

            quote_spanned! {span=>
                ::readme_code_extractor_proc::all!(#config_toml_content)
            }
        }
    }).into()
}

#[proc_macro]
pub fn nth(_input: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro]
pub fn create_nth_extractor_macro(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $name_of_new_extractor_macro:ident, $config_toml_content:literal ) => {

            let span = config_toml_content.span();
            quote_spanned! {span=>
                // We do HAVE TO `macro_export` it. Otherwise it can't be used in `#[doc = ... ]` or
                // `#![doc = ... ]` (which are processed in a crate separate to the crate that
                // called `all_by_file` macro).
                #[macro_export]
                macro_rules! #name_of_new_extractor_macro {
                    ($n:literal) => {
                        ::readme_code_extractor_proc::nth!($n, #config_toml_content);
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
