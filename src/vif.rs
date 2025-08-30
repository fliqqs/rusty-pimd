include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use libc::{
    IFF_LOOPBACK, IFF_MULTICAST, SIOCGIFCONF, SIOCGIFFLAGS, SIOCGIFINDEX, SIOCGIFNETMASK, ioctl,
    sockaddr_in,
};
use socket2::{Domain, Socket, Type};
use std::{ffi::CStr, io, mem, os::unix::io::AsRawFd, ptr};

#[derive(Debug)]
pub struct InterfaceInfo {
    pub name: String,
    pub ifindex: i32,
    pub addr: Option<std::net::Ipv4Addr>,
    pub netmask: Option<std::net::Ipv4Addr>,
    pub flags: i16,
}

pub fn list_interfaces() -> io::Result<Vec<InterfaceInfo>> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    let fd = sock.as_raw_fd();

    let buf_len = 32 * mem::size_of::<ifreq>();
    let mut buf = vec![0u8; buf_len];

    let mut ifc: ifconf = unsafe { mem::zeroed() };
    ifc.ifc_len = buf.len() as i32;
    ifc.ifc_ifcu.ifcu_buf = buf.as_mut_ptr() as *mut _;

    let ret = unsafe { ioctl(fd, SIOCGIFCONF as _, &mut ifc) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut results = Vec::new();
    let mut offset = 0;

    while offset < ifc.ifc_len as usize {
        let ifr_ptr = unsafe { buf.as_ptr().add(offset) as *const ifreq };
        let ifr = unsafe { &*ifr_ptr };

        // interface name
        let c_name = unsafe { CStr::from_ptr(ifr.ifr_ifrn.ifrn_name.as_ptr()) };
        let name = c_name.to_string_lossy().into_owned();

        // mutable copy so we can reuse for ioctls
        let mut ifr2: ifreq = unsafe { mem::zeroed() };
        unsafe {
            ptr::copy_nonoverlapping(ifr, &mut ifr2, 1);
        }

        // ifindex
        let idx = unsafe {
            if ioctl(fd, SIOCGIFINDEX as _, &mut ifr2) == 0 {
                ifr2.ifr_ifru.ifru_ivalue
            } else {
                -1
            }
        };

        // flags
        let flags = unsafe {
            if ioctl(fd, SIOCGIFFLAGS as _, &mut ifr2) == 0 {
                ifr2.ifr_ifru.ifru_flags
            } else {
                0
            }
        };

        // address (IPv4 only)
        let addr = unsafe {
            if ifr.ifr_ifru.ifru_addr.sa_family as i32 == libc::AF_INET {
                Some(std::net::Ipv4Addr::from(
                    (*(&ifr.ifr_ifru.ifru_addr as *const _ as *const sockaddr_in))
                        .sin_addr
                        .s_addr
                        .to_ne_bytes(),
                ))
            } else {
                None
            }
        };

        // netmask
        let mut ifr3: ifreq = unsafe { mem::zeroed() };
        unsafe {
            ptr::copy_nonoverlapping(ifr, &mut ifr3, 1);
        }
        let netmask = unsafe {
            if ioctl(fd, SIOCGIFNETMASK as _, &mut ifr3) == 0 {
                Some(std::net::Ipv4Addr::from(
                    (*(&ifr3.ifr_ifru.ifru_netmask as *const _ as *const sockaddr_in))
                        .sin_addr
                        .s_addr
                        .to_ne_bytes(),
                ))
            } else {
                None
            }
        };

        // ignore loopback / non-multicast
        if (flags & IFF_MULTICAST as i16) != 0 && (flags & IFF_LOOPBACK as i16) == 0 {
            results.push(InterfaceInfo {
                name,
                ifindex: idx,
                addr,
                netmask,
                flags,
            });
        }

        offset += mem::size_of::<ifreq>();
    }

    println!("Found interfaces: {:?}", results);
    Ok(results)
}
