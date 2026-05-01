#![doc = include_str!("../README.md")]

use core::str::FromStr;
use proc_macro::TokenStream as ProcTokenStream;
use proc_macro_rules::rules;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use readme_code_extractor_lib::public::{
    CodeBlock, Config, ConfigAndSpan, ConfigContentAndSpan, MacroDeepResult, MacroResult,
    MacroResultDeepExt, OwnedStringSlice, ReadmeBlock, ReadmeExtracted,
};
use readme_code_extractor_lib::{ok_or_fail, ok_or_fail_deep, true_or_fail_deep};

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

    Ok(ok_or_fail!(
        span,
        TokenStream::from_str(&prefix_stream),
        "The given TOML config file path is not well formed, or somehow the following \
            couldn't be parsed:\n{}\nError:\n{:?}",
        prefix_stream
    ))
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
    Ok(ok_or_fail!(
        index.span(),
        index_string.parse::<usize>(),
        "Expecting a non-negative (usize) index literal, but received: {}. Error: {:?}",
        index_string
    ))
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
            // @TODO use
            /*let _preamble_txt= if let Some(preamble_text) = readme_extracted.preamble_text() {};
            let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {};
            ...
            q.extend( q2);*/
            nth_by_config_content_and_span(prefix_stream, &cfg_content_and_span, code_block_index)
        }
    })
}
// ----
/*
/// Process (adjust and pass through) only code block with a given tag.
///
/// Configuration is in the first input. Tag is in the second input.
#[proc_macro]
pub fn tag(input: ProcTokenStream) -> ProcTokenStream {
    match nth_impl(input.into()) {
        Ok(input) => input.into(),
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}

fn tag_impl(input: TokenStream) -> MacroStreamResult {
    rules!(input => {
        ( $config_toml_content:literal @ $tag:literal) => {

            let cfg_content_and_span = readme_code_extractor_lib::public::config_content_and_span(
                &config_toml_content)?;

            let tag = readme_code_extractor_lib::public::string_literal_content(&tag);
            // @TODO use
            /*let _preamble_txt= if let Some(preamble_text) = readme_extracted.preamble_text() {};
            let preamble_code = if let Some(preamble_code) = readme_extracted.preamble_code() {};
            ...
            q.extend( q2);*/
            tag_by_config_content_and_span(TokenStream::new(), &cfg_content_and_span, &tag)
        }
    })
}*/
// ----

fn all_by_config_content_and_span(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
) -> MacroStreamResult {
    selected_by_config_content_and_span(prefix_stream, cfg_content_and_span, |_, _, _, _| Ok(true))
}

fn nth_by_config_content_and_span(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
    code_block_index: usize,
) -> MacroStreamResult {
    let span = cfg_content_and_span.span();
    selected_by_config_content_and_span(
        prefix_stream,
        cfg_content_and_span,
        |code_blocks, idx, _, _| {
            let _ = span;
            true_or_fail_deep!(
                idx < code_blocks.len(),
                "The received index {idx} is non-negative (usize), but it's outside of {} code blocks.",
                code_blocks.len()
            );
            Ok(idx == code_block_index)
        },
    )
}

/*fn tag_by_config_content_and_span(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
    tag: &OwnedStringSlice
) -> MacroStreamResult {
    let span = cfg_content_and_span.span();
    selected_by_config_content_and_span(
        prefix_stream,
        cfg_content_and_span,
        |code_blocks, idx, _, _| {
            let _ = span;
            true_or_fail!(
                span,
                idx < code_blocks.len(),
                "The received index {idx} is non-negative (usize), but it's outside of {} code blocks.",
                code_blocks.len()
            );
            Ok(idx == code_block_index)
        },
    )
}*/
// ----

fn selected_by_config_content_and_span<F>(
    prefix_stream: TokenStream,
    cfg_content_and_span: &impl ConfigContentAndSpan,
    code_block_filter: F,
) -> MacroStreamResult
where
    F: Fn(&Vec<&dyn CodeBlock>, usize, &dyn CodeBlock, &str) -> MacroDeepResult<bool>,
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
            ok_or_fail_deep!(
                TokenStream::from_str(input_string),
                "readme-code-extractor-proc: Parsing {} failed. Unpaired or incorrect Rust \
                 tokens. Input:\n{}\nError: {:?}",
                $err_intended_result_description,
                input_string
            )
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
    code_block_filter: F,
) -> MacroStreamDeepResult
where
    F: Fn(&Vec<&dyn CodeBlock>, usize, &dyn CodeBlock, &str) -> MacroDeepResult<bool>,
{
    let (has_tags, tags, tags_iter_or_cycle, after_tag): (
        bool,
        &[&str],
        &mut dyn Iterator<Item = &&str>,
        &str,
    ) = if let Some(headers) = config.ordinary_code_headers()
        && let Some(tags) = headers.tags()
    {
        (true, tags.tags(), &mut tags.tags().iter(), tags.after_tag())
    } else {
        (false, &[], &mut [""].iter().cycle(), "")
    };

    let (prefix_before_tag, max_tag_len) = if let Some(headers) = config.ordinary_code_headers() {
        (
            headers.prefix_before_tag(),
            tags.iter().map(|&s| s.len()).max().unwrap_or(0),
        )
    } else {
        ("", 0usize)
    };
    let ordinary_code_suffix = config.ordinary_code_suffix();

    let blocks = readme_extracted
        .non_preamble_blocks()
        .collect::<MacroDeepResult<Vec<_>>>()?;

    // @TODO apply backtick suffixes like "ignore" or "norun"
    let mut code_blocks = Vec::with_capacity(blocks.len() / 2 + 1);
    code_blocks.extend(blocks.iter().filter_map(ReadmeBlock::code));

    true_or_fail_deep!(
        !has_tags || code_blocks.len() == tags.len(),
        "Expecting number of blocks {} and number of tags {} to be the same!",
        code_blocks.len(),
        tags.len()
    );

    let max_code_block_len = code_blocks
        .iter()
        .map(|b| b.code().len())
        .max()
        .unwrap_or(0);

    let mut generated_per_block = {
        let config_generated_len_per_block =
            prefix_before_tag.len() + max_tag_len + after_tag.len() + ordinary_code_suffix.len();
        // @TODO triple_backtick_suffix- BUT only if we pass it through

        String::with_capacity(config_generated_len_per_block + max_code_block_len)
    };

    // @TODO preamble etc.
    let total_code_blocks_len = code_blocks.iter().map(|b| b.code().len()).sum::<usize>();

    let markdown_file_path = readme_extracted.markdown_file_path();
    let code_to_load_markdown_file =
        format!("const _: &str = ::std::include_str!(\"{markdown_file_path}\");\n");

    // We don't count the length of all tags. Using the maximum is good enough.
    let mut generated_all = String::with_capacity(
        code_to_load_markdown_file.len()
            + config.start_prefix().len()
            + total_code_blocks_len
            + max_code_block_len * code_blocks.len()
            + config.final_suffix().len(),
    );

    generated_all.push_str(&code_to_load_markdown_file);
    generated_all.push_str(config.start_prefix());

    for (code_block_idx, (&block, &tag)) in code_blocks.iter().zip(tags_iter_or_cycle).enumerate() {
        if !code_block_filter(&code_blocks, code_block_idx, block, tag)? {
            continue;
        }
        generated_per_block.clear();
        // @TODO triple_backtick_suffix
        generated_per_block.push_str(prefix_before_tag);
        generated_per_block.push_str(tag);
        generated_per_block.push_str(after_tag);

        let block_code = block.code();
        // Verify that the pushed part is a well-formed Rust token stream, that is, all parens (..),
        // brackets [..] and braces {..} are "balanced", string and char literals are well enclosed
        // etc.
        let _ = token_stream_from_str!(block_code, "Code block");
        generated_per_block.push_str(block_code);

        // Verify that the total output is a well-formed Rust token stream.
        generated_per_block.push_str(ordinary_code_suffix);
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
