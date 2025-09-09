include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use libc::{IPPROTO_IP, c_void, setsockopt};
use socket2::{Domain, Protocol, Socket, Type};
use std::env;
use std::io;
use std::os::fd::AsRawFd;

use crate::vif::setup_vifs;

pub struct Context {
    pub mroute_socket: Socket,
}

pub fn setup_context() -> io::Result<Context> {
    let mroute_socket = open_mroute_socket()?;
    let interfaces = crate::vif::list_interfaces()?;
    setup_vifs(interfaces, &mroute_socket);
    Ok(Context { mroute_socket })
}

pub fn cleanup_context(ctx: Context) {
    ctx.mroute_socket.shutdown(std::net::Shutdown::Both).ok();
}

fn open_mroute_socket() -> io::Result<Socket> {
    let proto = unsafe { Protocol::from(libc::IPPROTO_IGMP) };
    let sock_type = unsafe { Type::from(libc::SOCK_RAW) };
    let sock = Socket::new(Domain::IPV4, sock_type, Some(proto))?;
    let opval = 1;

    let ret = unsafe {
        setsockopt(
            sock.as_raw_fd(),
            IPPROTO_IP,
            (MRT_INIT as u32).try_into().unwrap(),
            &opval as *const _ as *const c_void,
            size_of_val(&opval).try_into().unwrap(),
        )
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(sock)
    }
}
