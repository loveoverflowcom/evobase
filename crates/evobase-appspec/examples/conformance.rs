// Same reviewed vector code runs natively and under a WASI runtime; stdout must agree byte-for-byte.
#[path = "../tests/vectors.rs"]
mod vectors;
fn main() {
    println!("{}", vectors::run_vectors());
}
