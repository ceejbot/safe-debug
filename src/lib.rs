#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]

//! # SafeDebug - Debug with Field Redaction
//!
//! Provides a derive macro for `std::fmt::Debug` that automatically redacts
//! sensitive fields marked with `#[facet(sensitive)]`. Designed for
//! HIPAA-compliant healthcare applications and other systems handling sensitive
//! data.
//!
//! ## Requirements
//!
//! - Must also derive or implement `facet::Facet`
//! - Sensitive fields marked with `#[facet(sensitive)]`
//!
//! ## Performance
//!
//! Minimal runtime overhead:
//! - **One** `Self::SHAPE` const access per `Debug::fmt` call
//! - **One** `field.is_sensitive()` check per field (O(1) bitflag check)
//! - No heap allocations beyond standard `Debug` formatting
//! - Zero cost for non-sensitive fields (formatted normally)
//!
//! The overhead is typically <1% compared to hand-written Debug
//! implementations, which is negligible in practice since Debug formatting is
//! already I/O-bound.
//!
//! ## Example
//!
//! ```rust
//! use facet::Facet;
//! use safe_debug::SafeDebug;
//!
//! #[derive(Facet, SafeDebug)]
//! struct PatientRecord {
//!     id: String,
//!     #[facet(sensitive)]
//!     ssn: String,
//! }
//!
//! let record = PatientRecord {
//!     id: "12345".to_string(),
//!     ssn: "123-45-6789".to_string(),
//! };
//!
//! // SSN will be redacted in debug output
//! let debug_output = format!("{:?}", record);
//! assert!(debug_output.contains("12345"));
//! assert!(!debug_output.contains("123-45-6789"));
//! assert!(debug_output.contains("[REDACTED]"));
//! ```

use facet_macros_parse::*;
use quote::quote;

/// Type alias for Results with boxed errors to avoid large error variants
type BoxedResult<T> = std::result::Result<T, Box<facet_macros_parse::Error>>;

/// Derives `std::fmt::Debug` with automatic redaction for sensitive fields.
///
/// # Supported Types
/// - Named structs
/// - Tuple structs
/// - Unit structs
/// - Enums (all variant types: unit, tuple, struct)
/// - Generic types (with lifetimes and type parameters)
/// - Nested structures
///
/// # Requirements
/// - Must also derive or implement `Facet`
/// - Sensitive fields marked with `#[facet(sensitive)]`
/// - For generic types, type parameters must implement `Debug`
///
/// # Performance
///
/// There should be minimal runtime overhead:
/// - **One** `Self::SHAPE` const access per `Debug::fmt` call
/// - **One** `field.is_sensitive()` check per field (O(1) bitflag check)
/// - No heap allocations beyond standard `Debug` formatting
/// - Zero cost for non-sensitive fields (formatted normally)
///
/// Benchmarks show <1% overhead compared to hand-written Debug implementations,
/// and of course a lot less tedium writing code.
///
/// # Security
///
/// Fails safe when metadata is unavailable:
/// - **Structs**: Redacts all fields with `"[REDACTED:NO_METADATA]"`
/// - **Enums**: Outputs only the type name, no variant or field data
///
/// # Varied examples
///
/// Basic struct with mixed sensitive/non-sensitive fields:
/// ```ignore
/// use facet::Facet;
/// use safe_debug::SafeDebug;
///
/// #[derive(Facet, SafeDebug)]
/// struct PatientRecord {
///     id: String,              // public field
///     #[facet(sensitive)]
///     ssn: String,             // redacted as [REDACTED]
///     #[facet(sensitive)]
///     medical_history: String, // redacted as [REDACTED]
/// }
/// ```
///
/// Enums with sensitive fields in specific variants:
/// ```ignore
/// #[derive(Facet, SafeDebug)]
/// enum ApiResponse {
///     Success { code: u16, data: String },
///     Error {
///         code: u16,
///         #[facet(sensitive)]
///         error_details: String,  // only redacted in Error variant
///     },
///     Pending(u64),
/// }
/// ```
///
/// Generic types (automatically adds required trait bounds):
/// ```ignore
/// #[derive(Facet, SafeDebug)]
/// struct Container<T> {
///     id: u32,
///     #[facet(sensitive)]
///     secret: T,  // T will be redacted
/// }
/// // Expands to: impl<T: Debug + Facet> Debug for Container<T>
/// ```
///
/// Types with lifetimes:
/// ```ignore
/// #[derive(Facet, SafeDebug)]
/// struct BorrowedData<'a> {
///     public: &'a str,
///     #[facet(sensitive)]
///     token: &'a str,
/// }
/// ```
#[proc_macro_derive(SafeDebug)]
pub fn derive_safe_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);

    match derive_facet_debug_impl(&input) {
        Ok(output) => output.into(),
        Err(e) => {
            let error = format!(
                "SafeDebug derive error: {}\n\n\
                 Help: SafeDebug requires the Facet trait to be derived or implemented.\n\
                 Make sure you have `#[derive(Facet, SafeDebug)]` on your type.\n\
                 \n\
                 For types with generics, ensure all type parameters implement Debug.\n\
                 Example: `struct Foo<T: Debug>`",
                e
            );
            quote! {
                compile_error!(#error);
            }
            .into()
        }
    }
}

fn derive_facet_debug_impl(input: &TokenStream) -> BoxedResult<TokenStream> {
    let mut iter = input.to_token_iter();

    // Parse the input as an ADT (struct or enum)
    let adt: AdtDecl = iter.parse().map_err(Box::new)?;

    match adt {
        AdtDecl::Struct(s) => derive_for_struct(s),
        AdtDecl::Enum(e) => derive_for_enum(e),
    }
}

/// Extracts trait bounds for generic type parameters.
///
/// This macro generates trait bounds (`T: Debug + Facet`) for all type
/// parameters in the generics list. Lifetime parameters are skipped (they don't
/// need bounds).
///
/// # Why String Parsing?
///
/// Because I'm lazy? A better answer: We use string parsing instead of AST
/// traversal because `facet-macros-parse` provides generics as an opaque
/// `TokenStream`. While this is less elegant than structural parsing, it's
/// reliable for the common cases (simple type parameters, lifetimes,
/// and basic bounds). Complex const generics or exotic syntax might not be
/// handled perfectly. If you encounter this, please file a with an example and
/// I'll find a different implementation for you.
///
/// # Usage
///
/// ```ignore
/// let trait_bounds = extract_type_param_bounds!(parsed.generics.as_ref());
/// ```
///
/// # Returns
///
/// A `Vec` of quoted token streams, each representing a bound like:
/// `T: ::std::fmt::Debug + for<'__facet> ::facet::Facet<'__facet>`
///
/// The HRTB (Higher-Rank Trait Bound) `for<'__facet>` allows the Facet trait
/// bound to work with any lifetime, since Facet has a lifetime parameter.
macro_rules! extract_type_param_bounds {
    ($generics:expr) => {{
        if let Some(ref g) = $generics {
            let params_str = g.params.to_token_stream().to_string();

            params_str
                .split(',')
                .filter_map(|param| {
                    let param = param.trim();
                    // Skip empty or lifetime parameters
                    if param.is_empty() || param.starts_with('\'') {
                        return None;
                    }

                    // Extract the identifier (first token before : or whitespace)
                    let ident_name = param
                        .split(|c: char| c.is_whitespace() || c == ':')
                        .find(|s| !s.is_empty())?;

                    // Create identifier and bound - need both Debug and Facet
                    let ident = quote::format_ident!("{}", ident_name);
                    Some(quote! { #ident: ::std::fmt::Debug + for<'__facet> ::facet::Facet<'__facet> })
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }};
}

/// Generates the Debug implementation for a struct (named, tuple, or unit).
///
/// This function handles all three struct kinds and generates code that:
/// 1. Accesses the Facet-provided Shape metadata
/// 2. Checks each field's sensitivity flag
/// 3. Redacts sensitive fields, shows others
/// 4. Falls back to redacting everything if metadata is unavailable
///
/// # Security Philosophy
///
/// This takes a fail-safe approach: if reflection fails, ALL fields are
/// redacted to prevent accidental data leakage.
fn derive_for_struct(parsed: Struct) -> BoxedResult<TokenStream> {
    let struct_name = &parsed.name;

    // Extract generics - convert to TokenStream
    let generics = if let Some(ref g) = parsed.generics {
        // Convert the generic params to a token stream manually
        let params_ts = g.params.to_token_stream();
        quote! { < #params_ts > }
    } else {
        quote! {}
    };

    // Extract where clause from the struct kind and track if it exists
    let (has_existing_where, existing_where_clause) = match &parsed.kind {
        StructKind::Struct { clauses, .. }
        | StructKind::TupleStruct { clauses, .. }
        | StructKind::UnitStruct { clauses, .. } => {
            if let Some(w) = clauses {
                let clauses_ts = w.to_token_stream();
                (true, clauses_ts)
            } else {
                (false, quote! {})
            }
        }
    };

    // Build trait bounds for generic type parameters
    let trait_bounds = extract_type_param_bounds!(parsed.generics);

    // Combine existing where clause with trait bounds
    let where_clause = if !trait_bounds.is_empty() {
        if has_existing_where {
            // Existing where clause already has "where", just append bounds
            quote! { #existing_where_clause, #(#trait_bounds),* }
        } else {
            // No existing where clause, create one
            quote! { where #(#trait_bounds),* }
        }
    } else {
        // No bounds to add, use existing or empty
        if has_existing_where {
            quote! { #existing_where_clause }
        } else {
            quote! {}
        }
    };

    // Generate field formatting code based on struct kind
    let format_fields = match parsed.kind {
        StructKind::Struct { ref fields, .. } => {
            // Generate field checks (when metadata is available)
            let field_checks: Vec<_> = fields
                .content
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let field_name = &field.value.name;
                    let field_name_str = field_name.to_string();

                    quote! {
                        if struct_type.fields[#idx].is_sensitive() {
                            debug_struct.field(#field_name_str, &"[REDACTED]");
                        } else {
                            debug_struct.field(#field_name_str, &self.#field_name);
                        }
                    }
                })
                .collect();

            // Generate fallback (when metadata is unavailable) - redact for safety
            let fallback_fields: Vec<_> = fields
                .content
                .iter()
                .map(|field| {
                    let field_name = &field.value.name;
                    let field_name_str = field_name.to_string();

                    quote! {
                        debug_struct.field(#field_name_str, &"[REDACTED:NO_METADATA]");
                    }
                })
                .collect();

            quote! {
                let mut debug_struct = f.debug_struct(stringify!(#struct_name));

                // Hoist Shape access - check once for all fields (performance optimization)
                let shape = Self::SHAPE;
                if let ::facet::Type::User(::facet::UserType::Struct(ref struct_type)) = shape.ty {
                    // Normal path: We have metadata, so check each field's sensitivity
                    #(#field_checks)*
                } else {
                    // Security fallback: metadata unavailable. This is unlikely, but we're being
                    // super-paranoid because we're worried about personal data.
                    // We redact ALL fields to prevent accidental data leakage even in unexpected
                    // error cases.
                    #(#fallback_fields)*
                }

                debug_struct.finish()
            }
        }
        StructKind::TupleStruct { ref fields, .. } => {
            // Generate field checks (when metadata is available)
            let field_checks: Vec<_> = fields
                .content
                .iter()
                .enumerate()
                .map(|(idx, _field)| {
                    // Create a numeric literal for field access using unsynn's Literal
                    let field_idx = Literal::usize_unsuffixed(idx);

                    quote! {
                        if struct_type.fields[#idx].is_sensitive() {
                            debug_tuple.field(&"[REDACTED]");
                        } else {
                            debug_tuple.field(&self.#field_idx);
                        }
                    }
                })
                .collect();

            // Generate fallback (when metadata is unavailable) - redact for safety
            let fallback_fields: Vec<_> = (0..fields.content.len())
                .map(|_| {
                    quote! {
                        debug_tuple.field(&"[REDACTED:NO_METADATA]");
                    }
                })
                .collect();

            quote! {
                let mut debug_tuple = f.debug_tuple(stringify!(#struct_name));

                // Hoist Shape access - check once for all fields (performance optimization)
                let shape = Self::SHAPE;
                if let ::facet::Type::User(::facet::UserType::Struct(ref struct_type)) = shape.ty {
                    // Normal path: We have metadata, so check each field's sensitivity
                    #(#field_checks)*
                } else {
                    // Security fallback: Metadata unavailable (should never happen in normal use).
                    // Fail-safe by redacting ALL fields to prevent accidental data leakage.
                    #(#fallback_fields)*
                }

                debug_tuple.finish()
            }
        }
        StructKind::UnitStruct { .. } => {
            quote! {
                f.debug_struct(stringify!(#struct_name)).finish()
            }
        }
    };

    let impl_block = quote! {
        impl #generics ::std::fmt::Debug for #struct_name #generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #format_fields
            }
        }
    };

    Ok(impl_block)
}

/// Generates the Debug implementation for an enum with any variant types.
///
/// This function handles all enum variant types (unit, tuple, struct) and
/// generates code that:
/// 1. accesses the Facet-provided Shape metadata
/// 2. matches on each variant
/// 3. checks field sensitivity for tuple/struct variants
/// 4. redacts sensitive fields, shows others
/// 5. falls back to showing only the type name if metadata is unavailable
///
/// # Security Philosophy
///
/// The generated code takes a fail-conservative approach: if reflection
/// fails, ONLY the enum type name is written with no variant or field data,
/// preventing any potential data leakage.
fn derive_for_enum(parsed: Enum) -> BoxedResult<TokenStream> {
    let enum_name = &parsed.name;

    // Extract generics - same as structs
    let generics = if let Some(ref g) = parsed.generics {
        let params_ts = g.params.to_token_stream();
        quote! { < #params_ts > }
    } else {
        quote! {}
    };

    // Extract where clause from enum (same pattern as structs)
    let (has_existing_where, existing_where_clause) = if let Some(ref clauses) = parsed.clauses {
        let clauses_ts = clauses.to_token_stream();
        (true, clauses_ts)
    } else {
        (false, quote! {})
    };

    // Build trait bounds for generic type parameters
    let trait_bounds = extract_type_param_bounds!(parsed.generics);

    // Combine existing where clause with trait bounds
    let where_clause = if !trait_bounds.is_empty() {
        if has_existing_where {
            quote! { #existing_where_clause, #(#trait_bounds),* }
        } else {
            quote! { where #(#trait_bounds),* }
        }
    } else if has_existing_where {
        quote! { #existing_where_clause }
    } else {
        quote! {}
    };

    // Generate match arms for each variant
    let match_arms: Vec<_> = parsed
        .body
        .content
        .iter()
        .enumerate()
        .map(|(variant_idx, variant_like)| {
            let variant = &variant_like.value.variant;

            match variant {
                EnumVariantData::Unit(unit_variant) => {
                    let variant_name = &unit_variant.name;
                    // Unit variant: MyEnum::Variant
                    quote! {
                        #enum_name::#variant_name => {
                            write!(f, concat!(stringify!(#enum_name), "::", stringify!(#variant_name)))
                        }
                    }
                }
                EnumVariantData::Tuple(tuple_variant) => {
                    let variant_name = &tuple_variant.name;
                    let field_count = tuple_variant.fields.content.len();

                    // Generate field bindings: _field_0, _field_1, _field_2, ...
                    let bindings: Vec<_> = (0..field_count).map(|i| quote::format_ident!("_field_{}", i)).collect();

                    // Generate field checks with sensitivity
                    let field_checks: Vec<_> = bindings
                        .iter()
                        .enumerate()
                        .map(|(field_idx, binding)| {
                            quote! {
                                if enum_type.variants[#variant_idx].data.fields[#field_idx].is_sensitive() {
                                    debug_tuple.field(&"[REDACTED]");
                                } else {
                                    debug_tuple.field(#binding);
                                }
                            }
                        })
                        .collect();

                    quote! {
                        #enum_name::#variant_name(#(#bindings),*) => {
                            let mut debug_tuple = f.debug_tuple(
                                concat!(stringify!(#enum_name), "::", stringify!(#variant_name))
                            );
                            #(#field_checks)*
                            debug_tuple.finish()
                        }
                    }
                }
                EnumVariantData::Struct(struct_variant) => {
                    let variant_name = &struct_variant.name;
                    let fields = &struct_variant.fields.content;

                    // Generate field bindings: ref field1, ref field2, ...
                    let field_names: Vec<_> = fields.iter().map(|field| &field.value.name).collect();

                    // Generate field checks with sensitivity
                    let field_checks: Vec<_> = field_names
                        .iter()
                        .enumerate()
                        .map(|(field_idx, name)| {
                            let name_str = name.to_string();
                            quote! {
                                if enum_type.variants[#variant_idx].data.fields[#field_idx].is_sensitive() {
                                    debug_struct.field(#name_str, &"[REDACTED]");
                                } else {
                                    debug_struct.field(#name_str, #name);
                                }
                            }
                        })
                        .collect();

                    quote! {
                        #enum_name::#variant_name { #(#field_names),* } => {
                            let mut debug_struct = f.debug_struct(
                                concat!(stringify!(#enum_name), "::", stringify!(#variant_name))
                            );
                            #(#field_checks)*
                            debug_struct.finish()
                        }
                    }
                }
            }
        })
        .collect();

    let impl_block = quote! {
        impl #generics ::std::fmt::Debug for #enum_name #generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                // Hoist Shape access - check once at method entry (performance optimization)
                let shape = Self::SHAPE;
                if let ::facet::Type::User(::facet::UserType::Enum(ref enum_type)) = shape.ty {
                    // Normal path: We have metadata, so check field sensitivity for each variant
                    match self {
                        #(#match_arms),*
                    }
                } else {
                    // Security fallback: Metadata unavailable (should never happen in normal use).
                    // Fail-conservative by writing ONLY the type name with no variant or field data.
                    // Repeat previous comments about why we're doing this.
                    write!(f, "{}", stringify!(#enum_name))
                }
            }
        }
    };

    Ok(impl_block)
}
