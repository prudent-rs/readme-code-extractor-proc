# readme-code-extractor

## TOML only

See [`prudent/readme-code-extractor-core` ->
`README.md`](https://github.com/prudent-rs/readme-code-extractor-core/blob/main/README.md) for why
we use TOML only.

## Inline (TOML) config only

We do NOT, and will NOT, support loading of the configuration (TOML) from a separate file. Why?

- It's against Rust macros. If a macro's input doesn't change, then the output should not either. In
  this case, the actual input to the macro would NOT be the config (TOML), but a (relative) file
  path to a file. Content of the file could change while the path stays the same. Then the macro
  invocation doesn't change, and the input file size/timestamp doesn't change either, so the
  compiler can assume no change - wrong.
- We could experiment with `build.rs` and it invalidating the build, but it's not intended for this.
