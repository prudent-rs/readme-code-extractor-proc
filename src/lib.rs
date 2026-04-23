#![doc = include_str!("../README.md")]

use core::str::FromStr;
use proc_macro::TokenStream as ProcTokenStream;
use proc_macro_rules::rules;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use readme_code_extractor_lib::public::{Config, ConfigAndSpan, ReadmeBlock, ReadmeExtracted};

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

// @TODO pass first param: file path
/// NOT public - for testing of [readme_code_extractor_lib::load_file] only. See
/// [readme_code_extractor_lib::load_file].
/*#[doc(hidden)]
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
}*/

macro_rules! token_stream_from_str {
    ($input_string:expr, $err_intended_result_description:expr) => {
        ({
            let input_string = $input_string;
            let result = TokenStream::from_str(input_string);
            if let Err(e) = result {
                panic!(
                    "readme-code-extractor-proc: Parsing {} failed. Unpaired or incorrect Rust \n
                     tokens. Input:\n{}\nError: {}",
                    $err_intended_result_description, input_string, e
                );
            }
            result.unwrap()
        })
    };
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
pub fn all(input: ProcTokenStream) -> ProcTokenStream {
    rules!(input.into() => {
        ( $config_toml_content:literal ) => {

            let config_content_and_span = readme_code_extractor_lib::public::config_content_and_span(&config_toml_content);
            let config_and_span = readme_code_extractor_lib::public::config_and_span(&config_content_and_span);
            let readme_loaded = readme_code_extractor_lib::public::readme_load(&config_and_span);
            let readme_extracted = readme_code_extractor_lib::public::readme_extract(&readme_loaded);

            let config = config_and_span.config();

            // @TODO use
            /*let _preamble_text= if let Some(preamble_text) = readme_extracted.preamble_text() {
                quote_spanned! {span=>
                    //@TODO
                }
            } else {
                TokenStream::new()
            };*/
            // @TODO
            /*let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {
                quote_spanned! {span=>
                    //@TODO
                }
            } else {
                TokenStream::new()
            };*/

            let _/*start_prefix*/ = if config.start_prefix().len() > 0 {
                token_stream_from_str!( config.start_prefix(), "Config::start_prefix")
            } else {
                TokenStream::new()
            };//@TODO use
            //-----

            //--
            /*let s = "Hi";
            let mut q = quote_spanned! {span=>
                #s
            };
            let q2 = quote_spanned! {span=>
            };
            q.extend( q2);
            q*/
            impl_all(config, readme_extracted, config_toml_content.span())
        }
    })
}

fn impl_all<'a>(
    config: &dyn Config,
    mut readme_extracted: impl ReadmeExtracted<'a>,
    span: Span,
) -> ProcTokenStream {
    let (inserts, after_insert) = if let Some(headers) = config.ordinary_code_headers()
        && let Some(inserts) = headers.inserts()
    {
        (inserts.inserts(), inserts.after_insert())
    } else {
        (&[][..], "")
    };

    let (prefix_before_insert, max_insert_len) =
        if let Some(headers) = config.ordinary_code_headers() {
            (
                headers.prefix_before_insert(),
                inserts.iter().map(|&s| s.len()).max().unwrap_or(0),
            )
        } else {
            ("", 0usize)
        };
    let ordinary_code_suffix = config.ordinary_code_suffix();

    let config_generated_len_per_block = prefix_before_insert.len()
        + max_insert_len
        + after_insert.len()
        + ordinary_code_suffix.len();

    let blocks = readme_extracted.non_preamble_blocks().collect::<Vec<_>>();

    // @TODO apply backtick suffixes like "ignore" or "norun"
    let mut code_blocks = Vec::with_capacity(blocks.len() / 2 + 1);
    code_blocks.extend(blocks.iter().filter_map(ReadmeBlock::code));

    assert_eq!(
        code_blocks.len(),
        inserts.len(),
        "Expecting number of blocks {} and number of inserts {} to be the same!",
        code_blocks.len(),
        inserts.len()
    );

    let max_code_block_len = code_blocks
        .iter()
        .map(|b| b.code().len())
        .max()
        .unwrap_or(0);
    // @TODO triple_backtick_suffix
    let mut code = String::with_capacity(config_generated_len_per_block + max_code_block_len);

    // @TODO preamble etc.
    let total_code_blocks_len = code_blocks.iter().map(|b| b.code().len()).sum::<usize>();
    // We don't count the length of all inserts. Using the maximum is good enough.
    let mut all_code =
        String::with_capacity(total_code_blocks_len + max_code_block_len * code_blocks.len());

    for (&block, &insert) in code_blocks.iter().zip(inserts.iter()) {
        code.clear();
        // @TODO triple_backtick_suffix
        code.push_str(prefix_before_insert);
        // @TODO insert
        code.push_str(insert);
        code.push_str(after_insert);
        code.push_str(block.code());

        let _ = token_stream_from_str!(
            &code[prefix_before_insert.len() + insert.len() + after_insert.len()..],
            "Code block"
        );

        code.push_str(ordinary_code_suffix);
        let _ = token_stream_from_str!(
            &code,
            "Extended code block: Prefix, insert, after_insert, the original code and suffix."
        );

        all_code.push_str(&code);
    }

    let ts = token_stream_from_str!(
        &all_code,
        "All code blocks extended, and with start_prefix and final_suffix"
    );
    // @TODO test if the span makes any difference - test with an error code
    quote_spanned! {span=>
        #ts
    }
    .into()
    //ts.into()
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
/*
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
}*/

#[proc_macro]
pub fn nth(_input: ProcTokenStream) -> ProcTokenStream {
    todo!()
}

/// This is like `readme_code_extractor::all_by_file``, except that `all_by_file` is a declarative
/// macro (macro by example, defined by `macro_rules`). However, [create_nth_extractor_macro] can't
/// be declarative. Why? Because it itself defines a new declarative macro which needs to have a
/// capturing variable (parameter). Capturing variables (parameters) start with a dollar character,
/// but if [create_nth_extractor_macro] itself were a declarative macro, it couldn't generate/return
/// a dollar character.
#[proc_macro]
pub fn create_nth_extractor_macro(input: ProcTokenStream) -> ProcTokenStream {
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
pub fn version(input: ProcTokenStream) -> ProcTokenStream {
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
