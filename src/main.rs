use std::ffi::c_void;
use std::fmt;
use std::mem::transmute;
use pretty_hex::*;
use std::mem;

type HModule = *const c_void;
type FarProc = *const c_void;
type Handle = *const c_void;

struct IpAddr([u8;4]);

impl fmt::Debug for IpAddr{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let [a,b,c,d] = self.0;
        write!(f,"{}.{}.{}.{}",a,b,c,d)
    }
} 


#[repr(C)]
#[derive(Debug)]
struct IpOptionInformation{
    ttl: u8,
    tos: u8,
    flags: u8,
    options_size: u8,
    options_data: u32,
}

extern "stdcall" {
    fn LoadLibrary(name: *const u8) -> HModule;
    fn GetProcAddress(module: HModule,name: *const u8) -> FarProc;
}

type IcmpSendEcho = extern "stdcall" fn(
    handle: Handle,
    ip_addr: IpAddr,
    request_data: *const u8,
    request_size: u16,
    request_options:  Option<&IpOptionInformation>,
    reply_buffer : *mut u8,
    reply_size: u32,
    timeout: u32, 
)->u32;

type IcmpCreateFile = extern "stdcall" fn() -> Handle;

#[repr(C)]
#[derive(Debug)]
struct IcmpEchoReply{
    address: IpAddr,
    status: u32,
    round_trip_time: u32,
    data_size: u16,
    reserved: u16,
    data: *const c_void,
    options: IpOptionInformation
}

fn main() {
    unsafe{

      
        let h = LoadLibrary("IPHLPAPI.dll\0".as_ptr());
        let IcmpCreateFile: IcmpCreateFile = transmute(GetProcAddress(h, b"IcmpCreateFile\0".as_ptr()));
        let IcmpSendEcho: IcmpSendEcho = transmute(GetProcAddress(h, b"IcmpSendEcho\0".as_ptr()));

        let handle = IcmpCreateFile();

        let data = "Shirshak Bajgain";

        let ip_opts = IpOptionInformation{
            ttl: 128,
            tos: 0,
            flags: 0,
            options_data: 0,
            options_size: 0
        };

        let mut reply_size = mem::size_of::<IcmpEchoReply>();

        let reply_buf_size = reply_size + 8 + data.len();
        let mut reply_buf = vec![0u8; reply_buf_size];

        let ret = IcmpSendEcho(
            handle,
            IpAddr([1,1,1,1]),   
            data.as_ptr(),
            data.len() as u16,
            Some(&ip_opts),
            reply_buf.as_mut_ptr(),
            reply_size  as u32,
            4000
        );

        if ret == 0 {
            panic!("IcmpSendEcho Failed! ret = {}",ret);
        }
        let reply: &IcmpEchoReply = mem::transmute(&reply_buf[0]);
        println!("{:#?}",*reply);

        let reply_data: *const u8 = mem::transmute(&reply_buf[reply_size + 8]);
        let reply_data = std::slice::from_raw_parts(reply_data, reply.data_size as usize);

        println!("{:?}", reply_data.hex_dump());
    };
}
