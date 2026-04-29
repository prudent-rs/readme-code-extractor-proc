#![doc = include_str!("../README.md")]

use core::str::FromStr;
use proc_macro::TokenStream as ProcTokenStream;
use proc_macro_rules::rules;
use proc_macro2::TokenStream;
use proc_macro2_diagnostics::Diagnostic;
use proc_macro2_diagnostics::SpanDiagnosticExt as _;
use quote::{quote, quote_spanned};
use readme_code_extractor_lib::public::{
    CodeBlock, Config, ConfigAndSpan, ConfigContentAndSpan, ReadmeBlock, ReadmeExtracted,
};
use syn::spanned::Spanned as _;

type MacroResult<T> = Result<T, Diagnostic>;

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
// ----
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

            let cfg_content_and_span = readme_code_extractor_lib::public::config_content_and_span(
                &config_toml_content);
            // @TODO use
            /*let _preamble_txt= if let Some(preamble_text) = readme_extracted.preamble_text() {};
            let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {};
            ...
            q.extend( q2);*/
            all_by_config_content_and_span(cfg_content_and_span)
        }
    })
}

/// Invoked by `readme_code_extractor::all_by_file`.
#[proc_macro]
pub fn all_by_file(input: ProcTokenStream) -> ProcTokenStream {
    rules!(input.into() => {
        ( $config_toml_content:literal ) => {

            let (cfg_content_and_span, toml_config_file_path) = readme_code_extractor_lib::public::config_content_and_span_by_file(
                &config_toml_content);

            let toml_config_file_path = toml_config_file_path.as_ref();
            let prefix_stream = format!("const _: &str = ::std::include_str!(\"{toml_config_file_path}\");\n");

            let prefix_stream = TokenStream::from_str(&prefix_stream).expect("TODO");

            selected_by_config_content_and_span(prefix_stream, cfg_content_and_span, |_, _, _| true)
        }
    })
}

#[proc_macro]
pub fn nth(input: ProcTokenStream) -> ProcTokenStream {
    rules!(input.into() => {
        ( $config_toml_content:literal @ $index:literal) => {

            let cfg_content_and_span = readme_code_extractor_lib::public::config_content_and_span(
                &config_toml_content);
            let code_block_index = match index.to_string().parse::<usize>() {
                Ok(value) => value,
                Err(err) => {
                    panic!("Expecting a non-negative (usize) index literal, but received: {err:?}")
                }
            };
            // @TODO use
            /*let _preamble_txt= if let Some(preamble_text) = readme_extracted.preamble_text() {};
            let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {};
            ...
            q.extend( q2);*/
            nth_by_config_content_and_span(cfg_content_and_span, code_block_index)
        }
    })
}
// ----

fn all_by_config_content_and_span(
    cfg_content_and_span: impl ConfigContentAndSpan,
) -> ProcTokenStream {
    selected_by_config_content_and_span(TokenStream::new(), cfg_content_and_span, |_, _, _| true)
}

fn nth_by_config_content_and_span(
    cfg_content_and_span: impl ConfigContentAndSpan,
    code_block_index: usize,
) -> ProcTokenStream {
    selected_by_config_content_and_span(TokenStream::new(), cfg_content_and_span, |idx, _, _| {
        idx == code_block_index
    })
}
// ----

fn selected_by_config_content_and_span<F: Fn(usize, &dyn CodeBlock, &str) -> bool>(
    prefix_stream: TokenStream,
    cfg_content_and_span: impl ConfigContentAndSpan,
    code_block_filter: F,
) -> ProcTokenStream {
    let config_and_span = readme_code_extractor_lib::public::config_and_span(&cfg_content_and_span);
    let readme_loaded = readme_code_extractor_lib::public::readme_load(&config_and_span);
    let readme_extracted = readme_code_extractor_lib::public::readme_extract(&readme_loaded);

    let config = config_and_span.config();

    impl_filtered(prefix_stream, config, readme_extracted, code_block_filter).into()
}

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

/// Param code_block_filter is a closure that takes:
/// - usize 0-based index of the code block being handled
/// - &dyn [CodeBlock]
/// - &str current code block's respective insert from
///   [readme_code_extractor_lib::public::config::headers::Inserts::inserts] (or an empty string
///   slice if there are no inserts).
/// and returns `bool` whether to include the code block or not.
fn impl_filtered<'a, F: Fn(usize, &dyn CodeBlock, &str) -> bool>(
    mut prefix_stream: TokenStream,
    config: &dyn Config,
    mut readme_extracted: impl ReadmeExtracted<'a>,
    code_block_filter: F,
) -> TokenStream {
    let (has_inserts, inserts, inserts_iter_or_cycle, after_insert): (
        bool,
        &[&str],
        &mut dyn Iterator<Item = &&str>,
        &str,
    ) = if let Some(headers) = config.ordinary_code_headers()
        && let Some(inserts) = headers.inserts()
    {
        (
            true,
            inserts.inserts(),
            &mut inserts.inserts().iter(),
            inserts.after_insert(),
        )
    } else {
        (false, &[], &mut [""].iter().cycle(), "")
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
    //panic!("code_blocks: {}", code_blocks.len());
    //
    //panic!("code_blocks[0]: {}", code_blocks[0].code());

    assert!(
        !has_inserts || code_blocks.len() == inserts.len(),
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

    let markdown_file_local_path = readme_extracted.markdown_file_local_path();
    let code_to_load_markdown_file =
        format!("const _: &str = ::std::include_str!(\"{markdown_file_local_path}\");\n");

    // We don't count the length of all inserts. Using the maximum is good enough.
    let mut all_code = String::with_capacity(
        code_to_load_markdown_file.len()
            + config.start_prefix().len()
            + total_code_blocks_len
            + max_code_block_len * code_blocks.len()
            + config.final_suffix().len(),
    );

    all_code.push_str(&code_to_load_markdown_file);
    all_code.push_str(config.start_prefix());

    for (code_block_idx, (&block, &insert)) in
        code_blocks.iter().zip(inserts_iter_or_cycle).enumerate()
    {
        if !code_block_filter(code_block_idx, block, insert) {
            continue;
        }
        code.clear();
        // @TODO triple_backtick_suffix
        code.push_str(prefix_before_insert);
        // @TODO insert
        code.push_str(insert);
        code.push_str(after_insert);

        let block_code = block.code();
        // Verify that the pushed part is a well-formed Rust token stream, that is, all parens (..),
        // brackets [..] and braces {..} are "balanced", string and char literals are well enclosed
        // etc.
        let _ = token_stream_from_str!(block_code, "Code block");
        code.push_str(block_code);

        // Verify a well-formed Rust token stream.
        code.push_str(ordinary_code_suffix);
        let _ = token_stream_from_str!(
            &code,
            "Extended code block: Prefix, insert, after_insert, the original code and suffix."
        );

        all_code.push_str(&code);
    }
    all_code.push_str(config.final_suffix());

    // Verify a well-formed Rust token stream.
    let main_token_stream = token_stream_from_str!(
        &all_code,
        "All code blocks extended, and with start_prefix and final_suffix"
    );
    prefix_stream.extend(core::iter::once(main_token_stream));
    prefix_stream
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
