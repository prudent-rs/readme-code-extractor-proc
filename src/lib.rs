#![doc = include_str!("../README.md")]

//use core::str::FromStr;
use proc_macro::TokenStream;
use proc_macro_rules::rules;
//use proc_macro2::Literal;
use quote::{quote, quote_spanned};
//use readme_code_extractor_lib::traits::Config;
//use std::path::Path;

const _ASSERT_README_CODE_EXTRACTOR_LIB_VERSION: () = {
    if !readme_code_extractor_lib::is_exact_version(env!("CARGO_PKG_VERSION")) {
        // See prudent-rs/readme_code_extractor -> src/lib.rs for an explanation on why here we
        // can't report more details.
        panic!(
            "prudent-rs/readme-code-extractor-proc is of different version than \
             prudent-rs/readme-code-extractor-lib. Please report this as an issue, along with \
             both versions."
        );
    }
};

/// NOT public - for testing of [readme_code_extractor_lib::load_file] only. See
/// [readme_code_extractor_lib::load_file].
#[doc(hidden)]
#[proc_macro]
pub fn test_load_file(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $literal:literal ) => {
            let content = readme_code_extractor_lib::load_file(&literal);
            quote! {
                #content
            }
        }
    })
    .into()
}

/// Process all code blocks in the given input.
///
/// The given input is
/// - specified as an ordinary, non-raw, string literals `"..."`. Ordinary string literals
///   - are good for multiline
///     - even though you do need to add a trailing backslash '\'
///     - they remove any leading whitespace on the second and successive lines
///   - need escaping of quote character '"" and backslash chacter '\'
/// - or: a raw string
///   - RAW strings ARE GOOD - NO ESCAPING!
///   - Good for backslashes and paths on Windows.
///   - Good for multiline: No need to add a trailing backslash on each line (other than the last
///     line).
///   - BAD for multiline: The leading indentation is NOT removed. So, you want the content to start
///     on a new line! But, such macros are likely to be used at their file's top level (rather than
///     in a module or a function), so the raw string's actual content starting on a new line at
///     column 0 should look OK.
#[proc_macro]
pub fn all(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $config_toml_content:literal ) => {

            let span = config_toml_content.span();
            // @TODO:
            let _ = span.local_file();

            let _file_content = "content";
            // @TODO construct the file path
            //let _ts = TokenStream::from_str(file_content).unwrap();

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

// @TODO remove
/*#[proc_macro]
pub fn dbg_print_span_of(input: TokenStream) -> TokenStream {
    use proc_macro::TokenTree;
    for tree in input {
        match tree {
            TokenTree::Literal(literal) => {
                let span = literal.span();
                panic!(
                    "span.local_file: {:?}, span.file: {:?}",
                    span.local_file(),
                    span.file()
                )
            }
            _ => {}
        }
    }
    panic!();
    /*rules!(input.into() => {
        ( $literal:literal ) => {

            let span = literal.span();
            if true {
                //let local_file = span.local_file().map_or(|path| path.to_string(), "None".to_owned());
                panic!( "span.local_file: {:?}, span.file: {:?}", span.local_file(), span.file())
            }
            quote! {}
        }
    }).into()*/
}
*/

// Invoked by `readme_code_extractor::all_by_file`.
#[doc(hidden)]
#[proc_macro]
pub fn all_by_file(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        ( $config_toml_file_relative_path:literal ) => {

            let span = config_toml_file_relative_path.span();
            let config_toml_content = readme_code_extractor_lib::load_file(
                &config_toml_file_relative_path);

            quote_spanned! {span=>
                ::readme_code_extractor_proc::all!(#config_toml_content)
            }
        }
    })
    .into()
}

#[proc_macro]
pub fn nth(_input: TokenStream) -> TokenStream {
    todo!()
}

/// This is like `readme_code_extractor::all_by_file``, except that `all_by_file` is a declarative
/// macro (macro by example, defined by `macro_rules`). However, [create_nth_extractor_macro] can't
/// be declarative. Why? Because it itself defines a new declarative macro which needs to have a
/// capturing variable (parameter). Capturing variables (parameters) start with a dollar character,
/// but if [create_nth_extractor_macro] itself were a declarative macro, it couldn't generate/return
/// a dollar character.
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
    })
    .into()
}

#[doc(hidden)]
#[proc_macro]
pub fn version(input: TokenStream) -> TokenStream {
    rules!(input.into() => {
        () => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            quote! {
                #VERSION
            }
        }
    })
    .into()
}
