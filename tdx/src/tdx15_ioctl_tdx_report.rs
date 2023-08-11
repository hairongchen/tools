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
// https://github.com/intel-innersource/os.linux.cloud.mvp.kernel-dev/blob/css-tdx-mvp-kernel-6.2/include/uapi/linux/tdx-guest.h#L40
pub struct tdx_report_req {
    reportdata:         [u8; REPORT_DATA_LEN as usize],
    tdreport:           [u8; TDX_REPORT_LEN as usize],
}

fn get_tdx15_report(device_node: File, report_data: String)-> String {
    let mut request: [u8; (REPORT_DATA_LEN + TDX_REPORT_LEN) as usize] = [0; (REPORT_DATA_LEN + TDX_REPORT_LEN) as usize];
    request[0..((REPORT_DATA_LEN as usize) -1)].copy_from_slice(&report_data.as_bytes()[0..((REPORT_DATA_LEN as usize) -1)]);

    ioctl_readwrite!(get_report15_ioctl, b'T', 1, tdx_report_req);

    let _res = match unsafe { get_report15_ioctl(device_node.as_raw_fd(), ptr::addr_of!(request) as *mut tdx_report_req) }{
        Err(e) => panic!("Fail to get report: {:?}", e),
        Ok(_r) => println!("successfully get TDX report"),
    };

    format!("{:?}", &request[(REPORT_DATA_LEN as usize) .. ])
}

fn get_tdx_report(device: String, report_data: String) -> String {

    let file = match File::options().read(true).write(true).open(&device) {
        Err(err) => panic!("couldn't open {}: {:?}", device, err),
        Ok(file) => file,
    };

    get_tdx15_report(file, report_data)
}

fn main() {
    let tdx_report = get_tdx_report("/dev/tdx_guest".to_string(), "1234567812345678123456781234567812345678123456781234567812345678".to_string());
    println!("Back with result: {}", tdx_report);

}
