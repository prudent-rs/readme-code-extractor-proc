#![doc = include_str!("../README.md")]

//use core::str::FromStr;
use proc_macro::TokenStream;
use proc_macro_rules::rules;
use proc_macro2::Literal;
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

/// Restriction: We support only config (toml) files that
/// - have paths
///   - specified as ordinary string literals `"..."`.
///     - whose characters/content (content of the path) don't include a quote '"' and some other
///       special character, including backslashes! (Its representation in an ordinary, non-raw Rust
///       string literal "...." (excluding the enclosing quotes) must be the same as it gets printed
///       (in common terminals/screens).)
///   - or: raw strings - TODO
///      - RAW strings ARE GOOD - NO ESCAPING!
///      - Good for backslashes and paths on Windows.
/// - are in UTF-8 (the config content).
///
/// Return content of the config (toml) file.
fn load_config_toml_file(config_toml_file_relative_path: &Literal) -> String {
    // There does exist
    // https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html#method.str_value, but
    // - it's unclear how to enable it (`procmacro2_semver_exempt` is NOT a feature); and
    // - it works with `nightly` Rust toolchain only.
    //
    // Instead,
    // - call `proc_macro2::Literal`'s
    //   [`to_string()`](https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html#impl-ToString-for-T)
    // - that returns `String`, whose **content** is enclosed within quotes `"` and any quotes (and
    //   special characters) are escaped.
    // - simply
    //   - if the string literal starts with a quote '"', remove the leading and trailing quotation
    //     marks (or, actually, slice it).
    //   - if the string literal starts with `r", r#", r##", r###"`, remove that and the appropriate
    //     trailing group `", "#, "xx, "xxx`.
    //
    // Hence a restriction mentioned in rustdoc of this function.
    let config_toml_file_path_enclosed = config_toml_file_relative_path.to_string();

    {
        // assertions
        let config_toml_file_path_enclosed_bytes = config_toml_file_path_enclosed.as_bytes();
        assert!(
            config_toml_file_path_enclosed_bytes[0] == b'"',
            "Expecting file path {config_toml_file_path_enclosed} to start with a quote \"."
        );

        assert!(
            config_toml_file_path_enclosed_bytes[config_toml_file_path_enclosed_bytes.len() - 1]
                == b'"',
            "Expecting file path {config_toml_file_path_enclosed} to end with a quote \"."
        );
    }

    let config_toml_file_path =
        &config_toml_file_path_enclosed[1..config_toml_file_path_enclosed.len() - 1];

    // Validate that the file path is compliant, by reversing the process, and then
    // comparing the original and the result `String`. We use
    // `proc_macro2::string(...).to_string()`.

    {
        //assertions
        let regenerated_file_path_literal = Literal::string(config_toml_file_path);
        let regenerated_file_path_enclosed = regenerated_file_path_literal.to_string();
        assert_eq!(
            config_toml_file_path_enclosed, regenerated_file_path_enclosed,
            "Can't parse/handle the given config (toml) file path literal (string) {}. It was \
             handled as {}.",
            config_toml_file_path_enclosed, regenerated_file_path_enclosed
        );
    }

    let cfg_file_path = {
        let invoker_file_path = config_toml_file_relative_path
            .span()
            .local_file()
            .unwrap_or_else(|| {
                // #TODO remove "all_by_file" from the erro message
                panic!(
                    "Rust source file that invoked readme_code_extractor::all_by_file! \
                     macro for config (toml) file with relative path \
                     {config_toml_file_relative_path} should have a known location."
                )
            });
        let invoker_parent_dir = invoker_file_path.parent().unwrap_or_else(|| {
            // #TODO remove "all_by_file" from the erro message
            panic!(
                "Rust source file that invoked readme_code_extractor::all_by_file! \
                 macro for config (toml) file with relative path {config_toml_file_relative_path} \
                 may exist, but we can't get its parent directory.",
            )
        });
        invoker_parent_dir.join(config_toml_file_path)
    };

    // Error handling is modelling https://doc.rust-lang.org/nightly/src/core/result.rs.html
    // > `fn unwrap_failed`, which invokes `panic!("{msg}: {error:?}");`
    std::fs::read_to_string(&cfg_file_path).unwrap_or_else(|e| {
        let cfg_file_path = cfg_file_path.to_str().unwrap_or("");
        panic!("Expecting a config (toml) file {cfg_file_path}, but opening it failed: {e:?}",)
    })
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
            let config_toml_content = load_config_toml_file(&config_toml_file_relative_path);

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
