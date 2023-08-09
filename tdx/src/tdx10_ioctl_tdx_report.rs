#[allow(unused_imports)]
use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::ptr;
use std::result::Result::Ok;

#[macro_use] extern crate nix;

const REPORT_DATA_LEN: u32 = 64;
const TDX_REPORT_LEN: u32 = 1024;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
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

fn get_tdx10_report(device_node: File, report_data: String)-> String {
    let sub_type: u8 = 0;
    let report_data_bytes = report_data.as_bytes();
    let mut report_data_array: [u8; 64] = [0; 64];
    report_data_array[0..63].copy_from_slice(&report_data_bytes[0..63]);
    let td_report: [u8; 1024] = [0; 1024];

    let request  = tdx_report_req {
        subtype: sub_type,
        reportdata: ptr::addr_of!(report_data_array) as u64,
        rpd_len: REPORT_DATA_LEN,
        tdreport: ptr::addr_of!(td_report) as u64,
        tdr_len: TDX_REPORT_LEN,
    };

    ioctl_readwrite!(get_quote_ioctl, b'T', 1, u64);

    let _res = match unsafe { get_quote_ioctl(device_node.as_raw_fd(), ptr::addr_of!(request) as *mut u64) }{
        Err(e) => panic!("Fail to get report: {:?}", e),
        Ok(_r) => println!("successfully get TDX report"),
    };

    return format!("{:?}", &td_report);
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

