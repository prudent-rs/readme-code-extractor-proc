#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::Literal;
use proc_macro_rules::rules;
use quote::{quote, quote_spanned};
//use readme_code_extractor_lib::types::Config;
use core::str::FromStr;
//use std::path::Path;

#[doc(hidden)]
const _ASSERT_README_CODE_EXTRACTOR_LIB_VERSION: () = {
    if !readme_code_extractor_lib::is_exact_version(env!("CARGO_PKG_VERSION")) {
        panic!(
            r"prudent-rs/readme-code-extractor-lib is of different version than
                 prudent-rs/readme-code-extractor-proc. Please report this as an issue, along with
                 both versions."
        );
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
            // @TODO construct the file path
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
        ( $config_toml_file_relative_path:literal ) => {

            let span = config_toml_file_relative_path.span();

            // There does exist
            // https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html#method.str_value,
            // but
            // - it's unclear how to enable it (`procmacro2_semver_exempt` is NOT a feature); and
            // - it works with `nightly` Rust toolchain only.
            //
            // Instead,
            // - we call `proc_macro2::Literal`'s
            //   [`to_string()`](https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html#impl-ToString-for-T)
            // - that returns `String`, whose **content** is enslosed within quotes `"` and any
            //   quotes (and special characters) are escaped.
            // - we simply remove the leading and trailing quotation mark `"` (or, actually, slice
            //   it).
            //
            // Hence a restriction: We support only such paths to config (toml) files that don't
            // include a quote '"' and maybe other special characters. (Its representation in an
            // ordinary, non-raw Rust string literal "...." must be the same as it gets printed (in
            // common terminals/screens).)
            let config_toml_file_path_enclosed = config_toml_file_relative_path.to_string();

            let config_toml_file_path_enclosed_bytes = config_toml_file_path_enclosed.as_bytes();
            assert!( config_toml_file_path_enclosed_bytes[0]==b'"',
                "Expecting file path {} to start with an enclosing quote \".",
                config_toml_file_path_enclosed);

            assert!( config_toml_file_path_enclosed_bytes[
                        config_toml_file_path_enclosed_bytes.len()-1
                    ]==b'"',
                    "Expecting file path {} to end with an enclosing quote \".",
                    config_toml_file_path_enclosed);

            let config_toml_file_path = &config_toml_file_path_enclosed[
                1..config_toml_file_path_enclosed.len()-1
            ];

            // Validate that the file path is compliant, by reversing the process, and then
            // comparing the original and the result `String`. We use
            // `proc_macro2::string(...).to_string()`.

            let regenerated_file_path_literal = Literal::string(config_toml_file_path);
            let regenerated_file_path_enclosed = regenerated_file_path_literal.to_string();
            assert_eq!(config_toml_file_path_enclosed, regenerated_file_path_enclosed,
                r"Can't parse/handle the given config (toml) file path literal (string) {}. It was 
                  handled as {}.", config_toml_file_path_enclosed, regenerated_file_path_enclosed);

            let file_path = span.local_file().unwrap_or_else(|| {
                panic!(r"Rust source file that invoked readme_code_extractor::all_by_file! macro
                         for config (toml) file with relative path {} should have a known location.", config_toml_file_relative_path)
            });
            let parent_dir = file_path.parent().unwrap_or_else(|| {
                panic!(r"Rust source file that invoked readme_code_extractor::all_by_file! macro for
                         config (toml) file with relative path {} may exist, but we can't get its
                         parent directory.",
                         config_toml_file_relative_path)
            });
            let cfg_file_path = parent_dir.join( config_toml_file_path );

            // Error handling is modelling https://doc.rust-lang.org/nightly/src/core/result.rs.html
            // > `fn unwrap_failed`, which invokes `panic!("{msg}: {error:?}");`
            let config_toml_content = std::fs::read_to_string(&cfg_file_path).unwrap_or_else(|e| {
                let cfg_file_path = cfg_file_path.to_str().unwrap_or("");
                panic!("Expecting a config (toml) file {}, but opening it failed: {:?}",
                    cfg_file_path, e)
            });

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
    })
    .into()
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
