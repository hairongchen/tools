pub mod tee_tdx_lib;
use tee_tdx_lib::*;

fn main() {

    let quote_vec = get_tdx_quote("1234567812345678123456781234567812345678123456781234567812345678".to_string());
    println!("quote size = {}", quote_vec.len());

}
