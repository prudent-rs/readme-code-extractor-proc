#![doc = include_str!("../README.md")]

use core::str::FromStr;
use proc_macro::TokenStream as ProcTokenStream;
use proc_macro_rules::rules;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use readme_code_extractor_lib::public::ext::*;
use readme_code_extractor_lib::public::{
    CodeBlock, Config, ConfigAndSpan, ConfigContentAndSpan, MacroDeepResult, MacroResult,
    OwnedStringSlice, ReadmeBlock, ReadmeExtracted, assert,
};

type MacroStreamResult = MacroResult<TokenStream>;
type MacroStreamDeepResult = MacroDeepResult<TokenStream>;

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
// ----

/// Process (adjust and pass through) all code blocks in the given input.
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
///
/// NOT filtering by "tag:" value in the triple backtick suffix (if any).
///
/// Whether the "tag:" value is passed through or not is controlled by TOML configuration.
#[proc_macro]
pub fn all(input: ProcTokenStream) -> ProcTokenStream {
    match all_impl(input.into()) {
        Ok(input) => input.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

fn all_impl(input: TokenStream) -> MacroStreamResult {
    rules!(input => {
        ( $config_toml_content:literal ) => {

            let cfg_content_and_span = readme_code_extractor_lib::public::config_content_and_span(
                &config_toml_content)?;
            // @TODO use
            /*let _preamble_txt= if let Some(preamble_text) = readme_extracted.preamble_text() {};
            let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {};
            ...
            q.extend( q2);*/
            all_by_config_content_and_span(TokenStream::new(), &cfg_content_and_span)
        }
    })
}
// ----

/// Process (adjust and pass through) all code blocks. Configuration is in a (TOML) file, its file
/// path is in the input.
///
/// See also [all].
#[proc_macro]
pub fn all_by_file(input: ProcTokenStream) -> ProcTokenStream {
    match all_by_file_impl(input.into()) {
        Ok(input) => input.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

/// Generate code that loads content of the given file to a string slice constant. That ensures the
/// user's code is re-built if the file gets modified.
fn load_file_to_const(span: Span, toml_config_file_path: &OwnedStringSlice) -> MacroStreamResult {
    let toml_config_file_path = toml_config_file_path.as_ref();
    let prefix_stream =
        format!("const _: &str = ::std::include_str!(\"{toml_config_file_path}\");\n");

    TokenStream::from_str(&prefix_stream).map_error_dbg_with_for(
        || {
            format!(
                "the TOML config file path is not well formed, or it failed to parse: {}",
                prefix_stream
            )
        },
        span,
    )
}

fn all_by_file_impl(input: TokenStream) -> MacroStreamResult {
    rules!(input => {
        ( $config_toml_file_path:literal ) => {

            let (cfg_content_and_span, toml_config_file_path) = readme_code_extractor_lib::public::config_content_and_span_by_file(
                &config_toml_file_path)?;

            let prefix_stream = load_file_to_const(
                cfg_content_and_span.span(), &toml_config_file_path)?;

            all_by_config_content_and_span(prefix_stream, &cfg_content_and_span)
        }
    })
}
// ----

/// Process (adjust and pass through) only n-th code block from the given input.
///
/// Configuration is in the first input. 0-based index is in the second input.
#[proc_macro]
pub fn nth(input: ProcTokenStream) -> ProcTokenStream {
    match nth_impl(input.into()) {
        Ok(input) => input.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

fn code_block_index(index: &Literal) -> MacroResult<usize> {
    let index_string = index.to_string();
    index_string.parse::<usize>().map_error_dbg_with_for(
        || {
            format!(
                "Expecting a non-negative (usize) index literal, but received: {}",
                index_string
            )
        },
        index.span(),
    )
}

fn nth_impl(input: TokenStream) -> MacroStreamResult {
    rules!(input => {
        ( $config_toml_content:literal @ $index:literal) => {

            let cfg_content_and_span = readme_code_extractor_lib::public::config_content_and_span(
                &config_toml_content)?;
            let code_block_index = code_block_index(&index)?;
            // @TODO use
            /*let _preamble_txt= if let Some(preamble_text) = readme_extracted.preamble_text() {};
            let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {};
            ...
            q.extend( q2);*/
            nth_by_config_content_and_span(TokenStream::new(), &cfg_content_and_span, code_block_index)
        }
    })
}
// ----

/// Process (adjust and pass through) only n-th code block from the given input.
///
/// Configuration is in a (TOML) file, its file path is in the first input. 0-based index is in the
/// second input.
#[proc_macro]
pub fn nth_by_file(input: ProcTokenStream) -> ProcTokenStream {
    match nth_by_file_impl(input.into()) {
        Ok(input) => input.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

fn nth_by_file_impl(input: TokenStream) -> MacroStreamResult {
    rules!(input => {
        ( $config_toml_file_path:literal @ $index:literal) => {

            let (cfg_content_and_span, toml_config_file_path) = readme_code_extractor_lib::public::config_content_and_span_by_file(
                &config_toml_file_path)?;

            let prefix_stream = load_file_to_const(
                cfg_content_and_span.span(), &toml_config_file_path)?;

            let code_block_index = code_block_index(&index)?;
            nth_by_config_content_and_span(prefix_stream, &cfg_content_and_span, code_block_index)
        }
    })
}
// ----
/// Process (adjust and pass through) only code block(s) with a given tag.
/// - if `one` then expecting exactly ONE matching code block.
/// - if `any` then any number of matching code blocks (including zero) is fine.
///
/// Configuration is in the first input. Tag is in the second input.
#[proc_macro]
pub fn tag(input: ProcTokenStream) -> ProcTokenStream {
    match tag_impl(input.into()) {
        Ok(input) => input.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

fn tag_impl(input: TokenStream) -> MacroStreamResult {
    rules!(input => {
        ( $config_toml_content:literal one @ $tag:literal) => {
            tag_impl_shared(config_toml_content, tag, true)
        }
        ( $config_toml_content:literal any @ $tag:literal) => {
            tag_impl_shared(config_toml_content, tag, false)
        }
    })
}
fn tag_impl_shared(
    config_toml_content: Literal,
    tag: Literal,
    tag_exactly_one_match: bool,
) -> MacroStreamResult {
    let cfg_content_and_span =
        readme_code_extractor_lib::public::config_content_and_span(&config_toml_content)?;

    let tag = readme_code_extractor_lib::public::string_literal_content(&tag)?;
    tag_by_config_content_and_span(
        TokenStream::new(),
        &cfg_content_and_span,
        tag.as_ref(),
        tag_exactly_one_match,
    )
}
// ----

fn all_by_config_content_and_span(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
) -> MacroStreamResult {
    selected_by_config_content_and_span(prefix_stream, cfg_content_and_span, |_, _, _| Ok(true))
}

fn nth_by_config_content_and_span(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
    code_block_index: usize,
) -> MacroStreamResult {
    selected_by_config_content_and_span(
        prefix_stream,
        cfg_content_and_span,
        |code_blocks, idx, _| {
            assert::true_or_error(idx < code_blocks.len(), || {
                format!(
                    "The index {idx} is non-negative (usize), but it's outside of {} code blocks.",
                    code_blocks.len()
                )
            })?;
            Ok(idx == code_block_index)
        },
    )
}

fn tag_by_config_content_and_span(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
    tag: &str,
    tag_exactly_one_match: bool,
) -> MacroStreamResult {
    let mut already_found = false;

    let result = selected_by_config_content_and_span(
        prefix_stream,
        cfg_content_and_span,
        |_, _, code_block| {
            assert::true_or_error(!tag_exactly_one_match || !already_found, || {
                format!("already found one code block with the same tag: {tag}")
            })?;
            let found = code_block.tag() == Some(tag);
            already_found |= found;
            Ok(found)
        },
    );
    assert::true_or_error(tag_exactly_one_match && !already_found, || {
        format!("did not find a code block with the given tag: {tag}")
    })
    .spanned(cfg_content_and_span.span())?;
    result
}
// ----

fn selected_by_config_content_and_span<F>(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
    code_block_filter: F,
) -> MacroStreamResult
where
    F: FnMut(&Vec<&dyn CodeBlock>, usize, &dyn CodeBlock) -> MacroDeepResult<bool>,
{
    let config_and_span = readme_code_extractor_lib::public::config_and_span(cfg_content_and_span)?;
    let readme_loaded = readme_code_extractor_lib::public::readme_load(&config_and_span)?;
    let span = config_and_span.span();
    let readme_extracted =
        readme_code_extractor_lib::public::readme_extract(&readme_loaded).spanned(span)?;

    let (config, span) = (config_and_span.config(), config_and_span.span());

    impl_filtered(prefix_stream, config, readme_extracted, code_block_filter).spanned(span)
}

macro_rules! token_stream_from_str {
    ($input_string:expr, $err_intended_result_description:expr) => {
        ({
            let input_string = $input_string;
            TokenStream::from_str(input_string).map_error_dbg_with(|| {
                format!(
                    "readme-code-extractor-proc failed to parse: {}\nunpaired or incorrect Rust \
                tokens in: {}",
                    $err_intended_result_description, input_string
                )
            })?
        })
    };
}

/// Param code_block_filter is a closure that takes:
/// - usize 0-based index of the code block being handled
/// - &dyn [CodeBlock]
/// - &str current code block's respective tag from
///   [readme_code_extractor_lib::public::config::headers::Tags::tags] (or an empty string
///   slice if there are no tags).
/// and returns `bool` whether to include the code block or not.
fn impl_filtered<'a, F>(
    mut prefix_stream: TokenStream,
    config: &dyn Config,
    mut readme_extracted: impl ReadmeExtracted<'a>,
    mut code_block_filter: F,
) -> MacroStreamDeepResult
where
    F: FnMut(&Vec<&dyn CodeBlock>, usize, &dyn CodeBlock) -> MacroDeepResult<bool>,
{
    let headers = config.code_headers();

    let blocks = readme_extracted
        .non_preamble_blocks()
        .collect::<MacroDeepResult<Vec<_>>>()?;

    // @TODO apply backtick suffixes like "ignore" or "norun"
    let mut code_blocks = Vec::with_capacity(blocks.len() / 2 + 1);
    code_blocks.extend(blocks.iter().filter_map(ReadmeBlock::code));

    let markdown_file_path = readme_extracted.markdown_file_path();
    let code_to_load_markdown_file =
        format!("const _: &str = ::std::include_str!(\"{markdown_file_path}\");\n");

    let (mut generated_per_block, mut generated_all) = {
        let config_generated_len_per_block =
            headers.top_prefix().len() + headers.tag_suffix().len() + headers.end_suffix().len();

        let max_code_block_and_tag_len = code_blocks
            .iter()
            .map(|b| b.code().len() + b.tag().map_or(0, |tag| tag.len()))
            .max()
            .unwrap_or(0);

        let generated_per_block =
            String::with_capacity(config_generated_len_per_block + max_code_block_and_tag_len);

        // @TODO preamble etc.
        let total_code_blocks_len = code_blocks.iter().map(|b| b.code().len()).sum::<usize>();

        // We don't count the length of all tags. Using the maximum is good enough.
        let generated_all = String::with_capacity(
            code_to_load_markdown_file.len()
                + config.start_prefix().len()
                + total_code_blocks_len
                + max_code_block_and_tag_len * code_blocks.len()
                + config.final_suffix().len(),
        );

        (generated_per_block, generated_all)
    };

    generated_all.push_str(&code_to_load_markdown_file);
    generated_all.push_str(config.start_prefix());

    for (code_block_idx, &block) in code_blocks.iter().enumerate() {
        if !code_block_filter(&code_blocks, code_block_idx, block)? {
            continue;
        }
        generated_per_block.clear();
        // @TODO triple_backtick_suffix
        generated_per_block.push_str(headers.top_prefix());
        if config.pass_through_tags() {
            generated_per_block.push_str(block.tag().unwrap_or(""));
        }
        generated_per_block.push_str(headers.tag_suffix());

        let block_code = block.code();
        // Verify that the pushed part is a well-formed Rust token stream, that is, all parens (..),
        // brackets [..] and braces {..} are "balanced", string and char literals are well enclosed
        // etc.
        let _ = token_stream_from_str!(block_code, "Code block");
        generated_per_block.push_str(block_code);

        // Verify that the total output is a well-formed Rust token stream.
        generated_per_block.push_str(headers.end_suffix());
        let _ = token_stream_from_str!(
            &generated_per_block,
            "Extended code block: Prefix, tag, after_tag, the original code and suffix."
        );

        generated_all.push_str(&generated_per_block);
    }
    generated_all.push_str(config.final_suffix());

    // Verify a well-formed Rust token stream.
    let main_token_stream = token_stream_from_str!(
        &generated_all,
        "Whole output: start_prefix, all code blocks extended, and final_suffix."
    );
    prefix_stream.extend(core::iter::once(main_token_stream));
    Ok(prefix_stream)
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
