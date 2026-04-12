/// Deriving SafeDebug without Facet should produce a helpful error.
use safe_debug::SafeDebug;

#[derive(SafeDebug)]
struct MissingFacet {
    name: String,
    #[facet(sensitive)]
    secret: String,
}

fn main() {}
