#[allow(unused_imports)]
use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::ptr;
use std::result::Result::Ok;
use std::mem::size_of_val;
use std::mem;

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

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/qgs_msg_lib/inc/qgs_msg_lib.h#L73C16-L73C34
#[derive(Debug)]
pub struct qgs_msg_header{
    major_version:      u16,
    minor_version:      u16,
    msg_type:           u32,
    size:               u32,    // size of the whole message, include this header, in byte
    error_code:         u32,    // used in response only
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/QuoteGeneration/quote_wrapper/qgs_msg_lib/inc/qgs_msg_lib.h#L81C15-L81C15
pub struct qgs_msg_get_quote_req{
    header:                 qgs_msg_header,                 // header.type = GET_QUOTE_REQ
    report_size:            u32,                            // cannot be 0
    id_list_size:           u32,                            // length of id_list, in byte, can be 0
    report_id_list:         [u8;TDX_REPORT_LEN as usize],   // report followed by id list
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
// https://github.com/intel-innersource/os.linux.cloud.mvp.kernel-dev/blob/mvp-tdx-5.19.17/arch/x86/include/uapi/asm/tdx.h#L86
pub struct tdx_quote_hdr {
    version:                u64,                            // Quote version, filled by TD
    status:                 u64,                            // Status code of Quote request, filled by VMM
    in_len:                 u32,                            // Length of TDREPORT, filled by TD
    out_len:                u32,                            // Length of Quote, filled by VMM
    data_len_be_bytes:      [u8; 4],                        // big-endian 4 bytes indicate the size of data following
    data:                   [u8;TDX_QUOTE_LEN as usize],    // Actual Quote data or TDREPORT on input
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
    header:             qgs_msg_header,         // header.type = GET_QUOTE_RESP
    selected_id_size:   u32,                    // can be 0 in case only one id is sent in request
    quote_size:         u32,                    // length of quote_data, in byte
    id_quote:           [u8;TDX_QUOTE_LEN],     // selected id followed by quote
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
    println!("get report size {}", td_report.len());

    format!("{:?}", &td_report)
}

fn get_tdx_report(device: String, report_data: String) -> String {

    let file = match File::options().read(true).write(true).open(&device) {
        Err(err) => panic!("couldn't open {}: {:?}", device, err),
        Ok(file) => file,
    };

    get_tdx10_report(file, report_data)
}

fn generate_qgs_quote_msg(report: [u8; TDX_REPORT_LEN as usize]) -> qgs_msg_get_quote_req{

    let qgs_header = qgs_msg_header{
        major_version:      1,
        minor_version:      0,
        msg_type:           0,
        size:               16+8+TDX_REPORT_LEN,   // header + report_size and id_list_size + TDX_REPORT_LEN
        error_code:         0,
    };

    let mut qgs_request = qgs_msg_get_quote_req {

        header:                 qgs_header,
        report_size:            TDX_REPORT_LEN,
        id_list_size:           0,
        report_id_list:         [0;TDX_REPORT_LEN as usize],
    };

    let td_report = report;
    println!("td_report size {}", td_report.len());

    qgs_request.report_id_list[0..((TDX_REPORT_LEN as usize) -1)].copy_from_slice(&td_report[0..((TDX_REPORT_LEN as usize) -1)]);

    return qgs_request;
}

fn get_tdx10_quote(device_node: File, report_data: String)-> String {

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
    println!("get report size now {}", td_report.len());


    let qgs_msg = generate_qgs_quote_msg(td_report);
    let mut quote_header = tdx_quote_hdr{
        version:    1,
        status:     0,
        in_len:     (size_of_val(&qgs_msg)+4) as u32,
        out_len:    0,
        data_len_be_bytes: (1048 as u32).to_be_bytes(),
        data:       [0;TDX_QUOTE_LEN as usize],
    };

    let qgs_msg_bytes = unsafe {
        let ptr = &qgs_msg as *const qgs_msg_get_quote_req as *const u8;
        std::slice::from_raw_parts(ptr, mem::size_of::<qgs_msg_get_quote_req>())
    };
    quote_header.data[0..(16+8+TDX_REPORT_LEN-1) as usize].copy_from_slice(&qgs_msg_bytes[0..((16+8+TDX_REPORT_LEN-1) as usize)]);

    let request = tdx_quote_req{
        buf:    ptr::addr_of!(quote_header) as u64,
        len:    TDX_QUOTE_LEN as u64,
    };

    ioctl_read!(get_quote10_ioctl, b'T', 2, u64);

    // error code can be seen from qgsd and can be checked from
    // https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/e7604e02331b3377f3766ed3653250e03af72d45/QuoteGeneration/quote_wrapper/tdx_quote/inc/td_ql_wrapper.h#L46
    let _res = match unsafe { get_quote10_ioctl(device_node.as_raw_fd(), ptr::addr_of!(request) as *mut u64) }{
        Err(e) => panic!("Fail to get quote: {:?}", e),
        Ok(_r) => println!("successfully get TDX quote"),
    };

    let out_len = quote_header.out_len;
    let qgs_msg_resp_size = unsafe { std::mem::transmute::<[u8; 4], u32>(quote_header.data_len_be_bytes) }.to_be();

    let qgs_msg_resp = unsafe {
        let raw_ptr = ptr::addr_of!(quote_header.data) as *mut qgs_msg_get_quote_resp;
        raw_ptr.as_mut().unwrap() as &mut qgs_msg_get_quote_resp
    };

    let major_version = qgs_msg_resp.header.major_version;
    let minor_version = qgs_msg_resp.header.minor_version;
    let msg_type = qgs_msg_resp.header.msg_type;
    let error_code = qgs_msg_resp.header.error_code;

    println!("qgs_msg_resp.header {:#?}", qgs_msg_resp.header);
    println!("qgs msg response size from tdx_quote_hdr: {}", qgs_msg_resp_size);
    println!("qgs_msg_get_quote_resp.quote_size: {}", qgs_msg_resp.quote_size);

    if out_len - qgs_msg_resp_size != 4 {
        panic!("TDX get quote: wrong quote size!");
    }

    if major_version != 1 || minor_version != 0 || msg_type != 1 || error_code != 0 {
       panic!("TDX get quote: quote response error!");
    }

    format!("{:?}", &qgs_msg_resp.id_quote[0..(qgs_msg_resp.quote_size as usize)])

}

fn main() {
    let tdx_report = get_tdx_report("/dev/tdx-guest".to_string(), "1234567812345678123456781234567812345678123456781234567812345678".to_string());
    println!("Back with report string of size: {}", tdx_report.len());

    let file = match File::options().read(true).write(true).open("/dev/tdx-guest") {
        Err(err) => panic!("couldn't open {}: {:?}", "/dev/tdx-guest", err),
        Ok(file) => file,
    };

    let tdx_quote = get_tdx10_quote(file, "1234567812345678123456781234567812345678123456781234567812345678".to_string());
    //println!("Back with quote string of: {}", tdx_quote);
    println!("Back with quote string of size: {}", tdx_quote.len());

}
