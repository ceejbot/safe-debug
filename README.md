# SafeDebug

A Rust derive macro for `std::fmt::Debug` that automatically redacts sensitive fields marked with `#[facet(sensitive)]`. Derive `Facet` and `SafeDebug`, then decorate fields with sensitive data with `#[facet(sensitive)]`. You'll get:

- a full `std::fmt::Debug` implementation-- respects `{:?}` and `{:#?}`
- automatic redaction of fields you've marked as sensitive
- fallback to full redaction on any failure to find valid structure metadata
- all the benefits of deriving [facet](https://facet.rs) and existing in that ecosystem

The benefit of opting into `facet` here is, for example, the ability to do other things based on the presence of the shape metadata and the sensitivity flag, which you might want when implementing things in a context where this matters

[![Tests](https://github.com/ceejbot/safe-debug/actions/workflows/test.yaml/badge.svg)](https://github.com/ceejbot/safe-debug/actions/workflows/test.yaml) [![Coverage](https://img.shields.io/badge/coverage-91%25-yellow)](https://github.com/ceejbot/safe-debug/actions)

## Example

```rust
use facet::Facet;
use safe_debug::SafeDebug;

#[derive(Facet, SafeDebug)]
struct PatientRecord {
    id: String,
    name: String,
    #[facet(sensitive)]
    ssn: String,
    #[facet(sensitive)]
    medical_history: String,
}

fn main() {
    let record = PatientRecord {
        id: "12345".to_string(),
        name: "John Doe".to_string(),
        ssn: "123-45-6789".to_string(),
        medical_history: "Confidential medical data".to_string(),
    };

    // While logging, somebody drops in data that includes fields with PHI in them...
    println!("useful but inattentive log line; record='{:?}'", record);
    // but we are saved because the output is:
    // useful but inattentive log line; record=PatientRecord { id: "12345", name: "John Doe", ssn: "[REDACTED]", medical_history: "[REDACTED]" }

    // Pretty-print also works
    println!("{:#?}", record);
}
```

See the [./examples directory](./examples) for more usage examples.

## Installation

Add `safe-debug` *and* `facet` to your crate's dependencies:

```bash
cargo add safe-debug
cargo add facet
```

Or add manually to your `Cargo.toml`:

```toml
[dependencies]
safe-debug = "0.1"
facet = "0.31"
```

## Usage

For any struct or enum where you want automatic sensitive field redaction:

1. Derive both `Facet` and `SafeDebug`
2. Mark sensitive fields with `#[facet(sensitive)]`
3. Use `{:?}` or `{:#?}` formatting as normal

The `Debug` implementation will automatically redact fields marked as sensitive.

## Troubleshooting

### Error: "the trait bound `YourType: Facet` is not satisfied"

Remember to derive `Facet` as well as `SafeDebug`:

```rust,ignore
#[derive(Facet, SafeDebug)]
struct YourType { /* ... */ }
```

### Error: "the trait bound `T: std::fmt::Debug` is not satisfied"

Make sure all your type parameters implement `Debug`:

```rust,ignore
#[derive(Facet, SafeDebug)]
struct Container<T: Debug> {  // add a bound here to enforce it
    data: T,
}
```

### Sensitive data appearing in logs

Make sure you're actually using the `Debug` trait (via `{:?}` or `{:#?}`), not `Display` or other formatting traits, via `{}`. This crate only derives `Debug`, leaving `Display` for you to control if you need it to print sensitive data for legitmate reasons.

## License

This crate uses the same licensing approach as `facet` itself.

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
