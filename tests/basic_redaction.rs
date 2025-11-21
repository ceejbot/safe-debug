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

#[test]
fn test_redaction_basic() {
    let record = PatientRecord {
        id: "12345".to_string(),
        name: "John Doe".to_string(),
        ssn: "123-45-6789".to_string(),
        medical_history: "Confidential medical data".to_string(),
    };

    let debug_output = format!("{:?}", record);

    // Public fields should be visible
    assert!(debug_output.contains("12345"), "ID should be visible");
    assert!(debug_output.contains("John Doe"), "Name should be visible");

    // Sensitive fields should be redacted
    assert!(!debug_output.contains("123-45-6789"), "SSN should be redacted");
    assert!(
        !debug_output.contains("Confidential medical data"),
        "Medical history should be redacted"
    );

    // Redaction marker should be present
    assert!(
        debug_output.contains("[REDACTED]"),
        "Redaction marker should be present"
    );
}

#[test]
fn test_pretty_debug() {
    let record = PatientRecord {
        id: "12345".to_string(),
        name: "John Doe".to_string(),
        ssn: "123-45-6789".to_string(),
        medical_history: "Confidential medical data".to_string(),
    };

    let debug_output = format!("{:#?}", record);

    // Should still redact in pretty-print mode
    assert!(
        !debug_output.contains("123-45-6789"),
        "SSN should be redacted in pretty mode"
    );
    assert!(
        debug_output.contains("[REDACTED]"),
        "Redaction marker should be present in pretty mode"
    );
}

#[derive(Facet, SafeDebug)]
struct AllPublic {
    field1: String,
    field2: i32,
}

#[test]
fn test_no_sensitive_fields() {
    let data = AllPublic {
        field1: "visible".to_string(),
        field2: 42,
    };

    let debug_output = format!("{:?}", data);

    // All fields should be visible
    assert!(debug_output.contains("visible"));
    assert!(debug_output.contains("42"));

    // No redaction marker should be present
    assert!(!debug_output.contains("[REDACTED]"));
}

#[derive(Facet, SafeDebug)]
struct AllSensitive {
    #[facet(sensitive)]
    secret1: String,
    #[facet(sensitive)]
    secret2: String,
}

#[test]
fn test_all_sensitive_fields() {
    let data = AllSensitive {
        secret1: "hidden1".to_string(),
        secret2: "hidden2".to_string(),
    };

    let debug_output = format!("{:?}", data);

    // All sensitive data should be redacted
    assert!(!debug_output.contains("hidden1"));
    assert!(!debug_output.contains("hidden2"));

    // Redaction markers should be present
    assert!(debug_output.contains("[REDACTED]"));
}

#[derive(Facet, SafeDebug)]
struct UnitStruct;

#[test]
fn test_unit_struct() {
    let data = UnitStruct;
    let debug_output = format!("{:?}", data);

    // Should just show the struct name
    assert!(debug_output.contains("UnitStruct"));
}

#[derive(Facet, SafeDebug)]
struct TupleStruct(String, #[facet(sensitive)] String, i32);

#[test]
fn test_tuple_struct() {
    let data = TupleStruct("visible".to_string(), "hidden".to_string(), 42);

    let debug_output = format!("{:?}", data);

    // First field should be visible
    assert!(debug_output.contains("visible"));

    // Second field should be redacted
    assert!(!debug_output.contains("hidden"));
    assert!(debug_output.contains("[REDACTED]"));

    // Third field should be visible
    assert!(debug_output.contains("42"));
}

// Struct with lifetime tests

#[derive(Facet, SafeDebug)]
struct WithLifetime<'a> {
    name: &'a str,
    value: u32,
}

#[test]
fn test_with_lifetime() {
    let data = WithLifetime {
        name: "test",
        value: 100,
    };

    let debug_output = format!("{:?}", data);

    // Both fields should be visible
    assert!(debug_output.contains("test"));
    assert!(debug_output.contains("100"));
}

#[derive(Facet, SafeDebug)]
struct WithSensitiveLifetime<'a> {
    id: u32,
    #[facet(sensitive)]
    secret: &'a str,
}

#[test]
fn test_with_sensitive_lifetime() {
    let secret_data = "password123";
    let data = WithSensitiveLifetime {
        id: 456,
        secret: secret_data,
    };

    let debug_output = format!("{:?}", data);

    // ID should be visible
    assert!(debug_output.contains("456"));

    // Secret should be redacted
    assert!(!debug_output.contains("password123"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Nested structs

#[derive(Facet, SafeDebug)]
struct Inner {
    public_data: String,
    #[facet(sensitive)]
    secret_data: String,
}

#[derive(Facet, SafeDebug)]
struct Outer {
    id: u32,
    inner: Inner,
}

#[test]
fn test_nested_struct() {
    let data = Outer {
        id: 999,
        inner: Inner {
            public_data: "visible".to_string(),
            secret_data: "hidden".to_string(),
        },
    };

    let debug_output = format!("{:?}", data);

    // Outer ID should be visible
    assert!(debug_output.contains("999"));

    // Inner public data should be visible
    assert!(debug_output.contains("visible"));

    // Inner secret should be redacted
    assert!(!debug_output.contains("hidden"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Edge cases

#[derive(Facet, SafeDebug)]
struct EmptyStruct {}

#[test]
fn test_empty_struct() {
    let data = EmptyStruct {};
    let debug_output = format!("{:?}", data);

    // Should just show the struct name
    assert!(debug_output.contains("EmptyStruct"));

    // No redaction needed
    assert!(!debug_output.contains("[REDACTED]"));
}

#[derive(Facet, SafeDebug)]
struct MixedLifetimeTuple<'a>(u32, #[facet(sensitive)] &'a str);

#[test]
fn test_lifetime_tuple_struct() {
    let secret = "secret";
    let data = MixedLifetimeTuple(42, secret);

    let debug_output = format!("{:?}", data);

    // First field should be visible
    assert!(debug_output.contains("42"));

    // Second field should be redacted
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Enum tests - Unit variant

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum SimpleStatus {
    Active,
    Inactive,
    Pending,
}

#[test]
fn test_enum_unit_variant() {
    let status = SimpleStatus::Active;
    let debug_output = format!("{:?}", status);

    // Should display enum name and variant
    assert!(debug_output.contains("SimpleStatus"));
    assert!(debug_output.contains("Active"));

    // Unit variants have no fields, so no redaction
    assert!(!debug_output.contains("[REDACTED]"));
}

#[test]
fn test_enum_unit_variant_inactive() {
    let status = SimpleStatus::Inactive;
    let debug_output = format!("{:?}", status);

    assert!(debug_output.contains("SimpleStatus"));
    assert!(debug_output.contains("Inactive"));
    assert!(!debug_output.contains("[REDACTED]"));
}

#[test]
fn test_enum_unit_variant_pending() {
    let status = SimpleStatus::Pending;
    let debug_output = format!("{:?}", status);

    assert!(debug_output.contains("SimpleStatus"));
    assert!(debug_output.contains("Pending"));
    assert!(!debug_output.contains("[REDACTED]"));
}

// Enum tests - Tuple variant

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum Response {
    Success(u32, String),
    Error(i32, #[facet(sensitive)] String),
}

#[test]
fn test_enum_tuple_variant_no_sensitive() {
    let response = Response::Success(200, "OK".to_string());
    let debug_output = format!("{:?}", response);

    // All fields should be visible
    assert!(debug_output.contains("Response"));
    assert!(debug_output.contains("Success"));
    assert!(debug_output.contains("200"));
    assert!(debug_output.contains("OK"));

    // No redaction needed
    assert!(!debug_output.contains("[REDACTED]"));
}

#[test]
fn test_enum_tuple_variant_with_sensitive() {
    let response = Response::Error(500, "Internal error: password123".to_string());
    let debug_output = format!("{:?}", response);

    // Error code should be visible
    assert!(debug_output.contains("Response"));
    assert!(debug_output.contains("Error"));
    assert!(debug_output.contains("500"));

    // Error message should be redacted
    assert!(!debug_output.contains("password123"));
    assert!(debug_output.contains("[REDACTED]"));
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum Edge {
    Empty,
    SingleTuple(()),
}

#[test]
fn test_enum_empty_variant() {
    let edge = Edge::Empty;
    let debug_output = format!("{:?}", edge);
    assert!(debug_output.contains("Edge"));
    assert!(debug_output.contains("Empty"));
}

#[test]
fn test_enum_empty_tuple() {
    let edge = Edge::SingleTuple(());
    let debug_output = format!("{:?}", edge);
    assert!(debug_output.contains("Edge::SingleTuple"));
}

// Enum tests - Struct variant

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum Message {
    Text {
        content: String,
    },
    Secure {
        id: u32,
        #[facet(sensitive)]
        auth_token: String,
        #[facet(sensitive)]
        payload: String,
    },
    Mixed(u32, #[facet(sensitive)] String),
}

#[test]
fn test_enum_struct_variant_no_sensitive() {
    let msg = Message::Text {
        content: "Hello world".to_string(),
    };
    let debug_output = format!("{:?}", msg);

    assert!(debug_output.contains("Message"));
    assert!(debug_output.contains("Text"));
    assert!(debug_output.contains("content"));
    assert!(debug_output.contains("Hello world"));
    assert!(!debug_output.contains("[REDACTED]"));
}

#[test]
fn test_enum_struct_variant_with_sensitive() {
    let msg = Message::Secure {
        id: 42,
        auth_token: "secret_token_123".to_string(),
        payload: "confidential_data".to_string(),
    };
    let debug_output = format!("{:?}", msg);

    // ID should be visible
    assert!(debug_output.contains("Message"));
    assert!(debug_output.contains("Secure"));
    assert!(debug_output.contains("id"));
    assert!(debug_output.contains("42"));

    // Sensitive fields should be redacted
    assert!(!debug_output.contains("secret_token_123"));
    assert!(!debug_output.contains("confidential_data"));
    assert!(debug_output.contains("[REDACTED]"));

    // Field names should still be visible
    assert!(debug_output.contains("auth_token"));
    assert!(debug_output.contains("payload"));
}

#[test]
fn test_enum_mixed_variants() {
    // Test that different variant types work together
    let messages = vec![
        Message::Text {
            content: "public".to_string(),
        },
        Message::Secure {
            id: 1,
            auth_token: "secret1".to_string(),
            payload: "secret2".to_string(),
        },
        Message::Mixed(99, "secret3".to_string()),
    ];

    for msg in &messages {
        let debug_output = format!("{:?}", msg);
        // All should format without panicking
        assert!(debug_output.contains("Message"));
    }
}

// Test generic struct first
#[derive(Facet, SafeDebug)]
struct GenericStruct<T> {
    value: T,
}

#[test]
fn test_generic_struct() {
    let s: GenericStruct<u32> = GenericStruct { value: 42 };
    let debug_output = format!("{:?}", s);
    assert!(debug_output.contains("42"));
}

// Enum tests - Generics and lifetimes

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum SimpleEnum<'a> {
    Borrowed(&'a str),
    Owned(String),
}

#[test]
fn test_enum_with_lifetime() {
    let data = "test data";
    let borrowed = SimpleEnum::Borrowed(data);
    let debug_output = format!("{:?}", borrowed);

    assert!(debug_output.contains("SimpleEnum"));
    assert!(debug_output.contains("Borrowed"));
    assert!(debug_output.contains("test data"));
}

#[test]
fn test_enum_with_lifetime_owned() {
    let owned = SimpleEnum::Owned("owned data".to_string());
    let debug_output = format!("{:?}", owned);

    assert!(debug_output.contains("SimpleEnum"));
    assert!(debug_output.contains("Owned"));
    assert!(debug_output.contains("owned data"));
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum Container<'a> {
    Borrowed {
        #[facet(sensitive)]
        data: &'a str,
    },
    Owned {
        data: String,
    },
}

#[test]
fn test_enum_generic_struct_variant() {
    let borrowed = Container::Borrowed { data: "secret" };
    let debug_output = format!("{:?}", borrowed);

    // Sensitive borrowed data should be redacted
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("[REDACTED]"));

    let owned = Container::Owned {
        data: "public".to_string(),
    };
    let debug_output = format!("{:?}", owned);

    // Public owned data should be visible
    assert!(debug_output.contains("public"));
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum GenericEnum<T> {
    Value(T),
    Empty,
}

#[test]
fn test_enum_generic_multiple_types() {
    let str_val: GenericEnum<String> = GenericEnum::Value("success".to_string());
    let debug_output = format!("{:?}", str_val);
    assert!(debug_output.contains("success"));

    let int_val: GenericEnum<i32> = GenericEnum::Value(100);
    let debug_output = format!("{:?}", int_val);
    assert!(debug_output.contains("100"));
}

#[test]
fn test_enum_generic_empty_variant() {
    let empty_str: GenericEnum<String> = GenericEnum::Empty;
    let debug_output = format!("{:?}", empty_str);
    assert!(debug_output.contains("GenericEnum"));
    assert!(debug_output.contains("Empty"));

    let empty_int: GenericEnum<i32> = GenericEnum::Empty;
    let debug_output = format!("{:?}", empty_int);
    assert!(debug_output.contains("GenericEnum"));
    assert!(debug_output.contains("Empty"));
}

// Enum tests - Nested structures

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum InnerEnum {
    Public(String),
    Secret(#[facet(sensitive)] String),
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum OuterEnum {
    Simple(u32),
    Nested { id: u32, inner: InnerEnum },
}

#[test]
fn test_nested_enum_simple_variant() {
    let outer = OuterEnum::Simple(42);
    let debug_output = format!("{:?}", outer);

    assert!(debug_output.contains("OuterEnum"));
    assert!(debug_output.contains("Simple"));
    assert!(debug_output.contains("42"));
}

#[test]
fn test_nested_enum_with_public_inner() {
    let outer = OuterEnum::Nested {
        id: 1,
        inner: InnerEnum::Public("visible".to_string()),
    };
    let debug_output = format!("{:?}", outer);

    assert!(debug_output.contains("Nested"));
    assert!(debug_output.contains("1"));
    assert!(debug_output.contains("Public"));
    assert!(debug_output.contains("visible"));
}

#[test]
fn test_nested_enum_with_secret_inner() {
    let outer = OuterEnum::Nested {
        id: 2,
        inner: InnerEnum::Secret("hidden".to_string()),
    };
    let debug_output = format!("{:?}", outer);

    // Outer ID visible
    assert!(debug_output.contains("Nested"));
    assert!(debug_output.contains("2"));

    // Inner type visible but content redacted
    assert!(debug_output.contains("Secret"));
    assert!(!debug_output.contains("hidden"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Enum tests - Security fallback
//
// When facet metadata is unavailable (shouldn't happen in practice),
// the fallback for enums is to just write the enum type name.
// This is conservative - we don't expose any variant or field data.
// This matches our security-hardened philosophy.

#[test]
fn test_enum_pretty_print() {
    let msg = Message::Secure {
        id: 42,
        auth_token: "secret123".to_string(),
        payload: "confidential".to_string(),
    };

    let debug_output = format!("{:#?}", msg);

    // Should still redact in pretty mode
    assert!(!debug_output.contains("secret123"));
    assert!(!debug_output.contains("confidential"));
    assert!(debug_output.contains("[REDACTED]"));

    // Should be formatted with newlines (pretty)
    assert!(debug_output.contains('\n'));
}

// Advanced scenarios - Multi-generic types

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct Pair<T, U> {
    first: T,
    #[facet(sensitive)]
    second: U,
}

#[test]
fn test_multi_generic_struct() {
    let pair = Pair {
        first: "public".to_string(),
        second: "secret".to_string(),
    };
    let debug_output = format!("{:?}", pair);

    // First field visible, second redacted
    assert!(debug_output.contains("public"));
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("[REDACTED]"));
}

#[test]
fn test_multi_generic_struct_different_types() {
    let pair = Pair {
        first: 42,
        second: vec![1, 2, 3],
    };
    let debug_output = format!("{:?}", pair);

    // First field visible, second redacted
    assert!(debug_output.contains("42"));
    assert!(!debug_output.contains("[1, 2, 3]"));
    assert!(debug_output.contains("[REDACTED]"));
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum Either<L, R> {
    Left(L),
    Right(#[facet(sensitive)] R),
}

#[test]
fn test_multi_generic_enum() {
    let left: Either<String, i32> = Either::Left("visible".to_string());
    let debug_output = format!("{:?}", left);
    assert!(debug_output.contains("visible"));
    assert!(!debug_output.contains("[REDACTED]"));

    let right: Either<String, i32> = Either::Right(42);
    let debug_output = format!("{:?}", right);
    assert!(!debug_output.contains("42"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Advanced scenarios - Multi-type parameters (where clauses tested separately)

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct Triple<T, U, V> {
    first: T,
    #[facet(sensitive)]
    second: U,
    third: V,
}

#[test]
fn test_three_type_parameters() {
    let triple = Triple {
        first: "public".to_string(),
        second: 42,
        third: vec![1, 2, 3],
    };
    let debug_output = format!("{:?}", triple);

    // First and third visible, second redacted
    assert!(debug_output.contains("public"));
    assert!(!debug_output.contains("42"));
    assert!(debug_output.contains("[1, 2, 3]"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Advanced scenarios - Mixed field types

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct Coordinates(f64, f64);

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct Location {
    name: String,
    coords: Coordinates,
    #[facet(sensitive)]
    owner: String,
}

#[test]
fn test_mixed_nested_types() {
    let location = Location {
        name: "Office".to_string(),
        coords: Coordinates(40.7128, -74.0060),
        owner: "Secret Corp".to_string(),
    };
    let debug_output = format!("{:?}", location);

    // Name and coordinates visible
    assert!(debug_output.contains("Office"));
    assert!(debug_output.contains("40.7128"));
    assert!(debug_output.contains("-74.006"));

    // Owner redacted
    assert!(!debug_output.contains("Secret Corp"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Edge cases - Deeply nested generics

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct Box<T> {
    inner: T,
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct SecureBox<T> {
    #[facet(sensitive)]
    inner: T,
}

#[test]
fn test_deeply_nested_generics() {
    // Four levels deep: Box<Box<Box<SecureBox<String>>>>
    let deeply_nested = Box {
        inner: Box {
            inner: Box {
                inner: SecureBox {
                    inner: "secret_data".to_string(),
                },
            },
        },
    };
    let debug_output = format!("{:?}", deeply_nested);

    // The deeply nested structure should be visible
    assert!(debug_output.contains("Box"));
    assert!(debug_output.contains("SecureBox"));

    // But the innermost sensitive data should be redacted
    assert!(!debug_output.contains("secret_data"));
    assert!(debug_output.contains("[REDACTED]"));
}

#[test]
fn test_nested_generic_enums() {
    // Outer Left is not sensitive, but inner Either::Right(42) IS sensitive
    // This correctly tests that sensitivity is preserved through nesting
    let nested: Either<Either<String, i32>, bool> = Either::Left(Either::Right(42));
    let debug_output = format!("{:?}", nested);
    assert!(debug_output.contains("Either"));
    assert!(debug_output.contains("Left"));
    // Inner Right variant is sensitive, so 42 should be redacted
    assert!(debug_output.contains("[REDACTED]"));
    assert!(!debug_output.contains("42"));

    // Outer Left with inner Left (no sensitivity at all)
    let not_sensitive: Either<Either<String, i32>, bool> = Either::Left(Either::Left("visible".to_string()));
    let debug_output = format!("{:?}", not_sensitive);
    assert!(debug_output.contains("visible"));
    assert!(!debug_output.contains("[REDACTED]"));

    // Outer Right IS sensitive, so its contents are redacted
    let nested_sensitive: Either<Either<String, i32>, bool> = Either::Right(true);
    let debug_output = format!("{:?}", nested_sensitive);
    assert!(debug_output.contains("Right"));
    assert!(debug_output.contains("[REDACTED]"));
    assert!(!debug_output.contains("true"));
}

// Edge cases - Lifetime-only generics

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct Borrowed<'a> {
    data: &'a str,
    #[facet(sensitive)]
    secret: &'a str,
}

#[test]
fn test_lifetime_only_struct() {
    let borrowed = Borrowed {
        data: "public",
        secret: "hidden",
    };
    let debug_output = format!("{:?}", borrowed);

    // Public data visible, secret redacted
    assert!(debug_output.contains("public"));
    assert!(!debug_output.contains("hidden"));
    assert!(debug_output.contains("[REDACTED]"));
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum RefEnum<'a> {
    Borrowed(&'a str),
    Secret(#[facet(sensitive)] &'a str),
}

#[test]
fn test_lifetime_only_enum() {
    let borrowed = RefEnum::Borrowed("visible");
    let debug_output = format!("{:?}", borrowed);
    assert!(debug_output.contains("visible"));
    assert!(!debug_output.contains("[REDACTED]"));

    let secret = RefEnum::Secret("hidden");
    let debug_output = format!("{:?}", secret);
    assert!(!debug_output.contains("hidden"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Edge cases - Mixed lifetimes and types

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct MixedParams<'a, T> {
    borrowed: &'a str,
    owned: T,
    #[facet(sensitive)]
    secret: String,
}

#[test]
fn test_mixed_lifetime_and_type_params() {
    let mixed = MixedParams {
        borrowed: "ref_data",
        owned: 42,
        secret: "secret_value".to_string(),
    };
    let debug_output = format!("{:?}", mixed);

    // Borrowed and owned visible
    assert!(debug_output.contains("ref_data"));
    assert!(debug_output.contains("42"));

    // Secret redacted
    assert!(!debug_output.contains("secret_value"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Edge cases - Zero-sized types

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct ZeroSized<T> {
    #[facet(sensitive)]
    _phantom: std::marker::PhantomData<T>,
    data: u32,
}

#[test]
fn test_zero_sized_type() {
    let zst: ZeroSized<String> = ZeroSized {
        _phantom: std::marker::PhantomData,
        data: 100,
    };
    let debug_output = format!("{:?}", zst);

    // Data visible
    assert!(debug_output.contains("100"));

    // PhantomData is redacted (sensitive marker)
    assert!(debug_output.contains("[REDACTED]"));
}

// Edge cases - Generic tuple struct with mixed sensitivity

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct GenericTuple<T, U>(T, #[facet(sensitive)] U);

#[test]
fn test_generic_tuple_struct() {
    let tuple = GenericTuple("public".to_string(), "secret".to_string());
    let debug_output = format!("{:?}", tuple);

    assert!(debug_output.contains("public"));
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("[REDACTED]"));
}

// Edge cases - Multiple references with same lifetime

#[derive(Facet, SafeDebug)]
#[repr(C)]
struct MultipleRefs<'a> {
    first: &'a str,
    #[facet(sensitive)]
    second: &'a str,
    third: &'a [u8],
}

#[test]
fn test_multiple_refs_same_lifetime() {
    let data = b"bytes";
    let multi_ref = MultipleRefs {
        first: "public",
        second: "secret",
        third: data,
    };
    let debug_output = format!("{:?}", multi_ref);

    assert!(debug_output.contains("public"));
    assert!(!debug_output.contains("secret"));
    assert!(debug_output.contains("[REDACTED]"));
    // Third field visible - byte array format
    assert!(debug_output.contains("98") || debug_output.contains("[")); // 98 is 'b' in ASCII
}
