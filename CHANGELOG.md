# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-20

### Added

- Initial release of `safe-debug`
- `SafeDebug` derive macro for automatic sensitive field redaction
- Support for named structs, tuple structs, and unit structs
- Comprehensive enum support:
  - Unit variants (`Variant`)
  - Tuple variants (`Variant(T, U)`) with per-field sensitivity
  - Struct variants (`Variant { field: T }`) with per-field sensitivity
  - Mixed variant types in same enum
- Full generic support:
  - Type parameters with auto-generated trait bounds
  - Lifetime parameters
  - Higher-ranked trait bounds (HRTB) for facet integration
  - Multi-generic types (`Pair<T, U>`, `Triple<T, U, V>`)
- Pretty-print formatting support (`{:#?}`)
- Security-first fallback behavior:
  - Structs: Redact all fields if metadata unavailable
  - Enums: Show only type name if metadata unavailable
- Zero unsafe code
- Minimal runtime overhead (single Shape access + per-field sensitivity check)
- MIT OR Apache-2.0 dual license

### Security

- Automatic `[REDACTED]` output for fields marked `#[facet(sensitive)]`
- Conservative fallback: fails safe by hiding data when metadata is unavailable
- No data leakage in debug output for HIPAA-compliant and other sensitive applications

[Unreleased]: https://github.com/ceejbot/safe-debug/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ceejbot/safe-debug/releases/tag/v0.1.0
