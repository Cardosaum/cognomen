# cognomen

*Cognomen* — Latin for *"an extra name given to a person or thing"* — is a
zero-cost procedural macro that gives every unit-like variant of an enum a
second name: a stable, case-configured string **label**.

It sits between an enum's Rust identifier and the strings a config file, log
line, or wire message actually carries. Case conversion happens at compile
time and is emitted as a `&'static str`, so calling `label()` has no runtime
cost.

```rust
use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    SingleProcess, // "single_process"
    MultiProcess,  // "multi_process"
}

assert_eq!(Mode::SingleProcess.as_str(), "single_process");
assert_eq!(Mode::MultiProcess.label(), "multi_process");
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

Underscore spellings (`kebab_case`) are accepted, as is the explicit
`#[cognomen(case = snake_case)]` form.

## Requirements

- Derive on **enums only**.
- **Unit variants only** (no fields).
- At least one variant.
- A `#[cognomen(<case style>)]` container attribute.

Violations are compile-time errors. The failure cases are pinned by
[trybuild](https://docs.rs/trybuild) UI tests under `tests/ui/`.

## Generated API

For an enum `E`, the derive adds two `const fn`s:

- `E::variant.label() -> &'static str` — the primary accessor.
- `E::variant.as_str() -> &'static str` — an ergonomic alias for config/log
  call sites.

## MSRV

Rust **1.71.1** — determined with `cargo-msrv` (bisect, default deps). The
floor is set by the pinned `proc-macro2` dependency (needs rustc ≥ 1.71);
cognomen's own code only requires `Option::is_some_and` (Rust 1.70).

## License

Dual-licensed under **MIT OR Apache-2.0** — see
[`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
Copyright (c) 2026 Cardosaum.