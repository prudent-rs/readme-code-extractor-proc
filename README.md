# readme-code-extractor


## Other details

### TOML only

See [`prudent/readme-code-extractor-core` ->
`README.md`](https://github.com/prudent-rs/readme-code-extractor-core/blob/main/README.md) for why
we use TOML only.

### Inline (TOML) config only

@TODO Our proc macro could also generate

if false {core::hint::black_box(core::include_str!("relative/cfg_file_path_here.toml"));}

@TODO - see if a change of such a file would re-build the package, even if the Rust source is not
changed at all.

We do NOT, and will NOT, support loading of the configuration (TOML) from a separate file. Why?

- It's against the intent and use of Rust macros. If a macro's input doesn't change, then the output
  should not either. In this case, the actual input to the macro would NOT be the config (TOML), but
  a (relative) file path to a file. Content of the file could change while the path stays the same.
  Then the macro invocation doesn't change, and the input file size/timestamp doesn't change either,
  so the compiler can assume no change. That would be incorrect.
- We could experiment with `build.rs` and it invalidating the build, but it's not intended for this.

### nth_*** extraction does repeat parsing

Every invocation of nth_*** extraction does parse the whole file (`README.md` or as configured). The
cost is irrelevant. We do NOT cache/store any data between the macro invocations. Rust macros must
not keep/depend on any state between their invocations.
