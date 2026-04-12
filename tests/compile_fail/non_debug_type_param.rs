/// A generic type parameter that doesn't implement Debug should fail to compile.
use facet::Facet;
use safe_debug::SafeDebug;

struct NotDebug;

#[derive(Facet, SafeDebug)]
struct Container<T> {
    value: T,
}

fn main() {
    let c = Container { value: NotDebug };
    // This should fail because NotDebug doesn't implement Debug
    println!("{:?}", c);
}
