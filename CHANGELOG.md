# Changelog

## 0.6.0

- Published as a Rust crate, `rhai-data-quality`. Each module is a Rhai package (`ContactsPackage`, `DatesPackage`, `NumbersPackage` and `TextPackage`), and `DataQualityPackage` has all four. Register one with `register_into_engine`, or with `register_into_engine_as` to use a namespace.
- The packages work in debug builds without raising Rhai's expression depth limits.
- Requires Rhai 1.25 or later.
- The test cases now run with `cargo test`, which replaces the separate test runner.

## 0.5.0

- `kebab_case`, `extract_hashtags_mentions` and `normalize_quotes` in text.

## 0.4.0 and earlier

See the [releases](https://github.com/rhaiscript/rhai-data-quality/releases).
