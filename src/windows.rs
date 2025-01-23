extern crate winapi;


use std::ffi::{CStr, OsString};
use std::os::windows::ffi::OsStringExt;
use std::{ptr, slice};
use std::net::Ipv4Addr;
use ipnet::Ipv4Net;
use winapi::um::iphlpapi::{GetAdaptersAddresses};
use winapi::shared::winerror::{ NO_ERROR};
use winapi::shared::minwindef::{ULONG};
use winapi::um::iptypes::{GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_MULTICAST, IP_ADAPTER_ADDRESSES, PIP_ADAPTER_GATEWAY_ADDRESS_LH};
use winapi::shared::ws2def::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKET_ADDRESS};
use winapi::um::ws2tcpip::{inet_ntop};
use winapi::ctypes::c_void;
use winapi::shared::in6addr::{in6_addr};
use winapi::shared::ws2ipdef::SOCKADDR_IN6;

/// # Exmaple
/// ```
///
/// let adapter_info = get_all_adapter_info();
/// let iter = all_adapter.iter();
///             for adapter in iter {
///                 println!("{:?}", adapter);
///             }
///
/// ```
#[derive(Debug)]
pub struct AdapterInfo {
    pub(crate) adapter_name: String,
    pub(crate) friendly_name: String,
    pub(crate) description: String,
    pub(crate) state: u32,
    pub(crate) in_addr: Vec<String>,
    pub(crate) in6_addr: Vec<String>,
    pub(crate) in_gw_addr: Vec<String>,
    pub(crate) in6_gw_addr: Vec<String>,
    pub(crate) in_dns_addr: Vec<String>,
    pub(crate) in6_dns_addr: Vec<String>,
    pub(crate) ip_mask: String,
}

/// # Exmaple
/// ```
///
/// let adapter_info = get_all_adapter_info();
/// let iter = all_adapter.iter();
///             for adapter in iter {
///                 println!("{:?}", adapter);
///             }
///
/// ```
pub unsafe fn get_all_adapter_info() -> Vec<AdapterInfo> {

    let mut adapter_info_list : Vec<AdapterInfo> = Vec::new();

    let mut bytes_out : ULONG = 0;
    let flag : ULONG = GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS | GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST;
    GetAdaptersAddresses(
        AF_UNSPEC as u32,
        flag,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut bytes_out);

    let mut buffer = vec![0u8; bytes_out as usize];
    let ptr_address = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES;
    let ret = GetAdaptersAddresses(
        AF_UNSPEC as u32,
        flag,
        ptr::null_mut(),
        ptr_address,
        &mut bytes_out,
    );
    if ret != NO_ERROR {
        panic!("Two GetAdaptersAddresses return code: {}", ret);
    }
    let mut adapter: *mut IP_ADAPTER_ADDRESSES = ptr_address;
    while !adapter.is_null() {
        let name : *const i8 = (*adapter).AdapterName;
        let mut adapter_info = AdapterInfo{
            adapter_name: CStr::from_ptr(name).to_string_lossy().into_owned(),
            friendly_name: pwchar_to_string((*adapter).FriendlyName),
            description: pwchar_to_string((*adapter).Description),
            state: (*adapter).OperStatus,
            in_addr: vec![],
            in6_addr: vec![],
            in_gw_addr: vec![],
            in6_gw_addr: vec![],
            in_dns_addr: vec![],
            in6_dns_addr: vec![],
            ip_mask: "".to_string(),
        };

        //获取IP地址
        let mut ip_addr = (*adapter).FirstUnicastAddress;
        while !ip_addr.is_null() {
            let addr : SOCKET_ADDRESS = (*ip_addr).Address;
            get_ip_addr(&addr, &mut adapter_info.in_addr, &mut adapter_info.in6_addr);
            ip_addr = (*ip_addr).Next;
        }

        //获取网关地址
        let mut gw_addr: PIP_ADAPTER_GATEWAY_ADDRESS_LH = (*adapter).FirstGatewayAddress;
        while !gw_addr.is_null() {
            let addr : SOCKET_ADDRESS = (*gw_addr).Address;
            get_ip_addr(&addr, &mut adapter_info.in_gw_addr, &mut adapter_info.in6_gw_addr);
            gw_addr = (*gw_addr).Next
        }
        //获取DNS服务器
        let mut dns_addr = (*adapter).FirstDnsServerAddress;
        while !dns_addr.is_null() {
            let addr : SOCKET_ADDRESS = (*dns_addr).Address;
            get_ip_addr(&addr, &mut adapter_info.in_dns_addr, &mut adapter_info.in6_dns_addr);
            dns_addr = (*dns_addr).Next
        }

        //获取ipv4的前缀长度
        let mut prefixs = (*adapter).FirstPrefix;
        let mut addr_list = Vec::<String>::new();
        while !prefixs.is_null() {
            let addr : SOCKET_ADDRESS = (*prefixs).Address;
            if (*addr.lpSockaddr).sa_family == AF_INET as u16 && (*adapter).OperStatus == 1 {
                get_ipv4_addr(&addr, &mut addr_list);
            }
            prefixs = (*prefixs).Next;
        }
        if addr_list.len() > 2 {
            adapter_info.ip_mask = calculate_subnet_info(&addr_list[0], &addr_list[2]).unwrap();
        }

        adapter_info_list.push(adapter_info);
        adapter = (*adapter).Next;
    }
    adapter_info_list
}

unsafe fn get_ipv4_addr(addr : &SOCKET_ADDRESS, in_addr : &mut Vec<String>) {
    if (*addr.lpSockaddr).sa_family == AF_INET as u16 {
        let sockaddr_in = *(addr.lpSockaddr as *const _ as *const SOCKADDR_IN);
        let ip_addr_ptr = &sockaddr_in.sin_addr as *const _ as *const c_void;
        let mut ip_str = [0u8;48];
        inet_ntop(AF_INET,ip_addr_ptr, ip_str.as_mut_ptr() as *mut i8, ip_str.len() as usize);
        (*in_addr).push(CStr::from_ptr(ip_str.as_ptr() as *const i8).to_str().unwrap().to_owned());
    }
}

unsafe fn get_ip_addr(addr : &SOCKET_ADDRESS, in_addr: &mut Vec<String>, in6_add: &mut Vec<String>){
    if (*addr.lpSockaddr).sa_family == AF_INET as u16 {
        let sockaddr_in = *(addr.lpSockaddr as *const _ as *const SOCKADDR_IN);
        let ip_addr_ptr = &sockaddr_in.sin_addr as *const _ as *const c_void;
        let mut ip_str = [0u8;48];
        inet_ntop(AF_INET,ip_addr_ptr, ip_str.as_mut_ptr() as *mut i8, ip_str.len() as usize);
        (*in_addr).push(CStr::from_ptr(ip_str.as_ptr() as *const i8).to_str().unwrap().to_owned());
    }
    if (*addr.lpSockaddr).sa_family == AF_INET6 as u16 {
        let sockaddr_in6  = *(addr.lpSockaddr as *const _ as *const SOCKADDR_IN6);
        let ip_addr_ptr = &sockaddr_in6.sin6_addr as *const in6_addr as *const c_void; // 直接转换为 *const c_void
        let mut ip_str = [0u8;48];
        inet_ntop(AF_INET6,ip_addr_ptr, ip_str.as_mut_ptr() as *mut i8, ip_str.len() as usize);
        (*in6_add).push(CStr::from_ptr(ip_str.as_ptr() as *const i8).to_str().unwrap().to_owned());
    }

}

pub fn calculate_subnet_info(network: &String, broadcast: &String) -> Option<String>  {
    let network_addr = network.parse().unwrap();
    for  prefix_len in (0..=32).rev() {
        let net = Ipv4Net::new(network_addr,prefix_len).unwrap();
        let broadcast_addr : Ipv4Addr = broadcast.parse().unwrap();
        if net.broadcast() == broadcast_addr {
            return Some(net.netmask().to_string());
        }
    }
    None
}

unsafe fn pwchar_to_string(pwchar: *const u16) -> String {
    assert!(!pwchar.is_null());
    let len = (0..).take_while(|&i| *pwchar.offset(i) != 0).count();
    let os_string = OsString::from_wide(slice::from_raw_parts(pwchar, len));
    match os_string.into_string() {
        Ok(s) => s,
        Err(e) => panic!("pwchar_to_string failed: {:?}", e),
    }
}