use facet::Facet;
use safe_debug::SafeDebug;

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
        error_details: String,
    },
    Pending(u64),
}

#[derive(Facet, SafeDebug)]
#[repr(C)]
enum AuthEvent<'a> {
    Login {
        user_id: u32,
        #[facet(sensitive)]
        session_token: &'a str,
    },
    Logout {
        user_id: u32,
    },
    Failed {
        #[facet(sensitive)]
        attempted_password: String,
        ip_address: String,
    },
}

fn main() {
    println!("=== Enum Debug Output Examples ===\n");

    // Success case - all fields visible
    let success = ApiResponse::Success {
        code: 200,
        data: "User profile retrieved".to_string(),
    };
    println!("Success response: {:?}", success);

    // Error case - sensitive details redacted
    let error = ApiResponse::Error {
        code: 500,
        error_details: "Database connection failed: password=admin123".to_string(),
    };
    println!("Error response: {:?}", error);

    // Pending case - simple tuple
    let pending = ApiResponse::Pending(12345);
    println!("Pending response: {:?}\n", pending);

    // Auth events with lifetimes
    let token = "secret_session_token_xyz";
    let login = AuthEvent::Login {
        user_id: 42,
        session_token: token,
    };
    println!("Login event: {:?}", login);

    let logout = AuthEvent::Logout { user_id: 42 };
    println!("Logout event: {:?}", logout);

    let failed = AuthEvent::Failed {
        attempted_password: "wrong_password123".to_string(),
        ip_address: "192.168.1.100".to_string(),
    };
    println!("Failed auth: {:?}\n", failed);

    // Pretty print mode
    println!("=== Pretty Print Mode ===\n");
    println!("{:#?}", error);
}
