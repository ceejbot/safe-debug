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

    // While logging, somebody drops in data that includes fields with PHI in
    // them...
    println!("very useful but inattentive log line; record='{:?}'", record);
    // but we are saved because the output is:
    // very useful but inattentive log line; record='PatientRecord { id: "12345",
    // name: "John Doe", ssn: "[REDACTED]", medical_history: "[REDACTED]"' }

    // Pretty-print also works
    println!("\nPretty-printing:\n{:#?}", record);
}
