#[allow(unused_imports)]
use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::ptr;
use std::result::Result::Ok;
use std::mem::size_of_val;

#[macro_use] extern crate nix;

const REPORT_DATA_LEN: u32 = 64;
const TDX_REPORT_LEN: u32 = 1024;
const TDX_QUOTE_LEN: usize = 4 * 4096;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel-innersource/os.linux.cloud.mvp.kernel-dev/blob/mvp-tdx-5.19.17/arch/x86/include/uapi/asm/tdx.h#L37
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
    let mut report_data_array: [u8; REPORT_DATA_LEN as usize] = [0; REPORT_DATA_LEN as usize];
    report_data_array[0..((REPORT_DATA_LEN as usize) -1)].copy_from_slice(&report_data_bytes[0..((REPORT_DATA_LEN as usize) -1)]);
    let td_report: [u8; TDX_REPORT_LEN as usize] = [0; TDX_REPORT_LEN as usize];

    let request  = tdx_report_req {
        subtype: sub_type,
        reportdata: ptr::addr_of!(report_data_array) as u64,
        rpd_len: REPORT_DATA_LEN,
        tdreport: ptr::addr_of!(td_report) as u64,
        tdr_len: TDX_REPORT_LEN,
    };

    ioctl_readwrite!(get_report10_ioctl, b'T', 1, u64);

    let _res = match unsafe { get_report10_ioctl(device_node.as_raw_fd(), ptr::addr_of!(request) as *mut u64) }{
        Err(e) => panic!("Fail to get report: {:?}", e),
        Ok(_r) => println!("successfully get TDX report"),
    };

    format!("{:?}", &td_report)
}

fn get_tdx_report(device: String, report_data: String) -> String {

    let file = match File::options().read(true).write(true).open(&device) {
        Err(err) => panic!("couldn't open {}: {:?}", device, err),
        Ok(file) => file,
    };

    get_tdx10_report(file, report_data)
}


#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/qgs_msg_lib/inc/qgs_msg_lib.h#L73C16-L73C34
pub struct qgs_msg_header{
    major_version:      u16,
    minor_version:      u16,
    r#type:               u32,
    size:               u32,    // size of the whole message, include this header, in byte
    error_code:         u32,    // used in response only
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/qgs_msg_lib/inc/qgs_msg_lib.h#L81C15-L81C15
pub struct qgs_msg_get_quote_req{
    header:                 qgs_msg_header,         // header.type = GET_QUOTE_REQ
    report_size:            u32,                    // cannot be 0
    id_list_size:           u32,                    // length of id_list, in byte, can be 0
    report_id_list:         [u8;TDX_REPORT_LEN as usize],    // report followed by id list
}

fn generate_qgs_quote_msg(report: String) -> qgs_msg_get_quote_req{

    let qgs_header = qgs_msg_header{
        major_version:      1,
        minor_version:      0,
        r#type:               0,
        size:               16+8+TDX_REPORT_LEN,   // header + report_size and id_list_size + TDX_REPORT_LEN
        error_code:         0,
    };

    let mut qgs_request = qgs_msg_get_quote_req {

        header:                 qgs_header,
        report_size:            TDX_REPORT_LEN,
        id_list_size:           0,
        report_id_list:         [0;TDX_REPORT_LEN as usize],
    };

    let td_report = report.as_bytes();
    qgs_request.report_id_list[0..((REPORT_DATA_LEN as usize) -1)].copy_from_slice(&td_report[0..((REPORT_DATA_LEN as usize) -1)]);

    return qgs_request;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel-innersource/os.linux.cloud.mvp.kernel-dev/blob/mvp-tdx-5.19.17/arch/x86/include/uapi/asm/tdx.h#L86
pub struct tdx_quote_hdr {
    version:    u64,            // Quote version, filled by TD
    status:     u64,            // Status code of Quote request, filled by VMM
    in_len:     u32,            // Length of TDREPORT, filled by TD
    out_len:    u32,            // Length of Quote, filled by VMM
    data_len:   u32,
    data:       u64,            // Actual Quote data or TDREPORT on input
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
// https://github.com/intel-innersource/os.linux.cloud.mvp.kernel-dev/blob/mvp-tdx-5.19.17/arch/x86/include/uapi/asm/tdx.h#L106
struct tdx_quote_req {
        buf:    u64,
        len:    u64,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/qgs_msg_lib/inc/qgs_msg_lib.h#L88C9-L93C2
pub struct qgs_msg_get_quote_resp {
    header:    qgs_msg_header,      // header.type = GET_QUOTE_RESP
    selected_id_size: u32,          // can be 0 in case only one id is sent in request
    quote_size: u32,                // length of quote_data, in byte
    id_quote: [u8;TDX_QUOTE_LEN],   // selected id followed by quote
}

fn get_tdx10_quote(device_node: File, report: String)-> String {

    let qgs_msg = generate_qgs_quote_msg(report);

    let quote_header = tdx_quote_hdr{
        version:    1,
        status:     0,
        in_len:     (size_of_val(&qgs_msg)+4) as u32,
        out_len:    0,
        data_len:   size_of_val(&qgs_msg),
        data:       ptr::addr_of!(qgs_msg) as u64,
    };

    let request = tdx_quote_req{
        buf:    ptr::addr_of!(quote_header) as u64,
        len:    TDX_QUOTE_LEN as u64,
    };

    ioctl_read!(get_quote10_ioctl, b'T', 2, u64);

    //request.len = 0;

    println!("1.0 get quote {:#0x}",request_code_read!(b'T', 0x02, 8) as u32);
    println!("1.0 get report {:#0x}",request_code_readwrite!(b'T', 0x01, 8) as u32);
    println!("quote_header.in_len = {}", quote_header.in_len);
    println!("request.len = {}", request.len);
    println!("request address: {:p}", &request);
    println!("header address: {:p}", &quote_header);
    //let qgs_msg_data = quote_header.data as tdx_quote_hdr;

    let req = unsafe {
        let raw_ptr = ptr::addr_of!(request) as *mut tdx_quote_req;
        raw_ptr.as_mut().unwrap() as &mut tdx_quote_req
    };
    //let req = &mut * { ptr::addr_of!(request).cast_mut() as *mut tdx_quote_req};
    println!("req raw ptr = {:#0x?}", req);
    println!("request raw ptr = {:#0x?}", &request);
    println!("req.len = {:}", req.len);

    let _res = match unsafe { get_quote10_ioctl(device_node.as_raw_fd(), ptr::addr_of!(request) as *mut u64) }{
        Err(e) => panic!("Fail to get quote: {:?}", e),
        Ok(_r) => println!("successfully get TDX quote"),
    };

    /*
    let major_version = qgs_msg.header.major_version;
    let minor_version = qgs_msg.header.minor_version;
    let r#type = qgs_msg.header.r#type;
    let error_code = qgs_msg.header.error_code;
    */

    let quote_size = qgs_msg.id_list_size;

    format!("{:?}", quote_size)

}

fn main() {
    let tdx_report = get_tdx_report("/dev/tdx-guest".to_string(), "1234567812345678123456781234567812345678123456781234567812345678".to_string());
    //println!("Back with report: {}", tdx_report);

    let file = match File::options().read(true).write(true).open("/dev/tdx-guest") {
        Err(err) => panic!("couldn't open {}: {:?}", "/dev/tdx-guest", err),
        Ok(file) => file,
    };

    let tdx_quote = get_tdx10_quote(file, tdx_report);
    println!("Back with quote: {}", tdx_quote);
}
