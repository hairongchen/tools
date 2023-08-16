pub mod tee_tdx_lib;
use tee_tdx_lib::*;

fn main() {
    let q = match get_tdx_quote(
        "1234567812345678123456781234567812345678123456781234567812345678".to_string(),
    ) {
        Err(e) => panic!("Fail to get TDX quote: {:?}", e),
        Ok(q) => q,
    };
    println!("quote size = {}", q.len());
}
