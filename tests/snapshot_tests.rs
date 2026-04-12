use facet::Facet;
use safe_debug::SafeDebug;

// --- Type definitions for snapshot tests ---

#[derive(Facet, SafeDebug)]
struct PatientRecord {
    id: String,
    name: String,
    #[facet(sensitive)]
    ssn: String,
    #[facet(sensitive)]
    medical_history: String,
}

#[derive(Facet, SafeDebug)]
struct TupleRecord(String, #[facet(sensitive)] String, i32);

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum ApiResponse {
    Success {
        code: u16,
        data: String,
    },
    Error {
        code: u16,
        #[facet(sensitive)]
        details: String,
    },
    Pending,
    Token(#[facet(sensitive)] String),
}

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

// --- Snapshot tests ---

#[test]
fn snapshot_named_struct() {
    let record = PatientRecord {
        id: "P-12345".to_string(),
        name: "Jane Doe".to_string(),
        ssn: "123-45-6789".to_string(),
        medical_history: "Chronic condition notes".to_string(),
    };
    insta::assert_snapshot!(format!("{:?}", record));
}

#[test]
fn snapshot_named_struct_pretty() {
    let record = PatientRecord {
        id: "P-12345".to_string(),
        name: "Jane Doe".to_string(),
        ssn: "123-45-6789".to_string(),
        medical_history: "Chronic condition notes".to_string(),
    };
    insta::assert_snapshot!(format!("{:#?}", record));
}

#[test]
fn snapshot_tuple_struct() {
    let record = TupleRecord("visible".to_string(), "secret-token".to_string(), 42);
    insta::assert_snapshot!(format!("{:?}", record));
}

#[test]
fn snapshot_enum_struct_variant() {
    let success = ApiResponse::Success {
        code: 200,
        data: "OK".to_string(),
    };
    insta::assert_snapshot!("enum_success", format!("{:?}", success));

    let error = ApiResponse::Error {
        code: 500,
        details: "password leaked in stack trace".to_string(),
    };
    insta::assert_snapshot!("enum_error_redacted", format!("{:?}", error));
}

#[test]
fn snapshot_enum_unit_and_tuple_variants() {
    let pending = ApiResponse::Pending;
    insta::assert_snapshot!("enum_pending", format!("{:?}", pending));

    let token = ApiResponse::Token("secret-bearer-token".to_string());
    insta::assert_snapshot!("enum_token_redacted", format!("{:?}", token));
}

#[test]
fn snapshot_nested_struct() {
    let data = Outer {
        id: 999,
        inner: Inner {
            public_data: "visible-info".to_string(),
            secret_data: "classified-payload".to_string(),
        },
    };
    insta::assert_snapshot!(format!("{:?}", data));
}

#[test]
fn snapshot_nested_struct_pretty() {
    let data = Outer {
        id: 999,
        inner: Inner {
            public_data: "visible-info".to_string(),
            secret_data: "classified-payload".to_string(),
        },
    };
    insta::assert_snapshot!(format!("{:#?}", data));
}
