fn main() {

    let report =  match tdx_attest::get_td_report("XUccU3O9poJXiX53jNGj1w2v4WVAw8TKDyWm8Y0xgJ2khEMyCSCiWfO/sYMEn5xoC8ES2VzXwmKRv9NVu3YnUA==".to_string()) {
        Err(e) => panic!("[get_td_report] Fail to get TDX report: {:?}", e),
        Ok(r) => r,
    };

    println!("{:?}",report);


    let quote = match tdx_attest::get_tdx_quote("XUccU3O9poJXiX53jNGj1w2v4WVAw8TKDyWm8Y0xgJ2khEMyCSCiWfO/sYMEn5xoC8ES2VzXwmKRv9NVu3YnUA==".to_string()) {
        Err(e) => panic!("[get_tdx_quote] Fail to get TDX quote: {:?}", e),
        //Ok(q) => base64::encode(q),
        Ok(q) => q,
    };

    println!("{:?}",quote);

}

