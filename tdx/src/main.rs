pub mod tee_tdx_lib;
use tee_tdx_lib::*;

fn main() {
    let q = get_tdx_quote(
        "1234567812345678123456781234567812345678123456781234567812345678".to_string(),
    );
    println!("quote size = {}", q.len());
}
