/// Const generics are not supported.
/// Both `Facet` and `SafeDebug` fail on const generic parameters.
/// Facet's derive rejects `const N: usize` outright, and SafeDebug's
/// string-based generics parser produces unparsable tokens.
use facet::Facet;
use safe_debug::SafeDebug;

#[derive(Facet, SafeDebug)]
struct FixedBuffer<const N: usize> {
    #[facet(sensitive)]
    data: [u8; N],
    label: String,
}

fn main() {
    let buf: FixedBuffer<4> = FixedBuffer {
        data: [1, 2, 3, 4],
        label: "test".to_string(),
    };
    println!("{:?}", buf);
}
