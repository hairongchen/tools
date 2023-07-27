#[allow(unused_imports)]
use std::os::unix::io::AsRawFd;
use std::fs::File;

#[macro_use] extern crate nix;

const REPORT_DATA_LEN: u32 = 64;
const TDX_REPORT_LEN: u32 = 1024;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct tdx_report_req {
    subtype:            u8,
    reportdata:         u64,
    rpd_len:            u32,
    tdreport:           u64,
    tdr_len:            u32,
}

impl std::fmt::Display for tdx_report_req {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(subtype: {}, reportdata: {:#0x}, rpd_len: {}, tdreport: {:#0x}, tdr_len: {})", self.subtype, self.reportdata, self.rpd_len, self.tdreport, self.tdr_len)
    }
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}


fn get_tdx10_report(device_node: File, report_data: String)-> String {

    let report_data_bytes = report_data.as_bytes();
    let mut report_data_array: [u8; 64] = [0; 64];
    report_data_array[0..63].copy_from_slice(&report_data_bytes[0..63]);
    let td_report: [u8; 1024] = [0; 1024];

    let ptr1_ref  = & report_data_array;
    let ptr1_raw_ptr = ptr1_ref as *const u8;
    let s1 = ptr1_raw_ptr as u64;

    let ptr2_ref = &td_report;
    let ptr2_raw_ptr = ptr2_ref as *const u8;
    let s2 = ptr2_raw_ptr as u64;

    let request  = tdx_report_req {
        subtype: 0,
        reportdata: s1,
        rpd_len: REPORT_DATA_LEN,
        tdreport: s2,
        tdr_len: TDX_REPORT_LEN,
    };

    ioctl_readwrite!(get_quote_ioctl, b'T', 1, u64);

    let ptr3_ref = &request;
    let ptr3_raw_ptr = ptr3_ref as *const tdx_report_req;
    let s3 = ptr3_raw_ptr as u64;

    println!("Used Display: {}", request);

    let _res = match unsafe { get_quote_ioctl(device_node.as_raw_fd(), s3 as *mut u64) }{
        Err(e) => println!("Fail to get report: {:?}", e),
        Ok(_r) => (),
    };

    return "OK".to_string();

}

fn get_tdx_report(device: String, report_data: String) -> String {

let file = match File::options().read(true).write(true).open(&device) {
    Err(err) => panic!("couldn't open {}: {:?}", device, err),
    Ok(file) => file,
};


    let report = get_tdx10_report(file, report_data);

    return report;
}

fn main() {

    let tdx_report = get_tdx_report("/dev/tdx-guest".to_string(), "1234567812345678123456781234567812345678123456781234567812345678".to_string());
    println!("Back with result: {}", tdx_report);

}

