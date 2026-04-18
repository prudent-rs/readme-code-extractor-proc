# readme-code-extractor-proc

## Other details

### TOML only

See [`prudent/readme-code-extractor-core` ->
`README.md`](https://github.com/prudent-rs/readme-code-extractor-core/blob/main/README.md) for why
we use TOML only.

### nth_*** extraction does repeat parsing

Every invocation of nth_*** extraction does parse the whole file (`README.md` or as configured). The
cost is irrelevant. We do NOT cache/store any data between the macro invocations. Rust macros must
not keep/depend on any state between their invocations.
