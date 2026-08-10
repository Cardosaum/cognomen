# cognomen

*Cognomen* is Latin for "an extra name given to a person or thing". It is a
zero-cost procedural macro that gives every unit-like variant of an enum a
second name: a stable, case-configured string **label**.

It sits between an enum's Rust identifier and the strings a config file, log
line, or wire message actually carries. Case conversion happens at compile
time and is emitted as a `&'static str`, so calling `label()` has no runtime
cost.

```rust
use cognomen::Cognomen;

// The first case listed is the default; every case gets a `label_<case>`
// accessor. `label()` / `as_str()` alias the default.
#[derive(Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Mode {
    SingleProcess, // "single_process" / "single-process"
    MultiProcess,  // "multi_process"  / "multi-process"
}

assert_eq!(Mode::SingleProcess.label(), "single_process");
assert_eq!(Mode::MultiProcess.as_str(), "multi_process");
assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
```

## Case styles

| `#[cognomen(...)]`         | `VariantName` becomes |
|---------------------------|-----------------------|
| `snake_case`              | `variant_name`        |
| `kebab-case`              | `variant-name`        |
| `camelCase`               | `variantName`         |
| `PascalCase`              | `VariantName`         |
| `SCREAMING_SNAKE_CASE`    | `VARIANT_NAME`        |
| `lower`                   | `variantname`         |
| `upper`                   | `VARIANTNAME`         |

Underscore spellings (`kebab_case`) are accepted.

List more than one case comma-separated; the **first** listed is the default
returned by `label()` / `as_str()`, and every listed case gets its own
`label_<case>` accessor (from the "Generated API" table below).

## Requirements

- Derive on **enums only**.
- **Unit variants only** (no fields).
- At least one variant.
- A `#[cognomen(<case style>)]` container attribute with one or more
  comma-separated cases (e.g. `#[cognomen(snake_case, kebab-case)]`). The
  **first** case is the default.

Violations are compile-time errors. The failure cases are pinned by
[trybuild](https://docs.rs/trybuild) UI tests under `tests/ui/`.

## Generated API

For an enum `E` with `#[cognomen(snake_case, kebab-case)]`, the derive adds a
`const fn` per declared case, plus two aliases for the default (first):

- `E::variant.label() -> &'static str`: the default (first) case.
- `E::variant.as_str() -> &'static str`: an ergonomic alias for `label`.
- `E::variant.label_snake() -> &'static str`: the `snake_case` label.
- `E::variant.label_kebab() -> &'static str`: the `kebab-case` label.

Accessor for each case:

| case                         | accessor                  |
|------------------------------|---------------------------|
| `snake_case`                 | `label_snake`             |
| `kebab-case`                 | `label_kebab`             |
| `camelCase`                  | `label_camel`             |
| `PascalCase`                 | `label_pascal`            |
| `SCREAMING_SNAKE_CASE`       | `label_screaming_snake`   |
| `lower`                      | `label_lower`             |
| `upper`                      | `label_upper`             |

For the reverse direction, the derive implements two fallible conversions
that accept a string in any declared case and return the matching variant:

- `TryFrom<&str> for E`, giving `E::try_from("single_process")`.
- `FromStr for E`, giving `"single_process".parse::<E>()`.

Both return a `FromLabelError` when the string matches no variant; the error
implements `Display` and `std::error::Error`, and reports the offending input.

```rust
assert_eq!("single-process".parse::<Mode>(), Ok(Mode::MultiProcess));
assert_eq!(Mode::try_from("multi_process"), Ok(Mode::MultiProcess));
```

## MSRV

Rust **1.71.1**, determined with `cargo-msrv` (bisect, default deps). The
floor is set by the pinned `proc-macro2` dependency (needs rustc 1.71 or
newer); cognomen's own code only requires `Option::is_some_and` (Rust 1.70).

## License

Dual-licensed under **MIT OR Apache-2.0**, see
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
Copyright (c) 2026 Matheus Cardoso.